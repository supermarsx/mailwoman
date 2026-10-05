//! t28-e12c — an OAuth authorization never carries more than its client may hold.
//!
//! `/oauth/decision` handed the scope in the request to `AuthServer::authorize`,
//! which stored it as posted. A self-registered (DCR) client is registered with
//! the DCR policy's `default_scope`, but nothing compared a later authorization
//! with it, so the client could ask the consenting user for any scope — including
//! `unattended_send`, which is an API-key privilege an administrator countersigns.
//!
//! Every leg drives the real router over HTTP in engine mode with a logged-in
//! mailbox user, and each narrowing is preceded by the same client obtaining
//! exactly its registered scope.
//!
//! Legs:
//!   * a DCR client asking for its registered scope gets it; asking for more gets
//!     the registered scope (narrowed, not refused), on the consent screen, in the
//!     token response, in introspection, and after two refreshes
//!   * a DCR client gets nothing once the policy's default scope grants nothing
//!   * a client an operator put in the registry has no ceiling, and still never
//!     gets `unattended_send`
//!
//! One more leg covers the other half of t28-e12c, the admin's view of the API-key
//! countersign: `GET /admin/api-keys` says which keys asked for unattended send
//! and which are approved, and carries no key hash.
//!
//! Run:
//!   cargo test -p mw-server --test t28_oauth_scope_clamp --locked -- --test-threads=1

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::OnceLock;

use serde_json::{Value, json};
use tokio::sync::{Mutex, MutexGuard};

use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config};
use mw_store::{OAuthClientRow, ServerKey, Store};

mod common;
use common::mock_mail::MockPop3;
use common::test_db;

const KEY_HEX: &str = "28e12c02c3d4e5f60718293a4b5c6d7e28e12c02c3d4e5f60718293a4b5c6d7e";
const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW_T28</div>";

const USER: (&str, &str) = ("alice@example.org", "Alice-Passw0rd!");
const REDIRECT: &str = "https://apps.example.org/cb";
const RESOURCE: &str = "https://mail.example.org/mcp";
const VERIFIER: &str = "t28-e12c-verifier-abcdefghijklmnopqrstuvwxyz";

// ── harness ──────────────────────────────────────────────────────────────────

/// Serialises the tests in this binary: engine-mode login reads `MW_ENGINE_TLS`.
async fn serial() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let guard = LOCK.get_or_init(Default::default).lock().await;
    // SAFETY: the guard serialises every mutation and every dependent read here.
    unsafe {
        std::env::set_var("MW_ENGINE_TLS", "plaintext");
        for k in ["MW_HEADER_AUTH", "MW_PASSWD_BACKEND", "MW_MCP_RESOURCE"] {
            std::env::remove_var(k);
        }
    }
    guard
}

struct Server {
    base: String,
    db: String,
}

impl Server {
    /// A second handle on the server's database, for the registry row an operator
    /// writes and for emptying the DCR policy's default scope.
    async fn store(&self) -> Store {
        Store::open(&self.db, ServerKey::from_hex(KEY_HEX).unwrap())
            .await
            .expect("open the server's store")
    }
}

/// An engine-mode server with the admin panel on, over its own SQLite file.
async fn spawn_engine() -> Server {
    let dir = test_db::unique_dir("mw-t28-e12c");
    let web: PathBuf = dir.join("web");
    std::fs::create_dir_all(&web).unwrap();
    std::fs::write(web.join("index.html"), INDEX_HTML).unwrap();
    let db = dir.join("mw.db").to_string_lossy().into_owned();
    let config = AppConfig {
        db_path: db.clone(),
        server_key_hex: Some(KEY_HEX.to_string()),
        web_dir: Some(web),
        cookie_secure: false,
        mode: ServerMode::Engine,
        hardening: HardeningConfig::default(),
        security: SecurityConfig::default(),
    };
    let v6 = V6Config {
        admin_enabled: true,
        admin_username: Some("root".into()),
        admin_password: Some("hunter2".into()),
        redis_url: None,
    };
    let app = mw_server::build_app_full(config, v6)
        .await
        .expect("build_app_full")
        .0;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });
    Server {
        base: format!("http://{addr}"),
        db,
    }
}

fn browser() -> reqwest::Client {
    reqwest::Client::builder()
        .cookie_store(true)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap()
}

fn bare() -> reqwest::Client {
    reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap()
}

/// A logged-in mailbox user: their POP3 server (kept alive) and their client.
struct Mailbox {
    _pop: MockPop3,
    client: reqwest::Client,
    account_id: String,
}

