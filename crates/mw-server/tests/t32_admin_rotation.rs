//! Admin console session id rotation over the real router.
//!
//! * a successful step-up replaces the session id (the privilege elevation);
//! * the new cookie keeps HttpOnly, SameSite=Strict and the console path, and
//!   the idle and absolute clocks (and the creation time) are those of the old id;
//! * the step-up proof follows the session;
//! * the old id is unauthenticated inside the grace window, and presenting it
//!   after the window revokes the whole admin session family;
//! * `admin.session.rotateSecs` is a registered, bounded, hot key.
use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use reqwest::{Client, Response};
use serde_json::{Value, json};
use std::net::SocketAddr;
mod common;
use common::test_db;

const KEY: &str = "5a69788796a5b4c3d2e1f00f1e2d3c4b5a69788796a5b4c3d2e1f00f1e2d3c4b";
const UA: &str = "Mozilla/5.0 (X11; Linux x86_64; rv:126.0) Gecko/20100101 Firefox/126.0";

async fn put(store: &mw_store::Store, key: &str, value: &str) {
    store
        .put_setting_policy(&mw_store::settings_policy::SettingPolicyRow {
            key: key.into(),
            global_value: Some(value.into()),
            allow_user_override: false,
            global_locked: false,
            updated_by: "admin-rotation-fixture".into(),
            updated_at: 0,
        })
        .await
        .unwrap();
}

async fn server() -> (String, mw_store::Store) {
    let web = test_db::unique_dir("admin-rotation-web");
    std::fs::write(web.join("index.html"), "<!doctype html><title>Test</title>").unwrap();
    let db = test_db::unique_db_path("admin-rotation")
        .to_string_lossy()
        .into_owned();
    let store = mw_store::Store::open(&db, mw_store::ServerKey::from_hex(KEY).unwrap())
        .await
        .unwrap();
    put(&store, "signin.retryCooldownSecs", "0").await;
    put(&store, "security.theft.rotationGraceSecs", "2").await;
    let (app, _) = build_app_full(
        AppConfig {
            db_path: db,
            server_key_hex: Some(KEY.into()),
            web_dir: Some(web),
            cookie_secure: false,
            mode: ServerMode::Proxy,
            hardening: HardeningConfig::default(),
            security: SecurityConfig::default(),
        },
        V6Config {
            admin_enabled: true,
            admin_username: Some("root".into()),
            admin_password: Some("hunter2".into()),
            redis_url: None,
        },
    )
    .await
    .unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });
    (format!("http://{address}"), store)
}

fn client() -> Client {
    Client::builder().no_proxy().build().unwrap()
}
fn cookie_values(response: &Response, name: &str) -> Vec<String> {
    response
        .headers()
        .get_all("set-cookie")
        .iter()
        .filter_map(|v| v.to_str().ok())
        .filter(|v| v.starts_with(&format!("{name}=")))
        .map(str::to_owned)
        .collect()
}
fn token_of(set_cookie: &str) -> String {
    set_cookie
        .split(';')
        .next()
        .unwrap()
        .split_once('=')
        .unwrap()
        .1
        .to_owned()
}
async fn get(c: &Client, base: &str, path: &str, token: &str) -> Response {
    c.get(format!("{base}/admin{path}"))
        .header("user-agent", UA)
        .header("cookie", format!("mw_admin_session={token}"))
        .send()
        .await
        .unwrap()
}
async fn step_up(c: &Client, base: &str, token: &str) -> Response {
    c.post(format!("{base}/admin/step-up"))
        .header("user-agent", UA)
        .header("origin", base)
        .header("cookie", format!("mw_admin_session={token}"))
        .json(&json!({"password": "hunter2"}))
        .send()
        .await
        .unwrap()
}

