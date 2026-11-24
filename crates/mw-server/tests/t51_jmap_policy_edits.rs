//! t51: the administrator edits that `jmap_api` makes to a JMAP request must be
//! the edits that are dispatched.
//!
//! `message_headers::apply` writes `compose.headers.forced` into every submitted
//! message and rewrites each submission to destroy-after-send when
//! `compose.sentFolder.skipCopy` is on; `delete_policy::admit` strips the
//! `mailwomanPermanent` marker. Both edit the parsed request in place. These
//! tests assert the effect on what the engine stores (engine mode) and on what
//! the upstream receives (proxy mode, cookie and native bearer sessions). An API
//! key on `/jmap/api` is admitted only for Sieve-only batches, so a composed
//! batch sent with one is refused before it reaches the upstream.
//!
//! Before the t51 fix, `jmap_api` re-parsed the raw body after the hooks ran and
//! dispatched that copy, so none of these edits took effect.
use std::sync::{Arc, Mutex};

use mw_server::{AppConfig, SecurityConfig, ServerMode, build_app};
use mw_store::settings_policy::SettingPolicyRow;
use mw_store::{DomainConfigRow, ServerKey, Store};
use serde_json::{Value, json};
mod common;
use common::{mock_mail::MockPop3, test_db};

// Sensitivity: not gated by `compose.headers.*`, accepted by the engine's
// optional-header floor, and one of the headers `Email/get` can read back.
const FORCED_NAME: &str = "Sensitivity";
const FORCED_VALUE: &str = "company-confidential";
const SUBJECT: &str = "t51 policy edit";
const USING: [&str; 3] = [
    "urn:ietf:params:jmap:core",
    "urn:ietf:params:jmap:mail",
    "urn:ietf:params:jmap:submission",
];

async fn set(store: &Store, key: &str, value: Value) {
    store
        .put_setting_policy(&SettingPolicyRow {
            key: key.into(),
            global_value: Some(value.to_string()),
            allow_user_override: false,
            global_locked: true,
            updated_by: "test".into(),
            updated_at: 1,
        })
        .await
        .unwrap();
}

async fn apply_policies(store: &Store) {
    set(store, "compose.headers.forced", json!([format!("{FORCED_NAME}={FORCED_VALUE}")])).await;
    set(store, "compose.sentFolder.skipCopy", json!(true)).await;
}

async fn serve(app: axum::Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .unwrap()
    });
    base
}

fn client() -> reqwest::Client {
    reqwest::Client::builder().no_proxy().cookie_store(true).build().unwrap()
}

async fn post(client: &reqwest::Client, base: &str, calls: Value) -> (u16, Value) {
    let response = client
        .post(format!("{base}/jmap/api"))
        .json(&json!({"using": USING, "methodCalls": calls}))
        .send()
        .await
        .unwrap();
    let status = response.status().as_u16();
    let text = response.text().await.unwrap();
    (status, serde_json::from_str(&text).unwrap_or(Value::String(text)))
}

fn result<'a>(response: &'a Value, call_id: &str) -> &'a Value {
    response["methodResponses"]
        .as_array()
        .and_then(|calls| calls.iter().find(|c| c[2] == call_id))
        .map(|c| &c[1])
        .unwrap_or(&Value::Null)
}

fn draft(account: &str, from: &str, mailbox: Option<&str>) -> Value {
    let mut spec = json!({
        "from": [{"email": from}],
        "to": [{"email": "bob@example.net"}],
        "subject": SUBJECT,
        "keywords": {"$draft": true, "$seen": true},
        "textBody": [{"partId": "p", "type": "text/plain"}],
        "bodyValues": {"p": {"value": "hello"}},
    });
    if let Some(mailbox) = mailbox {
        spec["mailboxIds"] = json!({ mailbox: true });
    }
    json!(["Email/set", {"accountId": account, "create": {"draft": spec}}, "e"])
}

