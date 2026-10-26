//! Integration tests for the V5 push relay + native bearer auth (plan §2.2/§2.3,
//! §3 e5). Exercised over loopback against the in-repo `mw-mock-jmap` upstream and
//! a fake Web Push endpoint — no live engine, no network, no Apple/Google infra.
//!
//! Covers the e5 acceptance:
//!   * native login returns a bearer token + a `/jmap/api` bearer request succeeds
//!     WHILE the cookie/browser path is byte-identical (both asserted);
//!   * a subscription round-trips (+ `/api/push/vapid` serves the public key);
//!   * client-supplied private/plaintext push endpoints are refused before dispatch;
//!   * CORS is OFF by default (the browser response has no `Access-Control-*`).

use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

use axum::Router;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::post;
use serde_json::{Value, json};
use tokio::sync::mpsc;

use mw_engine::StateChange;
use mw_server::{AppConfig, HardeningConfig, PushHandle, build_app_with_push};

mod common;
use common::test_db;

const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW</div>";

/// A well-formed browser subscription: the base64url of a valid uncompressed P-256
/// public key (derived from a fixed valid scalar so it is deterministic) + a
/// 16-byte auth secret. The dispatcher must accept these to encrypt a wake.
fn sample_keys() -> (String, String) {
    use base64::Engine as _;
    use p256::elliptic_curve::sec1::ToSec1Point;
    let secret = p256::SecretKey::from_slice(&[0x11u8; 32]).unwrap();
    let point = secret.public_key().to_sec1_point(false);
    let b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD;
    (b64.encode(point.as_bytes()), b64.encode([0x22u8; 16]))
}

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

async fn spawn_server() -> (String, SocketAddr, PushHandle) {
    let base = test_db::unique_dir("mw-pushv5");
    let web_dir = base.join("web");
    std::fs::create_dir_all(&web_dir).unwrap();
    std::fs::write(web_dir.join("index.html"), INDEX_HTML).unwrap();

    let config = AppConfig {
        db_path: base.join("mw.db").to_string_lossy().into_owned(),
        server_key_hex: None,
        web_dir: Some(PathBuf::from(&web_dir)),
        cookie_secure: false,
        mode: mw_server::ServerMode::Proxy,
        hardening: HardeningConfig::default(),
        security: mw_server::SecurityConfig {
            jmap_upstreams: mock_upstreams(),
            upstream_default: last_mock(),
            ..mw_server::SecurityConfig::default()
        },
    };
    let (app, push) = build_app_with_push(config).await.unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (format!("http://{addr}"), addr, push)
}

/// A fake Web Push endpoint that records the request it receives. Returns its full
/// URL + a receiver the test drains.
async fn spawn_fake_push() -> (String, mpsc::UnboundedReceiver<(HeaderMap, Vec<u8>)>) {
    let (tx, rx) = mpsc::unbounded_channel::<(HeaderMap, Vec<u8>)>();
    let app = Router::new().route("/wake", post(capture)).with_state(tx);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (format!("http://{addr}/wake"), rx)
}

async fn capture(
    State(tx): State<mpsc::UnboundedSender<(HeaderMap, Vec<u8>)>>,
    headers: HeaderMap,
    body: Bytes,
) -> StatusCode {
    let _ = tx.send((headers, body.to_vec()));
    StatusCode::CREATED
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap()
}

/// A client that does NOT keep cookies (proves the bearer path needs no cookie).
fn cookieless_client() -> reqwest::Client {
    reqwest::Client::builder().build().unwrap()
}

fn has_set_cookie(resp: &reqwest::Response, name: &str) -> bool {
    resp.headers()
        .get_all(reqwest::header::SET_COOKIE)
        .iter()
        .any(|v| v.to_str().unwrap_or_default().contains(&format!("{name}=")))
}

