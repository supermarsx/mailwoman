//! V7 BRIDGE SEND gate (t7-fix-bridgesend) — prove that an `EmailSubmission` for a
//! boot-loaded, plugin-backed (bridge) account is ROUTED to the bridge's `submit`
//! account-backend export (Graph `sendMail`), NOT refused, and that a bridge send
//! failure SURFACES an error instead of being silently dropped.
//!
//! Companion to `v7_boot_load.rs` (which proves the bridge MAIL *read* surface loads at
//! boot). This file closes the SEND gap: before the fix, a plugin-backed account's
//! submitter was `BridgeSubmitDenied` (every `EmailSubmission` refused with
//! `Unsupported`); now it is a `BridgeSubmitter` that drives the frozen
//! `AccountBackend::submit` export through the jail to the provider send API.
//!
//! Like `v7_boot_load.rs`, a recorded-fixture `HttpFetcher` is injected into the
//! `PluginHost` so the Graph bridge never dials `graph.microsoft.com` for real; the
//! store rows, the bundled component, the boot loader, and the engine's whole
//! `EmailSubmission/set` → `submit_email` → `MailSubmitter::submit` path are the real
//! ones. The Graph `POST /me/sendMail` fixture (`send_mail.json`, 202) is the send.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde_json::{Value, json};

use mw_engine::Engine;
use mw_plugin::{
    HostServices, HttpFetcher, HttpReq, HttpResp, OAuthTokenProvider, PluginHost, TrustRoot,
};
use mw_store::{
    AccountKind, BridgeAccountRow, Credentials, NewAccount, PluginRow, ServerKey, Store,
};

/// The fake bearer token the Graph fixtures accept (real tokens never enter the guest).
const FIXTURE_TOKEN: &str = "FIXTURE.ACCESS.TOKEN";

/// Point the externalized component resolver (`v7_mount::resolve_component`) at the
/// repo's canonical shipped layout `plugins/dist/<id>.wasm` (t9-e5): the server no
/// longer embeds the `.wasm` bytes, so the boot loader reads + digest-verifies them
/// from `MW_PLUGIN_DIR`. SAFETY: set once to a fixed path; every test in this binary
/// uses the same value, and the bytes byte-match the compiled-in digest pin.
fn point_plugin_dir_at_shipped_layout() {
    unsafe {
        std::env::set_var(
            "MW_PLUGIN_DIR",
            concat!(env!("CARGO_MANIFEST_DIR"), "/../../plugins/dist"),
        );
    }
}

/// A self-contained replay of the committed `plugins/bridge-graph/fixtures/*.json`
/// request→response pairs (method + longest `url_contains` first) — the same matcher
/// `v7_boot_load.rs`/e16 use, reimplemented here so the harness needs no dependency on
/// the bridge crate. `fail_send` forces a fatal status on `POST /me/sendMail` to prove
/// a bridge send failure surfaces (never silently drops).
struct GraphFixtureHttp {
    fixtures: Vec<(String, String, u16, Vec<u8>)>,
    seen: Mutex<Vec<(String, Option<String>)>>,
    fail_send: bool,
}

impl GraphFixtureHttp {
    fn load(fail_send: bool) -> Self {
        let dir =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../plugins/bridge-graph/fixtures");
        let mut fixtures = Vec::new();
        for entry in std::fs::read_dir(&dir)
            .unwrap_or_else(|e| panic!("read graph fixtures {}: {e}", dir.display()))
            .flatten()
        {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let v: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
            let method = v["method"].as_str().unwrap().to_string();
            let url_contains = v["url_contains"].as_str().unwrap().to_string();
            let status = v["status"].as_u64().unwrap() as u16;
            let body = if let Some(j) = v.get("body_json") {
                if j.is_null() {
                    Vec::new()
                } else {
                    serde_json::to_vec(j).unwrap()
                }
            } else if let Some(t) = v.get("body_text").and_then(Value::as_str) {
                t.as_bytes().to_vec()
            } else {
                Vec::new()
            };
            fixtures.push((method, url_contains, status, body));
        }
        fixtures.sort_by_key(|f| std::cmp::Reverse(f.1.len()));
        assert!(!fixtures.is_empty(), "graph fixtures loaded");
        Self {
            fixtures,
            seen: Mutex::new(Vec::new()),
            fail_send,
        }
    }

    /// How many host-mediated calls hit the provider send endpoint.
    fn send_calls(&self) -> usize {
        self.seen
            .lock()
            .unwrap()
            .iter()
            .filter(|(url, _)| url.contains("/sendMail"))
            .count()
    }
}

