use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use serde_json::{Value, json};
mod common;
#[tokio::test]
async fn settings_transfer_previews_enforces_revision_and_preserves_protected_settings() {
    let db = common::test_db::unique_db_path("configuration-transfer");
    let (router, _) = build_app_full(
        AppConfig {
            db_path: db.to_string_lossy().into(),
            server_key_hex: Some(
                "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff".into(),
            ),
            web_dir: None,
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
            router.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .unwrap();
    });
    let client = reqwest::Client::builder()
        .no_proxy()
        .cookie_store(true)
        .build()
        .unwrap();
    let endpoint = format!("{base}/admin/config/import");
    assert_eq!(
        client
            .get(format!("{base}/admin/config/export"))
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
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
    let bundle = json!({"format":"mailwoman-settings","version":1,"entries":[{"key":"mail.quickReply","globalValue":true,"locked":false,"allowUserOverride":true}]});
    let response = client
        .post(&endpoint)
        .json(&json!({"bundle":bundle}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let preview: Value = response.json().await.unwrap();
    assert_eq!(preview["preview"], true);
    let before: Value = client
        .get(format!("{base}/admin/config/export"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(
        !before["bundle"]["entries"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["key"] == "mail.quickReply")
    );
    let apply = json!({"bundle":bundle,"apply":true,"revisions":preview["revisions"]});
    assert_eq!(
        client
            .post(&endpoint)
            .json(&apply)
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
    let result = client.post(&endpoint).json(&apply).send().await.unwrap();
    assert_eq!(result.status(), 200);
    assert_eq!(result.json::<Value>().await.unwrap()["complete"], true);
    assert_eq!(
        client
            .post(&endpoint)
            .json(&apply)
            .send()
            .await
            .unwrap()
            .status(),
        412
    );
    let exported: Value = client
        .get(format!("{base}/admin/config/export"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(exported["bundle"]["entries"][0]["globalValue"], true);
    assert_eq!(exported["includesSecrets"], false);
    let mut forbidden = bundle.clone();
    forbidden["entries"][0]["key"] = json!("admin.allowLastSuperadminLoss");
    assert_eq!(
        client
            .post(&endpoint)
            .json(&json!({"bundle":forbidden}))
            .send()
            .await
            .unwrap()
            .status(),
        422
    );
    let mut invalid = bundle.clone();
    invalid["entries"][0]["globalValue"] = json!("invalid boolean");
    assert_eq!(
        client
            .post(&endpoint)
            .json(&json!({"bundle":invalid}))
            .send()
            .await
            .unwrap()
            .status(),
        422
    );
}
