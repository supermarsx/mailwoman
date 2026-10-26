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
    let first = client
        .post(format!("{base}/api/login"))
        .json(&json!({"username":mw_mock_jmap::USER,"password":mw_mock_jmap::PASS}))
        .send()
        .await
        .unwrap();
    assert_eq!(first.status(), 200);
    // Shared trusted peer key cannot be bypassed by choosing another surface or spoofing XFF.
    for path in ["/api/login", "/admin/login"] {
        let response = client
            .post(format!("{base}{path}"))
            .header("x-forwarded-for", "203.0.113.7")
            .json(&json!({"username":"root","password":"hunter2","native":true}))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), 429);
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
