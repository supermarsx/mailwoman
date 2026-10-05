//! t28-e12b — the countersign on an API key's unattended send belongs to an admin,
//! and a key belongs to its owner.
//!
//! `api_keys.unattended_send` is what the `/mcp` send gate reads as "an admin
//! countersigned this key" (`mcp::StoreCountersign`: the column, on a key that is
//! not revoked). Until 26.20 the column was a copy of the
//! `unattended_send` box in the scope the key's own creator posted to
//! `POST /api/keys`, no admin route wrote it, and `POST /api/keys/{prefix}/revoke`
//! revoked any key for any logged-in user.
//!
//! Every leg drives the real router over HTTP in engine mode, with two mailbox
//! users, and every refusal is preceded by the same credential working.
//!
//! Legs:
//!   * minting with the box ticked keeps the request and sets no countersign
//!   * writing an existing key back does not move the countersign, either way
//!   * only the owner revokes a key; anyone else gets the unknown-prefix answer
//!   * `PUT /admin/api-keys/{id}/unattended-send` needs an admin session, is
//!     audited, and refuses a key that did not ask or is revoked
//!
//! Run:
//!   cargo test -p mw-server --test t28_apikey_countersign --locked -- --test-threads=1

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::OnceLock;

use serde_json::{Value, json};
use tokio::sync::{Mutex, MutexGuard};

use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config};
use mw_store::{ApiKeyRow, ServerKey, Store};

mod common;
use common::mock_mail::MockPop3;
use common::test_db;

const KEY_HEX: &str = "28e12b02c3d4e5f60718293a4b5c6d7e28e12b02c3d4e5f60718293a4b5c6d7e";
const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW_T28</div>";

const USER_A: (&str, &str) = ("alice@example.org", "Alice-Passw0rd!");
const USER_B: (&str, &str) = ("bob@example.org", "Bob-Passw0rd!");

// ── harness ──────────────────────────────────────────────────────────────────

/// Serialises the tests in this binary: engine-mode login reads `MW_ENGINE_TLS`.
async fn serial() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let guard = LOCK.get_or_init(Default::default).lock().await;
    // SAFETY: the guard serialises every mutation and every dependent read here.
    unsafe {
        std::env::set_var("MW_ENGINE_TLS", "plaintext");
        for k in ["MW_HEADER_AUTH", "MW_PASSWD_BACKEND", "MW_MCP_RESOURCE"] {
            std::env::remove_var(k);
        }
    }
    guard
}

struct Server {
    base: String,
    db: String,
}

impl Server {
    /// A second handle on the server's database, for reading the column the
    /// routes do not show and for writing a key row back.
    async fn store(&self) -> Store {
        Store::open(&self.db, ServerKey::from_hex(KEY_HEX).unwrap())
            .await
            .expect("open the server's store")
    }
}

/// An engine-mode server with the admin panel on.
async fn spawn_engine() -> Server {
    let dir = test_db::unique_dir("mw-t28-e12b");
    let web: PathBuf = dir.join("web");
    std::fs::create_dir_all(&web).unwrap();
    std::fs::write(web.join("index.html"), INDEX_HTML).unwrap();
    let db = dir.join("mw.db").to_string_lossy().into_owned();
    let config = AppConfig {
        db_path: db.clone(),
        server_key_hex: Some(KEY_HEX.to_string()),
        web_dir: Some(web),
        cookie_secure: false,
        mode: ServerMode::Engine,
        hardening: HardeningConfig::default(),
        security: SecurityConfig::default(),
    };
    let v6 = V6Config {
        admin_enabled: true,
        admin_username: Some("root".into()),
        admin_password: Some("hunter2".into()),
        redis_url: None,
    };
    let app = mw_server::build_app_full(config, v6)
        .await
        .expect("build_app_full")
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
    Server {
        base: format!("http://{addr}"),
        db,
    }
}

fn browser() -> reqwest::Client {
    reqwest::Client::builder()
        .cookie_store(true)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap()
}

fn bare() -> reqwest::Client {
    reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap()
}

