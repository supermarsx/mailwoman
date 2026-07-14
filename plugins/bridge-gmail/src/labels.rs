//! True Gmail label semantics ↔ the engine's mailbox/flag model (plan §3 e12).
//!
//! Gmail has no folders: a message carries a *set* of labels. Some system labels
//! map to a mailbox **role** (a folder view — INBOX/SENT/…); a few map to a
//! **flag** (STARRED→Flagged, UNREAD→¬Seen, DRAFT→Draft); the rest are keywords.
//! User labels map to *both* a folder (so the multi-label message appears in each)
//! and a `Keyword`, preserving fidelity. This is the whole Gmail-quirk surface —
//! isolated here, per the plan's "quirks live only in the bridge" rule.

use crate::model::Flag;

/// Gmail system label ids that are NOT mailboxes/roles — they are flags or hidden.
pub const SYS_UNREAD: &str = "UNREAD";
pub const SYS_STARRED: &str = "STARRED";
pub const SYS_IMPORTANT: &str = "IMPORTANT";
pub const SYS_DRAFT: &str = "DRAFT";
pub const SYS_TRASH: &str = "TRASH";
pub const SYS_SPAM: &str = "SPAM";
pub const SYS_INBOX: &str = "INBOX";
pub const SYS_SENT: &str = "SENT";

/// The synthetic "All Mail" mailbox (Gmail's [Gmail]/All Mail). Gmail exposes no
/// `ALL` label id; we surface a virtual mailbox whose sync spans every message.
pub const ALL_MAIL_ID: &str = "__ALL_MAIL__";

/// Map a Gmail label id to a JMAP special-use role, or `None` for a plain folder.
/// Returns the lowercase role string the WIT `mailbox.role` field wants.
#[must_use]
pub fn label_to_role(label_id: &str) -> &'static str {
    match label_id {
        SYS_INBOX => "inbox",
        SYS_SENT => "sent",
        SYS_DRAFT => "drafts",
        SYS_TRASH => "trash",
        SYS_SPAM => "junk",
        ALL_MAIL_ID => "all",
        _ => "none",
    }
}

/// Whether a Gmail system label id denotes a *mailbox* (a folder view) rather than
/// a flag/keyword/hidden label. User labels (type != "system") are always mailboxes.
#[must_use]
pub fn system_label_is_mailbox(label_id: &str) -> bool {
    matches!(
        label_id,
        SYS_INBOX | SYS_SENT | SYS_DRAFT | SYS_TRASH | SYS_SPAM
    )
}

/// Labels that are purely presentational Gmail state (tabs / chat) — never a
/// mailbox and not surfaced as keywords either.
#[must_use]
pub fn is_hidden_label(label_id: &str) -> bool {
    label_id == "CHAT"
        || label_id.starts_with("CATEGORY_")
        || label_id == SYS_UNREAD
        || label_id == SYS_STARRED
        || label_id == SYS_IMPORTANT
}

/// Derive the engine flag set from a message's full Gmail `labelIds`.
///
/// * absence of `UNREAD` ⇒ `Seen` (Gmail models unread, IMAP models seen).
/// * `STARRED` ⇒ `Flagged`; `DRAFT` ⇒ `Draft`; `IMPORTANT` ⇒ `$Important` keyword.
/// * every user label (a non-system id) ⇒ a `Keyword`, so the multi-label state
///   survives round-trips even where no folder view is shown.
#[must_use]
pub fn labels_to_flags(label_ids: &[String]) -> Vec<Flag> {
    let mut flags = Vec::new();
    let has = |id: &str| label_ids.iter().any(|l| l == id);

    if !has(SYS_UNREAD) {
        flags.push(Flag::Seen);
    }
    if has(SYS_STARRED) {
        flags.push(Flag::Flagged);
    }
    if has(SYS_DRAFT) {
        flags.push(Flag::Draft);
    }
    if has(SYS_IMPORTANT) {
        flags.push(Flag::Keyword("$Important".to_string()));
    }
    for id in label_ids {
        if is_system_label_id(id) || is_hidden_label(id) {
            continue;
        }
        flags.push(Flag::Keyword(id.clone()));
    }
    flags
}

