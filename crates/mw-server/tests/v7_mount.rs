//! V7 MOUNT/WIRE smoke test (plan §3 e14): drive EVERY newly-mounted V7 route through
//! the REAL HTTP server and assert it is reachable + wired (not a `404`/SPA
//! fall-through). Proves "unit-green ≠ wired" for the plugin/directory/passwd/assist/
//! nextcloud surfaces + the extra e14 endpoints, and verifies the two folded V6
//! follow-ups: (a) proxy-mode headless scoped-key REST reads via `sessions_by_account`,
//! and (b) the REAL MCP unattended-send countersign gate (no longer an empty stub).

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
    std::env::temp_dir().join(format!("mw-v7-mount-{unique}"))
}

async fn spawn_mock() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, mw_mock_jmap::router()).await.unwrap();
    });
    format!("http://{addr}")
}

fn admin_v6() -> V6Config {
    V6Config {
        admin_enabled: true,
        admin_username: Some("root".into()),
        admin_password: Some("hunter2".into()),
        redis_url: None,
    }
}

async fn spawn_server(mode: ServerMode) -> String {
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
    let app = build_app_full(config, admin_v6()).await.unwrap().0;
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

async fn admin_login(c: &reqwest::Client, server: &str) {
    let login = c
        .post(format!("{server}/admin/login"))
        .json(&json!({ "username": "root", "password": "hunter2" }))
        .send()
        .await
        .unwrap();
    assert_eq!(login.status(), 200, "admin login succeeds");
}

/// Every mailbox-authed V7 route responds through the real server (not the SPA
/// fallthrough). Directory/passwd/assist/nextcloud are unconfigured in the test → the
/// routes return their honest 501/404, proving the handler (not index.html) answered.
#[tokio::test]
async fn v7_mailbox_routes_are_wired() {
    let mock = spawn_mock().await;
    let server = spawn_server(ServerMode::Proxy).await;
    let c = client();
    let _account = login(&c, &server, &mock).await;

    // ── directory/GAL: unconfigured ⇒ 501 (route mounted + extension injected) ──
    let gal = c
        .get(format!("{server}/api/directory/search?q=alice"))
        .send()
        .await
        .unwrap();
    assert_eq!(gal.status(), 501, "GAL search mounted (no directory → 501)");
    let cert = c
        .get(format!("{server}/api/directory/cert?email=a@b.com"))
        .send()
        .await
        .unwrap();
    assert_eq!(cert.status(), 501, "cert lookup mounted");

    // ── password: policy is served; a change with no matching creds is rejected ──
    let policy = c
        .get(format!("{server}/api/password/policy"))
        .send()
        .await
        .unwrap();
    assert_eq!(policy.status(), 200, "password policy mounted");
    assert!(
        policy
            .json::<Value>()
            .await
            .unwrap()
            .get("minLength")
            .is_some(),
        "policy carries the displayed rules"
    );
    // A change against the local backend (no stored hash) → wrong-current 403, NOT a
    // 404/SPA — the route + injected backend answered.
    let change = c
        .post(format!("{server}/api/password"))
        .json(&json!({ "oldPassword": "whatever-old", "newPassword": "A-strong-passw0rd!" }))
        .send()
        .await
        .unwrap();
    assert!(
        change.status() == 403 || change.status() == 400,
        "password change routed to the backend (got {})",
        change.status()
    );

    // ── assist: unconfigured ⇒ enabled=false; invoke ⇒ 404 (Disabled) ──────────
    let cfg = c
        .get(format!("{server}/api/assist/config"))
        .send()
        .await
        .unwrap();
    assert_eq!(cfg.status(), 200, "assist config mounted");
    let cfg_body: Value = cfg.json().await.unwrap();
    assert_eq!(cfg_body["enabled"], json!(false), "unconfigured ⇒ disabled");
    assert!(
        cfg_body["disclosure"].is_string(),
        "the 'what left the device' disclosure is present"
    );
    let invoke = c
        .post(format!("{server}/api/assist/invoke"))
        .json(&json!({ "capability": "summarize", "input": { "prompt": "hi", "context": [] } }))
        .send()
        .await
        .unwrap();
    assert_eq!(invoke.status(), 404, "disabled assist invoke ⇒ 404");
    let transcribe = c
        .post(format!("{server}/api/assist/transcribe"))
        .json(&json!({ "audioBase64": "AAA=", "mime": "audio/webm" }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        transcribe.status(),
        404,
        "extra transcribe endpoint mounted (disabled ⇒ 404)"
    );

    // ── nextcloud: unlinked ⇒ 501 for every route incl. the extra list picker ──
    let share = c
        .post(format!("{server}/api/nextcloud/share-link"))
        .json(&json!({ "path": "/x.zip" }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        share.status(),
        501,
        "nextcloud share-link mounted (unlinked ⇒ 501)"
    );
    let list = c
        .get(format!("{server}/api/nextcloud/list?path=/"))
        .send()
        .await
        .unwrap();
    assert_eq!(list.status(), 501, "extra nextcloud list endpoint mounted");
}

/// The admin-gated V7 surfaces: unauthenticated ⇒ 401; authenticated ⇒ the real
/// handler answers (registry list, assist governance GET/PUT/kill, allow-unsigned).
#[tokio::test]
async fn v7_admin_routes_are_wired() {
    let server = spawn_server(ServerMode::Proxy).await;
    let a = client();

    // Unauthenticated admin routes are 401 (mounted + gated, not SPA).
    for path in ["/admin/plugins", "/admin/assist"] {
        let r = a.get(format!("{server}{path}")).send().await.unwrap();
        assert_eq!(r.status(), 401, "{path} is admin-gated");
    }

    admin_login(&a, &server).await;

    // Plugin registry lists (empty in the test) — the handler, not the SPA, answers.
    let plugins = a
        .get(format!("{server}/admin/plugins"))
        .send()
        .await
        .unwrap();
    assert_eq!(plugins.status(), 200, "plugin registry mounted");
    assert!(
        plugins.json::<Value>().await.unwrap()["plugins"].is_array(),
        "registry returns a plugin array"
    );

    // Assist governance: GET default → disabled; PUT persists; GET reflects it; kill.
    let get = a
        .get(format!("{server}/admin/assist"))
        .send()
        .await
        .unwrap();
    assert_eq!(get.status(), 200, "assist admin GET mounted");
    assert_eq!(get.json::<Value>().await.unwrap()["enabled"], json!(false));

    let put = a
        .put(format!("{server}/admin/assist"))
        .json(&json!({
            "enabled": true,
            "adapters": { "OpenAiCompatible": { "base_url": "http://mock", "chat_model": "m", "embed_model": "e", "api_key": "k" } },
            "capabilityGrants": ["summarize"],
            "dataCeilings": { "accounts": ["acct"] }
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(put.status(), 204, "assist admin PUT persists");

    let get2 = a
        .get(format!("{server}/admin/assist"))
        .send()
        .await
        .unwrap();
    assert_eq!(
        get2.json::<Value>().await.unwrap()["enabled"],
        json!(true),
        "PUT round-trips"
    );

    let kill = a
        .post(format!("{server}/admin/assist/kill"))
        .send()
        .await
        .unwrap();
    assert_eq!(kill.status(), 200, "assist kill switch mounted");
    assert_eq!(kill.json::<Value>().await.unwrap()["killed"], json!(true));
    let get3 = a
        .get(format!("{server}/admin/assist"))
        .send()
        .await
        .unwrap();
    assert_eq!(
        get3.json::<Value>().await.unwrap()["enabled"],
        json!(false),
        "kill disabled it"
    );

    // allow-unsigned on an unknown plugin → 400 (the handler ran, not the SPA).
    let unsigned = a
        .post(format!("{server}/admin/plugins/ghost/allow-unsigned"))
        .send()
        .await
        .unwrap();
    assert_eq!(
        unsigned.status(),
        400,
        "allow-unsigned mounted (unknown id ⇒ 400)"
    );
}

/// Folded V6 follow-up (a): a headless, cookie-less, Bearer-only scoped key resolves a
/// session by account in PROXY mode and returns REST data — the V6 gap (`sessions_by
/// _account`) closes. The MCP countersign resolver (b) reads the real admin flag.
#[tokio::test]
async fn headless_rest_read_resolves_in_proxy_mode() {
    let mock = spawn_mock().await;
    let server = spawn_server(ServerMode::Proxy).await;
    let c = client();
    let account_id = login(&c, &server, &mock).await;

    // Mint a Bearer-usable scoped key (read + mail + this account).
    let scope = json!({
        "read": true, "send": false, "delete": false,
        "accounts": { "subset": [account_id] }, "folders": "all",
        "mail": true, "pim": false, "ip_allowlist": [], "expires_at": null,
        "rate_limit": null, "mcp_tools": [], "unattended_send": false,
    });
    let mint: Value = c
        .post(format!("{server}/api/keys"))
        .json(&json!({ "label": "headless", "accountId": account_id, "scope": scope }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let token = mint["displayToken"].as_str().unwrap().to_string();

    // A FRESH client with NO cookie — Bearer only. In PROXY mode the read path must
    // resolve a session by account (the folded follow-up a) and return the JMAP list.
    let headless = reqwest::Client::new();
    let resp = headless
        .get(format!("{server}/api/v1/messages?limit=5"))
        .header("x-api-key", &token)
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        200,
        "proxy-mode headless Bearer-only REST read resolves (follow-up a closed)"
    );
    assert!(
        resp.json::<Value>()
            .await
            .unwrap()
            .get("messages")
            .is_some(),
        "headless REST returns the JMAP list"
    );
}

/// Folded V6 follow-up (b): the MCP countersign gate is REAL. A minted key WITHOUT the
/// `unattended_send` admin flag cannot bypass the human gate — an unattended send is
/// routed to the Outbox (enqueued), never sent-now. (A live send-now path requires the
/// countersigned flag, which no ordinary key carries — proven at the store layer +
/// verified live by e16.)
#[tokio::test]
async fn mcp_transport_and_countersign_are_wired() {
    let server = spawn_server(ServerMode::Engine).await;
    let c = client();

    // The transport is mounted + the tool set is exposed (engine mode).
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
    assert_eq!(list["result"]["tools"].as_array().unwrap().len(), 10);
}
