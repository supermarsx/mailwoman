//! Gmail REST API v1 wire types (serde) + URL builders (plan §3 e12).
//!
//! Pure data: no I/O. [`crate::backend`] drives the [`crate::backend::Transport`]
//! with these URLs and parses the responses into these DTOs. Host-testable.
//!
//! Endpoint base: `https://gmail.googleapis.com/gmail/v1/users/me`. Auth is a
//! `Bearer` token the host mints (never in the guest). The guest reaches only
//! `gmail.googleapis.com` (manifest `net_allowlist`).

use serde::Deserialize;

/// The Gmail REST base for the bound account (`me` = the OAuth-bound user).
pub const BASE: &str = "https://gmail.googleapis.com/gmail/v1/users/me";

// ── URL builders ────────────────────────────────────────────────────────────────

#[must_use]
pub fn url_profile() -> String {
    format!("{BASE}/profile")
}

#[must_use]
pub fn url_labels() -> String {
    format!("{BASE}/labels")
}

/// List message ids carrying `label_id` (a page of the mailbox baseline).
#[must_use]
pub fn url_messages_list(label_id: &str) -> String {
    format!("{BASE}/messages?labelIds={}&maxResults=500", enc(label_id))
}

/// Get one message's metadata (labels/date) — cheap, no body.
#[must_use]
pub fn url_message_meta(id: &str) -> String {
    format!(
        "{BASE}/messages/{}?format=metadata&metadataHeaders=Message-ID",
        enc(id)
    )
}

/// Get one message as raw RFC822 (base64url in the `raw` field).
#[must_use]
pub fn url_message_raw(id: &str) -> String {
    format!("{BASE}/messages/{}?format=raw", enc(id))
}

/// List every message id (the synthetic "All Mail" mailbox — no label filter).
#[must_use]
pub fn url_messages_list_all() -> String {
    format!("{BASE}/messages?maxResults=500")
}

/// Incremental history since `start_history_id`, scoped to `label_id`.
#[must_use]
pub fn url_history(start_history_id: &str, label_id: &str) -> String {
    format!(
        "{BASE}/history?startHistoryId={}&labelId={}&maxResults=500",
        enc(start_history_id),
        enc(label_id)
    )
}

/// Incremental history since `start_history_id`, unscoped ("All Mail").
#[must_use]
pub fn url_history_all(start_history_id: &str) -> String {
    format!(
        "{BASE}/history?startHistoryId={}&maxResults=500",
        enc(start_history_id)
    )
}

/// `messages.modify` — add/remove label ids on a message.
#[must_use]
pub fn url_message_modify(id: &str) -> String {
    format!("{BASE}/messages/{}/modify", enc(id))
}

/// `messages.send` — send a raw RFC822 message.
#[must_use]
pub fn url_messages_send() -> String {
    format!("{BASE}/messages/send")
}

/// `drafts.create` — store a raw RFC822 draft.
#[must_use]
pub fn url_drafts_create() -> String {
    format!("{BASE}/drafts")
}

/// Minimal, allocation-light percent-encoding for the few characters that appear in
/// Gmail ids / label ids used in a query string (`/`, space, `#`, `?`, `&`, `+`).
fn enc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

// ── Response DTOs ────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct Profile {
    #[serde(rename = "historyId")]
    pub history_id: String,
}

#[derive(Debug, Deserialize)]
pub struct LabelsList {
    #[serde(default)]
    pub labels: Vec<Label>,
}

#[derive(Debug, Deserialize)]
pub struct Label {
    pub id: String,
    #[serde(default)]
    pub name: String,
    /// "system" | "user".
    #[serde(rename = "type", default)]
    pub label_type: String,
    #[serde(rename = "messagesTotal", default)]
    pub messages_total: u32,
    #[serde(rename = "messagesUnread", default)]
    pub messages_unread: u32,
}

impl Label {
    #[must_use]
    pub fn is_user(&self) -> bool {
        self.label_type == "user"
    }
}

#[derive(Debug, Deserialize)]
pub struct MessagesList {
    #[serde(default)]
    pub messages: Vec<MessageId>,
}

#[derive(Debug, Deserialize)]
pub struct MessageId {
    pub id: String,
}

/// A message object as returned by `messages.get` (metadata or raw) and embedded in
/// history records.
#[derive(Debug, Deserialize)]
pub struct Message {
    pub id: String,
    #[serde(rename = "labelIds", default)]
    pub label_ids: Vec<String>,
    #[serde(rename = "internalDate", default)]
    pub internal_date: Option<String>,
    /// base64url RFC822, present only for `format=raw`.
    #[serde(default)]
    pub raw: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct HistoryList {
    #[serde(default)]
    pub history: Vec<HistoryRecord>,
    /// The newest history id at the time of the response — the next cursor.
    #[serde(rename = "historyId", default)]
    pub history_id: String,
}

#[derive(Debug, Deserialize)]
pub struct HistoryRecord {
    #[serde(rename = "messagesAdded", default)]
    pub messages_added: Vec<HistoryMessage>,
    #[serde(rename = "messagesDeleted", default)]
    pub messages_deleted: Vec<HistoryMessage>,
    #[serde(rename = "labelsAdded", default)]
    pub labels_added: Vec<HistoryMessage>,
    #[serde(rename = "labelsRemoved", default)]
    pub labels_removed: Vec<HistoryMessage>,
}

#[derive(Debug, Deserialize)]
pub struct HistoryMessage {
    pub message: Message,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_labels_list() {
        let j = r#"{"labels":[
            {"id":"INBOX","name":"INBOX","type":"system","messagesTotal":3,"messagesUnread":1},
            {"id":"Label_9","name":"Receipts","type":"user","messagesTotal":10}
        ]}"#;
        let l: LabelsList = serde_json::from_str(j).unwrap();
        assert_eq!(l.labels.len(), 2);
        assert!(!l.labels[0].is_user());
        assert!(l.labels[1].is_user());
        assert_eq!(l.labels[0].messages_unread, 1);
    }

    #[test]
    fn parses_history_delta() {
        let j = r#"{"history":[
            {"messagesAdded":[{"message":{"id":"m5","labelIds":["INBOX","UNREAD"]}}]},
            {"labelsAdded":[{"message":{"id":"m1","labelIds":["INBOX","STARRED"]},"labelIds":["STARRED"]}]}
        ],"historyId":"10005"}"#;
        let h: HistoryList = serde_json::from_str(j).unwrap();
        assert_eq!(h.history_id, "10005");
        assert_eq!(h.history[0].messages_added[0].message.id, "m5");
        assert_eq!(h.history[1].labels_added[0].message.id, "m1");
    }

    #[test]
    fn url_builders_scope_to_label() {
        assert!(url_history("42", "INBOX").contains("startHistoryId=42"));
        assert!(url_history("42", "INBOX").contains("labelId=INBOX"));
        assert!(url_message_raw("m1").ends_with("m1?format=raw"));
    }
}
