//! A server restart (same database, same server key, a brand-new router and
//! process state) leaves signed-in people signed in, and keeps every deadline
//! where it was. Covers mailbox sessions, console sessions, the CSRF token,
//! step-up, the persistence setting, a changed server key and the pending
//! single sign-on flow.
//!
//! "Restart" here builds a second app over the same SQLite file: nothing in the
//! first app's memory is available to it, which is what a new process has.

use std::path::{Path, PathBuf};

use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use serde_json::{Value, json};

mod common;

const ADMIN_USER: &str = "root";
const ADMIN_PASS: &str = "restart-survival-admin-password";

struct Upstream(String);

async fn mock() -> Upstream {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let origin = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move {
        axum::serve(listener, mw_mock_jmap::router()).await.unwrap();
    });
    Upstream(origin)
}

struct Running {
    base: String,
    task: tokio::task::JoinHandle<()>,
}

impl Running {
    fn stop(self) {
        self.task.abort();
    }
}

async fn boot(db: &Path, key: &str, upstream: &Upstream, csrf_strict: bool) -> Running {
    let (app, _push) = build_app_full(
        AppConfig {
            db_path: db.to_string_lossy().into_owned(),
            server_key_hex: Some(key.into()),
            web_dir: None,
            cookie_secure: false,
            mode: ServerMode::Proxy,
            hardening: HardeningConfig {
                csrf_strict,
                ..HardeningConfig::default()
            },
            security: SecurityConfig {
                jmap_upstreams: Some(vec![upstream.0.clone()]),
                upstream_default: Some(upstream.0.clone()),
                ..SecurityConfig::default()
            },
        },
        V6Config {
            admin_enabled: true,
            admin_username: Some(ADMIN_USER.into()),
            admin_password: Some(ADMIN_PASS.into()),
            redis_url: None,
        },
    )
    .await
    .unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let task = tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .unwrap();
    });
    Running { base, task }
}

fn client() -> reqwest::Client {
    reqwest::Client::builder().no_proxy().build().unwrap()
}

fn cookie(resp: &reqwest::Response, name: &str) -> Option<String> {
    resp.headers()
        .get_all("set-cookie")
        .iter()
        .filter_map(|h| h.to_str().ok())
        .find_map(|h| {
            h.strip_prefix(&format!("{name}="))
                .map(|v| v.split(';').next().unwrap().to_owned())
        })
}

struct Login {
    session: String,
    csrf: Option<String>,
}

