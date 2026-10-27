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
async fn saved_search_filters_reject_malformed_and_unsupported_documents() {
    let upstream = serve(mw_mock_jmap::router()).await;
    let db = common::test_db::unique_db_path("saved-search-validation");
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
                ..SecurityConfig::default()
            },
        },
        V6Config::default(),
    )
    .await
    .unwrap();
    let base = serve(router).await;
    let endpoint = format!("{base}/api/account/saved-searches");
    let client = reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap();
    assert_eq!(client.get(&endpoint).send().await.unwrap().status(), 401);
    let login = client
        .post(format!("{base}/api/login"))
        .json(&json!({"username":mw_mock_jmap::USER,"password":mw_mock_jmap::PASS}))
        .send()
        .await
        .unwrap();
    assert_eq!(login.status(), 200);
    // Personal-settings writes carry the login's storage authority (0a11b581).
    let authority = login.json::<Value>().await.unwrap()["storageAuthority"]
        .as_str()
        .expect("login names a storageAuthority")
        .to_owned();
    for query in [
        "null".to_string(),
        "broken".into(),
        r#"{"unread":true}"#.into(),
        r#"{"subject":42}"#.into(),
        r#"{"before":"yesterday"}"#.into(),
        json!({"text":"x".repeat(4097)}).to_string(),
    ] {
        let response = client
            .put(&endpoint)
            .header("x-mailwoman-storage-authority", &authority)
            .json(&json!({"name":"Rejected","queryJson":query,"asFolder":true}))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), 400, "{}", response.text().await.unwrap());
    }
    let rows = client
        .get(&endpoint)
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();
    assert_eq!(rows["savedSearches"], json!([]));
    let query = json!({"subject":"Invoice","hasAttachment":true,"after":"2026-01-01T00:00:00Z"})
        .to_string();
    let saved = client
        .put(&endpoint)
        .header("x-mailwoman-storage-authority", &authority)
        .json(&json!({"name":"Invoices","queryJson":query,"asFolder":true}))
        .send()
        .await
        .unwrap();
    assert_eq!(saved.status(), 200, "{}", saved.text().await.unwrap());
    let rows = client
        .get(&endpoint)
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();
    assert_eq!(rows["savedSearches"].as_array().unwrap().len(), 1);
    assert_eq!(rows["savedSearches"][0]["queryJson"], query);
}