/// A logged-in admin-panel client.
async fn admin(base: &str) -> reqwest::Client {
    let c = browser();
    let r = c
        .post(format!("{base}/admin/login"))
        .json(&json!({ "username": "root", "password": "hunter2" }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200, "admin login");
    c
}

/// A mailbox user: their POP3 server (kept alive), a logged-in client, their id.
struct Mailbox {
    _pop: MockPop3,
    client: reqwest::Client,
    account_id: String,
}

async fn mailbox(base: &str, (user, pass): (&str, &str)) -> Mailbox {
    let pop = MockPop3::start(user, pass).await;
    let client = browser();
    let login = client
        .post(format!("{base}/api/login"))
        .json(&json!({ "jmapUrl": pop.url(), "username": user, "password": pass }))
        .send()
        .await
        .unwrap();
    assert_eq!(login.status(), 200, "control: {user} logs in");
    let account_id = login.json::<Value>().await.unwrap()["accountId"]
        .as_str()
        .expect("login names the account")
        .to_string();
    Mailbox {
        _pop: pop,
        client,
        account_id,
    }
}

/// A send-capable MCP scope, with the unattended-send box as given.
fn send_scope(account_id: &str, unattended: bool) -> Value {
    json!({
        "read": true, "send": true, "delete": false,
        "accounts": { "subset": [account_id] }, "folders": "all",
        "mail": true, "pim": false, "ip_allowlist": [], "expires_at": null,
        "rate_limit": null, "mcp_tools": ["mail.search", "mail.send"],
        "unattended_send": unattended,
    })
}

/// A minted key: its bearer token and the record the route returned.
struct Minted {
    token: String,
    prefix: String,
    record: Value,
}

async fn mint(base: &str, owner: &Mailbox, unattended: bool) -> Minted {
    let resp = owner
        .client
        .post(format!("{base}/api/keys"))
        .json(&json!({
            "label": "t28",
            "accountId": owner.account_id,
            "scope": send_scope(&owner.account_id, unattended),
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "control: the owner mints a key");
    let body: Value = resp.json().await.unwrap();
    let token = body["displayToken"]
        .as_str()
        .unwrap_or_else(|| panic!("/api/keys returned no displayToken: {body}"))
        .to_string();
    let record = body["record"].clone();
    let prefix = record["prefix"].as_str().unwrap().to_string();
    assert!(token.starts_with(&format!("mwk_{prefix}.")));
    Minted {
        token,
        prefix,
        record,
    }
}

/// Use a key: `GET /api/v1/mailboxes` with it as the bearer.
async fn key_status(base: &str, token: &str) -> u16 {
    bare()
        .get(format!("{base}/api/v1/mailboxes"))
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
        .status()
        .as_u16()
}

async fn stored(store: &Store, prefix: &str) -> ApiKeyRow {
    store
        .get_api_key(prefix)
        .await
        .unwrap()
        .unwrap_or_else(|| panic!("no api_keys row for {prefix}"))
}

/// The prefixes the `/mcp` send gate treats as countersigned now — the predicate
/// of `mcp::StoreCountersign`, applied to every key.
async fn countersigned(store: &Store) -> Vec<String> {
    store
        .list_api_keys()
        .await
        .unwrap()
        .into_iter()
        .filter(|k| k.unattended_send && k.revoked_at.is_none())
        .map(|k| k.key_prefix)
        .collect()
}

/// The owner's own view of a key in `GET /api/keys`.
async fn listed(base: &str, owner: &Mailbox, prefix: &str) -> Option<Value> {
    let keys: Vec<Value> = owner
        .client
        .get(format!("{base}/api/keys"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    keys.into_iter().find(|k| k["prefix"] == prefix)
}

async fn countersign(
    c: &reqwest::Client,
    base: &str,
    id: &str,
    approved: bool,
) -> reqwest::Response {
    c.put(format!("{base}/admin/api-keys/{id}/unattended-send"))
        .json(&json!({ "approved": approved }))
        .send()
        .await
        .unwrap()
}

async fn revoke(c: &reqwest::Client, base: &str, prefix: &str) -> (u16, Value) {
    let resp = c
        .post(format!("{base}/api/keys/{prefix}/revoke"))
        .send()
        .await
        .unwrap();
    let status = resp.status().as_u16();
    (status, resp.json().await.unwrap_or(Value::Null))
}

async fn audit_actions(admin: &reqwest::Client, base: &str, target: &str) -> Vec<String> {
    let entries: Vec<Value> = admin
        .get(format!("{base}/admin/audit?limit=200"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    // Newest first on the wire; oldest first here.
    entries
        .iter()
        .rev()
        .filter(|e| e["target"] == target && e["actorKind"] == "admin")
        .map(|e| e["action"].as_str().unwrap().to_string())
        .collect()
}

// ── 1. minting does not countersign ──────────────────────────────────────────

#[tokio::test]
async fn minting_with_unattended_send_ticked_does_not_countersign_the_key() {
    let _g = serial().await;
    let srv = spawn_engine().await;
    let base = &srv.base;
    let store = srv.store().await;
    let alice = mailbox(base, USER_A).await;

    let key = mint(base, &alice, true).await;
    assert_eq!(
        key_status(base, &key.token).await,
        200,
        "control: the key works"
    );

    // The request is kept, as a request.
    assert_eq!(key.record["scope"]["unattendedSend"], true);
    assert_eq!(key.record["unattendedSendApproved"], false);

    // The column the send gate reads is clear, and stays clear on use.
    assert!(
        !stored(&store, &key.prefix).await.unattended_send,
        "the creator's own request must not set the admin countersign"
    );
    assert_eq!(
        countersigned(&store).await,
        Vec::<String>::new(),
        "no key is countersigned"
    );
    let row = listed(base, &alice, &key.prefix).await.expect("listed");
    assert_eq!(row["scope"]["unattendedSend"], true);
    assert_eq!(row["unattendedSendApproved"], false);
}

// ── 2. writing a key back does not move the countersign ──────────────────────

#[tokio::test]
async fn writing_an_existing_key_back_does_not_move_the_countersign() {
    let _g = serial().await;
    let srv = spawn_engine().await;
    let base = &srv.base;
    let store = srv.store().await;
    let admin = admin(base).await;
    let alice = mailbox(base, USER_A).await;

    let key = mint(base, &alice, true).await;
    let original = stored(&store, &key.prefix).await;
    assert!(!original.unattended_send);

    // The same prefix written again claiming the countersign: not taken.
    let mut claiming = original.clone();
    claiming.unattended_send = true;
    claiming.last_used_at = Some("2026-10-05T00:00:00Z".into());
    store.put_api_key(&claiming).await.unwrap();
    let after = stored(&store, &key.prefix).await;
    assert_eq!(
        after.last_used_at.as_deref(),
        Some("2026-10-05T00:00:00Z"),
        "control: the write did replace the row"
    );
    assert!(!after.unattended_send, "an upsert must not set the flag");

    // Minting again is a different key; it does not touch this one either.
    let second = mint(base, &alice, true).await;
    assert_ne!(second.prefix, key.prefix);
    assert!(!stored(&store, &key.prefix).await.unattended_send);
    assert!(!stored(&store, &second.prefix).await.unattended_send);

    // Once an admin has countersigned, writing the key back does not clear it.
    assert_eq!(
        countersign(&admin, base, &key.prefix, true).await.status(),
        200
    );
    assert!(stored(&store, &key.prefix).await.unattended_send);
    store.put_api_key(&original).await.unwrap();
    assert!(
        stored(&store, &key.prefix).await.unattended_send,
        "an upsert must not clear the flag"
    );
    assert_eq!(countersigned(&store).await, vec![key.prefix.clone()]);
    assert_eq!(
        key_status(base, &key.token).await,
        200,
        "the key still works"
    );
}

// ── 3. only the owner revokes a key ──────────────────────────────────────────

#[tokio::test]
async fn only_the_owner_revokes_a_key() {
    let _g = serial().await;
    let srv = spawn_engine().await;
    let base = &srv.base;
    let store = srv.store().await;
    let alice = mailbox(base, USER_A).await;
    let bob = mailbox(base, USER_B).await;
    assert_ne!(alice.account_id, bob.account_id);

    let key = mint(base, &alice, false).await;
    let bobs = mint(base, &bob, false).await;
    assert_eq!(key_status(base, &key.token).await, 200, "control: A's key");
    assert_eq!(key_status(base, &bobs.token).await, 200, "control: B's key");

    // What a prefix nobody holds looks like.
    let unknown = revoke(&bob.client, base, "0000000000000000").await;
    assert_eq!(unknown.0, 404, "an unknown prefix: {unknown:?}");

    // B on A's key: the same answer, and nothing happens to the key.
    let foreign = revoke(&bob.client, base, &key.prefix).await;
    assert_eq!(foreign, unknown, "someone else's key looks like no key");
    assert!(stored(&store, &key.prefix).await.revoked_at.is_none());
    assert_eq!(
        key_status(base, &key.token).await,
        200,
        "A's key still works after B tried to revoke it"
    );
    assert!(listed(base, &alice, &key.prefix).await.is_some());

    // No session at all.
    assert_eq!(revoke(&bare(), base, &key.prefix).await.0, 401);
    assert!(stored(&store, &key.prefix).await.revoked_at.is_none());

    // A revokes A's key; B's is untouched.
    let own = revoke(&alice.client, base, &key.prefix).await;
    assert_eq!(own, (200, json!({ "ok": true })));
    assert!(stored(&store, &key.prefix).await.revoked_at.is_some());
    assert_eq!(key_status(base, &key.token).await, 401, "revoked");
    assert!(listed(base, &alice, &key.prefix).await.is_none());
    assert_eq!(key_status(base, &bobs.token).await, 200, "B's key lives");
}

// ── 4. the countersign route ─────────────────────────────────────────────────

#[tokio::test]
async fn the_countersign_is_set_only_with_an_admin_session() {
    let _g = serial().await;
    let srv = spawn_engine().await;
    let base = &srv.base;
    let store = srv.store().await;
    let admin = admin(base).await;
    let alice = mailbox(base, USER_A).await;

    let key = mint(base, &alice, true).await;
    let id = stored(&store, &key.prefix).await.id;

    // Not without a session, not with the owner's mailbox session, not with the key.
    assert_eq!(countersign(&bare(), base, &id, true).await.status(), 401);
    assert_eq!(
        countersign(&alice.client, base, &id, true).await.status(),
        401,
        "a mailbox session is not an admin session"
    );
    let with_key = bare()
        .put(format!("{base}/admin/api-keys/{id}/unattended-send"))
        .bearer_auth(&key.token)
        .json(&json!({ "approved": true }))
        .send()
        .await
        .unwrap();
    assert_eq!(with_key.status(), 401);
    assert!(!stored(&store, &key.prefix).await.unattended_send);
    assert_eq!(audit_actions(&admin, base, &id).await, Vec::<String>::new());

    // An admin approves: the column, the owner's view and the audit log agree.
    let resp = countersign(&admin, base, &id, true).await;
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["unattendedSendApproved"], true);
    assert_eq!(body["prefix"], key.prefix.as_str());
    assert!(stored(&store, &key.prefix).await.unattended_send);
    assert_eq!(countersigned(&store).await, vec![key.prefix.clone()]);
    let row = listed(base, &alice, &key.prefix).await.expect("listed");
    assert_eq!(row["unattendedSendApproved"], true);
    assert_eq!(
        audit_actions(&admin, base, &id).await,
        ["api-key-unattended-send-approved"]
    );

    // And withdraws.
    assert_eq!(countersign(&admin, base, &id, false).await.status(), 200);
    assert!(!stored(&store, &key.prefix).await.unattended_send);
    assert_eq!(
        audit_actions(&admin, base, &id).await,
        [
            "api-key-unattended-send-approved",
            "api-key-unattended-send-withdrawn"
        ]
    );

    // A key whose owner did not ask is not countersigned.
    let plain = mint(base, &alice, false).await;
    assert_eq!(
        countersign(&admin, base, &plain.prefix, true)
            .await
            .status(),
        409
    );
    assert!(!stored(&store, &plain.prefix).await.unattended_send);

    // Nor is a revoked one, nor one that does not exist.
    assert_eq!(revoke(&alice.client, base, &key.prefix).await.0, 200);
    assert_eq!(countersign(&admin, base, &id, true).await.status(), 409);
    assert!(!stored(&store, &key.prefix).await.unattended_send);
    assert_eq!(
        countersign(&admin, base, "0000000000000000", true)
            .await
            .status(),
        404
    );
    assert_eq!(countersigned(&store).await, Vec::<String>::new());
}
