//! t48: console settings that the server now applies, driven over HTTP.
//!
//! - `/.well-known/security.txt` is built from `about.securityContact` and
//!   `about.securityPolicyUrl`, and is a 404 without a contact.
//! - A domain's `import.maxMessages` and `import.maxMessageBytes` refuse an
//!   import over them, and `/api/import/formats` reports them.
//!
//! The unit tests next to each reader cover the environment-pinned keys
//! (`newly_effective.rs`, `masked.rs`) and the webhook limits (`webhooks.rs`).
mod common;

use mw_server::{AppConfig, SecurityConfig, ServerMode, build_app};
use mw_store::settings_policy::{SettingPolicyRow, SettingScope, SettingValueRow};
use mw_store::{ServerKey, Store};
use serde_json::{Value, json};

const KEY: &str = "5a69788796a5b4c3d2e1f00f1e2d3c4b5a69788796a5b4c3d2e1f00f1e2d3c4b";

async fn serve(app: axum::Router) -> String {
    let socket = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", socket.local_addr().unwrap());
    tokio::spawn(async move {
        axum::serve(
            socket,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .unwrap();
    });
    base
}

async fn put_global(store: &Store, key: &str, json: &str) {
    store
        .put_setting_policy(&SettingPolicyRow {
            key: key.into(),
            global_value: Some(json.into()),
            allow_user_override: false,
            global_locked: false,
            updated_by: "test".into(),
            updated_at: 1,
        })
        .await
        .unwrap();
}

async fn put_domain(store: &Store, domain: &str, key: &str, json: &str) {
    store
        .put_setting_value(&SettingValueRow {
            scope: SettingScope::Domain,
            scope_id: domain.into(),
            key: key.into(),
            value: json.into(),
            locked: false,
            updated_by: "test".into(),
            updated_at: 1,
        })
        .await
        .unwrap();
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .no_proxy()
        .cookie_store(true)
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .unwrap()
}

#[tokio::test]
async fn security_txt_follows_the_about_settings() {
    let path = common::test_db::unique_db_path("t48-security-txt");
    let store = Store::open(path.to_str().unwrap(), ServerKey::from_hex(KEY).unwrap())
        .await
        .unwrap();
    let app = build_app(AppConfig {
        db_path: path.to_string_lossy().into(),
        server_key_hex: Some(KEY.into()),
        web_dir: None,
        cookie_secure: false,
        mode: ServerMode::Proxy,
        hardening: Default::default(),
        security: SecurityConfig::default(),
    })
    .await
    .unwrap();
    let base = serve(app).await;
    let http = client();
    let url = format!("{base}/.well-known/security.txt");

    let absent = http.get(&url).send().await.unwrap();
    assert_eq!(absent.status(), 404, "no contact, no file");

    put_global(&store, "about.securityPolicyUrl", "\"https://example.org/policy\"").await;
    assert_eq!(http.get(&url).send().await.unwrap().status(), 404, "a policy alone is not enough");

    put_global(&store, "about.securityContact", "\"mailto:security@example.org\"").await;
    let resp = http.get(&url).send().await.unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(
        resp.headers()["content-type"].to_str().unwrap(),
        "text/plain; charset=utf-8"
    );
    let body = resp.text().await.unwrap();
    let lines: Vec<&str> = body.lines().collect();
    assert_eq!(lines[0], "Contact: mailto:security@example.org");
    assert!(lines[1].starts_with("Expires: ") && lines[1].ends_with("T00:00:00Z"), "{body}");
    let expires = chrono::DateTime::parse_from_rfc3339(&lines[1]["Expires: ".len()..]).unwrap();
    let days = (expires.with_timezone(&chrono::Utc) - chrono::Utc::now()).num_days();
    assert!((363..=365).contains(&days), "expires in about a year: {body}");
    assert_eq!(lines[2], "Policy: https://example.org/policy");
    assert_eq!(lines.len(), 3, "{body}");
}

#[tokio::test]
async fn a_domain_import_limit_refuses_a_larger_import() {
    let mail = common::mock_imap::MockImap::start("owner@example.org", "test-password").await;
    let root = common::test_db::unique_dir("t48-import-limits");
    let uploads = root.join("uploads");
    std::fs::create_dir_all(&uploads).unwrap();
    // Startup-only fixture variables; this binary runs with --test-threads=1.
    unsafe {
        std::env::set_var("MW_ENGINE_TLS", "plaintext");
        std::env::set_var("MW_UPLOAD_DIR", &uploads);
    }
    let db = root.join("db.sqlite");
    let store = Store::open(db.to_str().unwrap(), ServerKey::from_hex(KEY).unwrap())
        .await
        .unwrap();
    common::auth::no_signin_cooldown(&store).await;
    let app = build_app(AppConfig {
        db_path: db.to_string_lossy().into_owned(),
        server_key_hex: Some(KEY.into()),
        web_dir: None,
        cookie_secure: false,
        mode: ServerMode::Engine,
        hardening: Default::default(),
        security: SecurityConfig {
            upstream_default: Some(mail.url()),
            ..Default::default()
        },
    })
    .await
    .unwrap();
    let base = serve(app).await;
    let http = client();
    let response = http
        .post(format!("{base}/api/login"))
        .json(&json!({"username":"owner@example.org","password":"test-password"}))
        .send()
        .await
        .unwrap();
    let status = response.status();
    let login: Value = response.json().await.unwrap();
    assert_eq!(status, 200, "{login}");

    let formats = |http: reqwest::Client| {
        let url = format!("{base}/api/import/formats");
        async move { http.get(url).send().await.unwrap().json::<Value>().await.unwrap() }
    };
    let before = formats(http.clone()).await;
    assert_eq!(before["maxMessages"], 20_000, "{before}");
    assert_eq!(before["maxBytes"], 64 * 1024 * 1024, "{before}");

    put_domain(&store, "example.org", "import.maxMessages", "2").await;
    put_domain(&store, "example.org", "import.maxMessageBytes", "200").await;
    // Another domain's value does not apply here.
    put_domain(&store, "other.example", "import.maxMessages", "1").await;
    let after = formats(http.clone()).await;
    assert_eq!(after["maxMessages"], 2, "{after}");
    assert_eq!(after["maxBytes"], 200, "{after}");

    let msg = |n: u32| {
        format!(
            "From sender@example.org Mon Jan  1 00:00:00 2024\r\nMessage-ID: <t48-{n}@example.org>\r\n\
             From: sender@example.org\r\nSubject: {n}\r\n\r\nbody {n}\r\n"
        )
    };
    let three = format!("{}{}{}", msg(1), msg(2), msg(3));
    let refused = http
        .post(format!("{base}/api/import/mbox"))
        .header("content-type", "application/mbox")
        .body(three)
        .send()
        .await
        .unwrap();
    let status = refused.status();
    let body: Value = refused.json().await.unwrap();
    assert_eq!(status, 400, "{body}");
    assert!(
        body["error"].as_str().unwrap().contains("too many messages"),
        "{body}"
    );

    let big = format!(
        "Message-ID: <t48-big@example.org>\r\nFrom: sender@example.org\r\nSubject: big\r\n\r\n{}\r\n",
        "x".repeat(300)
    );
    let refused = http
        .post(format!("{base}/api/import/eml"))
        .header("content-type", "message/rfc822")
        .body(big)
        .send()
        .await
        .unwrap();
    let status = refused.status();
    let body: Value = refused.json().await.unwrap();
    assert_eq!(status, 413, "{body}");
    assert_eq!(body["maxBytes"], 200, "{body}");

    // Within both limits: accepted.
    let two = format!("{}{}", msg(1), msg(2));
    let ok = http
        .post(format!("{base}/api/import/mbox"))
        .header("content-type", "application/mbox")
        .body(two)
        .send()
        .await
        .unwrap();
    let status = ok.status();
    let body: Value = ok.json().await.unwrap();
    assert_eq!(status, 200, "{body}");
    assert_eq!(body["total"], 2, "{body}");
}
