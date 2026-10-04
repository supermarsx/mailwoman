//! t27-e3 (OH-1) — the admin panel's per-account `disabled` and
//! `force_password_change` flags are read on every path that establishes or uses a
//! mailbox principal.
//!
//! Before 26.20 the panel wrote, persisted and audited both flags and nothing read
//! them: a disabled account logged in and kept its sessions, keys and tokens, and
//! "revoke sessions" deleted nothing because the panel names an account by login
//! name while `sessions.account_id` holds a different identifier.
//!
//! Every leg drives the real router over HTTP and sets the flag through the admin
//! route (`PUT /admin/users/{id}/flags`), so the key the writer stores under and the
//! key the reader looks up are proved to agree. Every refusal is preceded, in the
//! same server and for the same credential, by the same request succeeding — a
//! refusal that would also happen with the flag clear proves nothing.
//!
//! Legs:
//!   * password login + cookie session (proxy mode), flag id in a different case
//!   * the "revoke sessions" button
//!   * a pending second-factor login, and a login for an enrolled account
//!   * native bearer session
//!   * header-auth (`X-Remote-User`)
//!   * SSO callback (mock IdP)
//!   * API key on `/api/v1`, API key and OAuth access token on `/mcp`, the OAuth
//!     refresh grant and introspection (engine mode)
//!   * API key in proxy mode (the account id does not map back to a login name
//!     once the sessions are gone; the key must read nothing)
//!   * `force_password_change`: the hold, its allow-list, and its release
//!
//! Run:
//!   cargo test -p mw-server --test t27_account_flags --locked -- --test-threads=1

use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tokio::sync::{Mutex, MutexGuard};

use mw_mfa::totp::{self, TotpParams};
use mw_server::sso::{SsoEntry, SsoMeta};
use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config};
use mw_sso::{
    BeginRedirect, ClaimMap, FirstLoginPolicy, Metadata, PendingState, Redirect, SsoCallback,
    SsoError, SsoIdentity, SsoKind, SsoLogin, SsoScope,
};
use mw_store::{AccountKind, Credentials, NewAccount, OAuthClientRow, ServerKey, Store};

mod common;
use common::mock_mail::MockPop3;
use common::test_db;

const KEY_HEX: &str = "27e3a1b2c3d4e5f60718293a4b5c6d7e27e3a1b2c3d4e5f60718293a4b5c6d7e";
const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW_T27</div>";

/// The engine-mode mailbox the scripted POP3 server accepts.
const ENGINE_USER: &str = "owner@example.org";
const ENGINE_PASS: &str = "Old-Passw0rd!";

const CLIENT_ID: &str = "client-t27";
const REDIRECT: &str = "https://app.example/cb";
const RESOURCE: &str = "https://t27.example/mcp";
const VERIFIER: &str = "verifier-t27-abc-123-verifier-t27-abc-123-xyz";
const STATE_TOKEN: &str = "t27fixedstatetoken0123456789abcd";

/// The body the gate answers a held account with (contract with the web client).
fn held_body() -> Value {
    json!({ "error": "password change required", "passwordChangeRequired": true })
}

// ── harness ──────────────────────────────────────────────────────────────────

/// Serialises the tests in this binary: several set process environment
/// (`MW_ENGINE_TLS`, `MW_HEADER_AUTH*`) that the server reads per request.
async fn serial() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let guard = LOCK.get_or_init(Default::default).lock().await;
    // SAFETY: the guard serialises every mutation and every dependent read here.
    unsafe {
        std::env::set_var("MW_ENGINE_TLS", "plaintext");
        for k in [
            "MW_HEADER_AUTH",
            "MW_HEADER_AUTH_TRUSTED_IPS",
            "MW_HEADER_AUTH_HEADER",
            "MW_HEADER_AUTH_PASSWORD",
            "MW_HEADER_AUTH_JMAP_URL",
            "MW_LDAP_BIND_AUTH",
            "MW_PASSWD_BACKEND",
            "MW_MCP_RESOURCE",
            "MW_WEBAUTHN_ORIGIN",
        ] {
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
    /// A second handle on the server's database, for seeding what no HTTP route
    /// seeds (a TOTP secret, an OAuth client, a local password hash) and for
    /// counting rows.
    async fn store(&self) -> Store {
        Store::open(&self.db, ServerKey::from_hex(KEY_HEX).unwrap())
            .await
            .expect("open the server's store")
    }
}

fn admin_v6() -> V6Config {
    V6Config {
        admin_enabled: true,
        admin_username: Some("root".into()),
        admin_password: Some("hunter2".into()),
        redis_url: None,
    }
}

fn app_config(mode: ServerMode, upstreams: Vec<String>) -> (AppConfig, String) {
    let dir = test_db::unique_dir("mw-t27-e3");
    let web: PathBuf = dir.join("web");
    std::fs::create_dir_all(&web).unwrap();
    std::fs::write(web.join("index.html"), INDEX_HTML).unwrap();
    let db = dir.join("mw.db").to_string_lossy().into_owned();
    let config = AppConfig {
        db_path: db.clone(),
        server_key_hex: Some(KEY_HEX.to_string()),
        web_dir: Some(web),
        cookie_secure: false,
        mode,
        hardening: HardeningConfig::default(),
        security: SecurityConfig {
            jmap_upstreams: Some(upstreams),
            ..SecurityConfig::default()
        },
    };
    (config, db)
}

async fn serve(app: axum::Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        // As `main.rs` serves it: header-auth needs the peer address.
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });
    format!("http://{addr}")
}