async fn mailbox(base: &str) -> Mailbox {
    let (user, pass) = USER;
    let pop = MockPop3::start(user, pass).await;
    let client = browser();
    let login = client
        .post(format!("{base}/api/login"))
        .json(&json!({ "jmapUrl": pop.url(), "username": user, "password": pass }))
        .send()
        .await
        .unwrap();
    assert_eq!(login.status(), 200, "control: {user} logs in");
    let account_id = login.json::<Value>().await.unwrap()["accountId"]
        .as_str()
        .expect("login names the account")
        .to_string();
    Mailbox {
        _pop: pop,
        client,
        account_id,
    }
}

/// The scope the DCR policy grants in these tests, in `mw_oauth::Scope` wire form:
/// read mail in two folders, one MCP tool.
fn registered_scope() -> Value {
    json!({
        "read": true, "send": false, "delete": false,
        "accounts": "all", "folders": { "subset": ["inbox", "archive"] },
        "mail": true, "pim": false, "ip_allowlist": [], "expires_at": null,
        "rate_limit": null, "mcp_tools": ["mail.search"],
        "unattended_send": false,
    })
}

/// `registered_scope()` plus a verb, a surface, every folder, a second tool and
/// unattended send.
fn over_broad_scope() -> Value {
    json!({
        "read": true, "send": true, "delete": true,
        "accounts": "all", "folders": "all",
        "mail": true, "pim": true, "ip_allowlist": [], "expires_at": null,
        "rate_limit": null, "mcp_tools": ["mail.search", "mail.send"],
        "unattended_send": true,
    })
}

/// The scope that grants nothing (`mw_oauth::dcr::no_scope`).
fn no_scope() -> Value {
    json!({
        "read": false, "send": false, "delete": false,
        "accounts": { "subset": [] }, "folders": { "subset": [] },
        "mail": false, "pim": false, "ip_allowlist": [], "expires_at": null,
        "rate_limit": null, "mcp_tools": [],
        "unattended_send": false,
    })
}

/// A logged-in admin-panel client.
async fn admin(base: &str) -> reqwest::Client {
    let c = browser();
    let login = c
        .post(format!("{base}/admin/login"))
        .json(&json!({ "username": "root", "password": "hunter2" }))
        .send()
        .await
        .unwrap();
    assert_eq!(login.status(), 200, "admin login");
    c
}

