//! Production middleware and the authenticated process summary share counters.
use mw_server::{AppConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use serde_json::{Value, json};
mod common;

#[tokio::test]
async fn summary_requires_admin_and_counts_real_requests_without_paths() {
    let (app, _) = build_app_full(
        AppConfig {
            db_path: common::test_db::unique_db_path("http-metrics")
                .to_string_lossy()
                .into(),
            server_key_hex: None,
            web_dir: None,
            cookie_secure: false,
            mode: ServerMode::Proxy,
            hardening: Default::default(),
            security: SecurityConfig::default(),
        },
        V6Config {
            admin_enabled: true,
            admin_username: Some("root".into()),
            admin_password: Some("synthetic-test-password".into()),
            redis_url: None,
        },
    )
    .await
    .unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    let client = reqwest::Client::builder()
        .no_proxy()
        .cookie_store(true)
        .build()
        .unwrap();
    let path = format!("{base}/admin/metrics/summary");
    assert_eq!(client.get(&path).send().await.unwrap().status(), 401);
    assert_eq!(
        client
            .post(format!("{base}/admin/login"))
            .json(&json!({"username":"root","password":"synthetic-test-password"}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    for _ in 0..3 {
        assert_eq!(
            client
                .get(format!("{base}/healthz?secret=should-never-be-a-label"))
                .send()
                .await
                .unwrap()
                .status(),
            200
        );
    }
    let response = client.get(path).send().await.unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.headers()["cache-control"], "no-store");
    let data: Value = response.json().await.unwrap();
    assert_eq!(data["perProcess"], true);
    assert_eq!(data["series"].as_array().unwrap().len(), 60);
    let api = data["totals"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["class"] == "api")
        .unwrap();
    assert!(api["requests"].as_u64().unwrap() >= 3);
    let admin = data["totals"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["class"] == "admin")
        .unwrap();
    assert!(admin["errors4xx"].as_u64().unwrap() >= 1);
    assert!(!data.to_string().contains("should-never-be-a-label"));
    let endpoint = mw_server::admin_router::registry()
        .into_iter()
        .find(|endpoint| endpoint.suffix == "/metrics/summary")
        .unwrap();
    assert!(
        matches!(endpoint.access, mw_server::admin_router::Access::Cap(cap)
        if cap == mw_server::capabilities::Capability::ObservabilityRead && cap.unscoped_only())
    );
}
