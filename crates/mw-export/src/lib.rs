#![forbid(unsafe_code)]
//! `mw-export` — server-side message export (plan §0.8, §3 e3, SPEC §10.5).
//!
//! Single + bulk export to **EML** (raw), **mbox** (`From ` separators with
//! `>From` quoting), **TXT**, and **Markdown** (from sanitized HTML), plus
//! conversation-as-one-document. Print-to-PDF is the browser print pipeline
//! (web-side, not here); MSG/OFT/DOCX are V7 (out of scope).
//!
//! ## Scaffolder note (e0)
//! e0 authors ONLY the frozen [`Format`] enum + the export entry points. e3
//! owns the whole crate — the mbox `From `/`>From` framing, the HTML→Markdown
//! path, and the bulk stream. Bodies are `todo!()`.

/// A raw RFC 5322 message (the export unit). e3 may enrich this at integration
/// (e.g. carrying the JMAP `receivedAt` for the mbox `From ` line).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RawEmail {
    pub raw: Vec<u8>,
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
}

pub type Result<T> = std::result::Result<T, ExportError>;

/// Export a single message to the given format.
#[allow(unused_variables)]
pub fn export_one(email: &RawEmail, format: Format) -> Result<Vec<u8>> {
    todo!("e3")
}

/// Export many messages to one document (mbox concatenation, or a joined
/// conversation for TXT/Markdown).
#[allow(unused_variables)]
pub fn export_many(emails: &[RawEmail], format: Format) -> Result<Vec<u8>> {
    todo!("e3")
}
