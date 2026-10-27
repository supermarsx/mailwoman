//! t28-e18 (f34) — webhooks as stored objects: admin create/list/delete, inbound
//! calls signed per webhook with replay protection, outbound URLs checked when they
//! are entered, and typed events delivered only to the endpoints that ask for them.
//!
//! Before 26.20: inbound calls were checked against one process-wide secret
//! (`MW_WEBHOOK_INBOUND_SECRET`), named any account in the body, could run any
//! action and could be replayed; there was no way to create or delete a webhook
//! (`put_webhook` was only called by tests); and an outbound endpoint received one
//! opaque `StateChange` for every change.
//!
//! The admin and inbound legs drive the real router over HTTP in proxy mode. The
//! typed-event leg runs the production dispatcher (`webhooks::spawn_with`) against
//! a real engine; its address policy admits loopback so the receiver can stand in
//! for a public endpoint (the production refusal of loopback is covered by
//! `t32_h5_webhook_egress` and by the registration leg here).
//!
//! Run:
//!   cargo test -p mw-server --test t28_webhooks --locked -- --test-threads=1

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use axum::body::Bytes;
use axum::http::HeaderMap;
use serde_json::{Value, json};

use mw_engine::account::AccountRuntime;
use mw_engine::backend::{
    AccountBackend, BackendCaps, ChangeSink, Flag, MailboxDelta, MailboxRole, MessageRef,
    MoveOutcome, RawMailbox, RawMailboxRef, RawMessage, SyncCursor, WatchHandle,
};
use mw_engine::{Engine, MailSubmitter};
use mw_server::webhooks::{
    self, InMemoryWebhookActionSink, InMemoryWebhookRegistry, InboundAction, NONCE_HEADER,
    SIGNATURE_HEADER, TIMESTAMP_HEADER, WEBHOOK_ID_HEADER, WebhookActionSink, WebhookEndpoint,
    WebhookRegistry,
};
use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use mw_smtp::{Outgoing, SubmissionResult};
use mw_store::{AccountKind, Credentials, NewAccount, ServerKey, Store};

mod common;
use common::test_db;

const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW_T28_E18</div>";

fn web_dir() -> PathBuf {
    let dir = test_db::unique_dir("mw-t28-e18-web");
    std::fs::write(dir.join("index.html"), INDEX_HTML).unwrap();
    dir
}

fn config() -> AppConfig {
    AppConfig {
        db_path: test_db::unique_db_path("mw-t28-e18")
            .to_string_lossy()
            .to_string(),
        server_key_hex: None,
        web_dir: Some(web_dir()),
        cookie_secure: false,
        mode: ServerMode::Proxy,
        hardening: HardeningConfig::default(),
        security: SecurityConfig::default(),
    }
}

fn admin_v6() -> V6Config {
    V6Config {
        admin_enabled: true,
        admin_username: Some("root".into()),
        admin_password: Some("hunter2".into()),
        redis_url: None,
    }
}

async fn spawn_server() -> String {
    let app = build_app_full(config(), admin_v6())
        .await
        .expect("server boots")
        .0;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });
    format!("http://{addr}")
}

