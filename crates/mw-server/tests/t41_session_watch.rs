//! Session binding and theft detection over the real router (SQLite).
//!
//! * a browser session is bound to its client on first use; a different browser
//!   family is answered `403 step_up_required` (the default webmail action);
//! * the flag survives a restart (second app on the same database);
//! * a request with no User-Agent is never acted on;
//! * a bearer request and a sign-out are exempt;
//! * the revoke action deletes the session;
//! * a rotated session id presented again after the grace window revokes the
//!   family; inside the grace window it does not.
//!
//! Country-dependent rules (travel, concurrent use, country-change alert) need a
//! GeoIP database and are covered by the unit tests in `session_theft.rs`.
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use tokio::net::TcpListener;

use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};

mod common;
use common::test_db;

const KEY_HEX: &str = "0f1e2d3c4b5a69788796a5b4c3d2e1f00f1e2d3c4b5a69788796a5b4c3d2e1f0";
const CHROME: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36";
const CHROME_NEXT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36";
const FIREFOX: &str = "Mozilla/5.0 (X11; Linux x86_64; rv:126.0) Gecko/20100101 Firefox/126.0";

async fn spawn_mock() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let _ = axum::serve(listener, mw_mock_jmap::router()).await;
    });
    format!("http://{addr}")
}

async fn policy(db: &Path, key: &str, json_value: &str) {
    mw_store::Store::open(
        db.to_str().unwrap(),
        mw_store::ServerKey::from_hex(KEY_HEX).unwrap(),
    )
    .await
    .unwrap()
    .put_setting_policy(&mw_store::settings_policy::SettingPolicyRow {
        key: key.into(),
        global_value: Some(json_value.into()),
        allow_user_override: false,
        global_locked: false,
        updated_by: "test-fixture".into(),
        updated_at: 0,
    })
    .await
    .unwrap();
}

async fn spawn_server(dir: &Path, upstream: &str) -> String {
    let web: PathBuf = dir.join("web");
    std::fs::create_dir_all(&web).unwrap();
    std::fs::write(web.join("index.html"), "<!doctype html><title>MW</title>").unwrap();
    let config = AppConfig {
        db_path: dir.join("mw.db").to_string_lossy().into_owned(),
        server_key_hex: Some(KEY_HEX.to_string()),
        web_dir: Some(web),
        cookie_secure: false,
        mode: ServerMode::Proxy,
        hardening: HardeningConfig::default(),
        security: SecurityConfig {
            jmap_upstreams: Some(vec![upstream.to_string()]),
            ..SecurityConfig::default()
        },
    };
    let app = build_app_full(config, V6Config::default()).await.expect("boots").0;
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let _ = axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await;
    });
    format!("http://{addr}")
}

fn client() -> reqwest::Client {
    reqwest::Client::builder().no_proxy().redirect(reqwest::redirect::Policy::none()).build().unwrap()
}

async fn login(c: &reqwest::Client, base: &str, ua: &str) -> String {
    let r = c
        .post(format!("{base}/api/login"))
        .header("user-agent", ua)
        .json(&json!({"username": mw_mock_jmap::USER, "password": mw_mock_jmap::PASS}))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200, "login");
    r.headers()
        .get_all("set-cookie")
        .iter()
        .filter_map(|h| h.to_str().ok())
        .find_map(|h| h.strip_prefix("mw_session=").map(|v| v.split(';').next().unwrap().to_owned()))
        .expect("session cookie")
}

async fn me(c: &reqwest::Client, base: &str, session: &str, ua: Option<&str>) -> (u16, Value) {
    let mut req = c.get(format!("{base}/api/me")).header("cookie", format!("mw_session={session}"));
    if let Some(ua) = ua {
        req = req.header("user-agent", ua);
    }
    let r = req.send().await.unwrap();
    let status = r.status().as_u16();
    (status, r.json().await.unwrap_or(Value::Null))
}

