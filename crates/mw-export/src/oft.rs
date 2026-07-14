//! Outlook `.oft` template export + import (plan §3 e5, §1.7, SPEC §10.6).
//!
//! An `.oft` template is the **same CFB / MS-OXMSG container** as a `.msg`
//! (§10.6) — Outlook distinguishes a template from a message by file extension
//! and message class, not by container shape. So export reuses the
//! [`crate::msg`] writer, and import reuses its reader. **Scope floor:** body +
//! attachments + headers (deep fidelity out, §28.8).
//!
//! # Hostile-parse boundary (plan §1.7, SPEC §7.5)
//! Template *import* ([`from_oft`]) parses an **untrusted** CFB container — the
//! same hostile-input concern documented on [`crate::msg::read_msg`], which this
//! delegates to. See that module's `SEAM(e14/e16)` note: until `mw-render`
//! grows a CFB job frame, callers importing an attacker-supplied `.oft` must
//! route the bytes through the render jail; only trusted/test fixtures are
//! parsed in-process here.

use crate::Result;
use crate::msg::{self, ParsedMsg};

/// The message class stored in the template. Outlook keys a template off the
/// `.oft` extension rather than a distinct class, so we keep the standard
/// `IPM.Note` here — a re-imported template is a normal message to fill in.
const TEMPLATE_CLASS: &str = "IPM.Note";

/// Export one RFC 5322 message to `.oft` template bytes.
///
/// Byte-identical container to `.msg` today (the two formats share MS-OXMSG);
/// the distinction is the `.oft` extension the web layer assigns on download.
pub fn to_oft(raw: &[u8]) -> Result<Vec<u8>> {
    msg::write_cfb_message(raw, TEMPLATE_CLASS)
}

/// Import an `.oft` template back into its floor properties (subject / body /
/// headers / attachments). **Hostile input** — see the module note.
pub fn from_oft(bytes: &[u8]) -> Result<ParsedMsg> {
    msg::read_msg(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &[u8] = b"From: templates@example.com\r\n\
Subject: Weekly status template\r\n\
Content-Type: text/plain\r\n\
\r\n\
Fill in your status here.\r\n";

    #[test]
    fn template_round_trips() {
        let bytes = to_oft(SAMPLE).unwrap();
        let parsed = from_oft(&bytes).unwrap();
        assert_eq!(parsed.subject.as_deref(), Some("Weekly status template"));
        assert!(
            parsed
                .body
                .as_deref()
                .unwrap()
                .contains("Fill in your status")
        );
        assert!(
            parsed
                .headers
                .unwrap()
                .contains("Subject: Weekly status template")
        );
    }

    #[test]
    fn is_a_cfb_container() {
        let bytes = to_oft(SAMPLE).unwrap();
        assert_eq!(
            &bytes[..8],
            &[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1]
        );
    }
}
