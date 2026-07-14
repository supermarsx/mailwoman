//! Conversions across the store seam (plan §1.9): the engine owns every mapping
//! between its typed values and the opaque JSON/string primitives `mw-store`
//! persists.
//!
//! `mw-store` deliberately does not depend on `mw-engine`, so the engine is the
//! single place that knows how a [`SyncCursor`] becomes `cursor_json`, how a
//! `Vec<Flag>` becomes `flags_json`, how a [`MailboxRole`] becomes the store's
//! role string, and how IMAP flags project onto JMAP keywords.

use std::collections::HashMap;

use crate::backend::{Flag, MailboxRole, SyncCursor};

/// Serialize the engine's flag set to the opaque `flags_json` the store keeps.
///
/// Uses serde's default enum encoding, e.g. `["Seen","Flagged"]` /
/// `[{"Keyword":"$Forwarded"}]` — round-tripped only by [`flags_from_json`].
pub fn flags_to_json(flags: &[Flag]) -> String {
    serde_json::to_string(flags).unwrap_or_else(|_| "[]".to_string())
}

/// Parse `flags_json` back into the engine's flag set (empty on malformed input).
pub fn flags_from_json(json: &str) -> Vec<Flag> {
    serde_json::from_str(json).unwrap_or_default()
}

/// Serialize a sync cursor to the opaque `cursor_json` the store keeps verbatim.
pub fn cursor_to_json(cursor: &SyncCursor) -> String {
    serde_json::to_string(cursor).unwrap_or_default()
}

/// Parse `cursor_json` back into a [`SyncCursor`], if it is well-formed.
pub fn cursor_from_json(json: &str) -> Option<SyncCursor> {
    serde_json::from_str(json).ok()
}

/// The universal first-sync cursor: a UID window with a sentinel UIDVALIDITY of
/// `0` so the IMAP backend adopts whatever the mailbox actually reports without
/// treating it as a UIDVALIDITY change (see `mw-imap`'s `sync_uid_window`).
pub fn initial_cursor() -> SyncCursor {
    SyncCursor::UidWindow {
        uidvalidity: 0,
        uidnext: 1,
    }
}

/// Map a special-use role to the store's opaque lowercase role string.
///
/// [`MailboxRole::None`] carries no role, so it maps to a SQL `NULL`.
pub fn role_to_store(role: MailboxRole) -> Option<&'static str> {
    match role {
        MailboxRole::Inbox => Some("inbox"),
        MailboxRole::Archive => Some("archive"),
        MailboxRole::Drafts => Some("drafts"),
        MailboxRole::Sent => Some("sent"),
        MailboxRole::Trash => Some("trash"),
        MailboxRole::Junk => Some("junk"),
        MailboxRole::Flagged => Some("flagged"),
        MailboxRole::All => Some("all"),
        MailboxRole::None => None,
    }
}

/// JMAP `Mailbox.sortOrder` — a stable ordering the UI can rely on.
pub fn role_sort_order(role: Option<&str>) -> u32 {
    match role {
        Some("inbox") => 0,
        Some("drafts") => 1,
        Some("sent") => 2,
        Some("archive") => 3,
        Some("junk") => 4,
        Some("trash") => 5,
        Some("all") => 6,
        Some("flagged") => 7,
        _ => 10,
    }
}

/// A human display name for a mailbox, from its role or the leaf of its path.
pub fn display_name(imap_name: &str, role: Option<&str>) -> String {
    match role {
        Some("inbox") => "Inbox".to_string(),
        Some("drafts") => "Drafts".to_string(),
        Some("sent") => "Sent".to_string(),
        Some("archive") => "Archive".to_string(),
        Some("junk") => "Spam".to_string(),
        Some("trash") => "Trash".to_string(),
        Some("all") => "All Mail".to_string(),
        Some("flagged") => "Flagged".to_string(),
        _ => {
            let leaf = imap_name
                .rsplit(['/', '.'])
                .next()
                .unwrap_or(imap_name)
                .trim();
            if leaf.eq_ignore_ascii_case("inbox") {
                "Inbox".to_string()
            } else if leaf.is_empty() {
                imap_name.to_string()
            } else {
                leaf.to_string()
            }
        }
    }
}

/// Project the engine's flags onto the JMAP `keywords` map (RFC 8621 §4.1.1).
///
/// Only the flags with a JMAP keyword equivalent are surfaced; IMAP-internal
/// `\Recent`/`\Deleted` are omitted (they are not JMAP keywords).
pub fn flags_to_keywords(flags: &[Flag]) -> HashMap<String, bool> {
    let mut map = HashMap::new();
    for f in flags {
        if let Some(k) = flag_keyword(f) {
            map.insert(k, true);
        }
    }
    map
}