async fn spawn_mock_jmap() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, mw_mock_jmap::router()).await.unwrap();
    });
    format!("http://{addr}")
}

/// A proxy-mode server in front of a fresh mock JMAP upstream.
async fn spawn_proxy() -> (Server, String) {
    let mock = spawn_mock_jmap().await;
    let (config, db) = app_config(ServerMode::Proxy, vec![mock.clone()]);
    let app = mw_server::build_app_full(config, admin_v6())
        .await
        .expect("build_app_full")
        .0;
    let base = serve(app).await;
    (Server { base, db }, mock)
}

/// An engine-mode server (the mode that mounts `/mcp`).
async fn spawn_engine() -> Server {
    let (config, db) = app_config(ServerMode::Engine, Vec::new());
    let app = mw_server::build_app_full(config, admin_v6())
        .await
        .expect("build_app_full")
        .0;
    let base = serve(app).await;
    Server { base, db }
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

/// A logged-in admin-panel client.
async fn admin(base: &str) -> reqwest::Client {
    let c = browser();
    let r = c
        .post(format!("{base}/admin/login"))
        .json(&json!({ "username": "root", "password": "hunter2" }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200, "admin login");
    c
}

/// Set an account's flags the way the panel does.
async fn set_flags(admin: &reqwest::Client, base: &str, id: &str, disabled: bool, force: bool) {
    let r = admin
        .put(format!("{base}/admin/users/{id}/flags"))
        .json(&json!({
            "zeroAccess": false,
            "forcePasswordChange": force,
            "remoteCacheWipe": false,
            "disabled": disabled,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 204, "the panel's flag write is accepted");
}

async fn proxy_login(
    c: &reqwest::Client,
    base: &str,
    mock: &str,
    user: &str,
    pass: &str,
) -> reqwest::Response {
    c.post(format!("{base}/api/login"))
        .json(&json!({ "jmapUrl": mock, "username": user, "password": pass }))
        .send()
        .await
        .unwrap()
}

async fn engine_login(c: &reqwest::Client, base: &str, pop: &MockPop3) -> reqwest::Response {
    c.post(format!("{base}/api/login"))
        .json(&json!({ "jmapUrl": pop.url(), "username": ENGINE_USER, "password": ENGINE_PASS }))
        .send()
        .await
        .unwrap()
}

fn sets_session_cookie(resp: &reqwest::Response) -> bool {
    resp.headers()
        .get_all(reqwest::header::SET_COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .any(|c| c.starts_with("mw_session=") && !c.starts_with("mw_session=;"))
}

async fn me(c: &reqwest::Client, base: &str) -> reqwest::Response {
    c.get(format!("{base}/api/me")).send().await.unwrap()
}

/// An empty JMAP request: reaches the handler behind `authed` and needs no data.
async fn jmap(c: &reqwest::Client, base: &str) -> reqwest::Response {
    c.post(format!("{base}/jmap/api"))
        .json(&json!({ "using": ["urn:ietf:params:jmap:core"], "methodCalls": [] }))
        .send()
        .await
        .unwrap()
}

fn sha256_hex(s: &str) -> String {
    Sha256::digest(s.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn read_scope(account_id: &str) -> Value {
    json!({
        "read": true, "send": false, "delete": false,
        "accounts": { "subset": [account_id] }, "folders": "all",
        "mail": true, "pim": false, "ip_allowlist": [], "expires_at": null,
        "rate_limit": null, "mcp_tools": ["mail.search"], "unattended_send": false,
    })
}

async fn mint_api_key(c: &reqwest::Client, base: &str, account_id: &str) -> String {
    let mint: Value = c
        .post(format!("{base}/api/keys"))
        .json(&json!({ "label": "t27", "accountId": account_id, "scope": read_scope(account_id) }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    mint["displayToken"]
        .as_str()
        .unwrap_or_else(|| panic!("/api/keys returned no displayToken: {mint}"))
        .to_string()
}

// ── 1. password login + cookie session ───────────────────────────────────────

#[tokio::test]
async fn disabled_account_is_refused_at_login_and_loses_its_session() {
    let _g = serial().await;
    let (srv, mock) = spawn_proxy().await;
    let base = &srv.base;
    let (user, pass) = (mw_mock_jmap::USER, mw_mock_jmap::PASS);
    let store = srv.store().await;
    let admin = admin(base).await;

    // Control: the account logs in and its session works.
    let c = browser();
    let login = proxy_login(&c, base, &mock, user, pass).await;
    assert_eq!(login.status(), 200, "control: login succeeds");
    let body: Value = login.json().await.unwrap();
    assert!(
        body.get("passwordChangeRequired").is_none(),
        "no hold on an unflagged account: {body}"
    );
    let account_id = body["accountId"].as_str().unwrap().to_string();
    assert_eq!(me(&c, base).await.status(), 200, "control: /api/me");
    assert_eq!(jmap(&c, base).await.status(), 200, "control: /jmap/api");
    assert_eq!(
        store.sessions_by_account(&account_id).await.unwrap().len(),
        1
    );

    // What a wrong password looks like: a disabled account must look the same.
    let wrong = proxy_login(&browser(), base, &mock, user, "not-the-password").await;
    assert_eq!(wrong.status(), 401);
    let wrong_body: Value = wrong.json().await.unwrap();

    set_flags(&admin, base, user, true, false).await;

    // Disabling deletes the session rows; it does not wait for the next request.
    assert_eq!(
        store.sessions_by_account(&account_id).await.unwrap().len(),
        0,
        "setting `disabled` revokes the account's sessions"
    );
    assert_eq!(
        me(&c, base).await.status(),
        401,
        "the session established before the flag was set is refused"
    );
    assert_eq!(jmap(&c, base).await.status(), 401);

    // A new login with the right password is refused, indistinguishably.
    let refused = proxy_login(&browser(), base, &mock, user, pass).await;
    assert_eq!(refused.status(), 401, "a disabled account cannot log in");
    assert!(
        !sets_session_cookie(&refused),
        "no session cookie is issued"
    );
    let refused_body: Value = refused.json().await.unwrap();
    assert_eq!(
        refused_body, wrong_body,
        "the refusal does not disclose that the account exists and is disabled"
    );
    assert_eq!(
        store.sessions_by_account(&account_id).await.unwrap().len(),
        0,
        "the refused login left no session behind"
    );

    // Re-enable: the account works again (the refusal was the flag, nothing else).
    set_flags(&admin, base, user, false, false).await;
    let c2 = browser();
    assert_eq!(
        proxy_login(&c2, base, &mock, user, pass).await.status(),
        200
    );
    assert_eq!(me(&c2, base).await.status(), 200);

    // The panel's id in a different case names the same account.
    set_flags(&admin, base, "TestUser@EXAMPLE.org", true, false).await;
    assert_eq!(
        me(&c2, base).await.status(),
        401,
        "a flag written under a case variant of the login name applies"
    );
    assert_eq!(
        proxy_login(&browser(), base, &mock, user, pass)
            .await
            .status(),
        401
    );
    // ...and clearing it under the canonical spelling clears that same record.
    set_flags(&admin, base, user, false, false).await;
    assert_eq!(
        proxy_login(&browser(), base, &mock, user, pass)
            .await
            .status(),
        200,
        "one record per account, whatever case the panel used"
    );
}

// ── 2. the "revoke sessions" button ──────────────────────────────────────────

#[tokio::test]
async fn revoke_sessions_deletes_the_sessions_of_the_named_login() {
    let _g = serial().await;
    let (srv, mock) = spawn_proxy().await;
    let base = &srv.base;
    let (user, pass) = (mw_mock_jmap::USER, mw_mock_jmap::PASS);
    let store = srv.store().await;
    let admin = admin(base).await;

    let (c1, c2) = (browser(), browser());
    let login = proxy_login(&c1, base, &mock, user, pass).await;
    assert_eq!(login.status(), 200);
    let account_id = login.json::<Value>().await.unwrap()["accountId"]
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(
        proxy_login(&c2, base, &mock, user, pass).await.status(),
        200
    );
    assert_ne!(
        account_id, user,
        "the premise: the session's account id is not the panel's id"
    );
    assert_eq!(
        store.sessions_by_account(&account_id).await.unwrap().len(),
        2
    );
    assert_eq!(me(&c1, base).await.status(), 200);

    let r = admin
        .post(format!("{base}/admin/users/{user}/revoke-sessions"))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    let body: Value = r.json().await.unwrap();
    assert_eq!(
        body["count"],
        json!(2),
        "both sessions are reported revoked"
    );
    assert_eq!(
        store.sessions_by_account(&account_id).await.unwrap().len(),
        0,
        "the rows are gone"
    );
    assert_eq!(me(&c1, base).await.status(), 401);
    assert_eq!(me(&c2, base).await.status(), 401);

    // Revocation is not a ban: the account can log in again.
    assert_eq!(
        proxy_login(&browser(), base, &mock, user, pass)
            .await
            .status(),
        200
    );
}

// ── 3. second factor ─────────────────────────────────────────────────────────

#[tokio::test]
async fn disabled_account_gets_no_second_factor_challenge_and_no_session() {
    let _g = serial().await;
    let (srv, mock) = spawn_proxy().await;
    let base = &srv.base;
    let (user, pass) = (mw_mock_jmap::USER, mw_mock_jmap::PASS);
    let store = srv.store().await;
    let admin = admin(base).await;

    // Enrol TOTP on the account.
    let first = proxy_login(&browser(), base, &mock, user, pass).await;
    let account_id = first.json::<Value>().await.unwrap()["accountId"]
        .as_str()
        .unwrap()
        .to_string();
    let secret = totp::generate_secret();
    store
        .put_totp_secret(&account_id, &secret, true)
        .await
        .unwrap();

    let verify = |c: reqwest::Client, token: String, at: u64| {
        let url = format!("{base}/api/login/2fa");
        let code = totp::totp_at(&secret, at, &TotpParams::default());
        async move {
            c.post(url)
                .json(&json!({ "pendingToken": token, "method": "totp", "code": code }))
                .send()
                .await
                .unwrap()
        }
    };

    // Control: password → challenge → code → session. The control spends the
    // PREVIOUS time step so the replay guard still accepts the current one below.
    let c = browser();
    let challenged: Value = proxy_login(&c, base, &mock, user, pass)
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(challenged["twofaRequired"], json!(true), "{challenged}");
    let token = challenged["pendingToken"].as_str().unwrap().to_string();
    let done = verify(c.clone(), token, now_unix() - 30).await;
    assert_eq!(
        done.status(),
        200,
        "control: the factor completes the login"
    );
    assert!(sets_session_cookie(&done));

    // A login that is pending when the account is disabled must not complete.
    let c2 = browser();
    let pending: Value = proxy_login(&c2, base, &mock, user, pass)
        .await
        .json()
        .await
        .unwrap();
    let pending_token = pending["pendingToken"].as_str().unwrap().to_string();

    set_flags(&admin, base, user, true, false).await;

    let late = verify(c2.clone(), pending_token, now_unix()).await;
    assert_eq!(
        late.status(),
        401,
        "a challenge issued before the flag was set does not yield a session"
    );
    assert!(!sets_session_cookie(&late));
    assert_eq!(me(&c2, base).await.status(), 401);

    // A fresh login is refused outright — no challenge, no pending token.
    let fresh = proxy_login(&browser(), base, &mock, user, pass).await;
    assert_eq!(fresh.status(), 401);
    let fresh_body: Value = fresh.json().await.unwrap();
    assert!(
        fresh_body.get("twofaRequired").is_none() && fresh_body.get("pendingToken").is_none(),
        "no second-factor challenge is minted for a disabled account: {fresh_body}"
    );
    assert_eq!(
        store.sessions_by_account(&account_id).await.unwrap().len(),
        0
    );
}

// ── 4. native bearer ─────────────────────────────────────────────────────────

#[tokio::test]
async fn native_bearer_session_is_refused_after_disable() {
    let _g = serial().await;
    let (srv, mock) = spawn_proxy().await;
    let base = &srv.base;
    let (user, pass) = (mw_mock_jmap::USER, mw_mock_jmap::PASS);
    let store = srv.store().await;
    let admin = admin(base).await;

    let native_login = || async {
        bare()
            .post(format!("{base}/api/login"))
            .json(&json!({
                "jmapUrl": mock, "username": user, "password": pass, "clientType": "native",
            }))
            .send()
            .await
            .unwrap()
    };
    let login = native_login().await;
    assert_eq!(login.status(), 200);
    let token = login.json::<Value>().await.unwrap()["token"]
        .as_str()
        .expect("native login returns a bearer token")
        .to_string();
    let bearer_me = || async {
        bare()
            .get(format!("{base}/api/me"))
            .bearer_auth(&token)
            .send()
            .await
            .unwrap()
    };
    assert_eq!(bearer_me().await.status(), 200, "control: the bearer works");
    assert!(
        store
            .get_native_session(&sha256_hex(&token))
            .await
            .unwrap()
            .is_some()
    );

    set_flags(&admin, base, user, true, false).await;

    assert_eq!(
        bearer_me().await.status(),
        401,
        "a bearer token issued before the flag was set is refused"
    );
    assert!(
        store
            .get_native_session(&sha256_hex(&token))
            .await
            .unwrap()
            .is_none(),
        "the native marker is deleted with the session"
    );
    assert_eq!(
        native_login().await.status(),
        401,
        "no new bearer token for a disabled account"
    );
}

// ── 5. header-auth ───────────────────────────────────────────────────────────

#[tokio::test]
async fn header_auth_does_not_mint_a_session_for_a_disabled_account() {
    let _g = serial().await;
    // SAFETY: under the `serial` guard; cleared again by the next `serial()`.
    unsafe {
        std::env::set_var("MW_HEADER_AUTH", "1");
        std::env::set_var("MW_HEADER_AUTH_TRUSTED_IPS", "127.0.0.0/8, ::1/128");
    }
    let (srv, mock) = spawn_proxy().await;
    let base = &srv.base;
    let admin = admin(base).await;
    let asserted = "asserted@example.org";

    let header_login = |c: reqwest::Client| {
        let req = c
            .post(format!("{base}/api/login"))
            .header("x-remote-user", asserted)
            .json(&json!({ "jmapUrl": mock, "username": "", "password": "" }));
        async move { req.send().await.unwrap() }
    };

    let c = browser();
    let ok = header_login(c.clone()).await;
    assert_eq!(ok.status(), 200, "control: the asserted identity logs in");
    assert_eq!(me(&c, base).await.status(), 200);

    set_flags(&admin, base, asserted, true, false).await;

    assert_eq!(me(&c, base).await.status(), 401, "the existing session");
    let refused = header_login(browser()).await;
    assert_eq!(
        refused.status(),
        401,
        "a trusted proxy's assertion does not override the flag"
    );
    assert!(!sets_session_cookie(&refused));
}

// ── 6. SSO ───────────────────────────────────────────────────────────────────

struct MockProvider {
    identity: SsoIdentity,
}

#[async_trait]
impl SsoLogin for MockProvider {
    async fn begin(&self, relay_state: Option<String>) -> Result<BeginRedirect, SsoError> {
        Ok(BeginRedirect {
            url: format!("https://idp.example/authorize?state={STATE_TOKEN}"),
            state_token: STATE_TOKEN.to_string(),
            pending: PendingState::Oidc {
                pkce_verifier: "verifier".into(),
                nonce: "nonce".into(),
                relay_state,
            },
        })
    }
    async fn complete(&self, _callback: SsoCallback) -> Result<SsoIdentity, SsoError> {
        Ok(self.identity.clone())
    }
    fn metadata(&self) -> Option<Metadata> {
        None
    }
    fn logout(&self, _subject: &str) -> Option<Redirect> {
        None
    }
}

#[tokio::test]
async fn sso_callback_does_not_mint_a_session_for_a_disabled_account() {
    let _g = serial().await;
    let user = "alice@acme.test";
    let (config, db) = app_config(ServerMode::Engine, Vec::new());
    // The allowlisted engine account the IdP identity resolves to.
    Store::open(&db, ServerKey::from_hex(KEY_HEX).unwrap())
        .await
        .unwrap()
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "imap.example",
                port: 993,
                tls: "implicit",
                username: user,
                sync_policy_json: "{}",
            },
            &Credentials {
                username: user.into(),
                password: "unused".into(),
            },
        )
        .await
        .unwrap();
    let provider = SsoEntry {
        provider: Arc::new(MockProvider {
            identity: SsoIdentity {
                subject: "subject-alice".into(),
                email: Some(user.into()),
                display_name: Some("Alice".into()),
                groups: vec![],
                claims: BTreeMap::new(),
            },
        }),
        meta: SsoMeta {
            display_name: "Acme".into(),
            kind: SsoKind::Oidc,
            scope: SsoScope::Deployment,
            enabled: true,
            first_login_policy: FirstLoginPolicy::Allowlist,
            claim_map: ClaimMap::default(),
        },
    };
    let app = mw_server::build_app_with_sso_mock(
        config,
        admin_v6(),
        vec![("corp-oidc".to_string(), provider)],
    )
    .await
    .expect("server boots")
    .0;
    let base = serve(app).await;
    let admin = admin(&base).await;

    // One full SSO round trip; returns the callback response.
    let sso_login = || async {
        bare()
            .get(format!("{base}/api/sso/corp-oidc/begin"))
            .send()
            .await
            .unwrap();
        bare()
            .get(format!(
                "{base}/api/sso/corp-oidc/callback?code=abc&state={STATE_TOKEN}"
            ))
            .send()
            .await
            .unwrap()
    };
    let session_of = |resp: &reqwest::Response| -> Option<String> {
        resp.headers()
            .get_all(reqwest::header::SET_COOKIE)
            .iter()
            .filter_map(|v| v.to_str().ok())
            .find_map(|c| c.strip_prefix("mw_session="))
            .and_then(|rest| rest.split(';').next())
            .filter(|v| !v.is_empty())
            .map(str::to_string)
    };
    let cookie_me = |sid: String| {
        let req = bare()
            .get(format!("{base}/api/me"))
            .header("Cookie", format!("mw_session={sid}"));
        async move { req.send().await.unwrap() }
    };

    let ok = sso_login().await;
    assert_eq!(
        ok.status().as_u16(),
        303,
        "control: the SSO login completes"
    );
    let sid = session_of(&ok).expect("control: the callback sets a session cookie");
    assert_eq!(cookie_me(sid.clone()).await.status(), 200);

    set_flags(&admin, &base, user, true, false).await;

    assert_eq!(
        cookie_me(sid).await.status(),
        401,
        "the SSO session established before the flag was set is refused"
    );
    let refused = sso_login().await;
    assert_eq!(
        refused.status().as_u16(),
        401,
        "the IdP vouching for the user does not override the flag"
    );
    assert!(session_of(&refused).is_none(), "no session cookie");
}

// ── 7. API key, OAuth token, refresh grant, /mcp (engine mode) ───────────────

async fn mcp_call(base: &str, bearer: &str, account_id: &str) -> (u16, Value) {
    let resp = bare()
        .post(format!("{base}/mcp"))
        .bearer_auth(bearer)
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": { "name": "mail.search", "arguments": { "account": account_id, "query": "x" } },
        }))
        .send()
        .await
        .unwrap();
    let status = resp.status().as_u16();
    (status, resp.json().await.unwrap_or(Value::Null))
}

fn rpc_error(v: &Value) -> Option<i64> {
    v.get("error").and_then(|e| e["code"].as_i64())
}

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

/// Authorization-code + PKCE through the server's own routes; returns the token
/// endpoint's body (`access_token`, `refresh_token`).
async fn mint_oauth_pair(c: &reqwest::Client, base: &str, account_id: &str) -> Value {
    let decision: Value = c
        .post(format!("{base}/oauth/decision"))
        .json(&json!({ "approve": true, "params": {
            "clientId": CLIENT_ID,
            "redirectUri": REDIRECT,
            "codeChallenge": mw_oauth::challenge_s256(VERIFIER),
            "codeChallengeMethod": "S256",
            "resource": RESOURCE,
            "scope": read_scope(account_id),
        }}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let redirect = decision["redirectUri"]
        .as_str()
        .unwrap_or_else(|| panic!("decision returned no redirect: {decision}"));
    let code = percent_decode(
        redirect
            .split("code=")
            .nth(1)
            .and_then(|r| r.split('&').next())
            .unwrap_or_else(|| panic!("no code in redirect: {redirect}")),
    );
    let pair: Value = c
        .post(format!("{base}/oauth/token"))
        .json(&json!({
            "grant_type": "authorization_code",
            "code": code,
            "redirect_uri": REDIRECT,
            "client_id": CLIENT_ID,
            "code_verifier": VERIFIER,
            "resource": RESOURCE,
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(pair["access_token"].is_string(), "token endpoint: {pair}");
    pair
}

async fn refresh(base: &str, refresh_token: &str) -> reqwest::Response {
    bare()
        .post(format!("{base}/oauth/token"))
        .json(&json!({
            "grant_type": "refresh_token",
            "refresh_token": refresh_token,
            "client_id": CLIENT_ID,
        }))
        .send()
        .await
        .unwrap()
}

async fn introspect_active(base: &str, token: &str) -> Value {
    let v: Value = bare()
        .post(format!("{base}/oauth/introspect"))
        .json(&json!({ "token": token }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    v["active"].clone()
}

async fn key_mailboxes(base: &str, key: &str) -> reqwest::Response {
    bare()
        .get(format!("{base}/api/v1/mailboxes"))
        .bearer_auth(key)
        .send()
        .await
        .unwrap()
}

#[tokio::test]
async fn disabled_account_keys_and_tokens_are_refused_on_rest_mcp_and_refresh() {
    let _g = serial().await;
    let pop = MockPop3::start(ENGINE_USER, ENGINE_PASS).await;
    let srv = spawn_engine().await;
    let base = &srv.base;
    let store = srv.store().await;
    let admin = admin(base).await;
    store
        .put_oauth_client(&OAuthClientRow {
            client_id: CLIENT_ID.into(),
            name: "t27 client".into(),
            redirect_uris_json: json!([REDIRECT]).to_string(),
            approved_by: "admin".into(),
            created_at: "2026-01-01T00:00:00Z".into(),
        })
        .await
        .unwrap();

    let c = browser();
    let login = engine_login(&c, base, &pop).await;
    assert_eq!(login.status(), 200, "control: engine login");
    let account_id = login.json::<Value>().await.unwrap()["accountId"]
        .as_str()
        .unwrap()
        .to_string();

    // Credentials issued while the account is enabled.
    let key = mint_api_key(&c, base, &account_id).await;
    let pair = mint_oauth_pair(&c, base, &account_id).await;
    let access = pair["access_token"].as_str().unwrap().to_string();
    let refresh_1 = pair["refresh_token"].as_str().unwrap().to_string();

    // Controls: each credential works on each surface.
    assert_eq!(
        key_mailboxes(base, &key).await.status(),
        200,
        "control: the API key reads /api/v1/mailboxes"
    );
    let (status, body) = mcp_call(base, &key, &account_id).await;
    assert!(
        status == 200 && rpc_error(&body) != Some(-32001),
        "control: the API key is authorised on /mcp: {status} {body}"
    );
    let (status, body) = mcp_call(base, &access, &account_id).await;
    assert!(
        status == 200 && rpc_error(&body) != Some(-32001),
        "control: the OAuth access token is authorised on /mcp: {status} {body}"
    );
    assert_eq!(introspect_active(base, &access).await, json!(true));
    let rotated = refresh(base, &refresh_1).await;
    assert_eq!(rotated.status(), 200, "control: the refresh grant works");
    let rotated: Value = rotated.json().await.unwrap();
    let access = rotated["access_token"].as_str().unwrap().to_string();
    let refresh_2 = rotated["refresh_token"].as_str().unwrap().to_string();
    let (status, body) = mcp_call(base, &access, &account_id).await;
    assert!(status == 200 && rpc_error(&body) != Some(-32001), "{body}");

    set_flags(&admin, base, ENGINE_USER, true, false).await;

    let refused = key_mailboxes(base, &key).await;
    assert_eq!(
        refused.status(),
        401,
        "an API key minted before the flag was set is refused on /api/v1"
    );
    assert_eq!(
        refused.json::<Value>().await.unwrap(),
        json!({ "error": "invalid api key" }),
        "refused as an unknown key, without naming the account's state"
    );
    let (status, body) = mcp_call(base, &key, &account_id).await;
    assert!(
        status == 401 || rpc_error(&body) == Some(-32001),
        "the API key is refused on /mcp: {status} {body}"
    );
    let (status, body) = mcp_call(base, &access, &account_id).await;
    assert!(
        status == 401 || rpc_error(&body) == Some(-32001),
        "the OAuth access token is refused on /mcp: {status} {body}"
    );
    assert_eq!(
        introspect_active(base, &access).await,
        json!(false),
        "a disabled account's token does not introspect as active"
    );
    let no_refresh = refresh(base, &refresh_2).await;
    assert_eq!(
        no_refresh.status(),
        400,
        "the refresh grant mints nothing for a disabled account"
    );
    assert!(
        no_refresh.json::<Value>().await.unwrap()["access_token"].is_null(),
        "no token in the refusal"
    );

    // Re-enable: the same key and the same refresh token work again, so the
    // refusals above were the flag and nothing was consumed by them.
    set_flags(&admin, base, ENGINE_USER, false, false).await;
    let again = browser();
    assert_eq!(engine_login(&again, base, &pop).await.status(), 200);
    assert_eq!(key_mailboxes(base, &key).await.status(), 200);
    assert_eq!(refresh(base, &refresh_2).await.status(), 200);
}

// ── 8. API key in proxy mode ─────────────────────────────────────────────────

#[tokio::test]
async fn proxy_mode_key_of_a_disabled_account_reads_nothing() {
    let _g = serial().await;
    let (srv, mock) = spawn_proxy().await;
    let base = &srv.base;
    let (user, pass) = (mw_mock_jmap::USER, mw_mock_jmap::PASS);
    let admin = admin(base).await;

    let c = browser();
    let login = proxy_login(&c, base, &mock, user, pass).await;
    let account_id = login.json::<Value>().await.unwrap()["accountId"]
        .as_str()
        .unwrap()
        .to_string();
    let key = mint_api_key(&c, base, &account_id).await;
    let read = || async {
        bare()
            .get(format!("{base}/api/v1/messages?limit=5"))
            .bearer_auth(&key)
            .send()
            .await
            .unwrap()
    };
    let before = read().await;
    assert_eq!(before.status(), 200, "control: the key reads the mailbox");
    assert!(
        before.json::<Value>().await.unwrap()["messages"].is_array(),
        "control: it returns message data"
    );

    set_flags(&admin, base, user, true, false).await;

    // In proxy mode a key's account id is the upstream's, which maps to a login
    // name only through a live session — and disabling deleted those. Whether the
    // key is refused by name or simply has no upstream credentials left, it must
    // not return mailbox data.
    let after = read().await;
    let status = after.status().as_u16();
    let body = after.text().await.unwrap_or_default();
    eprintln!("[t27-e3] proxy-mode key after disable → {status} {body}");
    assert_ne!(status, 200, "the key reads nothing after disable: {body}");
    assert!(
        !body.contains("\"messages\""),
        "no mailbox data in the refusal: {body}"
    );
}

// ── 9. force_password_change ─────────────────────────────────────────────────

fn argon2_hash(pw: &str) -> String {
    use argon2::Argon2;
    use argon2::password_hash::{PasswordHasher, SaltString};
    let salt = SaltString::encode_b64(b"t27-fixed-salt16").unwrap();
    Argon2::default()
        .hash_password(pw.as_bytes(), &salt)
        .unwrap()
        .to_string()
}

/// The `forcePasswordChange` flag as the admin panel's user list reports it.
async fn panel_force_flag(admin: &reqwest::Client, base: &str, id: &str) -> Value {
    let users: Value = admin
        .get(format!("{base}/admin/users"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    users
        .as_array()
        .unwrap()
        .iter()
        .find(|u| u["accountId"] == id)
        .unwrap_or_else(|| panic!("{id} is not in the panel's user list: {users}"))["flags"]
        ["forcePasswordChange"]
        .clone()
}

#[tokio::test]
async fn forced_password_change_holds_the_account_until_the_password_is_changed() {
    let _g = serial().await;
    let pop = MockPop3::start(ENGINE_USER, ENGINE_PASS).await;
    let srv = spawn_engine().await;
    let base = &srv.base;
    let store = srv.store().await;
    let admin = admin(base).await;

    // The panel lists provisioned users; provision this one so its flags can be
    // read back through the panel.
    let (local, domain) = ENGINE_USER.split_once('@').unwrap();
    let r = admin
        .post(format!("{base}/admin/users"))
        .json(&json!({
            "domain": domain, "username": local,
            "quota": { "bytesLimit": 0, "msgLimit": 0 },
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 204, "provision the user");

    let c = browser();
    let login = engine_login(&c, base, &pop).await;
    assert_eq!(login.status(), 200);
    let body: Value = login.json().await.unwrap();
    assert!(body.get("passwordChangeRequired").is_none(), "{body}");
    let account_id = body["accountId"].as_str().unwrap().to_string();
    // The default `Local` password backend verifies against this hash.
    store
        .set_setting(
            &format!("passwd_local:{account_id}"),
            &argon2_hash(ENGINE_PASS),
        )
        .await
        .unwrap();
    let key = mint_api_key(&c, base, &account_id).await;

    let policy = || async {
        c.get(format!("{base}/api/password/policy"))
            .send()
            .await
            .unwrap()
    };
    let change = |old: &'static str, new: &'static str| {
        let req = c
            .post(format!("{base}/api/password"))
            .json(&json!({ "oldPassword": old, "newPassword": new }));
        async move { req.send().await.unwrap() }
    };

    // Controls, before the flag: nothing is held.
    assert_eq!(jmap(&c, base).await.status(), 200, "control: /jmap/api");
    let session = c.get(format!("{base}/jmap/session")).send().await.unwrap();
    assert_eq!(session.status(), 200, "control: /jmap/session");
    assert_eq!(
        key_mailboxes(base, &key).await.status(),
        200,
        "control: key"
    );
    let me_body: Value = me(&c, base).await.json().await.unwrap();
    assert!(me_body.get("passwordChangeRequired").is_none(), "{me_body}");
    let p: Value = policy().await.json().await.unwrap();
    assert_eq!(p["forceChange"], json!(false), "{p}");
    assert!(
        p["minLength"].is_number(),
        "existing policy fields kept: {p}"
    );
    assert_eq!(
        panel_force_flag(&admin, base, ENGINE_USER).await,
        json!(false)
    );

    set_flags(&admin, base, ENGINE_USER, false, true).await;

    // The established session is held: everything but the allow-list is 403.
    for (what, resp) in [
        ("POST /jmap/api", jmap(&c, base).await),
        (
            "GET /jmap/session",
            c.get(format!("{base}/jmap/session")).send().await.unwrap(),
        ),
        (
            "GET /api/keys",
            c.get(format!("{base}/api/keys")).send().await.unwrap(),
        ),
        (
            "POST /api/session/rotate",
            c.post(format!("{base}/api/session/rotate"))
                .send()
                .await
                .unwrap(),
        ),
        ("API key on /api/v1", key_mailboxes(base, &key).await),
    ] {
        assert_eq!(resp.status(), 403, "{what} is held");
        assert_eq!(
            resp.json::<Value>().await.unwrap(),
            held_body(),
            "{what} answers with the contract body"
        );
    }
    let (status, body) = mcp_call(base, &key, &account_id).await;
    assert_eq!(status, 403, "the key is held on /mcp too: {body}");
    assert_eq!(body, held_body());

    // The allow-list.
    let held_me = me(&c, base).await;
    assert_eq!(held_me.status(), 200, "GET /api/me is allowed while held");
    let held_me: Value = held_me.json().await.unwrap();
    assert_eq!(held_me["passwordChangeRequired"], json!(true), "{held_me}");
    assert_eq!(held_me["username"], json!(ENGINE_USER));
    let p = policy().await;
    assert_eq!(p.status(), 200, "GET /api/password/policy is allowed");
    assert_eq!(p.json::<Value>().await.unwrap()["forceChange"], json!(true));
    let index = c.get(format!("{base}/")).send().await.unwrap();
    assert_eq!(index.status(), 200, "static assets are served while held");
    assert!(index.text().await.unwrap().contains("MW_T27"));

    // A new login succeeds and says so; logout works while held.
    let c2 = browser();
    let held_login = engine_login(&c2, base, &pop).await;
    assert_eq!(held_login.status(), 200, "a held account can still log in");
    assert!(sets_session_cookie(&held_login));
    let held_login: Value = held_login.json().await.unwrap();
    assert_eq!(
        held_login["passwordChangeRequired"],
        json!(true),
        "{held_login}"
    );
    assert_eq!(jmap(&c2, base).await.status(), 403);
    let out = c2.post(format!("{base}/api/logout")).send().await.unwrap();
    assert_eq!(out.status(), 204, "POST /api/logout is allowed while held");
    assert_eq!(me(&c2, base).await.status(), 401, "and it logs out");

    // A failed change keeps its ordinary failure shape and does not release.
    let bad = change("not-the-old-password", "New-Str0ng-Pass!").await;
    assert_eq!(bad.status(), 403, "wrong current password");
    let bad: Value = bad.json().await.unwrap();
    assert!(
        bad.get("passwordChangeRequired").is_none() && bad["error"].is_string(),
        "a wrong-password 403 is not the gate's 403: {bad}"
    );
    let weak = change(ENGINE_PASS, "short").await;
    assert_eq!(weak.status(), 400, "policy violation");
    assert_eq!(jmap(&c, base).await.status(), 403, "still held");
    assert_eq!(
        panel_force_flag(&admin, base, ENGINE_USER).await,
        json!(true)
    );

    // A successful change releases the hold and clears the flag.
    let ok = change(ENGINE_PASS, "New-Str0ng-Pass!").await;
    assert_eq!(ok.status(), 200, "the change succeeds");
    let ok: Value = ok.json().await.unwrap();
    assert_eq!(ok["changed"], json!(true), "{ok}");
    assert!(
        ok["credentialsResealed"].is_number() && ok["zeroaccessRewrapRequired"].is_boolean(),
        "the success body keeps its fields: {ok}"
    );
    assert_eq!(
        panel_force_flag(&admin, base, ENGINE_USER).await,
        json!(false),
        "the flag the panel set is the flag the change cleared"
    );
    let me_body: Value = me(&c, base).await.json().await.unwrap();
    assert!(me_body.get("passwordChangeRequired").is_none(), "{me_body}");
    assert_eq!(jmap(&c, base).await.status(), 200, "released: /jmap/api");
    assert_eq!(
        key_mailboxes(base, &key).await.status(),
        200,
        "released: key"
    );
    assert_eq!(
        policy().await.json::<Value>().await.unwrap()["forceChange"],
        json!(false)
    );
}
