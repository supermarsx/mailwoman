use base64::Engine as _;
use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use serde_json::{Value, json};
mod common;

const RAW: &str = "From: daemon@example.org\r\nContent-Type: multipart/report; report-type=delivery-status; boundary=b\r\n\r\n--b\r\nContent-Type: text/plain\r\n\r\nFailed\r\n--b\r\nContent-Type: message/delivery-status\r\n\r\nReporting-MTA: dns; mx.example.org\r\n\r\nFinal-Recipient: rfc822; person@example.org\r\nAction: failed\r\nStatus: 4.4.7\r\nDiagnostic-Code: smtp; retries exhausted\r\n--b--\r\n";

#[tokio::test]
async fn raw_report_parser_requires_auth_csrf_and_limits_and_accepts_native() {
    let upstream = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let upstream_url = format!("http://{}", upstream.local_addr().unwrap());
    tokio::spawn(async move {
        axum::serve(upstream, mw_mock_jmap::router()).await.unwrap();
    });
    let path = common::test_db::unique_db_path("dsn-http");
    let key = "1111111111111111111111111111111111111111111111111111111111111111";
    let (app, _) = build_app_full(
        AppConfig {
            db_path: path.to_string_lossy().into(),
            server_key_hex: Some(key.into()),
            web_dir: None,
            cookie_secure: false,
            mode: ServerMode::Proxy,
            hardening: HardeningConfig {
                csrf_strict: true,
                ..Default::default()
            },
            security: SecurityConfig {
                jmap_upstreams: Some(vec![upstream_url.clone()]),
                upstream_default: Some(upstream_url),
                ..Default::default()
            },
        },
        V6Config {
            admin_enabled: false,
            admin_username: None,
            admin_password: None,
            redis_url: None,
        },
    )
    .await
    .unwrap();
    let store = mw_store::Store::open(
        &path.to_string_lossy(),
        mw_store::ServerKey::from_hex(key).unwrap(),
    )
    .await
    .unwrap();
    store
        .put_setting_policy(&mw_store::settings_policy::SettingPolicyRow {
            key: "signin.retryCooldownSecs".into(),
            global_value: Some("0".into()),
            allow_user_override: false,
            global_locked: false,
            updated_by: "fixture".into(),
            updated_at: 1,
        })
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
    let endpoint = format!("{base}/api/mail/delivery-status");
    let payload = json!({"bytesBase64":base64::engine::general_purpose::STANDARD.encode(RAW)});
    let anonymous = client
        .post(&endpoint)
        .bearer_auth("invalid")
        .json(&payload)
        .send()
        .await
        .unwrap();
    assert_eq!(anonymous.status(), 401);
    let login = client
        .post(format!("{base}/api/login"))
        .header("Origin", &base)
        .json(&json!({"username":mw_mock_jmap::USER,"password":mw_mock_jmap::PASS}))
        .send()
        .await
        .unwrap();
    assert_eq!(login.status(), 200);
    let login: Value = login.json().await.unwrap();
    let csrf = login["csrfToken"].as_str().unwrap();
    assert_eq!(
        client
            .post(&endpoint)
            .header("Origin", &base)
            .json(&payload)
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    assert_eq!(
        client
            .post(&endpoint)
            .header("Origin", "https://hostile.invalid")
            .header("x-csrf-token", csrf)
            .json(&payload)
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    let response = client
        .post(&endpoint)
        .header("Origin", &base)
        .header("x-csrf-token", csrf)
        .json(&payload)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.headers()["cache-control"], "private, no-store");
    let parsed: Value = response.json().await.unwrap();
    assert_eq!(parsed["report"]["recipients"][0]["status"], "4.4.7");
    for (body, status) in [
        (json!({"bytesBase64":"%%%"}), 400),
        (json!({"bytesBase64":"YQ==","url":"http://localhost"}), 422),
        (
            json!({"bytesBase64":"A".repeat((8usize*1024*1024).div_ceil(3)*4+4)}),
            413,
        ),
    ] {
        assert_eq!(
            client
                .post(&endpoint)
                .header("Origin", &base)
                .header("x-csrf-token", csrf)
                .json(&body)
                .send()
                .await
                .unwrap()
                .status(),
            status
        );
    }
    let native = reqwest::Client::builder().no_proxy().build().unwrap();
    let login=native.post(format!("{base}/api/login")).header("Origin","tauri://localhost").json(&json!({"username":mw_mock_jmap::USER,"password":mw_mock_jmap::PASS,"clientType":"native"})).send().await.unwrap();
    assert_eq!(login.status(), 200);
    let login: Value = login.json().await.unwrap();
    let response = native
        .post(&endpoint)
        .bearer_auth(login["token"].as_str().unwrap())
        .json(&payload)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
}
