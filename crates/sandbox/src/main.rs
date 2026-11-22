use std::io::{self, Write};
use std::process::ExitCode;

fn main() -> ExitCode {
    let arguments: Vec<_> = std::env::args_os().skip(1).take(2).collect();
    let is_preflight = arguments.len() == 1 && arguments[0] == "preflight";
    let is_help = arguments.len() == 1 && (arguments[0] == "--help" || arguments[0] == "-h");
    let (message, status) = if is_preflight {
        (bunbox_sandbox::preflight_report(), 2)
    } else if is_help {
        (
            "Usage: bunbox preflight\nReports blockers; arbitrary-code execution is disabled.\n",
            0,
        )
    } else {
        let _ = writeln!(
            io::stderr().lock(),
            "Usage: bunbox preflight; execution is disabled."
        );
        return ExitCode::from(64);
    };
    if let Err(error) = io::stdout().lock().write_all(message.as_bytes()) {
        let _ = writeln!(
            io::stderr().lock(),
            "Cannot write preflight report: {error}"
        );
        return ExitCode::FAILURE;
    }
    ExitCode::from(status)
}
