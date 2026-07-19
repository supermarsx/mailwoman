#![forbid(unsafe_code)]
//! Render worker protocol: newline-delimited JSON over stdio.
//!
//! The parent (`mw-server`) spawns `mw-render` per job, writes one
//! [`Job`] line to stdin, reads one [`Output`] line from stdout, and the
//! process exits. This is the SPEC §7.5 process boundary: hostile input
//! is parsed in a disposable child with no network and no secrets.

use base64::Engine as _;
use serde::{Deserialize, Serialize};

/// Second-layer WASM media jail (SPEC §7.5, plan t16 S5): the hostile CFB/MS-OXMSG
/// parse and remote-image re-encode run inside a wasmtime sandbox, not native Rust.
/// `media_jail::reencode_image` is the entry the image proxy (t16-e6) consumes.
pub mod media_jail;

pub const MAX_INPUT_BYTES: usize = 4 * 1024 * 1024; // parser resource limit (SPEC §7.2)

/// One render job. **Untagged** so the existing HTML sanitize frame
/// (`{"html": …}`) is byte-identical (existing callers unchanged); the V7 CFB
/// import frame (`{"cfbBase64": …}`) selects the disposable `.oft`/`.msg` parse
/// (plan §3 e14/e5, SPEC §7.5 boundary).
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Job {
    /// Sanitize untrusted message HTML (the V2 path — unchanged).
    Html { html: String },
    /// Import an untrusted `.oft`/`.msg` CFB template: parse it in this child and
    /// return the sanitized body + subject. The hostile CFB parse runs inside the
    /// WASM media jail (`media_jail::parse_cfb`, §7.5) — never native Rust, and the
    /// parent never parses the compound file.
    Cfb {
        #[serde(rename = "cfbBase64")]
        cfb_base64: String,
    },
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Output {
    pub html: String,
    /// The imported template's subject (CFB jobs only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
}

/// Process one job line: parse, enforce limits, sanitize (HTML) or import (CFB).
pub fn process_line(line: &str) -> Result<String, String> {
    if line.len() > MAX_INPUT_BYTES {
        return Err("input exceeds size limit".to_string());
    }
    let job: Job = serde_json::from_str(line).map_err(|e| format!("bad job frame: {e}"))?;
    let output = match job {
        Job::Html { html } => Output {
            html: mw_sanitize::sanitize_email_html(&html),
            subject: None,
        },
        Job::Cfb { cfb_base64 } => {
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(cfb_base64.as_bytes())
                .map_err(|e| format!("bad base64 cfb: {e}"))?;
            if bytes.len() > MAX_INPUT_BYTES {
                return Err("cfb exceeds size limit".to_string());
            }
            // Hostile CFB parse — runs INSIDE the WASM media jail (SPEC §7.5), never
            // as native Rust in this child. Malformed input returns a clean error.
            let parsed =
                media_jail::parse_cfb(&bytes).map_err(|e| format!("oft import failed: {e}"))?;
            Output {
                html: mw_sanitize::sanitize_email_html(&parsed.body),
                subject: parsed.subject,
            }
        }
    };
    serde_json::to_string(&output).map_err(|e| format!("encode failed: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_sanitizes() {
        // The existing HTML frame is byte-identical (`{"html": …}`).
        let line = r#"{"html":"<p>ok</p><script>bad()</script>"}"#;
        let out = process_line(line).unwrap();
        let parsed: Output = serde_json::from_str(&out).unwrap();
        assert!(parsed.html.contains("<p>ok</p>"));
        assert!(!parsed.html.contains("script"));
        assert!(parsed.subject.is_none());
    }

    #[test]
    fn cfb_import_frame_parses_in_the_child() {
        // A real `.oft` round-trip through the disposable child: write a template,
        // then import it via the CFB job frame — the hostile parse is isolated here.
        let raw = b"Subject: Weekly status template\r\n\r\nFill me in.\r\n";
        let oft = mw_export::export_one(
            &mw_export::RawEmail::new(raw.to_vec()),
            mw_export::Format::Oft,
        )
        .expect("write .oft");
        let b64 = base64::engine::general_purpose::STANDARD.encode(&oft);
        let line = serde_json::to_string(&Job::Cfb { cfb_base64: b64 }).unwrap();
        let out = process_line(&line).unwrap();
        let parsed: Output = serde_json::from_str(&out).unwrap();
        assert_eq!(parsed.subject.as_deref(), Some("Weekly status template"));
    }

    #[test]
    fn rejects_garbage_frames() {
        assert!(process_line("not json").is_err());
    }

    #[test]
    fn rejects_bad_base64_cfb() {
        assert!(process_line(r#"{"cfbBase64":"!!!not-base64!!!"}"#).is_err());
    }
}
