//! t14-E-e2e — LEG 2: server-metadata admin write round-trip.
//!
//! Proves the admin-gated `/jmap/api` passthrough (E-mount, plan §Wave-B) reaches
//! a SELECTED provisioned account's REAL Dovecot METADATA store end-to-end, using
//! the broadened `mw_admin_session` cookie Path (`b2ce8ae`, now `Path=/` so the
//! browser attaches it to `/jmap/api`):
//!
//!   1. an admin logs in through the REAL `/admin/login` route,
//!   2. drives `ServerMetadata/set` (a value) against the account's Dovecot backend
//!      through `/jmap/api` with only the admin cookie,
//!   3. `ServerMetadata/get` reflects the value (RFC 5464 GETMETADATA, incl. the
//!      26.13 synchronizing-literal fix), and
//!   4. `ServerMetadata/set` with NIL (no `value`) removes it — a follow-up get no
//!      longer returns the value.
//!
//! This is the "wired" proof: the admin session (a SEPARATE session from a mailbox
//! JMAP session) round-trips metadata to a real server's annotation store via the
//! additive passthrough, which only engages after the normal mailbox-cookie auth
//! fails and is scoped to exactly the four metadata/ACL methods.
//!
//! ## Running
//!   scripts/dovecot-t13/gen-certs.sh
//!   docker compose -f docker-compose.ci.yml up -d --wait dovecot-t13
//!   MW_T14_LIVE=1 cargo test -p mw-server --test t14_metadata_admin -- --nocapture --test-threads=1

use std::net::SocketAddr;
use std::path::PathBuf;

use serde_json::{Value, json};

use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use mw_store::{AccountKind, Credentials, NewAccount, ServerKey, Store};

const IMAP_PLAINTEXT: u16 = 3143; // dovecot-t13 plaintext IMAP (imap_metadata enabled)
const USER: &str = "testuser";
const PASS: &str = "testpass";
const ADMIN_USER: &str = "root";
const ADMIN_PASS: &str = "hunter2";
// build_app_full and the pre-seed store MUST share this key so the sealed account
// credentials the passthrough reconnects with decrypt.
const SERVER_KEY_HEX: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
const ENTRY: &str = "/private/vendor/mw-t14";

fn live() -> bool {
    std::env::var("MW_T14_LIVE").ok().as_deref() == Some("1")
}
fn host() -> String {
    std::env::var("MW_T14_HOST").unwrap_or_else(|_| "127.0.0.1".into())
}

fn unique() -> String {
    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    format!(
        "{}_{}_{}",
        std::process::id(),
        SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    )
}

fn web_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("mw-t14-md-web-{}", unique()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("index.html"),
        "<!doctype html><title>MW</title><div id=app>MW</div>",
    )
    .unwrap();
    dir
}

/// Pre-seed a provisioned account (dovecot-t13 plaintext IMAP) into `db_path`,
/// sealed under `SERVER_KEY_HEX`, and return its account id. `build_app_full` later
/// opens the same db and reconnects this account via `engine_mode::ensure_account`.
async fn seed_account(db_path: &str) -> String {
    let key = ServerKey::from_hex(SERVER_KEY_HEX).unwrap();
    let store = Store::open(db_path, key)
        .await
        .expect("open pre-seed store");
    store
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: &host(),
                port: IMAP_PLAINTEXT,
                tls: "plaintext",
                username: USER,
                sync_policy_json: "{}",
            },
            &Credentials {
                username: USER.into(),
                password: PASS.into(),
            },
        )
        .await
        .unwrap()
}

async fn spawn(db_path: String) -> String {
    let config = AppConfig {
        db_path,
        server_key_hex: Some(SERVER_KEY_HEX.into()),
        web_dir: Some(web_dir()),
        cookie_secure: false,
        mode: ServerMode::Engine,
        hardening: HardeningConfig::default(),
        security: SecurityConfig::default(),
    };
    let v6 = V6Config {
        admin_enabled: true,
        admin_username: Some(ADMIN_USER.into()),
        admin_password: Some(ADMIN_PASS.into()),
        redis_url: None,
    };
    let app = build_app_full(config, v6).await.expect("server boots").0;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    format!("http://{addr}")
}

