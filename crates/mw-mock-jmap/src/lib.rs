//! A minimal, self-contained JMAP server sufficient to exercise the
//! Mailwoman proxy + sanitizer end-to-end in tests and CI. In-memory,
//! mutable, deterministic. NOT a real JMAP implementation — it mirrors
//! only the surface the V0 web client uses (SPEC §27 V0, plan §2).

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::Router;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use base64::Engine as _;
use serde_json::{Value, json};

pub const USER: &str = "testuser@example.org";
pub const PASS: &str = "testpass";
pub const ACCOUNT_ID: &str = "acct-1";

/// A hostile HTML message body used to prove the sanitizer is wired.
/// Contains a sentinel script, an event handler, and a tracking pixel.
pub const HOSTILE_HTML: &str = concat!(
    "<h1>Invoice</h1><p>Please review.</p>",
    "<script>window.__mw_pwned = true;</script>",
    "<img src=\"https://tracker.evil.example/pixel.gif?id=42\" width=\"1\" height=\"1\">",
    "<a href=\"javascript:alert(1)\" onclick=\"steal()\">click</a>",
);

#[derive(Clone)]
struct MailStore {
    inner: Arc<Mutex<Inner>>,
}

struct Inner {
    // email id -> (email json)
    emails: HashMap<String, Value>,
    // ordering for query determinism (newest first)
    order: Vec<String>,
    next_id: u64,
}

impl MailStore {
    fn seeded() -> Self {
        let mut emails = HashMap::new();
        let mut order = Vec::new();

        let seed = [
            (
                "e1",
                "Anna Ng",
                "anna@example.org",
                "Welcome to Mailwoman",
                "<p>Hi there, welcome aboard!</p>",
                "inbox",
            ),
            (
                "e2",
                "Billing",
                "billing@shop.example",
                "Your invoice is ready",
                HOSTILE_HTML,
                "inbox",
            ),
            (
                "e3",
                "Carlos",
                "carlos@example.org",
                "Lunch tomorrow?",
                "<p>Are you free at noon?</p>",
                "inbox",
            ),
        ];

        for (id, name, addr, subject, html, mailbox) in seed {
            let mailbox_id = if mailbox == "inbox" {
                "mb-inbox"
            } else {
                "mb-sent"
            };
            emails.insert(
                id.to_string(),
                json!({
                    "id": id,
                    "blobId": format!("blob-{id}"),
                    "threadId": format!("t-{id}"),
                    "mailboxIds": { mailbox_id: true },
                    "keywords": {},
                    "from": [{ "name": name, "email": addr }],
                    "to": [{ "name": "Test User", "email": USER }],
                    "subject": subject,
                    "receivedAt": "2026-07-12T09:00:00Z",
                    "sentAt": "2026-07-12T09:00:00Z",
                    "preview": "…",
                    "hasAttachment": false,
                    "size": html.len(),
                    "bodyValues": { "1": { "value": html, "isTruncated": false } },
                    "htmlBody": [{ "partId": "1", "type": "text/html", "size": html.len() }],
                    "textBody": [{ "partId": "1", "type": "text/html", "size": html.len() }]
                }),
            );
            order.push(id.to_string());
        }

        Self {
            inner: Arc::new(Mutex::new(Inner {
                emails,
                order,
                next_id: 100,
            })),
        }
    }
}

fn check_auth(headers: &HeaderMap) -> bool {
    let Some(v) = headers.get("authorization").and_then(|v| v.to_str().ok()) else {
        return false;
    };
    let Some(b64) = v.strip_prefix("Basic ") else {
        return false;
    };
    let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(b64) else {
        return false;
    };
    decoded == format!("{USER}:{PASS}").into_bytes()
}

pub fn router() -> Router {
    let store = MailStore::seeded();
    Router::new()
        .route("/.well-known/jmap", get(session))
        .route("/jmap/session", get(session))
        .route("/jmap", post(api))
        .route("/jmap/download/{accountId}/{blobId}/{name}", get(download))
        .route("/healthz", get(|| async { "ok" }))
        .with_state(store)
}

