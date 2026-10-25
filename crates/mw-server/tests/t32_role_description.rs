use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use serde_json::{Value, json};
mod common;

#[tokio::test]
async fn custom_role_descriptions_obey_existing_proofs_etags_and_builtin_protection() {
    let web = common::test_db::unique_dir("role-description-web");
    std::fs::write(web.join("index.html"), "test").unwrap();
    let (app, _) = build_app_full(
        AppConfig {
            db_path: common::test_db::unique_db_path("role-description")
                .to_string_lossy()
                .into(),
            server_key_hex: None,
            web_dir: Some(web),
            cookie_secure: false,
            mode: ServerMode::Proxy,
            hardening: HardeningConfig::default(),
            security: SecurityConfig::default(),
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
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .unwrap();
    });
    let client = reqwest::Client::builder()
        .no_proxy()
        .cookie_store(true)
        .build()
        .unwrap();
    assert_eq!(
        client
            .post(format!("{base}/admin/login"))
            .json(&json!({"username":"root","password":"hunter2"}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    let body = json!({"name":"Support purpose","description":"Read support details.","capabilities":["users.read"]});
    assert_eq!(
        client
            .post(format!("{base}/admin/roles"))
            .json(&body)
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    assert_eq!(
        client
            .post(format!("{base}/admin/step-up"))
            .json(&json!({"password":"hunter2"}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    let response = client
        .post(format!("{base}/admin/roles"))
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 201);
    let tag = response.headers()["etag"].to_str().unwrap().to_string();
    let created: Value = response.json().await.unwrap();
    assert_eq!(created["description"], "Read support details.");
    assert_eq!(created["capabilities"], json!(["users.read"]));
    let url = format!("{base}/admin/roles/{}", created["id"].as_str().unwrap());
    let mut update = body.clone();
    update["description"] = json!("Revised support purpose.");
    let response = client
        .put(&url)
        .header("if-match", &tag)
        .json(&update)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let new_tag = response.headers()["etag"].to_str().unwrap().to_string();
    assert_ne!(tag, new_tag);
    assert_eq!(
        client
            .put(&url)
            .header("if-match", &tag)
            .json(&body)
            .send()
            .await
            .unwrap()
            .status(),
        412
    );
    update["description"] = json!("x".repeat(1025));
    assert_eq!(
        client
            .put(&url)
            .header("if-match", &new_tag)
            .json(&update)
            .send()
            .await
            .unwrap()
            .status(),
        422
    );
    update.as_object_mut().unwrap().remove("description");
    let preserved: Value = client
        .put(&url)
        .header("if-match", &new_tag)
        .json(&update)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(preserved["description"], "Revised support purpose.");
    assert_eq!(
        client
            .put(format!("{base}/admin/roles/helpdesk"))
            .json(&body)
            .send()
            .await
            .unwrap()
            .status(),
        409
    );
}
