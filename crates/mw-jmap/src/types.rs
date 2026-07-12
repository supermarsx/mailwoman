use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// RFC 8620 §2 — Session resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    #[serde(default)]
    pub capabilities: HashMap<String, Value>,
    #[serde(default)]
    pub accounts: HashMap<String, Account>,
    #[serde(default)]
    pub primary_accounts: HashMap<String, String>,
    #[serde(default)]
    pub username: String,
    pub api_url: String,
    #[serde(default)]
    pub download_url: String,
    #[serde(default)]
    pub upload_url: String,
    #[serde(default)]
    pub event_source_url: String,
    #[serde(default)]
    pub state: String,
}

impl Session {
    /// The primary mail account id, falling back to the first account.
    pub fn primary_mail_account(&self) -> Option<&str> {
        self.primary_accounts
            .get("urn:ietf:params:jmap:mail")
            .map(String::as_str)
            .or_else(|| self.accounts.keys().next().map(String::as_str))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub is_personal: bool,
    #[serde(default)]
    pub is_read_only: bool,
    #[serde(default)]
    pub account_capabilities: HashMap<String, Value>,
}

/// RFC 8620 §3.2 — a method call / response triple `[name, args, callId]`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invocation(pub String, pub Value, pub String);

impl Invocation {
    pub fn new(name: impl Into<String>, args: Value, call_id: impl Into<String>) -> Self {
        Self(name.into(), args, call_id.into())
    }
    pub fn name(&self) -> &str {
        &self.0
    }
    pub fn args(&self) -> &Value {
        &self.1
    }
    pub fn call_id(&self) -> &str {
        &self.2
    }
}

/// RFC 8620 §3.3 — Request object.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub using: Vec<String>,
    pub method_calls: Vec<Invocation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_ids: Option<HashMap<String, String>>,
}

impl Request {
    pub fn mail(method_calls: Vec<Invocation>) -> Self {
        Self {
            using: vec![
                "urn:ietf:params:jmap:core".into(),
                "urn:ietf:params:jmap:mail".into(),
                "urn:ietf:params:jmap:submission".into(),
            ],
            method_calls,
            created_ids: None,
        }
    }
}

/// RFC 8620 §3.4 — Response object.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub method_responses: Vec<Invocation>,
    #[serde(default)]
    pub session_state: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_ids: Option<HashMap<String, String>>,
}

impl Response {
    /// First response invocation matching `name`, if any.
    pub fn find(&self, name: &str) -> Option<&Invocation> {
        self.method_responses.iter().find(|i| i.name() == name)
    }
}

/// RFC 8621 §2 — Mailbox.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mailbox {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub parent_id: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub sort_order: u32,
    #[serde(default)]
    pub total_emails: u64,
    #[serde(default)]
    pub unread_emails: u64,
    #[serde(default)]
    pub total_threads: u64,
    #[serde(default)]
    pub unread_threads: u64,
}

/// RFC 8621 §4.1.2.3 — EmailAddress.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmailAddress {
    #[serde(default)]
    pub name: Option<String>,
    pub email: String,
}

/// RFC 8621 §4.1.4 — body part metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmailBodyPart {
    #[serde(default)]
    pub part_id: Option<String>,
    #[serde(default)]
    pub blob_id: Option<String>,
    #[serde(default)]
    pub size: u64,
    #[serde(default)]
    pub r#type: Option<String>,
    #[serde(default)]
    pub charset: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub cid: Option<String>,
    #[serde(default)]
    pub disposition: Option<String>,
}

/// RFC 8621 §4.1.4 — fetched body value.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmailBodyValue {
    pub value: String,
    #[serde(default)]
    pub is_encoding_problem: bool,
    #[serde(default)]
    pub is_truncated: bool,
}

