#![forbid(unsafe_code)]
//! `mw-mime` — RFC 5322 / MIME ↔ JMAP `Email` bridge (plan §0/§2.2, SPEC §6.3).
//!
//! Two directions:
//! - [`parse`] turns raw RFC822 bytes (via `mail-parser`) into a [`Parsed`] pair
//!   of a [`mw_jmap::Email`] (the exact shape `Email/get` must return) and a
//!   [`ParsedEnvelope`] carrying the threading headers (`Message-ID`,
//!   `In-Reply-To`, `References`) the engine's JWZ threading needs.
//! - [`build`] serializes a [`ComposeRequest`] (via `mail-builder`) into the raw
//!   bytes `mw-smtp` submits and the engine `APPEND`s to Sent/Drafts.
//!
//! Every function is pure over its inputs — no I/O, no global state — so the
//! engine can fd-pass hostile bytes into a `mw-render` jail worker and call
//! [`parse`] there. Parsing untrusted input never panics (see the `fuzz/`
//! target and the corpus smoke test); malformed input yields
//! [`MimeError::Parse`] or a best-effort partial [`Email`].

mod build;
mod parse;

pub use build::{Attachment, ComposeRequest, build};
pub use parse::{Parsed, ParsedEnvelope, PartBlob, decode_charset, parse, part_blob};

// Re-export the frozen JMAP types callers map to/from, so downstream crates
// (mw-smtp, mw-engine) need not depend on `mw-jmap` directly for these.
pub use mw_jmap::{Email, EmailAddress, EmailBodyPart, EmailBodyValue};

/// Errors from MIME parse/build.
#[derive(Debug, thiserror::Error)]
pub enum MimeError {
    /// The input could not be parsed as a MIME message at all.
    #[error("mime parse error: {0}")]
    Parse(String),
    /// A message could not be serialized from the compose request.
    #[error("mime build error: {0}")]
    Build(String),
}