#[async_trait]
impl HttpFetcher for GraphFixtureHttp {
    async fn fetch(&self, req: HttpReq) -> Result<HttpResp, String> {
        let authorization = req
            .headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case("authorization"))
            .map(|(_, v)| v.clone());
        self.seen
            .lock()
            .unwrap()
            .push((req.url.clone(), authorization));
        // Simulate a fatal provider send rejection so the failure path can be asserted.
        if self.fail_send && req.url.contains("/sendMail") {
            return Ok(HttpResp {
                status: 500,
                headers: vec![("Content-Type".into(), "application/json".into())],
                body: br#"{"error":{"code":"InternalServerError"}}"#.to_vec(),
            });
        }
        for (method, url_contains, status, body) in &self.fixtures {
            if method.eq_ignore_ascii_case(&req.method) && req.url.contains(url_contains) {
                return Ok(HttpResp {
                    status: *status,
                    headers: vec![("Content-Type".into(), "application/json".into())],
                    body: body.clone(),
                });
            }
        }
        Err(format!("no fixture for {} {}", req.method, req.url))
    }
}

struct FixtureOAuth;
#[async_trait]
impl OAuthTokenProvider for FixtureOAuth {
    async fn token(&self, _account: &str) -> Result<String, String> {
        Ok(FIXTURE_TOKEN.to_string())
    }
}

fn fixture_registry(http: Arc<GraphFixtureHttp>) -> mw_server::plugins::PluginRegistry {
    let host = PluginHost::try_new(
        HostServices {
            http,
            oauth: Arc::new(FixtureOAuth),
            ..HostServices::default()
        },
        TrustRoot::empty(),
    )
    .unwrap();
    Arc::new(Mutex::new(host))
}

async fn jmap(engine: &Engine, account_id: &str, calls: Value) -> Value {
    engine
        .handle_jmap(account_id, &json!({ "methodCalls": calls }))
        .await
}

fn method_result<'a>(resp: &'a Value, call_id: &str) -> &'a Value {
    resp["methodResponses"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r[2] == call_id)
        .map(|r| &r[1])
        .unwrap_or(&Value::Null)
}

/// Seed the persisted 0008 rows a real admin would create (an approved+enabled
/// `bridge-graph` plugin + a `bridge_accounts` binding + the account), returning the
/// account id.
async fn seed_bridge_account(store: &Store) -> String {
    let account_id = store
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "graph.microsoft.com",
                port: 443,
                tls: "implicit",
                username: "me@vogue-homes.com",
                sync_policy_json: "{}",
            },
            &Credentials {
                username: "me@vogue-homes.com".into(),
                password: "unused-bridge-uses-oauth".into(),
            },
        )
        .await
        .unwrap();
    store
        .put_plugin(&PluginRow {
            id: "bridge-graph".into(),
            name: "Microsoft Graph bridge".into(),
            version: "26.8.0".into(),
            signature_hex: None,
            approved_by: Some("admin@vogue-homes.com".into()),
            enabled: true,
            capabilities_json: r#"["account-backend","net","addrbook-source"]"#.into(),
            net_allowlist_json: r#"["graph.microsoft.com","login.microsoftonline.com"]"#.into(),
            limits_json: "{}".into(),
            created_at: "2026-07-14T00:00:00Z".into(),
        })
        .await
        .unwrap();
    store
        .put_bridge_account(&BridgeAccountRow {
            account_id: account_id.clone(),
            bridge_id: "bridge-graph".into(),
            oauth_ref: Some("graph-token-ref".into()),
            extra_json: "{}".into(),
        })
        .await
        .unwrap();
    account_id
}

/// Compose a draft and submit it in one request against the boot-loaded bridge account.
fn compose_and_submit() -> Value {
    json!([
        ["Email/set", { "create": { "draft": {
            "from": [{ "email": "me@vogue-homes.com" }],
            "to": [{ "email": "friend@example.org" }],
            "subject": "Sent through the bridge",
            "bodyValues": { "1": { "value": "Hi from the Graph bridge!" } },
            "textBody": [{ "partId": "1", "type": "text/plain" }]
        } } }, "c1"],
        ["EmailSubmission/set", { "create": { "sub1": { "emailId": "#draft" } } }, "c2"]
    ])
}

