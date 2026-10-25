//! Diagnostics execute real bounded probes, persist history and require admin grants.
use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use serde_json::{Value, json};

#[tokio::test]
async fn authentication_history_is_privileged_filtered_and_redacted() {
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
            .get(format!("{base}/admin/logs/auth"))
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    for _ in 0..2 {
        assert_eq!(
            client
                .post(format!("{base}/admin/login"))
                .json(
                    &json!({"username":"unknown@example.org","password":"never-log-this-password"})
                )
                .send()
                .await
                .unwrap()
                .status(),
            401
        );
    }
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
    let response = client
        .get(format!(
            "{base}/admin/logs/auth?q=unknown&surface=admin&result=failure"
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.headers()["cache-control"], "private, no-store");
    let text = response.text().await.unwrap();
    assert!(!text.contains("never-log-this-password"));
    assert!(!text.contains(secret));
    let value: Value = serde_json::from_str(&text).unwrap();
    let events = value["events"].as_array().unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["repeatCount"], 2);
    assert_eq!(events[0]["backend"], "admin-local");
    assert_eq!(events[0]["method"], "password");
    assert!(events[0]["accountId"].is_null());
    for query in [
        "limit=0",
        "limit=101",
        "surface=invalid",
        "result=invalid",
        "cursor=invalid",
    ] {
        assert_eq!(
            client
                .get(format!("{base}/admin/logs/auth?{query}"))
                .send()
                .await
                .unwrap()
                .status(),
            422,
            "{query}"
        );
    }
    let value: Value = client
        .get(format!("{base}/admin/logs/auth?limit=1"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(value["cursor"].is_string());
    let next: Value = client
        .get(format!("{base}/admin/logs/auth"))
        .query(&[("cursor", value["cursor"].as_str().unwrap())])
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(!next["events"].as_array().unwrap().is_empty());
    assert_eq!(
        client
            .post(format!("{base}/admin/step-up"))
            .json(&json!({"password":secret}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    let role: Value = client
        .post(format!("{base}/admin/roles"))
        .json(&json!({"name":"Scoped log reader","capabilities":["logs.read"]}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    for (name, roles) in [
        (
            "scoped",
            json!([{"roleId":role["id"],"domains":["example.org"]}]),
        ),
        ("no-grants", json!([])),
    ] {
        assert_eq!(
            client
                .post(format!("{base}/admin/admins"))
                .json(&json!({"username":name,"password":"long test password","roles":roles}))
                .send()
                .await
                .unwrap()
                .status(),
            201
        );
        let actor = reqwest::Client::builder()
            .cookie_store(true)
            .no_proxy()
            .build()
            .unwrap();
        assert_eq!(
            actor
                .post(format!("{base}/admin/login"))
                .json(&json!({"username":name,"password":"long test password"}))
                .send()
                .await
                .unwrap()
                .status(),
            200
        );
        assert_eq!(
            actor
                .get(format!("{base}/admin/logs/application"))
                .send()
                .await
                .unwrap()
                .status(),
            403,
            "{name}"
        );
        assert_eq!(
            actor
                .get(format!("{base}/admin/logs/auth"))
                .send()
                .await
                .unwrap()
                .status(),
            403,
            "{name}"
        );
    }
    let operational = client
        .get(format!("{base}/admin/logs/application"))
        .send()
        .await
        .unwrap();
    assert_eq!(operational.status(), 200);
    let operational: Value = operational.json().await.unwrap();
    assert_eq!(operational["processLocal"], true);
    assert_eq!(
        reqwest::Client::new()
            .get(format!("{base}/admin/logs/application"))
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    task.abort();
}
