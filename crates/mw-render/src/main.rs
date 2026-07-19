#![forbid(unsafe_code)]
//! Disposable render worker binary. One job per invocation:
//! reads a single JSON line from stdin, writes a single JSON line to
//! stdout, exits. Errors go to stderr with a non-zero exit code.

use std::io::{BufRead, Write};

fn main() -> std::process::ExitCode {
    // SPEC §7.5: confine this disposable child BEFORE reading any hostile input. On
    // Linux this installs the seccomp + Landlock + namespace + rlimit kernel jail; if
    // a jail was expected (the DQ4 default on Linux) but could not be installed, fail
    // closed — never parse hostile input unconfined. On non-Linux it is a degraded
    // no-op and processing continues (documented; the WASM media jail still applies).
    match mw_render::enter_render_jail() {
        Ok(report) => {
            if let Some(reason) = &report.degraded {
                eprintln!("mw-render: jail degraded: {reason}");
            }
        }
        Err(e) => {
            eprintln!("mw-render: refusing to run without the required jail: {e}");
            return std::process::ExitCode::FAILURE;
        }
    }

    let stdin = std::io::stdin();
    let mut line = String::new();
    if let Err(e) = stdin.lock().read_line(&mut line) {
        eprintln!("mw-render: read error: {e}");
        return std::process::ExitCode::FAILURE;
    }
    match mw_render::process_line(line.trim_end()) {
        Ok(out) => {
            let mut stdout = std::io::stdout().lock();
            if writeln!(stdout, "{out}").is_err() {
                return std::process::ExitCode::FAILURE;
            }
            std::process::ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("mw-render: {e}");
            std::process::ExitCode::FAILURE
        }
    }
}
