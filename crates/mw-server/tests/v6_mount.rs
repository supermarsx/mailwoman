//! V6 MOUNT/WIRE smoke test (plan §3 e11): drive EVERY newly-mounted route through
//! the REAL HTTP server and assert it is reachable + wired (not a `404`/SPA
//! fall-through). Proves "unit-green ≠ wired": admin panel, OAuth/API-key surface,
//! zero-access, REST v1, `/metrics`, `/errors`, inbound webhook, and the MCP
//! Streamable-HTTP transport all respond through `build_app_full`.

use std::net::SocketAddr;
use std::path::PathBuf;

use serde_json::{Value, json};

use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};

const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW_TEST_INDEX</div>";

fn unique_base() -> PathBuf {
    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let unique = format!(
        "{}-{}",
        std::process::id(),
        SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    );
    std::env::temp_dir().join(format!("mw-v6-mount-{unique}"))
}

async fn spawn_mock() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, mw_mock_jmap::router()).await.unwrap();
    });
    format!("http://{addr}")
}

async fn spawn_server(mode: ServerMode, v6: V6Config) -> String {
    let base = unique_base();
    let web_dir = base.join("web");
    std::fs::create_dir_all(&web_dir).unwrap();
    std::fs::write(web_dir.join("index.html"), INDEX_HTML).unwrap();
    let config = AppConfig {
        db_path: base.join("mw.db").to_string_lossy().into_owned(),
        server_key_hex: None,
        web_dir: Some(web_dir),
        cookie_secure: false,
        mode,
        hardening: HardeningConfig::default(),
        security: SecurityConfig::default(),
    };
    let app = build_app_full(config, v6).await.unwrap().0;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    format!("http://{addr}")
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap()
}

fn admin_v6() -> V6Config {
    V6Config {
        admin_enabled: true,
        admin_username: Some("root".into()),
        admin_password: Some("hunter2".into()),
        redis_url: None,
    }
}

