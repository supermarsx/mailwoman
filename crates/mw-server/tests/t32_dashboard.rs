//! Recovery coverage for the database admin account and role routes.
use std::net::SocketAddr;

use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use reqwest::{Client, Response};
use serde_json::{Value, json};

mod common;
use common::test_db;

const KEY: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
async fn server() -> (String, mw_store::Store, sqlx::SqlitePool) {
    let web = test_db::unique_dir("dashboard-web");
    std::fs::write(web.join("index.html"), "<!doctype html><title>Test</title>").unwrap();
    let db = test_db::unique_db_path("dashboard");
    let store = mw_store::Store::open(db.to_str().unwrap(),mw_store::ServerKey::from_hex(KEY).unwrap()).await.unwrap();
    let pool = sqlx::SqlitePool::connect(&format!("sqlite://{}",db.display())).await.unwrap();
    let config = AppConfig {
        db_path: db.to_string_lossy().into(),
        server_key_hex: Some(KEY.into()),
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
    (format!("http://{addr}"), store, pool)
}

fn client() -> Client {
    Client::builder()
        .no_proxy()
        .cookie_store(true)
        .build()
        .unwrap()
}

async fn login(c: &Client, base: &str, username: &str, password: &str) -> Response {
    tokio::time::sleep(std::time::Duration::from_millis(3100)).await;
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
    let (base, _, _) = server().await;
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
    assert_eq!(data["counts"]["activeSessions"],0);
    assert!(data["unavailable"].get("counts.activeSessions").is_none());
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

#[tokio::test]
async fn dashboard_counts_unexpired_sessions_without_extending_activity() {
    let (base, store, pool) = server().await;
    let root = root(&base).await;
    for domain in ["active.test", "disabled.test"] {
        store.put_domain_config(&mw_store::DomainConfigRow{name:domain.into(),enabled:domain=="active.test",upstream_json:json!({"imap":{"host":"localhost","port":993}}).to_string(),allowlist_json:"[]".into(),blocklist_json:"[]".into(),preferred:false,settings_json:"{}".into(),updated_at:0}).await.unwrap();
    }
    store.put_domain_alias("alias.test","active.test").await.unwrap();
    let mut ids=Vec::new();
    for username in ["live@active.test","alias@alias.test","idle@active.test","old@active.test","blocked@active.test","off@disabled.test"] {
        let id=store.create_session(username,username,"engine","engine",&mw_store::Credentials{username:username.into(),password:"fixture".into()}).await.unwrap();
        ids.push(id);
    }
    let old=(chrono::Utc::now()-chrono::Duration::days(16)).to_rfc3339();
    sqlx::query("UPDATE sessions SET activity_at=?1 WHERE id=?2").bind(&old).bind(&ids[2]).execute(&pool).await.unwrap();
    sqlx::query("UPDATE sessions SET created_at=?1 WHERE id=?2").bind(&old).bind(&ids[3]).execute(&pool).await.unwrap();
    store.set_setting("v6:admin:flags:blocked@active.test",r#"{"disabled":true}"#).await.unwrap();
    let before=store.session_activity_clock(&ids[0]).await.unwrap();
    let response=root.get(format!("{base}/admin/dashboard")).send().await.unwrap();
    assert_eq!(response.status(),200);
    let value:Value=response.json().await.unwrap();
    assert_eq!(value["counts"]["activeSessions"],2,"{value}");
    assert!(value["unavailable"].get("counts.activeSessions").is_none());
    assert_eq!(store.session_activity_clock(&ids[0]).await.unwrap(),before);
    store.delete_session(&ids[0]).await.unwrap();
    let value:Value=root.get(format!("{base}/admin/dashboard")).send().await.unwrap().json().await.unwrap();
    assert_eq!(value["counts"]["activeSessions"],1);
}