/// **The headline proof.** A boot-loaded, plugin-backed account submits an
/// `EmailSubmission`; assert it is NOT refused and that it ROUTES to the bridge's
/// `submit` export — verified against the recorded Graph `POST /me/sendMail` fixture —
/// exactly once (no double-send from draft/Sent appends).
#[tokio::test]
async fn boot_loaded_bridge_email_submission_routes_to_the_bridge_submit_export() {
    point_plugin_dir_at_shipped_layout();
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let account_id = seed_bridge_account(&store).await;

    let http = Arc::new(GraphFixtureHttp::load(false));
    let registry = fixture_registry(http.clone());
    let engine = Arc::new(Engine::new(store.clone()));

    // The REAL boot loader wires the bridge backend + the new BridgeSubmitter.
    let (loaded, _bridge_caps) =
        mw_server::v7_mount::load_plugin_backends(&engine, &registry, &store).await;
    assert_eq!(loaded, 1, "the boot loader auto-loaded the bridge backend");
    assert!(engine.is_plugin_backed(&account_id));

    // Populate the mailbox tree through the jail (message sync may partially fail
    // against the fixtures; the folder tree + local drafts/sent still work).
    let _ = engine.resync(&account_id).await;
    assert_eq!(
        http.send_calls(),
        0,
        "sync/read of a bridge account must never call the send endpoint"
    );

    // Compose a draft + submit it in one request — the real EmailSubmission path.
    let resp = jmap(&engine, &account_id, compose_and_submit()).await;

    // The draft was created and the submission was ACCEPTED (not the old
    // `BridgeSubmitDenied` → notCreated/Unsupported refusal).
    assert!(
        method_result(&resp, "c1")["created"]["draft"]["id"].is_string(),
        "draft created locally (no upstream append/send): {resp}"
    );
    let sub = &method_result(&resp, "c2");
    assert!(
        sub["created"]["sub1"]["id"].is_string(),
        "EmailSubmission accepted + routed to the bridge (NOT refused): {resp}"
    );
    assert!(
        sub["notCreated"].get("sub1").is_none(),
        "submission was not refused: {resp}"
    );

    // It ROUTED to the bridge `submit` export: exactly one host-mediated `/me/sendMail`
    // call — the draft-create + Sent-file APPENDs are skipped for a bridge (they would
    // otherwise each re-trigger a provider send), so precisely one send fired.
    assert_eq!(
        http.send_calls(),
        1,
        "the submission fired the provider send exactly once (no double-send)"
    );

    // The send carried the host-minted transient token, and only Graph was reached.
    let sent = http
        .seen
        .lock()
        .unwrap()
        .iter()
        .find(|(url, _)| url.contains("/sendMail"))
        .cloned()
        .expect("a /me/sendMail call was recorded");
    assert!(
        sent.0.contains("graph.microsoft.com"),
        "send reached Graph: {}",
        sent.0
    );
    assert_eq!(
        sent.1.as_deref(),
        Some(format!("Bearer {FIXTURE_TOKEN}").as_str()),
        "the bridge send carried the host-minted transient token"
    );

    // The sent copy is surfaced on the JMAP Sent mailbox (local re-file) and the draft
    // is gone from Drafts — the send is real end-to-end, not a no-op.
    let mb = jmap(&engine, &account_id, json!([["Mailbox/get", {}, "mb"]])).await;
    let sent_id = method_result(&mb, "mb")["list"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["role"] == "sent")
        .expect("sent mailbox")["id"]
        .as_str()
        .unwrap()
        .to_string();
    let after = jmap(
        &engine,
        &account_id,
        json!([["Email/query", { "filter": { "inMailbox": sent_id } }, "qs"]]),
    )
    .await;
    assert_eq!(
        method_result(&after, "qs")["ids"].as_array().unwrap().len(),
        1,
        "the sent message appears on the JMAP Sent surface: {after}"
    );
}

/// A bridge send FAILURE surfaces an error (never a silent drop): with `/me/sendMail`
/// forced to a fatal 500, the `EmailSubmission` is refused (`notCreated`), not reported
/// as sent.
#[tokio::test]
async fn boot_loaded_bridge_send_failure_surfaces_an_error() {
    point_plugin_dir_at_shipped_layout();
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let account_id = seed_bridge_account(&store).await;

    let http = Arc::new(GraphFixtureHttp::load(true)); // fail the send
    let registry = fixture_registry(http.clone());
    let engine = Arc::new(Engine::new(store.clone()));
    let (loaded, _bridge_caps) =
        mw_server::v7_mount::load_plugin_backends(&engine, &registry, &store).await;
    assert_eq!(loaded, 1);
    let _ = engine.resync(&account_id).await;

    let resp = jmap(&engine, &account_id, compose_and_submit()).await;

    // The submission was attempted (the send endpoint WAS reached — routing works) …
    assert_eq!(
        http.send_calls(),
        1,
        "the failing submission still routed to the bridge send endpoint"
    );
    // … but the provider's fatal 500 surfaced as a refused submission, NOT a silent
    // success. The message is not falsely reported as sent.
    let sub = &method_result(&resp, "c2");
    assert!(
        sub["created"].get("sub1").is_none(),
        "a failed bridge send must NOT report the submission as created: {resp}"
    );
    assert!(
        sub["notCreated"]["sub1"].is_object(),
        "a failed bridge send surfaces a structured error (never silently dropped): {resp}"
    );
}
