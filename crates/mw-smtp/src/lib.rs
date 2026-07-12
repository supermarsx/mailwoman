#![forbid(unsafe_code)]
//! `mw-smtp` — MVP submission client (plan §0/§3, SPEC §6.1): 465 implicit-TLS
//! and 587 STARTTLS, EHLO capability parse, SASL PLAIN/LOGIN/XOAUTH2,
//! MAIL FROM / RCPT TO / DATA with SIZE and 8BITMIME.
//!
//! Scope-limited for V1: no DSN / REQUIRETLS / CHUNKING (those are §6.1
//! "later"). It exists because the V1 "daily-drivable" exit criterion is
//! impossible without send.
//!
//! Scaffolder note (e0): [`Submitter`] is a compiling stub with a `todo!()`
//! body; the transport, SASL and envelope logic are filled in by e4.

/// A message ready for submission: envelope + already-serialized MIME bytes
/// (built by `mw-mime`).
#[derive(Debug, Clone)]
pub struct Outgoing<'a> {
    /// Envelope sender (MAIL FROM).
    pub mail_from: &'a str,
    /// Envelope recipients (RCPT TO).
    pub rcpt_to: &'a [String],
    /// Serialized RFC822 bytes (DATA).
    pub raw: &'a [u8],
}

/// Per-recipient acceptance outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecipientOutcome {
    /// RCPT TO accepted (2xx).
    Accepted,
    /// RCPT TO rejected, with the server reply.
    Rejected { code: u16, reason: String },
}

/// Result of a submission: overall acceptance plus per-recipient outcomes.
#[derive(Debug, Clone)]
pub struct SubmissionResult {
    /// Outcome per recipient, parallel to `Outgoing::rcpt_to`.
    pub recipients: Vec<RecipientOutcome>,
}

/// Errors from SMTP submission.
#[derive(Debug, thiserror::Error)]
pub enum SmtpError {
    #[error("smtp transport error: {0}")]
    Transport(String),
    #[error("smtp authentication failed: {0}")]
    Auth(String),
    #[error("smtp protocol error: {0}")]
    Protocol(String),
}

/// Submission client bound to one account's submission server.
#[derive(Debug, Default)]
pub struct Submitter {
    // e4: tokio+rustls connection, negotiated EHLO capabilities, SASL creds.
}

impl Submitter {
    /// Submit one message; the engine calls `AccountBackend::append(Sent, …)`
    /// on success unless the server auto-files (plan §2.1).
    pub async fn submit(&self, _msg: Outgoing<'_>) -> Result<SubmissionResult, SmtpError> {
        todo!("e4: EHLO/STARTTLS/AUTH/MAIL/RCPT/DATA")
    }
}
