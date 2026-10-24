//! t30-e3 — a session row that does not open under the server's key does not
//! lock its account out.
//!
//! A session row holds the upstream credentials sealed under the server key
//! (`MW_SERVER_KEY`). A row written under another key stays in the table and can
//! never be opened again: the server was started with a different key, or with
//! none (it then generates one per start), or two servers with different keys
//! share a database.
//!
//! Since 26.20 the account gate collects the names an account is known by on every
//! login, and it collected them by unsealing every session row of the account. One
//! row that did not open made the gate report its flags unreadable, and
//! `POST /api/login` answered 503 for that account from then on (CI: `e2e-v6`,
//! `v6_e2e.rs:351`). The other readers of an account's sessions failed the same
//! way: a headless API key in proxy mode and `POST /api/password`.
//!
//! Every leg drives the real router over HTTP, against two servers that share one
//! database and hold different keys. Each asserts its premise first — the row is
//! there and does not open under the second key — and each refusal is preceded by
//! the same request succeeding.
//!
//! Legs:
//!   * a login on the second server is accepted, and its session works
//!   * the gate is not weakened: a name recorded at a login on the first server is
//!     still read, so a flag under it refuses the account (t27-f1 O1); a flag
//!     record that cannot be read still refuses with 503
//!   * a database error on the read of the session rows is not a row that does
//!     not open: the gate still refuses the login with 503
//!   * a key reads mail and the password is changed through the rows that open;
//!     the current-password check still refuses a wrong password
//!
//! Run:
//!   cargo test -p mw-server --test t30_session_key_change --locked -- --test-threads=1

use std::net::SocketAddr;
use std::path::PathBuf;

use serde_json::{Value, json};

use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config};
use mw_store::{Credentials, ServerKey, Store};

mod common;
use common::test_db;

/// The key of the server the first login is made on.
const KEY_A: &str = "30e3a1b2c3d4e5f60718293a4b5c6d7e30e3a1b2c3d4e5f60718293a4b5c6d7e";
/// The key of the server the same database is then served under.
const KEY_B: &str = "30e3b1a2d3c4f5e61708392a5b4c7d6e30e3b1a2d3c4f5e61708392a5b4c7d6e";
const INDEX_HTML: &str = "<!doctype html><title>Mailwoman</title><div id=app>MW_T30</div>";

/// A second login name the alias upstream accepts for the mock's one mailbox.
const ALIAS: &str = "tester";

// ── harness ──────────────────────────────────────────────────────────────────

/// One database and one web directory, served by more than one server.
struct Deployment {
    db: String,
    web: PathBuf,
}

impl Deployment {
    fn new() -> Self {
        let dir = test_db::unique_dir("mw-t30-e3-key");
        let web = dir.join("web");
        std::fs::create_dir_all(&web).unwrap();
        std::fs::write(web.join("index.html"), INDEX_HTML).unwrap();
        Deployment {
            db: dir.join("mw.db").to_string_lossy().into_owned(),
            web,
        }
    }

    /// A proxy-mode server over this database, holding `key`.
    async fn serve(&self, key: &str, upstream: &str) -> String {
        let config = AppConfig {
            db_path: self.db.clone(),
            server_key_hex: Some(key.to_string()),
            web_dir: Some(self.web.clone()),
            cookie_secure: false,
            mode: ServerMode::Proxy,
            hardening: HardeningConfig::default(),
            security: SecurityConfig {
                jmap_upstreams: Some(vec![upstream.to_string()]),
                ..SecurityConfig::default()
            },
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
        format!("http://{addr}")
    }

    /// A handle on the database under `key`: for the premises (which rows open),
    /// and for seeding what no HTTP route writes.
    async fn store(&self, key: &str) -> Store {
        Store::open(&self.db, ServerKey::from_hex(key).unwrap())
            .await
            .expect("open the deployment's store")
    }
}

async fn spawn_mock_jmap() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, mw_mock_jmap::router()).await.unwrap();
    });
    format!("http://{addr}")
}

