//! t17-e-e2e — MCP RFC 8707 audience enforcement is DEFAULT-ON (L6), LIVE at `/mcp`.
//!
//! 26.16 enforced token audience only when `MW_MCP_RESOURCE` was set. 26.17 makes it
//! default-on: when `MW_MCP_RESOURCE` is unset, the `/mcp` endpoint's canonical
//! resource is DERIVED from the deployment's configured public origin
//! (`MW_WEBAUTHN_ORIGIN`), so a wrong-audience bearer token is rejected without any
//! explicit MCP config. `mw-mcp` unit-tests the authorizer and `mw-server` unit-tests
//! `resolve_mcp_resource`; THIS leg proves the whole thing WIRED end-to-end over real
//! HTTP:
//!   * With `MW_WEBAUTHN_ORIGIN` set + `MW_MCP_RESOURCE` UNSET, a real OAuth token
//!     minted (through the live `/oauth/decision` + `/oauth/token` routes) for the
//!     WRONG resource is REJECTED at `/mcp` (JSON-RPC scope-denied −32001) before any
//!     tool runs — by default.
//!   * A token minted for the RIGHT resource (`<origin>/mcp`) passes the audience gate
//!     (NOT −32001).
//!   * An API key (no resource binding) is EXEMPT — it passes the audience gate too.
//!
//! Everything is minted through the server's OWN OAuth routes against the SAME store,
//! so the live authorizer introspects real tokens — no test-only shim.
//!
//! Run:
//!   cargo test -p mw-server --test t17_mcp_audience -- --nocapture --test-threads=1

use std::net::SocketAddr;
use std::path::PathBuf;

use serde_json::{Value, json};

use mw_oauth::{Scope, ScopeSelector, challenge_s256};
use mw_server::{AppConfig, build_app};
use mw_store::{Credentials, OAuthClientRow, ServerKey, Store};

const KEY_HEX: &str = "0f1e2d3c4b5a69788796a5b4c3d2e1f00f1e2d3c4b5a69788796a5b4c3d2e1f0";
const PUBLIC_ORIGIN: &str = "https://mcp.example";
const ACCOUNT: &str = "acct-mcp";
const REDIRECT: &str = "https://app.example/cb";
const CLIENT_ID: &str = "client-t17";
// A PKCE verifier ≥ 43 chars.
const VERIFIER: &str = "verifier-abc-123-verifier-abc-123-verifier-xyz";

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
    let dir = std::env::temp_dir().join(format!("mw-t17-mcp-{}", unique()));
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

/// Seed a session + an approved OAuth client on the same store the server uses.
async fn seed(db_path: &str) -> String {
    let store = Store::open(db_path, ServerKey::from_hex(KEY_HEX).unwrap())
        .await
        .unwrap();
    store
        .put_oauth_client(&OAuthClientRow {
            client_id: CLIENT_ID.into(),
            name: "T17 MCP client".into(),
            redirect_uris_json: json!([REDIRECT]).to_string(),
            approved_by: "admin".into(),
            created_at: "2026-01-01T00:00:00Z".into(),
        })
        .await
        .unwrap();
    store
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
        .unwrap()
}

