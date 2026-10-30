//! t99: the app's private JSON reads answer a strong ETag and 304 on a match,
//! stay out of shared caches, and the endpoints that chose `no-store` keep it.
use mw_server::{AppConfig, SecurityConfig, ServerMode, build_app};
use mw_store::{DomainConfigRow, ServerKey, Store};
use serde_json::json;
mod common;
use common::{mock_mail::MockPop3, test_db};

const KEY: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

async fn server(store_path: &str, mail: &MockPop3) -> String {
    let store = Store::open(store_path, ServerKey::from_hex(KEY).unwrap())
        .await
        .unwrap();
    store
        .put_domain_config(&DomainConfigRow {
            name: "example.org".into(),
            upstream_json: json!({"pop3":{"host":"127.0.0.1","port":mail.addr.port(),"tls":"none","allowPrivate":true}}).to_string(),
            allowlist_json: "[]".into(),
            blocklist_json: "[]".into(),
            enabled: true,
            preferred: false,
            settings_json: "{}".into(),
            updated_at: 1,
        })
        .await
        .unwrap();
    store
        .put_setting_policy(&mw_store::settings_policy::SettingPolicyRow {
            key: "signin.retryCooldownSecs".into(),
            global_value: Some("0".into()),
            allow_user_override: false,
            global_locked: false,
            updated_by: "t99-fixture".into(),
            updated_at: 0,
        })
        .await
        .unwrap();
    let config = AppConfig {
        db_path: store_path.into(),
        server_key_hex: Some(KEY.into()),
        mode: ServerMode::Engine,
        cookie_secure: false,
        web_dir: None,
        hardening: Default::default(),
        security: SecurityConfig {
            upstream_default: Some(mail.url()),
            ..Default::default()
        },
    };
    let app = build_app(config).await.unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    format!("http://{addr}")
}

#[tokio::test]
async fn private_json_reads_revalidate_with_304_and_no_store_stays_no_store() {
    let mail = MockPop3::start("alice@example.org", "testpass").await;
    let db = test_db::unique_dir("t99-private-etag")
        .join("db.sqlite")
        .to_string_lossy()
        .into_owned();
    let base = server(&db, &mail).await;
    let client = reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap();
    let login = client
        .post(format!("{base}/api/login"))
        .json(&json!({"username":"alice@example.org","password":"testpass"}))
        .send()
        .await
        .unwrap();
    assert_eq!(login.status(), 200, "{}", login.text().await.unwrap());

    for path in ["/jmap/session", "/api/settings"] {
        let first = client.get(format!("{base}{path}")).send().await.unwrap();
        assert_eq!(first.status(), 200, "{path}");
        let etag = first.headers()["etag"].to_str().unwrap().to_owned();
        assert!(etag.starts_with('"'), "{path}: strong tag {etag}");
        assert_eq!(
            first.headers()["cache-control"],
            "private, no-cache",
            "{path}"
        );
        let vary = first.headers()["vary"]
            .to_str()
            .unwrap()
            .to_ascii_lowercase();
        assert!(
            vary.contains("cookie") && vary.contains("x-mailwoman-link"),
            "{path}: {vary}"
        );
        let body = first.bytes().await.unwrap();
        assert!(!body.is_empty());

        let again = client
            .get(format!("{base}{path}"))
            .header("if-none-match", &etag)
            .send()
            .await
            .unwrap();
        assert_eq!(again.status(), 304, "{path}");
        assert_eq!(again.headers()["etag"], etag.as_str());
        // Outer layers still apply to a 304 (security headers).
        assert!(
            again.headers().contains_key("x-content-type-options"),
            "{path}"
        );
        assert!(again.bytes().await.unwrap().is_empty());

        let stale = client
            .get(format!("{base}{path}"))
            .header("if-none-match", "\"not-this-one\"")
            .send()
            .await
            .unwrap();
        assert_eq!(stale.status(), 200, "{path}");
        assert_eq!(
            stale.bytes().await.unwrap(),
            body,
            "{path}: same content, same bytes"
        );
    }

    // Answers that chose no-store keep it and get no validator.
    for path in [
        "/api/accounts",
        "/api/analytics/status",
        "/api/public-config",
    ] {
        let r = client
            .get(format!("{base}{path}"))
            .header("if-none-match", "*")
            .send()
            .await
            .unwrap();
        assert_ne!(r.status(), 304, "{path}");
        if let Some(cc) = r.headers().get("cache-control") {
            assert!(cc.to_str().unwrap().contains("no-store"), "{path}: {cc:?}");
        }
        assert!(!r.headers().contains_key("etag"), "{path}");
    }

    // Signed out: a 401 is never tagged or 304'd.
    let anonymous = reqwest::Client::new();
    let r = anonymous
        .get(format!("{base}/api/settings"))
        .header("if-none-match", "*")
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 401);
    assert!(!r.headers().contains_key("etag"));
}