fn basic(user: &str, pass: &str) -> String {
    use base64::Engine as _;
    format!(
        "Basic {}",
        base64::engine::general_purpose::STANDARD.encode(format!("{user}:{pass}"))
    )
}

/// The mock JMAP upstream behind a front that also accepts the login name
/// [`ALIAS`] for its one mailbox. Like a mail server with aliases, it reports the
/// mailbox's canonical name (`mw_mock_jmap::USER`) whichever name was used.
async fn spawn_alias_jmap() -> String {
    let front = mw_mock_jmap::router().layer(axum::middleware::from_fn(
        |mut req: axum::extract::Request, next: axum::middleware::Next| async move {
            let presented = req
                .headers()
                .get("authorization")
                .and_then(|v| v.to_str().ok())
                .map(str::to_string);
            if presented.as_deref() == Some(basic(ALIAS, mw_mock_jmap::PASS).as_str()) {
                req.headers_mut().insert(
                    "authorization",
                    basic(mw_mock_jmap::USER, mw_mock_jmap::PASS)
                        .parse()
                        .unwrap(),
                );
            }
            next.run(req).await
        },
    ));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, front).await.unwrap();
    });
    format!("http://{addr}")
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

async fn login(
    c: &reqwest::Client,
    base: &str,
    _mock: &str,
    user: &str,
    pass: &str,
) -> reqwest::Response {
    c.post(format!("{base}/api/login"))
        .json(&json!({ "username": user, "password": pass }))
        .send()
        .await
        .unwrap()
}

