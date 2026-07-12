#![forbid(unsafe_code)]
//! `mw-mime` — RFC 5322 / MIME ↔ JMAP `Email` bridge (plan §0, SPEC §6.3).
//!
//! Parses untrusted RFC822 bytes (`mail-parser`) into `mw_jmap::Email` and
//! builds outgoing MIME (`mail-builder`) from drafts/submissions. All parsing
//! of untrusted bytes is designed to run inside the `mw-render` jail.
//!
//! Scaffolder note (e0): these are compiling stubs. The real mapping, charset
//! handling, torture-corpus tests and the `cargo-fuzz` target are added by e1.

use mw_jmap::Email;

/// Errors from MIME parse/build.
#[derive(Debug, thiserror::Error)]
pub enum MimeError {
    /// The input could not be parsed as a MIME message.
    #[error("mime parse error: {0}")]
    Parse(String),
    /// A message could not be serialized from the JMAP model.
    #[error("mime build error: {0}")]
    Build(String),
}

/// Parse raw RFC822 bytes into a JMAP `Email` (headers, addresses, body
/// structure, bodyValues, hasAttachment, size, preview).
///
/// Runs inside the render jail over untrusted bytes.
pub fn parse(_raw: &[u8]) -> Result<Email, MimeError> {
    todo!("e1: mail-parser -> mw_jmap::Email mapping")
}

/// Build the RFC822 bytes for an outgoing message (draft or submission) from a
/// JMAP `Email`.
pub fn build(_email: &Email) -> Result<Vec<u8>, MimeError> {
    todo!("e1: mw_jmap::Email -> mail-builder MIME")
}
