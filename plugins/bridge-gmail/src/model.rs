//! Transport-neutral value types + the frozen-ABI encoding helpers.
//!
//! These plain types mirror the `mailwoman:plugin` WIT records closely, but carry
//! NO `wit-bindgen` dependency, so the whole Gmail backend ([`crate::backend`]) is
//! host-compilable and unit-testable without a wasm toolchain. The wasm component
//! ([`crate::component`], gated to `target_family = "wasm"`) converts these ↔ the
//! generated WIT types at the boundary.
//!
//! ## Frozen-ABI passthrough (host adapter contract — see `mw-plugin/src/adapter.rs`)
//! The `mw-plugin` host adapter carries a plugin backend's `message-ref.raw` string
//! and `sync-cursor.opaque` bytes VERBATIM: it wraps whatever this guest emits into
//! the additive `mw_engine::MessageRef::Plugin { raw }` / `SyncCursor::Plugin { opaque }`
//! variants and hands the exact same bytes back on the next call (t7-fix-msgref). So
//! this guest emits its own **provider-native** encodings, no engine-JSON disguise:
//!
//! * **Cursor** — the Gmail `historyId` rides `opaque` as its raw UTF-8 bytes.
//! * **Message ref** — a Gmail message id (a 16-hex-digit string) plus its owning
//!   label (needed for a correct multi-label move/store) are packed into `raw` as
//!   `"<labelId>\u{1f}<gmailId>"`. The engine treats `raw` as opaque; only this bridge
//!   decodes it.

/// Field separator packed into the message-ref `raw` (US, 0x1F — never appears in a
/// Gmail label id or message id).
const SEP: char = '\u{1f}';

/// A server-authoritative flag (mirrors `mw_engine::Flag` / the WIT `flag`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Flag {
    Seen,
    Answered,
    Flagged,
    Deleted,
    Draft,
    Recent,
    Keyword(String),
}

/// A backend mailbox coordinate (mirrors the WIT `mailbox-ref`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MailboxRef {
    pub name: String,
    pub uidvalidity: u32,
}

/// A backend message reference (mirrors the WIT `message-ref`). `raw` is the
/// provider-native id the host adapter round-trips verbatim as
/// `MessageRef::Plugin { raw }` (see the passthrough note).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageRef {
    pub raw: String,
    pub mailbox: MailboxRef,
}

impl MessageRef {
    /// Build a ref for a Gmail message id owned by `label`, packing both into the
    /// opaque `raw` the host adapter passes through as `MessageRef::Plugin`.
    pub fn for_gmail(label: &str, gmail_id: &str) -> Self {
        Self {
            raw: format!("{label}{SEP}{gmail_id}"),
            mailbox: MailboxRef {
                name: label.to_string(),
                uidvalidity: 1,
            },
        }
    }

    /// Decode `(labelId, gmailId)` back out of a ref the host handed us. Tolerant of
    /// a bare id without the separator (⇒ whole string is the id, no label), and of an
    /// empty ref (⇒ not ours).
    pub fn decode_gmail(&self) -> Option<(String, String)> {
        if self.raw.is_empty() {
            return None;
        }
        match self.raw.split_once(SEP) {
            Some((label, id)) => Some((label.to_string(), id.to_string())),
            None => Some((String::new(), self.raw.clone())),
        }
    }
}

/// A mailbox as enumerated by the backend (mirrors the WIT `mailbox`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mailbox {
    pub mailbox_ref: MailboxRef,
    /// JMAP special-use role, lowercased ("inbox"|"archive"|…|"none").
    pub role: String,
    pub parent: Option<String>,
    pub total: u32,
    pub unread: u32,
}

/// Full raw bytes for one message (mirrors the WIT `raw-message`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawMessage {
    pub message_ref: MessageRef,
    pub raw: Vec<u8>,
    pub msg_flags: Vec<Flag>,
    pub internaldate: Option<String>,
}

/// An opaque, backend-chosen sync cursor (mirrors the WIT `sync-cursor`). `opaque`
/// carries the Gmail `historyId` bytes, which the host adapter round-trips verbatim as
/// `SyncCursor::Plugin { opaque }`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncCursor {
    pub opaque: Vec<u8>,
}

impl SyncCursor {
    /// Carry a Gmail `historyId` as the raw `opaque` bytes.
    pub fn from_history_id(history_id: &str) -> Self {
        Self {
            opaque: history_id.as_bytes().to_vec(),
        }
    }

    /// Extract the Gmail `historyId` if this cursor carries one. Returns `None` for an
    /// empty / non-UTF-8 cursor ⇒ the caller does a full sync.
    pub fn history_id(&self) -> Option<String> {
        if self.opaque.is_empty() {
            return None;
        }
        String::from_utf8(self.opaque.clone()).ok()
    }
}

/// The set of changes an incremental sync produced (mirrors the WIT `mailbox-delta`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MailboxDelta {
    pub added: Vec<MessageRef>,
    pub removed: Vec<MessageRef>,
    pub flag_changes: Vec<(MessageRef, Vec<Flag>)>,
    pub next_cursor: SyncCursor,
}

/// A change event (mirrors the WIT `change-event`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeEvent {
    MailboxChanged(MailboxRef),
    Disconnected,
}

/// Backend feature flags (mirrors the WIT `backend-caps`). Gmail advertises `move`
/// (label re-tagging) but none of the Outlook-native caps.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendCaps {
    pub idle: bool,
    pub move_cap: bool,
    pub reactions: bool,
    pub voting: bool,
    pub recall: bool,
    pub focused_sync: bool,
}

/// A coarse bridge error (mirrors `mw_engine::EngineError` / the WIT `plugin-error`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BridgeError {
    Protocol(String),
    Auth(String),
    Transport(String),
    Unsupported(String),
    MailboxNotFound(String),
    Other(String),
}

pub type Result<T> = std::result::Result<T, BridgeError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gmail_ref_packs_label_and_id_for_plugin_passthrough() {
        let r = MessageRef::for_gmail("INBOX", "18c2ab0011ffee00");
        // The host adapter passes `raw` through verbatim as MessageRef::Plugin — it is
        // the packed provider-native id, NOT engine-JSON.
        assert_eq!(r.raw, "INBOX\u{1f}18c2ab0011ffee00");
        let (label, id) = r.decode_gmail().unwrap();
        assert_eq!(label, "INBOX");
        assert_eq!(id, "18c2ab0011ffee00");
    }

    #[test]
    fn bare_id_decodes_with_empty_label() {
        let r = MessageRef {
            raw: "18c2ab0011ffee00".into(),
            mailbox: MailboxRef {
                name: "INBOX".into(),
                uidvalidity: 1,
            },
        };
        assert_eq!(
            r.decode_gmail(),
            Some((String::new(), "18c2ab0011ffee00".into()))
        );
    }

    #[test]
    fn cursor_carries_history_id_as_raw_bytes() {
        let c = SyncCursor::from_history_id("9876");
        // The historyId rides `opaque` as its raw UTF-8 bytes (no JSON wrapping).
        assert_eq!(c.opaque, b"9876");
        assert_eq!(c.history_id().as_deref(), Some("9876"));
    }

    #[test]
    fn empty_cursor_means_full_sync() {
        assert_eq!(SyncCursor { opaque: vec![] }.history_id(), None);
    }
}