/// The session id a login response sets, if it sets one.
fn session_cookie(resp: &reqwest::Response) -> Option<String> {
    resp.headers()
        .get_all(reqwest::header::SET_COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .filter_map(|c| c.strip_prefix("mw_session="))
        .map(|rest| rest.split(';').next().unwrap_or_default().to_string())
        .find(|id| !id.is_empty())
}

async fn me(c: &reqwest::Client, base: &str) -> reqwest::Response {
    c.get(format!("{base}/api/me")).send().await.unwrap()
}

/// `GET /api/me` presenting `session` as the cookie, with no cookie jar.
async fn me_with(base: &str, session: &str) -> reqwest::Response {
    bare()
        .get(format!("{base}/api/me"))
        .header(reqwest::header::COOKIE, format!("mw_session={session}"))
        .send()
        .await
        .unwrap()
}

/// An empty JMAP request: reaches the handler behind `authed` and needs no data.
async fn jmap(c: &reqwest::Client, base: &str) -> reqwest::Response {
    c.post(format!("{base}/jmap/api"))
        .json(&json!({ "using": ["urn:ietf:params:jmap:core"], "methodCalls": [] }))
        .send()
        .await
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

/// Set an account's `disabled` flag the way the panel does.
async fn set_disabled(admin: &reqwest::Client, base: &str, id: &str, disabled: bool) {
    let r = admin
        .put(format!("{base}/admin/users/{id}/flags"))
        .json(&json!({
            "zeroAccess": false,
            "forcePasswordChange": false,
            "remoteCacheWipe": false,
            "disabled": disabled,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 204, "the panel's flag write is accepted");
}

/// A login on the first server, and the same database then served under another
/// key. Returns what the legs need, with the premise already asserted: the row
/// the login wrote is in the table and does not open under the second key.
struct KeyChange {
    dep: Deployment,
    mock: String,
    /// The server holding [`KEY_A`].
    a: String,
    /// The server holding [`KEY_B`].
    b: String,
    account_id: String,
    /// The id of the session opened on `a`.
    session_a: String,
}

async fn key_change(mock: String, login_name: &str) -> KeyChange {
    let dep = Deployment::new();
    let a = dep.serve(KEY_A, &mock).await;
    let first = login(&browser(), &a, &mock, login_name, mw_mock_jmap::PASS).await;
    assert_eq!(first.status(), 200, "control: login on the first server");
    let session_a = session_cookie(&first).expect("the first login sets a session cookie");
    let body: Value = first.json().await.unwrap();
    let account_id = body["accountId"].as_str().unwrap().to_string();
    assert_eq!(
        body["username"],
        json!(mw_mock_jmap::USER),
        "the upstream reports the mailbox's canonical name: {body}"
    );
    assert_eq!(me_with(&a, &session_a).await.status(), 200);

    let b = dep.serve(KEY_B, &mock).await;

    // The premise: the row is there, and it does not open under the second key.
    let opened = dep.store(KEY_A).await.get_session(&session_a).await;
    assert_eq!(
        opened.expect("the row opens under its own key").account_id,
        account_id
    );
    assert!(
        dep.store(KEY_B)
            .await
            .get_session(&session_a)
            .await
            .is_err(),
        "the premise: the row does not open under the second key"
    );
    // What such a row is to the second server: not a session.
    assert_eq!(
        me_with(&b, &session_a).await.status(),
        401,
        "a row that does not open authenticates nothing"
    );

    KeyChange {
        dep,
        mock,
        a,
        b,
        account_id,
        session_a,
    }
}

// ── 1. the login ─────────────────────────────────────────────────────────────

/// The defect: with one session row of the account sealed under another key, the
/// second server answered every login to that account with 503.
#[tokio::test]
async fn a_login_is_accepted_when_a_session_row_was_sealed_under_another_key() {
    let k = key_change(spawn_mock_jmap().await, mw_mock_jmap::USER).await;
    let (user, pass) = (mw_mock_jmap::USER, mw_mock_jmap::PASS);

    let c = browser();
    let second = login(&c, &k.b, &k.mock, user, pass).await;
    let status = second.status();
    let cookie = session_cookie(&second);
    let body = second.text().await.unwrap();
    assert_eq!(
        status, 200,
        "POST /api/login on the server holding another key: {body}"
    );
    let body: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(body["accountId"], json!(k.account_id), "the same account");
    let session_b = cookie.expect("the login sets a session cookie");
    assert_ne!(session_b, k.session_a);
    assert_eq!(me(&c, &k.b).await.status(), 200, "the new session works");
    assert_eq!(jmap(&c, &k.b).await.status(), 200);

    // A wrong password is still a wrong password, not a server error.
    let wrong = login(&browser(), &k.b, &k.mock, user, "not-the-password").await;
    assert_eq!(wrong.status(), 401);

    // The login neither removed nor rewrote the row it could not open: the first
    // server still accepts it, and the second still does not.
    assert_eq!(me_with(&k.a, &k.session_a).await.status(), 200);
    assert_eq!(me_with(&k.b, &k.session_a).await.status(), 401);
    // And a second login, with that row and the new one both present.
    assert_eq!(
        login(&browser(), &k.b, &k.mock, user, pass).await.status(),
        200
    );
}

// ── 2. the gate still reads the recorded names, and still fails closed ───────

/// t27-f1 O1 across the key change. The first server recorded the name typed at
/// login (`v6:account:names:<id>`) and sealed it in the session row. Under the
/// second key the row is unreadable, so the record is the only place the name is
/// known from: a flag under it must still refuse the account.
#[tokio::test]
async fn an_account_disabled_under_a_name_recorded_before_the_key_change_is_refused() {
    let k = key_change(spawn_alias_jmap().await, ALIAS).await;
    let (user, pass) = (mw_mock_jmap::USER, mw_mock_jmap::PASS);
    let store_a = k.dep.store(KEY_A).await;
    let store_b = k.dep.store(KEY_B).await;
    let admin = admin(&k.b).await;

    // Controls on the second server: the account logs in under the canonical
    // name (the only name this server has seen typed), and its session works.
    let c = browser();
    let control = login(&c, &k.b, &k.mock, user, pass).await;
    assert_eq!(control.status(), 200, "control: login before the flag");
    assert_eq!(me(&c, &k.b).await.status(), 200);
    let wrong_body: Value = login(&browser(), &k.b, &k.mock, user, "not-the-password")
        .await
        .json()
        .await
        .unwrap();

    set_disabled(&admin, &k.b, ALIAS, true).await;

    // Disabling deletes the account's session rows. Put one back as the first
    // server would write it, so the refusals below are made with a row present
    // that does not open — the state the defect was in.
    let dead = store_a
        .create_session(
            &k.account_id,
            user,
            &k.mock,
            &k.mock,
            &Credentials {
                username: ALIAS.into(),
                password: pass.into(),
            },
        )
        .await
        .unwrap();
    assert!(
        store_b.get_session(&dead).await.is_err(),
        "the premise: a row of the account does not open under the second key"
    );

    for name in [user, ALIAS] {
        let refused = login(&browser(), &k.b, &k.mock, name, pass).await;
        assert_eq!(
            refused.status(),
            401,
            "an account disabled under {ALIAS} does not log in as {name}"
        );
        assert!(session_cookie(&refused).is_none(), "no session is issued");
        assert_eq!(
            refused.json::<Value>().await.unwrap(),
            wrong_body,
            "the refusal is the wrong-password answer"
        );
    }
    assert_eq!(
        me(&c, &k.b).await.status(),
        401,
        "the session opened before the flag is gone"
    );

    // Re-enabled: the login works again with the unopenable row still present, so
    // the refusals above were the flag and not the row.
    set_disabled(&admin, &k.b, ALIAS, false).await;
    assert!(store_a.get_session(&dead).await.is_ok());
    assert!(store_b.get_session(&dead).await.is_err());
    assert_eq!(
        login(&browser(), &k.b, &k.mock, user, pass).await.status(),
        200,
        "re-enabled"
    );

    // A flag record that cannot be read still refuses. It is the alias's record,
    // which a login under the canonical name reaches only through the recorded
    // names.
    store_b
        .set_setting(&format!("v6:admin:flags:{ALIAS}"), "{not json")
        .await
        .unwrap();
    let unreadable = login(&browser(), &k.b, &k.mock, user, pass).await;
    assert_eq!(
        unreadable.status(),
        503,
        "flags that cannot be read refuse the login"
    );
    assert!(session_cookie(&unreadable).is_none());
    set_disabled(&admin, &k.b, ALIAS, false).await;
    assert_eq!(
        login(&browser(), &k.b, &k.mock, user, pass).await.status(),
        200,
        "the record is valid again"
    );
}

/// A row that does not open is one thing; a read of the session rows that fails
/// in the database is another, and the gate must still refuse on it. The table is
/// renamed away, which no route can do, so the gate's read of the account's
/// session rows is the first statement of the login to fail. 503 is the gate's
/// own refusal (`account_gate::unavailable`); a login that got past the gate
/// would fail later, writing its session, with 500.
#[tokio::test]
async fn a_database_error_on_the_session_rows_still_refuses_the_login() {
    let k = key_change(spawn_mock_jmap().await, mw_mock_jmap::USER).await;
    let (user, pass) = (mw_mock_jmap::USER, mw_mock_jmap::PASS);
    assert_eq!(
        login(&browser(), &k.b, &k.mock, user, pass).await.status(),
        200,
        "control: the login is accepted with the row that does not open present"
    );

    let raw = sqlx::SqlitePool::connect_with(
        sqlx::sqlite::SqliteConnectOptions::new().filename(&k.dep.db),
    )
    .await
    .unwrap();
    sqlx::query("ALTER TABLE sessions RENAME TO sessions_away")
        .execute(&raw)
        .await
        .unwrap();
    let refused = login(&browser(), &k.b, &k.mock, user, pass).await;
    let status = refused.status();
    assert!(session_cookie(&refused).is_none(), "no session is issued");
    let body = refused.text().await.unwrap();
    assert_eq!(
        status, 503,
        "a database error reading the session rows refuses the login at the gate: {body}"
    );

    // With the table back the login is accepted again: the refusal was the
    // database error.
    sqlx::query("ALTER TABLE sessions_away RENAME TO sessions")
        .execute(&raw)
        .await
        .unwrap();
    assert_eq!(
        login(&browser(), &k.b, &k.mock, user, pass).await.status(),
        200
    );
}

// ── 3. the other readers of an account's sessions ────────────────────────────

fn argon2_hash(pw: &str) -> String {
    use argon2::Argon2;
    use argon2::password_hash::{PasswordHasher, SaltString};
    let salt = SaltString::encode_b64(b"t30-fixed-salt16").unwrap();
    Argon2::default()
        .hash_password(pw.as_bytes(), &salt)
        .unwrap()
        .to_string()
}

/// A headless API key in proxy mode borrows the upstream credentials of one of
/// the account's sessions (`scope_mw::rest_session`), and `POST /api/password`
/// checks the current password against them and re-seals them
/// (`v7_mount::CurrentPasswordCheck`, `Store::reseal_account_credentials`). With
/// one row that did not open, the key read answered 502 and the password change
/// 502.
#[tokio::test]
async fn a_key_and_a_password_change_use_the_session_rows_that_open() {
    let k = key_change(spawn_mock_jmap().await, mw_mock_jmap::USER).await;
    let (user, pass) = (mw_mock_jmap::USER, mw_mock_jmap::PASS);
    let store_a = k.dep.store(KEY_A).await;
    let store_b = k.dep.store(KEY_B).await;
    let base = &k.b;

    let c = browser();
    let second = login(&c, base, &k.mock, user, pass).await;
    assert_eq!(second.status(), 200, "login on the second server");
    let session_b = session_cookie(&second).expect("a session cookie");

    // The key, presented with no cookie: its mail read goes upstream with the
    // credentials of the session that opens.
    let scope = json!({
        "read": true, "send": false, "delete": false,
        "accounts": { "subset": [&k.account_id] }, "folders": "all",
        "mail": true, "pim": false, "ip_allowlist": [], "expires_at": null,
        "rate_limit": null, "mcp_tools": ["mail.search"], "unattended_send": false,
    });
    let mint: Value = c
        .post(format!("{base}/api/keys"))
        .json(&json!({ "label": "t30", "accountId": &k.account_id, "scope": scope }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let key = mint["displayToken"]
        .as_str()
        .unwrap_or_else(|| panic!("/api/keys returned no displayToken: {mint}"))
        .to_string();
    let read = bare()
        .get(format!("{base}/api/v1/messages?limit=5"))
        .bearer_auth(&key)
        .send()
        .await
        .unwrap();
    let status = read.status();
    let body = read.text().await.unwrap();
    assert_eq!(status, 200, "the key reads the mailbox: {body}");
    assert!(
        serde_json::from_str::<Value>(&body).unwrap()["messages"].is_array(),
        "{body}"
    );

    // The password change. The default `Local` backend verifies against this hash.
    store_b
        .set_setting(
            &format!("passwd_local:{}", k.account_id),
            &argon2_hash(pass),
        )
        .await
        .unwrap();
    let change = |old: &'static str, new: &'static str| {
        let req = c
            .post(format!("{base}/api/password"))
            .json(&json!({ "oldPassword": old, "newPassword": new }));
        async move { req.send().await.unwrap() }
    };
    // The current-password check is made against the rows that open, and is
    // still made.
    let wrong = change("not-the-old-password", "New-Str0ng-Pass!").await;
    let status = wrong.status();
    let body = wrong.text().await.unwrap();
    assert_eq!(status, 403, "a wrong current password is refused: {body}");

    let ok = change(pass, "New-Str0ng-Pass!").await;
    let status = ok.status();
    let body = ok.text().await.unwrap();
    assert_eq!(status, 200, "the password change: {body}");
    let body: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(body["changed"], json!(true), "{body}");
    assert_eq!(
        body["credentialsResealed"],
        json!(1),
        "the one session that opens is re-sealed: {body}"
    );
    assert_eq!(
        store_b
            .get_session(&session_b)
            .await
            .unwrap()
            .credentials
            .password,
        "New-Str0ng-Pass!",
        "the session that opens holds the new password"
    );
    assert_eq!(
        store_a
            .get_session(&k.session_a)
            .await
            .unwrap()
            .credentials
            .password,
        pass,
        "the row sealed under the other key is left as it was"
    );
}
