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

// ── t16 (26.16, e3): login second-factor gate ────────────────────────────────

fn totp_now(secret: &[u8]) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    mw_mfa::totp::totp_at(secret, now, &mw_mfa::totp::TotpParams::default())
}

fn has_session_cookie(resp: &reqwest::Response) -> bool {
    resp.headers()
        .get_all(reqwest::header::SET_COOKIE)
        .iter()
        .any(|v| v.to_str().unwrap_or_default().contains("mw_session="))
}

/// The full acceptance path: a user enrols TOTP, and every subsequent login is then
/// gated on the second factor (no password-only downgrade). Recovery codes log in
/// and are single-use; a wrong code is a uniform 401.
#[tokio::test]
async fn totp_enrolment_gates_subsequent_logins() {
    let mock = spawn_mock().await;
    let (server, _web) = spawn_server().await;

    // 1) First login (no factor yet) → a normal cookie session.
    let c = client();
    let resp = do_login(&c, &server, &mock).await;
    assert_eq!(resp.status(), 200);
    assert!(has_session_cookie(&resp), "opt-in user logs in normally");

    // 2) Enrol + confirm TOTP; the confirm response reveals the recovery codes once.
    let begin: Value = c
        .post(format!("{server}/api/account/2fa/totp/begin"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let secret = mw_mfa::totp::base32_decode(begin["secret"].as_str().unwrap()).unwrap();
    let confirm: Value = c
        .post(format!("{server}/api/account/2fa/totp/confirm"))
        .json(&json!({ "code": totp_now(&secret) }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(confirm["ok"], json!(true));
    let recovery: Vec<String> = confirm["recoveryCodes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    assert_eq!(recovery.len(), 10, "DEFAULT_RECOVERY_CODES");

    // 3) A fresh login is now CHALLENGED, not sessioned — no cookie until 2FA clears.
    let c2 = client();
    let resp2 = do_login(&c2, &server, &mock).await;
    assert_eq!(resp2.status(), 200);
    assert!(!has_session_cookie(&resp2), "no session cookie before 2FA");
    let ch: Value = resp2.json().await.unwrap();
    assert_eq!(ch["twofaRequired"], json!(true));
    let pending = ch["pendingToken"].as_str().unwrap().to_string();
    assert!(
        ch["factors"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f == "totp"),
        "totp is an offered factor: {ch}"
    );

    // 4) A valid TOTP completes the login (session cookie issued).
    let step = c2
        .post(format!("{server}/api/login/2fa"))
        .json(&json!({ "pendingToken": pending, "method": "totp", "code": totp_now(&secret) }))
        .send()
        .await
        .unwrap();
    assert_eq!(step.status(), 200);
    assert!(
        has_session_cookie(&step),
        "the second factor issues the session"
    );
    let done: Value = step.json().await.unwrap();
    assert_eq!(done["accountId"], json!(mw_mock_jmap::ACCOUNT_ID));

    // 5) A wrong TOTP is a uniform 401 (pending token stays live for a retry).
    let c3 = client();
    let ch3: Value = do_login(&c3, &server, &mock).await.json().await.unwrap();
    let pending3 = ch3["pendingToken"].as_str().unwrap().to_string();
    let bad = c3
        .post(format!("{server}/api/login/2fa"))
        .json(&json!({ "pendingToken": pending3, "method": "totp", "code": "000000" }))
        .send()
        .await
        .unwrap();
    assert_eq!(bad.status(), 401, "wrong code rejected");

    // 6) A recovery code logs in on the same pending token, and is single-use.
    let good = c3
        .post(format!("{server}/api/login/2fa"))
        .json(&json!({ "pendingToken": pending3, "method": "recovery", "code": recovery[0] }))
        .send()
        .await
        .unwrap();
    assert_eq!(good.status(), 200, "recovery code is break-glass");
    assert!(has_session_cookie(&good));

    // The consumed recovery code cannot be replayed on a new login attempt.
    let c4 = client();
    let ch4: Value = do_login(&c4, &server, &mock).await.json().await.unwrap();
    let pending4 = ch4["pendingToken"].as_str().unwrap().to_string();
    let replay = c4
        .post(format!("{server}/api/login/2fa"))
        .json(&json!({ "pendingToken": pending4, "method": "recovery", "code": recovery[0] }))
        .send()
        .await
        .unwrap();
    assert_eq!(replay.status(), 401, "recovery code is single-use");
}

/// The 2FA challenge withholds the session, and an unknown/opt-out user path is
/// unchanged (no factor, no policy → straight to a session). Also proves the
/// session-management list surfaces the current session without leaking its id.
#[tokio::test]
async fn session_list_marks_current_without_leaking_id() {
    let mock = spawn_mock().await;
    let (server, _web) = spawn_server().await;
    let c = client();
    let login: Value = do_login(&c, &server, &mock).await.json().await.unwrap();
    // Opt-in user, no factor: straight session (unchanged default path).
    assert_eq!(login["ok"], json!(true));

    let list: Value = c
        .get(format!("{server}/api/account/sessions"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let sessions = list["sessions"].as_array().unwrap();
    assert_eq!(sessions.len(), 1, "one active session");
    assert_eq!(sessions[0]["current"], json!(true), "flagged current");
    // The raw opaque session id must never appear in the listing.
    let handle = sessions[0]["handle"].as_str().unwrap();
    assert_eq!(
        handle.len(),
        16,
        "handle is a truncated hash, not the token"
    );
}