async fn session(headers: HeaderMap) -> impl IntoResponse {
    if !check_auth(&headers) {
        return (StatusCode::UNAUTHORIZED, "unauthorized").into_response();
    }
    axum::Json(json!({
        "capabilities": {
            "urn:ietf:params:jmap:core": { "maxSizeUpload": 50_000_000, "maxConcurrentRequests": 4 },
            "urn:ietf:params:jmap:mail": {},
            "urn:ietf:params:jmap:submission": {},
            "urn:mailwoman:crypto": {},
            "urn:mailwoman:security": {}
        },
        "accounts": {
            ACCOUNT_ID: { "name": USER, "isPersonal": true, "isReadOnly": false, "accountCapabilities": {
                "urn:mailwoman:crypto": {},
                "urn:mailwoman:security": {}
            } }
        },
        "primaryAccounts": {
            "urn:ietf:params:jmap:mail": ACCOUNT_ID,
            "urn:ietf:params:jmap:submission": ACCOUNT_ID,
            "urn:mailwoman:crypto": ACCOUNT_ID,
            "urn:mailwoman:security": ACCOUNT_ID
        },
        "username": USER,
        "apiUrl": "/jmap",
        "downloadUrl": "/jmap/download/{accountId}/{blobId}/{name}",
        "uploadUrl": "/jmap/upload/{accountId}",
        "eventSourceUrl": "/jmap/eventsource",
        "state": "session-0"
    }))
    .into_response()
}

async fn api(
    State(store): State<MailStore>,
    headers: HeaderMap,
    body: String,
) -> impl IntoResponse {
    if !check_auth(&headers) {
        return (StatusCode::UNAUTHORIZED, "unauthorized").into_response();
    }
    let req: Value = match serde_json::from_str(&body) {
        Ok(v) => v,
        Err(e) => return (StatusCode::BAD_REQUEST, format!("bad json: {e}")).into_response(),
    };
    let empty = vec![];
    let calls = req
        .get("methodCalls")
        .and_then(Value::as_array)
        .unwrap_or(&empty);
    let mut responses: Vec<Value> = Vec::new();

    for call in calls {
        let Some(arr) = call.as_array() else { continue };
        if arr.len() < 3 {
            continue;
        }
        let name = arr[0].as_str().unwrap_or_default();
        let call_id = arr[2].as_str().unwrap_or("c0");
        // Resolve JMAP result references (RFC 8620 §3.7) before dispatch: an arg
        // key like "#ids" -> {resultOf, name, path} is replaced by the value at
        // `path` inside the referenced prior response, stored under "ids". The web
        // client chains Email/query -> Email/get this way, so the mock must too.
        let mut args = arr[1].clone();
        resolve_references(&mut args, &responses);
        let resp_args = dispatch(&store, name, &args);
        // RFC 8620 §3.6.1: a method-level error is returned as ["error", obj, callId],
        // NOT ["<Method>", {type}, callId]. dispatch() tags unimplemented methods with
        // `type: "unknownMethod"`; surface those under the "error" name so clients that
        // key error handling on found[0] == "error" behave correctly.
        let resp_name = if resp_args.get("type").and_then(Value::as_str) == Some("unknownMethod") {
            "error"
        } else {
            name
        };
        responses.push(json!([resp_name, resp_args, call_id]));
    }

    axum::Json(json!({
        "methodResponses": responses,
        "sessionState": "session-0"
    }))
    .into_response()
}

/// Serve a blob download (RFC 8620 §6.2). The body echoes the substituted URL
/// coordinates so a proxy test can prove the request was forwarded verbatim with
/// auth injected; a real server would stream the blob bytes here.
async fn download(
    headers: HeaderMap,
    Path((account, blob, name)): Path<(String, String, String)>,
) -> Response {
    if !check_auth(&headers) {
        return (StatusCode::UNAUTHORIZED, "unauthorized").into_response();
    }
    let mut resp = Response::new(Body::from(format!("BLOB:{account}:{blob}:{name}")));
    let h = resp.headers_mut();
    h.insert(
        header::CONTENT_TYPE,
        "application/octet-stream".parse().unwrap(),
    );
    h.insert(
        header::CONTENT_DISPOSITION,
        format!("attachment; filename=\"{name}\"").parse().unwrap(),
    );
    resp
}

