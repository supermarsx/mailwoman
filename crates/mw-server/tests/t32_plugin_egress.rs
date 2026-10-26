use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use serde_json::{Value, json};
mod common;

#[tokio::test]
async fn internal_destination_permissions_require_proof_and_remain_class_scoped() {
    let mock_socket = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let mock_url = format!("http://{}", mock_socket.local_addr().unwrap());
    let mock_task = tokio::spawn(async move {
        axum::serve(mock_socket, mw_mock_jmap::router())
            .await
            .unwrap();
    });
    let (app, _) = build_app_full(
        AppConfig {
            db_path: common::test_db::unique_db_path("plugin-egress")
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

    let endpoint = format!("{base}/admin/egress/destinations");
    assert_eq!(c.get(&endpoint).send().await.unwrap().status(), 401);
    assert_eq!(
        c.post(format!("{base}/admin/login"))
            .json(&json!({"username":"root","password":"hunter2"}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    let grant =
        json!({"trafficClass":"plugins","destination":"ews.corp.test:443","note":"Approved EWS"});
    assert_eq!(
        c.post(&endpoint)
            .json(&grant)
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
        c.post(format!("{base}/admin/analytics/allowlist"))
            .json(&json!({"destination":"ews.corp.test:443"}))
            .send()
            .await
            .unwrap()
            .status(),
        204
    );
    let ip = "10.20.0.7".parse().unwrap();
    let plan = mw_egress::plan::current().unwrap();
    assert!(plan.admits(
        mw_egress::TrafficClass::Analytics,
        "ews.corp.test",
        443,
        &ip
    ));
    assert!(!plan.admits(mw_egress::TrafficClass::Plugins, "ews.corp.test", 443, &ip));
    for invalid in [
        json!({"trafficClass":"plugins","destination":"169.254.169.254"}),
        json!({"trafficClass":"remote_content","destination":"10.20.0.0/16"}),
    ] {
        assert_eq!(
            c.post(&endpoint)
                .json(&invalid)
                .send()
                .await
                .unwrap()
                .status(),
            422
        );
    }
    let response = c.post(&endpoint).json(&grant).send().await.unwrap();
    assert_eq!(response.status(), 200, "{}", response.text().await.unwrap());
    let rows: Value = c.get(&endpoint).send().await.unwrap().json().await.unwrap();
    let plugin = rows["destinations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["trafficClass"] == "plugins")
        .unwrap();
    let id = plugin["id"].as_str().unwrap();
    assert!(mw_egress::plan::current().unwrap().admits(
        mw_egress::TrafficClass::Plugins,
        "ews.corp.test",
        443,
        &ip
    ));
    assert_eq!(
        c.post(&endpoint)
            .json(&grant)
            .send()
            .await
            .unwrap()
            .status(),
        422
    );
    assert_eq!(
        c.delete(format!("{endpoint}/{id}"))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    let plan = mw_egress::plan::current().unwrap();
    assert!(!plan.admits(mw_egress::TrafficClass::Plugins, "ews.corp.test", 443, &ip));
    assert!(plan.admits(
        mw_egress::TrafficClass::Analytics,
        "ews.corp.test",
        443,
        &ip
    ));
    task.abort();
    mock_task.abort();
}