#[tokio::test]
async fn step_up_rotates_the_admin_session_id_and_keeps_its_clocks() {
    let (base, store) = server().await;
    let c = client();
    let login = c
        .post(format!("{base}/admin/login"))
        .header("user-agent", UA)
        .json(&json!({"username":"root","password":"hunter2"}))
        .send()
        .await
        .unwrap();
    assert_eq!(login.status(), 200);
    let first = cookie_values(&login, "mw_admin_session");
    assert_eq!(first.len(), 1);
    let a = token_of(&first[0]);
    assert_eq!(get(&c, &base, "/session", &a).await.status(), 200);
    let before = store.list_admin_sessions(0).await.unwrap();
    assert_eq!(before.len(), 1);

    // Step-up: the id changes.
    let stepped = step_up(&c, &base, &a).await;
    assert_eq!(stepped.status(), 200);
    let rotated = cookie_values(&stepped, "mw_admin_session");
    assert_eq!(rotated.len(), 1, "one new admin cookie: {:?}", stepped.headers());
    let b = token_of(&rotated[0]);
    assert_ne!(a, b);
    // Same flags as the sign-in cookie.
    for flag in ["HttpOnly", "SameSite=Strict", "Path=/admin", "Max-Age="] {
        assert!(rotated[0].contains(flag), "{flag} in {}", rotated[0]);
    }
    assert!(!rotated[0].contains("Secure"), "cookie_secure is off in this fixture");
    assert_eq!(cookie_values(&stepped, "mw_csrf").len(), 1, "the CSRF cookie follows the id");

    // Clocks: one row, same creation time and absolute deadline, never extended.
    let after = store.list_admin_sessions(0).await.unwrap();
    assert_eq!(after.len(), 1, "the old id is gone");
    assert_ne!(after[0].token_hash, before[0].token_hash);
    assert_eq!(after[0].created_at, before[0].created_at);
    assert_eq!(after[0].absolute_expires_at, before[0].absolute_expires_at);
    assert!(after[0].expires_at <= after[0].absolute_expires_at);

    // The new id works and carries the step-up proof; the old id does not work.
    let status: Value = get(&c, &base, "/step-up", &b).await.json().await.unwrap();
    assert_eq!(status["active"], true, "the proof followed the session: {status}");
    assert_eq!(get(&c, &base, "/session", &a).await.status(), 401, "inside the grace window: unauthenticated");
    assert_eq!(get(&c, &base, "/session", &b).await.status(), 200, "and not theft yet");

    // Reuse after the grace window ends the whole family and raises an alert.
    tokio::time::sleep(std::time::Duration::from_millis(3300)).await;
    let reused = get(&c, &base, "/session", &a).await;
    assert_eq!(reused.status(), 401);
    let body: Value = reused.json().await.unwrap();
    assert_eq!(body["code"], "sessionRevoked", "{body}");
    assert_eq!(get(&c, &base, "/session", &b).await.status(), 401, "the rotated session is revoked with it");
    assert!(store.list_admin_sessions(0).await.unwrap().is_empty());
    let ring = store.get_setting("sec-alerts:v1:recent").await.unwrap().unwrap_or_default();
    assert!(ring.contains("rotationReuse"), "an admin alert was recorded: {ring}");
}

#[tokio::test]
async fn ordinary_requests_do_not_rotate_before_the_interval() {
    let (base, store) = server().await;
    let c = client();
    let login = c
        .post(format!("{base}/admin/login"))
        .header("user-agent", UA)
        .json(&json!({"username":"root","password":"hunter2"}))
        .send()
        .await
        .unwrap();
    let a = token_of(&cookie_values(&login, "mw_admin_session")[0]);
    for _ in 0..3 {
        let r = get(&c, &base, "/session", &a).await;
        assert_eq!(r.status(), 200);
        assert!(cookie_values(&r, "mw_admin_session").is_empty());
    }
    assert_eq!(store.list_admin_sessions(0).await.unwrap().len(), 1);
}

#[test]
fn the_interval_is_a_bounded_hot_global_key() {
    let def = mw_server::settings_registry::lookup("admin.session.rotateSecs").unwrap();
    assert_eq!(def.default_value(), json!(900));
    assert!(mw_server::settings_registry::validate(def, &json!(86_401), "/v").is_err());
    assert!(mw_server::settings_registry::validate(def, &json!(-1), "/v").is_err());
    assert!(mw_server::settings_registry::validate(def, &json!(0), "/v").is_ok());
}