/// Turn DCR on through the admin route with `default_scope`, then self-register a
/// client. Returns its `client_id`.
async fn register_dcr_client(base: &str, default_scope: &Value) -> String {
    let admin = admin(base).await;
    let put = admin
        .put(format!("{base}/admin/oauth-dcr"))
        .json(&json!({
            "enabled": true,
            "allowedRedirectHostSuffixes": ["example.org"],
            "defaultScope": default_scope,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(put.status(), 200, "the admin enables DCR");

    let created = bare()
        .post(format!("{base}/oauth/register"))
        .json(&json!({ "redirect_uris": [REDIRECT], "client_name": "t28-e12c" }))
        .send()
        .await
        .unwrap();
    assert_eq!(created.status(), 201, "the client self-registers");
    created.json::<Value>().await.unwrap()["client_id"]
        .as_str()
        .expect("client_id")
        .to_string()
}

fn authorize_params(client_id: &str, scope: &Value) -> Value {
    json!({
        "clientId": client_id,
        "redirectUri": REDIRECT,
        "state": "s1",
        "codeChallenge": mw_oauth::challenge_s256(VERIFIER),
        "codeChallengeMethod": "S256",
        "resource": RESOURCE,
        "scope": scope,
    })
}

/// What the consent screen is told approving would grant.
async fn consent_scope(base: &str, user: &Mailbox, client_id: &str, scope: &Value) -> Value {
    let resp = user
        .client
        .post(format!("{base}/oauth/consent"))
        .json(&authorize_params(client_id, scope))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "consent display");
    resp.json::<Value>().await.unwrap()["requestedScope"].clone()
}

/// The user approves `scope` for `client_id`; the client exchanges the code.
/// Returns the `/oauth/token` response body.
async fn authorize_and_exchange(
    base: &str,
    user: &Mailbox,
    client_id: &str,
    scope: &Value,
) -> Value {
    let decided = user
        .client
        .post(format!("{base}/oauth/decision"))
        .json(&json!({ "approve": true, "params": authorize_params(client_id, scope) }))
        .send()
        .await
        .unwrap();
    assert_eq!(decided.status(), 200, "the user approves");
    let body: Value = decided.json().await.unwrap();
    let redirect = body["redirectUri"].as_str().expect("redirectUri");
    let code = redirect
        .split_once("?code=")
        .unwrap_or_else(|| panic!("no code in {redirect}"))
        .1
        .split('&')
        .next()
        .unwrap()
        .to_string();

    let resp = bare()
        .post(format!("{base}/oauth/token"))
        .json(&json!({
            "grant_type": "authorization_code",
            "code": code,
            "redirect_uri": REDIRECT,
            "client_id": client_id,
            "code_verifier": VERIFIER,
            "resource": RESOURCE,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "the code is exchanged");
    resp.json().await.unwrap()
}

async fn refresh(base: &str, client_id: &str, refresh_token: &str) -> Value {
    let resp = bare()
        .post(format!("{base}/oauth/token"))
        .json(&json!({
            "grant_type": "refresh_token",
            "refresh_token": refresh_token,
            "client_id": client_id,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "the refresh token is exchanged");
    resp.json().await.unwrap()
}

/// `POST /oauth/introspect` for a token; asserts it is active and returns its scope.
async fn introspected_scope(base: &str, token: &Value) -> Value {
    let body: Value = bare()
        .post(format!("{base}/oauth/introspect"))
        .json(&json!({ "token": token }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(body["active"], true, "the token is live: {body}");
    body["scope"].clone()
}

// ── 1. a DCR client is held to its registered scope ──────────────────────────

#[tokio::test]
async fn a_dcr_client_asking_for_more_than_it_registered_gets_what_it_registered() {
    let _g = serial().await;
    let srv = spawn_engine().await;
    let user = mailbox(&srv.base).await;
    let client_id = register_dcr_client(&srv.base, &registered_scope()).await;

    // Precondition: asking for exactly the registered scope yields it.
    let exact = authorize_and_exchange(&srv.base, &user, &client_id, &registered_scope()).await;
    assert_eq!(exact["scope"], registered_scope());
    assert_eq!(
        introspected_scope(&srv.base, &exact["access_token"]).await,
        registered_scope()
    );

    // Asking for more: the consent screen and the token both show the registered
    // scope — the request is narrowed, not refused.
    assert_eq!(
        consent_scope(&srv.base, &user, &client_id, &over_broad_scope()).await,
        registered_scope(),
        "the consent screen lists what approving would grant"
    );
    let broad = authorize_and_exchange(&srv.base, &user, &client_id, &over_broad_scope()).await;
    assert_eq!(broad["scope"], registered_scope());
    assert_eq!(
        introspected_scope(&srv.base, &broad["access_token"]).await,
        registered_scope()
    );
    assert_eq!(
        introspected_scope(&srv.base, &broad["refresh_token"]).await,
        registered_scope()
    );

    // Refreshing, twice, does not widen it.
    let mut refresh_token = broad["refresh_token"].as_str().unwrap().to_string();
    for round in 1..=2 {
        let next = refresh(&srv.base, &client_id, &refresh_token).await;
        assert_eq!(next["scope"], registered_scope(), "refresh {round}");
        assert_eq!(
            introspected_scope(&srv.base, &next["access_token"]).await,
            registered_scope(),
            "refresh {round}"
        );
        refresh_token = next["refresh_token"].as_str().unwrap().to_string();
    }
}

// ── 2. a policy that grants nothing leaves a DCR client with nothing ─────────

#[tokio::test]
async fn a_dcr_client_gets_nothing_when_the_policy_default_scope_grants_nothing() {
    let _g = serial().await;
    let srv = spawn_engine().await;
    let user = mailbox(&srv.base).await;
    let client_id = register_dcr_client(&srv.base, &registered_scope()).await;

    // Precondition: under the policy it registered with, the client gets a scope.
    let before = authorize_and_exchange(&srv.base, &user, &client_id, &over_broad_scope()).await;
    assert_eq!(before["scope"], registered_scope());

    // The ceiling is the policy row as it reads at consent. A default scope that
    // is not a whole `Scope` object reads as the scope that grants nothing.
    let store = srv.store().await;
    let mut row = store
        .get_oauth_dcr_policy()
        .await
        .unwrap()
        .expect("the policy row the admin wrote");
    row.default_scope_json = "{}".into();
    store.put_oauth_dcr_policy(&row).await.unwrap();

    let after = authorize_and_exchange(&srv.base, &user, &client_id, &over_broad_scope()).await;
    assert_eq!(after["scope"], no_scope());
    assert_eq!(
        introspected_scope(&srv.base, &after["access_token"]).await,
        no_scope()
    );
}

// ── 3. no ceiling for an operator's client, and still no unattended send ─────

#[tokio::test]
async fn consent_never_grants_unattended_send_even_without_a_ceiling() {
    let _g = serial().await;
    let srv = spawn_engine().await;
    let user = mailbox(&srv.base).await;
    let client_id = "operator-client";
    srv.store()
        .await
        .put_oauth_client(&OAuthClientRow {
            client_id: client_id.into(),
            name: "Operator client".into(),
            redirect_uris_json: json!([REDIRECT]).to_string(),
            approved_by: "root".into(),
            created_at: "2026-10-05T00:00:00Z".into(),
        })
        .await
        .expect("the operator registers a client");

    let mut expected = over_broad_scope();
    expected["unattended_send"] = json!(false);

    assert_eq!(
        consent_scope(&srv.base, &user, client_id, &over_broad_scope()).await,
        expected
    );
    let tokens = authorize_and_exchange(&srv.base, &user, client_id, &over_broad_scope()).await;
    // Everything else it asked for is granted: there is no registered scope to
    // hold it to.
    assert_eq!(tokens["scope"], expected);
    assert_eq!(
        introspected_scope(&srv.base, &tokens["access_token"]).await,
        expected
    );
    let next = refresh(
        &srv.base,
        client_id,
        tokens["refresh_token"].as_str().unwrap(),
    )
    .await;
    assert_eq!(next["scope"], expected);
}

// ── 4. the admin key list shows the request and the approval ─────────────────

/// The user mints an API key whose scope has `unattended_send` as given.
/// Returns its prefix.
async fn mint_key(base: &str, owner: &Mailbox, unattended: bool) -> String {
    let resp = owner
        .client
        .post(format!("{base}/api/keys"))
        .json(&json!({
            "label": "t28-e12c",
            "accountId": owner.account_id,
            "scope": {
                "read": true, "send": true, "delete": false,
                "accounts": { "subset": [owner.account_id] }, "folders": "all",
                "mail": true, "pim": false, "ip_allowlist": [], "expires_at": null,
                "rate_limit": null, "mcp_tools": ["mail.send"],
                "unattended_send": unattended,
            },
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "control: the owner mints a key");
    resp.json::<Value>().await.unwrap()["record"]["prefix"]
        .as_str()
        .expect("prefix")
        .to_string()
}

/// One key's row in `GET /admin/api-keys`.
async fn admin_listed(admin: &reqwest::Client, base: &str, prefix: &str) -> Value {
    let rows: Vec<Value> = admin
        .get(format!("{base}/admin/api-keys"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    rows.into_iter()
        .find(|r| r["prefix"] == prefix)
        .unwrap_or_else(|| panic!("no row for {prefix}"))
}

#[tokio::test]
async fn the_admin_key_list_says_which_keys_asked_and_which_are_approved() {
    let _g = serial().await;
    let srv = spawn_engine().await;
    let user = mailbox(&srv.base).await;
    let admin = admin(&srv.base).await;
    let plain = mint_key(&srv.base, &user, false).await;
    let asked = mint_key(&srv.base, &user, true).await;

    let row = admin_listed(&admin, &srv.base, &plain).await;
    assert_eq!(row["unattendedSendRequested"], false);
    assert_eq!(row["unattendedSendApproved"], false);

    let row = admin_listed(&admin, &srv.base, &asked).await;
    assert_eq!(row["accountId"], user.account_id, "the row names the owner");
    assert_eq!(row["unattendedSendRequested"], true);
    assert_eq!(
        row["unattendedSendApproved"], false,
        "asking approves nothing"
    );

    // The row never carries the stored hash, under any name.
    let stored_hash = srv
        .store()
        .await
        .get_api_key(&asked)
        .await
        .unwrap()
        .expect("the key row")
        .key_hash;
    assert!(!stored_hash.is_empty());
    assert!(
        !row.to_string().contains(&stored_hash),
        "the admin list leaks the key hash: {row}"
    );

    // The id in the row is the one the countersign route takes; after an approval
    // the list says so, and after a withdrawal it says that.
    let id = row["id"].as_str().expect("id").to_string();
    for approved in [true, false] {
        let put = admin
            .put(format!("{}/admin/api-keys/{id}/unattended-send", srv.base))
            .json(&json!({ "approved": approved }))
            .send()
            .await
            .unwrap();
        assert_eq!(put.status(), 200);
        let row = admin_listed(&admin, &srv.base, &asked).await;
        assert_eq!(row["unattendedSendRequested"], true);
        assert_eq!(row["unattendedSendApproved"], approved);
    }

    // The two refusals the panel words: a key that did not ask, and an unknown id.
    let plain_id = admin_listed(&admin, &srv.base, &plain).await["id"]
        .as_str()
        .unwrap()
        .to_string();
    for (id, status) in [(plain_id.as_str(), 409), ("no-such-key", 404)] {
        let put = admin
            .put(format!("{}/admin/api-keys/{id}/unattended-send", srv.base))
            .json(&json!({ "approved": true }))
            .send()
            .await
            .unwrap();
        assert_eq!(put.status(), status, "approving {id}");
    }
    let row = admin_listed(&admin, &srv.base, &plain).await;
    assert_eq!(row["unattendedSendApproved"], false);
}
