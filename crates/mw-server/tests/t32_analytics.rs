use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use serde_json::{Value, json};
mod common;

#[tokio::test]
async fn analytics_is_off_by_default_and_configuration_requires_fresh_proof_and_revision() {
    let mock_socket = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let mock_url = format!("http://{}", mock_socket.local_addr().unwrap());
    let mock_task = tokio::spawn(async move {
        axum::serve(mock_socket, mw_mock_jmap::router())
            .await
            .unwrap();
    });
    let (app, _) = build_app_full(
        AppConfig {
            db_path: common::test_db::unique_db_path("analytics")
                .to_string_lossy()
                .into(),
            server_key_hex: None,
            web_dir: None,
            cookie_secure: false,
            mode: ServerMode::Proxy,
            hardening: HardeningConfig::default(),
            security: SecurityConfig {
                upstream_default: Some(mock_url.clone()),
                jmap_upstreams: Some(vec![mock_url]),
                ..SecurityConfig::default()
            },
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
    let task = tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .unwrap();
    });
    let c = reqwest::Client::builder()
        .no_proxy()
        .cookie_store(true)
        .build()
        .unwrap();
    assert_eq!(
        c.get(format!("{base}/admin/analytics"))
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    assert_eq!(
        c.post(format!("{base}/api/analytics/events"))
            .json(&json!({"screen":"mail"}))
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    assert_eq!(
        c.post(format!("{base}/admin/login"))
            .json(&json!({"username":"root","password":"hunter2"}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    let response = c
        .get(format!("{base}/admin/analytics"))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.headers()["cache-control"], "private, no-store");
    let tag = response.headers()["etag"].to_str().unwrap().to_owned();
    let view: Value = response.json().await.unwrap();
    assert_eq!(view["config"]["enabled"], false);
    assert!(view["status"]["uniqueVisitors"].is_null());
    let value = json!({"enabled":true,"provider":"matomo","endpoint":"https://stats.example/matomo.php","siteId":"1","allowHttp":false});
    assert_eq!(
        c.put(format!("{base}/admin/analytics"))
            .header("if-match", &tag)
            .json(&value)
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
    assert_eq!(
        c.post(format!("{base}/admin/step-up"))
            .json(&json!({"password":"hunter2"}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    assert_eq!(
        c.put(format!("{base}/admin/analytics"))
            .json(&value)
            .send()
            .await
            .unwrap()
            .status(),
        428
    );
    let response = c
        .put(format!("{base}/admin/analytics"))
        .header("if-match", &tag)
        .json(&value)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200, "{}", response.text().await.unwrap());
    assert_eq!(
        c.put(format!("{base}/admin/analytics"))
            .header("if-match", &tag)
            .json(&value)
            .send()
            .await
            .unwrap()
            .status(),
        412
    );
    // Secret rotation is one atomic config revision; responses never expose plaintext.
    let response = c
        .get(format!("{base}/admin/analytics"))
        .send()
        .await
        .unwrap();
    let tag = response.headers()["etag"].to_str().unwrap().to_owned();
    let ga4 = json!({"enabled":true,"provider":"ga4","endpoint":"https://www.google-analytics.com/mp/collect","siteId":"G-TEST123","allowHttp":false,"apiSecret":"ga4_fixture_secret"});
    let response = c
        .put(format!("{base}/admin/analytics"))
        .header("if-match", &tag)
        .json(&ga4)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let next_tag = response.headers()["etag"].to_str().unwrap().to_owned();
    let raw = response.text().await.unwrap();
    assert!(!raw.contains("ga4_fixture_secret"));
    assert!(!raw.contains("apiSecretSealed"));
    assert_eq!(
        serde_json::from_str::<Value>(&raw).unwrap()["config"]["apiSecretConfigured"],
        true
    );
    let mut cleared = ga4.clone();
    cleared.as_object_mut().unwrap().remove("apiSecret");
    cleared["clearApiSecret"] = json!(true);
    assert_eq!(
        c.put(format!("{base}/admin/analytics"))
            .header("if-match", &next_tag)
            .json(&cleared)
            .send()
            .await
            .unwrap()
            .status(),
        422
    );
    cleared["enabled"] = json!(false);
    let response = c
        .put(format!("{base}/admin/analytics"))
        .header("if-match", &next_tag)
        .json(&cleared)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(
        response.json::<Value>().await.unwrap()["config"]["apiSecretConfigured"],
        false
    );
    let response = c
        .get(format!("{base}/admin/analytics"))
        .send()
        .await
        .unwrap();
    let tag = response.headers()["etag"].to_str().unwrap().to_owned();
    assert_eq!(
        c.put(format!("{base}/admin/analytics"))
            .header("if-match", tag)
            .json(&value)
            .send()
            .await
            .unwrap()
            .status(),
        200
    );

    assert_eq!(
        c.post(format!("{base}/admin/analytics/allowlist"))
            .json(&json!({"destination":"169.254.169.254"}))
            .send()
            .await
            .unwrap()
            .status(),
        422
    );
    assert_eq!(
        c.post(format!("{base}/admin/analytics/allowlist"))
            .json(&json!({"destination":"collector.corp.example:443"}))
            .send()
            .await
            .unwrap()
            .status(),
        204
    );
    let view: Value = c
        .get(format!("{base}/admin/analytics"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(view["allowlist"].as_array().unwrap().len(), 1);
    assert_eq!(view["status"]["queued"], 0);
    assert_eq!(view["status"]["delivered"], 0);
    let user = reqwest::Client::builder()
        .no_proxy()
        .cookie_store(true)
        .build()
        .unwrap();
    // Console and mailbox share this test client's source IP. Respect the
    // default successful-sign-in cooldown before making a different login.
    tokio::time::sleep(std::time::Duration::from_secs(4)).await;
    let response = user
        .post(format!("{base}/api/login"))
        .json(&json!({"username":mw_mock_jmap::USER,"password":mw_mock_jmap::PASS}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200, "{}", response.text().await.unwrap());
    assert_eq!(
        user.post(format!("{base}/api/analytics/events"))
            .json(&json!({"screen":"mail"}))
            .send()
            .await
            .unwrap()
            .status(),
        204
    );
    let me: Value = user
        .get(format!("{base}/api/me"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let authority = me["storageAuthority"].as_str().unwrap();
    let response = user
        .get(format!("{base}/api/account/preferences"))
        .send()
        .await
        .unwrap();
    let tag = response.headers()["etag"].to_str().unwrap().to_owned();
    assert_eq!(
        user.put(format!("{base}/api/account/preferences"))
            .header("x-mailwoman-storage-authority", authority)
            .header("if-match", tag)
            .json(&json!({"version":1,"preferences":{"analyticsConsent":true}}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    for privacy_header in ["DNT", "Sec-GPC"] {
        assert_eq!(
            user.post(format!("{base}/api/analytics/events"))
                .header(privacy_header, "1")
                .json(&json!({"screen":"mail"}))
                .send()
                .await
                .unwrap()
                .status(),
            204
        );
    }
    assert_eq!(
        user.post(format!("{base}/api/analytics/events"))
            .json(&json!({"screen":"mail","subject":"private"}))
            .send()
            .await
            .unwrap()
            .status(),
        422
    );
    let view: Value = c
        .get(format!("{base}/admin/analytics"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(view["status"]["queued"], 0);
    assert_eq!(view["status"]["delivered"], 0);
    assert_eq!(view["status"]["dropped"], 0);
    task.abort();
    mock_task.abort();
}
