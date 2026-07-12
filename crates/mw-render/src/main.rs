#![forbid(unsafe_code)]
//! Disposable render worker binary. One job per invocation:
//! reads a single JSON line from stdin, writes a single JSON line to
//! stdout, exits. Errors go to stderr with a non-zero exit code.

use std::io::{BufRead, Write};

fn main() -> std::process::ExitCode {
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
