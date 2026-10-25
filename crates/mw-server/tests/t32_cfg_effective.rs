//! Production-router catalogue acceptance; no live upstream is contacted.
use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use serde_json::{Value, json};

#[tokio::test]
async fn effective_catalogue_uses_runtime_state_and_redacts_nested_urls() {
    let dir = std::env::temp_dir().join(format!("mw-config-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    let app = build_app_full(AppConfig {
        db_path:dir.join("config.db").to_string_lossy().into_owned(),
        server_key_hex:Some("11".repeat(32)), web_dir:None, cookie_secure:false,
        mode:ServerMode::Proxy, hardening:HardeningConfig::default(),
        security:SecurityConfig {
            jmap_upstreams:Some(vec!["https://alice:secret-password@example.org/jmap?token=secret-query#secret-fragment".into()]),
            upstream_default:Some("https://alice:secret-password@example.org/jmap?token=secret-query".into()),
            ..Default::default()
        },
    },V6Config{admin_enabled:true,admin_username:Some("root".into()),admin_password:Some("test-password".into()),redis_url:None}).await.unwrap().0;
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
            .get(format!("{base}/admin/config/effective"))
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    assert_eq!(
        client
            .post(format!("{base}/admin/login"))
            .json(&json!({"username":"root","password":"test-password"}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    let response = client
        .get(format!("{base}/admin/config/effective"))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(
        response.headers().get("cache-control").unwrap(),
        "private, no-store"
    );
    let etag = response
        .headers()
        .get("etag")
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();
    let body = response.text().await.unwrap();
    for secret in ["secret-password", "secret-query", "secret-fragment"] {
        assert!(!body.contains(secret), "{secret} leaked");
    }
    let rows: Vec<Value> = serde_json::from_str(&body).unwrap();
    let find = |id: &str| rows.iter().find(|r| r["id"] == id).unwrap();
    assert_eq!(find("MW_MODE")["value"], "proxy");
    assert_eq!(
        find("MW_JMAP_UPSTREAMS")["value"],
        json!(["https://example.org/jmap"])
    );
    assert_eq!(find("MW_DB_PATH")["valueStatus"], "resolved");
    assert_eq!(
        find("MW_DB_PATH")["value"],
        dir.join("config.db").to_string_lossy().as_ref()
    );
    assert_eq!(find("MW_SERVER_KEY")["value"], json!({"set":true}));
    assert_eq!(find("MW_SERVER_KEY")["persistent"], true);
    assert_eq!(find("ui.language")["valueStatus"], "resolvedPolicy");
    let again = client
        .get(format!("{base}/admin/config/effective"))
        .send()
        .await
        .unwrap();
    assert_eq!(again.headers().get("etag").unwrap(), etag.as_str());
    let setting = client
        .get(format!("{base}/admin/settings/ui.language"))
        .send()
        .await
        .unwrap();
    assert_eq!(setting.status(), 200);
    let setting_etag = setting.headers().get("etag").unwrap().clone();
    let changed = client
        .put(format!("{base}/admin/settings/ui.language"))
        .header("If-Match", setting_etag)
        .json(&json!({"globalValue":"fr"}))
        .send()
        .await
        .unwrap();
    assert_eq!(changed.status(), 200, "{}", changed.text().await.unwrap());
    let updated = client
        .get(format!("{base}/admin/config/effective"))
        .send()
        .await
        .unwrap();
    assert_ne!(updated.headers().get("etag").unwrap(), etag.as_str());
    let updated: Vec<Value> = updated.json().await.unwrap();
    let language = updated.iter().find(|r| r["id"] == "ui.language").unwrap();
    assert_eq!(language["value"], "fr");
    assert_eq!(language["source"], "stored");
    task.abort();
}