/// Log in through the REAL `/admin/login` route → the `mw_admin_session` cookie as
/// a ready-to-send `Cookie` header value.
async fn admin_login(c: &reqwest::Client, base: &str) -> String {
    let resp = c
        .post(format!("{base}/admin/login"))
        .json(&json!({ "username": ADMIN_USER, "password": ADMIN_PASS }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "admin login succeeds");
    resp.headers()
        .get(reqwest::header::SET_COOKIE)
        .expect("login sets a cookie")
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_string()
}

/// POST a full JMAP request to `/jmap/api` with only the admin cookie; return the
/// first method-response's args.
async fn admin_jmap(c: &reqwest::Client, base: &str, cookie: &str, call: Value) -> Value {
    let resp = c
        .post(format!("{base}/jmap/api"))
        .header(reqwest::header::COOKIE, cookie)
        .json(&json!({ "using": ["urn:ietf:params:jmap:core"], "methodCalls": [call] }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        200,
        "admin metadata passthrough should reach the account backend"
    );
    let body: Value = resp.json().await.unwrap();
    body["methodResponses"][0][1].clone()
}

#[tokio::test]
async fn admin_server_metadata_set_get_remove_roundtrip_live() {
    if !live() {
        eprintln!("\n[t14 metadata SKIP] MW_T14_LIVE!=1 — real Dovecot METADATA not driven.\n");
        return;
    }
    let db = std::env::temp_dir().join(format!("mw-t14-md-{}.db", unique()));
    let db_path = db.to_string_lossy().into_owned();
    let account_id = seed_account(&db_path).await;
    let base = spawn(db_path).await;
    let c = reqwest::Client::new();
    let cookie = admin_login(&c, &base).await;

    // A unique value so a re-run never reads a stale annotation.
    let value = format!("t14-e2e-{}", unique());

    // 1. SET the annotation against the SELECTED account's Dovecot backend.
    let set = admin_jmap(
        &c,
        &base,
        &cookie,
        json!(["ServerMetadata/set",
            { "accountId": account_id, "mailboxId": Value::Null, "entry": ENTRY, "value": value },
            "s"]),
    )
    .await;
    assert!(
        set.get("updated").and_then(|u| u.get(ENTRY)).is_some(),
        "SET must land in `updated` (reached Dovecot SETMETADATA): {set}"
    );
    assert!(
        set.get("notUpdated")
            .and_then(|n| n.as_object())
            .map(|m| m.is_empty())
            .unwrap_or(true),
        "SET must not report notUpdated: {set}"
    );

    // 2. GET reflects the value (RFC 5464 GETMETADATA + the 26.13 literal fix).
    let got = admin_jmap(
        &c,
        &base,
        &cookie,
        json!(["ServerMetadata/get",
            { "accountId": account_id, "mailboxId": Value::Null, "entries": [ENTRY] },
            "g"]),
    )
    .await;
    let read = got["list"]
        .as_array()
        .expect("get list")
        .iter()
        .find(|e| e["entry"] == ENTRY)
        .and_then(|e| e["value"].as_str());
    assert_eq!(
        read,
        Some(value.as_str()),
        "GETMETADATA must round-trip the exact value written: {got}"
    );

    // 3. NIL (no `value`) removes the annotation.
    let del = admin_jmap(
        &c,
        &base,
        &cookie,
        json!(["ServerMetadata/set",
            { "accountId": account_id, "mailboxId": Value::Null, "entry": ENTRY },
            "d"]),
    )
    .await;
    assert!(
        del.get("updated").and_then(|u| u.get(ENTRY)).is_some(),
        "NIL removal must land in `updated`: {del}"
    );

    // 4. A follow-up GET no longer returns the value (removed).
    let after = admin_jmap(
        &c,
        &base,
        &cookie,
        json!(["ServerMetadata/get",
            { "accountId": account_id, "mailboxId": Value::Null, "entries": [ENTRY] },
            "g2"]),
    )
    .await;
    let still = after["list"]
        .as_array()
        .expect("get list")
        .iter()
        .find(|e| e["entry"] == ENTRY)
        .and_then(|e| e["value"].as_str());
    assert_ne!(
        still,
        Some(value.as_str()),
        "after NIL removal the annotation value must be gone: {after}"
    );
}
