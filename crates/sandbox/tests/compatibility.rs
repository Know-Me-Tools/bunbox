use std::io::{self, Cursor, Read};
use std::process::Command;

use bunbox_sandbox::{FrameError, FrameLimit, read_frame, write_frame};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn framing_round_trips_multiple_binary_messages() -> TestResult {
    let limit = FrameLimit::new(16)?;
    let mut wire = Vec::new();
    for payload in [b"hello".as_slice(), &[0, 255, 10, 13]] {
        write_frame(&mut wire, payload, limit)?;
    }
    let mut input = Cursor::new(wire);
    assert_eq!(read_frame(&mut input, limit)?, Some(b"hello".to_vec()));
    assert_eq!(read_frame(&mut input, limit)?, Some(vec![0, 255, 10, 13]));
    assert_eq!(read_frame(&mut input, limit)?, None);
    Ok(())
}

#[test]
fn oversized_header_is_rejected_before_reading_payload() -> TestResult {
    struct HeaderOnly(Cursor<[u8; 4]>);
    impl Read for HeaderOnly {
        fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
            assert!(self.0.position() < 4, "must not request oversized payload");
            self.0.read(output)
        }
    }
    let mut input = HeaderOnly(Cursor::new(u32::MAX.to_be_bytes()));
    assert!(matches!(
        read_frame(&mut input, FrameLimit::new(8)?),
        Err(FrameError::TooLarge { .. })
    ));
    Ok(())
}

#[test]
fn truncated_headers_and_payloads_are_not_clean_eof() -> TestResult {
    for bytes in [vec![0], vec![0, 0], vec![0, 0, 0], vec![0, 0, 0, 2, 7]] {
        assert!(matches!(
            read_frame(&mut Cursor::new(bytes), FrameLimit::new(8)?),
            Err(FrameError::Io(error)) if error.kind() == io::ErrorKind::UnexpectedEof
        ));
    }
    Ok(())
}

#[test]
fn invalid_limits_and_empty_frames_are_rejected() -> TestResult {
    assert!(FrameLimit::new(0).is_err());
    assert!(FrameLimit::new(usize::MAX).is_err());
    let limit = FrameLimit::new(8)?;
    assert!(matches!(
        read_frame(&mut Cursor::new([0; 4]), limit),
        Err(FrameError::Empty)
    ));
    let mut output = Vec::new();
    assert!(matches!(
        write_frame(&mut output, &[], limit),
        Err(FrameError::Empty)
    ));
    assert!(matches!(
        write_frame(&mut output, &[1; 9], limit),
        Err(FrameError::TooLarge { .. })
    ));
    assert!(output.is_empty());
    Ok(())
}

#[test]
fn fragmented_reads_and_interrupted_header_are_supported() -> TestResult {
    struct Fragmented {
        source: Cursor<Vec<u8>>,
        interrupted: bool,
    }
    impl Read for Fragmented {
        fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
            if !self.interrupted {
                self.interrupted = true;
                return Err(io::Error::from(io::ErrorKind::Interrupted));
            }
            let length = output.len().min(1);
            self.source.read(&mut output[..length])
        }
    }
    let mut input = Fragmented {
        source: Cursor::new(vec![0, 0, 0, 2, 3, 4]),
        interrupted: false,
    };
    assert_eq!(
        read_frame(&mut input, FrameLimit::new(2)?)?,
        Some(vec![3, 4])
    );
    Ok(())
}

#[test]
fn transport_errors_preserve_their_sources() -> TestResult {
    use std::error::Error;
    struct BrokenTransport;
    impl Read for BrokenTransport {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::from(io::ErrorKind::BrokenPipe))
        }
    }
    impl io::Write for BrokenTransport {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::Error::from(io::ErrorKind::BrokenPipe))
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let limit = FrameLimit::new(8)?;
    for result in [
        read_frame(&mut BrokenTransport, limit).map(|_| ()),
        write_frame(&mut BrokenTransport, b"x", limit),
    ] {
        let error = result.err().ok_or("expected a transport error")?;
        assert!(error.source().is_some());
        assert!(error.to_string().contains("frame transport error"));
    }
    for (error, expected) in [
        (FrameError::InvalidLimit, "frame limit"),
        (FrameError::Empty, "empty frame"),
        (
            FrameError::TooLarge {
                announced: 9,
                maximum: 8,
            },
            "exceeds limit",
        ),
    ] {
        assert!(error.source().is_none());
        assert!(error.to_string().contains(expected));
    }
    Ok(())
}

#[test]
fn help_documents_preflight_without_claiming_execution_support() -> TestResult {
    let output = Command::new(env!("CARGO_BIN_EXE_bunbox"))
        .arg("--help")
        .output()?;
    assert!(output.status.success());
    assert!(String::from_utf8(output.stdout)?.contains("execution is disabled"));
    assert!(output.stderr.is_empty());
    Ok(())
}

#[test]
fn executable_fails_closed_and_reports_missing_runtime_evidence() -> TestResult {
    let output = Command::new(env!("CARGO_BIN_EXE_bunbox"))
        .arg("preflight")
        .output()?;
    assert_eq!(output.status.code(), Some(2));
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("status: blocked"));
    assert!(stdout.contains("microsandbox 0.7.2"));
    assert!(stdout.contains("VM execution: not tested"));
    assert!(output.stderr.is_empty());
    Ok(())
}

#[test]
fn executable_never_treats_submitted_code_as_host_commands() -> TestResult {
    let output = Command::new(env!("CARGO_BIN_EXE_bunbox"))
        .args(["run", "return 42"])
        .output()?;
    assert_eq!(output.status.code(), Some(64));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8(output.stderr)?.contains("execution is disabled"));
    Ok(())
}