async fn login(c: &reqwest::Client, server: &str, mock: &str) -> String {
    let resp = c
        .post(format!("{server}/api/login"))
        .json(&json!({ "jmapUrl": mock, "username": mw_mock_jmap::USER, "password": mw_mock_jmap::PASS }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "mailbox login must succeed");
    let body: Value = resp.json().await.unwrap();
    body["accountId"].as_str().unwrap().to_string()
}

/// The user-facing V6 surfaces (cookie-authed) + the unauthenticated surfaces, all
/// mounted through the real proxy-mode server.
#[tokio::test]
async fn v6_user_and_unauth_routes_are_wired() {
    let mock = spawn_mock().await;
    let server = spawn_server(ServerMode::Proxy, admin_v6()).await;
    let c = client();
    let account_id = login(&c, &server, &mock).await;

    // ── scoped API keys: mint (shown once) → list → revoke ────────────────────
    let scope = json!({
        "read": true, "send": false, "delete": false,
        "accounts": { "subset": [account_id] }, "folders": "all",
        "mail": true, "pim": false, "ip_allowlist": [], "expires_at": null,
        "rate_limit": null, "mcp_tools": [], "unattended_send": false,
    });
    let mint: Value = c
        .post(format!("{server}/api/keys"))
        .json(&json!({ "label": "smoke", "accountId": account_id, "scope": scope }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let token = mint["displayToken"]
        .as_str()
        .expect("display token shown once");
    assert!(token.starts_with("mwk_"), "key wire format: {token}");
    let prefix = mint["record"]["prefix"].as_str().unwrap().to_string();

    let keys: Value = c
        .get(format!("{server}/api/keys"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(
        keys.as_array()
            .unwrap()
            .iter()
            .any(|k| k["prefix"] == prefix),
        "minted key is listed"
    );

    let revoke = c
        .post(format!("{server}/api/keys/{prefix}/revoke"))
        .send()
        .await
        .unwrap();
    assert_eq!(revoke.status(), 200, "key revoke wired");

    // ── OAuth 2.1 endpoints (mounted + functional) ────────────────────────────
    let consent: Value = c
        .post(format!("{server}/oauth/consent"))
        .json(&json!({
            "responseType": "code", "clientId": "unregistered-client",
            "redirectUri": "https://app.example/cb", "codeChallenge": "abc",
            "codeChallengeMethod": "S256", "resource": "https://api.example",
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        consent["approved"],
        json!(false),
        "unregistered client is not approved"
    );

    let tok = c
        .post(format!("{server}/oauth/token"))
        .json(&json!({ "grant_type": "authorization_code", "code": "bogus", "client_id": "x", "code_verifier": "y", "resource": "z", "redirect_uri": "r" }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        tok.status(),
        400,
        "invalid grant is a clean 400 (route mounted)"
    );

    let intro: Value = c
        .post(format!("{server}/oauth/introspect"))
        .json(&json!({ "token": "bogus" }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        intro["active"],
        json!(false),
        "unknown token introspects inactive"
    );

    let rev = c
        .post(format!("{server}/oauth/revoke"))
        .json(&json!({ "token": "bogus" }))
        .send()
        .await
        .unwrap();
    assert_eq!(rev.status(), 200, "revoke is idempotent (RFC 7009)");

    // ── zero-access: enable → status shows ciphertext material, never a key ────
    let enable = c
        .post(format!("{server}/api/zeroaccess/enable"))
        .json(&json!({ "saltB64": "c2FsdA==", "kdfParams": { "mCost": 19456, "tCost": 2, "pCost": 1 }, "wrappedDataKeyB64": "d3JhcHBlZA==" }))
        .send()
        .await
        .unwrap();
    assert_eq!(enable.status(), 200, "zero-access enable wired");
    let za: Value = c
        .get(format!("{server}/api/zeroaccess"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(za["enabled"], json!(true));
    assert!(
        za["wrappedDataKeyB64"].is_string(),
        "server returns wrapped material only"
    );

    // pairing relay (server relays ciphertext only)
    let offer: Value = c
        .post(format!("{server}/api/zeroaccess/pair/offer"))
        .json(&json!({ "publicB64": "cHVi" }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let pairing_id = offer["pairingId"].as_str().unwrap().to_string();
    c.post(format!("{server}/api/zeroaccess/pair/envelope"))
        .json(&json!({ "pairingId": pairing_id, "envelopeB64": "ZW52" }))
        .send()
        .await
        .unwrap();
    let got: Value = c
        .get(format!(
            "{server}/api/zeroaccess/pair/envelope/{pairing_id}"
        ))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        got["envelopeB64"],
        json!("ZW52"),
        "pairing envelope relayed verbatim"
    );

    // ── REST v1 (cookie-authed, translates to the JMAP surface) ───────────────
    let rest = c
        .get(format!("{server}/api/v1/messages?limit=5"))
        .send()
        .await
        .unwrap();
    assert_eq!(
        rest.status(),
        200,
        "/api/v1/messages wired to the JMAP surface"
    );
    let rest_body: Value = rest.json().await.unwrap();
    assert!(
        rest_body.get("messages").is_some(),
        "REST returns the JMAP list verbatim"
    );

    // ── observability + error tunnel + inbound webhook (mounted) ──────────────
    let metrics = c.get(format!("{server}/metrics")).send().await.unwrap();
    assert_eq!(
        metrics.status(),
        401,
        "/metrics is bearer-gated (never open)"
    );

    let errors = c
        .post(format!("{server}/errors"))
        .json(&json!({ "message": "boom", "context": { "subject": "TOP SECRET", "from": "a@b.com" } }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        errors.status(),
        202,
        "/errors scrubber tunnel accepts + scrubs"
    );

    let inbound = c
        .post(format!("{server}/api/webhooks/inbound"))
        .json(&json!({}))
        .send()
        .await
        .unwrap();
    // No inbound secret configured in the test → the handler (not the SPA) returns 404.
    assert_eq!(
        inbound.status(),
        404,
        "inbound webhook handler mounted (disabled → 404)"
    );
}

/// The admin panel: login on the SEPARATE session domain, provision a user, and see
/// the audit entry — real provisioning through the mounted `/admin/*` surface.
#[tokio::test]
async fn admin_panel_is_wired_and_audits() {
    let server = spawn_server(ServerMode::Proxy, admin_v6()).await;
    let a = client();

    // Unauthenticated → 401 gate.
    let gated = a
        .get(format!("{server}/admin/session"))
        .send()
        .await
        .unwrap();
    assert_eq!(gated.status(), 401);

    // Login on the admin session domain.
    let login = a
        .post(format!("{server}/admin/login"))
        .json(&json!({ "username": "root", "password": "hunter2" }))
        .send()
        .await
        .unwrap();
    assert_eq!(login.status(), 200, "admin login succeeds");
    let sess: Value = a
        .get(format!("{server}/admin/session"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(sess["username"], json!("root"));

    // Provision a user → 204, then it appears in the audit log + user list.
    let prov = a
        .post(format!("{server}/admin/users"))
        .json(&json!({ "domain": "example.com", "username": "alice", "quota": { "bytesLimit": 1000, "msgLimit": 50 } }))
        .send()
        .await
        .unwrap();
    assert_eq!(prov.status(), 204, "provision user wired");

    let users: Value = a
        .get(format!("{server}/admin/users"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(
        users
            .as_array()
            .unwrap()
            .iter()
            .any(|u| u["accountId"] == json!("alice@example.com")),
        "provisioned user is listed"
    );

    let audit: Value = a
        .get(format!("{server}/admin/audit?limit=10"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(
        audit
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["action"] == json!("user-provisioned")),
        "provisioning wrote an audit entry"
    );

    // A domain round-trips.
    let save = a
        .put(format!("{server}/admin/domains/example.com"))
        .json(&json!({ "name": "example.com", "upstreamJson": "{}", "allowlist": [], "blocklist": [] }))
        .send()
        .await
        .unwrap();
    assert_eq!(save.status(), 204);
    let domains: Value = a
        .get(format!("{server}/admin/domains"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(
        domains
            .as_array()
            .unwrap()
            .iter()
            .any(|d| d["name"] == json!("example.com"))
    );

    // Wrong password fails closed.
    let bad = a
        .post(format!("{server}/admin/login"))
        .json(&json!({ "username": "root", "password": "wrong" }))
        .send()
        .await
        .unwrap();
    assert_eq!(bad.status(), 401);
}

/// `admin.enabled = false` makes the panel unreachable (401 on every admin route).
#[tokio::test]
async fn admin_disabled_unmounts_the_panel() {
    let v6 = V6Config {
        admin_enabled: false,
        admin_username: Some("root".into()),
        admin_password: Some("hunter2".into()),
        redis_url: None,
    };
    let server = spawn_server(ServerMode::Proxy, v6).await;
    let a = client();
    let sess = a
        .get(format!("{server}/admin/session"))
        .send()
        .await
        .unwrap();
    assert_eq!(sess.status(), 401, "disabled panel: /admin/session is 401");
    let login = a
        .post(format!("{server}/admin/login"))
        .json(&json!({ "username": "root", "password": "hunter2" }))
        .send()
        .await
        .unwrap();
    assert_eq!(login.status(), 401, "disabled panel: login refused");
}

/// The MCP Streamable-HTTP transport is nested at `/mcp` in engine mode: a real
/// JSON-RPC `initialize` + `tools/list` round-trip returns the frozen 10-tool set.
#[tokio::test]
async fn mcp_streamable_http_is_wired() {
    let server = spawn_server(ServerMode::Engine, admin_v6()).await;
    let c = client();

    let init: Value = c
        .post(format!("{server}/mcp"))
        .json(&json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {} }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(init["result"]["serverInfo"]["name"], json!("mailwoman-mcp"));

    let list: Value = c
        .post(format!("{server}/mcp"))
        .json(&json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {} }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let tools = list["result"]["tools"].as_array().expect("tools array");
    assert_eq!(tools.len(), 10, "the frozen §2.4 MCP tool set is exposed");
    assert!(
        tools.iter().any(|t| t["name"] == json!("mail.send")),
        "mail.send is enumerated (gated at call time)"
    );
    // A read tool declares its output untrusted (prompt-injection posture).
    let search = tools
        .iter()
        .find(|t| t["name"] == json!("mail.search"))
        .unwrap();
    assert_eq!(search["_meta"]["untrustedOutput"], json!(true));
}
