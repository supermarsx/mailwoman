//! Integration tests for the V4 `mw-server` crypto/security endpoints (plan §3
//! e7): WKD publishing, ARF report submission, DLP config load, the watermark
//! honesty overlay, and the crypto/mail-rule realtime push path. Everything runs
//! over loopback in proxy mode against the in-repo mock JMAP upstream; the parts
//! that require the local engine store (the ARF report body) are covered at the
//! HTTP surface here and end-to-end by e10.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

use futures_util::StreamExt;
use serde_json::{Value, json};

use mw_engine::StateChange;
use mw_server::{
    AppConfig, HardeningConfig, PushHandle, SecurityConfig, WatermarkConfig, build_app_with_push,
    wkd,
};

const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW</div>";

async fn spawn_mock() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, mw_mock_jmap::router()).await.unwrap();
    });
    format!("http://{addr}")
}

/// Spawn mw-server (proxy mode) with an explicit [`SecurityConfig`], returning
/// (base URL, SocketAddr, push handle, temp dir root for cleanup-free scoping).
async fn spawn_server(security: SecurityConfig) -> (String, SocketAddr, PushHandle) {
    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let unique = format!(
        "{}-{}",
        std::process::id(),
        SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    );
    let base = std::env::temp_dir().join(format!("mw-sec-test-{unique}"));
    let web_dir = base.join("web");
    std::fs::create_dir_all(&web_dir).unwrap();
    std::fs::write(web_dir.join("index.html"), INDEX_HTML).unwrap();

    let config = AppConfig {
        db_path: base.join("mw.db").to_string_lossy().into_owned(),
        server_key_hex: None,
        web_dir: Some(web_dir),
        cookie_secure: false,
        mode: mw_server::ServerMode::Proxy,
        hardening: HardeningConfig::default(),
        security,
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

/// Log in; returns the `mw_session=…` cookie pair for the WS handshake.
async fn login(c: &reqwest::Client, server: &str, mock: &str) -> String {
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
    for v in resp.headers().get_all(reqwest::header::SET_COOKIE) {
        if let Ok(s) = v.to_str()
            && let Some(pair) = s.split(';').next()
            && pair.trim_start().starts_with("mw_session=")
        {
            return pair.trim().to_string();
        }
    }
    panic!("login did not set mw_session");
}

/// A temp directory unique to the caller.
fn tempdir(tag: &str) -> PathBuf {
    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let p = std::env::temp_dir().join(format!("mw-sec-{tag}-{}-{n}", std::process::id()));
    std::fs::create_dir_all(&p).unwrap();
    p
}

// ── WKD publishing ────────────────────────────────────────────────────────────

#[tokio::test]
async fn wkd_advanced_method_serves_a_published_key() {
    let dir = tempdir("wkd");
    let key = b"\x98\x33binary-openpgp-public-key-bytes".to_vec();
    std::fs::write(dir.join("alice@example.org"), &key).unwrap();

    let security = SecurityConfig {
        wkd_dir: Some(dir),
        ..SecurityConfig::default()
    };
    let (server, _addr, _push) = spawn_server(security).await;
    let c = reqwest::Client::new();

    let hash = wkd::wkd_hash("alice");
    let resp = c
        .get(format!(
            "{server}/.well-known/openpgpkey/example.org/hu/{hash}"
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "WKD serves the provisioned identity");
    assert_eq!(
        resp.headers()
            .get(reqwest::header::CONTENT_TYPE)
            .unwrap()
            .to_str()
            .unwrap(),
        "application/octet-stream"
    );
    assert_eq!(
        resp.headers()
            .get(reqwest::header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .unwrap(),
        "*"
    );
    assert_eq!(resp.bytes().await.unwrap().as_ref(), key.as_slice());
}

#[tokio::test]
async fn wkd_direct_method_uses_host_header() {
    let dir = tempdir("wkd");
    let key = b"direct-method-key".to_vec();
    // The direct method derives the domain from Host, which in loopback tests is
    // 127.0.0.1 — publish the identity under that domain.
    std::fs::write(dir.join("bob@127.0.0.1"), &key).unwrap();
    let security = SecurityConfig {
        wkd_dir: Some(dir),
        ..SecurityConfig::default()
    };
    let (server, _addr, _push) = spawn_server(security).await;
    let c = reqwest::Client::new();

    let hash = wkd::wkd_hash("bob");
    let resp = c
        .get(format!("{server}/.well-known/openpgpkey/hu/{hash}"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(resp.bytes().await.unwrap().as_ref(), key.as_slice());
}

#[tokio::test]
async fn wkd_unknown_key_is_404() {
    let dir = tempdir("wkd");
    let security = SecurityConfig {
        wkd_dir: Some(dir),
        ..SecurityConfig::default()
    };
    let (server, _addr, _push) = spawn_server(security).await;
    let c = reqwest::Client::new();
    let hash = wkd::wkd_hash("nobody");
    let resp = c
        .get(format!(
            "{server}/.well-known/openpgpkey/example.org/hu/{hash}"
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
    // WKD 404s still carry the permissive CORS header.
    assert_eq!(
        resp.headers()
            .get(reqwest::header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .unwrap(),
        "*"
    );
}

#[tokio::test]
async fn wkd_policy_file_exists() {
    let (server, _addr, _push) = spawn_server(SecurityConfig::default()).await;
    let c = reqwest::Client::new();
    let resp = c
        .get(format!("{server}/.well-known/openpgpkey/policy"))
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        200,
        "policy file signals a WKD-enabled domain"
    );
    let resp = c
        .get(format!(
            "{server}/.well-known/openpgpkey/example.org/policy"
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn wkd_rejects_path_traversal() {
    let dir = tempdir("wkd");
    let security = SecurityConfig {
        wkd_dir: Some(dir),
        ..SecurityConfig::default()
    };
    let (server, _addr, _push) = spawn_server(security).await;
    let c = reqwest::Client::new();
    // An invalid hash (wrong length / non-zbase32) is rejected before any lookup.
    let resp = c
        .get(format!(
            "{server}/.well-known/openpgpkey/example.org/hu/not-a-valid-hash"
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
}

// ── DLP config load ───────────────────────────────────────────────────────────

const DLP_RULES: &str = r#"[
  {"id":"pan-block","name":"Card number block","enabled":true,"priority":10,
   "conditions":{"detectors":["pan"],"customRegex":null,"dictionaries":[],
     "attachmentTypes":[],"maxAttachmentSize":null,"recipientDomains":[],
     "recipientDomainMode":null,"classification":null},
   "action":"block","message":"Card numbers may not leave the organisation."}
]"#;

#[tokio::test]
async fn dlp_config_surfaces_active_rules() {
    let mock = spawn_mock().await;
    let security = SecurityConfig {
        dlp_rules: Some(DLP_RULES.to_string()),
        ..SecurityConfig::default()
    };
    let (server, _addr, _push) = spawn_server(security).await;
    let c = client();
    login(&c, &server, &mock).await;

    let resp = c
        .get(format!("{server}/api/security/dlp/config"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["count"], 1);
    let rule = &body["list"][0];
    assert_eq!(rule["id"], "pan-block");
    assert_eq!(rule["action"], "block");
    assert_eq!(rule["conditions"]["detectors"][0], "pan");
}

#[tokio::test]
async fn dlp_config_empty_when_unconfigured() {
    let mock = spawn_mock().await;
    let (server, _addr, _push) = spawn_server(SecurityConfig::default()).await;
    let c = client();
    login(&c, &server, &mock).await;
    let resp = c
        .get(format!("{server}/api/security/dlp/config"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["count"], 0);
    assert!(body["list"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn dlp_config_requires_auth() {
    let (server, _addr, _push) = spawn_server(SecurityConfig::default()).await;
    let c = client();
    let resp = c
        .get(format!("{server}/api/security/dlp/config"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
}

// ── Watermark honesty overlay ─────────────────────────────────────────────────

#[tokio::test]
async fn watermark_config_is_honest() {
    let mock = spawn_mock().await;
    let security = SecurityConfig {
        watermark: WatermarkConfig {
            enabled: true,
            opacity: 0.1,
        },
        ..SecurityConfig::default()
    };
    let (server, _addr, _push) = spawn_server(security).await;
    let c = client();
    login(&c, &server, &mock).await;

    let resp = c
        .get(format!("{server}/api/security/watermark"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["enabled"], true);
    assert_eq!(body["honest"], true);
    assert_eq!(body["identity"], mw_mock_jmap::USER);
    let note = body["note"].as_str().unwrap();
    assert!(
        note.contains("cannot prevent"),
        "note must be honest: {note}"
    );
}

#[tokio::test]
async fn watermark_config_requires_auth() {
    let (server, _addr, _push) = spawn_server(SecurityConfig::default()).await;
    let c = client();
    let resp = c
        .get(format!("{server}/api/security/watermark"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
}

// ── ARF report submission ─────────────────────────────────────────────────────

#[tokio::test]
async fn arf_report_requires_auth() {
    let (server, _addr, _push) = spawn_server(SecurityConfig::default()).await;
    let c = client();
    let resp = c
        .post(format!("{server}/api/security/report"))
        .json(&json!({"emailId":"1","kind":"phishing"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
}

#[tokio::test]
async fn arf_report_501_without_abuse_address() {
    let mock = spawn_mock().await;
    let (server, _addr, _push) = spawn_server(SecurityConfig::default()).await;
    let c = client();
    login(&c, &server, &mock).await;
    let resp = c
        .post(format!("{server}/api/security/report"))
        .json(&json!({"emailId":"1","kind":"phishing"}))
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        501,
        "ARF needs MW_ABUSE_ADDRESS to be configured"
    );
}

#[tokio::test]
async fn arf_report_requires_engine_mode() {
    let mock = spawn_mock().await;
    let security = SecurityConfig {
        abuse_address: Some("abuse@example.org".to_string()),
        ..SecurityConfig::default()
    };
    let (server, _addr, _push) = spawn_server(security).await;
    let c = client();
    login(&c, &server, &mock).await;
    // Proxy mode has no local store to fetch the reported message from.
    let resp = c
        .post(format!("{server}/api/security/report"))
        .json(&json!({"emailId":"1","kind":"junk"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 501);
}

#[tokio::test]
async fn arf_report_rejects_bad_kind() {
    let mock = spawn_mock().await;
    let security = SecurityConfig {
        abuse_address: Some("abuse@example.org".to_string()),
        ..SecurityConfig::default()
    };
    let (server, _addr, _push) = spawn_server(security).await;
    let c = client();
    login(&c, &server, &mock).await;
    let resp = c
        .post(format!("{server}/api/security/report"))
        .json(&json!({"emailId":"1","kind":"nonsense"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
}

// ── Crypto/MailRule realtime push ─────────────────────────────────────────────

/// The push channel is datatype-agnostic (`StateChange::to_wire`), so a
/// `CryptoKey`/`MailRule` mutation broadcasts over the SAME verified WS/SSE path a
/// mail/PIM change uses. When e6 extends `StateChange` with the `CryptoKey`/
/// `MailRule` changed keys they flow through this loop unchanged — the server adds
/// nothing. This test proves the wire path with an injected change.
fn a_change() -> StateChange {
    StateChange {
        account_id: mw_mock_jmap::ACCOUNT_ID.to_string(),
        email: "9".into(),
        mailbox: "4".into(),
        submission: "1".into(),
        thread: "9".into(),
        crypto_key: "3".into(),
        mail_rule: "2".into(),
    }
}

#[tokio::test]
async fn crypto_state_change_reaches_a_ws_client() {
    use tokio_tungstenite::tungstenite::Message;
    use tokio_tungstenite::tungstenite::client::IntoClientRequest;
    use tokio_tungstenite::tungstenite::http::header::COOKIE;

    let mock = spawn_mock().await;
    let (server, addr, push) = spawn_server(SecurityConfig::default()).await;
    let c = client();
    let cookie = login(&c, &server, &mock).await;

    let mut req = format!("ws://{addr}/jmap/ws")
        .into_client_request()
        .unwrap();
    req.headers_mut().insert(COOKIE, cookie.parse().unwrap());
    let (mut ws, _resp) = tokio_tungstenite::connect_async(req).await.unwrap();

    // Simulate the engine broadcasting after a key/rule change.
    assert_eq!(push.send(a_change()), 1, "one WS subscriber connected");

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
    assert!(wire["changed"][mw_mock_jmap::ACCOUNT_ID].is_object());
    // The crypto/rule state tokens reach the client in the changed map (plan §2.2).
    assert_eq!(wire["changed"][mw_mock_jmap::ACCOUNT_ID]["CryptoKey"], "3");
    assert_eq!(wire["changed"][mw_mock_jmap::ACCOUNT_ID]["MailRule"], "2");
    drop(ws);
}
