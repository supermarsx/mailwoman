//! t28-c1 (t27 verification O4) — what the engine does for a disabled account.
//!
//! Before this, an engine-mode login for a disabled account was refused only
//! after `engine_mode::engine_login` had done all its work: the account row was
//! written (or its credentials re-sealed), the runtime registered, the mailbox
//! resynced and a watch started. And an account that was connected when the
//! admin disabled it stayed connected and kept syncing until its next refused
//! login or a restart.
//!
//! Every leg drives the real router over HTTP in engine mode against a scripted
//! POP3 server that counts accepted `PASS` commands. The POP3 backend opens one
//! authenticated session per operation, so that count is the number of
//! operations the server ran against the mailbox.
//!
//! Legs:
//!   * a refused login of a disabled account runs one operation — the credential
//!     check — and no more, and answers exactly as a wrong password does
//!   * a disabled name that has never logged in gets no account row
//!   * disabling a connected account drops its runtime at the flag write: once
//!     re-enabled, the next caller has to connect it again (a runtime that had
//!     survived would be reused without a connection)
//!   * while disabled, the admin metadata passthrough does not connect it
//!
//! What this does not observe: the watch loop itself. Dropping the runtime is
//! what ends it (`mw_engine::Engine::unregister`); the POP3 poll interval is five
//! minutes, so no poll falls inside a test run either way.
//!
//! Run:
//!   cargo test -p mw-server --test t28_disabled_engine_work --locked -- --test-threads=1

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::Duration;

use serde_json::{Value, json};
use tokio::sync::{Mutex, MutexGuard};

use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config};
use mw_store::{ServerKey, Store};

mod common;
use common::mock_mail::MockPop3;
use common::test_db;

const KEY_HEX: &str = "28c1a1b2c3d4e5f60718293a4b5c6d7e28c1a1b2c3d4e5f60718293a4b5c6d7e";
const USER: &str = "owner@example.org";
const PASS: &str = "Old-Passw0rd!";

/// Long enough for a resync or a watch start that a login had kicked off to
/// reach the mock: both run before `engine_login` returns, and the mock is on
/// loopback.
const SETTLE: Duration = Duration::from_millis(1500);