/// Minimal percent-decode for a redirect-URI query value.
fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%'
            && i + 2 < b.len()
            && let Ok(v) = u8::from_str_radix(&s[i + 1..i + 3], 16)
        {
            out.push(v);
            i += 3;
            continue;
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Drive `/oauth/decision` + `/oauth/token` to mint a real access token bound to
/// `resource`, through the server's own routes (the live authorizer will introspect it).
async fn mint_oauth_token(c: &reqwest::Client, base: &str, cookie: &str, resource: &str) -> String {
    let challenge = challenge_s256(VERIFIER);
    let params = json!({
        "clientId": CLIENT_ID,
        "redirectUri": REDIRECT,
        "codeChallenge": challenge,
        "codeChallengeMethod": "S256",
        "resource": resource,
        "scope": mail_search_scope(),
    });
    let decision: Value = c
        .post(format!("{base}/oauth/decision"))
        .header("Cookie", format!("mw_session={cookie}"))
        .json(&json!({ "approve": true, "params": params }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let redirect = decision["redirectUri"]
        .as_str()
        .unwrap_or_else(|| panic!("decision returned no redirect: {decision}"));
    let code_raw = redirect
        .split("code=")
        .nth(1)
        .and_then(|r| r.split('&').next())
        .unwrap_or_else(|| panic!("no code in redirect: {redirect}"));
    let code = percent_decode(code_raw);

    let token: Value = c
        .post(format!("{base}/oauth/token"))
        .json(&json!({
            "grant_type": "authorization_code",
            "code": code,
            "redirect_uri": REDIRECT,
            "client_id": CLIENT_ID,
            "code_verifier": VERIFIER,
            "resource": resource,
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    token["access_token"]
        .as_str()
        .unwrap_or_else(|| panic!("token endpoint returned no access_token: {token}"))
        .to_string()
}

/// Mint an API key (no resource binding) via the server's `/api/keys` route.
async fn mint_api_key(c: &reqwest::Client, base: &str, cookie: &str) -> String {
    let resp: Value = c
        .post(format!("{base}/api/keys"))
        .header("Cookie", format!("mw_session={cookie}"))
        .json(&json!({ "label": "t17", "scope": mail_search_scope() }))
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
async fn mcp_audience_is_enforced_by_default_and_api_keys_are_exempt() {
    // Default-on config: a public origin is configured, but NO explicit MCP resource.
    // SAFETY: env set before build_app (which reads it once), single-threaded test.
    unsafe {
        std::env::set_var("MW_WEBAUTHN_ORIGIN", PUBLIC_ORIGIN);
        std::env::remove_var("MW_MCP_RESOURCE");
    }

    let db = temp_db();
    let addr = spawn_engine_server(&db).await;
    let base = format!("http://{addr}");
    let cookie = seed(&db).await;
    let c = reqwest::Client::new();

    // The endpoint's derived canonical resource (default-on): `<origin>/mcp`.
    let right_resource = format!("{PUBLIC_ORIGIN}/mcp");
    let wrong_resource = "https://attacker.example/mcp";

    // WRONG audience → rejected by default (−32001), before any tool runs.
    let wrong_token = mint_oauth_token(&c, &base, &cookie, wrong_resource).await;
    let wrong_resp = mcp_call(&c, &base, &wrong_token).await;
    assert_eq!(
        error_code(&wrong_resp),
        Some(-32001),
        "a WRONG-audience token must be rejected at /mcp by default (MW_MCP_RESOURCE unset, \
         origin-derived resource): {wrong_resp}"
    );

    // RIGHT audience → passes the audience gate (NOT the −32001 audience denial). The
    // tool may then succeed or fail on backend grounds, but it is NOT an audience reject.
    let right_token = mint_oauth_token(&c, &base, &cookie, &right_resource).await;
    let right_resp = mcp_call(&c, &base, &right_token).await;
    assert_ne!(
        error_code(&right_resp),
        Some(-32001),
        "a RIGHT-audience token must clear the audience gate: {right_resp}"
    );

    // API KEY → no resource binding → EXEMPT from the audience check (also NOT −32001).
    let api_key = mint_api_key(&c, &base, &cookie).await;
    let key_resp = mcp_call(&c, &base, &api_key).await;
    assert_ne!(
        error_code(&key_resp),
        Some(-32001),
        "an API key carries no audience binding and is exempt from the RFC 8707 check: {key_resp}"
    );

    eprintln!(
        "[t17 mcp] wrong-aud → {:?}; right-aud → {:?}; api-key → {:?} (only wrong-aud is −32001)",
        error_code(&wrong_resp),
        error_code(&right_resp),
        error_code(&key_resp)
    );
}
