//! Mail operations — the `account-backend` seam over Microsoft Graph. Pure and
//! target-independent: every function drives a [`GraphClient`] and returns the plain
//! [`crate::types`] the `wasm32` guest maps to WIT. Focused-Inbox state rides on the
//! `$Focused`/`$Other` keywords (plan §2.5); the delta `@odata.deltaLink` is the
//! opaque [`SyncCursor`] the engine persists (`SyncCursor::Plugin`).

use base64::Engine as _;

use crate::graph::{BridgeError, GraphClient, Result, Transport};
use crate::model::{MailFolder, MailFoldersResponse, MessagesDeltaResponse};
use crate::types::{
    ChangeEvent, Flag, Mailbox, MailboxDelta, MailboxRef, MessageRef, RawMessage, SyncCursor,
    KEYWORD_FOCUSED, KEYWORD_OTHER,
};

/// Map a Graph `wellKnownName` to a JMAP special-use role (lowercased).
fn role_of(folder: &MailFolder) -> String {
    match folder.well_known_name.as_deref() {
        Some("inbox") => "inbox",
        Some("archive") => "archive",
        Some("drafts") => "drafts",
        Some("sentitems") => "sent",
        Some("deleteditems") => "trash",
        Some("junkemail") => "junk",
        Some("outbox") => "none",
        _ => "none",
    }
    .to_string()
}

/// The addressing key for a folder: the well-known name (stable, human-legible,
/// Graph-addressable at `/me/mailFolders/{name}`) when present, else the opaque id
/// (also Graph-addressable). This is what a later `sync_mailbox` re-addresses from.
fn addressing_key(folder: &MailFolder) -> String {
    folder
        .well_known_name
        .clone()
        .filter(|w| !w.is_empty())
        .unwrap_or_else(|| folder.id.clone())
}

/// `GET /me/mailFolders` → the mailbox list. Follows `@odata.nextLink` paging.
pub fn list_mailboxes<T: Transport>(client: &GraphClient<'_, T>) -> Result<Vec<Mailbox>> {
    let mut out = Vec::new();
    let mut path = "/me/mailFolders?$top=100".to_string();
    loop {
        let page: MailFoldersResponse = client.get_json(&path)?;
        for folder in &page.value {
            out.push(Mailbox {
                mailbox_ref: MailboxRef {
                    name: addressing_key(folder),
                    uidvalidity: 1,
                },
                role: role_of(folder),
                parent: folder.parent_folder_id.clone(),
                total: folder.total_item_count,
                unread: folder.unread_item_count,
            });
        }
        match page.next_link {
            Some(next) => path = next,
            None => break,
        }
    }
    Ok(out)
}

/// The Graph delta URL for a folder's first sync (no cursor). `$select` keeps the
/// payload lean and pulls exactly the properties the flag mapping needs.
fn initial_delta_path(mbox: &MailboxRef) -> String {
    format!(
        "/me/mailFolders/{}/messages/delta?$select=isRead,flag,inferenceClassification,receivedDateTime",
        mbox.name
    )
}

/// Derive the flag set for a message from its Graph properties.
fn flags_of(m: &crate::model::GraphMessage) -> Vec<Flag> {
    let mut flags = Vec::new();
    if m.is_read == Some(true) {
        flags.push(Flag::Seen);
    }
    if let Some(f) = &m.flag {
        if f.flag_status.as_deref() == Some("flagged") {
            flags.push(Flag::Flagged);
        }
    }
    match m.inference_classification.as_deref() {
        Some("focused") => flags.push(Flag::Keyword(KEYWORD_FOCUSED.to_string())),
        Some("other") => flags.push(Flag::Keyword(KEYWORD_OTHER.to_string())),
        _ => {}
    }
    flags
}

/// `GET …/messages/delta` (or the stored deltaLink) → an incremental [`MailboxDelta`].
/// The returned cursor is the next `@odata.deltaLink` (or `@odata.nextLink` mid-page)
/// as opaque bytes, which the engine round-trips back on the next call.
pub fn sync_mailbox<T: Transport>(
    client: &GraphClient<'_, T>,
    mbox: &MailboxRef,
    cursor: &SyncCursor,
) -> Result<MailboxDelta> {
    let path = if cursor.opaque.is_empty() {
        initial_delta_path(mbox)
    } else {
        String::from_utf8(cursor.opaque.clone())
            .map_err(|e| BridgeError::Protocol(format!("cursor not utf-8: {e}")))?
    };

    let page: MessagesDeltaResponse = client.get_json(&path)?;

    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut flag_changes = Vec::new();
    for m in &page.value {
        let mref = MessageRef {
            raw: m.id.clone(),
            mailbox: mbox.clone(),
        };
        if m.removed.is_some() {
            removed.push(mref);
        } else {
            flag_changes.push((mref.clone(), flags_of(m)));
            added.push(mref);
        }
    }

    let next = page
        .delta_link
        .or(page.next_link)
        .unwrap_or_default()
        .into_bytes();

    Ok(MailboxDelta {
        added,
        removed,
        flag_changes,
        next_cursor: SyncCursor { opaque: next },
    })
}

