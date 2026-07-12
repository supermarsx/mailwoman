#![forbid(unsafe_code)]
//! Render worker protocol: newline-delimited JSON over stdio.
//!
//! The parent (`mw-server`) spawns `mw-render` per job, writes one
//! [`Job`] line to stdin, reads one [`Output`] line from stdout, and the
//! process exits. This is the SPEC §7.5 process boundary: hostile input
//! is parsed in a disposable child with no network and no secrets.

use serde::{Deserialize, Serialize};

pub const MAX_INPUT_BYTES: usize = 4 * 1024 * 1024; // parser resource limit (SPEC §7.2)

#[derive(Debug, Serialize, Deserialize)]
pub struct Job {
    pub html: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Output {
    pub html: String,
}

/// Process one job line: parse, enforce limits, sanitize.
pub fn process_line(line: &str) -> Result<String, String> {
    if line.len() > MAX_INPUT_BYTES {
        return Err("input exceeds size limit".to_string());
    }
    let job: Job = serde_json::from_str(line).map_err(|e| format!("bad job frame: {e}"))?;
    let output = Output {
        html: mw_sanitize::sanitize_email_html(&job.html),
    };
    serde_json::to_string(&output).map_err(|e| format!("encode failed: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_sanitizes() {
        let line = serde_json::to_string(&Job {
            html: "<p>ok</p><script>bad()</script>".into(),
        })
        .unwrap();
        let out = process_line(&line).unwrap();
        let parsed: Output = serde_json::from_str(&out).unwrap();
        assert!(parsed.html.contains("<p>ok</p>"));
        assert!(!parsed.html.contains("script"));
    }

    #[test]
    fn rejects_garbage_frames() {
        assert!(process_line("not json").is_err());
    }
}