/// RFC 8621 §4 — Email (subset of properties V0 uses).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Email {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub blob_id: Option<String>,
    #[serde(default)]
    pub thread_id: Option<String>,
    #[serde(default)]
    pub mailbox_ids: HashMap<String, bool>,
    #[serde(default)]
    pub keywords: HashMap<String, bool>,
    #[serde(default)]
    pub from: Option<Vec<EmailAddress>>,
    #[serde(default)]
    pub to: Option<Vec<EmailAddress>>,
    #[serde(default)]
    pub cc: Option<Vec<EmailAddress>>,
    #[serde(default)]
    pub bcc: Option<Vec<EmailAddress>>,
    #[serde(default)]
    pub reply_to: Option<Vec<EmailAddress>>,
    #[serde(default)]
    pub subject: Option<String>,
    #[serde(default)]
    pub received_at: Option<String>,
    #[serde(default)]
    pub sent_at: Option<String>,
    #[serde(default)]
    pub preview: Option<String>,
    #[serde(default)]
    pub has_attachment: bool,
    #[serde(default)]
    pub size: u64,
    #[serde(default)]
    pub body_values: HashMap<String, EmailBodyValue>,
    #[serde(default)]
    pub text_body: Vec<EmailBodyPart>,
    #[serde(default)]
    pub html_body: Vec<EmailBodyPart>,
    /// Non-inline attachment parts (RFC 8621 §4.1.4). The engine fills each
    /// part's `blobId` so the web app can download it via `downloadUrl`.
    #[serde(default)]
    pub attachments: Vec<EmailBodyPart>,
}

/// RFC 8620 §5.3 — SetError.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetError {
    pub r#type: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn session_round_trip_and_primary_account() {
        let raw = json!({
            "capabilities": {"urn:ietf:params:jmap:core": {"maxSizeUpload": 50_000_000}},
            "accounts": {"a1": {"name": "test@example.org", "isPersonal": true, "isReadOnly": false, "accountCapabilities": {}}},
            "primaryAccounts": {"urn:ietf:params:jmap:mail": "a1"},
            "username": "test@example.org",
            "apiUrl": "https://mail.example.org/jmap",
            "downloadUrl": "https://mail.example.org/download/{accountId}/{blobId}/{name}?type={type}",
            "uploadUrl": "https://mail.example.org/upload/{accountId}",
            "eventSourceUrl": "https://mail.example.org/events",
            "state": "s0"
        });
        let s: Session = serde_json::from_value(raw.clone()).unwrap();
        assert_eq!(s.primary_mail_account(), Some("a1"));
        assert_eq!(s.api_url, "https://mail.example.org/jmap");
        let back = serde_json::to_value(&s).unwrap();
        assert_eq!(back["apiUrl"], raw["apiUrl"]);
        assert_eq!(back["primaryAccounts"], raw["primaryAccounts"]);
    }

    #[test]
    fn invocation_is_a_json_triple() {
        let inv = Invocation::new("Mailbox/get", json!({"accountId": "a1"}), "c0");
        let v = serde_json::to_value(&inv).unwrap();
        assert_eq!(v, json!(["Mailbox/get", {"accountId": "a1"}, "c0"]));
        let round: Invocation = serde_json::from_value(v).unwrap();
        assert_eq!(round.name(), "Mailbox/get");
        assert_eq!(round.call_id(), "c0");
    }

    #[test]
    fn request_envelope_shape() {
        let req = Request::mail(vec![Invocation::new(
            "Email/query",
            json!({"accountId": "a1", "filter": {"inMailbox": "mb1"}}),
            "q0",
        )]);
        let v = serde_json::to_value(&req).unwrap();
        assert_eq!(v["using"][0], "urn:ietf:params:jmap:core");
        assert_eq!(v["methodCalls"][0][0], "Email/query");
        assert!(v.get("createdIds").is_none());
    }

    #[test]
    fn email_parses_bodies_and_defaults() {
        let raw = json!({
            "id": "e1",
            "mailboxIds": {"mb1": true},
            "from": [{"name": "Anna", "email": "anna@example.org"}],
            "subject": "Hello",
            "receivedAt": "2026-07-12T00:00:00Z",
            "bodyValues": {"1": {"value": "<p>hi</p>", "isTruncated": false}},
            "htmlBody": [{"partId": "1", "type": "text/html", "size": 9}]
        });
        let e: Email = serde_json::from_value(raw).unwrap();
        assert_eq!(e.subject.as_deref(), Some("Hello"));
        assert_eq!(e.html_body[0].part_id.as_deref(), Some("1"));
        assert_eq!(e.body_values["1"].value, "<p>hi</p>");
        assert!(!e.has_attachment);
    }

    #[test]
    fn response_find() {
        let resp = Response {
            method_responses: vec![
                Invocation::new("Mailbox/get", json!({"list": []}), "c0"),
                Invocation::new("Email/query", json!({"ids": []}), "c1"),
            ],
            session_state: "s1".into(),
            created_ids: None,
        };
        assert!(resp.find("Email/query").is_some());
        assert!(resp.find("Email/set").is_none());
    }
}
