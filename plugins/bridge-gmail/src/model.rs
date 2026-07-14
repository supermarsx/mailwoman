//! Transport-neutral value types + the frozen-ABI encoding helpers.
//!
//! These plain types mirror the `mailwoman:plugin` WIT records closely, but carry
//! NO `wit-bindgen` dependency, so the whole Gmail backend ([`crate::backend`]) is
//! host-compilable and unit-testable without a wasm toolchain. The wasm component
//! ([`crate::component`], gated to `target_family = "wasm"`) converts these ↔ the
//! generated WIT types at the boundary.
//!
//! ## Two frozen-ABI smuggles (host adapter contract — see `mw-plugin/src/adapter.rs`)
//! The `mw-plugin` host adapter JSON-encodes the engine's `MessageRef`/`SyncCursor`
//! into the WIT `message-ref.raw` string / `sync-cursor.opaque` bytes and decodes
//! the guest's return values the same way. So every ref/cursor this guest emits
//! MUST be **valid JSON of an `mw_engine::MessageRef` / `mw_engine::SyncCursor`**.
//!
//! * **Cursor** — the engine grew an additive `SyncCursor::Plugin { opaque }`
//!   variant (t7-e8) exactly for bridges. We carry the Gmail `historyId` there
//!   losslessly: `{"kind":"plugin","opaque":<historyId bytes>}`.
//! * **Message ref** — the engine `MessageRef` enum has NO plugin variant (only
//!   `Imap`/`Pop3`), and a Gmail message id is a 16-hex-digit string that does not
//!   fit the `Imap { uid: u32 }` shape. So we smuggle the Gmail id (plus its owning
//!   label, for a correct multi-label move/store) through the string-carrying
//!   `Pop3 { uidl }` variant: `uidl = "<labelId>\u{1f}<gmailId>"`. The engine
//!   treats `uidl` as opaque and round-trips it verbatim; only this bridge decodes
//!   it. (An additive `MessageRef::Plugin { opaque }` would be cleaner — the same
//!   call the coordinator made for `SyncCursor::Plugin`; flagged in the e12 report.)

use serde::{Deserialize, Serialize};

/// Field separator packed into the smuggled `Pop3.uidl` (US, 0x1F — never appears
/// in a Gmail label id or message id).
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
/// engine-`MessageRef` JSON the host adapter round-trips (see the smuggle note).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageRef {
    pub raw: String,
    pub mailbox: MailboxRef,
}

impl MessageRef {
    /// Build a ref for a Gmail message id owned by `label`, encoding both into a
    /// `Pop3.uidl`-shaped engine-ref JSON the host adapter accepts.
    pub fn for_gmail(label: &str, gmail_id: &str) -> Self {
        let uidl = format!("{label}{SEP}{gmail_id}");
        let raw = serde_json::to_string(&EngineRef::Pop3 { uidl }).unwrap_or_default();
        Self {
            raw,
            mailbox: MailboxRef {
                name: label.to_string(),
                uidvalidity: 1,
            },
        }
    }

    /// Decode `(labelId, gmailId)` back out of a ref the host handed us. Tolerant of
    /// a bare uidl without the separator (⇒ whole string is the id, no label).
    pub fn decode_gmail(&self) -> Option<(String, String)> {
        let er: EngineRef = serde_json::from_str(&self.raw).ok()?;
        let uidl = match er {
            EngineRef::Pop3 { uidl } => uidl,
            // An Imap-shaped ref cannot carry a Gmail id — not ours.
            EngineRef::Imap { .. } => return None,
        };
        match uidl.split_once(SEP) {
            Some((label, id)) => Some((label.to_string(), id.to_string())),
            None => Some((String::new(), uidl)),
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
/// is the engine-`SyncCursor` JSON the host adapter round-trips.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncCursor {
    pub opaque: Vec<u8>,
}

impl SyncCursor {
    /// Wrap a Gmail `historyId` as an engine `SyncCursor::Plugin { opaque }`.
    pub fn from_history_id(history_id: &str) -> Self {
        let inner = EngineCursor::Plugin {
            opaque: history_id.as_bytes().to_vec(),
        };
        Self {
            opaque: serde_json::to_vec(&inner).unwrap_or_default(),
        }
    }

    /// Extract the Gmail `historyId` if this cursor carries one. Returns `None` for
    /// an empty / non-plugin / unparseable cursor ⇒ the caller does a full sync.
    pub fn history_id(&self) -> Option<String> {
        if self.opaque.is_empty() {
            return None;
        }
        let inner: EngineCursor = serde_json::from_slice(&self.opaque).ok()?;
        match inner {
            EngineCursor::Plugin { opaque } if !opaque.is_empty() => String::from_utf8(opaque).ok(),
            _ => None,
        }
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

// ── Frozen-ABI serde mirrors (byte-identical to the `mw_engine` derives) ─────────

/// Externally-tagged serde mirror of `mw_engine::MessageRef` — must match its
/// derive exactly so the host adapter's `from_str::<mw_engine::MessageRef>` accepts
/// what we emit. `mw_engine::MessageRef` has no serde container attr ⇒ external
/// tagging: `{"Pop3":{"uidl":"…"}}` / `{"Imap":{…}}`.
#[derive(Debug, Serialize, Deserialize)]
enum EngineRef {
    Imap {
        mailbox: EngineMailboxRef,
        uidvalidity: u32,
        uid: u32,
    },
    Pop3 {
        uidl: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct EngineMailboxRef {
    name: String,
    uidvalidity: u32,
}

/// Internally-tagged serde mirror of `mw_engine::SyncCursor` — it derives
/// `#[serde(tag = "kind", rename_all = "snake_case")]`, so the `Plugin` variant is
/// `{"kind":"plugin","opaque":[…]}`.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum EngineCursor {
    Plugin {
        opaque: Vec<u8>,
    },
    // Other variants (qresync/condstore/uid_window/pop3_uidl) are never emitted by
    // this bridge; deserialization of them simply yields "no history id".
    #[serde(other)]
    Other,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gmail_ref_round_trips_through_engine_ref_json() {
        let r = MessageRef::for_gmail("INBOX", "18c2ab0011ffee00");
        // The host adapter decodes `raw` as an `mw_engine::MessageRef` — assert it
        // is the Pop3 shape carrying our packed uidl.
        assert!(r.raw.contains("Pop3"));
        assert!(r.raw.contains("18c2ab0011ffee00"));
        let (label, id) = r.decode_gmail().unwrap();
        assert_eq!(label, "INBOX");
        assert_eq!(id, "18c2ab0011ffee00");
    }

    #[test]
    fn cursor_carries_history_id_as_plugin_variant() {
        let c = SyncCursor::from_history_id("9876");
        // Must be the engine's internally-tagged plugin cursor JSON.
        let s = String::from_utf8(c.opaque.clone()).unwrap();
        assert!(s.contains("\"kind\":\"plugin\""));
        assert_eq!(c.history_id().as_deref(), Some("9876"));
    }

    #[test]
    fn empty_or_foreign_cursor_means_full_sync() {
        assert_eq!(SyncCursor { opaque: vec![] }.history_id(), None);
        // A non-plugin engine cursor (e.g. an initial qresync) ⇒ full sync.
        let foreign = br#"{"kind":"qresync","uidvalidity":1,"highestmodseq":2}"#.to_vec();
        assert_eq!(SyncCursor { opaque: foreign }.history_id(), None);
    }
}
