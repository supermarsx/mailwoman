//! Engine-facing wire encoding. The `mw-plugin` host adapter carries a plugin
//! backend's `message-ref.raw` string and `sync-cursor.opaque` bytes VERBATIM,
//! wrapping them into `mw_engine::MessageRef::Plugin { raw }` /
//! `SyncCursor::Plugin { opaque }` and handing the exact same bytes back on the next
//! call (see `crates/mw-plugin/src/adapter.rs`, t7-fix-msgref). So this bridge emits
//! its own **provider-native** encodings, no engine-JSON disguise:
//!
//! EWS identifies items by an opaque `ItemId` (+ a `ChangeKey` that advances as the
//! item is modified) and resumes sync from a `SyncState` token. The bridge packs the
//! `ItemId` + `ChangeKey` into `message-ref.raw` and carries the `SyncState` bytes in
//! `sync-cursor.opaque`, both round-tripped losslessly — no synthetic-UID map needed.

/// Field separator packed into the message-ref `raw` (US, 0x1F — never appears in a
/// base64 EWS `ItemId` or `ChangeKey`).
const SEP: char = '\u{1f}';

/// Pack an EWS `(item_id, change_key)` into the opaque `message-ref.raw` string the
/// host adapter passes through as `MessageRef::Plugin { raw }`.
#[must_use]
pub fn encode_msgref(item_id: &str, change_key: &str) -> String {
    format!("{item_id}{SEP}{change_key}")
}

/// Recover `(item_id, change_key)` from a `message-ref.raw`. Tolerant of a bare
/// `item_id` without the separator (⇒ empty change key). Returns `None` for an empty
/// ref (⇒ not ours).
#[must_use]
pub fn decode_msgref(raw: &str) -> Option<(String, String)> {
    if raw.is_empty() {
        return None;
    }
    match raw.split_once(SEP) {
        Some((id, ck)) => Some((id.to_string(), ck.to_string())),
        None => Some((raw.to_string(), String::new())),
    }
}

/// Carry our EWS `SyncState` string as the raw `sync-cursor.opaque` bytes the host
/// round-trips verbatim as `SyncCursor::Plugin { opaque }` (plan §3 e8).
#[must_use]
pub fn encode_cursor(sync_state: &str) -> Vec<u8> {
    sync_state.as_bytes().to_vec()
}

/// Recover the EWS `SyncState` string from the `sync-cursor.opaque` bytes the host
/// handed to `sync-mailbox` (empty ⇒ a fresh full sync).
#[must_use]
pub fn decode_cursor(opaque: &[u8]) -> String {
    String::from_utf8_lossy(opaque).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn msgref_round_trips_item_id_and_change_key() {
        let raw = encode_msgref("AAMkITEM==", "CK-9");
        assert_eq!(raw, "AAMkITEM==\u{1f}CK-9");
        assert_eq!(
            decode_msgref(&raw),
            Some(("AAMkITEM==".into(), "CK-9".into()))
        );
    }

    #[test]
    fn bare_item_id_decodes_with_empty_change_key() {
        assert_eq!(
            decode_msgref("AAMkITEM=="),
            Some(("AAMkITEM==".into(), String::new()))
        );
        assert_eq!(decode_msgref(""), None);
    }

    #[test]
    fn cursor_round_trips_sync_state() {
        let opaque = encode_cursor("SYNCSTATEXYZ==");
        assert_eq!(opaque, b"SYNCSTATEXYZ==");
        assert_eq!(decode_cursor(&opaque), "SYNCSTATEXYZ==");
        // An empty cursor decodes to a fresh sync.
        assert_eq!(decode_cursor(&[]), "");
    }
}
