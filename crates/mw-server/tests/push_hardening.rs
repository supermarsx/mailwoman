//! Integration tests for the V2 `mw-server` deltas (plan §3 e10): realtime push
//! over `/jmap/ws` + `/jmap/eventsource`, and the §7.4 hardening (Origin check,
//! strict CSRF double-submit, session timeouts). Everything runs over loopback
//! against the in-repo mock JMAP upstream — no live engine or network.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

use futures_util::StreamExt;
use serde_json::{Value, json};

use mw_engine::StateChange;
use mw_server::{AppConfig, HardeningConfig, PushHandle, build_app_with_push};

const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW</div>";

async fn spawn_mock() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, mw_mock_jmap::router()).await.unwrap();
    });
    format!("http://{addr}")
}

/// Spawn mw-server with the given hardening config; return (base URL, SocketAddr,
/// push handle).
async fn spawn_server(hardening: HardeningConfig) -> (String, SocketAddr, PushHandle) {
    // A monotonic per-process counter guarantees a distinct DB path per test.
    // (A timestamp is NOT reliable here: on Windows SystemTime resolution is
    // coarse enough that parallel tests collide, sharing a SQLite file and
    // racing `sqlx::migrate!` → "UNIQUE constraint failed: _sqlx_migrations".)
    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let unique = format!(
        "{}-{}",
        std::process::id(),
        SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    );
    let base = std::env::temp_dir().join(format!("mw-push-test-{unique}"));
    let web_dir = base.join("web");
    std::fs::create_dir_all(&web_dir).unwrap();
    std::fs::write(web_dir.join("index.html"), INDEX_HTML).unwrap();

    let config = AppConfig {
        db_path: base.join("mw.db").to_string_lossy().into_owned(),
        server_key_hex: None,
        web_dir: Some(PathBuf::from(&web_dir)),
        cookie_secure: false,
        mode: mw_server::ServerMode::Proxy,
        hardening,
        security: mw_server::SecurityConfig::default(),
    };
    let (app, push) = build_app_with_push(config).await.unwrap();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (format!("http://{addr}"), addr, push)
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap()
}

