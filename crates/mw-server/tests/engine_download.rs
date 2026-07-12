//! Engine-mode HTTP acceptance for `/jmap/download` + `/api/export` (plan §3
//! e14). Env-gated on `GREENMAIL_IMAP` and `#[ignore]` otherwise, mirroring
//! `engine_live.rs`: the CI conformance job runs it against Greenmail, driving
//! the exact routes the browser hits in engine mode.
//!
//! The always-runnable acceptance for the blob/export logic lives in
//! `mw-engine`'s `tests/v2.rs` (`fetch_blob_*`, over a `FakeBackend`); this test
//! proves the HTTP wiring — route registration, cookie auth, content headers —
//! against a real IMAP account.

use std::net::SocketAddr;
use std::path::PathBuf;

use serde_json::{Value, json};

use mw_server::{AppConfig, HardeningConfig, ServerMode, build_app};

async fn spawn_engine_server() -> (String, PathBuf) {
    // Monotonic counter, not a timestamp: coarse Windows clock resolution lets
    // parallel tests collide on the DB path and race sqlx migrations.
    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let unique = format!(
        "{}-{}",
        std::process::id(),
        SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    );
    let base = std::env::temp_dir().join(format!("mw-engine-dl-{unique}"));
    let web_dir = base.join("web");
    std::fs::create_dir_all(&web_dir).unwrap();
    std::fs::write(
        web_dir.join("index.html"),
        "<!doctype html><title>MW</title>",
    )
    .unwrap();

    let config = AppConfig {
        db_path: base.join("mw.db").to_string_lossy().into_owned(),
        server_key_hex: None,
        web_dir: Some(web_dir.clone()),
        cookie_secure: false,
        mode: ServerMode::Engine,
        hardening: HardeningConfig::default(),
    };
    let app = build_app(config).await.unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (format!("http://{addr}"), web_dir)
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap()
}

/// Resolve the inbox mailbox id, then the ids of its messages (possibly empty).
async fn inbox_message_ids(c: &reqwest::Client, server: &str) -> Vec<String> {
    let mb: Value = c
        .post(format!("{server}/jmap/api"))
        .json(&json!({
            "using": ["urn:ietf:params:jmap:mail"],
            "methodCalls": [["Mailbox/get", {}, "mb"]]
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let inbox = mb["methodResponses"][0][1]["list"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["role"] == "inbox")
        .expect("an inbox")["id"]
        .as_str()
        .unwrap()
        .to_string();

    let q: Value = c
        .post(format!("{server}/jmap/api"))
        .json(&json!({
            "using": ["urn:ietf:params:jmap:mail"],
            "methodCalls": [["Email/query", { "filter": { "inMailbox": inbox } }, "q"]]
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    q["methodResponses"][0][1]["ids"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect()
}

#[tokio::test]
#[ignore = "requires a live IMAP server; set GREENMAIL_IMAP=host:port"]
async fn download_and_export_over_http_engine_mode() {
    let Ok(addr) = std::env::var("GREENMAIL_IMAP") else {
        return;
    };
    let user = std::env::var("GREENMAIL_USER").unwrap_or_else(|_| "user@example.org".into());
    let pass = std::env::var("GREENMAIL_PASS").unwrap_or_else(|_| "pass".into());

    let (server, _web) = spawn_engine_server().await;
    let c = client();

    // Engine-mode login reads `jmapUrl` as an `imap://` server URL. The
    // conformance job sets MW_ENGINE_TLS=plaintext for Greenmail.
    let login = c
        .post(format!("{server}/api/login"))
        .json(&json!({ "jmapUrl": format!("imap://{addr}"), "username": user, "password": pass }))
        .send()
        .await
        .unwrap();
    assert_eq!(login.status(), 200, "engine login must succeed");
    let body: Value = login.json().await.unwrap();
    let account = body["accountId"].as_str().unwrap().to_string();

    // A bogus blobId (authed) proves the route is wired and the engine resolves
    // it to nothing → 404 (not a 401, and not an index.html fall-through).
    let missing = c
        .get(format!("{server}/jmap/download/{account}/00deadbeef/x.eml"))
        .send()
        .await
        .unwrap();
    assert_eq!(missing.status(), 404);

    // Exercise the real download + every export format when the inbox has mail.
    let ids = inbox_message_ids(&c, &server).await;
    let Some(id) = ids.first() else {
        return;
    };

    // Whole-message download → RFC822.
    let dl = c
        .get(format!("{server}/jmap/download/{account}/{id}/message.eml"))
        .send()
        .await
        .unwrap();
    assert_eq!(dl.status(), 200);
    assert_eq!(
        dl.headers()
            .get(reqwest::header::CONTENT_TYPE)
            .unwrap()
            .to_str()
            .unwrap(),
        "message/rfc822"
    );
    let raw = dl.bytes().await.unwrap();
    assert!(!raw.is_empty(), "downloaded message body is empty");

    // Every mw-export format is reachable and returns a body.
    for (fmt, ctype) in [
        ("eml", "message/rfc822"),
        ("mbox", "application/mbox"),
        ("txt", "text/plain"),
        ("md", "text/markdown"),
    ] {
        let resp = c
            .get(format!("{server}/api/export/{id}?format={fmt}"))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200, "export {fmt} status");
        let ct = resp
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        assert!(ct.starts_with(ctype), "export {fmt} content-type: {ct}");
        assert!(
            !resp.bytes().await.unwrap().is_empty(),
            "export {fmt} empty"
        );
    }
}
