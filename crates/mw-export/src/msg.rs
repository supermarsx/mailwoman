//! MS-OXMSG `.msg` export (plan §3 e5, SPEC §10.6). SCAFFOLD stub (e0).
//!
//! e5 writes the MS-OXMSG Compound-File-Binary container via `cfb` (declared in the
//! root manifest) with its own MS-OXMSG property layer on top. **Scope floor:**
//! faithful body + attachments + headers (deep write fidelity for embedded objects /
//! custom named properties is best-effort, §28.8). A CFB-parse fuzz target lands
//! alongside (§25). Until then this returns [`crate::ExportError::Unimplemented`].

use crate::{ExportError, Result};

/// Export one RFC 5322 message to `.msg` bytes. Stub until e5.
pub fn to_msg(_raw: &[u8]) -> Result<Vec<u8>> {
    Err(ExportError::Unimplemented("msg (MS-OXMSG)"))
}
