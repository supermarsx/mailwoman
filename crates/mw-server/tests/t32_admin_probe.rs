use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use mw_store::{ServerKey, Store, settings_policy::SettingPolicyRow};
use serde_json::{Value, json};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
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
fn client(ip: u8) -> reqwest::Client {
    reqwest::Client::builder()
        .no_proxy()
        .cookie_store(true)
        .local_address(IpAddr::V4(Ipv4Addr::new(127, 0, 0, ip)))
        .build()
        .unwrap()
}
async fn fixture(values: &[(&str, Value)]) -> (String, Store) {
    let upstream = serve(mw_mock_jmap::router()).await;
    let db = common::test_db::unique_db_path("admin-probe-http");
    let key = "5a69788796a5b4c3d2e1f00f1e2d3c4b5a69788796a5b4c3d2e1f00f1e2d3c4b";
    let store = Store::open(db.to_str().unwrap(), ServerKey::from_hex(key).unwrap())
        .await
        .unwrap();
    for (key, value) in
        std::iter::once(("signin.retryCooldownSecs", json!(0))).chain(values.iter().cloned())
    {
        store
            .put_setting_policy(&SettingPolicyRow {
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
    let (router, _) = build_app_full(
        AppConfig {
            db_path: db.to_string_lossy().into(),
            server_key_hex: Some(key.into()),
            web_dir: None,
            cookie_secure: false,
            mode: ServerMode::Proxy,
            hardening: HardeningConfig::default(),
            security: SecurityConfig {
                upstream_default: Some(upstream.clone()),
                jmap_upstreams: Some(vec![upstream]),
                ..Default::default()
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
    (serve(router).await, store)
}

#[tokio::test]
async fn only_unmatched_admin_discovery_is_an_opt_in_signal() {
    // This integration binary has one test; no other task mutates this environment.
    unsafe {
        std::env::set_var("MW_ADMIN_PATH", "/ops-probe-test");
    }
    let (base, store) = fixture(&[
        ("security.protection.adaptiveMode", json!("enforce")),
        ("security.protection.adaptiveAccounts", json!(false)),
        ("security.protection.adaptiveAdmins", json!(false)),
        ("security.protection.signals", json!(["admin-route-miss"])),
        ("security.protection.shortAbsoluteAnomalies", json!(2)),
        ("security.protection.shortRatioPercent", json!(0)),
        ("security.protection.longAbsoluteAnomalies", json!(0)),
        ("security.protection.longRatioPercent", json!(0)),
        ("security.protection.throttleSecs", json!(60)),
    ])
    .await;
    let browser = client(2);
    assert_eq!(
        browser
            .post(format!("{base}/ops-probe-test/login"))
            .json(&json!({"username":"root","password":"hunter2"}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    for path in [
        "/ops-probe-test/domains/missing.example",
        "/ops-probe-test/assets/stale.js",
        "/ops-probe-test/stale.woff2",
        "/ops-probe-test/login/obsolete",
        "/elsewhere/discovery",
    ] {
        assert_eq!(
            browser
                .get(format!("{base}{path}"))
                .send()
                .await
                .unwrap()
                .status(),
            404,
            "{path}"
        );
    }
    assert_eq!(
        browser
            .get(format!("{base}/ops-probe-test/login"))
            .send()
            .await
            .unwrap()
            .status(),
        405
    );
    assert!(
        store
            .request_protection_events(chrono::Utc::now().timestamp(), None, 50)
            .await
            .unwrap()
            .is_empty()
    );
    for path in [
        "/ops-probe-test/phpmyadmin",
        "/ops-probe-test/hidden-console",
    ] {
        assert_eq!(
            browser
                .get(format!("{base}{path}"))
                .send()
                .await
                .unwrap()
                .status(),
            404
        );
    }
    let events = store
        .request_protection_events(chrono::Utc::now().timestamp(), None, 50)
        .await
        .unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(
        events[0].snapshot.last_signal,
        Some(mw_store::request_protection_model::Signal::AdminRouteMiss)
    );
    assert_eq!(events[0].snapshot.short.anomalous, 2);
    let blocked = browser.get(format!("{base}/api/me")).send().await.unwrap();
    assert_eq!(blocked.status(), 429);
    assert!(blocked.headers().contains_key("retry-after"));
}
