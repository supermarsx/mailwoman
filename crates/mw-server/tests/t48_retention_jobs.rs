//! t48 part 3 through the real router: the server creates one scheduled task
//! for each of `user_retention`, `retention_archive` and `gc_uploads`.
//!
//! * An installation that already has accounts gets a 7-day preview: a
//!   scheduled run before `armAfter` changes nothing and records "would
//!   change N"; after `armAfter` the same task deletes.
//! * Seeding runs again at every start and never adds a second task, and a
//!   task the administrator switched off or deleted stays that way.
//! * A fresh installation (no accounts) is armed at once.
//! * `GET/PUT /admin/retention/jobs` shows and switches the jobs.
use std::time::Duration;

use mw_server::jobs;
use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use mw_store::jobs::JobStatus;
use mw_store::{AccountKind, Credentials, FsUploadBackend, NewAccount, ServerKey, Store};
use serde_json::{Value, json};

mod common;

const KEY: &str = "4848484848484848484848484848484848484848484848484848484848484848";
const KINDS: [&str; 3] = ["user_retention", "retention_archive", "gc_uploads"];

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

async fn app(db: &std::path::Path, upstream: &str) -> String {
    let (app, _) = build_app_full(
        AppConfig {
            db_path: db.to_string_lossy().into(),
            server_key_hex: Some(KEY.into()),
            web_dir: None,
            cookie_secure: false,
            mode: ServerMode::Proxy,
            hardening: HardeningConfig::default(),
            security: SecurityConfig {
                jmap_upstreams: Some(vec![upstream.to_string()]),
                upstream_default: Some(upstream.to_string()),
                ..SecurityConfig::default()
            },
        },
        V6Config {
            admin_enabled: true,
            admin_username: Some("root".into()),
            admin_password: Some("retention-jobs-root".into()),
            redis_url: None,
        },
    )
    .await
    .unwrap();
    serve(app).await
}

async fn tasks_of(store: &Store, kind: &str) -> Vec<mw_store::jobs::ScheduledTaskRow> {
    store
        .list_scheduled_tasks()
        .await
        .unwrap()
        .into_iter()
        .filter(|t| t.kind == kind)
        .collect()
}

async fn wait_seeded(store: &Store) {
    wait_for("the three retention tasks", || async {
        let all = store.list_scheduled_tasks().await.unwrap();
        KINDS.iter().all(|k| all.iter().any(|t| t.kind == *k))
    })
    .await;
}

fn params(t: &mw_store::jobs::ScheduledTaskRow) -> Value {
    serde_json::from_str(&t.params_json).unwrap()
}

/// Make the task due now and wait for a finished run newer than `after`.
async fn run_due(store: &Store, kind: &str, after: Option<String>) -> mw_store::jobs::JobRunRow {
    let mut t = tasks_of(store, kind).await.remove(0);
    t.next_run_at = Some(chrono::Utc::now().timestamp() - 1);
    assert!(store.update_scheduled_task(&t).await.unwrap());
    wait_for("a scheduled run", || {
        let after = after.clone();
        async move {
            store.last_job_run(kind).await.unwrap().is_some_and(|r| {
                Some(&r.id) != after.as_ref() && r.status != JobStatus::Running
            })
        }
    })
    .await;
    store.last_job_run(kind).await.unwrap().unwrap()
}

