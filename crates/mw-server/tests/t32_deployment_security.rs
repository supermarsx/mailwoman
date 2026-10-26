use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use serde_json::{Value, json};
mod common;
#[tokio::test]
async fn assessment_reports_actual_policy_without_secrets_and_requires_global_diagnostics() {
    let path = common::test_db::unique_db_path("deployment-security");
    let secret = "deployment-assessment-password";
    let app = build_app_full(
        AppConfig {
            db_path: path.to_string_lossy().into_owned(),
            server_key_hex: Some("13".repeat(32)),
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
    let store = mw_store::Store::open(
        &path.to_string_lossy(),
        mw_store::ServerKey::from_bytes(&[0x13; 32]).unwrap(),
    )
    .await
    .unwrap();
    for (key, value) in [
        ("signin.retryCooldownSecs", json!(0)),
        ("admin.session.idleSecs", json!(10000)),
    ] {
        store
            .put_setting_policy(&mw_store::settings_policy::SettingPolicyRow {
                key: key.into(),
                global_value: Some(value.to_string()),
                allow_user_override: false,
                global_locked: false,
                updated_by: "fixture".into(),
                updated_at: 1,
            })
            .await
            .unwrap();
    }
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let client = reqwest::Client::builder()
        .cookie_store(true)
        .no_proxy()
        .build()
        .unwrap();
    let endpoint = format!("{base}/admin/security/deployment");
    assert_eq!(client.get(&endpoint).send().await.unwrap().status(), 401);
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
        .get(&endpoint)
        .header("x-forwarded-proto", "https")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.headers()["cache-control"], "private, no-store");
    let text = response.text().await.unwrap();
    assert!(!text.contains(secret));
    assert!(!text.contains(&"13".repeat(32)));
    assert!(!text.contains(path.to_string_lossy().as_ref()));
    let report: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(report["scope"], "current-process-and-request");
    let checks = report["checks"].as_array().unwrap();
    let value = |id: &str| checks.iter().find(|row| row["id"] == id).unwrap()["value"].clone();
    assert_eq!(
        value("https")["effectiveRequestHttps"],
        false,
        "untrusted header must not promote transport"
    );
    assert_eq!(value("secureCookies"), false);
    assert_eq!(value("connectionNegotiation")["requiresRestart"], true);
    assert_eq!(
        value("connectionNegotiation")["source"],
        "built-in-listener-startup-configuration"
    );
    assert_eq!(value("connectionNegotiation")["tlsMaximum"], 128);
    assert_eq!(value("httpAdmission")["requiresRestart"], true);
    assert_eq!(value("httpAdmission")["adaptive"], false);
    assert_eq!(value("httpAdmission")["headerBytes"], 32768);
    assert_eq!(value("httpAdmission")["http2Streams"], 32);

    assert_eq!(value("connectionNegotiation")["tlsTimeoutMs"], 10000);
    assert_eq!(value("connectionNegotiation")["proxyTimeoutMs"], 5000);

    assert_eq!(
        value("adminSessions")["idleSeconds"],
        7200,
        "report runtime clamp, not configured 10000"
    );
    assert_eq!(value("adminFactors"), 1, "environment admin is included");
    assert_eq!(report["limitations"].as_array().unwrap().len(), 4);
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
        .json(&json!({"name":"Deployment reader","capabilities":["diagnostics.read"]}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    for (name, roles, expected) in [
        (
            "scoped",
            json!([{"roleId":role["id"],"domains":["example.org"]}]),
            403,
        ),
        ("reader", json!([{"roleId":role["id"]}]), 200),
        ("denied", json!([]), 403),
    ] {
        let created = client
            .post(format!("{base}/admin/admins"))
            .json(&json!({"username":name,"password":"bounded test password","roles":roles}))
            .send()
            .await
            .unwrap();
        assert_eq!(created.status(), 201, "{name}");
        let actor = reqwest::Client::builder()
            .cookie_store(true)
            .no_proxy()
            .build()
            .unwrap();
        assert_eq!(
            actor
                .post(format!("{base}/admin/login"))
                .json(&json!({"username":name,"password":"bounded test password"}))
                .send()
                .await
                .unwrap()
                .status(),
            200
        );
        assert_eq!(
            actor.get(&endpoint).send().await.unwrap().status().as_u16(),
            expected,
            "{name}"
        );
    }
    task.abort();
}