async fn user_login(c: &reqwest::Client, base: &str) -> Login {
    let resp = c
        .post(format!("{base}/api/login"))
        .json(&json!({"username": mw_mock_jmap::USER, "password": mw_mock_jmap::PASS}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "login");
    Login {
        session: cookie(&resp, "mw_session").expect("session cookie"),
        csrf: cookie(&resp, "mw_csrf"),
    }
}

async fn me(c: &reqwest::Client, base: &str, session: &str) -> u16 {
    c.get(format!("{base}/api/me"))
        .header("Cookie", format!("mw_session={session}"))
        .send()
        .await
        .unwrap()
        .status()
        .as_u16()
}

async fn pool(db: &Path) -> sqlx::SqlitePool {
    sqlx::SqlitePool::connect(&format!("sqlite:{}", db.display()))
        .await
        .unwrap()
}

async fn set_policy(db: &Path, key: &str, value: &str) {
    sqlx::query(
        "INSERT INTO setting_policy(key,global_value,updated_by,updated_at) VALUES(?,?, 'restart-test',0) \
         ON CONFLICT(key) DO UPDATE SET global_value=excluded.global_value",
    )
    .bind(key)
    .bind(value)
    .execute(&pool(db).await)
    .await
    .unwrap();
}

fn fixture() -> (PathBuf, String) {
    (
        common::test_db::unique_db_path("session-restart-survival"),
        "5a".repeat(32),
    )
}

#[tokio::test]
async fn mailbox_session_survives_a_restart_and_deadlines_do_not_move() {
    let upstream = mock().await;
    let (db, key) = fixture();
    let c = client();
    let first = boot(&db, &key, &upstream, false).await;
    set_policy(&db, "signin.retryCooldownSecs", "0").await;
    let login = user_login(&c, &first.base).await;
    assert_eq!(me(&c, &first.base, &login.session).await, 200);

    let p = pool(&db).await;
    let before: (String, String) =
        sqlx::query_as("SELECT created_at, activity_at FROM sessions WHERE id=?")
            .bind(&login.session)
            .fetch_one(&p)
            .await
            .unwrap();
    first.stop();

    let second = boot(&db, &key, &upstream, false).await;
    assert_eq!(
        me(&c, &second.base, &login.session).await,
        200,
        "the same cookie works on a fresh router"
    );
    // The upstream credentials were opened lazily with the server key: a JMAP call
    // that needs them works without signing in again.
    let session = c
        .get(format!("{}/jmap/session", second.base))
        .header("Cookie", format!("mw_session={}", login.session))
        .send()
        .await
        .unwrap();
    assert_eq!(session.status(), 200, "upstream credentials re-open");
    let after: (String, String) =
        sqlx::query_as("SELECT created_at, activity_at FROM sessions WHERE id=?")
            .bind(&login.session)
            .fetch_one(&p)
            .await
            .unwrap();
    assert_eq!(before, after, "a restart extends neither deadline");

    // The absolute deadline is stored: moving creation into the past ends the
    // session on the next process too.
    sqlx::query("UPDATE sessions SET created_at='2000-01-01T00:00:00+00:00' WHERE id=?")
        .bind(&login.session)
        .execute(&p)
        .await
        .unwrap();
    second.stop();
    let third = boot(&db, &key, &upstream, false).await;
    assert_eq!(me(&c, &third.base, &login.session).await, 401);
    third.stop();
}

#[tokio::test]
async fn a_restart_does_not_look_like_a_new_device() {
    let upstream = mock().await;
    let (db, key) = fixture();
    let c = client();
    let first = boot(&db, &key, &upstream, false).await;
    set_policy(&db, "signin.retryCooldownSecs", "0").await;
    let login = user_login(&c, &first.base).await;
    let p = pool(&db).await;
    let events = |p: sqlx::SqlitePool| async move {
        let events: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM signin_event")
            .fetch_one(&p)
            .await
            .unwrap();
        let devices: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM remembered_device")
            .fetch_one(&p)
            .await
            .unwrap();
        (events, devices)
    };
    let before = events(p.clone()).await;
    first.stop();
    let second = boot(&db, &key, &upstream, false).await;
    assert_eq!(me(&c, &second.base, &login.session).await, 200);
    assert_eq!(
        events(p).await,
        before,
        "serving a restored session records no sign-in and no new device"
    );
    second.stop();
}

#[tokio::test]
async fn csrf_token_stays_valid_across_a_restart() {
    let upstream = mock().await;
    let (db, key) = fixture();
    let c = client();
    let first = boot(&db, &key, &upstream, true).await;
    set_policy(&db, "signin.retryCooldownSecs", "0").await;
    let login = user_login(&c, &first.base).await;
    let token = login.csrf.expect("strict mode issues a csrf cookie");
    first.stop();

    let second = boot(&db, &key, &upstream, true).await;
    let post = |token: &str| {
        c.post(format!("{}/jmap/api", second.base))
            .header(
                "Cookie",
                format!("mw_session={}; mw_csrf={token}", login.session),
            )
            .header("x-csrf-token", token.to_owned())
            .json(&json!({"using": ["urn:ietf:params:jmap:core"], "methodCalls": []}))
            .send()
    };
    assert_ne!(
        post(&token).await.unwrap().status(),
        403,
        "the token issued before the restart still verifies"
    );
    assert_eq!(
        post("not-the-token").await.unwrap().status(),
        403,
        "a forged token is still refused"
    );
    second.stop();
}

#[tokio::test]
async fn console_session_survives_a_restart_with_its_absolute_deadline() {
    let upstream = mock().await;
    let (db, key) = fixture();
    let c = client();
    let first = boot(&db, &key, &upstream, false).await;
    let resp = c
        .post(format!("{}/admin/login", first.base))
        .json(&json!({"username": ADMIN_USER, "password": ADMIN_PASS}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let name = mw_admin::config::session_cookie_name();
    let token = cookie(&resp, &name).expect("console cookie");

    let p = pool(&db).await;
    let row = || {
        let p = p.clone();
        async move {
            sqlx::query_as::<_, (i64, i64)>(
                "SELECT expires_at, absolute_expires_at FROM admin_sessions",
            )
            .fetch_one(&p)
            .await
            .unwrap()
        }
    };
    let (_, absolute_before) = row().await;
    first.stop();

    let second = boot(&db, &key, &upstream, false).await;
    let get = |base: &str| {
        c.get(format!("{base}/admin/session"))
            .header("Cookie", format!("{name}={token}"))
            .send()
    };
    assert_eq!(get(&second.base).await.unwrap().status(), 200);
    let (idle_after, absolute_after) = row().await;
    assert_eq!(
        absolute_before, absolute_after,
        "the absolute deadline does not reset on restart"
    );
    assert!(idle_after <= absolute_after);

    // Past its absolute deadline the stored session is refused by the next process.
    sqlx::query("UPDATE admin_sessions SET absolute_expires_at = 1")
        .execute(&p)
        .await
        .unwrap();
    second.stop();
    let third = boot(&db, &key, &upstream, false).await;
    assert_eq!(get(&third.base).await.unwrap().status(), 401);
    third.stop();
}

#[tokio::test]
async fn step_up_proof_is_per_session_and_not_carried_across_a_restart() {
    let upstream = mock().await;
    let (db, key) = fixture();
    let c = client();
    let first = boot(&db, &key, &upstream, false).await;
    set_policy(&db, "signin.retryCooldownSecs", "0").await;
    let one = user_login(&c, &first.base).await;
    let two = user_login(&c, &first.base).await;
    let status = |base: String, session: String| {
        let c = c.clone();
        async move {
            c.get(format!("{base}/api/security/step-up"))
                .header("Cookie", format!("mw_session={session}"))
                .send()
                .await
                .unwrap()
                .json::<Value>()
                .await
                .unwrap()
        }
    };
    let minted = c
        .post(format!("{}/api/security/step-up", first.base))
        .header("Cookie", format!("mw_session={}", one.session))
        .json(&json!({"password": mw_mock_jmap::PASS}))
        .send()
        .await
        .unwrap();
    assert!(minted.status().is_success(), "mint: {}", minted.status());
    assert_eq!(
        status(first.base.clone(), one.session.clone()).await["active"],
        true
    );
    assert_eq!(
        status(first.base.clone(), two.session.clone()).await["active"],
        false,
        "another session of the same account has no proof"
    );
    first.stop();

    let second = boot(&db, &key, &upstream, false).await;
    assert_eq!(me(&c, &second.base, &one.session).await, 200);
    assert_eq!(
        status(second.base.clone(), one.session.clone()).await["active"],
        false,
        "a proof is memory-only by design: the user confirms again after a restart"
    );
    second.stop();
}

#[tokio::test]
async fn turning_persistence_off_ends_sessions_at_start() {
    let upstream = mock().await;
    let (db, key) = fixture();
    let c = client();
    let first = boot(&db, &key, &upstream, false).await;
    set_policy(&db, "signin.retryCooldownSecs", "0").await;
    let login = user_login(&c, &first.base).await;
    assert_eq!(me(&c, &first.base, &login.session).await, 200);
    set_policy(&db, "session.persistAcrossRestarts", "false").await;
    first.stop();

    let second = boot(&db, &key, &upstream, false).await;
    assert_eq!(
        me(&c, &second.base, &login.session).await,
        401,
        "session.persistAcrossRestarts=false ends sessions when the server starts"
    );
    second.stop();
}

#[tokio::test]
async fn a_different_server_key_invalidates_sessions_without_crashing() {
    let upstream = mock().await;
    let (db, key) = fixture();
    let c = client();
    let first = boot(&db, &key, &upstream, false).await;
    set_policy(&db, "signin.retryCooldownSecs", "0").await;
    let login = user_login(&c, &first.base).await;
    first.stop();

    let second = boot(&db, &"7b".repeat(32), &upstream, false).await;
    assert_eq!(
        me(&c, &second.base, &login.session).await,
        401,
        "sealed credentials do not open under another key; the person signs in again"
    );
    // The server is up and the console reports why.
    let resp = c
        .post(format!("{}/admin/login", second.base))
        .json(&json!({"username": ADMIN_USER, "password": ADMIN_PASS}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let name = mw_admin::config::session_cookie_name();
    let token = cookie(&resp, &name).unwrap();
    let dashboard: Value = c
        .get(format!("{}/admin/security/dashboard", second.base))
        .header("Cookie", format!("{name}={token}"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let sessions = &dashboard["posture"]["sessions"];
    assert_eq!(sessions["survive"], false, "{dashboard}");
    assert_eq!(sessions["reason"], "server-key-changed");
    second.stop();
}

#[tokio::test]
async fn sealed_pending_state_is_shared_by_instances_and_single_use() {
    let (db, key) = fixture();
    let key = mw_store::ServerKey::from_hex(&key).unwrap();
    let a = mw_store::Store::open(&db.to_string_lossy(), key.clone())
        .await
        .unwrap();
    let b = mw_store::Store::open(&db.to_string_lossy(), key)
        .await
        .unwrap();
    a.put_ephemeral("sso-pending", "state-token", b"{\"nonce\":\"n\"}", 600)
        .await
        .unwrap();
    // Another instance (or the same one after a restart) redeems it once.
    assert_eq!(
        b.take_ephemeral("sso-pending", "state-token")
            .await
            .unwrap()
            .as_deref(),
        Some(&b"{\"nonce\":\"n\"}"[..])
    );
    assert_eq!(
        a.take_ephemeral("sso-pending", "state-token").await.unwrap(),
        None
    );
    // The lookup key is hashed and the payload sealed: the table shows neither.
    a.put_ephemeral("sso-pending", "t2", b"secret-nonce", 600)
        .await
        .unwrap();
    let raw: (String, Vec<u8>) = sqlx::query_as("SELECT key_hash, sealed FROM ephemeral_state")
        .fetch_one(&pool(&db).await)
        .await
        .unwrap();
    assert_ne!(raw.0, "t2");
    assert!(!String::from_utf8_lossy(&raw.1).contains("secret-nonce"));
}

#[tokio::test]
async fn remembered_device_is_still_recognised_after_a_restart() {
    let (db, key) = fixture();
    let key = mw_store::ServerKey::from_hex(&key).unwrap();
    let now = chrono::Utc::now().timestamp();
    let store = mw_store::Store::open(&db.to_string_lossy(), key.clone())
        .await
        .unwrap();
    store
        .put_remembered_device(
            &mw_store::RememberedDevice {
                id: "device-1".into(),
                account_id: "acct-1".into(),
                label: "Laptop".into(),
                created_at: now,
                expires_at: now + 86_400,
                last_used_at: now,
            },
            "hashed-token",
        )
        .await
        .unwrap();
    drop(store);
    let restarted = mw_store::Store::open(&db.to_string_lossy(), key)
        .await
        .unwrap();
    assert!(
        restarted
            .use_remembered_device("acct-1", "hashed-token", now + 60)
            .await
            .unwrap(),
        "the device cookie still matches its stored hash"
    );
    assert!(
        !restarted
            .use_remembered_device("acct-1", "hashed-token", now + 90_000)
            .await
            .unwrap(),
        "its own expiry is unchanged by the restart"
    );
}
