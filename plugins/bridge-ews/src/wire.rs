//! Engine-facing wire encoding. The `mw-plugin` host adapter decodes a guest
//! `message-ref.raw` string as JSON of `mw_engine::MessageRef`, and a
//! `sync-cursor.opaque` blob as JSON of `mw_engine::SyncCursor` (see
//! `crates/mw-plugin/src/adapter.rs`). A wasm guest cannot depend on the heavy
//! `mw-engine` crate, so this module defines **byte-for-byte-compatible serde
//! mirrors** and hand-drives them with `serde_json`. Wire compatibility is proven
//! against the real `mw-engine` types in `tests/wire_compat.rs`.
//!
//! EWS uses opaque `ItemId`s, not IMAP UIDs, so the bridge synthesizes a stable
//! per-mailbox `uid` for each item and carries the real `ItemId`/`ChangeKey` in its
//! own session/KV map (`crate::state`). The engine only ever sees a
//! `MessageRef::Imap { mailbox, uidvalidity, uid }`.

use serde::{Deserialize, Serialize};

/// Mirror of `mw_engine::RawMailboxRef`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MboxRef {
    pub name: String,
    pub uidvalidity: u32,
}

/// Mirror of `mw_engine::MessageRef` (externally-tagged enum — the serde default).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MsgRef {
    Imap {
        mailbox: MboxRef,
        uidvalidity: u32,
        uid: u32,
    },
    Pop3 {
        uidl: String,
    },
}

impl MsgRef {
    /// Build the synthetic IMAP-shaped ref the engine persists for an EWS item.
    #[must_use]
    pub fn imap(mailbox: &str, uidvalidity: u32, uid: u32) -> Self {
        MsgRef::Imap {
            mailbox: MboxRef {
                name: mailbox.to_string(),
                uidvalidity,
            },
            uidvalidity,
            uid,
        }
    }

    /// The `(uid)` if this is an IMAP-shaped ref.
    #[must_use]
    pub fn uid(&self) -> Option<u32> {
        match self {
            MsgRef::Imap { uid, .. } => Some(*uid),
            MsgRef::Pop3 { .. } => None,
        }
    }
}

/// Encode a `MsgRef` as the JSON string the host adapter puts in `message-ref.raw`.
#[must_use]
pub fn encode_msgref(r: &MsgRef) -> String {
    serde_json::to_string(r).unwrap_or_default()
}

/// Decode a host `message-ref.raw` JSON string back into a `MsgRef`.
#[must_use]
pub fn decode_msgref(raw: &str) -> Option<MsgRef> {
    serde_json::from_str(raw).ok()
}

/// Serialize mirror of `mw_engine::SyncCursor::Plugin { opaque }`
/// (internally-tagged `#[serde(tag = "kind", rename_all = "snake_case")]`).
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum CursorOut {
    Plugin { opaque: Vec<u8> },
}

/// Permissive peek at whatever cursor the engine handed back — recovers the inner
/// `opaque` payload (our EWS `SyncState`) if this was a `Plugin` cursor, else empty.
#[derive(Debug, Default, Deserialize)]
struct CursorPeek {
    #[serde(default)]
    opaque: Vec<u8>,
}

/// Encode our EWS `SyncState` string as the `sync-cursor.opaque` bytes the host
/// stores (`SyncCursor::Plugin { opaque }` JSON, so the engine round-trips it
/// verbatim per plan §3 e8).
#[must_use]
pub fn encode_cursor(sync_state: &str) -> Vec<u8> {
    serde_json::to_vec(&CursorOut::Plugin {
        opaque: sync_state.as_bytes().to_vec(),
    })
    .unwrap_or_default()
}

/// Recover the EWS `SyncState` string from the `sync-cursor.opaque` bytes the host
/// handed to `sync-mailbox` (empty ⇒ a fresh full sync).
#[must_use]
pub fn decode_cursor(opaque: &[u8]) -> String {
    if opaque.is_empty() {
        return String::new();
    }
    let peek: CursorPeek = serde_json::from_slice(opaque).unwrap_or_default();
    String::from_utf8(peek.opaque).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn msgref_round_trips() {
        let r = MsgRef::imap("INBOX", 3, 42);
        let s = encode_msgref(&r);
        assert!(s.contains("\"Imap\""));
        assert_eq!(decode_msgref(&s), Some(r));
    }

    #[test]
    fn cursor_round_trips_sync_state() {
        let opaque = encode_cursor("SYNCSTATEXYZ==");
        assert_eq!(decode_cursor(&opaque), "SYNCSTATEXYZ==");
        // A non-plugin/empty cursor decodes to a fresh sync.
        assert_eq!(decode_cursor(&[]), "");
        assert_eq!(
            decode_cursor(br#"{"kind":"uid_window","uidvalidity":1,"uidnext":9}"#),
            ""
        );
    }
}
