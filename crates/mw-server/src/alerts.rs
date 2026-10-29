//! Security alerts to a mailbox user (t32-ext §14.3 "Alerts", §15 M4; e57 seam).
//!
//! [`emit`] is the one call for "tell the account holder": a sign-in from a new
//! device, a 2FA change, a new delegate, a new forwarding rule, a new API key, a
//! sign-in from a new country. The alert is recorded in the audit log
//! (`category = "alert"`, target `account:<id>`), which is where the user's
//! activity list reads it.
//!
//! Inbox delivery uses the engine backend APPEND or guarded proxy JMAP import.
//! Delivery runs with a bounded timeout; refusal retains the durable activity
//! event. The engine refresh publishes its ordinary mail change notification.

use serde_json::{Value, json};

use crate::AppState;
use crate::audit::{SecurityEvent, security_event};

/// `audit_log.category` of a user alert.
pub(crate) const CATEGORY_ALERT: &str = "alert";

/// What the alert is about.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlertKind {
    /// A sign-in from a device this account has not used before.
    NewDevice,
    /// A factor was added or removed, or recovery codes were regenerated.
    TwoFactorChanged,
    /// Someone was given access to this mailbox.
    DelegateAdded,
    /// A rule now forwards or redirects mail.
    ForwardingAdded,
    /// An API key was created for this account.
    ApiKeyCreated,
    /// A sign-in from a country this account has not signed in from (GeoIP on).
    NewCountry,
}

impl AlertKind {
    #[cfg(test)]
    pub const ALL: [AlertKind; 6] = [
        AlertKind::NewDevice,
        AlertKind::TwoFactorChanged,
        AlertKind::DelegateAdded,
        AlertKind::ForwardingAdded,
        AlertKind::ApiKeyCreated,
        AlertKind::NewCountry,
    ];

    /// The wire id (`detail.alert` in the audit row, the web client's key).
    pub fn as_str(self) -> &'static str {
        match self {
            AlertKind::NewDevice => "newDevice",
            AlertKind::TwoFactorChanged => "twoFactorChanged",
            AlertKind::DelegateAdded => "delegateAdded",
            AlertKind::ForwardingAdded => "forwardingAdded",
            AlertKind::ApiKeyCreated => "apiKeyCreated",
            AlertKind::NewCountry => "newCountry",
        }
    }
}

/// Raise `kind` for `account_id`. `detail` is context for the user (device label,
/// country, delegate kind); it is redacted like every audit detail, so it carries
/// no address or secret. Returns the audit row id, or `None` when the write failed
/// (logged; the caller's action is not undone).
pub(crate) async fn emit(
    state: &AppState,
    account_id: &str,
    kind: AlertKind,
    detail: Value,
) -> Option<String> {
    let mut body = json!({ "alert": kind.as_str() });
    if let (Some(b), Value::Object(extra)) = (body.as_object_mut(), detail) {
        for (k, v) in extra {
            if k != "alert" {
                b.insert(k, v);
            }
        }
    }
    let id = security_event(
        state,
        SecurityEvent::system("user-alert")
            .target(format!("account:{account_id}"))
            .detail(body)
            .category(CATEGORY_ALERT),
    )
    .await;
    let state = state.clone();
    let account = account_id.to_owned();
    tokio::spawn(async move {
        match tokio::time::timeout(
            std::time::Duration::from_secs(30),
            deliver(&state, &account, kind),
        )
        .await
        {
            Ok(Ok(())) => {}
            Ok(Err(e)) => tracing::warn!("security alert delivery failed; activity retained: {e}"),
            Err(_) => tracing::warn!("security alert delivery timed out; activity retained"),
        }
    });
    id
}

/// The mailbox grants one JMAP exchange made: for each `MailboxRights/set` call
/// that gave an identifier rights (a non-empty `rights`; null revokes and `""`
/// clears) and was answered with `updated`, the rights granted. Feeds
/// [`AlertKind::DelegateAdded`]; the identifier is not returned, so the alert
/// never names the delegate.
pub(crate) fn delegation_grants(request: &Value, response: &Value) -> Vec<String> {
    let Some(calls) = request.get("methodCalls").and_then(Value::as_array) else {
        return Vec::new();
    };
    let answers = response
        .get("methodResponses")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default();
    calls
        .iter()
        .filter(|call| call.get(0).and_then(Value::as_str) == Some("MailboxRights/set"))
        .filter_map(|call| {
            let rights = call.get(1)?.get("rights")?.as_str()?.trim();
            if rights.is_empty() {
                return None;
            }
            let id = call.get(2)?.as_str()?;
            answers
                .iter()
                .any(|answer| {
                    answer.get(0).and_then(Value::as_str) == Some("MailboxRights/set")
                        && answer.get(2).and_then(Value::as_str) == Some(id)
                        && answer
                            .get(1)
                            .and_then(|r| r.get("updated"))
                            .and_then(Value::as_object)
                            .is_some_and(|u| !u.is_empty())
                })
                .then(|| rights.to_string())
        })
        .collect()
}