fn dispatch(store: &MailStore, name: &str, args: &Value) -> Value {
    match name {
        "Mailbox/get" => json!({
            "accountId": ACCOUNT_ID,
            "state": "mb-0",
            "list": [
                { "id": "mb-inbox", "name": "Inbox", "role": "inbox", "sortOrder": 0,
                  "totalEmails": inbox_count(store), "unreadEmails": 0, "totalThreads": 0, "unreadThreads": 0 },
                { "id": "mb-sent", "name": "Sent", "role": "sent", "sortOrder": 1,
                  "totalEmails": sent_count(store), "unreadEmails": 0, "totalThreads": 0, "unreadThreads": 0 }
            ],
            "notFound": []
        }),
        "Email/query" => {
            let inner = store.inner.lock().unwrap();
            let mailbox = args
                .get("filter")
                .and_then(|f| f.get("inMailbox"))
                .and_then(Value::as_str)
                .unwrap_or("mb-inbox");
            let ids: Vec<String> = inner
                .order
                .iter()
                .filter(|id| {
                    inner
                        .emails
                        .get(*id)
                        .and_then(|e| e.get("mailboxIds"))
                        .and_then(|m| m.get(mailbox))
                        .and_then(Value::as_bool)
                        .unwrap_or(false)
                })
                .cloned()
                .collect();
            json!({ "accountId": ACCOUNT_ID, "queryState": "q-0", "ids": ids, "total": ids.len(),
                    "position": 0, "canCalculateChanges": false })
        }
        "Email/get" => {
            let inner = store.inner.lock().unwrap();
            let empty = vec![];
            let ids = args.get("ids").and_then(Value::as_array).unwrap_or(&empty);
            let list: Vec<Value> = ids
                .iter()
                .filter_map(|id| id.as_str())
                .filter_map(|id| inner.emails.get(id).cloned())
                .collect();
            let not_found: Vec<Value> = ids
                .iter()
                .filter_map(|id| id.as_str())
                .filter(|id| !inner.emails.contains_key(*id))
                .map(|id| json!(id))
                .collect();
            json!({ "accountId": ACCOUNT_ID, "state": "e-0", "list": list, "notFound": not_found })
        }
        "Email/set" => {
            let mut inner = store.inner.lock().unwrap();
            let mut created = serde_json::Map::new();
            if let Some(creates) = args.get("create").and_then(Value::as_object) {
                for (client_id, spec) in creates {
                    inner.next_id += 1;
                    let id = format!("e{}", inner.next_id);
                    let mut email = spec.clone();
                    email["id"] = json!(id);
                    email["blobId"] = json!(format!("blob-{id}"));
                    email["threadId"] = json!(format!("t-{id}"));
                    if email.get("mailboxIds").is_none() {
                        email["mailboxIds"] = json!({ "mb-drafts": true });
                    }
                    inner.emails.insert(id.clone(), email);
                    inner.order.insert(0, id.clone());
                    created.insert(
                        client_id.clone(),
                        json!({ "id": id, "blobId": format!("blob-{id}") }),
                    );
                }
            }
            json!({ "accountId": ACCOUNT_ID, "oldState": "e-0", "newState": "e-1",
                    "created": created, "updated": {}, "destroyed": [] })
        }
        "EmailSubmission/set" => {
            // Mark the referenced draft as sent: move it into Sent.
            let mut inner = store.inner.lock().unwrap();
            let mut created = serde_json::Map::new();
            if let Some(creates) = args.get("create").and_then(Value::as_object) {
                for (client_id, spec) in creates {
                    let email_id = spec.get("emailId").and_then(Value::as_str).map(|s| {
                        s.strip_prefix('#')
                            .map(String::from)
                            .unwrap_or_else(|| s.to_string())
                    });
                    if let Some(eid) = email_id {
                        // Resolve a creation-id reference (#draft) to the newest email.
                        let real_id = if spec
                            .get("emailId")
                            .and_then(Value::as_str)
                            .map(|s| s.starts_with('#'))
                            .unwrap_or(false)
                        {
                            inner.order.first().cloned().unwrap_or(eid)
                        } else {
                            eid
                        };
                        if let Some(email) = inner.emails.get_mut(&real_id) {
                            email["mailboxIds"] = json!({ "mb-sent": true });
                        }
                    }
                    inner.next_id += 1;
                    let sub_id = format!("sub-{}", inner.next_id);
                    created.insert(client_id.clone(), json!({ "id": sub_id }));
                }
            }
            json!({ "accountId": ACCOUNT_ID, "oldState": "s-0", "newState": "s-1",
                    "created": created, "updated": {}, "destroyed": [] })
        }
        other => security_case(other, args).unwrap_or_else(|| {
            json!({ "type": "unknownMethod", "description": format!("mock does not implement {other}") })
        }),
    }
}

