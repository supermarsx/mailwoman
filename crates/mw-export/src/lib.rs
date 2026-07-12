#![forbid(unsafe_code)]
//! `mw-export` — server-side message export (plan §0.8, §3 e3, SPEC §10.5).
//!
//! Single + bulk export to **EML** (raw RFC 5322 bytes), **mbox** (mboxrd:
//! `From ` separators with `>From` body quoting), **TXT** (headers + decoded
//! text body), and **Markdown** (sanitized-shape HTML → Markdown). Bulk export
//! streams into any [`std::io::Write`] sink so a large mailbox is never held in
//! memory at once, and a thread can be exported as one concatenated document.
//!
//! Print-to-PDF is the browser print pipeline (web-side, not here); MSG/OFT/DOCX
//! are V7 (out of scope).

use std::borrow::Borrow;
use std::io::Write;

mod html2md;
mod markdown;
mod mbox;
mod text;

pub use html2md::html_to_markdown;
pub use mbox::split as split_mbox;

/// A raw RFC 5322 message — the export unit.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RawEmail {
    pub raw: Vec<u8>,
}

impl RawEmail {
    #[must_use]
    pub fn new(raw: Vec<u8>) -> Self {
        Self { raw }
    }
}

impl From<Vec<u8>> for RawEmail {
    fn from(raw: Vec<u8>) -> Self {
        Self { raw }
    }
}

impl From<&[u8]> for RawEmail {
    fn from(raw: &[u8]) -> Self {
        Self { raw: raw.to_vec() }
    }
}

/// Target export format (plan §0.8).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Eml,
    Mbox,
    Txt,
    Markdown,
}

#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error("parse error: {0}")]
    Parse(String),
    #[error("render error: {0}")]
    Render(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, ExportError>;

/// The divider between messages in a joined TXT/Markdown conversation document.
const CONVERSATION_DIVIDER: &str = "\n\n---\n\n";

/// Export a single message to the given format.
///
/// For [`Format::Eml`] this is the raw bytes verbatim, so a re-parse of the
/// output equals a re-parse of the input.
pub fn export_one(email: &RawEmail, format: Format) -> Result<Vec<u8>> {
    match format {
        Format::Eml => Ok(email.raw.clone()),
        Format::Mbox => mbox::to_entry(&email.raw),
        Format::Txt => text::to_txt(&email.raw),
        Format::Markdown => markdown::to_markdown(&email.raw),
    }
}

/// Export many messages to one document held in memory.
///
/// - **mbox** → the natural multi-message container (concatenated entries).
/// - **EML** → the raw messages concatenated (use [`export_one`] per message
///   when discrete `.eml` files are wanted; the web layer zips those).
/// - **TXT / Markdown** → a single conversation document, messages joined by a
///   `---` divider.
///
/// Prefer [`export_stream`] for large sets — this is a convenience wrapper that
/// collects the stream into a `Vec`.
pub fn export_many(emails: &[RawEmail], format: Format) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    export_stream(emails, format, &mut buf)?;
    Ok(buf)
}

/// Stream a bulk export into `out`, writing each message as it is produced so
/// the whole corpus is never buffered at once.
///
/// Accepts anything iterable yielding items that borrow a [`RawEmail`] — e.g. a
/// `&[RawEmail]`, or a lazy iterator pulling messages from a store.
pub fn export_stream<W, I>(emails: I, format: Format, out: &mut W) -> Result<()>
where
    W: Write,
    I: IntoIterator,
    I::Item: Borrow<RawEmail>,
{
    let mut first = true;
    for email in emails {
        let email = email.borrow();
        match format {
            // Container formats frame each message themselves.
            Format::Mbox => out.write_all(&mbox::to_entry(&email.raw)?)?,
            Format::Eml => out.write_all(&email.raw)?,
            // Conversation documents get a divider between entries.
            Format::Txt | Format::Markdown => {
                if !first {
                    out.write_all(CONVERSATION_DIVIDER.as_bytes())?;
                }
                let rendered = if format == Format::Txt {
                    text::to_txt(&email.raw)?
                } else {
                    markdown::to_markdown(&email.raw)?
                };
                // Trim the per-message trailing newline so the divider controls spacing.
                out.write_all(trim_trailing_newline(&rendered))?;
            }
        }
        first = false;
    }
    Ok(())
}

fn trim_trailing_newline(bytes: &[u8]) -> &[u8] {
    let mut end = bytes.len();
    while end > 0 && (bytes[end - 1] == b'\n' || bytes[end - 1] == b'\r') {
        end -= 1;
    }
    &bytes[..end]
}