/// The Gmail label-id add/remove sets that realise an engine flag change
/// (`messages.modify`). Returns `(add_label_ids, remove_label_ids)`.
///
/// This is the inverse of [`labels_to_flags`] for the mutable flags: toggling
/// `Seen` toggles `UNREAD` the other way; `Flagged`↔`STARRED`; `Deleted`↔`TRASH`;
/// a `Keyword` is taken as a label id to add/remove directly.
#[must_use]
pub fn flags_to_label_ops(add: &[Flag], remove: &[Flag]) -> (Vec<String>, Vec<String>) {
    let mut add_ids: Vec<String> = Vec::new();
    let mut rem_ids: Vec<String> = Vec::new();

    for f in add {
        match f {
            // Marking Seen REMOVES the UNREAD label.
            Flag::Seen => rem_ids.push(SYS_UNREAD.into()),
            Flag::Flagged => add_ids.push(SYS_STARRED.into()),
            Flag::Deleted => add_ids.push(SYS_TRASH.into()),
            Flag::Keyword(k) => add_ids.push(k.clone()),
            // Gmail has no Answered/Recent label; Draft is set by the draft API.
            Flag::Answered | Flag::Recent | Flag::Draft => {}
        }
    }
    for f in remove {
        match f {
            // Removing Seen ADDS UNREAD back.
            Flag::Seen => add_ids.push(SYS_UNREAD.into()),
            Flag::Flagged => rem_ids.push(SYS_STARRED.into()),
            Flag::Deleted => rem_ids.push(SYS_TRASH.into()),
            Flag::Keyword(k) => rem_ids.push(k.clone()),
            Flag::Answered | Flag::Recent | Flag::Draft => {}
        }
    }
    (add_ids, rem_ids)
}

/// Whether `id` is a Gmail *system* label id (all-caps reserved names). User labels
/// are typically `Label_<n>` or arbitrary names and are treated as keywords/folders.
fn is_system_label_id(id: &str) -> bool {
    matches!(
        id,
        SYS_INBOX
            | SYS_SENT
            | SYS_DRAFT
            | SYS_TRASH
            | SYS_SPAM
            | SYS_UNREAD
            | SYS_STARRED
            | SYS_IMPORTANT
    ) || id.starts_with("CATEGORY_")
        || id == "CHAT"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_labels_map_to_roles() {
        assert_eq!(label_to_role("INBOX"), "inbox");
        assert_eq!(label_to_role("SENT"), "sent");
        assert_eq!(label_to_role("DRAFT"), "drafts");
        assert_eq!(label_to_role("TRASH"), "trash");
        assert_eq!(label_to_role("SPAM"), "junk");
        assert_eq!(label_to_role(ALL_MAIL_ID), "all");
        assert_eq!(label_to_role("Label_7"), "none");
    }

    #[test]
    fn unread_and_star_map_to_flags_correctly() {
        // An unread, starred INBOX message with a user label.
        let ids = vec![
            "INBOX".to_string(),
            "UNREAD".to_string(),
            "STARRED".to_string(),
            "Label_Receipts".to_string(),
        ];
        let flags = labels_to_flags(&ids);
        assert!(!flags.contains(&Flag::Seen), "UNREAD ⇒ not Seen");
        assert!(flags.contains(&Flag::Flagged), "STARRED ⇒ Flagged");
        assert!(
            flags.contains(&Flag::Keyword("Label_Receipts".to_string())),
            "user label ⇒ keyword"
        );
        // INBOX/UNREAD/STARRED must NOT leak in as keywords.
        assert!(!flags.contains(&Flag::Keyword("INBOX".to_string())));
        assert!(!flags.contains(&Flag::Keyword("UNREAD".to_string())));

        // A read message (no UNREAD) ⇒ Seen.
        let read = vec!["INBOX".to_string()];
        assert!(labels_to_flags(&read).contains(&Flag::Seen));
    }

    #[test]
    fn flag_ops_invert_label_state() {
        // Mark Seen ⇒ remove UNREAD; add Flagged ⇒ add STARRED.
        let (add, rem) = flags_to_label_ops(&[Flag::Seen, Flag::Flagged], &[]);
        assert!(add.contains(&"STARRED".to_string()));
        assert!(rem.contains(&"UNREAD".to_string()));

        // Un-Seen ⇒ add UNREAD back; un-Flag ⇒ remove STARRED.
        let (add, rem) = flags_to_label_ops(&[], &[Flag::Seen, Flag::Flagged]);
        assert!(add.contains(&"UNREAD".to_string()));
        assert!(rem.contains(&"STARRED".to_string()));
    }
}