#[tokio::test]
async fn existing_install_previews_for_seven_days_then_deletes() {
    jobs::set_tick(Duration::from_millis(100));
    jobs::pause_all(false);
    let upstream = serve(mw_mock_jmap::router()).await;
    let db = common::test_db::unique_db_path("t48-retention-jobs");
    let root = common::test_db::unique_dir("t48-retention-jobs-uploads");
    let store = Store::open(db.to_str().unwrap(), ServerKey::from_hex(KEY).unwrap())
        .await
        .unwrap()
        .with_upload_backend(std::sync::Arc::new(FsUploadBackend::new(root.clone())));
    let account = store
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "imap.example.org",
                port: 993,
                tls: "implicit",
                username: "existing@example.org",
                sync_policy_json: "{}",
            },
            &Credentials {
                username: "existing@example.org".into(),
                password: "pw".into(),
            },
        )
        .await
        .unwrap();
    let upload = store.put_upload(&account, "text/plain", b"stale").await.unwrap();
    let pool = sqlx::SqlitePool::connect(&format!("sqlite://{}", db.display()))
        .await
        .unwrap();
    sqlx::query("UPDATE uploaded_blobs SET created_at = ?1 WHERE blob_id = ?2")
        .bind((chrono::Utc::now() - chrono::Duration::days(40)).to_rfc3339())
        .bind(&upload)
        .execute(&pool)
        .await
        .unwrap();

    let before = chrono::Utc::now().timestamp();
    let base = app(&db, &upstream).await;
    wait_seeded(&store).await;
    for kind in KINDS {
        let t = tasks_of(&store, kind).await;
        assert_eq!(t.len(), 1, "{kind}");
        assert!(t[0].enabled, "{kind}");
        assert_eq!(t[0].created_by, "system");
        let arm = params(&t[0])["armAfter"].as_i64().expect("armAfter on an install with accounts");
        assert!(arm >= before + 7 * 86_400 && arm <= before + 7 * 86_400 + 120, "{kind} {arm}");
    }
    let gc = tasks_of(&store, "gc_uploads").await.remove(0);
    assert_eq!(params(&gc)["olderThanHours"], 720);

    // Before armAfter: a scheduled run only counts.
    let preview = run_due(&store, "gc_uploads", None).await;
    assert_eq!(preview.status, JobStatus::Ok, "{preview:?}");
    let detail: Value = serde_json::from_str(preview.detail_json.as_deref().unwrap()).unwrap();
    assert_eq!(detail["dryRun"], true, "{detail}");
    assert_eq!(detail["wouldChange"], 1, "{detail}");
    assert!(preview.summary.as_deref().unwrap().contains("would change 1"), "{preview:?}");
    assert!(store.get_upload(&account, &upload).await.unwrap().is_some(), "preview kept the upload");

    // The console shows the switch, the preview and the arm date.
    let admin = client();
    assert_eq!(
        admin
            .post(format!("{base}/admin/login"))
            .json(&json!({"username":"root","password":"retention-jobs-root"}))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    let listed: Value = admin
        .get(format!("{base}/admin/retention/jobs"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let job = listed["jobs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|j| j["kind"] == "gc_uploads")
        .unwrap()
        .clone();
    assert_eq!(job["enabled"], true, "{job}");
    assert_eq!(job["previewOnly"], true, "{job}");
    assert_eq!(job["lastPreview"]["wouldChange"], 1, "{job}");
    assert_eq!(job["armAfter"], params(&gc)["armAfter"], "{job}");

    // After armAfter the same task deletes.
    let mut t = tasks_of(&store, "gc_uploads").await.remove(0);
    let mut p = params(&t);
    p["armAfter"] = json!(chrono::Utc::now().timestamp() - 1);
    t.params_json = p.to_string();
    assert!(store.update_scheduled_task(&t).await.unwrap());
    let armed = run_due(&store, "gc_uploads", Some(preview.id.clone())).await;
    assert_eq!(armed.status, JobStatus::Ok, "{armed:?}");
    let detail: Value = serde_json::from_str(armed.detail_json.as_deref().unwrap()).unwrap();
    assert_eq!(detail["removed"], 1, "{detail}");
    assert!(store.get_upload(&account, &upload).await.unwrap().is_none(), "armed run removed the upload");

    // Switch off through the console; unknown kinds are 404.
    let off: Value = admin
        .put(format!("{base}/admin/retention/jobs/user_retention"))
        .json(&json!({"enabled": false}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(off["enabled"], false, "{off}");
    assert_eq!(
        admin
            .put(format!("{base}/admin/retention/jobs/vacuum"))
            .json(&json!({"enabled": true}))
            .send()
            .await
            .unwrap()
            .status(),
        404
    );
    // Delete another task by hand, then start the server again: seeding adds
    // nothing, the switched-off task stays off and the deleted one stays gone.
    let archive = tasks_of(&store, "retention_archive").await.remove(0);
    assert!(store.delete_scheduled_task(&archive.id).await.unwrap());
    let _again = app(&db, &upstream).await;
    tokio::time::sleep(Duration::from_millis(1500)).await;
    let ur = tasks_of(&store, "user_retention").await;
    assert_eq!(ur.len(), 1);
    assert!(!ur[0].enabled, "a switched-off task is not switched back on");
    assert!(tasks_of(&store, "retention_archive").await.is_empty(), "a deleted task is not recreated");
    assert_eq!(tasks_of(&store, "gc_uploads").await.len(), 1);

    // Switching a deleted job on creates its task again, with the preview.
    let on: Value = admin
        .put(format!("{base}/admin/retention/jobs/retention_archive"))
        .json(&json!({"enabled": true}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(on["enabled"], true, "{on}");
    assert_eq!(on["previewOnly"], true, "{on}");
    let _ = std::fs::remove_dir_all(&root);
}

#[tokio::test]
async fn fresh_install_is_armed_at_once() {
    jobs::set_tick(Duration::from_millis(100));
    jobs::pause_all(false);
    let upstream = serve(mw_mock_jmap::router()).await;
    let db = common::test_db::unique_db_path("t48-retention-jobs-fresh");
    let store = Store::open(db.to_str().unwrap(), ServerKey::from_hex(KEY).unwrap())
        .await
        .unwrap();
    let _base = app(&db, &upstream).await;
    wait_seeded(&store).await;
    for kind in KINDS {
        let t = tasks_of(&store, kind).await;
        assert_eq!(t.len(), 1, "{kind}");
        assert!(params(&t[0]).get("armAfter").is_none(), "{kind}: {}", t[0].params_json);
    }
}
