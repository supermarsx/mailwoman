//! End-to-end integration tests for `mw-server`, exercised against the
//! in-repo `mw-mock-jmap` upstream over a real loopback socket. Proves the
//! login → cookie → proxy → sanitize path is genuinely wired (plan §3 e6).

use std::net::SocketAddr;
use std::path::PathBuf;

use serde_json::{Value, json};

use mw_server::{AppConfig, build_app};

/// Spawn the mock JMAP upstream on an ephemeral port; return its base URL.
async fn spawn_mock() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, mw_mock_jmap::router()).await.unwrap();
    });
    format!("http://{addr}")
}

/// Spawn mw-server (with a fresh temp DB + web dir containing an index) on an
/// ephemeral port; return (base URL, web dir so tests can assert its content).
async fn spawn_server() -> (String, PathBuf) {
    // Monotonic counter, not a timestamp: coarse Windows clock resolution lets
    // parallel tests collide on the DB path and race sqlx migrations.
    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let unique = format!(
        "{}-{}",
        std::process::id(),
        SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    );
    let base = std::env::temp_dir().join(format!("mw-server-test-{unique}"));
    let web_dir = base.join("web");
    std::fs::create_dir_all(&web_dir).unwrap();
    std::fs::write(web_dir.join("index.html"), INDEX_HTML).unwrap();
    let db_path = base.join("mw.db");

    let config = AppConfig {
        db_path: db_path.to_string_lossy().into_owned(),
        server_key_hex: None,
        web_dir: Some(web_dir.clone()),
        cookie_secure: false,
        mode: mw_server::ServerMode::Proxy,
        hardening: mw_server::HardeningConfig::default(),
        security: mw_server::SecurityConfig::default(),
    };
    let app = build_app(config).await.unwrap();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (format!("http://{addr}"), web_dir)
}

const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW_TEST_INDEX</div>";

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap()
}

async fn do_login(c: &reqwest::Client, server: &str, mock: &str) -> reqwest::Response {
    c.post(format!("{server}/api/login"))
        .json(&json!({
            "jmapUrl": mock,
            "username": mw_mock_jmap::USER,
            "password": mw_mock_jmap::PASS,
        }))
        .send()
        .await
        .unwrap()
}

