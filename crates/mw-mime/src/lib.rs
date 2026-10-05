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
//!   [`build_with`] is the checked form: it also takes inline parts and a
//!   read-receipt request ([`ComposeExtras`]) and refuses any value that could
//!   add a header line.
//!
//! Read receipts (RFC 8098): [`build_mdn`] writes a report, [`parse_mdn`] reads
//! one, and [`Parsed::receipt`] carries the request headers of a received
//! message.
//!
//! The parse functions are pure over their inputs — no I/O, no global state —
//! so the engine can fd-pass hostile bytes into a `mw-render` jail worker and
//! call [`parse`] there. Parsing untrusted input never panics (see the `fuzz/`
//! target and the corpus smoke test); malformed input yields
//! [`MimeError::Parse`] or a best-effort partial [`Email`]. The build
//! functions do no I/O either, but read the clock (`Date`, boundaries) and,
//! through `mail-builder`, the host name when no `Message-ID` is supplied.

mod build;
mod check;
mod mdn;
mod parse;

pub use build::{
    Attachment, ComposeExtras, ComposeRequest, InlinePart, build, build_with, html_cid_references,
};
pub use check::{generate_content_id, validate_content_id, validate_mailbox};
pub use mdn::{
    MdnActionMode, MdnDisposition, MdnInput, MdnReport, MdnSendingMode, build_mdn, parse_mdn,
};
pub use parse::{
    Parsed, ParsedEnvelope, PartBlob, ReceiptHeaders, decode_charset, parse, part_blob,
};

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
