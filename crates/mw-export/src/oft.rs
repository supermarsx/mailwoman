//! Outlook `.oft` template export (plan §3 e5, SPEC §10.6). SCAFFOLD stub (e0).
//!
//! e5 reads/writes the OFT template (a CFB container, same `cfb` layer as `.msg`).
//! Until then this returns [`crate::ExportError::Unimplemented`].

use crate::{ExportError, Result};

/// Export one RFC 5322 message to `.oft` template bytes. Stub until e5.
pub fn to_oft(_raw: &[u8]) -> Result<Vec<u8>> {
    Err(ExportError::Unimplemented("oft (Outlook template)"))
}
