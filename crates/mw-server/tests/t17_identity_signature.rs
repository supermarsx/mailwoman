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

fn unique() -> String {
    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    format!(
        "{}-{}",
        std::process::id(),
        SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    )
}

const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW</div>";

async fn spawn_mock() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, mw_mock_jmap::router()).await.unwrap();
    });
    format!("http://{addr}")
}

/// Returns (bound addr, db path). The db path lets the test seed the mock account row
/// the identity FK requires (sqlx enables `foreign_keys` by default).
async fn spawn_server() -> (SocketAddr, String) {
    let base = std::env::temp_dir().join(format!("mw-t17-ident-{}", unique()));
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
        security: mw_server::SecurityConfig::default(),
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

async fn login(c: &reqwest::Client, base: &str, mock: &str) {
    let body: Value = c
        .post(format!("{base}/api/login"))
        .json(&json!({ "jmapUrl": mock, "username": mw_mock_jmap::USER, "password": mw_mock_jmap::PASS }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(body["ok"], json!(true), "plain login (no 2FA): {body}");
}

#[tokio::test]
async fn identity_signature_name_round_trips_through_the_prefs_route() {
    let mock = spawn_mock().await;
    let (addr, db_path) = spawn_server().await;
    seed_mock_account(&db_path).await;
    let base = format!("http://{addr}");
    let c = client();
    login(&c, &base, &mock).await;

    // POST an identity carrying signatureName (the 26.16-dropped, 26.17-persisted field).
    let create = c
        .post(format!("{base}/api/account/identities"))
        .json(&json!({
            "name": "Work",
            "email": "work@example.org",
            "signatureName": "work-template",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(create.status(), 200, "identity create accepted");
    let created: Value = create.json().await.unwrap();
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
    assert_eq!(mine["email"], json!("work@example.org"));

    // An identity POSTed WITHOUT signatureName comes back without it (Option is honoured,
    // not defaulted to a stale value) — proves the column is genuinely per-row.
    let create2: Value = c
        .post(format!("{base}/api/account/identities"))
        .json(&json!({ "name": "Plain", "email": "plain@example.org" }))
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
