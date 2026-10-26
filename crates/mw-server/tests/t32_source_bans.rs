use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use serde_json::{Value,json};
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
async fn bans_apply_hot_to_sessions_but_not_health_and_ignore_spoofed_forwarding() {
    let upstream = serve(mw_mock_jmap::router()).await;
    let db = common::test_db::unique_db_path("notification-policy");
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
                ..Default::default()
            },
        },
        V6Config {
            admin_enabled: false,
            admin_username: None,
            admin_password: None,
            redis_url: None,
        },
    )
    .await
    .unwrap();
    let base = serve(router).await;
    let pool = sqlx::SqlitePool::connect(&format!("sqlite:{}", db.display()))
        .await
        .unwrap();
    let client = reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap();
    let login = client
        .post(format!("{base}/api/login"))
        .json(&json!({"username":mw_mock_jmap::USER,"password":mw_mock_jmap::PASS}))
        .send()
        .await
        .unwrap();
    assert_eq!(login.status(), 200);
    let _account = login.json::<Value>().await.unwrap()["accountId"]
        .as_str()
        .unwrap()
        .to_owned();

    let url = format!("{base}/api/account/notifications");
    assert_eq!(client.get(&url).send().await.unwrap().status(),200);
    let save = |entries: Value| {
        let pool=pool.clone();
        async move { sqlx::query("INSERT INTO settings(key,value) VALUES('v6:admin:bans',?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value")
            .bind(entries.to_string()).execute(&pool).await.unwrap(); }
    };
    let ban=|ip:&str,expiry:Value| json!({"ip":ip,"reason":"test","banned_at":"2026-01-01T00:00:00Z","expires_at":expiry});
    save(json!([ban("203.0.113.25",Value::Null)])).await;
    // An untrusted client cannot choose another person's source address.
    assert_eq!(client.get(&url).header("x-forwarded-for","203.0.113.25").send().await.unwrap().status(),200);
    // Legacy IPv4-mapped IPv6 spelling must match the canonical peer too.
    save(json!([ban("::ffff:127.0.0.1",Value::Null)])).await;
    for path in ["/api/account/notifications","/api/login","/admin/login","/"] {
        let response=client.get(format!("{base}{path}")).header("x-forwarded-for","203.0.113.26").send().await.unwrap();
        assert_eq!(response.status(),403,"{path}");
        assert_eq!(response.headers()["cache-control"],"no-store");
    }
    assert_eq!(client.get(format!("{base}/healthz")).send().await.unwrap().status(),200);
    save(json!([ban("127.0.0.1",json!("2000-01-01T00:00:00Z"))])).await;
    assert_eq!(client.get(&url).send().await.unwrap().status(),200);
    save(json!([ban("127.0.0.1",json!("invalid-legacy-expiry"))])).await;
    assert_eq!(client.get(&url).send().await.unwrap().status(),403);
    save(json!([])).await;
    assert_eq!(client.get(&url).send().await.unwrap().status(),200);
}
