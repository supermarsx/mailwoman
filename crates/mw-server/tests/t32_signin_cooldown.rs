use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use serde_json::{Value, json};
use std::net::SocketAddr;
mod common;
async fn serve(router: axum::Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(
            listener,
            router.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });
    format!("http://{address}")
}
#[tokio::test]
async fn trusted_peer_cooldown_is_shared_and_hot_configurable() {
    let upstream = serve(mw_mock_jmap::router()).await;
    let db = common::test_db::unique_db_path("signin-cooldown");
    let (router, _) = build_app_full(
        AppConfig {
            db_path: db.to_string_lossy().into(),
            server_key_hex: None,
            web_dir: None,
            cookie_secure: false,
            mode: ServerMode::Proxy,
            hardening: HardeningConfig {
                csrf_strict: true,
                ..HardeningConfig::default()
            },
            security: SecurityConfig {
                upstream_default: Some(upstream.clone()),
                jmap_upstreams: Some(vec![upstream]),
                ..SecurityConfig::default()
            },
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
    let base = serve(router).await;
    let pool = sqlx::SqlitePool::connect(&format!("sqlite:{}", db.display()))
        .await
        .unwrap();
    let client = reqwest::Client::new();
    let config: Value = client
        .get(format!("{base}/api/public-config"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(config["retryCooldownSecs"], 3);
    // t33 D1: a sign-in that issues a session gives its charge back, so a DIFFERENT
    // user behind the same address (here: the admin, on the other surface, which
    // shares the per-network key) is not made to wait.
    let first = client
        .post(format!("{base}/api/login"))
        .json(&json!({"username":mw_mock_jmap::USER,"password":mw_mock_jmap::PASS}))
        .send()
        .await
        .unwrap();
    assert_eq!(first.status(), 200);
    let other_user = client
        .post(format!("{base}/admin/login"))
        .json(&json!({"username":"root","password":"hunter2"}))
        .send()
        .await
        .unwrap();
    assert_eq!(
        other_user.status(),
        200,
        "a success must not start the wait for the next user on the same address"
    );
    let again = client
        .post(format!("{base}/api/login"))
        .json(&json!({"username":mw_mock_jmap::USER,"password":mw_mock_jmap::PASS}))
        .send()
        .await
        .unwrap();
    assert_eq!(again.status(), 200, "nor on the mailbox surface");
    // A native (bearer-token) sign-in issues its session without cookies; it is a
    // success too, so the next sign-in from the address is not held.
    let native = client
        .post(format!("{base}/api/login"))
        .json(&json!({"username":mw_mock_jmap::USER,"password":mw_mock_jmap::PASS,"clientType":"native"}))
        .send()
        .await
        .unwrap();
    assert_eq!(native.status(), 200);
    let native: Value = native.json().await.unwrap();
    assert!(
        native["token"].is_string(),
        "native login issues a token: {native}"
    );
    let after_native = client
        .post(format!("{base}/admin/login"))
        .json(&json!({"username":"root","password":"hunter2"}))
        .send()
        .await
        .unwrap();
    assert_eq!(
        after_native.status(),
        200,
        "a native success must not start the wait either"
    );

    // A FAILED attempt starts the wait, exactly as every attempt did before D1.
    let failed = client
        .post(format!("{base}/api/login"))
        .json(&json!({"username":mw_mock_jmap::USER,"password":"wrong"}))
        .send()
        .await
        .unwrap();
    assert_eq!(failed.status(), 401);
    // Shared trusted peer key cannot be bypassed by choosing another surface, another
    // user, the right password, or spoofing XFF.
    for (path, username, password) in [
        ("/api/login", mw_mock_jmap::USER, mw_mock_jmap::PASS),
        ("/admin/login", "root", "hunter2"),
        ("/api/login", mw_mock_jmap::USER, "wrong-again"),
    ] {
        let response = client
            .post(format!("{base}{path}"))
            .header("x-forwarded-for", "203.0.113.7")
            .json(&json!({"username":username,"password":password,"clientType":"native"}))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), 429, "{path} as {username}");
        assert_eq!(response.headers()["cache-control"], "no-store");
        let retry = response.headers()["retry-after"]
            .to_str()
            .unwrap()
            .parse::<u64>()
            .unwrap();
        assert!((1..=3).contains(&retry));
        let body: Value = response.json().await.unwrap();
        assert_eq!(body["code"], "signinCooldown");
        assert_eq!(body["retryAfterSecs"], retry);
    }
    sqlx::query("INSERT INTO setting_policy(key,global_value,updated_by,updated_at) VALUES('signin.retryCooldownSecs','0','test',0)").execute(&pool).await.unwrap();
    for _ in 0..2 {
        let response = client
            .post(format!("{base}/admin/login"))
            .json(&json!({"username":"root","password":"hunter2"}))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), 200);
    }
}

/// t33 D1, the admin surface's own failure: a wrong admin password starts the wait
/// for the whole address, and the admin's correct password inside the window is
/// still refused (the success only counts once it is admitted).
#[tokio::test]
async fn a_failed_admin_attempt_paces_the_next_attempt_from_that_address() {
    let upstream = serve(mw_mock_jmap::router()).await;
    let db = common::test_db::unique_db_path("signin-cooldown-admin-fail");
    let (router, _) = build_app_full(
        AppConfig {
            db_path: db.to_string_lossy().into(),
            server_key_hex: None,
            web_dir: None,
            cookie_secure: false,
            mode: ServerMode::Proxy,
            hardening: HardeningConfig::default(),
            security: SecurityConfig {
                upstream_default: Some(upstream.clone()),
                jmap_upstreams: Some(vec![upstream]),
                ..SecurityConfig::default()
            },
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
    let base = serve(router).await;
    let client = reqwest::Client::new();
    let failed = client
        .post(format!("{base}/admin/login"))
        .json(&json!({"username":"root","password":"not-it"}))
        .send()
        .await
        .unwrap();
    assert_eq!(failed.status(), 401);
    for (path, username, password) in [
        ("/admin/login", "root", "hunter2"),
        ("/api/login", mw_mock_jmap::USER, mw_mock_jmap::PASS),
    ] {
        let response = client
            .post(format!("{base}{path}"))
            .json(&json!({"username":username,"password":password}))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), 429, "{path} inside the window");
        let body: Value = response.json().await.unwrap();
        assert_eq!(body["code"], "signinCooldown");
    }
}