/// `GET /me/messages/{id}/$value` per ref → the raw RFC 5322 MIME.
pub fn fetch_raw<T: Transport>(
    client: &GraphClient<'_, T>,
    refs: &[MessageRef],
) -> Result<Vec<RawMessage>> {
    let mut out = Vec::with_capacity(refs.len());
    for r in refs {
        let raw = client.get_bytes(&format!("/me/messages/{}/$value", r.raw))?;
        out.push(RawMessage {
            message_ref: r.clone(),
            raw,
            msg_flags: Vec::new(),
            internaldate: None,
        });
    }
    Ok(out)
}

/// Build the PATCH body for a flag mutation. `$Focused`/`$Other` keyword adds map to
/// the Graph `inferenceClassification` write (Focused-Inbox sync, plan §2.5).
fn patch_body(add: &[Flag], remove: &[Flag]) -> serde_json::Value {
    let mut body = serde_json::Map::new();
    if add.contains(&Flag::Seen) {
        body.insert("isRead".into(), serde_json::Value::Bool(true));
    }
    if remove.contains(&Flag::Seen) {
        body.insert("isRead".into(), serde_json::Value::Bool(false));
    }
    if add.contains(&Flag::Flagged) {
        body.insert(
            "flag".into(),
            serde_json::json!({ "flagStatus": "flagged" }),
        );
    }
    if remove.contains(&Flag::Flagged) {
        body.insert(
            "flag".into(),
            serde_json::json!({ "flagStatus": "notFlagged" }),
        );
    }
    for f in add {
        if let Flag::Keyword(k) = f {
            if k == KEYWORD_FOCUSED {
                body.insert("inferenceClassification".into(), "focused".into());
            } else if k == KEYWORD_OTHER {
                body.insert("inferenceClassification".into(), "other".into());
            }
        }
    }
    serde_json::Value::Object(body)
}

/// `PATCH /me/messages/{id}` per ref → apply a flag mutation.
pub fn store_flags<T: Transport>(
    client: &GraphClient<'_, T>,
    refs: &[MessageRef],
    add: &[Flag],
    remove: &[Flag],
) -> Result<()> {
    let body = patch_body(add, remove);
    if body.as_object().map(|o| o.is_empty()).unwrap_or(true) {
        // Nothing Graph-expressible (e.g. only IMAP-only flags) — a no-op success,
        // never a failure (the engine keeps its own copy).
        return Ok(());
    }
    for r in refs {
        client.patch_ignore(&format!("/me/messages/{}", r.raw), &body)?;
    }
    Ok(())
}

/// `POST /me/messages/{id}/move` per ref → move to `to` (a Graph folder id/name).
pub fn move_messages<T: Transport>(
    client: &GraphClient<'_, T>,
    refs: &[MessageRef],
    to: &MailboxRef,
) -> Result<()> {
    let body = serde_json::json!({ "destinationId": to.name });
    for r in refs {
        client.post_ignore(&format!("/me/messages/{}/move", r.raw), &body)?;
    }
    Ok(())
}

/// `POST /me/sendMail` with a base64 RFC 5322 MIME body (Graph's `text/plain` MIME
/// send path). Returns a synthetic ref — `sendMail` yields no server id.
pub fn submit<T: Transport>(
    client: &GraphClient<'_, T>,
    mbox: &MailboxRef,
    raw: &[u8],
) -> Result<MessageRef> {
    let b64 = base64::engine::general_purpose::STANDARD.encode(raw);
    client.post_raw("/me/sendMail", b64.into_bytes(), "text/plain")?;
    Ok(MessageRef {
        raw: format!("sent:{}", mbox.name),
        mailbox: mbox.clone(),
    })
}

/// A best-effort change poll. Graph has no server-push socket in this ABI; the host
/// adapter drives `sync_mailbox` on its cadence, so the poll is intentionally empty
/// (plan §2.1: the WIT `poll-changes` replaces the WASI async stream).
pub fn poll_changes() -> Vec<ChangeEvent> {
    Vec::new()
}