async fn admin_client(server: &str) -> reqwest::Client {
    let c = reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap();
    let r = c
        .post(format!("{server}/admin/login"))
        .json(&json!({ "username": "root", "password": "hunter2" }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200, "admin login");
    c
}

fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

/// A signed inbound call.
async fn call(
    server: &str,
    id: &str,
    secret: &str,
    ts: i64,
    nonce: &str,
    body: &Value,
) -> (u16, Value) {
    let body = serde_json::to_vec(body).unwrap();
    let ts = ts.to_string();
    let sig = webhooks::sign_inbound(secret.as_bytes(), &ts, nonce, &body);
    let r = reqwest::Client::new()
        .post(format!("{server}/api/webhooks/inbound"))
        .header(WEBHOOK_ID_HEADER, id)
        .header(TIMESTAMP_HEADER, &ts)
        .header(NONCE_HEADER, nonce)
        .header(SIGNATURE_HEADER, sig)
        .header("content-type", "application/json")
        .body(body)
        .send()
        .await
        .unwrap();
    let status = r.status().as_u16();
    (status, r.json().await.unwrap_or(Value::Null))
}

fn run_rules() -> Value {
    json!({ "action": { "kind": "run_rules", "envelope": { "from": "a@b.example", "subject": "x" } } })
}

/// Inbound: per-webhook secret, bound account, allowed actions, timestamp window,
/// nonce once. Ends with the process-global sink cleared.
#[tokio::test]
async fn inbound_calls_are_signed_per_webhook_and_cannot_be_replayed() {
    let server = spawn_server().await;
    let admin = admin_client(&server).await;
    let sink = Arc::new(InMemoryWebhookActionSink::new());
    webhooks::set_inbound_dispatcher(Some(sink.clone() as Arc<dyn WebhookActionSink>));

    let made: Value = admin
        .post(format!("{server}/admin/webhooks"))
        .json(&json!({
            "direction": "inbound", "accountId": "alice@example.org",
            "label": "CRM", "actions": ["run_rules"],
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let id = made["webhook"]["id"].as_str().expect("id").to_string();
    let secret = made["secret"]
        .as_str()
        .expect("secret shown once")
        .to_string();
    assert_eq!(secret.len(), 64);

    // Accepted once, for the webhook's own account.
    let (s, body) = call(
        &server,
        &id,
        &secret,
        now(),
        "nonce-aaaaaaaaaaaa-1",
        &run_rules(),
    )
    .await;
    assert_eq!(s, 202, "a correctly signed call is accepted: {body}");
    assert_eq!(body["dispatched"], true);
    let fired = sink.dispatched();
    assert_eq!(fired.len(), 1);
    assert_eq!(
        fired[0].0, "alice@example.org",
        "the webhook's bound account"
    );
    assert!(matches!(fired[0].1, InboundAction::RunRules { .. }));

    // The same signed request again: refused, and nothing runs.
    let ts = now();
    let (s, _) = call(
        &server,
        &id,
        &secret,
        ts,
        "nonce-bbbbbbbbbbbb-2",
        &run_rules(),
    )
    .await;
    assert_eq!(s, 202, "precondition: a fresh nonce is accepted");
    let (s, body) = call(
        &server,
        &id,
        &secret,
        ts,
        "nonce-bbbbbbbbbbbb-2",
        &run_rules(),
    )
    .await;
    assert_eq!(s, 409, "the replay is refused: {body}");
    assert_eq!(sink.dispatched().len(), 2, "the replay ran nothing");

    // A stale timestamp, correctly signed: 401. Just inside the window: accepted.
    let (s, _) = call(
        &server,
        &id,
        &secret,
        now() - 301,
        "nonce-cccccccccccc-3",
        &run_rules(),
    )
    .await;
    assert_eq!(s, 401, "a timestamp five minutes old is refused");
    let (s, _) = call(
        &server,
        &id,
        &secret,
        now() - 290,
        "nonce-dddddddddddd-4",
        &run_rules(),
    )
    .await;
    assert_eq!(s, 202, "inside the window is accepted");

    // Wrong secret, unknown id: the same 401.
    let (s, _) = call(
        &server,
        &id,
        "0".repeat(64).as_str(),
        now(),
        "nonce-eeeeeeeeeeee-5",
        &run_rules(),
    )
    .await;
    assert_eq!(s, 401);
    let (s, _) = call(
        &server,
        "whin_nope",
        &secret,
        now(),
        "nonce-ffffffffffff-6",
        &run_rules(),
    )
    .await;
    assert_eq!(s, 401);

    // Another account, or an action the webhook was not given: 403.
    let mut other = run_rules();
    other["account"] = json!("mallory@example.org");
    let (s, _) = call(&server, &id, &secret, now(), "nonce-gggggggggggg-7", &other).await;
    assert_eq!(s, 403, "a body naming another account is refused");
    let mut own = run_rules();
    own["account"] = json!("alice@example.org");
    let (s, _) = call(&server, &id, &secret, now(), "nonce-hhhhhhhhhhhh-8", &own).await;
    assert_eq!(s, 202, "naming its own account is fine");
    let sieve = json!({ "action": { "kind": "sieve", "action": { "kind": "move", "mailbox": "Archive" }, "message_id": "m1" } });
    let (s, _) = call(&server, &id, &secret, now(), "nonce-iiiiiiiiiiii-9", &sieve).await;
    assert_eq!(s, 403, "sieve was not among this webhook's actions");

    // No headers at all (the old call shape): 401.
    let old = reqwest::Client::new()
        .post(format!("{server}/api/webhooks/inbound"))
        .json(&run_rules())
        .send()
        .await
        .unwrap();
    assert_eq!(old.status(), 401);

    // Deleted: its secret stops working.
    let gone = admin
        .delete(format!("{server}/admin/webhooks/{id}"))
        .send()
        .await
        .unwrap();
    assert_eq!(gone.status(), 204);
    let (s, _) = call(
        &server,
        &id,
        &secret,
        now(),
        "nonce-jjjjjjjjjjjj-10",
        &run_rules(),
    )
    .await;
    assert_eq!(s, 401, "a deleted webhook accepts nothing");

    webhooks::set_inbound_dispatcher(None);
}

/// Admin: outbound URLs go through the egress gate at registration; list hides
/// secrets and carries an etag; a stale If-Match is 412; writes are audited.
#[tokio::test]
async fn admin_registration_checks_the_url_and_audits() {
    let server = spawn_server().await;
    let admin = admin_client(&server).await;

    // Not signed in: 401.
    let anon = reqwest::Client::new()
        .get(format!("{server}/admin/webhooks"))
        .send()
        .await
        .unwrap();
    assert_eq!(anon.status(), 401);

    // A loopback URL is refused when it is entered, with the field named.
    let r = admin
        .post(format!("{server}/admin/webhooks"))
        .json(&json!({
            "direction": "outbound", "accountId": "alice@example.org",
            "url": "http://127.0.0.1:9/hook", "events": ["message.received"],
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 422);
    let body: Value = r.json().await.unwrap();
    assert_eq!(body["code"], "webhookUrlRefused", "{body}");
    assert_eq!(body["field"], "/url");
    for url in [
        "http://169.254.169.254/latest",
        "http://[::1]/x",
        "http://10.0.0.5/x",
    ] {
        let r = admin
            .post(format!("{server}/admin/webhooks"))
            .json(&json!({
                "direction": "outbound", "accountId": "alice@example.org",
                "url": url, "events": ["message.received"],
            }))
            .send()
            .await
            .unwrap();
        assert_eq!(r.status(), 422, "{url} is refused at registration");
    }

    // Unknown event, no event, unknown direction: 422 with the field.
    for (body, field) in [
        (
            json!({ "direction": "outbound", "accountId": "a", "url": "https://1.1.1.1/h", "events": ["nope"] }),
            "/events",
        ),
        (
            json!({ "direction": "outbound", "accountId": "a", "url": "https://1.1.1.1/h", "events": [] }),
            "/events",
        ),
        (
            json!({ "direction": "sideways", "accountId": "a" }),
            "/direction",
        ),
        (
            json!({ "direction": "inbound", "accountId": "", "actions": ["run_rules"] }),
            "/accountId",
        ),
    ] {
        let r = admin
            .post(format!("{server}/admin/webhooks"))
            .json(&body)
            .send()
            .await
            .unwrap();
        assert_eq!(r.status(), 422, "{body}");
        let got: Value = r.json().await.unwrap();
        assert_eq!(got["field"], field, "{got}");
    }

    // dryRun validates and writes nothing.
    let r = admin
        .post(format!("{server}/admin/webhooks"))
        .json(&json!({
            "direction": "outbound", "accountId": "alice@example.org",
            "url": "https://1.1.1.1/hook", "events": ["message.received"], "dryRun": true,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    let dry: Value = r.json().await.unwrap();
    assert_eq!(dry["dryRun"], true);
    assert_eq!(dry["impact"]["host"], "1.1.1.1");
    let list: Value = admin
        .get(format!("{server}/admin/webhooks"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(list["webhooks"], json!([]), "dryRun wrote nothing");
    let etag0 = list["etag"].as_str().unwrap().to_string();

    // A public literal address passes the gate without a DNS lookup.
    let r = admin
        .post(format!("{server}/admin/webhooks"))
        .header("if-match", &etag0)
        .json(&json!({
            "direction": "outbound", "accountId": "alice@example.org", "label": "Ticketing",
            "url": "https://1.1.1.1/hook", "events": ["message.received", "message.received"],
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 201);
    let made: Value = r.json().await.unwrap();
    let id = made["webhook"]["id"].as_str().unwrap().to_string();
    assert_eq!(made["webhook"]["events"], json!(["message.received"]));
    assert_eq!(made["webhook"]["createdBy"], "admin:root");
    assert_eq!(made["secret"].as_str().unwrap().len(), 64);

    // The list hides the secret and moved on from etag0.
    let r = admin
        .get(format!("{server}/admin/webhooks"))
        .send()
        .await
        .unwrap();
    let header_etag = r.headers()["etag"].to_str().unwrap().to_string();
    let list: Value = r.json().await.unwrap();
    assert_eq!(list["webhooks"].as_array().unwrap().len(), 1);
    assert!(list["webhooks"][0].get("secret").is_none(), "{list}");
    assert!(!list.to_string().contains(made["secret"].as_str().unwrap()));
    assert_eq!(header_etag, list["etag"].as_str().unwrap());
    assert_ne!(list["etag"], json!(etag0));
    assert!(
        list["eventKinds"]
            .as_array()
            .unwrap()
            .contains(&json!("message.received"))
    );

    // A write against the old etag: 412 naming who changed it.
    let r = admin
        .delete(format!("{server}/admin/webhooks/{id}"))
        .header("if-match", &etag0)
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 412);
    let stale: Value = r.json().await.unwrap();
    assert_eq!(stale["code"], "stale");
    assert_eq!(stale["updatedBy"], "admin:root");

    // With the current etag the delete goes through.
    let r = admin
        .delete(format!("{server}/admin/webhooks/{id}"))
        .header("if-match", header_etag)
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 204);
    let r = admin
        .delete(format!("{server}/admin/webhooks/{id}"))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 404);

    // Both writes are in the audit log under webhook:<id>, without the secret.
    let audit: Value = admin
        .get(format!("{server}/admin/audit?limit=50"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let target = format!("webhook:{id}");
    let actions: Vec<&str> = audit
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["target"] == target.as_str())
        .map(|e| e["action"].as_str().unwrap())
        .collect();
    assert!(
        actions.contains(&"webhook.create") && actions.contains(&"webhook.delete"),
        "{audit}"
    );
    assert!(!audit.to_string().contains(made["secret"].as_str().unwrap()));
}

/// Start-up refuses the removed global secret.
#[tokio::test]
async fn the_removed_global_inbound_secret_stops_start_up() {
    // SAFETY: this binary runs its tests on one thread (`--test-threads=1`), and no
    // other test here reads this variable.
    unsafe { std::env::set_var("MW_WEBHOOK_INBOUND_SECRET", "old") };
    let result = build_app_full(config(), admin_v6()).await;
    unsafe { std::env::remove_var("MW_WEBHOOK_INBOUND_SECRET") };
    let err = result
        .err()
        .expect("start-up refuses the variable")
        .to_string();
    assert!(
        err.contains("MW_WEBHOOK_INBOUND_SECRET is set but is no longer read"),
        "{err}"
    );
}

// ── typed events through the production dispatcher ───────────────────────────

const UIDVALIDITY: u32 = 100;

/// An IMAP account whose INBOX the test can add to between resyncs.
struct ScriptedBackend {
    inbox: Mutex<Vec<u32>>,
}

fn raw(uid: u32) -> Vec<u8> {
    format!(
        "Message-ID: <wh-{uid}@x>\r\nFrom: alice@example.org\r\nTo: me@example.org\r\n\
         Subject: Hook {uid}\r\nDate: 01 Jul 2026 09:00:00 +0000\r\n\r\nbody {uid}\r\n"
    )
    .into_bytes()
}

#[async_trait]
impl AccountBackend for ScriptedBackend {
    async fn capabilities(&self) -> mw_engine::backend::Result<BackendCaps> {
        Ok(BackendCaps {
            uidplus: true,
            r#move: true,
            special_use: true,
            ..BackendCaps::default()
        })
    }

    async fn list_mailboxes(&self) -> mw_engine::backend::Result<Vec<RawMailbox>> {
        let inbox = self.inbox.lock().unwrap();
        Ok(vec![RawMailbox {
            mailbox_ref: RawMailboxRef {
                name: "INBOX".into(),
                uidvalidity: UIDVALIDITY,
            },
            role: MailboxRole::Inbox,
            parent: None,
            uidnext: inbox.iter().max().copied().unwrap_or(0) + 1,
            highestmodseq: 0,
            total: inbox.len() as u32,
            unread: inbox.len() as u32,
        }])
    }

    async fn sync_mailbox(
        &self,
        mbox: &RawMailboxRef,
        cursor: &SyncCursor,
    ) -> mw_engine::backend::Result<MailboxDelta> {
        let from = match cursor {
            SyncCursor::UidWindow { uidnext, .. } => *uidnext,
            _ => 1,
        };
        let inbox = self.inbox.lock().unwrap();
        let added = inbox
            .iter()
            .filter(|u| **u >= from)
            .map(|u| MessageRef::Imap {
                mailbox: mbox.clone(),
                uidvalidity: UIDVALIDITY,
                uid: *u,
            })
            .collect();
        let next = inbox
            .iter()
            .max()
            .copied()
            .unwrap_or(0)
            .max(from.saturating_sub(1))
            + 1;
        Ok(MailboxDelta {
            added,
            flag_changes: Vec::new(),
            removed: Vec::new(),
            next_cursor: SyncCursor::UidWindow {
                uidvalidity: UIDVALIDITY,
                uidnext: next,
            },
        })
    }

    async fn fetch_raw(&self, refs: &[MessageRef]) -> mw_engine::backend::Result<Vec<RawMessage>> {
        Ok(refs
            .iter()
            .filter_map(|r| match r {
                MessageRef::Imap { uid, .. } => Some(RawMessage {
                    message_ref: r.clone(),
                    raw: raw(*uid),
                    flags: Vec::new(),
                    internaldate: Some("2026-07-01T09:00:00Z".into()),
                }),
                _ => None,
            })
            .collect())
    }

    async fn store_flags(
        &self,
        _r: &[MessageRef],
        _add: &[Flag],
        _rm: &[Flag],
    ) -> mw_engine::backend::Result<()> {
        Ok(())
    }

    async fn move_messages(
        &self,
        _r: &[MessageRef],
        _to: &RawMailboxRef,
    ) -> mw_engine::backend::Result<MoveOutcome> {
        Ok(MoveOutcome::Uidplus {
            uidvalidity: UIDVALIDITY,
            uids: vec![9001],
        })
    }

    async fn append(
        &self,
        mbox: &RawMailboxRef,
        _raw: &[u8],
        _flags: &[Flag],
    ) -> mw_engine::backend::Result<MessageRef> {
        Ok(MessageRef::Imap {
            mailbox: mbox.clone(),
            uidvalidity: UIDVALIDITY,
            uid: 7000,
        })
    }

    async fn watch(&self, _sink: ChangeSink) -> mw_engine::backend::Result<WatchHandle> {
        let (tx, _rx) = tokio::sync::watch::channel(false);
        Ok(WatchHandle::new(tx))
    }
}

struct NullSubmitter;

#[async_trait]
impl MailSubmitter for NullSubmitter {
    async fn submit(&self, msg: Outgoing) -> mw_engine::backend::Result<SubmissionResult> {
        Ok(SubmissionResult {
            accepted: msg.rcpt_to,
            rejected: Vec::new(),
        })
    }
}

/// What each receiver saw: the event header of every delivery, and the bodies.
#[derive(Clone, Default)]
struct Seen(Arc<Mutex<Vec<(String, Value)>>>);

async fn spawn_receiver() -> (String, Seen) {
    let seen = Seen::default();
    let app = axum::Router::new()
        .route(
            "/hook",
            axum::routing::post(
                |axum::extract::State(seen): axum::extract::State<Seen>,
                 headers: HeaderMap,
                 body: Bytes| async move {
                    let event = headers
                        .get("x-mailwoman-event")
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or_default()
                        .to_string();
                    let body: Value = serde_json::from_slice(&body).unwrap_or(Value::Null);
                    seen.0.lock().unwrap().push((event, body));
                    axum::http::StatusCode::OK
                },
            ),
        )
        .with_state(seen.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (format!("http://{addr}/hook"), seen)
}

async fn wait_for(seen: &Seen, n: usize) {
    for _ in 0..100 {
        if seen.0.lock().unwrap().len() >= n {
            return;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

#[tokio::test]
async fn a_message_received_webhook_fires_on_ingest_and_not_on_a_flag_change() {
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let account_id = store
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "imap.example.org",
                port: 993,
                tls: "implicit",
                username: "me@example.org",
                sync_policy_json: "{}",
            },
            &Credentials {
                username: "me@example.org".into(),
                password: "pw".into(),
            },
        )
        .await
        .unwrap();
    let engine = Arc::new(Engine::new(store));
    let backend = Arc::new(ScriptedBackend {
        inbox: Mutex::new(vec![1]),
    });
    engine.register_backend(
        account_id.clone(),
        AccountRuntime::new(
            backend.clone() as Arc<dyn AccountBackend>,
            Arc::new(NullSubmitter) as Arc<dyn MailSubmitter>,
            "me@example.org",
        ),
    );

    let (received_url, received) = spawn_receiver().await;
    let (flags_url, flags) = spawn_receiver().await;
    let endpoint = |id: &str, url: &str, events: &[&str]| WebhookEndpoint {
        id: id.into(),
        account_id: account_id.clone(),
        url: url.into(),
        secret: b"s".to_vec(),
        events: events.iter().map(|s| s.to_string()).collect(),
    };
    let registry: Arc<dyn WebhookRegistry> = Arc::new(InMemoryWebhookRegistry::new(vec![
        endpoint("r", &received_url, &["message.received"]),
        endpoint("f", &flags_url, &["message.flags"]),
    ]));
    let (_tx, state_changes) = tokio::sync::broadcast::channel(8);
    webhooks::spawn_with(registry, Some(&engine), state_changes, |ip| {
        ip.is_loopback() || mw_egress::ip_allowed(ip)
    });

    // Ingest: the message.received endpoint is called, the flags one is not.
    engine.resync(&account_id).await.unwrap();
    wait_for(&received, 1).await;
    {
        let got = received.0.lock().unwrap().clone();
        assert_eq!(got.len(), 1, "one new message, one delivery: {got:?}");
        assert_eq!(got[0].0, "message.received");
        assert_eq!(got[0].1["type"], "message.received");
        assert_eq!(got[0].1["accountId"], json!(account_id));
    }

    // A flag change: the flags endpoint is called, the received one is not.
    let mailboxes = engine
        .handle_jmap(
            &account_id,
            &json!({ "methodCalls": [["Mailbox/get", {}, "m"]] }),
        )
        .await;
    let inbox = mailboxes["methodResponses"][0][1]["list"]
        .as_array()
        .expect("mailboxes")
        .iter()
        .find(|m| m["role"] == "inbox")
        .expect("inbox")["id"]
        .clone();
    let ids = engine
        .handle_jmap(
            &account_id,
            &json!({ "methodCalls": [["Email/query", { "filter": { "inMailbox": inbox } }, "q"]] }),
        )
        .await;
    let id = ids["methodResponses"][0][1]["ids"][0]
        .as_str()
        .expect("one message")
        .to_string();
    let set = engine
        .handle_jmap(
            &account_id,
            &json!({ "methodCalls": [["Email/set", { "update": { &id: { "keywords/$flagged": true } } }, "s"]] }),
        )
        .await;
    assert!(
        set["methodResponses"][0][1]["updated"]
            .as_object()
            .is_some_and(|u| u.contains_key(&id)),
        "setup: the flag was set: {set}"
    );
    wait_for(&flags, 1).await;
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert_eq!(
        received.0.lock().unwrap().len(),
        1,
        "the flag change did not call the message.received endpoint"
    );
    let got = flags.0.lock().unwrap().clone();
    assert_eq!(got.len(), 1, "{got:?}");
    assert_eq!(got[0].0, "message.flags");
    assert_eq!(got[0].1["ids"], json!([id]));

    // A second delivery of new mail reaches the received endpoint again.
    backend.inbox.lock().unwrap().push(2);
    engine.resync(&account_id).await.unwrap();
    wait_for(&received, 2).await;
    assert_eq!(received.0.lock().unwrap().len(), 2);
    assert_eq!(flags.0.lock().unwrap().len(), 1);
}
