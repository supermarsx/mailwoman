//! The trusted-header override belongs only to sessions issued by that path.
use mw_server::{AppConfig, SecurityConfig, ServerMode, build_app};
use mw_store::{Credentials, ServerKey, Store};
mod common;

#[tokio::test]
async fn explicit_header_authority_survives_restart_without_lending_access() {
    const KEY: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
    const URL: &str = "https://header.example/jmap";
    // This integration binary has one test; no concurrent environment readers.
    unsafe {
        std::env::set_var("MW_HEADER_AUTH", "1");
        std::env::set_var("MW_HEADER_AUTH_JMAP_URL", URL);
    }
    let path = common::test_db::unique_dir("header-authority").join("mw.db");
    let store = Store::open(path.to_str().unwrap(), ServerKey::from_hex(KEY).unwrap())
        .await
        .unwrap();
    store
        .set_setting("v6:upstream_import:done", "1")
        .await
        .unwrap();
    let credentials = Credentials {
        username: "bare-user".into(),
        password: "service-secret".into(),
    };
    let header = store
        .create_session("account", "bare-user", URL, URL, &credentials)
        .await
        .unwrap();
    store
        .set_session_authentication(&header, "header-auth")
        .await
        .unwrap();
    let ordinary = store
        .create_session("account", "bare-user", URL, URL, &credentials)
        .await
        .unwrap();
    let app = build_app(AppConfig {
        db_path: path.to_string_lossy().into_owned(),
        server_key_hex: Some(KEY.into()),
        mode: ServerMode::Proxy,
        cookie_secure: false,
        web_dir: None,
        hardening: Default::default(),
        security: SecurityConfig::default(),
    })
    .await
    .unwrap();
    assert!(store.get_session(&header).await.is_ok());
    assert!(store.get_session(&ordinary).await.is_err());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let server = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    let client = reqwest::Client::builder().no_proxy().build().unwrap();
    let response = client
        .get(format!("{base}/api/me"))
        .header("cookie", format!("mw_session={header}"))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200, "{}", response.text().await.unwrap());
    // A password session inserted after startup still cannot borrow the override.
    let ordinary = store
        .create_session("account", "bare-user", URL, URL, &credentials)
        .await
        .unwrap();
    let response = client
        .get(format!("{base}/api/me"))
        .header("cookie", format!("mw_session={ordinary}"))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 401);
    server.abort();
}
