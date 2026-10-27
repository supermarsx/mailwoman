//! t17-e-e2e — Identity.signatureName persistence (0020 column) over the REAL
//! prefs HTTP route.
//!
//! 26.16 accepted `signatureName` on an identity then dropped it (no column). 26.17
//! adds the 0020 `identities.signature_name` column and wires it through `v2.rs` +
//! `prefs_routes.rs`. This leg proves the WHOLE path: a real `POST
//! /api/account/identities` with `signatureName` set, behind a real session, then a
//! `GET` that returns the same value — persisted through the new column and the prefs
//! route (not just unit-tested in isolation).

use std::net::SocketAddr;
use std::path::PathBuf;

use serde_json::{Value, json};
use sqlx::sqlite::SqliteConnectOptions;

use mw_server::{AppConfig, build_app};

mod common;
use common::test_db;

const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW</div>";

/// Origins of the mock upstreams this test binary has started. Proxy mode refuses a
/// loopback JMAP upstream unless the operator allowlist names it (t24 B2), so the
/// server under test lists these in `SecurityConfig::jmap_upstreams` — the
/// production control an operator uses for an internal upstream, not a test switch.
/// A test must start its mock before its server.
static MOCK_UPSTREAMS: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());

/// The mock upstreams started so far, as an allowlist.
fn mock_upstreams() -> Option<Vec<String>> {
    Some(MOCK_UPSTREAMS.lock().unwrap().clone())
}

// The mock this test started last. The server under test signs in to it as
// its configured upstream (`MW_UPSTREAM_DEFAULT`): a login can no longer name
// one (t32-h9). Per thread, so tests running in parallel keep their own.
thread_local! {
    static LAST_MOCK: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}

fn last_mock() -> Option<String> {
    LAST_MOCK.with(|m| m.borrow().clone())
}

async fn spawn_mock() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, mw_mock_jmap::router()).await.unwrap();
    });
    let origin = format!("http://{addr}");
    MOCK_UPSTREAMS.lock().unwrap().push(origin.clone());
    LAST_MOCK.with(|m| *m.borrow_mut() = Some(origin.clone()));
    origin
}

/// Returns (bound addr, db path). The db path lets the test seed the mock account row
/// the identity FK requires (sqlx enables `foreign_keys` by default).
async fn spawn_server() -> (SocketAddr, String) {
    let base = test_db::unique_dir("mw-t17-ident");
    let web = base.join("web");
    std::fs::create_dir_all(&web).unwrap();
    std::fs::write(web.join("index.html"), INDEX_HTML).unwrap();
    let db_path = base.join("mw.db").to_string_lossy().into_owned();
    let config = AppConfig {
        db_path: db_path.clone(),
        server_key_hex: None,
        web_dir: Some(web as PathBuf),
        cookie_secure: false,
        mode: mw_server::ServerMode::Proxy,
        hardening: mw_server::HardeningConfig::default(),
        security: mw_server::SecurityConfig {
            jmap_upstreams: mock_upstreams(),
            upstream_default: last_mock(),
            ..mw_server::SecurityConfig::default()
        },
    };
    let app = build_app(config).await.unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (addr, db_path)
}

/// Seed the `accounts` row for the mock account so the `identities` FK is satisfied
/// (the login derives `account_id == mw_mock_jmap::ACCOUNT_ID`, but proxy-mode login
/// does not create an accounts row — a real deployment always has one).
async fn seed_mock_account(db_path: &str) {
    let opts = SqliteConnectOptions::new().filename(db_path);
    let pool = sqlx::SqlitePool::connect_with(opts).await.unwrap();
    sqlx::query(
        "INSERT OR IGNORE INTO accounts (id, kind, host, port, tls, username, sealed_creds, sync_policy_json)
         VALUES (?1, 'imap', 'h', 993, 'implicit', ?2, ?3, '{}')",
    )
    .bind(mw_mock_jmap::ACCOUNT_ID)
    .bind(mw_mock_jmap::USER)
    .bind(vec![0u8; 8])
    .execute(&pool)
    .await
    .unwrap();
    pool.close().await;
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap()
}

/// Log in and return the session's `storageAuthority`, which identity writes must
/// send back (0a11b581).
async fn login(c: &reqwest::Client, base: &str, _mock: &str) -> String {
    let body: Value = c
        .post(format!("{base}/api/login"))
        .json(&json!({ "username": mw_mock_jmap::USER, "password": mw_mock_jmap::PASS }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(body["ok"], json!(true), "plain login (no 2FA): {body}");
    common::auth::storage_authority(&body)
}

#[tokio::test]
async fn identity_signature_name_round_trips_through_the_prefs_route() {
    let mock = spawn_mock().await;
    let (addr, db_path) = spawn_server().await;
    seed_mock_account(&db_path).await;
    let base = format!("http://{addr}");
    let c = client();
    let authority = login(&c, &base, &mock).await;

    // POST an identity carrying signatureName (the 26.16-dropped, 26.17-persisted field).
    // Its address is the upstream's own identity: since 16bc4191 a saved identity must
    // name an address the upstream's Identity/get lists.
    let create = c
        .post(format!("{base}/api/account/identities"))
        .header(common::auth::STORAGE_AUTHORITY_HEADER, &authority)
        .json(&json!({
            "name": "Work",
            "email": mw_mock_jmap::USER,
            "signatureName": "work-template",
        }))
        .send()
        .await
        .unwrap();
    let status = create.status();
    let created: Value = create.json().await.unwrap_or(Value::Null);
    assert_eq!(status, 200, "identity create accepted: {created}");
    assert_eq!(created["ok"], json!(true), "create ok: {created}");
    let id = created["id"]
        .as_str()
        .expect("server assigned an id")
        .to_string();

    // GET the identities back → signatureName survives (0020 column + prefs route).
    let list: Value = c
        .get(format!("{base}/api/account/identities"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let identities = list["identities"].as_array().expect("identities array");
    let mine = identities
        .iter()
        .find(|i| i["id"] == json!(id))
        .unwrap_or_else(|| panic!("created identity present: {list}"));
    assert_eq!(
        mine["signatureName"],
        json!("work-template"),
        "signatureName persisted + round-tripped through the prefs route: {mine}"
    );
    assert_eq!(mine["email"], json!(mw_mock_jmap::USER));

    // An identity POSTed WITHOUT signatureName comes back without it (Option is honoured,
    // not defaulted to a stale value) — proves the column is genuinely per-row.
    let create2: Value = c
        .post(format!("{base}/api/account/identities"))
        .header(common::auth::STORAGE_AUTHORITY_HEADER, &authority)
        .json(&json!({ "name": "Plain", "email": mw_mock_jmap::USER }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let id2 = create2["id"].as_str().unwrap().to_string();
    let list2: Value = c
        .get(format!("{base}/api/account/identities"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let plain = list2["identities"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["id"] == json!(id2))
        .unwrap();
    assert!(
        plain.get("signatureName").is_none() || plain["signatureName"].is_null(),
        "an identity with no signatureName does not fabricate one: {plain}"
    );
}