/// Serialises the tests in this binary: the server reads `MW_ENGINE_TLS` per login.
async fn serial() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let guard = LOCK.get_or_init(Default::default).lock().await;
    // SAFETY: the guard serialises every mutation and every dependent read here.
    unsafe {
        std::env::set_var("MW_ENGINE_TLS", "plaintext");
        for k in ["MW_HEADER_AUTH", "MW_LDAP_BIND_AUTH", "MW_PASSWD_BACKEND"] {
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
    async fn store(&self) -> Store {
        Store::open(&self.db, ServerKey::from_hex(KEY_HEX).unwrap())
            .await
            .expect("open the server's store")
    }
}

async fn spawn_engine() -> Server {
    let dir = test_db::unique_dir("mw-t28-c1");
    let web: PathBuf = dir.join("web");
    std::fs::create_dir_all(&web).unwrap();
    std::fs::write(
        web.join("index.html"),
        "<!doctype html><title>Mailwoman</title>",
    )
    .unwrap();
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

async fn login(base: &str, pop: &MockPop3, password: &str) -> reqwest::Response {
    browser()
        .post(format!("{base}/api/login"))
        .json(&json!({ "jmapUrl": pop.url(), "username": USER, "password": password }))
        .send()
        .await
        .unwrap()
}

/// The admin metadata passthrough for `account_id`: a `/jmap/api` request with an
/// admin session and no mailbox session, which connects the account if this
/// process has no runtime for it.
async fn admin_metadata(admin: &reqwest::Client, base: &str, account_id: &str) -> u16 {
    admin
        .post(format!("{base}/jmap/api"))
        .json(&json!({
            "using": ["urn:ietf:params:jmap:core"],
            "methodCalls": [
                ["ServerMetadata/get", { "accountId": account_id, "mailboxId": null }, "sm"],
            ],
        }))
        .send()
        .await
        .unwrap()
        .status()
        .as_u16()
}

#[tokio::test]
async fn a_refused_login_of_a_disabled_account_runs_the_credential_check_and_nothing_else() {
    let _g = serial().await;
    let pop = MockPop3::start(USER, PASS).await;
    let srv = spawn_engine().await;
    let base = &srv.base;
    let admin = admin(base).await;

    // Control: the login works, and it runs more than one operation (the
    // credential check, then the resync) — so "exactly one" below is a difference.
    let ok = login(base, &pop, PASS).await;
    assert_eq!(ok.status(), 200, "control: engine login");
    tokio::time::sleep(SETTLE).await;
    let enabled_ops = pop.logins_accepted();
    assert!(
        enabled_ops >= 2,
        "control: an accepted login authenticates and then syncs ({enabled_ops} operation(s))"
    );

    // What a wrong password is answered with, for the comparison below.
    let wrong = login(base, &pop, "not-the-password").await;
    let wrong_status = wrong.status();
    let wrong_body: Value = wrong.json().await.unwrap();
    assert_eq!(wrong_status, 401);

    set_disabled(&admin, base, USER, true).await;
    tokio::time::sleep(SETTLE).await;
    let before = pop.logins_accepted();

    let refused = login(base, &pop, PASS).await;
    assert_eq!(
        refused.status(),
        wrong_status,
        "a disabled account is refused"
    );
    assert_eq!(
        refused.json::<Value>().await.unwrap(),
        wrong_body,
        "with the body a wrong password gets"
    );
    tokio::time::sleep(SETTLE).await;
    assert_eq!(
        pop.logins_accepted() - before,
        1,
        "the refused login authenticated once and ran no other operation on the mailbox"
    );
}

#[tokio::test]
async fn a_disabled_name_that_never_logged_in_gets_no_account_row() {
    let _g = serial().await;
    let pop = MockPop3::start(USER, PASS).await;
    let srv = spawn_engine().await;
    let base = &srv.base;
    let admin = admin(base).await;
    let store = srv.store().await;

    set_disabled(&admin, base, USER, true).await;
    let refused = login(base, &pop, PASS).await;
    assert_eq!(refused.status(), 401, "a disabled account is refused");
    tokio::time::sleep(SETTLE).await;
    assert_eq!(
        pop.logins_accepted(),
        1,
        "the password was checked against the mail server, once"
    );
    assert_eq!(
        store.list_accounts().await.unwrap().len(),
        0,
        "the refused login wrote no account row"
    );

    // Control: the same login, enabled, is what writes the row.
    set_disabled(&admin, base, USER, false).await;
    assert_eq!(login(base, &pop, PASS).await.status(), 200);
    assert_eq!(store.list_accounts().await.unwrap().len(), 1);
}

#[tokio::test]
async fn disabling_a_connected_account_drops_its_runtime_at_the_flag_write() {
    let _g = serial().await;
    let pop = MockPop3::start(USER, PASS).await;
    let srv = spawn_engine().await;
    let base = &srv.base;
    let admin = admin(base).await;

    let ok = login(base, &pop, PASS).await;
    assert_eq!(ok.status(), 200, "control: engine login");
    let account_id = ok.json::<Value>().await.unwrap()["accountId"]
        .as_str()
        .unwrap()
        .to_string();
    tokio::time::sleep(SETTLE).await;

    // Control: while the account is connected the passthrough reuses its
    // runtime and connects nothing.
    let connected = pop.logins_accepted();
    assert_eq!(admin_metadata(&admin, base, &account_id).await, 200);
    assert_eq!(
        pop.logins_accepted(),
        connected,
        "control: a connected account is not connected again"
    );

    // Disable. No login and no request of the account follows, so nothing but
    // the flag write itself can have dropped the runtime.
    set_disabled(&admin, base, USER, true).await;
    tokio::time::sleep(SETTLE).await;
    let disabled = pop.logins_accepted();

    // While disabled, a caller with no session does not connect it.
    assert_eq!(
        admin_metadata(&admin, base, &account_id).await,
        502,
        "the passthrough reports a disabled account as unavailable"
    );
    tokio::time::sleep(SETTLE).await;
    assert_eq!(
        pop.logins_accepted(),
        disabled,
        "nothing reached the mail server for the disabled account"
    );

    // Re-enabled: the runtime is gone, so the next caller connects the account.
    set_disabled(&admin, base, USER, false).await;
    assert_eq!(admin_metadata(&admin, base, &account_id).await, 200);
    assert!(
        pop.logins_accepted() > disabled,
        "the runtime was dropped when the flag was set: re-enabled, the account had to be connected again"
    );
}
