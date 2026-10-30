//! t103: the admin Health panel reports a measured value or a precise disabled
//! reason for every component, never "not reported".
//!
//! * proxy mode: sync, outbox, search and plugins are `disabled` with how to
//!   enable them; database, migrations and render are measured;
//! * engine mode: every component is measured; seeded submissions are counted
//!   by state; an enabled plugin is listed with its version and limits;
//! * only an admin with unscoped Dashboard read sees it;
//! * the endpoint answers within its budget and serves a cached result.
use std::net::SocketAddr;
use std::time::{Duration, Instant};

use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use reqwest::Client;
use serde_json::{Value, json};

mod common;
use common::test_db;

const KEY: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";
const COMPONENTS: [&str; 7] = [
    "database",
    "migrations",
    "sync",
    "outbox",
    "search",
    "render",
    "plugins",
];

async fn server(mode: ServerMode) -> (String, std::path::PathBuf) {
    let web = test_db::unique_dir("t103-web");
    std::fs::write(web.join("index.html"), "<!doctype html><title>Test</title>").unwrap();
    let db = test_db::unique_db_path("t103");
    let config = AppConfig {
        db_path: db.to_string_lossy().into(),
        server_key_hex: Some(KEY.into()),
        web_dir: Some(web),
        cookie_secure: false,
        mode,
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
    (format!("http://{addr}"), db)
}

fn client() -> Client {
    Client::builder()
        .no_proxy()
        .cookie_store(true)
        .build()
        .unwrap()
}

async fn login(base: &str, user: &str, password: &str) -> Client {
    tokio::time::sleep(Duration::from_millis(3100)).await;
    let c = client();
    let r = c
        .post(format!("{base}/admin/login"))
        .json(&json!({"username": user, "password": password}))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200, "login {user}");
    c
}

async fn root(base: &str) -> Client {
    let c = login(base, "root", "hunter2").await;
    let r = c
        .post(format!("{base}/admin/step-up"))
        .json(&json!({"password": "hunter2"}))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    c
}