// ── V4 crypto/security parity seed (plan §1.5/§2, e0) ────────────────────────
//
// The mock emits STATIC-FIXTURE responses whose shapes are FROZEN byte-for-byte
// with §2.1/§2.2 so the web builds against the correct crypto/security shapes
// (the V2/V3 lesson: the mock MUST match what e6's engine will emit — a shared
// golden-shape test in e6 asserts engine↔mock parity). Field names/enum tokens
// here are the contract; do not drift without a coordinator re-broadcast.

/// A frozen `CryptoKey` fixture (§2.1) — an own, verified, Autocrypt PGP key.
fn fixture_crypto_key() -> Value {
    json!({
        "id": "key-pgp-1",
        "kind": "pgp",
        "isOwn": true,
        "addresses": [USER],
        "fingerprint": "ABCD1234ABCD1234ABCD1234ABCD1234ABCD1234",
        "keyId": "ABCD1234ABCD1234",
        "algorithm": "ed25519",
        "createdAt": "2026-07-12T09:00:00Z",
        "expiresAt": null,
        "publicKeyArmored": "-----BEGIN PGP PUBLIC KEY BLOCK-----\n(mock)\n-----END PGP PUBLIC KEY BLOCK-----",
        "certPem": null,
        "trust": "verified",
        "autocrypt": true,
        "source": "generated",
        "hasPrivate": true,
        "encryptedPrivateBackup": null,
        "verifiedAt": "2026-07-12T09:00:00Z",
        "keyHistory": [
            { "fingerprint": "ABCD1234ABCD1234ABCD1234ABCD1234ABCD1234", "seenAt": "2026-07-12T09:00:00Z" }
        ]
    })
}

/// A frozen `SecurityVerdict` fixture (§2.1) for `email_id` — DKIM/SPF/DMARC pass,
/// a signed cleartext message, one Received hop, no attachment risk.
fn fixture_verdict(email_id: &str) -> Value {
    json!({
        "emailId": email_id,
        "auth": {
            "dkim": { "result": "pass", "domain": "example.org", "selector": "sel1" },
            "spf": { "result": "pass", "domain": "example.org" },
            "dmarc": { "result": "pass", "policy": "reject", "aligned": true },
            "arc": { "result": "none", "chainLength": 0 }
        },
        "plainLanguage": "This message passed all sender-authentication checks (DKIM, SPF, DMARC).",
        "received": [
            { "index": 0, "byHost": "mx.example.org", "fromHost": "sender.example.org",
              "protocol": "ESMTPS", "timestamp": "2026-07-12T09:00:00Z", "delayMs": 1200,
              "asn": 64500, "asnOrg": "Example Networks", "country": "PT" }
        ],
        "signature": {
            "kind": "pgp", "status": "verified", "signerKeyId": "ABCD1234ABCD1234",
            "algorithm": "ed25519", "keyCreatedAt": "2026-07-12T09:00:00Z", "keyExpiresAt": null,
            "chainStatus": "trusted", "revocationStatus": "good", "keyChanged": false
        },
        "encryption": { "kind": "none", "isEncrypted": false, "decryptsClientSide": false },
        "attachments": [],
        "anomalies": []
    })
}

/// A frozen `DlpRule` fixture (§2.1) — a card-number (PAN) block rule.
fn fixture_dlp_rule() -> Value {
    json!({
        "id": "rule-pan",
        "name": "Block card numbers",
        "enabled": true,
        "priority": 10,
        "conditions": {
            "detectors": ["pan"], "customRegex": null, "dictionaries": [], "attachmentTypes": [],
            "maxAttachmentSize": null, "recipientDomains": [], "recipientDomainMode": null,
            "classification": null
        },
        "action": "block",
        "message": "This message appears to contain a payment card number and cannot be sent."
    })
}

/// A frozen `MailRule` fixture (§2.1) — a block-sender rule (move to Junk + stop).
fn fixture_mail_rule() -> Value {
    json!({
        "id": "mr-1",
        "name": "Block spammer",
        "matchAll": false,
        "conditions": [ { "type": "from", "op": "is", "value": "spammer@bad.example" } ],
        "actions": [ { "type": "move", "value": "Junk" }, { "type": "stop", "value": null } ],
        "enabled": true,
        "runsAt": "engine"
    })
}

