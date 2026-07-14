//! Word `.docx` export (plan §3 e5, SPEC §10.6). SCAFFOLD stub (e0).
//!
//! e5 renders the message (body + attachments + headers) to DOCX via `docx-rs`
//! (declared in the root manifest), through the existing sanitizer + print
//! pipeline. Until then this returns [`crate::ExportError::Unimplemented`].

use crate::{ExportError, Result};

/// Export one RFC 5322 message to `.docx` bytes. Stub until e5.
pub fn to_docx(_raw: &[u8]) -> Result<Vec<u8>> {
    Err(ExportError::Unimplemented("docx"))
}