/// Log in and return the raw `mw_session=..` cookie pair (for the WS client,
/// which does not share reqwest's jar).
async fn login_session_cookie(c: &reqwest::Client, server: &str, mock: &str) -> String {
    let resp = c
        .post(format!("{server}/api/login"))
        .json(&json!({
            "jmapUrl": mock,
            "username": mw_mock_jmap::USER,
            "password": mw_mock_jmap::PASS,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "login should succeed");
    set_cookie_pair(&resp, "mw_session").expect("login sets mw_session")
}

fn set_cookie_pair(resp: &reqwest::Response, name: &str) -> Option<String> {
    for v in resp.headers().get_all(reqwest::header::SET_COOKIE) {
        let s = v.to_str().ok()?;
        if let Some(rest) = s.split(';').next()
            && rest.trim_start().starts_with(&format!("{name}="))
        {
            return Some(rest.trim().to_string());
        }
    }
    None
}

fn a_change() -> StateChange {
    StateChange {
        account_id: mw_mock_jmap::ACCOUNT_ID.to_string(),
        email: "42".into(),
        mailbox: "3".into(),
        submission: "1".into(),
        thread: "42".into(),
        crypto_key: "0".into(),
        mail_rule: "0".into(),
    }
}

#[tokio::test]
async fn ws_delivers_state_change_to_authenticated_client() {
    use tokio_tungstenite::tungstenite::Message;
    use tokio_tungstenite::tungstenite::client::IntoClientRequest;
    use tokio_tungstenite::tungstenite::http::header::COOKIE;

    let mock = spawn_mock().await;
    let (server, addr, push) = spawn_server(HardeningConfig::default()).await;
    let c = client();
    let cookie = login_session_cookie(&c, &server, &mock).await;

    let mut req = format!("ws://{addr}/jmap/ws")
        .into_client_request()
        .unwrap();
    req.headers_mut().insert(COOKIE, cookie.parse().unwrap());
    let (mut ws, _resp) = tokio_tungstenite::connect_async(req).await.unwrap();

    // Simulate the engine broadcasting a change.
    assert_eq!(push.send(a_change()), 1, "one WS subscriber is connected");

    // The next non-ping frame is the RFC 8887 StateChange.
    let frame = loop {
        let msg = tokio::time::timeout(Duration::from_secs(5), ws.next())
            .await
            .expect("a frame arrives")
            .expect("stream open")
            .expect("no ws error");
        match msg {
            Message::Text(t) => break t.to_string(),
            Message::Ping(_) | Message::Pong(_) => continue,
            other => panic!("unexpected frame: {other:?}"),
        }
    };
    let wire: Value = serde_json::from_str(&frame).unwrap();
    assert_eq!(wire["@type"], "StateChange");
    assert_eq!(wire["changed"][mw_mock_jmap::ACCOUNT_ID]["Email"], "42");
    assert_eq!(
        wire["changed"][mw_mock_jmap::ACCOUNT_ID]["EmailSubmission"],
        "1"
    );
    // V4: the crypto/rule changed-map keys ride the same StateChange (plan §2.2).
    assert_eq!(wire["changed"][mw_mock_jmap::ACCOUNT_ID]["CryptoKey"], "0");
    assert_eq!(wire["changed"][mw_mock_jmap::ACCOUNT_ID]["MailRule"], "0");
    drop(ws);
}

#[tokio::test]
async fn ws_without_cookie_is_rejected() {
    use tokio_tungstenite::tungstenite::client::IntoClientRequest;

    let (_server, addr, _push) = spawn_server(HardeningConfig::default()).await;
    let req = format!("ws://{addr}/jmap/ws")
        .into_client_request()
        .unwrap();
    let err = tokio_tungstenite::connect_async(req)
        .await
        .expect_err("unauthenticated upgrade must fail");
    let msg = err.to_string();
    assert!(
        msg.contains("401") || msg.to_lowercase().contains("unauthorized"),
        "expected a 401 rejection, got: {msg}"
    );
}

#[tokio::test]
async fn sse_streams_the_same_state_change() {
    let mock = spawn_mock().await;
    let (server, _addr, push) = spawn_server(HardeningConfig::default()).await;
    let c = client();
    login_session_cookie(&c, &server, &mock).await;

    let resp = c
        .get(format!("{server}/jmap/eventsource"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(
        resp.headers()
            .get(reqwest::header::CONTENT_TYPE)
            .unwrap()
            .to_str()
            .unwrap(),
        "text/event-stream"
    );

    // Subscription is live once the response headers are back; now broadcast.
    assert_eq!(push.send(a_change()), 1);

    let mut stream = resp.bytes_stream();
    let mut buf = String::new();
    let found = tokio::time::timeout(Duration::from_secs(5), async {
        while let Some(chunk) = stream.next().await {
            buf.push_str(&String::from_utf8_lossy(&chunk.unwrap()));
            if buf.contains("StateChange") {
                return true;
            }
        }
        false
    })
    .await
    .unwrap_or(false);
    assert!(found, "SSE never delivered a StateChange; got: {buf:?}");
    assert!(buf.contains("data:"), "SSE frame must use data: lines");
}

#[tokio::test]
async fn cross_origin_write_is_rejected() {
    let mock = spawn_mock().await;
    let (server, _addr, _push) = spawn_server(HardeningConfig::default()).await;
    let c = client();
    login_session_cookie(&c, &server, &mock).await;

    // A cross-site Origin on a state-changing request is refused (403), even
    // with a valid session cookie.
    let resp = c
        .post(format!("{server}/jmap/api"))
        .header(reqwest::header::ORIGIN, "https://evil.example")
        .json(&json!({ "using": [], "methodCalls": [] }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403, "cross-origin write must be blocked");
}

#[tokio::test]
async fn strict_csrf_requires_a_matching_token() {
    let hardening = HardeningConfig {
        csrf_strict: true,
        ..HardeningConfig::default()
    };
    let mock = spawn_mock().await;
    let (server, _addr, _push) = spawn_server(hardening).await;
    let c = client();

    // Login is exempt (no prior token) and must still work; capture the csrf.
    let resp = c
        .post(format!("{server}/api/login"))
        .json(&json!({
            "jmapUrl": mock,
            "username": mw_mock_jmap::USER,
            "password": mw_mock_jmap::PASS,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let token = body["csrfToken"].as_str().unwrap().to_string();

    // Without the header: blocked.
    let blocked = c
        .post(format!("{server}/jmap/api"))
        .json(&json!({ "using": [], "methodCalls": [] }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        blocked.status(),
        403,
        "strict CSRF must block a tokenless write"
    );

    // With the matching header (cookie supplied by the jar): allowed through the
    // guard (proxy answers 200).
    let allowed = c
        .post(format!("{server}/jmap/api"))
        .header("x-csrf-token", token)
        .json(&json!({
            "using": ["urn:ietf:params:jmap:core"],
            "methodCalls": []
        }))
        .send()
        .await
        .unwrap();
    assert_ne!(
        allowed.status(),
        403,
        "valid CSRF token must pass the guard"
    );
}

#[tokio::test]
async fn idle_timeout_expires_the_session() {
    // idle=0 → any request after login is already past the idle window.
    let hardening = HardeningConfig {
        idle_timeout: Duration::from_secs(0),
        ..HardeningConfig::default()
    };
    let mock = spawn_mock().await;
    let (server, _addr, _push) = spawn_server(hardening).await;
    let c = client();
    login_session_cookie(&c, &server, &mock).await;

    // The follow-up request is rejected as expired, and the session is gone.
    let me = c.get(format!("{server}/api/me")).send().await.unwrap();
    assert_eq!(me.status(), 401, "idle-expired session must be rejected");
}