/// The requested `ids` array as owned strings (empty when absent).
fn requested_ids(args: &Value) -> Vec<String> {
    args.get("ids")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

/// Handle a V4 crypto/security method with a frozen static fixture, or `None` if
/// `name` is not a crypto/security family method.
fn security_case(name: &str, args: &Value) -> Option<Value> {
    let v = match name {
        "CryptoKey/get" => json!({
            "accountId": ACCOUNT_ID, "state": "ck-0",
            "list": [fixture_crypto_key()], "notFound": []
        }),
        "CryptoKey/query" => json!({
            "accountId": ACCOUNT_ID, "queryState": "ck-0", "ids": ["key-pgp-1"],
            "total": 1, "position": 0, "canCalculateChanges": false
        }),
        "CryptoKey/lookup" => json!({
            "accountId": ACCOUNT_ID, "list": [fixture_crypto_key()], "notFound": []
        }),
        "CryptoKey/set" => json!({
            "accountId": ACCOUNT_ID, "oldState": "ck-0", "newState": "ck-1",
            "created": {}, "updated": {}, "destroyed": []
        }),
        "CryptoKey/setTrust" => json!({ "accountId": ACCOUNT_ID, "updated": {} }),
        "CryptoKey/changes" => json!({
            "accountId": ACCOUNT_ID, "oldState": "ck-0", "newState": "ck-0",
            "created": [], "updated": [], "destroyed": [], "hasMoreChanges": false
        }),
        "SecurityVerdict/get" => {
            let ids = requested_ids(args);
            let list: Vec<Value> = if ids.is_empty() {
                vec![fixture_verdict("e1")]
            } else {
                ids.iter().map(|id| fixture_verdict(id)).collect()
            };
            json!({ "accountId": ACCOUNT_ID, "state": "sv-0", "list": list, "notFound": [] })
        }
        "SenderControl/set" => json!({ "updated": true, "mailRuleId": "mr-1" }),
        "MailRule/get" => json!({
            "accountId": ACCOUNT_ID, "state": "mr-0",
            "list": [fixture_mail_rule()], "notFound": []
        }),
        "MailRule/set" => json!({
            "accountId": ACCOUNT_ID, "oldState": "mr-0", "newState": "mr-1",
            "created": {}, "updated": {}, "destroyed": []
        }),
        "MailRule/changes" => json!({
            "accountId": ACCOUNT_ID, "oldState": "mr-0", "newState": "mr-0",
            "created": [], "updated": [], "destroyed": [], "hasMoreChanges": false
        }),
        "Dlp/getRules" => json!({ "list": [fixture_dlp_rule()] }),
        // No findings by default (a clean draft); a card-number body would return
        // a blocking DlpVerdict — e4/e10 seed that case explicitly.
        "Dlp/scan" => json!({ "list": [] }),
        _ => return None,
    };
    Some(v)
}

/// Resolve JMAP result references (RFC 8620 §3.7) in a method's arguments.
/// A key like `"#ids"` with value `{ "resultOf": <callId>, "name": <method>,
/// "path": <json-pointer> }` is replaced by the value found at `path` inside the
/// referenced prior response's arguments, stored under the de-`#`'d key
/// (e.g. `"ids"`). Unresolvable references are dropped.
fn resolve_references(args: &mut Value, responses: &[Value]) {
    let Some(obj) = args.as_object() else {
        return;
    };
    let ref_keys: Vec<String> = obj.keys().filter(|k| k.starts_with('#')).cloned().collect();
    for key in ref_keys {
        let spec = args[key.as_str()].clone();
        if let (Some(result_of), Some(path)) = (
            spec.get("resultOf").and_then(Value::as_str),
            spec.get("path").and_then(Value::as_str),
        ) {
            let resolved = responses
                .iter()
                .find(|r| r.get(2).and_then(Value::as_str) == Some(result_of))
                .and_then(|r| r.get(1))
                .and_then(|a| a.pointer(path))
                .cloned();
            if let Some(value) = resolved {
                let target = key.trim_start_matches('#');
                args[target] = value;
            }
        }
        if let Some(map) = args.as_object_mut() {
            map.remove(&key);
        }
    }
}

fn inbox_count(store: &MailStore) -> u64 {
    count_in(store, "mb-inbox")
}
fn sent_count(store: &MailStore) -> u64 {
    count_in(store, "mb-sent")
}
fn count_in(store: &MailStore, mailbox: &str) -> u64 {
    let inner = store.inner.lock().unwrap();
    inner
        .emails
        .values()
        .filter(|e| {
            e.get("mailboxIds")
                .and_then(|m| m.get(mailbox))
                .and_then(Value::as_bool)
                .unwrap_or(false)
        })
        .count() as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddr;

    async fn spawn() -> SocketAddr {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, router()).await.unwrap();
        });
        addr
    }

    fn auth() -> String {
        format!(
            "Basic {}",
            base64::engine::general_purpose::STANDARD.encode(format!("{USER}:{PASS}"))
        )
    }

    #[tokio::test]
    async fn requires_auth() {
        let addr = spawn().await;
        let c = reqwest::Client::new();
        let resp = c
            .get(format!("http://{addr}/jmap/session"))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 401);
    }

    #[tokio::test]
    async fn resolves_result_references() {
        // Email/query -> Email/get chained by a `#ids` result reference, exactly
        // as the web client issues it. Without reference resolution Email/get
        // sees no `ids` and returns an empty list (regression guard).
        let addr = spawn().await;
        let c = reqwest::Client::new();
        let res = c
            .post(format!("http://{addr}/jmap"))
            .header("authorization", auth())
            .json(&json!({
                "using": ["urn:ietf:params:jmap:mail"],
                "methodCalls": [
                    ["Email/query", {"filter": {"inMailbox": "mb-inbox"}}, "q"],
                    ["Email/get", {
                        "#ids": {"resultOf": "q", "name": "Email/query", "path": "/ids"},
                        "properties": ["id", "subject"]
                    }, "g"]
                ]
            }))
            .send()
            .await
            .unwrap()
            .json::<Value>()
            .await
            .unwrap();
        assert_eq!(res["methodResponses"][1][2], "g");
        let list = res["methodResponses"][1][1]["list"].as_array().unwrap();
        assert_eq!(list.len(), 3);
    }

    #[tokio::test]
    async fn query_get_and_send_flow() {
        let addr = spawn().await;
        let c = reqwest::Client::new();

        // Query inbox
        let q = c
            .post(format!("http://{addr}/jmap"))
            .header("authorization", auth())
            .json(&json!({
                "using": ["urn:ietf:params:jmap:mail"],
                "methodCalls": [["Email/query", {"filter": {"inMailbox": "mb-inbox"}}, "c0"]]
            }))
            .send()
            .await
            .unwrap()
            .json::<Value>()
            .await
            .unwrap();
        let ids = q["methodResponses"][0][1]["ids"].as_array().unwrap();
        assert_eq!(ids.len(), 3);

        // Create a draft then submit it; assert it lands in Sent.
        let send = c
            .post(format!("http://{addr}/jmap"))
            .header("authorization", auth())
            .json(&json!({
                "using": ["urn:ietf:params:jmap:mail", "urn:ietf:params:jmap:submission"],
                "methodCalls": [
                    ["Email/set", {"create": {"draft": {
                        "mailboxIds": {"mb-drafts": true},
                        "from": [{"email": USER}],
                        "to": [{"email": USER}],
                        "subject": "Hello me",
                        "bodyValues": {"1": {"value": "<p>hi</p>"}},
                        "htmlBody": [{"partId": "1", "type": "text/html"}]
                    }}}, "c1"],
                    ["EmailSubmission/set", {"create": {"sub1": {"emailId": "#draft"}}}, "c2"]
                ]
            }))
            .send()
            .await
            .unwrap()
            .json::<Value>()
            .await
            .unwrap();
        assert!(send["methodResponses"][0][1]["created"]["draft"]["id"].is_string());

        // Sent should now contain one message.
        let sent = c
            .post(format!("http://{addr}/jmap"))
            .header("authorization", auth())
            .json(&json!({
                "using": ["urn:ietf:params:jmap:mail"],
                "methodCalls": [["Email/query", {"filter": {"inMailbox": "mb-sent"}}, "c0"]]
            }))
            .send()
            .await
            .unwrap()
            .json::<Value>()
            .await
            .unwrap();
        assert_eq!(
            sent["methodResponses"][0][1]["ids"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
    }
}
