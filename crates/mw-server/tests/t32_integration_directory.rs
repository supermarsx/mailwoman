//! Real admin login, sealed LDAP credentials and configuration concurrency.
use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use serde_json::{Value, json};
#[tokio::test]
async fn directory_configuration_is_redacted_concurrent_and_restart_aware() {
    let dir = std::env::temp_dir().join(format!("mw-directory-admin-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("directory.db").to_string_lossy().into_owned();
    let key = "11".repeat(32);
    let config = AppConfig {
        db_path: path.clone(),
        server_key_hex: Some(key),
        web_dir: None,
        cookie_secure: false,
        mode: ServerMode::Proxy,
        hardening: HardeningConfig::default(),
        security: SecurityConfig::default(),
    };
    let v6 = V6Config {
        admin_enabled: true,
        admin_username: Some("root".into()),
        admin_password: Some("test-password".into()),
        redis_url: None,
    };
    let app = build_app_full(config, v6).await.unwrap().0;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let client = reqwest::Client::builder()
        .cookie_store(true)
        .no_proxy()
        .build()
        .unwrap();
    let endpoint = format!("{base}/admin/integrations/directory");
    assert_eq!(client.get(&endpoint).send().await.unwrap().status(), 401);
    assert_eq!(
        client
            .put(&endpoint)
            .json(&json!({"entries":[]}))
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
    let first: Value = client
        .get(&endpoint)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(first["restartRequired"], false);
    let entry = json!({"id":"primary","url":"ldaps://directory.example","baseDn":"dc=example","bindDn":"cn=service","tls":"ldaps","priority":0,"attrMap":{},"enabled":true,"connectionMode":"direct","allowPrivate":false,"password":{"action":"set","value":"unique-ldap-secret"}});
    let missing = client
        .put(&endpoint)
        .json(&json!({"entries":[entry.clone()]}))
        .send()
        .await
        .unwrap();
    assert_eq!(missing.status(), 428);
    let saved = client
        .put(&endpoint)
        .header("if-match", first["etag"].as_str().unwrap())
        .json(&json!({"entries":[entry.clone()]}))
        .send()
        .await
        .unwrap();
    assert_eq!(saved.status(), 200, "{}", saved.text().await.unwrap());
    let response = client.get(&endpoint).send().await.unwrap();
    assert_eq!(response.status(), 200);
    let text = response.text().await.unwrap();
    assert!(!text.contains("unique-ldap-secret"));
    let saved: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(saved["entries"][0]["password"], json!({"set":true}));
    assert_eq!(saved["restartRequired"], true);
    assert_eq!(
        client
            .put(&endpoint)
            .header("if-match", first["etag"].as_str().unwrap())
            .json(&json!({"entries":[]}))
            .send()
            .await
            .unwrap()
            .status(),
        412
    );
    let mut retarget = entry.clone();
    retarget["url"] = json!("ldaps://attacker.example");
    retarget["password"] = json!({"action":"keep"});
    assert_eq!(
        client
            .put(&endpoint)
            .header("if-match", saved["etag"].as_str().unwrap())
            .json(&json!({"entries":[retarget]}))
            .send()
            .await
            .unwrap()
            .status(),
        422
    );
    let mut clear = entry;
    clear["password"] = json!({"action":"clear"});
    assert_eq!(
        client
            .put(&endpoint)
            .header("if-match", saved["etag"].as_str().unwrap())
            .json(&json!({"entries":[clear]}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    let cleared: Value = client
        .get(&endpoint)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(cleared["entries"][0]["password"]["set"], false);
    assert_eq!(
        client
            .put(&endpoint)
            .header("if-match", cleared["etag"].as_str().unwrap())
            .json(&json!({"entries":[]}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    task.abort();
}
