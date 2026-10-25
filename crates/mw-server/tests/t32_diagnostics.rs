//! Diagnostics execute real bounded probes, persist history and require admin grants.
use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use serde_json::{Value, json};

#[tokio::test]
async fn server_suite_is_authenticated_redacted_persisted_and_rate_limited() {
    let dir = std::env::temp_dir().join(format!("mw-diagnostics-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    let secret = "diagnostic-admin-password";
    let app = build_app_full(
        AppConfig {
            db_path: dir.join("diag.db").to_string_lossy().into_owned(),
            server_key_hex: Some("12".repeat(32)),
            web_dir: None,
            cookie_secure: false,
            mode: ServerMode::Proxy,
            hardening: HardeningConfig::default(),
            security: SecurityConfig::default(),
        },
        V6Config {
            admin_enabled: true,
            admin_username: Some("root".into()),
            admin_password: Some(secret.into()),
            redis_url: None,
        },
    )
    .await
    .unwrap()
    .0;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let client = reqwest::Client::builder()
        .cookie_store(true)
        .no_proxy()
        .build()
        .unwrap();
    assert_eq!(
        client
            .get(format!("{base}/admin/diag/runs"))
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    assert_eq!(
        client
            .post(format!("{base}/admin/login"))
            .json(&json!({"username":"root","password":secret}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    assert_eq!(
        client
            .post(format!("{base}/admin/diag/run"))
            .json(&json!({"suite":"unimplemented"}))
            .send()
            .await
            .unwrap()
            .status(),
        422
    );
    let response = client
        .post(format!("{base}/admin/diag/run"))
        .json(&json!({"suite":"server"}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.headers()["cache-control"], "private, no-store");
    let text = response.text().await.unwrap();
    assert!(!text.contains(secret));
    assert!(!text.contains(&"12".repeat(32)));
    let report: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(report["run"]["status"], "pass", "{report}");
    assert_eq!(report["run"]["results"].as_array().unwrap().len(), 5);
    assert_eq!(report["historySaved"], true);
    assert_eq!(report["auditSaved"], true);
    let history: Value = client
        .get(format!("{base}/admin/diag/runs"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(history["runs"][0]["id"], report["run"]["id"]);
    assert_eq!(
        client
            .post(format!("{base}/admin/diag/run"))
            .json(&json!({"suite":"server"}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    assert_eq!(
        client
            .post(format!("{base}/admin/diag/run"))
            .json(&json!({"suite":"server"}))
            .send()
            .await
            .unwrap()
            .status(),
        429
    );
    task.abort();
}
