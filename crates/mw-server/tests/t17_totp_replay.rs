//! t17-e-e2e — TOTP replay-within-window rejection (L1) over the REAL login gate.
//!
//! 26.16's `totp_verify` was stateless: a valid RFC-6238 code could be REPLAYED
//! within its ~90 s validity across login attempts. 26.17 remembers the last-consumed
//! step per account (0021 `totp_secrets.last_step`) and rejects any matched step `<=`
//! it, via a CAS `advance_totp_last_step`. `mw-mfa` unit-tests the counter and
//! `mw-store` unit-tests the CAS; THIS leg proves the LOGIN path is wired: the same
//! code that just logged a user in is refused (uniform 401) on a fresh login, while a
//! later-step code still works.
//!
//! Run:
//!   cargo test -p mw-server --test t17_totp_replay -- --nocapture --test-threads=1

use std::net::SocketAddr;
use std::path::PathBuf;

use serde_json::{Value, json};

use mw_mfa::totp::{self, TotpParams};
use mw_server::{AppConfig, build_app};
use mw_store::{ServerKey, Store};

// FIXED key so the seed store + the server seal/unseal the TOTP secret under the same key.
const KEY_HEX: &str = "0f1e2d3c4b5a69788796a5b4c3d2e1f00f1e2d3c4b5a69788796a5b4c3d2e1f0";

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

async fn spawn_server(db_path: &str) -> SocketAddr {
    let base = PathBuf::from(db_path)
        .parent()
        .unwrap()
        .join(format!("web-{}", unique()));
    std::fs::create_dir_all(&base).unwrap();
    std::fs::write(base.join("index.html"), INDEX_HTML).unwrap();
    let config = AppConfig {
        db_path: db_path.to_string(),
        server_key_hex: Some(KEY_HEX.to_string()),
        web_dir: Some(base),
        cookie_secure: false,
        mode: mw_server::ServerMode::Proxy,
        hardening: mw_server::HardeningConfig::default(),
        security: mw_server::SecurityConfig::default(),
    };
    let app = build_app(config).await.expect("build_app");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    addr
}

fn temp_db() -> String {
    let dir = std::env::temp_dir().join(format!("mw-t17-totp-{}", unique()));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join("mw.db").to_string_lossy().into_owned()
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap()
}

fn login_body(mock: &str) -> Value {
    json!({ "jmapUrl": mock, "username": mw_mock_jmap::USER, "password": mw_mock_jmap::PASS })
}

fn set_session_cookie(resp: &reqwest::Response) -> bool {
    resp.headers()
        .get_all(reqwest::header::SET_COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .any(|c| c.starts_with("mw_session="))
}

fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Begin a fresh login and return its pendingToken (a new browser each time).
async fn begin_login(c: &reqwest::Client, base: &str, mock: &str) -> String {
    let body: Value = c
        .post(format!("{base}/api/login"))
        .json(&login_body(mock))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        body["twofaRequired"],
        json!(true),
        "enrolled → 2FA demanded: {body}"
    );
    body["pendingToken"].as_str().unwrap().to_string()
}

async fn submit_totp(
    c: &reqwest::Client,
    base: &str,
    pending: &str,
    code: &str,
) -> reqwest::StatusCode {
    c.post(format!("{base}/api/login/2fa"))
        .json(&json!({ "pendingToken": pending, "method": "totp", "code": code }))
        .send()
        .await
        .unwrap()
        .status()
}

#[tokio::test]
async fn a_totp_code_cannot_be_replayed_within_its_window_on_login() {
    let db = temp_db();
    let mock = spawn_mock().await;
    let addr = spawn_server(&db).await;
    let base = format!("http://{addr}");

    // Seed a CONFIRMED TOTP secret for the mock account (as enrolment would).
    let secret = totp::generate_secret();
    let store = Store::open(&db, ServerKey::from_hex(KEY_HEX).unwrap())
        .await
        .unwrap();
    store
        .put_totp_secret(mw_mock_jmap::ACCOUNT_ID, &secret, true)
        .await
        .unwrap();
    drop(store);

    let params = TotpParams::default();
    let now = now_unix();
    // The code for the CURRENT step, and a code for the NEXT step (a later counter,
    // still inside the verifier's +1 window so it matches without a 30 s wait).
    let code_now = totp::totp_at(&secret, now, &params);
    let code_next = totp::totp_at(&secret, now + 30, &params);
    assert_ne!(code_now, code_next, "the two steps yield different codes");

    // Login #1: the current code clears the factor → session issued, step consumed.
    let c1 = client();
    let p1 = begin_login(&c1, &base, &mock).await;
    let s1 = submit_totp(&c1, &base, &p1, &code_now).await;
    assert_eq!(s1, 200, "a fresh TOTP code logs in");

    // Login #2 (a brand-new browser): REPLAY the very same code within its window.
    let c2 = client();
    let p2 = begin_login(&c2, &base, &mock).await;
    let replay = c2
        .post(format!("{base}/api/login/2fa"))
        .json(&json!({ "pendingToken": p2, "method": "totp", "code": code_now }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        replay.status(),
        401,
        "REPLAY of an already-consumed TOTP code must be rejected (uniform 401)"
    );
    assert!(
        !set_session_cookie(&replay),
        "a replayed code issues NO session"
    );

    // Login #3: a later-step code (step advanced) still works → the guard rejects only
    // the consumed step, not the user.
    let c3 = client();
    let p3 = begin_login(&c3, &base, &mock).await;
    let later = c3
        .post(format!("{base}/api/login/2fa"))
        .json(&json!({ "pendingToken": p3, "method": "totp", "code": code_next }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        later.status(),
        200,
        "a later-step code still logs in (only the consumed step is burned)"
    );
    assert!(
        set_session_cookie(&later),
        "the later code issues a session"
    );
}