/// The JMAP keyword for a flag, or `None` for IMAP-only flags.
fn flag_keyword(flag: &Flag) -> Option<String> {
    match flag {
        Flag::Seen => Some("$seen".to_string()),
        Flag::Answered => Some("$answered".to_string()),
        Flag::Flagged => Some("$flagged".to_string()),
        Flag::Draft => Some("$draft".to_string()),
        Flag::Keyword(k) => Some(k.clone()),
        Flag::Deleted | Flag::Recent => None,
    }
}

/// Invert [`flags_to_keywords`]: a JMAP keyword map back into engine flags.
///
/// Only keys set to `true` count. The system keywords map back to their IMAP
/// system flag; anything else becomes a [`Flag::Keyword`].
pub fn keywords_to_flags(keywords: &HashMap<String, bool>) -> Vec<Flag> {
    let mut out = Vec::new();
    for (k, on) in keywords {
        if !on {
            continue;
        }
        out.push(match k.as_str() {
            "$seen" => Flag::Seen,
            "$answered" => Flag::Answered,
            "$flagged" => Flag::Flagged,
            "$draft" => Flag::Draft,
            other => Flag::Keyword(other.to_string()),
        });
    }
    out
}

/// Compute the `(add, remove)` flag deltas to take `current` to `desired`.
///
/// IMAP-internal `\Recent`/`\Deleted` are never emitted — they are not
/// keyword-controllable — so a keyword update leaves them untouched upstream.
pub fn flag_delta(current: &[Flag], desired: &[Flag]) -> (Vec<Flag>, Vec<Flag>) {
    let keywordy = |f: &Flag| !matches!(f, Flag::Deleted | Flag::Recent);
    let add = desired
        .iter()
        .filter(|f| keywordy(f) && !current.contains(f))
        .cloned()
        .collect();
    let remove = current
        .iter()
        .filter(|f| keywordy(f) && !desired.contains(f))
        .cloned()
        .collect();
    (add, remove)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_json_round_trips() {
        let flags = vec![Flag::Seen, Flag::Keyword("$forwarded".into())];
        let json = flags_to_json(&flags);
        assert_eq!(flags_from_json(&json), flags);
        assert_eq!(flags_from_json("not json"), Vec::<Flag>::new());
    }

    #[test]
    fn cursor_json_round_trips() {
        let c = SyncCursor::Condstore {
            uidvalidity: 7,
            modseq: 99,
        };
        let json = cursor_to_json(&c);
        assert_eq!(cursor_from_json(&json), Some(c));
        assert!(json.contains("condstore"));
    }

    #[test]
    fn plugin_cursor_round_trips_native_bytes() {
        // A bridge carries a native token (e.g. a Graph deltaLink) verbatim — the
        // engine persists it losslessly through the opaque `cursor_json` column.
        let deltalink =
            b"https://graph.microsoft.com/v1.0/.../delta?$deltatoken=ABC.123\x00\xff".to_vec();
        let c = SyncCursor::Plugin {
            opaque: deltalink.clone(),
        };
        let json = cursor_to_json(&c);
        // Struct-variant internal tagging encodes cleanly (a newtype `Vec<u8>` could
        // not) — the tag + the bytes survive.
        assert!(json.contains("plugin"), "tag present: {json}");
        assert_eq!(cursor_from_json(&json), Some(c));
        match cursor_from_json(&json) {
            Some(SyncCursor::Plugin { opaque }) => assert_eq!(opaque, deltalink),
            other => panic!("expected a Plugin cursor, got {other:?}"),
        }
    }

    #[test]
    fn keywords_map_both_ways() {
        let flags = vec![Flag::Seen, Flag::Draft, Flag::Deleted, Flag::Recent];
        let kw = flags_to_keywords(&flags);
        assert_eq!(kw.get("$seen"), Some(&true));
        assert_eq!(kw.get("$draft"), Some(&true));
        assert!(!kw.contains_key("$deleted")); // IMAP-only, not a keyword
        assert_eq!(kw.len(), 2);

        let back = keywords_to_flags(&kw);
        assert!(back.contains(&Flag::Seen) && back.contains(&Flag::Draft));
    }

    #[test]
    fn flag_delta_ignores_internal_flags() {
        let current = vec![Flag::Deleted, Flag::Flagged];
        let desired = vec![Flag::Seen];
        let (add, remove) = flag_delta(&current, &desired);
        assert_eq!(add, vec![Flag::Seen]);
        assert_eq!(remove, vec![Flag::Flagged]); // \Deleted preserved
    }

    #[test]
    fn display_names_and_roles() {
        assert_eq!(display_name("INBOX", Some("inbox")), "Inbox");
        assert_eq!(display_name("[Gmail]/Sent Mail", Some("sent")), "Sent");
        assert_eq!(display_name("Lists/rust", None), "rust");
        assert_eq!(role_to_store(MailboxRole::Junk), Some("junk"));
        assert_eq!(role_to_store(MailboxRole::None), None);
        assert!(role_sort_order(Some("inbox")) < role_sort_order(Some("sent")));
    }
}
