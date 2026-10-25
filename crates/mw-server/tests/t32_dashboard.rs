//! Recovery coverage for the database admin account and role routes.
use std::net::SocketAddr;

use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use reqwest::{Client, Response};
use serde_json::{Value, json};

mod common;
use common::test_db;

async fn server() -> String {
    let web = test_db::unique_dir("dashboard-web");
    std::fs::write(web.join("index.html"), "<!doctype html><title>Test</title>").unwrap();
    let config = AppConfig {
        db_path: test_db::unique_db_path("dashboard")
            .to_string_lossy()
            .into(),
        server_key_hex: None,
        web_dir: Some(web),
        cookie_secure: false,
        mode: ServerMode::Proxy,
        hardening: HardeningConfig::default(),
        security: SecurityConfig::default(),
    };
    let (app, _) = build_app_full(
        config,
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
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });
    format!("http://{addr}")
}

fn client() -> Client {
    Client::builder()
        .no_proxy()
        .cookie_store(true)
        .build()
        .unwrap()
}

async fn login(c: &Client, base: &str, username: &str, password: &str) -> Response {
    c.post(format!("{base}/admin/login"))
        .json(&json!({"username": username, "password": password}))
        .send()
        .await
        .unwrap()
}

async fn confirm(c: &Client, base: &str, password: &str) {
    let response = c
        .post(format!("{base}/admin/step-up"))
        .json(&json!({"password": password}))
        .send()
        .await
        .unwrap();
    assert_eq!(
        response.status(),
        200,
        "step-up: {}",
        response.text().await.unwrap()
    );
}

async fn root(base: &str) -> Client {
    let c = client();
    assert_eq!(login(&c, base, "root", "hunter2").await.status(), 200);
    confirm(&c, base, "hunter2").await;
    c
}

#[tokio::test]
async fn dashboard_is_real_bounded_and_scope_filtered() {
    let base = server().await;
    let anonymous = client();
    assert_eq!(
        anonymous
            .get(format!("{base}/admin/dashboard"))
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    let root = root(&base).await;
    for domain in ["allowed.example", "other.example"] {
        let response = root
            .put(format!("{base}/admin/domains/{domain}"))
            .json(&json!({"enabled":false}))
            .send()
            .await
            .unwrap();
        assert!(
            response.status().is_success(),
            "{}",
            response.text().await.unwrap()
        );
    }
    let response = root
        .get(format!("{base}/admin/dashboard"))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200, "{}", response.text().await.unwrap());
    let response = root
        .get(format!("{base}/admin/dashboard"))
        .send()
        .await
        .unwrap();
    assert_eq!(response.headers()["cache-control"], "no-store");
    let data: Value = response.json().await.unwrap();
    assert_eq!(data["counts"]["domains"], 2);
    assert_eq!(data["counts"]["domainsDisabled"], 2);
    assert!(data["server"]["startedAt"].as_u64().unwrap() > 0);
    assert!(data["health"]["dbPingMs"].as_f64().unwrap() >= 0.0);
    assert!(data["counts"]["activeSessions"].is_null());
    assert!(data["unavailable"]["counts.activeSessions"].is_string());
    assert!(!data["recentAudit"].as_array().unwrap().is_empty());
    assert!(data["recentAudit"][0]["id"].is_string());
    let role: Value = root
        .post(format!("{base}/admin/roles"))
        .json(&json!({"name":"Dashboard reader","capabilities":["dashboard.read","audit.read","audit.export"]}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    for (name, roles) in [
        (
            "scoped",
            json!([{"roleId":role["id"],"domains":["allowed.example"]}]),
        ),
        ("denied", json!([])),
    ] {
        let response = root
            .post(format!("{base}/admin/admins"))
            .json(&json!({"username":name,"password":"long test password","roles":roles}))
            .send()
            .await
            .unwrap();
        assert!(
            response.status().is_success(),
            "{}",
            response.text().await.unwrap()
        );
    }
    let scoped = client();
    assert_eq!(
        login(&scoped, &base, "scoped", "long test password")
            .await
            .status(),
        200
    );
    let response = scoped
        .get(format!("{base}/admin/dashboard"))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let data: Value = response.json().await.unwrap();
    assert_eq!(data["counts"]["domains"], 1);
    assert!(data["counts"]["activeAdminSessions"].is_null());
    for key in ["server", "security", "storage", "health", "traffic"] {
        assert!(data[key].is_null(), "{key}");
    }
    assert_eq!(data["recentAudit"], json!([]));
    assert!(!data.to_string().contains("other.example"));
    // Both JSON and export must scope by the changed target, not the actor.
    let audit: Value = scoped
        .get(format!("{base}/admin/audit?limit=1000"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(!audit.as_array().unwrap().is_empty());
    for row in audit.as_array().unwrap() {
        assert_eq!(row["target"], "domain:allowed.example");
    }
    let export = scoped
        .get(format!("{base}/admin/audit/export?limit=1000"))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert!(!export.contains("other.example"));
    for line in export.lines() {
        let row: Value = serde_json::from_str(line).unwrap();
        assert_eq!(row["target"], "domain:allowed.example");
    }
    let denied = client();
    assert_eq!(
        login(&denied, &base, "denied", "long test password")
            .await
            .status(),
        200
    );
    assert_eq!(
        denied
            .get(format!("{base}/admin/dashboard"))
            .send()
            .await
            .unwrap()
            .status(),
        403
    );
}
