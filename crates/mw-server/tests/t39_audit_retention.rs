//! t39 L5 through the real router: admin audit retention (routes, bounds, the
//! scheduled `audit_prune` job, the prune audit entry), auth-log actor links,
//! and sign-in enumeration parity (an existing and an invented name get the
//! same answer and the same history row).
use std::time::Duration;

use mw_server::jobs;
use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use mw_store::jobs::JobStatus;
use mw_store::{AuditRow, ServerKey, Store};
use serde_json::{Value, json};

mod common;

const KEY: &str = "3333333333333333333333333333333333333333333333333333333333333333";

async fn serve(app: axum::Router) -> String {
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
    base
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .no_proxy()
        .cookie_store(true)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap()
}

async fn wait_for<F, Fut>(what: &str, mut f: F)
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    for _ in 0..300 {
        if f().await {
            return;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    panic!("timed out waiting for {what}");
}

/// The sign-in pacing (`signin.retryCooldownSecs`, 3 s per source) charges every
/// failed attempt; wait it out so each attempt is answered on its merits.
async fn paced() {
    tokio::time::sleep(Duration::from_millis(3200)).await;
}

/// The parts of a response a client can tell apart.
async fn observable(response: reqwest::Response) -> (u16, Option<String>, String) {
    let status = response.status().as_u16();
    let kind = response
        .headers()
        .get("content-type")
        .map(|v| v.to_str().unwrap().to_string());
    (status, kind, response.text().await.unwrap())
}

#[tokio::test]
async fn audit_retention_prunes_on_schedule_links_actors_and_does_not_enumerate() {
    jobs::set_tick(Duration::from_millis(100));
    jobs::pause_all(false);
    let upstream = serve(mw_mock_jmap::router()).await;
    let db = common::test_db::unique_db_path("t39-audit-retention");
    let store = Store::open(db.to_str().unwrap(), ServerKey::from_hex(KEY).unwrap())
        .await
        .unwrap();
    let (app, _) = build_app_full(
        AppConfig {
            db_path: db.to_string_lossy().into(),
            server_key_hex: Some(KEY.into()),
            web_dir: None,
            cookie_secure: false,
            mode: ServerMode::Proxy,
            hardening: HardeningConfig::default(),
            security: SecurityConfig {
                jmap_upstreams: Some(vec![upstream.clone()]),
                upstream_default: Some(upstream),
                ..SecurityConfig::default()
            },
        },
        V6Config {
            admin_enabled: true,
            admin_username: Some("root".into()),
            admin_password: Some("retention-root-pass".into()),
            redis_url: None,
        },
    )
    .await
    .unwrap();
    let base = serve(app).await;

    // ── Enumeration parity: an existing and an invented administrator name. ──
    let mut answers = Vec::new();
    for username in ["root", "no-such-admin"] {
        paced().await;
        answers.push(
            observable(
                client()
                    .post(format!("{base}/admin/login"))
                    .json(&json!({"username":username,"password":"wrong-password-value"}))
                    .send()
                    .await
                    .unwrap(),
            )
            .await,
        );
    }
    assert_eq!(answers[0].0, 401);
    assert_eq!(
        answers[0], answers[1],
        "admin sign-in must not tell names apart"
    );
    // The same for the webmail sign-in (the mock upstream knows USER).
    let mut answers = Vec::new();
    for username in [mw_mock_jmap::USER, "nobody@example.invalid"] {
        paced().await;
        answers.push(
            observable(
                client()
                    .post(format!("{base}/api/login"))
                    .json(&json!({"username":username,"password":"wrong-password-value"}))
                    .send()
                    .await
                    .unwrap(),
            )
            .await,
        );
    }
    assert!(answers[0].0 >= 400, "{answers:?}");
    assert_eq!(
        answers[0], answers[1],
        "webmail sign-in must not tell names apart"
    );

    paced().await;
    let admin = client();
    assert_eq!(
        admin
            .post(format!("{base}/admin/login"))
            .json(&json!({"username":"root","password":"retention-root-pass"}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    paced().await;
    let user = client();
    assert_eq!(
        user.post(format!("{base}/api/login"))
            .json(&json!({"username":mw_mock_jmap::USER,"password":mw_mock_jmap::PASS}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );

    // ── Auth-log actor links. ──
    let history: Value = admin
        .get(format!("{base}/admin/logs/auth?limit=100"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let events = history["events"].as_array().unwrap();
    let failed: Vec<&Value> = events.iter().filter(|e| e["result"] == "failure").collect();
    assert!(failed.len() >= 4, "{events:?}");
    for e in &failed {
        assert!(e["accountId"].is_null(), "{e}");
        assert!(
            e.get("actorLabel").is_none() && e.get("actorLink").is_none(),
            "{e}"
        );
    }
    let shape = |e: &Value| {
        let mut keys: Vec<String> = e.as_object().unwrap().keys().cloned().collect();
        keys.sort();
        keys
    };
    assert!(
        failed.iter().all(|e| shape(e) == shape(failed[0])),
        "failed rows for existing and invented names have the same fields"
    );
    let admin_ok = events
        .iter()
        .find(|e| e["surface"] == "admin" && e["result"] == "success")
        .unwrap();
    assert_eq!(admin_ok["actorLink"]["section"], "admins", "{admin_ok}");
    assert_eq!(admin_ok["actorLabel"], "root");
    let webmail_ok = events
        .iter()
        .find(|e| e["surface"] == "webmail" && e["result"] == "success")
        .unwrap();
    assert_eq!(webmail_ok["actorLink"]["section"], "users", "{webmail_ok}");
    assert_eq!(webmail_ok["actorLink"]["id"], webmail_ok["accountId"]);
    assert!(
        webmail_ok["actorLabel"]
            .as_str()
            .unwrap()
            .contains(mw_mock_jmap::USER),
        "{webmail_ok}"
    );

    // ── Retention: nothing set, nothing scheduled. ──
    let now = chrono::Utc::now();
    let old = (now - chrono::Duration::days(100)).to_rfc3339();
    for (id, category) in [
        ("t39-old-admin", None),
        ("t39-old-security", Some("security")),
    ] {
        let row = AuditRow {
            id: id.into(),
            ts: old.clone(),
            actor: "root".into(),
            actor_kind: "admin".into(),
            action: "fixture".into(),
            target: None,
            detail_json: "{}".into(),
            ip: None,
        };
        match category {
            Some(c) => store.append_categorized_audit(&row, c, None).await.unwrap(),
            None => store.append_audit(&row).await.unwrap(),
        }
    }
    let retention: Value = admin
        .get(format!("{base}/admin/audit/retention"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(retention["rows"], json!([]));
    assert!(retention["task"].is_null());
    assert_eq!(retention["bounds"], json!({"minDays":30,"maxDays":3650}));
    assert_eq!(
        retention["categories"],
        json!(["*", "admin", "security", "alert"])
    );
    let pruned: Value = admin
        .post(format!("{base}/admin/audit/retention/prune"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(pruned["report"]["total"], 0, "no policy keeps everything");

    // Bounds, unknown categories, unknown fields.
    for (path, body, status) in [
        ("*", json!({"days":29}), 422),
        ("*", json!({"days":3651}), 422),
        ("*", json!({"days":30,"extra":1}), 422),
        ("nope", json!({"days":30}), 404),
    ] {
        assert_eq!(
            admin
                .put(format!("{base}/admin/audit/retention/{path}"))
                .json(&body)
                .send()
                .await
                .unwrap()
                .status(),
            status,
            "{path} {body}"
        );
    }
    // Security keeps everything; the default keeps 30 days.
    for (path, body) in [
        ("security", json!({"days":null,"legalHold":false})),
        ("*", json!({"days":30})),
    ] {
        let response = admin
            .put(format!("{base}/admin/audit/retention/{path}"))
            .json(&body)
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), 200, "{path}");
    }
    let retention: Value = admin
        .get(format!("{base}/admin/audit/retention"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(retention["task"]["enabled"], true, "{retention}");
    assert_eq!(retention["rows"][0]["category"], "*");
    assert_eq!(retention["rows"][0]["days"], 30);
    assert_eq!(retention["rows"][0]["updatedBy"], "root");

    // The scheduler runs the task once it is due.
    let task_id = retention["task"]["id"].as_str().unwrap().to_string();
    let mut task = store.get_scheduled_task(&task_id).await.unwrap().unwrap();
    task.next_run_at = Some(now.timestamp() - 1);
    assert!(store.update_scheduled_task(&task).await.unwrap());
    wait_for("the scheduled audit prune", || async {
        store
            .last_job_run("audit_prune")
            .await
            .unwrap()
            .is_some_and(|r| r.status == JobStatus::Ok)
    })
    .await;
    let run = store.last_job_run("audit_prune").await.unwrap().unwrap();
    assert_eq!(run.summary.as_deref(), Some("audit entries removed: 1"));
    let audit: Vec<Value> = admin
        .get(format!("{base}/admin/audit?limit=200"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let ids: Vec<&str> = audit.iter().map(|e| e["id"].as_str().unwrap()).collect();
    assert!(
        !ids.contains(&"t39-old-admin"),
        "the old admin entry is pruned"
    );
    assert!(
        ids.contains(&"t39-old-security"),
        "security keeps every entry"
    );
    let entry = audit
        .iter()
        .find(|e| e["action"] == "audit-pruned")
        .expect("the prune is itself audited");
    assert!(
        entry["detailJson"]
            .as_str()
            .unwrap()
            .contains("\"admin\":1"),
        "{entry}"
    );
    assert!(
        audit
            .iter()
            .any(|e| e["action"] == "audit-retention-changed"),
        "the policy change is audited"
    );
    let retention: Value = admin
        .get(format!("{base}/admin/audit/retention"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(
        retention["lastPrune"]["ts"].as_i64().is_some(),
        "{retention}"
    );

    // Reset returns security to the default; prune now removes it.
    let response = admin
        .delete(format!("{base}/admin/audit/retention/security"))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let pruned: Value = admin
        .post(format!("{base}/admin/audit/retention/prune"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(pruned["report"]["removed"]["security"], 1, "{pruned}");

    // Signed out: refused.
    assert_eq!(
        client()
            .get(format!("{base}/admin/audit/retention"))
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
}