#[tokio::test]
async fn binding_flags_a_different_browser_and_the_flag_survives_a_restart() {
    let upstream = spawn_mock().await;
    let dir = test_db::unique_dir("mw-t41-binding");
    let base = spawn_server(&dir, &upstream).await;
    policy(&dir.join("mw.db"), "signin.retryCooldownSecs", "0").await;
    let c = client();
    let session = login(&c, &base, CHROME).await;

    // First use binds the session; an auto-updated browser is the same client.
    assert_eq!(me(&c, &base, &session, Some(CHROME)).await.0, 200);
    assert_eq!(me(&c, &base, &session, Some(CHROME_NEXT)).await.0, 200);
    // No User-Agent at all is recorded, never acted on.
    assert_eq!(me(&c, &base, &session, None).await.0, 200);

    // A bearer-style request is exempt: it is refused by the session layer
    // (not a native session), never by the binding layer.
    let bearer = c
        .get(format!("{base}/api/me"))
        .header("authorization", "Bearer not-a-native-token")
        .header("cookie", format!("mw_session={session}"))
        .header("user-agent", FIREFOX)
        .send()
        .await
        .unwrap();
    assert_ne!(bearer.status().as_u16(), 403, "binding must not judge a bearer request");

    // A different browser family and OS: step-up required.
    let (status, body) = me(&c, &base, &session, Some(FIREFOX)).await;
    assert_eq!(status, 403, "{body}");
    assert_eq!(body["code"], "step_up_required");
    assert_eq!(body["action"], "sessionVerification");
    // The original browser is gated too until the holder confirms.
    assert_eq!(me(&c, &base, &session, Some(CHROME)).await.0, 403);
    // Sign-out stays reachable.
    let out = c
        .post(format!("{base}/api/logout"))
        .header("origin", &base)
        .header("cookie", format!("mw_session={session}"))
        .send()
        .await
        .unwrap();
    assert!(out.status().is_success() || out.status().as_u16() == 204, "{}", out.status());

    // A second session, flagged, then a restart on the same database.
    let second = login(&c, &base, CHROME).await;
    assert_eq!(me(&c, &base, &second, Some(CHROME)).await.0, 200);
    assert_eq!(me(&c, &base, &second, Some(FIREFOX)).await.0, 403);
    let restarted = spawn_server(&dir, &upstream).await;
    assert_eq!(
        me(&c, &restarted, &second, Some(CHROME)).await.0,
        403,
        "the flag is persisted, not held in memory"
    );
    // The user-visible notice was stored for the account.
    let notices = c
        .get(format!("{restarted}/api/account/security/session-alerts"))
        .header("cookie", format!("mw_session={second}"))
        .send()
        .await
        .unwrap();
    // Gated like everything else except the step-up and notice routes.
    assert_eq!(notices.status(), 200);
    let notices: Value = notices.json().await.unwrap();
    let list = notices["notices"].as_array().unwrap();
    assert!(
        list.iter().any(|n| n["kind"] == "fingerprintMismatch" && n["action"] == "step-up"),
        "{notices}"
    );
    let first = &list[0];
    assert!(first["session"].as_str().unwrap().len() == 16);
    assert!(!notices.to_string().contains(&second), "the cookie never appears in a notice");
}

#[tokio::test]
async fn revoke_action_deletes_the_session() {
    let upstream = spawn_mock().await;
    let dir = test_db::unique_dir("mw-t41-revoke");
    let db = dir.join("mw.db");
    // Create the database through a first boot, then set the policies.
    let _ = spawn_server(&dir, &upstream).await;
    policy(&db, "signin.retryCooldownSecs", "0").await;
    policy(&db, "security.sessionBinding.webmailAction", "\"revoke\"").await;
    let base = spawn_server(&dir, &upstream).await;
    let c = client();
    let session = login(&c, &base, CHROME).await;
    assert_eq!(me(&c, &base, &session, Some(CHROME)).await.0, 200);
    let (status, body) = me(&c, &base, &session, Some(FIREFOX)).await;
    assert_eq!((status, body["code"].as_str()), (401, Some("sessionRevoked")));
    assert_eq!(me(&c, &base, &session, Some(CHROME)).await.0, 401, "the session is gone");
}

#[tokio::test]
async fn reusing_a_rotated_session_id_revokes_the_family_but_not_inside_the_grace_window() {
    let upstream = spawn_mock().await;
    let dir = test_db::unique_dir("mw-t41-rotation");
    let db = dir.join("mw.db");
    let _ = spawn_server(&dir, &upstream).await;
    policy(&db, "signin.retryCooldownSecs", "0").await;
    policy(&db, "security.theft.rotationGraceSecs", "2").await;
    let base = spawn_server(&dir, &upstream).await;
    let c = client();
    let rotate = |session: String| {
        let c = c.clone();
        let base = base.clone();
        async move {
            let r = c
                .post(format!("{base}/api/session/rotate"))
                .header("origin", &base)
                .header("user-agent", CHROME)
                .header("cookie", format!("mw_session={session}"))
                .send()
                .await
                .unwrap();
            assert_eq!(r.status(), 200, "rotate");
            r.headers()
                .get_all("set-cookie")
                .iter()
                .filter_map(|h| h.to_str().ok())
                .find_map(|h| h.strip_prefix("mw_session=").map(|v| v.split(';').next().unwrap().to_owned()))
                .unwrap()
        }
    };

    let old = login(&c, &base, CHROME).await;
    assert_eq!(me(&c, &base, &old, Some(CHROME)).await.0, 200);
    let new = rotate(old.clone()).await;
    assert_ne!(old, new);
    assert_eq!(me(&c, &base, &new, Some(CHROME)).await.0, 200, "the new id inherits the binding");

    // Inside the grace window the retired id is just unauthenticated (an in-flight
    // request); it is not treated as theft and the new session lives.
    assert_eq!(me(&c, &base, &old, Some(CHROME)).await.0, 401);
    assert_eq!(me(&c, &base, &new, Some(CHROME)).await.0, 200);

    // After the window the same retired id ends the whole family.
    tokio::time::sleep(std::time::Duration::from_millis(3300)).await;
    let (status, body) = me(&c, &base, &old, Some(CHROME)).await;
    assert_eq!((status, body["code"].as_str()), (401, Some("sessionRevoked")), "{body}");
    assert_eq!(me(&c, &base, &new, Some(CHROME)).await.0, 401, "the rotated session is revoked with it");
}