fn message(notice: &crate::server_text::Notice) -> Vec<u8> {
    use base64::Engine as _;
    let mut words = Vec::new();
    let mut chunk = String::new();
    for ch in notice.subject.chars() {
        if chunk.len() + ch.len_utf8() > 42 {
            words.push(format!(
                "=?UTF-8?B?{}?=",
                base64::engine::general_purpose::STANDARD.encode(chunk.as_bytes())
            ));
            chunk.clear();
        }
        chunk.push(ch);
    }
    if !chunk.is_empty() {
        words.push(format!(
            "=?UTF-8?B?{}?=",
            base64::engine::general_purpose::STANDARD.encode(chunk.as_bytes())
        ));
    }
    let subject = words.join("\r\n ");
    format!("From: Mailwoman Security <security@mailwoman.invalid>\r\nSubject: {subject}\r\nDate: {}\r\nMessage-ID: <{}@mailwoman.invalid>\r\nAuto-Submitted: auto-generated\r\nX-Auto-Response-Suppress: All\r\nMIME-Version: 1.0\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Transfer-Encoding: 8bit\r\nContent-Language: {}\r\n\r\n{}\r\n\r\n{}\r\n",chrono::Utc::now().to_rfc2822(),uuid::Uuid::new_v4(),notice.locale,notice.description,notice.guidance).into_bytes()
}
async fn deliver(state: &AppState, account: &str, kind: AlertKind) -> Result<(), String> {
    let notice = crate::server_text::account_notice(state, account, kind).await;
    let raw = message(&notice);
    if let Some(engine) = &state.engine {
        return engine
            .deliver_security_notice(account, &raw)
            .await
            .map_err(|e| e.to_string());
    }
    let sessions = state
        .store
        .sessions_by_account(account)
        .await
        .map_err(|e| e.to_string())?;
    let mut session = sessions
        .into_iter()
        .next()
        .ok_or("account has no delivery session")?;
    crate::domain_upgrade::refresh_session(state, &mut session)
        .await
        .map_err(|_| "delivery account authority changed")?;
    deliver_proxy(&state.security, &session, raw).await
}