#[tokio::test]
async fn native_login_returns_bearer_and_cookie_path_is_byte_identical() {
    let _mock = spawn_mock().await;
    let (server, _addr, _push) = spawn_server().await;

    // ── Native login: bearer token in the body, NO cookies. ──
    let native = cookieless_client()
        .post(format!("{server}/api/login"))
        .json(&json!({
            "username": mw_mock_jmap::USER,
            "password": mw_mock_jmap::PASS,
            "clientType": "native",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(native.status(), 200);
    assert!(
        !has_set_cookie(&native, "mw_session"),
        "native login must NOT set the session cookie"
    );
    let nbody: Value = native.json().await.unwrap();
    let token = nbody["token"]
        .as_str()
        .expect("native login returns a token");
    assert!(!token.is_empty());
    assert_eq!(nbody["accountId"], json!(mw_mock_jmap::ACCOUNT_ID));
    assert!(nbody.get("csrfToken").is_none(), "no CSRF token for bearer");

    // A bearer `/jmap/api` request succeeds with no cookie and no Origin/CSRF.
    let bearer = cookieless_client()
        .post(format!("{server}/jmap/api"))
        .header(reqwest::header::AUTHORIZATION, format!("Bearer {token}"))
        .json(&json!({ "using": ["urn:ietf:params:jmap:core"], "methodCalls": [] }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        bearer.status(),
        200,
        "bearer /jmap/api must proxy successfully"
    );

    // A bogus bearer token is rejected.
    let bad = cookieless_client()
        .post(format!("{server}/jmap/api"))
        .header(reqwest::header::AUTHORIZATION, "Bearer not-a-real-token")
        .json(&json!({ "using": [], "methodCalls": [] }))
        .send()
        .await
        .unwrap();
    assert_eq!(bad.status(), 401, "unknown bearer token must be rejected");

    tokio::time::sleep(Duration::from_secs(4)).await; // independent login obeys configured cooldown
    // ── Browser login: byte-identical to before — sets the session + CSRF cookies,
    // returns a csrfToken, and NO bearer token. ──
    let browser = client()
        .post(format!("{server}/api/login"))
        .json(&json!({
            "username": mw_mock_jmap::USER,
            "password": mw_mock_jmap::PASS,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(browser.status(), 200);
    assert!(
        has_set_cookie(&browser, "mw_session"),
        "browser login still sets the session cookie"
    );
    assert!(has_set_cookie(&browser, "mw_csrf"));
    let bbody: Value = browser.json().await.unwrap();
    assert!(bbody["csrfToken"].is_string());
    assert!(
        bbody.get("token").is_none(),
        "browser login mints no bearer"
    );
}

#[tokio::test]
async fn subscription_round_trips_and_vapid_public_is_served() {
    let mock = spawn_mock().await;
    let (server, _addr, _push) = spawn_server().await;
    let c = client();
    login_cookie(&c, &server, &mock).await;

    // VAPID public key is served (generated on first boot).
    let vapid: Value = c
        .get(format!("{server}/api/push/vapid"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let public_key = vapid["publicKey"]
        .as_str()
        .expect("VAPID public key served");
    assert!(!public_key.is_empty());
    let (p256dh, auth) = sample_keys();

    // Subscribe → { id, vapidPublicKey }; the served key matches.
    let sub: Value = c
        .post(format!("{server}/api/push/subscribe"))
        .json(&json!({
            "transport": "webpush",
            "endpoint": "https://push.example/abc",
            "keys": { "p256dh": p256dh, "auth": auth },
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(sub["id"].is_string());
    assert_eq!(sub["vapidPublicKey"].as_str(), Some(public_key));

    // Re-subscribing the same endpoint is idempotent (still succeeds).
    let resub = c
        .post(format!("{server}/api/push/subscribe"))
        .json(&json!({
            "transport": "webpush",
            "endpoint": "https://push.example/abc",
            "keys": { "p256dh": p256dh, "auth": auth },
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resub.status(), 200);

    // Unsubscribe by endpoint → 204.
    let unsub = c
        .post(format!("{server}/api/push/unsubscribe"))
        .json(&json!({ "endpoint": "https://push.example/abc" }))
        .send()
        .await
        .unwrap();
    assert_eq!(unsub.status(), 204);
}

#[tokio::test]
async fn client_supplied_private_push_endpoint_never_receives_a_wake() {
    let mock = spawn_mock().await;
    let (server, _addr, push) = spawn_server().await;
    let (fake_endpoint, mut rx) = spawn_fake_push().await;
    let c = client();
    login_cookie(&c, &server, &mock).await;

    // Subscribe with the fake endpoint as the push target.
    let (p256dh, auth) = sample_keys();
    let resp = c
        .post(format!("{server}/api/push/subscribe"))
        .json(&json!({
            "transport": "webpush",
            "endpoint": fake_endpoint,
            "keys": { "p256dh": p256dh, "auth": auth },
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);

    // Simulate the engine broadcasting a change for the subscribed account.
    push.send(StateChange {
        account_id: mw_mock_jmap::ACCOUNT_ID.to_string(),
        email: "42".into(),
        mailbox: "3".into(),
        submission: "1".into(),
        thread: "42".into(),
        crypto_key: "0".into(),
        mail_rule: "0".into(),
    });

    assert!(
        tokio::time::timeout(Duration::from_millis(250), rx.recv())
            .await
            .is_err(),
        "no request may reach a client-supplied private endpoint"
    );
}

#[tokio::test]
async fn cors_answers_only_the_native_shell_origins() {
    let (server, _addr, _push) = spawn_server().await;
    let c = client();

    // No CORS headers on a plain response…
    let resp = c.get(format!("{server}/healthz")).send().await.unwrap();
    assert!(
        resp.headers()
            .get(reqwest::header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .is_none(),
        "browser deployments must see no Access-Control-Allow-Origin"
    );

    // …nor for a web page's origin…
    let resp = c
        .get(format!("{server}/healthz"))
        .header(reqwest::header::ORIGIN, "https://evil.example")
        .send()
        .await
        .unwrap();
    assert!(
        resp.headers()
            .get(reqwest::header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .is_none()
    );

    // …but a Tauri shell origin is answered with no `MW_NATIVE_ORIGINS` set (t31
    // D10: the fixed Tauri origins are native origins by default).
    let resp = c
        .get(format!("{server}/healthz"))
        .header(reqwest::header::ORIGIN, "tauri://localhost")
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.headers()
            .get(reqwest::header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .and_then(|v| v.to_str().ok()),
        Some("tauri://localhost")
    );
}

// ── helpers ────────────────────────────────────────────────────────────────

/// Log in (browser/cookie path) so the reqwest cookie jar carries the session.
async fn login_cookie(c: &reqwest::Client, server: &str, _mock: &str) {
    let resp = c
        .post(format!("{server}/api/login"))
        .json(&json!({
            "username": mw_mock_jmap::USER,
            "password": mw_mock_jmap::PASS,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "login should succeed");
    let saved=c.put(format!("{server}/api/account/notifications")).json(&json!({"enabled":true,"rules":[],"quietHours":{"enabled":false,"start":"22:00","end":"07:00","timezone":"UTC"}})).send().await.unwrap();
    assert_eq!(saved.status(), 200, "explicit account notification opt-in");
}