#[tokio::test]
async fn login_sets_cookie_and_persists_session() {
    let mock = spawn_mock().await;
    let (server, _web) = spawn_server().await;
    let c = client();

    let resp = do_login(&c, &server, &mock).await;
    assert_eq!(resp.status(), 200);
    assert!(
        resp.headers()
            .get_all(reqwest::header::SET_COOKIE)
            .iter()
            .any(|v| v.to_str().unwrap_or_default().contains("mw_session=")),
        "login must set the mw_session cookie"
    );
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["ok"], json!(true));
    assert_eq!(body["accountId"], json!(mw_mock_jmap::ACCOUNT_ID));
    assert_eq!(body["username"], json!(mw_mock_jmap::USER));

    // The session persists: /api/me now returns the identity via the cookie.
    let me: Value = c
        .get(format!("{server}/api/me"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(me["accountId"], json!(mw_mock_jmap::ACCOUNT_ID));
}

#[tokio::test]
async fn bad_credentials_yield_401() {
    let mock = spawn_mock().await;
    let (server, _web) = spawn_server().await;
    let c = client();
    let resp = c
        .post(format!("{server}/api/login"))
        .json(&json!({ "jmapUrl": mock, "username": "nobody", "password": "wrong" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
}

#[tokio::test]
async fn unauthenticated_proxy_is_rejected() {
    let (server, _web) = spawn_server().await;
    let c = client();
    // No login → no cookie → 401 on every guarded route.
    let resp = c
        .post(format!("{server}/jmap/api"))
        .json(&json!({ "using": [], "methodCalls": [] }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);

    let resp = c.get(format!("{server}/api/me")).send().await.unwrap();
    assert_eq!(resp.status(), 401);
}

#[tokio::test]
async fn authed_proxy_returns_mock_data() {
    let mock = spawn_mock().await;
    let (server, _web) = spawn_server().await;
    let c = client();
    do_login(&c, &server, &mock).await;

    // Rewritten session points the browser back at us.
    let session: Value = c
        .get(format!("{server}/jmap/session"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(session["apiUrl"], json!("/jmap/api"));

    // Mailbox/get via the proxy returns the mock's Inbox + Sent.
    let mailboxes: Value = c
        .post(format!("{server}/jmap/api"))
        .json(&json!({
            "using": ["urn:ietf:params:jmap:core", "urn:ietf:params:jmap:mail"],
            "methodCalls": [["Mailbox/get", { "accountId": mw_mock_jmap::ACCOUNT_ID }, "c0"]]
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let names: Vec<&str> = mailboxes["methodResponses"][0][1]["list"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|m| m["name"].as_str())
        .collect();
    assert!(names.contains(&"Inbox"), "got mailboxes: {names:?}");
    assert!(names.contains(&"Sent"), "got mailboxes: {names:?}");

    // Email/query the Inbox yields the seeded messages.
    let query: Value = c
        .post(format!("{server}/jmap/api"))
        .json(&json!({
            "using": ["urn:ietf:params:jmap:mail"],
            "methodCalls": [["Email/query", { "filter": { "inMailbox": "mb-inbox" } }, "c0"]]
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        query["methodResponses"][0][1]["ids"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
}

#[tokio::test]
async fn sanitize_strips_hostile_script() {
    let mock = spawn_mock().await;
    let (server, _web) = spawn_server().await;
    let c = client();
    do_login(&c, &server, &mock).await;

    let out: Value = c
        .post(format!("{server}/api/sanitize"))
        .json(&json!({ "html": mw_mock_jmap::HOSTILE_HTML }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let html = out["html"].as_str().unwrap();
    assert!(!html.contains("script"), "script survived: {html}");
    assert!(!html.contains("__mw_pwned"), "sentinel survived: {html}");
    assert!(!html.contains("onclick"), "event handler survived: {html}");
    assert!(!html.contains("javascript:"), "js url survived: {html}");
    // Benign content is preserved.
    assert!(html.contains("Invoice"), "content dropped: {html}");
}

#[tokio::test]
async fn static_index_is_served() {
    let (server, _web) = spawn_server().await;
    let c = client();

    let root = c.get(format!("{server}/")).send().await.unwrap();
    assert_eq!(root.status(), 200);
    assert!(root.text().await.unwrap().contains("MW_TEST_INDEX"));

    // Unknown non-asset route falls back to the SPA index.
    let spa = c
        .get(format!("{server}/mailbox/inbox"))
        .send()
        .await
        .unwrap();
    assert_eq!(spa.status(), 200);
    assert!(spa.text().await.unwrap().contains("MW_TEST_INDEX"));

    // Security headers are present (SPEC §7.4).
    let resp = c.get(format!("{server}/healthz")).send().await.unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(resp.headers().get("x-frame-options").unwrap(), "DENY");
    assert!(
        resp.headers()
            .get("content-security-policy")
            .unwrap()
            .to_str()
            .unwrap()
            .contains("default-src 'none'")
    );
}

#[tokio::test]
async fn download_requires_auth() {
    let (server, _web) = spawn_server().await;
    let c = client();
    // No cookie → the download route is guarded like every other JMAP route.
    let resp = c
        .get(format!("{server}/jmap/download/acct-1/blob-e1/file.bin"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
}

#[tokio::test]
async fn download_proxies_upstream_blob_with_injected_auth() {
    let mock = spawn_mock().await;
    let (server, _web) = spawn_server().await;
    let c = client();
    do_login(&c, &server, &mock).await;

    // Proxy mode: the server fetches the upstream downloadUrl (auth injected)
    // and streams the bytes back. The mock echoes the substituted coordinates.
    let resp = c
        .get(format!(
            "{server}/jmap/download/{}/blob-e1/invoice.eml",
            mw_mock_jmap::ACCOUNT_ID
        ))
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
        "application/octet-stream"
    );
    let body = resp.text().await.unwrap();
    assert_eq!(body, "BLOB:acct-1:blob-e1:invoice.eml");
}

#[tokio::test]
async fn upload_route_proxies_to_upstream() {
    let mock = spawn_mock().await;
    let (server, _web) = spawn_server().await;
    let c = client();
    do_login(&c, &server, &mock).await;

    // Proxy mode: the upload is forwarded to the upstream uploadUrl with auth injected
    // and the client's Content-Type, and the upstream `{accountId, blobId, type, size}`
    // JSON is relayed back verbatim (the symmetric counterpart of the download proxy).
    let payload = b"upload-body-bytes";
    let resp = c
        .post(format!("{server}/jmap/upload/{}", mw_mock_jmap::ACCOUNT_ID))
        .header(reqwest::header::CONTENT_TYPE, "text/plain")
        .body(payload.to_vec())
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["accountId"], mw_mock_jmap::ACCOUNT_ID);
    assert_eq!(body["blobId"], format!("upload-{}", payload.len()));
    assert_eq!(body["type"], "text/plain");
    assert_eq!(body["size"], payload.len());
}

#[tokio::test]
async fn upload_over_max_size_is_413() {
    let mock = spawn_mock().await;
    let (server, _web) = spawn_server().await;
    let c = client();
    do_login(&c, &server, &mock).await;

    // A body above the advertised maxSizeUpload (50_000_000) is refused with 413 before
    // any storage write or upstream forwarding — this cap is enforced in both modes.
    let oversize = vec![0u8; 50_000_001];
    let resp = c
        .post(format!("{server}/jmap/upload/{}", mw_mock_jmap::ACCOUNT_ID))
        .header(reqwest::header::CONTENT_TYPE, "application/octet-stream")
        .body(oversize)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 413);
}

#[tokio::test]
async fn upload_route_rejects_foreign_account() {
    let mock = spawn_mock().await;
    let (server, _web) = spawn_server().await;
    let c = client();
    do_login(&c, &server, &mock).await;

    // A session may only upload to its own account (scoped like the download route).
    let resp = c
        .post(format!("{server}/jmap/upload/someone-else"))
        .header(reqwest::header::CONTENT_TYPE, "text/plain")
        .body(b"x".to_vec())
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403);
}

#[tokio::test]
async fn export_is_engine_mode_only_in_proxy() {
    let mock = spawn_mock().await;
    let (server, _web) = spawn_server().await;
    let c = client();
    do_login(&c, &server, &mock).await;

    // Proxy mode has no server-side body store, so /api/export is a clean 501.
    let resp = c
        .get(format!("{server}/api/export/e1?format=eml"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 501);
}

#[tokio::test]
async fn logout_clears_session() {
    let mock = spawn_mock().await;
    let (server, _web) = spawn_server().await;
    let c = client();
    do_login(&c, &server, &mock).await;

    let resp = c.post(format!("{server}/api/logout")).send().await.unwrap();
    assert_eq!(resp.status(), 204);

    // After logout the cookie is cleared and the session is gone.
    let me = c.get(format!("{server}/api/me")).send().await.unwrap();
    assert_eq!(me.status(), 401);
}
