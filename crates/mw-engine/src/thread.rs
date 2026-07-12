//! Engine-side threading (plan §1.7). Threading is canonical here — computed the
//! same way for IMAP, POP3, and Gmail — so a server's own `THREAD`/`X-GM-THRID`
//! is at most an accelerator, never the source of truth.
//!
//! V1 uses a JWZ-derived rule that is deterministic and incremental: a message's
//! **thread root** is the first entry of its `References` chain (the original
//! message), falling back to `In-Reply-To`, and finally to its own `Message-ID`.
//! Every message sharing a root joins the same thread, so an original and all of
//! its replies (which each carry the original at `References[0]`) collapse into
//! one thread without a second pass over the mailbox.

use mw_mime::ParsedEnvelope;

/// The thread-root Message-ID for a parsed message.
///
/// Angle brackets are already stripped by `mw-mime`. Returns `None` only when the
/// message has no usable identity at all (no `References`, no `In-Reply-To`, no
/// `Message-ID`), in which case the caller keys the thread off the stable id.
pub fn thread_root(envelope: &ParsedEnvelope) -> Option<String> {
    if let Some(first) = envelope.references.first() {
        return Some(first.clone());
    }
    if let Some(irt) = &envelope.in_reply_to {
        return Some(irt.clone());
    }
    envelope.message_id.clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(mid: &str, irt: Option<&str>, refs: &[&str]) -> ParsedEnvelope {
        ParsedEnvelope {
            message_id: Some(mid.to_string()),
            in_reply_to: irt.map(String::from),
            references: refs.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn original_roots_on_itself() {
        assert_eq!(
            thread_root(&env("root@x", None, &[])).as_deref(),
            Some("root@x")
        );
    }

    #[test]
    fn reply_roots_on_references_head() {
        let e = env("reply2@x", Some("reply1@x"), &["root@x", "reply1@x"]);
        assert_eq!(thread_root(&e).as_deref(), Some("root@x"));
    }

    #[test]
    fn reply_without_references_uses_in_reply_to() {
        let e = env("reply@x", Some("root@x"), &[]);
        assert_eq!(thread_root(&e).as_deref(), Some("root@x"));
    }
}
