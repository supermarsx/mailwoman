//! t18-e-e2e — a no-resource OAuth token is REJECTED at `/mcp` (R3), LIVE, while an
//! API key stays exempt.
//!
//! 26.17 made MCP audience enforcement default-on and rejected a WRONG-audience token.
//! 26.18 (R3) closes the enforcement side of the issuance invariant: issuance MANDATES
//! an RFC 8707 resource, so an OAuth token bound to NO resource reaching an enforcing
//! `/mcp` is anomalous and is now REJECTED (`missing audience`) — belt-and-suspenders,
//! over-blocking toward availability. API keys carry no resource binding BY DESIGN and
//! stay EXEMPT. `mw-mcp` unit-tests the authorizer branch
//! (`no_resource_oauth_token_is_rejected_under_enforcement`); THIS leg proves it WIRED
//! through the real `/mcp` route with the live authorizer introspecting a REAL stored
//! token.
//!
//! Because issuance refuses to mint a no-resource token, we persist one directly at the
//! store seam the live authorizer reads from (`Store::put_oauth_token`, `resource:
//! None`, `kind: access`) — the exact anomaly R3 defends against — then present it as a
//! bearer at `/mcp`. An API key is minted through the server's own `/api/keys` route.
//!
//! Run:
//!   cargo test -p mw-server --test t18_e2e_mcp_noresource -- --nocapture --test-threads=1

use std::net::SocketAddr;
use std::path::PathBuf;

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use mw_oauth::{Scope, ScopeSelector};
use mw_server::{AppConfig, build_app};
use mw_store::{Credentials, OAuthTokenRow, ServerKey, Store};

const KEY_HEX: &str = "0f1e2d3c4b5a69788796a5b4c3d2e1f00f1e2d3c4b5a69788796a5b4c3d2e1f0";
const PUBLIC_ORIGIN: &str = "https://mcp.example";
const ACCOUNT: &str = "acct-mcp-nores";

fn unique() -> String {
    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    format!(
        "{}-{}",
        std::process::id(),
        SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    )
}

const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW</div>";

async fn spawn_engine_server(db_path: &str) -> SocketAddr {
    let web = PathBuf::from(db_path)
        .parent()
        .unwrap()
        .join(format!("web-{}", unique()));
    std::fs::create_dir_all(&web).unwrap();
    std::fs::write(web.join("index.html"), INDEX_HTML).unwrap();
    let config = AppConfig {
        db_path: db_path.to_string(),
        server_key_hex: Some(KEY_HEX.to_string()),
        web_dir: Some(web),
        cookie_secure: false,
        mode: mw_server::ServerMode::Engine, // /mcp is only mounted in engine mode
        hardening: mw_server::HardeningConfig::default(),
        security: mw_server::SecurityConfig::default(),
    };
    let app = build_app(config).await.expect("build_app engine mode");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    addr
}

fn temp_db() -> String {
    let dir = std::env::temp_dir().join(format!("mw-t18-mcpnr-{}", unique()));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join("mw.db").to_string_lossy().into_owned()
}

/// A scope granting exactly `mail.search` on the account.
fn mail_search_scope() -> Scope {
    let mut s = Scope::read_only(ACCOUNT);
    s.mail = true;
    s.accounts = ScopeSelector::All;
    s.mcp_tools = vec!["mail.search".to_string()];
    s
}

