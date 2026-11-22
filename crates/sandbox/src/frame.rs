use std::fmt;
use std::io::{self, Read, Write};

/// Absolute per-frame ceiling, independent of caller-selected smaller limits.
const MAX_FRAME_BYTES: usize = 1024 * 1024;

/// Validated payload byte limit for a four-byte, big-endian length prefix.
#[derive(Clone, Copy, Debug)]
pub struct FrameLimit(usize);

impl FrameLimit {
    /// Set a nonzero maximum, no greater than the one-MiB protocol ceiling.
    pub fn new(bytes: usize) -> Result<Self, FrameError> {
        if bytes == 0 || bytes > MAX_FRAME_BYTES {
            return Err(FrameError::InvalidLimit);
        }
        Ok(Self(bytes))
    }

    fn validate(self, bytes: usize) -> Result<(), FrameError> {
        if bytes == 0 {
            return Err(FrameError::Empty);
        }
        if bytes > self.0 {
            return Err(FrameError::TooLarge {
                announced: bytes,
                maximum: self.0,
            });
        }
        Ok(())
    }
}

/// Framing failures. A decoder error invalidates the stream; do not resynchronize.
#[derive(Debug)]
pub enum FrameError {
    /// The configured maximum is zero or exceeds the hard protocol ceiling.
    InvalidLimit,
    /// Empty frames are not valid protocol messages.
    Empty,
    /// Reject an oversized payload before reading or allocating its contents.
    TooLarge {
        /// Announced payload size.
        announced: usize,
        /// Configured limit.
        maximum: usize,
    },
    /// The bounded payload allocation could not be reserved.
    Allocation(std::collections::TryReserveError),
    /// Transport error, including truncated headers or payloads.
    Io(io::Error),
}

impl fmt::Display for FrameError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLimit => write!(formatter, "frame limit must be 1..={MAX_FRAME_BYTES}"),
            Self::Empty => formatter.write_str("empty frame"),
            Self::TooLarge { announced, maximum } => {
                write!(formatter, "frame size {announced} exceeds limit {maximum}")
            }
            Self::Allocation(error) => write!(formatter, "frame allocation failed: {error}"),
            Self::Io(error) => write!(formatter, "frame transport error: {error}"),
        }
    }
}

impl std::error::Error for FrameError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Allocation(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for FrameError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

/// Read one bounded frame; `None` means EOF at a frame boundary only.
///
/// This synchronous primitive is for compatibility fixtures, not direct use on
/// Tokio executor threads. It provides no deadline and cannot bound buffers in
/// the supplied reader. The eventual runtime transport needs its own audit.
pub fn read_frame(
    reader: &mut impl Read,
    limit: FrameLimit,
) -> Result<Option<Vec<u8>>, FrameError> {
    let mut header = [0_u8; 4];
    loop {
        match reader.read(&mut header[..1]) {
            Ok(0) => return Ok(None),
            Ok(_) => break,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error.into()),
        }
    }
    reader.read_exact(&mut header[1..])?;
    let length = u32::from_be_bytes(header) as usize;
    limit.validate(length)?;
    let mut payload = Vec::new();
    payload
        .try_reserve_exact(length)
        .map_err(FrameError::Allocation)?;
    payload.resize(length, 0);
    reader.read_exact(&mut payload)?;
    Ok(Some(payload))
}

/// Write one frame after validating its size. Does not flush the caller's writer.
///
/// An I/O failure can leave a partial frame; close that stream rather than retry.
/// Message-count and total-byte budgets are a separate caller responsibility.
pub fn write_frame(
    writer: &mut impl Write,
    payload: &[u8],
    limit: FrameLimit,
) -> Result<(), FrameError> {
    limit.validate(payload.len())?;
    // Validation caps length at one MiB, so the wire conversion is lossless.
    writer.write_all(&(payload.len() as u32).to_be_bytes())?;
    writer.write_all(payload)?;
    Ok(())
}
