//! Phase-zero compatibility primitives; not a sandbox execution backend.
#![deny(missing_docs)]

mod frame;

pub use frame::{FrameError, FrameLimit, read_frame, write_frame};

/// Render the known blockers without probing, downloading, or starting a runtime.
///
/// This reports repository readiness, not dynamically detected host capabilities.
pub fn preflight_report() -> &'static str {
    concat!(
        "BunBox phase 0\n",
        "status: blocked\n",
        "Execution is disabled; there is no host-process fallback.\n",
        "microsandbox 0.7.2: candidate only; SDK event queue is unbounded.\n",
        "Streaming execution needs an independent host watchdog.\n",
        "rmcp 3.4.1: absent from checked registry; 3.4.0 remains a candidate.\n",
        "VM execution: not tested\n",
        "Network/resource enforcement: not tested\n",
        "Stop/destroy and crash recovery: not tested\n",
        "Guest image digest: not selected\n",
        "See docs/compatibility.md for source evidence and next gates.\n",
    )
}