async fn deliver_proxy(
    security: &crate::SecurityConfig,
    session: &mw_store::Session,
    raw: Vec<u8>,
) -> Result<(), String> {
    let account = &session.account_id;
    let upstream = crate::upstream_session(security, &session.jmap_url, &session.credentials)
        .await
        .map_err(|_| "upstream session refused")?;
    let api = crate::resolve_api_url(&session.jmap_url, &upstream.api_url);
    let client = crate::upstream_client(security, &session.jmap_url, &api, &session.credentials)
        .await
        .map_err(|_| "upstream policy refused")?;
    let call = json!({"using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:mail"],"methodCalls":[["Mailbox/get",{"accountId":account},"inbox"]]});
    let (status, response) = client
        .request_raw(
            &api,
            serde_json::to_vec(&call).map_err(|e| e.to_string())?.into(),
        )
        .await
        .map_err(|e| e.to_string())?;
    if !status.is_success() {
        return Err("mailbox discovery failed".into());
    }
    let response: Value = serde_json::from_slice(&response).map_err(|e| e.to_string())?;
    let inbox = response["methodResponses"][0][1]["list"]
        .as_array()
        .and_then(|rows| rows.iter().find(|row| row["role"] == "inbox"))
        .and_then(|row| row["id"].as_str())
        .ok_or("no upstream inbox")?;
    let account_segment: String = account
        .bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() || b"-._~".contains(&b) {
                (b as char).to_string()
            } else {
                format!("%{b:02X}")
            }
        })
        .collect();
    let upload = crate::resolve_api_url(
        &session.jmap_url,
        &upstream.upload_url.replace("{accountId}", &account_segment),
    );
    let uploader =
        crate::upstream_client(security, &session.jmap_url, &upload, &session.credentials)
            .await
            .map_err(|_| "upload policy refused")?;
    let (status, _, bytes) = uploader
        .post_bytes(&upload, "message/rfc822", raw.into())
        .await
        .map_err(|e| e.to_string())?;
    if !status.is_success() {
        return Err("alert upload failed".into());
    }
    let uploaded: Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    let blob = uploaded["blobId"].as_str().ok_or("upload missing blob")?;
    let import = json!({"using":["urn:ietf:params:jmap:core","urn:ietf:params:jmap:mail"],"methodCalls":[["Email/import",{"accountId":account,"emails":{"alert":{"blobId":blob,"mailboxIds":{inbox:true}}}},"alert"]]});
    let (status, bytes) = client
        .request_raw(
            &api,
            serde_json::to_vec(&import)
                .map_err(|e| e.to_string())?
                .into(),
        )
        .await
        .map_err(|e| e.to_string())?;
    let result: Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    if !status.is_success() || result["methodResponses"][0][1]["created"]["alert"].is_null() {
        return Err("alert import failed".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    #[tokio::test]
    async fn proxy_notice_uploads_localized_message_and_imports_into_real_inbox() {
        use axum::{
            Json, Router,
            routing::{get, post},
        };
        use std::sync::{Arc, Mutex};
        let uploaded = Arc::new(Mutex::new(Vec::<u8>::new()));
        let capture = uploaded.clone();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let origin = format!("http://{}", listener.local_addr().unwrap());
        let router = Router::new()
            .route("/.well-known/jmap",get(||async {Json(json!({"apiUrl":"/jmap","uploadUrl":"/upload/{accountId}","username":"alice"}))}))
            .route("/upload/{account}",post(move |body:axum::body::Bytes| { let capture=capture.clone();async move { *capture.lock().unwrap()=body.to_vec();Json(json!({"blobId":"alert-blob"})) }}))
            .route("/jmap",post(|Json(body):Json<Value>|async move {
                if body["methodCalls"][0][0] == "Mailbox/get" { Json(json!({"methodResponses":[["Mailbox/get",{"list":[{"id":"inbox-id","role":"inbox"}]},"inbox"]]})) }
                else { assert_eq!(body["methodCalls"][0][1]["emails"]["alert"]["mailboxIds"]["inbox-id"],true);Json(json!({"methodResponses":[["Email/import",{"created":{"alert":{"id":"delivered"}}},"alert"]]})) }
            }));
        let task = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
        let store = mw_store::Store::open_in_memory(mw_store::ServerKey::generate())
            .await
            .unwrap();
        let creds = mw_store::Credentials {
            username: "alice".into(),
            password: "upstream-secret".into(),
        };
        let id = store
            .create_session("alice", "alice", &origin, &format!("{origin}/jmap"), &creds)
            .await
            .unwrap();
        let session = store.get_session(&id).await.unwrap();
        let config = crate::SecurityConfig {
            jmap_upstreams: Some(vec![origin]),
            ..Default::default()
        };
        deliver_proxy(
            &config,
            &session,
            message(&crate::server_text::notice("es", AlertKind::ApiKeyCreated)),
        )
        .await
        .unwrap();
        let raw = String::from_utf8(uploaded.lock().unwrap().clone()).unwrap();
        assert!(raw.contains("Content-Language: es"));
        assert!(raw.contains("clave de aplicación"));
        assert!(!raw.contains("upstream-secret"));
        assert!(raw.contains("Auto-Submitted: auto-generated"));
        task.abort();
    }

    #[test]
    fn only_answered_grants_count_as_delegations() {
        let request = json!({"methodCalls":[
            ["MailboxRights/set",{"mailboxId":"m","identifier":"bob","rights":"lrs"},"grant"],
            ["MailboxRights/set",{"mailboxId":"m","identifier":"eve","rights":"lr"},"refused"],
            ["MailboxRights/set",{"mailboxId":"m","identifier":"bob"},"revoke"],
            ["MailboxRights/set",{"mailboxId":"m","identifier":"bob","rights":""},"clear"],
            ["Mailbox/get",{},"other"]
        ]});
        let response = json!({"methodResponses":[
            ["MailboxRights/set",{"updated":{"bob":null}},"grant"],
            ["MailboxRights/set",{"notUpdated":{"eve":{"type":"forbidden"}}},"refused"],
            ["MailboxRights/set",{"updated":{"bob":null}},"revoke"],
            ["MailboxRights/set",{"updated":{"bob":null}},"clear"],
            ["Mailbox/get",{"list":[]},"other"]
        ]});
        assert_eq!(delegation_grants(&request, &response), ["lrs"]);
        assert!(delegation_grants(&json!({}), &response).is_empty());
        assert!(delegation_grants(&request, &json!({})).is_empty());
    }

    #[test]
    fn wire_ids_are_distinct() {
        let ids: BTreeSet<_> = AlertKind::ALL.iter().map(|k| k.as_str()).collect();
        assert_eq!(ids.len(), AlertKind::ALL.len());
    }
}
