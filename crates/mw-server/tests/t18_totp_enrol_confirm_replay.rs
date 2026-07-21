//! t18-e2 — TOTP enrol-confirm code cannot be replayed at first login (R5).
//!
//! 26.17 bound the LOGIN path against replay (0021 `totp_secrets.last_step`, a CAS
//! `advance_totp_last_step`), but the ENROLMENT-confirmation path verified the code
//! WITHOUT advancing `last_step` — so the very code a user typed to confirm TOTP
//! enrolment could be replayed at the first login within its ~90 s window. 26.18
//! advances `last_step` on enrol-confirm too, closing that window.
//!
//! THIS leg proves the wiring end-to-end over HTTP: enrol a TOTP factor through the
//! real `POST /api/account/2fa/totp/confirm` route, then show the confirming code is
//! REFUSED (uniform 401) at a fresh login while a later-step code still works — i.e.
//! enrolment binds the code, without locking out the legitimate next code.
//!
//! Run:
//!   cargo test -p mw-server --test t18_totp_enrol_confirm_replay -- --nocapture --test-threads=1

use std::net::SocketAddr;
use std::path::PathBuf;

use serde_json::{Value, json};

use mw_mfa::totp::{self, TotpParams};
use mw_server::{AppConfig, build_app};
use mw_store::{ServerKey, Store};

// FIXED key so the seed store + the server seal/unseal the TOTP secret under one key.
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
    let dir = std::env::temp_dir().join(format!("mw-t18-enrol-{}", unique()));
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

/// Begin a fresh 2FA login and return its pendingToken (a new browser each time).
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
        "confirmed factor → 2FA demanded: {body}"
    );
    body["pendingToken"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn enrol_confirm_totp_code_cannot_be_replayed_at_first_login() {
    let db = temp_db();
    let mock = spawn_mock().await;
    let addr = spawn_server(&db).await;
    let base = format!("http://{addr}");

    // Seed an UNCONFIRMED TOTP secret (as `totp/begin` would): the account is NOT yet
    // enrolled, so a password-only login still issues a session to drive enrolment.
    let secret = totp::generate_secret();
    let store = Store::open(&db, ServerKey::from_hex(KEY_HEX).unwrap())
        .await
        .unwrap();
    store
        .put_totp_secret(mw_mock_jmap::ACCOUNT_ID, &secret, false)
        .await
        .unwrap();
    drop(store);

    let params = TotpParams::default();
    let now = now_unix();
    // The confirming code (current step) and a later-step code (still inside the +1
    // window, so it matches without a 30 s wait).
    let code_confirm = totp::totp_at(&secret, now, &params);
    let code_next = totp::totp_at(&secret, now + 30, &params);
    assert_ne!(
        code_confirm, code_next,
        "the two steps yield different codes"
    );

    // STEP 1: password-only login (secret still unconfirmed → not enrolled) → session.
    let c1 = client();
    let login = c1
        .post(format!("{base}/api/login"))
        .json(&login_body(&mock))
        .send()
        .await
        .unwrap();
    assert_eq!(login.status(), 200, "password login runs");
    assert!(
        set_session_cookie(&login),
        "an unconfirmed factor does not gate the session"
    );

    // STEP 2: confirm TOTP enrolment with `code_confirm`. This both confirms the
    // factor AND (the 26.18 fix) advances `last_step` to the confirming code's step.
    let confirm = c1
        .post(format!("{base}/api/account/2fa/totp/confirm"))
        .json(&json!({ "code": code_confirm }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        confirm.status(),
        200,
        "enrol-confirm succeeds with a live code"
    );

    // STEP 3: a brand-new browser logs in and REPLAYS the enrol-confirm code within
    // its window → must be rejected (uniform 401) with no session.
    let c2 = client();
    let p2 = begin_login(&c2, &base, &mock).await;
    let replay = c2
        .post(format!("{base}/api/login/2fa"))
        .json(&json!({ "pendingToken": p2, "method": "totp", "code": code_confirm }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        replay.status(),
        401,
        "REPLAY of the enrol-confirm TOTP code must be rejected at first login"
    );
    assert!(
        !set_session_cookie(&replay),
        "a replayed enrol-confirm code issues NO session"
    );

    // STEP 4: a later-step code still logs in → enrolment burned only the consumed
    // step, not the user (normal enrol+login still works).
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
        "a later-step code still logs in (only the confirmed step is burned)"
    );
    assert!(
        set_session_cookie(&later),
        "the later code issues a session"
    );
}