/// The at-rest token hash the store keys on: hex(SHA-256(token)) — matches
/// `mw_oauth`'s `sha256_hex`, the transform introspection applies to the bearer.
fn sha256_hex(input: &str) -> String {
    let digest = Sha256::digest(input.as_bytes());
    let mut s = String::with_capacity(digest.len() * 2);
    for b in digest {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

/// Seed a session (for `/api/keys`) and persist a NO-RESOURCE active access token
/// directly at the store seam the live authorizer introspects (issuance would refuse
/// to mint one — that is exactly the anomaly R3 defends against). Returns the raw
/// bearer for the no-resource token.
async fn seed(db_path: &str) -> (String, String) {
    let store = Store::open(db_path, ServerKey::from_hex(KEY_HEX).unwrap())
        .await
        .unwrap();
    let cookie = store
        .create_session(
            ACCOUNT,
            "user@example.org",
            "http://upstream.invalid",
            "http://upstream.invalid",
            &Credentials {
                username: "user@example.org".into(),
                password: "pw".into(),
            },
        )
        .await
        .unwrap();

    let bearer = format!("noresource-access-{}", unique());
    store
        .put_oauth_token(&OAuthTokenRow {
            token_hash: sha256_hex(&bearer),
            client_id: "client-t18".into(),
            account_id: ACCOUNT.into(),
            scopes_json: serde_json::to_string(&mail_search_scope()).unwrap(),
            resource: None, // ← the anomaly: a no-audience OAuth token
            kind: "access".into(),
            expires_at: "2099-01-01T00:00:00Z".into(),
            created_at: "2026-01-01T00:00:00Z".into(),
            revoked_at: None,
            pkce_challenge: None,
        })
        .await
        .unwrap();
    (cookie, bearer)
}

/// Mint an API key (no resource binding) via the server's `/api/keys` route.
async fn mint_api_key(c: &reqwest::Client, base: &str, cookie: &str) -> String {
    let resp: Value = c
        .post(format!("{base}/api/keys"))
        .header("Cookie", format!("mw_session={cookie}"))
        .json(&json!({ "label": "t18", "scope": mail_search_scope() }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    resp["displayToken"]
        .as_str()
        .unwrap_or_else(|| panic!("/api/keys returned no displayToken: {resp}"))
        .to_string()
}

/// Call `mail.search` at `/mcp` with a bearer token; return the JSON-RPC response.
async fn mcp_call(c: &reqwest::Client, base: &str, bearer: &str) -> Value {
    c.post(format!("{base}/mcp"))
        .header("Authorization", format!("Bearer {bearer}"))
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": { "name": "mail.search", "arguments": { "account": ACCOUNT, "query": "x" } },
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}

fn error_code(resp: &Value) -> Option<i64> {
    resp.get("error").and_then(|e| e["code"].as_i64())
}

#[tokio::test]
async fn no_resource_oauth_token_is_rejected_at_mcp_and_api_keys_stay_exempt() {
    // Default-on enforcement: a public origin is configured, no explicit MCP resource.
    // SAFETY: env set before build_app (which reads it once), single-threaded test.
    unsafe {
        std::env::set_var("MW_WEBAUTHN_ORIGIN", PUBLIC_ORIGIN);
        std::env::remove_var("MW_MCP_RESOURCE");
    }

    let db = temp_db();
    let addr = spawn_engine_server(&db).await;
    let base = format!("http://{addr}");
    let (cookie, no_resource_bearer) = seed(&db).await;
    let c = reqwest::Client::new();

    // NO-RESOURCE OAuth token → rejected under default-on enforcement (−32001, scope
    // denied / "missing audience") before any tool runs.
    let nr_resp = mcp_call(&c, &base, &no_resource_bearer).await;
    assert_eq!(
        error_code(&nr_resp),
        Some(-32001),
        "a NO-RESOURCE OAuth token must be rejected at /mcp under default-on enforcement: {nr_resp}"
    );

    // API KEY → carries no resource binding by design → EXEMPT (NOT −32001).
    let api_key = mint_api_key(&c, &base, &cookie).await;
    let key_resp = mcp_call(&c, &base, &api_key).await;
    assert_ne!(
        error_code(&key_resp),
        Some(-32001),
        "an API key carries no audience binding and is exempt from the RFC 8707 check: {key_resp}"
    );

    eprintln!(
        "[t18 mcp-noresource] no-resource OAuth token → {:?} (−32001 = rejected); api-key → {:?} (exempt)",
        error_code(&nr_resp),
        error_code(&key_resp)
    );
}
