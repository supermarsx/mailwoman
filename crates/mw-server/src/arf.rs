//! ARF (Abuse Reporting Format, RFC 5965) report generation + relay (SPEC §7.3
//! sender-controls, plan §3 e7). The `/api/security/report` endpoint builds a
//! `multipart/report; report-type=feedback-report` message wrapping the reported
//! mail and relays it to the configured abuse address (`MW_ABUSE_ADDRESS`).
//!
//! ## Relay transport
//! When `MW_ABUSE_SPOOL` is set the built report is written there as an `.eml`
//! file for a relay agent (or the deployment MTA) to pick up — a real, testable
//! delivery mechanism with no coupling to the engine's account submitter. Direct
//! SMTP submission through the account `MailSubmitter` (plan §1.9) is the engine's
//! job and lands with the `SenderControl/set` report path (e6) — this module owns
//! only the honest, well-formed report artifact.

use std::path::{Path, PathBuf};

/// The kind of abuse being reported, mapped to an RFC 7960 / RFC 5965
/// `Feedback-Type` token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedbackKind {
    Phishing,
    Junk,
}

impl FeedbackKind {
    /// Parse the client-supplied `kind` token.
    pub fn from_token(s: &str) -> Option<Self> {
        match s {
            "phishing" | "fraud" => Some(Self::Phishing),
            "junk" | "spam" | "abuse" => Some(Self::Junk),
            _ => None,
        }
    }

    /// The `Feedback-Type` header value.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Phishing => "fraud",
            Self::Junk => "abuse",
        }
    }
}

/// Build a well-formed RFC 5965 ARF report: an outer `multipart/report;
/// report-type=feedback-report` with three parts — a human-readable explanation,
/// the machine-readable `message/feedback-report`, and the original message
/// (`message/rfc822`). Interpolated header values are stripped of CR/LF to prevent
/// header injection; the random boundary prevents body-part injection via `note`.
pub fn build_report(
    kind: FeedbackKind,
    reporter: &str,
    abuse_address: &str,
    reporting_domain: &str,
    original: &[u8],
    note: Option<&str>,
) -> Vec<u8> {
    let boundary = format!("mw-arf-{}", uuid::Uuid::new_v4().simple());
    let feedback_type = kind.as_str();
    let reporter = header_safe(reporter);
    let abuse_address = header_safe(abuse_address);
    let reporting_domain = header_safe(reporting_domain);
    let human = match note {
        Some(n) => body_safe(n),
        None => match kind {
            FeedbackKind::Phishing => "A phishing message, reported via Mailwoman.".to_string(),
            FeedbackKind::Junk => "A spam/abuse message, reported via Mailwoman.".to_string(),
        },
    };

    let head = format!(
        "From: {reporter}\r\n\
         To: {abuse_address}\r\n\
         Subject: Abuse report ({feedback_type})\r\n\
         MIME-Version: 1.0\r\n\
         Content-Type: multipart/report; report-type=feedback-report;\r\n\
         \tboundary=\"{boundary}\"\r\n\
         \r\n\
         --{boundary}\r\n\
         Content-Type: text/plain; charset=utf-8\r\n\
         \r\n\
         {human}\r\n\
         \r\n\
         --{boundary}\r\n\
         Content-Type: message/feedback-report\r\n\
         \r\n\
         Feedback-Type: {feedback_type}\r\n\
         User-Agent: Mailwoman/1.0\r\n\
         Version: 1\r\n\
         Original-Rcpt-To: {reporter}\r\n\
         Reported-Domain: {reporting_domain}\r\n\
         \r\n\
         --{boundary}\r\n\
         Content-Type: message/rfc822\r\n\
         \r\n"
    );

    let mut out = Vec::with_capacity(head.len() + original.len() + boundary.len() + 8);
    out.extend_from_slice(head.as_bytes());
    out.extend_from_slice(original);
    out.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
    out
}

/// Write a built report to the spool directory as a unique `.eml` file, returning
/// its path. Creates the directory if missing.
pub fn spool(dir: &Path, report: &[u8]) -> std::io::Result<PathBuf> {
    std::fs::create_dir_all(dir)?;
    let path = dir.join(format!("{}.eml", uuid::Uuid::new_v4().simple()));
    std::fs::write(&path, report)?;
    Ok(path)
}

/// Strip CR/LF (and other controls) from a value interpolated into a header.
fn header_safe(v: &str) -> String {
    v.chars().filter(|c| !c.is_control()).collect()
}

/// Sanitize free-text destined for the human-readable body part: drop CR/`--`
/// runs that could forge a MIME boundary and cap the length.
fn body_safe(v: &str) -> String {
    v.replace('\r', "")
        .replace("--", "-\u{2011}") // neutralise boundary-lookalikes
        .chars()
        .take(2000)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feedback_kind_tokens() {
        assert_eq!(
            FeedbackKind::from_token("phishing"),
            Some(FeedbackKind::Phishing)
        );
        assert_eq!(FeedbackKind::from_token("junk"), Some(FeedbackKind::Junk));
        assert_eq!(FeedbackKind::from_token("spam"), Some(FeedbackKind::Junk));
        assert_eq!(FeedbackKind::from_token("nonsense"), None);
        assert_eq!(FeedbackKind::Phishing.as_str(), "fraud");
        assert_eq!(FeedbackKind::Junk.as_str(), "abuse");
    }

    #[test]
    fn report_is_a_wellformed_feedback_report() {
        let original = b"From: bad@evil.example\r\nSubject: buy now\r\n\r\nspam body";
        let report = build_report(
            FeedbackKind::Junk,
            "victim@example.org",
            "abuse@example.org",
            "example.org",
            original,
            None,
        );
        let text = String::from_utf8(report).unwrap();
        // Outer multipart/report envelope.
        assert!(text.contains("Content-Type: multipart/report; report-type=feedback-report;"));
        assert!(text.contains("To: abuse@example.org"));
        // The three RFC 5965 required feedback fields.
        assert!(text.contains("Content-Type: message/feedback-report"));
        assert!(text.contains("Feedback-Type: abuse"));
        assert!(text.contains("User-Agent: Mailwoman/1.0"));
        assert!(text.contains("Version: 1"));
        // The original message is embedded as message/rfc822.
        assert!(text.contains("Content-Type: message/rfc822"));
        assert!(text.contains("Subject: buy now"));
        // Closing boundary.
        assert!(text.trim_end().ends_with("--"));
    }

    #[test]
    fn header_injection_is_neutralised() {
        let report = build_report(
            FeedbackKind::Phishing,
            "victim@example.org\r\nBcc: leak@evil.example",
            "abuse@example.org",
            "example.org",
            b"original",
            None,
        );
        let text = String::from_utf8(report).unwrap();
        // No injected header line: the CRLF was stripped, so "Bcc:" never starts a line.
        assert!(!text.contains("\r\nBcc:"));
        // The value is folded into the From value instead.
        assert!(text.contains("From: victim@example.orgBcc: leak@evil.example"));
    }

    #[test]
    fn spool_writes_the_report() {
        let dir = std::env::temp_dir().join(format!("mw-arf-spool-{}", std::process::id()));
        let report = build_report(
            FeedbackKind::Junk,
            "v@example.org",
            "abuse@example.org",
            "example.org",
            b"x",
            None,
        );
        let path = spool(&dir, &report).unwrap();
        assert!(path.exists());
        assert_eq!(std::fs::read(&path).unwrap(), report);
    }
}
