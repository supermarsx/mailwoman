use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
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
async fn setting(pool: &sqlx::SqlitePool, key: &str, value: Value) {
    sqlx::query("INSERT INTO setting_policy(key,global_value,updated_by,updated_at) VALUES(?1,?2,'test',0) ON CONFLICT(key) DO UPDATE SET global_value=excluded.global_value").bind(key).bind(value.to_string()).execute(pool).await.unwrap();
}
#[tokio::test]
async fn notification_policy_and_push_ownership_are_enforced_hot() {
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
    let logged_in=login.json::<Value>().await.unwrap();
    let authority=logged_in["storageAuthority"].as_str().unwrap();
    let config_url = format!("{base}/api/account/notifications");
    let config: Value = client
        .get(&config_url)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(config["enabled"], false);
    assert_eq!(config["persistentStorageAllowed"], false);
    let endpoint = "https://push.example/owned";
    let subscribe = json!({"transport":"unifiedpush","endpoint":endpoint});
    assert_eq!(
        client
            .post(format!("{base}/api/push/subscribe"))
            .header("x-mailwoman-storage-authority",authority).json(&subscribe)
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    let mut config = json!({"enabled":true,"rules":[],"quietHours":{"enabled":false,"start":"22:00","end":"07:00","timezone":"UTC"}});
    assert_eq!(
        client
            .put(&config_url)
            .header("x-mailwoman-storage-authority",authority).json(&config)
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    let first: Value = client
        .post(format!("{base}/api/push/subscribe"))
        .header("x-mailwoman-storage-authority",authority).json(&subscribe)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(first["id"].is_string());
    let owner:String=sqlx::query_scalar("SELECT owner_subject FROM personal_push_subscription WHERE id=?1").bind(first["id"].as_str().unwrap()).fetch_one(&pool).await.unwrap();
    let again: Value = client
        .post(format!("{base}/api/push/subscribe"))
        .header("x-mailwoman-storage-authority",authority).json(&subscribe)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(again["id"], first["id"]);
    let hash = Sha256::digest(endpoint.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let permission = format!("{base}/api/push/permission?endpointHash={hash}");
    let allowed = client.get(&permission).send().await.unwrap();
    assert_eq!(allowed.headers()["cache-control"], "private, no-store");
    let allowed = allowed.json::<Value>().await.unwrap();
    assert_eq!(allowed["allowed"], true);
    assert_eq!(allowed["registered"], true);
    sqlx::query("INSERT INTO personal_push_subscription(id,owner_subject,transport,endpoint,created_at) VALUES('foreign','another','unifiedpush','https://push.example/foreign','2026-01-01T00:00:00Z')").execute(&pool).await.unwrap();
    for body in [
        json!({"id":"foreign"}),
        json!({"endpoint":"https://push.example/foreign"}),
    ] {
        assert_eq!(
            client
                .post(format!("{base}/api/push/unsubscribe"))
                .header("x-mailwoman-storage-authority",authority).json(&body)
                .send()
                .await
                .unwrap()
                .status(),
            204
        );
    }
    let count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM personal_push_subscription WHERE id='foreign'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(count, 1);
    let foreign_hash = Sha256::digest(b"https://push.example/foreign")
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let foreign = client
        .get(format!("{base}/api/push/permission?endpointHash={foreign_hash}"))
        .send().await.unwrap().json::<Value>().await.unwrap();
    assert_eq!(foreign["allowed"], false);
    assert_eq!(foreign["registered"], false);

    assert_eq!(
        client
            .post(format!("{base}/api/push/subscribe"))
            .header("x-mailwoman-storage-authority",authority).json(&json!({"transport":"unifiedpush","endpoint":"https://push.example/foreign"}))
            .send()
            .await
            .unwrap()
            .status(),
        409
    );
    setting(&pool, "notifications.pushAllowed", json!(false)).await;
    assert_eq!(
        client
            .get(&permission)
            .send()
            .await
            .unwrap()
            .json::<Value>()
            .await
            .unwrap()["allowed"],
        false
    );
    assert_eq!(
        client
            .post(format!("{base}/api/push/subscribe"))
            .header("x-mailwoman-storage-authority",authority).json(&subscribe)
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    setting(&pool, "notifications.pushAllowed", json!(true)).await;
    config["rules"] = json!([{"id":"mute","label":"Private","match":"private","action":"mute"}]);
    assert_eq!(
        client
            .put(&config_url)
            .header("x-mailwoman-storage-authority",authority).json(&config)
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    assert_eq!(
        client
            .get(&permission)
            .send()
            .await
            .unwrap()
            .json::<Value>()
            .await
            .unwrap()["allowed"],
        false
    );
    config["quietHours"]["start"] = json!("é:00");
    assert_eq!(
        client
            .put(&config_url)
            .header("x-mailwoman-storage-authority",authority).json(&config)
            .send()
            .await
            .unwrap()
            .status(),
        400
    );
    assert_eq!(
        client
            .post(format!("{base}/api/push/subscribe"))
            .header("x-mailwoman-storage-authority",authority).json(&json!({"transport":"unifiedpush","endpoint":"https://127.0.0.1/private"}))
            .send()
            .await
            .unwrap()
            .status(),
        400
    );
    assert_eq!(
        client
            .post(format!("{base}/api/push/unsubscribe"))
            .header("x-mailwoman-storage-authority",authority).json(&json!({"id":first["id"]}))
            .send()
            .await
            .unwrap()
            .status(),
        204
    );
    let count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM personal_push_subscription WHERE owner_subject=?1")
            .bind(&owner)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(count, 0);
}