/// Engine mode: the stored draft carries the forced header and the held
/// submission records "skip the sent copy".
#[tokio::test]
async fn engine_mode_dispatches_the_policy_edited_request() {
    const KEY: &str = "5100112233445566778899aabbccddeeff00112233445566778899aabbccdd51";
    let mail = common::mock_imap::MockImap::start_drafts("alice@example.org", "testpass").await;
    let fallback = MockPop3::start("unused", "unused").await;
    let db = test_db::unique_dir("t51-policy-edits-engine")
        .join("db.sqlite")
        .to_string_lossy()
        .into_owned();
    let store = Store::open(&db, ServerKey::from_hex(KEY).unwrap()).await.unwrap();
    set(&store, "signin.retryCooldownSecs", json!(0)).await;
    store
        .put_domain_config(&DomainConfigRow {
            name: "example.org".into(),
            upstream_json: json!({"imap":{"host":"127.0.0.1","port":mail.addr.port(),"tls":"none","allowPrivate":true}}).to_string(),
            allowlist_json: "[]".into(),
            blocklist_json: "[]".into(),
            enabled: true,
            preferred: false,
            settings_json: "{}".into(),
            updated_at: 1,
        })
        .await
        .unwrap();
    let base = serve(
        build_app(AppConfig {
            db_path: db.clone(),
            server_key_hex: Some(KEY.into()),
            mode: ServerMode::Engine,
            cookie_secure: false,
            web_dir: None,
            hardening: Default::default(),
            security: SecurityConfig { upstream_default: Some(fallback.url()), ..Default::default() },
        })
        .await
        .unwrap(),
    )
    .await;
    let user = client();
    let signed: Value = user
        .post(format!("{base}/api/login"))
        .json(&json!({"username":"alice@example.org","password":"testpass"}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let account = signed["accountId"].as_str().expect("signed in").to_string();
    apply_policies(&store).await;

    let (status, boxes) = post(&user, &base, json!([["Mailbox/get", {"accountId": account}, "m"]])).await;
    assert_eq!(status, 200, "{boxes}");
    let drafts = result(&boxes, "m")["list"]
        .as_array()
        .and_then(|list| list.iter().find(|m| m["role"] == "drafts"))
        .and_then(|m| m["id"].as_str())
        .unwrap_or_else(|| panic!("a Drafts mailbox: {boxes}"))
        .to_string();

    // The manual hold keeps the submission off the wire, so the test needs no SMTP.
    let (status, sent) = post(
        &user,
        &base,
        json!([
            draft(&account, "alice@example.org", Some(&drafts)),
            ["EmailSubmission/set", {"accountId": account, "create": {"send": {
                "emailId": "#draft", "mailwomanHold": "manual"
            }}}, "s"],
        ]),
    )
    .await;
    assert_eq!(status, 200, "{sent}");
    let email_id = result(&sent, "e")["created"]["draft"]["id"]
        .as_str()
        .unwrap_or_else(|| panic!("draft created: {sent}"))
        .to_string();
    let submission_id = result(&sent, "s")["created"]["send"]["id"]
        .as_str()
        .unwrap_or_else(|| panic!("submission created: {sent}"))
        .to_string();

    let (status, read) = post(
        &user,
        &base,
        json!([
            ["Email/get", {"accountId": account, "ids": [email_id],
                "properties": [format!("header:{FORCED_NAME}:asText"), "subject"]}, "g"],
            ["EmailSubmission/get", {"accountId": account, "ids": [submission_id]}, "q"],
        ]),
    )
    .await;
    assert_eq!(status, 200, "{read}");
    assert_eq!(
        result(&read, "g")["list"][0][format!("header:{FORCED_NAME}:asText")],
        json!(FORCED_VALUE),
        "compose.headers.forced is written into the submitted message: {read}"
    );
    assert_eq!(
        result(&read, "q")["list"][0]["mailwomanSentCopy"],
        json!({"skip": true}),
        "compose.sentFolder.skipCopy makes the submission destroy-after-send: {read}"
    );
}

/// A proxy-mode server over the mock JMAP upstream, recording every JMAP body
/// the upstream receives.
struct Proxy {
    base: String,
    store: Store,
    seen: Arc<Mutex<Vec<Value>>>,
}

impl Proxy {
    async fn start(key: &str, label: &str) -> Self {
        let seen: Arc<Mutex<Vec<Value>>> = Arc::default();
        let capture = seen.clone();
        let upstream = serve(mw_mock_jmap::router().layer(axum::middleware::from_fn(
            move |request: axum::extract::Request, next: axum::middleware::Next| {
                let capture = capture.clone();
                async move {
                    let (parts, body) = request.into_parts();
                    let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
                    if let Ok(value) = serde_json::from_slice::<Value>(&bytes)
                        && value.get("methodCalls").is_some()
                    {
                        capture.lock().unwrap().push(value);
                    }
                    next.run(axum::extract::Request::from_parts(parts, axum::body::Body::from(bytes)))
                        .await
                }
            },
        )))
        .await;
        let db = test_db::unique_dir(label)
            .join("db.sqlite")
            .to_string_lossy()
            .into_owned();
        let store = Store::open(&db, ServerKey::from_hex(key).unwrap()).await.unwrap();
        let base = serve(
            build_app(AppConfig {
                db_path: db,
                server_key_hex: Some(key.into()),
                web_dir: None,
                cookie_secure: false,
                mode: ServerMode::Proxy,
                hardening: Default::default(),
                security: SecurityConfig {
                    upstream_default: Some(upstream.clone()),
                    jmap_upstreams: Some(vec![upstream]),
                    ..Default::default()
                },
            })
            .await
            .unwrap(),
        )
        .await;
        Self { base, store, seen }
    }

    /// The composed batch: a draft, its submission with a sent-copy move, and a
    /// destroy carrying the server-only `mailwomanPermanent` marker.
    fn batch() -> Value {
        let account = mw_mock_jmap::ACCOUNT_ID;
        json!({"using": USING, "methodCalls": [
            draft(account, mw_mock_jmap::USER, None),
            ["EmailSubmission/set", {"accountId": account,
                "create": {"send": {"emailId": "#draft"}},
                "onSuccessUpdateEmail": {"#send": {"keywords/$draft": null}}}, "s"],
            ["Email/set", {"accountId": account, "destroy": ["gone"], "mailwomanPermanent": true}, "d"],
        ]})
    }

    fn forwarded(&self) -> Option<Value> {
        self.seen
            .lock()
            .unwrap()
            .iter()
            .rev()
            .find(|r| r.to_string().contains(SUBJECT))
            .cloned()
    }

    /// The upstream received the batch with every policy edit applied.
    fn assert_edited(&self) {
        let forwarded = self.forwarded().expect("the batch reached the upstream");
        let calls = forwarded["methodCalls"].as_array().unwrap();
        assert_eq!(
            calls[0][1]["create"]["draft"][format!("header:{FORCED_NAME}:asText")],
            json!(FORCED_VALUE),
            "compose.headers.forced is written into the forwarded message: {forwarded}"
        );
        let submission = &calls[1][1];
        assert_eq!(
            submission["onSuccessDestroyEmail"],
            json!(["#send"]),
            "compose.sentFolder.skipCopy rewrites the forwarded submission: {forwarded}"
        );
        assert!(
            submission.get("onSuccessUpdateEmail").is_none(),
            "the sent-copy move is removed under skipCopy: {forwarded}"
        );
        assert!(
            !forwarded.to_string().contains("mailwomanPermanent"),
            "the internal mailwomanPermanent marker is not forwarded upstream: {forwarded}"
        );
    }
}

/// Proxy mode, cookie session: the request the upstream receives carries the
/// forced header, the destroy-after-send rewrite, and no `mailwomanPermanent`.
#[tokio::test]
async fn proxy_mode_forwards_the_policy_edited_request() {
    let proxy = Proxy::start(
        "51aa112233445566778899aabbccddeeff00112233445566778899aabbccdd51",
        "t51-policy-edits-proxy",
    )
    .await;
    let user = client();
    let login = user
        .post(format!("{}/api/login", proxy.base))
        .json(&json!({"username": mw_mock_jmap::USER, "password": mw_mock_jmap::PASS}))
        .send()
        .await
        .unwrap();
    assert_eq!(login.status().as_u16(), 200);
    apply_policies(&proxy.store).await;

    let response = user
        .post(format!("{}/jmap/api", proxy.base))
        .json(&Proxy::batch())
        .send()
        .await
        .unwrap();
    assert_eq!(response.status().as_u16(), 200, "{}", response.text().await.unwrap());
    proxy.assert_edited();
}

/// Proxy mode, native bearer session (`Authorization: Bearer`, no cookie): the
/// same edits reach the upstream.
#[tokio::test]
async fn proxy_mode_bearer_session_forwards_the_policy_edited_request() {
    let proxy = Proxy::start(
        "51bb112233445566778899aabbccddeeff00112233445566778899aabbccdd51",
        "t51-policy-edits-bearer",
    )
    .await;
    let bare = reqwest::Client::builder().no_proxy().build().unwrap();
    let login: Value = bare
        .post(format!("{}/api/login", proxy.base))
        .json(&json!({"username": mw_mock_jmap::USER, "password": mw_mock_jmap::PASS, "clientType": "native"}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let token = login["token"]
        .as_str()
        .unwrap_or_else(|| panic!("native login returns a bearer token: {login}"))
        .to_string();
    apply_policies(&proxy.store).await;

    let response = bare
        .post(format!("{}/jmap/api", proxy.base))
        .bearer_auth(&token)
        .json(&Proxy::batch())
        .send()
        .await
        .unwrap();
    assert_eq!(response.status().as_u16(), 200, "{}", response.text().await.unwrap());
    proxy.assert_edited();
}

/// Proxy mode, API key as the bearer: `/jmap/api` admits a key only for a
/// Sieve-only batch, so the composed batch is refused and nothing unedited
/// reaches the upstream.
#[tokio::test]
async fn proxy_mode_api_key_cannot_forward_an_unedited_batch() {
    let proxy = Proxy::start(
        "51cc112233445566778899aabbccddeeff00112233445566778899aabbccdd51",
        "t51-policy-edits-key",
    )
    .await;
    let user = client();
    let login: Value = user
        .post(format!("{}/api/login", proxy.base))
        .json(&json!({"username": mw_mock_jmap::USER, "password": mw_mock_jmap::PASS}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let account = login["accountId"].as_str().expect("signed in").to_string();
    common::auth::user_step_up(&user, &proxy.base, mw_mock_jmap::PASS).await;
    let minted: Value = user
        .post(format!("{}/api/keys", proxy.base))
        .json(&json!({"label": "t51", "accountId": account, "scope": {
            "read": true, "send": true, "delete": true,
            "accounts": {"subset": [account]}, "folders": "all",
            "mail": true, "pim": false, "ip_allowlist": [], "expires_at": null,
            "rate_limit": null, "mcp_tools": [], "unattended_send": true,
        }}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let key = minted["displayToken"]
        .as_str()
        .unwrap_or_else(|| panic!("a shown-once key: {minted}"))
        .to_string();
    apply_policies(&proxy.store).await;

    let response = reqwest::Client::builder()
        .no_proxy()
        .build()
        .unwrap()
        .post(format!("{}/jmap/api", proxy.base))
        .bearer_auth(&key)
        .json(&Proxy::batch())
        .send()
        .await
        .unwrap();
    let status = response.status().as_u16();
    assert!(
        status == 401 || status == 403,
        "an API key cannot send a composed JMAP batch: {status} {}",
        response.text().await.unwrap()
    );
    assert!(proxy.forwarded().is_none(), "nothing reached the upstream");
}