async fn health(c: &Client, base: &str, fresh: bool) -> Value {
    let started = Instant::now();
    let r = c
        .get(format!(
            "{base}/admin/dashboard/health{}",
            if fresh { "?fresh=1" } else { "" }
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    assert_eq!(r.headers()["cache-control"], "no-store");
    let v: Value = r.json().await.unwrap();
    // Every probe is bounded at 1.5 s and they run side by side.
    assert!(
        started.elapsed() < Duration::from_secs(3),
        "health took {:?}",
        started.elapsed()
    );
    v
}

/// Every component has a status from the four, a reason, and either measured
/// metrics or (disabled) how to enable it. Nothing says "did not report".
fn assert_complete(v: &Value) {
    for id in COMPONENTS {
        let c = &v["components"][id];
        let status = c["status"]
            .as_str()
            .unwrap_or_else(|| panic!("{id}: no status in {c}"));
        assert!(
            ["ok", "warn", "fail", "disabled"].contains(&status),
            "{id}: {status}"
        );
        let reason = c["reason"].as_str().unwrap();
        assert!(!reason.is_empty(), "{id}: empty reason");
        assert!(!reason.contains("did not report"), "{id}: {reason}");
        assert!(c["probeUs"].as_u64().is_some(), "{id}: no probe time");
        if status == "disabled" {
            assert!(
                c["enable"].as_str().is_some_and(|s| !s.is_empty()),
                "{id}: no enable hint"
            );
        } else {
            assert!(
                c["metrics"].as_object().is_some_and(|m| !m.is_empty()),
                "{id}: no metrics in {c}"
            );
        }
    }
}

#[tokio::test]
async fn proxy_mode_says_disabled_and_why_and_measures_the_rest() {
    let (base, _) = server(ServerMode::Proxy).await;
    assert_eq!(
        client()
            .get(format!("{base}/admin/dashboard/health"))
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    let c = root(&base).await;
    let v = health(&c, &base, true).await;
    assert_complete(&v);
    for id in ["sync", "outbox", "search", "plugins"] {
        let comp = &v["components"][id];
        assert_eq!(comp["status"], "disabled", "{id}: {comp}");
        assert!(
            comp["reason"].as_str().unwrap().starts_with("Proxy mode"),
            "{id}"
        );
        assert!(
            comp["enable"].as_str().unwrap().contains("MW_MODE=engine"),
            "{id}"
        );
    }
    let db = &v["components"]["database"]["metrics"];
    assert!(db["pingMs"].as_f64().unwrap() >= 0.0);
    assert_eq!(db["backend"], "sqlite");
    assert!(db["poolSize"].as_u64().unwrap() >= 1);
    assert!(db["dbBytes"].as_u64().unwrap() > 0);
    assert!(db["slowQueriesLastHour"].is_null());
    assert!(v["components"]["database"]["notes"]["slowQueriesLastHour"].is_string());
    let mig = &v["components"]["migrations"];
    assert_eq!(mig["status"], "ok");
    assert_eq!(mig["metrics"]["applied"], mig["metrics"]["expected"]);
    assert_eq!(mig["metrics"]["pending"], json!([]));
    assert_eq!(
        mig["metrics"]["latest"]["version"],
        mig["metrics"]["latestExpected"]["version"]
    );
    // The dashboard summary carries the same panel.
    let dash: Value = c
        .get(format!("{base}/admin/dashboard"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_complete(&dash["health"]);
    assert_eq!(
        dash["health"]["migration"]["applied"],
        mig["metrics"]["applied"]
    );
}

#[tokio::test]
async fn engine_mode_measures_every_component_and_counts_the_outbox() {
    let (base, db) = server(ServerMode::Engine).await;
    // Seed one account and a submission in every state, written by the store's
    // own schema (no message body is needed to count them).
    let pool = sqlx::SqlitePool::connect(&format!("sqlite://{}", db.display()))
        .await
        .unwrap();
    sqlx::query("INSERT INTO accounts (id, kind, host, port, tls, username, sealed_creds) VALUES ('acc-1','imap','127.0.0.1',1,'none','a@example.test',x'00')")
        .execute(&pool)
        .await
        .unwrap();
    let old = "2020-01-01T00:00:00Z";
    let future = "2999-01-01T00:00:00Z";
    for (id, status, send_at, attempts, hold) in [
        ("s-queued", "pending", None, 0, None),
        ("s-deferred", "pending", None, 2, None),
        ("s-scheduled", "pending", Some(future), 0, None),
        ("s-held", "pending", None, 0, Some("manual")),
        ("s-sending", "sending:claim", None, 0, None),
        ("s-failed", "failed", None, 5, None),
        ("s-sent", "final", None, 0, None),
    ] {
        sqlx::query("INSERT INTO submissions (id, account_id, email_id, send_at, undo_status, hold_seconds, created_at, attempts, hold) VALUES (?1,'acc-1','e',?2,?3,0,?4,?5,?6)")
            .bind(id)
            .bind(send_at)
            .bind(status)
            .bind(old)
            .bind(attempts)
            .bind(hold)
            .execute(&pool)
            .await
            .unwrap();
    }
    let c = root(&base).await;

    // An enabled first-party plugin is listed with its version and limits.
    for (path, body) in [
        ("/admin/plugins", Some(json!({"id":"spam-rspamd"}))),
        ("/admin/plugins/spam-rspamd/approve", None),
        ("/admin/plugins/spam-rspamd/enable", None),
    ] {
        let mut req = c.post(format!("{base}{path}"));
        if let Some(b) = body {
            req = req.json(&b);
        }
        let r = req.send().await.unwrap();
        assert!(
            r.status().is_success(),
            "{path}: {}",
            r.text().await.unwrap()
        );
    }

    let v = health(&c, &base, true).await;
    assert_complete(&v);
    for id in COMPONENTS {
        assert_ne!(
            v["components"][id]["status"], "disabled",
            "{id} in engine mode: {}",
            v["components"][id]
        );
    }

    let sync = &v["components"]["sync"];
    assert_eq!(sync["status"], "ok", "{sync}");
    assert_eq!(sync["metrics"]["accounts"], 0);
    assert_eq!(sync["metrics"]["errorsLastHour"], 0);

    let outbox = &v["components"]["outbox"]["metrics"];
    assert_eq!(outbox["queued"], 1, "{outbox}");
    assert_eq!(outbox["deferred"], 1);
    assert_eq!(outbox["scheduled"], 1);
    assert_eq!(outbox["held"], 1);
    assert_eq!(outbox["sending"], 1);
    assert_eq!(outbox["failed"], 1);
    assert!(outbox["oldestDueAgeSecs"].as_i64().unwrap() > 3600);
    assert_eq!(
        v["components"]["outbox"]["status"], "fail",
        "a message due for years is a failure"
    );
    assert_eq!(outbox["submission"]["configured"], 0);

    let search = &v["components"]["search"];
    assert_eq!(search["metrics"]["present"], true);
    assert_eq!(
        search["metrics"]["persistent"], true,
        "a file database puts the index beside it"
    );
    assert!(search["metrics"]["documents"].as_u64().is_some());
    assert!(
        search["metrics"]["bytes"].as_u64().is_some_and(|b| b > 0),
        "{search}"
    );
    assert!(
        search["metrics"]["lastCommitAgeSecs"].as_f64().is_some(),
        "{search}"
    );

    let plugins = &v["components"]["plugins"];
    assert_eq!(plugins["metrics"]["installed"], 1, "{plugins}");
    assert_eq!(plugins["metrics"]["enabled"], 1);
    let p = &plugins["metrics"]["plugins"][0];
    assert_eq!(p["id"], "spam-rspamd");
    assert!(p["version"].as_str().is_some_and(|s| !s.is_empty()));
    assert_eq!(p["limits"]["memoryMb"], 32);
    assert!(plugins["metrics"]["trapsLastHour"].as_u64().is_some());
    assert!(plugins["metrics"]["hostPingMs"].as_f64().is_some());

    let render = &v["components"]["render"];
    if render["metrics"]["workerConfigured"] == true {
        assert_eq!(render["status"], "ok", "{render}");
        assert_eq!(render["metrics"]["version"], env!("CARGO_PKG_VERSION"));
        assert!(render["metrics"]["pingMs"].as_f64().unwrap() > 0.0);
    } else {
        assert!(render["reason"].as_str().unwrap().contains("MW_RENDER_BIN"));
    }

    // Within the cache window the same result comes back; `fresh` re-probes.
    let again = health(&c, &base, false).await;
    assert_eq!(again["checkedAt"], v["checkedAt"]);
    assert!(!again["history"].as_array().unwrap().is_empty());

    // The numbers reach /metrics too.
    mw_server::observability::set_metrics_token(Some("t103-token".into()));
    let metrics = client()
        .get(format!("{base}/metrics"))
        .bearer_auth("t103-token")
        .send()
        .await
        .unwrap();
    assert_eq!(metrics.status(), 200);
    let text = metrics.text().await.unwrap();
    assert!(
        text.contains("mw_health_status{component=\"outbox\"} 2"),
        "{text}"
    );
    assert!(text.contains("mw_health_outbox_queued 1"), "{text}");
    assert!(text.contains("mw_health_database_ping_ms"), "{text}");
}

#[tokio::test]
async fn only_an_unscoped_dashboard_reader_sees_health() {
    let (base, _) = server(ServerMode::Proxy).await;
    let root = root(&base).await;
    let role: Value = root
        .post(format!("{base}/admin/roles"))
        .json(&json!({"name":"Dashboard reader","capabilities":["dashboard.read"]}))
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
        ("global", json!([{"roleId":role["id"]}])),
        ("nobody", json!([])),
    ] {
        let r = root
            .post(format!("{base}/admin/admins"))
            .json(&json!({"username":name,"password":"long test password","roles":roles}))
            .send()
            .await
            .unwrap();
        assert!(r.status().is_success(), "{}", r.text().await.unwrap());
    }
    for (name, expected) in [("scoped", 403), ("nobody", 403), ("global", 200)] {
        let c = login(&base, name, "long test password").await;
        let r = c
            .get(format!("{base}/admin/dashboard/health"))
            .send()
            .await
            .unwrap();
        assert_eq!(r.status(), expected, "{name}");
    }
}
