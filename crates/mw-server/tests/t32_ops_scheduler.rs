//! t32-e72 (worker f12) — the scheduler and its DB lease, over real apps.
//!
//! Each test boots one or two real apps (`build_app_full`) on one SQLite file and
//! drives the scheduler through the rows an administrator's API writes
//! (`scheduled_task`), opened next to the apps with the same key. Job kinds used
//! here are registered with `jobs::register` and count their runs.
//!
//! * Two processes (two apps) on one database run a due task once.
//! * A lease another process holds and renews is respected; an expired lease and
//!   a run whose heartbeat stopped are taken over (the old row becomes
//!   `abandoned`, a new run happens once).
//! * A run whose row was marked abandoned elsewhere is stopped.
//! * A running row of the kind blocks a second run (one run per kind).
//! * `only_in_window` skips a due task when no maintenance window is active.
//! * The history records instance, trigger, status and summary; the task records
//!   its last run, last status and next run.
//!
//! SQLite only: the store methods (lease claim, one run per kind under concurrent
//! starts) are run on Postgres by `mw-store`'s `jobs::tests::postgres_legs`.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use chrono::Utc;
use mw_server::jobs::{self, JobOutput, JobSpec};
use mw_server::{AppConfig, HardeningConfig, SecurityConfig, ServerMode, V6Config, build_app_full};
use mw_store::jobs::{JobRunQuery, JobRunRow, JobStatus, ScheduledTaskRow};
use mw_store::{ServerKey, Store};
use serde_json::json;

mod common;
use common::test_db;

const KEY_HEX: &str = "5a69788796a5b4c3d2e1f00f1e2d3c4b5a69788796a5b4c3d2e1f00f1e2d3c4b";

fn config(db: &str, upstream: String) -> AppConfig {
    let web: PathBuf = PathBuf::from(db).parent().unwrap().join("web");
    std::fs::create_dir_all(&web).unwrap();
    std::fs::write(web.join("index.html"), "<!doctype html><title>MW</title>").unwrap();
    AppConfig {
        db_path: db.to_string(),
        server_key_hex: Some(KEY_HEX.to_string()),
        web_dir: Some(web),
        cookie_secure: false,
        mode: ServerMode::Proxy,
        hardening: HardeningConfig::default(),
        security: SecurityConfig {
            jmap_upstreams: Some(vec![upstream.clone()]),
            upstream_default: Some(upstream),
            ..SecurityConfig::default()
        },
    }
}

async fn spawn_mock_jmap() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, mw_mock_jmap::router()).await.unwrap();
    });
    format!("http://{addr}")
}

fn new_db(tag: &str) -> String {
    test_db::unique_dir(tag)
        .join("mw.db")
        .to_string_lossy()
        .into_owned()
}

/// Boot `n` apps on `db` (their schedulers start with them) and open the store.
async fn apps(db: &str, n: usize) -> (Vec<axum::Router>, Store) {
    jobs::set_tick(Duration::from_millis(100));
    jobs::pause_all(false);
    let upstream = spawn_mock_jmap().await;
    let mut out = Vec::new();
    for _ in 0..n {
        out.push(
            build_app_full(config(db, upstream.clone()), V6Config::default())
                .await
                .expect("app boots")
                .0,
        );
    }
    let store = Store::open(db, ServerKey::from_hex(KEY_HEX).unwrap())
        .await
        .unwrap();
    (out, store)
}

fn task(id: &str, kind: &str, next: i64) -> ScheduledTaskRow {
    let now = Utc::now().timestamp();
    ScheduledTaskRow {
        id: id.into(),
        kind: kind.into(),
        schedule_json: json!({"everySecs": 3600}).to_string(),
        params_json: json!({"n": 1}).to_string(),
        enabled: true,
        only_in_window: false,
        created_by: "root".into(),
        created_at: now,
        updated_at: now,
        last_run_at: None,
        last_status: None,
        next_run_at: Some(next),
        lease_owner: None,
        lease_until: None,
    }
}

async fn runs_of(store: &Store, kind: &str) -> Vec<JobRunRow> {
    store
        .list_job_runs(&JobRunQuery {
            kind: Some(kind.into()),
            limit: 100,
            ..Default::default()
        })
        .await
        .unwrap()
}

/// Poll until `f` holds (10 s).
async fn wait_for<F, Fut>(what: &str, mut f: F)
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    for _ in 0..200 {
        if f().await {
            return;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    panic!("timed out waiting for {what}");
}

static ONCE_RUNS: AtomicUsize = AtomicUsize::new(0);

#[tokio::test]
async fn two_processes_on_one_database_run_a_due_task_once() {
    jobs::register(
        "t_once",
        JobSpec::new(|ctx| {
            Box::pin(async move {
                ONCE_RUNS.fetch_add(1, Ordering::SeqCst);
                // Long enough for both schedulers to see the task due several times.
                tokio::time::sleep(Duration::from_millis(600)).await;
                Ok(JobOutput::new(
                    "counted",
                    json!({ "params": ctx.params, "by": ctx.triggered_by }),
                ))
            })
        }),
    );
    let db = new_db("t32-ops-sched-once");
    let (_apps, store) = apps(&db, 2).await;
    let now = Utc::now().timestamp();
    store
        .insert_scheduled_task(&task("once", "t_once", now - 1))
        .await
        .unwrap();
    wait_for("the run to finish", || async {
        runs_of(&store, "t_once")
            .await
            .iter()
            .any(|r| r.status == JobStatus::Ok)
    })
    .await;
    // Several more passes of both schedulers.
    tokio::time::sleep(Duration::from_millis(800)).await;
    assert_eq!(
        ONCE_RUNS.load(Ordering::SeqCst),
        1,
        "the task ran more than once"
    );
    let runs = runs_of(&store, "t_once").await;
    assert_eq!(runs.len(), 1, "{runs:?}");
    let r = &runs[0];
    assert_eq!(r.task_id.as_deref(), Some("once"));
    assert_eq!(r.triggered_by, "scheduler");
    assert_eq!(r.summary.as_deref(), Some("counted"));
    assert!(
        r.instance.contains('@'),
        "instance = host@start: {}",
        r.instance
    );
    let detail: serde_json::Value =
        serde_json::from_str(r.detail_json.as_deref().unwrap()).unwrap();
    assert_eq!(detail["params"], json!({"n": 1}));
    let t = store.get_scheduled_task("once").await.unwrap().unwrap();
    assert_eq!(t.last_status.as_deref(), Some("ok"));
    assert_eq!((t.lease_owner, t.lease_until), (None, None));
    let next = t.next_run_at.unwrap();
    assert!(
        next >= now + 3590,
        "next run an hour later: {next} vs {now}"
    );
}

static TAKEOVER_RUNS: AtomicUsize = AtomicUsize::new(0);

#[tokio::test]
async fn a_live_lease_is_respected_and_an_expired_one_is_taken_over() {
    jobs::register(
        "t_takeover",
        JobSpec::new(|_| {
            Box::pin(async {
                TAKEOVER_RUNS.fetch_add(1, Ordering::SeqCst);
                Ok(JobOutput::new("taken over", json!({})))
            })
        })
        .lease(Duration::from_secs(2)),
    );
    let db = new_db("t32-ops-sched-takeover");
    let (_apps, store) = apps(&db, 1).await;
    let now = Utc::now().timestamp();
    // Another process holds a live lease and its run's heartbeat is fresh.
    let mut t = task("tk", "t_takeover", now - 1);
    t.lease_owner = Some("other-host@1".into());
    t.lease_until = Some(now + 3);
    store.insert_scheduled_task(&t).await.unwrap();
    let held = JobRunRow {
        id: "held-run".into(),
        task_id: Some("tk".into()),
        kind: "t_takeover".into(),
        instance: "other-host@1".into(),
        started_at: now - 1,
        heartbeat_at: now,
        finished_at: None,
        status: JobStatus::Running,
        summary: None,
        detail_json: None,
        triggered_by: "scheduler".into(),
    };
    assert!(store.start_job_run(&held).await.unwrap());
    tokio::time::sleep(Duration::from_millis(1200)).await;
    assert_eq!(
        TAKEOVER_RUNS.load(Ordering::SeqCst),
        0,
        "ran under a live lease"
    );
    // The other process dies: its lease expires (2 s) and its heartbeat ages past
    // 3x the lease (6 s). Then this process abandons the row and runs the task.
    wait_for("the abandoned row", || async {
        store
            .get_job_run("held-run")
            .await
            .unwrap()
            .is_some_and(|r| r.status == JobStatus::Abandoned)
    })
    .await;
    wait_for("the takeover run", || async {
        TAKEOVER_RUNS.load(Ordering::SeqCst) == 1
    })
    .await;
    wait_for("the takeover run's row", || async {
        runs_of(&store, "t_takeover")
            .await
            .iter()
            .any(|r| r.status == JobStatus::Ok && r.instance != "other-host@1")
    })
    .await;
    tokio::time::sleep(Duration::from_millis(500)).await;
    assert_eq!(TAKEOVER_RUNS.load(Ordering::SeqCst), 1);
    let held = store.get_job_run("held-run").await.unwrap().unwrap();
    assert!(held.finished_at.is_some());
}

static STOPPED_FINISHED: AtomicUsize = AtomicUsize::new(0);
static STOPPED_STARTED: AtomicUsize = AtomicUsize::new(0);

#[tokio::test]
async fn a_run_marked_abandoned_elsewhere_is_stopped() {
    jobs::register(
        "t_stopped",
        JobSpec::new(|_| {
            Box::pin(async {
                STOPPED_STARTED.fetch_add(1, Ordering::SeqCst);
                tokio::time::sleep(Duration::from_secs(5)).await;
                STOPPED_FINISHED.fetch_add(1, Ordering::SeqCst);
                Ok(JobOutput::new("finished", json!({})))
            })
        })
        .lease(Duration::from_secs(1)),
    );
    let db = new_db("t32-ops-sched-stopped");
    let (_apps, store) = apps(&db, 1).await;
    let now = Utc::now().timestamp();
    store
        .insert_scheduled_task(&task("st", "t_stopped", now - 1))
        .await
        .unwrap();
    wait_for("the run to start", || async {
        STOPPED_STARTED.load(Ordering::SeqCst) == 1
    })
    .await;
    let run = runs_of(&store, "t_stopped").await.remove(0);
    assert_eq!(run.status, JobStatus::Running);
    // Another process decides this run is dead.
    assert_eq!(
        store
            .abandon_stale_job_runs("t_stopped", i64::MAX, now)
            .await
            .unwrap(),
        1
    );
    tokio::time::sleep(Duration::from_millis(5500)).await;
    assert_eq!(
        STOPPED_FINISHED.load(Ordering::SeqCst),
        0,
        "the run kept going"
    );
    let run = store.get_job_run(&run.id).await.unwrap().unwrap();
    assert_eq!(run.status, JobStatus::Abandoned);
}

static BLOCKED_RUNS: AtomicUsize = AtomicUsize::new(0);

#[tokio::test]
async fn a_running_row_of_the_kind_blocks_a_second_run() {
    jobs::register(
        "t_blocked",
        JobSpec::new(|_| {
            Box::pin(async {
                BLOCKED_RUNS.fetch_add(1, Ordering::SeqCst);
                Ok(JobOutput::new("ran", json!({})))
            })
        }),
    );
    let db = new_db("t32-ops-sched-blocked");
    let (_apps, store) = apps(&db, 1).await;
    let now = Utc::now().timestamp();
    let manual = JobRunRow {
        id: "manual".into(),
        task_id: None,
        kind: "t_blocked".into(),
        instance: "elsewhere@1".into(),
        started_at: now,
        heartbeat_at: now,
        finished_at: None,
        status: JobStatus::Running,
        summary: None,
        detail_json: None,
        triggered_by: "root".into(),
    };
    assert!(store.start_job_run(&manual).await.unwrap());
    store
        .insert_scheduled_task(&task("bl", "t_blocked", now - 1))
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(800)).await;
    assert_eq!(BLOCKED_RUNS.load(Ordering::SeqCst), 0);
    // The task stays due and gave its lease back.
    let t = store.get_scheduled_task("bl").await.unwrap().unwrap();
    assert_eq!(t.next_run_at, Some(now - 1));
    assert_eq!(t.lease_owner, None);
    // When the other run finishes, the task runs.
    store
        .finish_job_run("manual", JobStatus::Ok, now + 1, None, None)
        .await
        .unwrap();
    wait_for("the task's run", || async {
        BLOCKED_RUNS.load(Ordering::SeqCst) == 1
    })
    .await;
}

static WINDOW_RUNS: AtomicUsize = AtomicUsize::new(0);

#[tokio::test]
async fn builtin_jobs_perform_store_work_and_report_engine_unavailable() {
    let db = new_db("t32-jobs-builtin");
    let (_apps, store) = apps(&db, 1).await;
    assert!(
        store.list_scheduled_tasks().await.unwrap().is_empty(),
        "nothing enabled by default"
    );
    let now = Utc::now().timestamp();
    for kind in ["vacuum", "gc_uploads", "reindex", "rethread_all"] {
        store
            .insert_scheduled_task(&task(kind, kind, now - 1))
            .await
            .unwrap();
    }
    wait_for("builtins", || async {
        let runs = store
            .list_job_runs(&JobRunQuery {
                limit: 10,
                ..Default::default()
            })
            .await
            .unwrap();
        runs.len() == 4 && runs.iter().all(|r| r.status != JobStatus::Running)
    })
    .await;
    for kind in ["vacuum", "gc_uploads"] {
        assert_eq!(
            store.last_job_run(kind).await.unwrap().unwrap().status,
            JobStatus::Ok
        );
    }
    for kind in ["reindex", "rethread_all"] {
        let r = store.last_job_run(kind).await.unwrap().unwrap();
        assert_eq!(r.status, JobStatus::Failed);
        assert!(r.summary.unwrap().contains("requires engine mode"));
    }
}

static LOST_LEASE_FINISHED: AtomicUsize = AtomicUsize::new(0);
#[tokio::test]
async fn losing_task_lease_cancels_work_and_cannot_release_new_owner() {
    jobs::register(
        "t_lease_lost",
        JobSpec::new(|_| {
            Box::pin(async {
                tokio::time::sleep(Duration::from_secs(4)).await;
                LOST_LEASE_FINISHED.fetch_add(1, Ordering::SeqCst);
                Ok(JobOutput::new("too late", json!({})))
            })
        })
        .lease(Duration::from_secs(1)),
    );
    let db = new_db("t32-jobs-lease-loss");
    let (_apps, store) = apps(&db, 1).await;
    let now = Utc::now().timestamp();
    store
        .insert_scheduled_task(&task("lost", "t_lease_lost", now - 1))
        .await
        .unwrap();
    wait_for("running", || async {
        runs_of(&store, "t_lease_lost")
            .await
            .iter()
            .any(|r| r.status == JobStatus::Running)
    })
    .await;
    assert!(
        store
            .claim_task_lease("lost", "replacement", now + 10, now + 100)
            .await
            .unwrap()
    );
    wait_for("lease cancellation", || async {
        runs_of(&store, "t_lease_lost")
            .await
            .iter()
            .any(|r| r.status == JobStatus::Cancelled)
    })
    .await;
    tokio::time::sleep(Duration::from_secs(4)).await;
    assert_eq!(LOST_LEASE_FINISHED.load(Ordering::SeqCst), 0);
    assert_eq!(
        store
            .get_scheduled_task("lost")
            .await
            .unwrap()
            .unwrap()
            .lease_owner
            .as_deref(),
        Some("replacement")
    );
}

static WINDOW_FINISHED: AtomicUsize = AtomicUsize::new(0);
#[tokio::test]
async fn persisted_global_window_allows_run_and_cancellation_stops_it() {
    jobs::register(
        "t_window_end",
        JobSpec::new(|_| {
            Box::pin(async {
                tokio::time::sleep(Duration::from_secs(4)).await;
                WINDOW_FINISHED.fetch_add(1, Ordering::SeqCst);
                Ok(JobOutput::new("too late", json!({})))
            })
        })
        .lease(Duration::from_secs(1)),
    );
    let db = new_db("t32-jobs-window-end");
    let (_apps, store) = apps(&db, 1).await;
    let pool =
        sqlx::SqlitePool::connect_with(sqlx::sqlite::SqliteConnectOptions::new().filename(&db))
            .await
            .unwrap();
    let now = Utc::now().timestamp();
    sqlx::query("INSERT INTO maintenance_window (id,mode,starts_at,ends_at,created_by,created_at,updated_by,updated_at) VALUES ('window','full',?1,?2,'root',?1,'root',?1)")
        .bind(now-1).bind(now+60).execute(&pool).await.unwrap();
    let mut t = task("window_task", "t_window_end", now - 1);
    t.only_in_window = true;
    store.insert_scheduled_task(&t).await.unwrap();
    wait_for("window run", || async {
        runs_of(&store, "t_window_end")
            .await
            .iter()
            .any(|r| r.status == JobStatus::Running)
    })
    .await;
    sqlx::query("UPDATE maintenance_window SET cancelled_at = ?1 WHERE id = 'window'")
        .bind(now)
        .execute(&pool)
        .await
        .unwrap();
    wait_for("window cancellation", || async {
        runs_of(&store, "t_window_end")
            .await
            .iter()
            .any(|r| r.status == JobStatus::Cancelled)
    })
    .await;
    tokio::time::sleep(Duration::from_secs(4)).await;
    assert_eq!(WINDOW_FINISHED.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn only_in_window_skips_a_due_task_outside_a_window() {
    jobs::register(
        "t_window",
        JobSpec::new(|_| {
            Box::pin(async {
                WINDOW_RUNS.fetch_add(1, Ordering::SeqCst);
                Ok(JobOutput::new("ran", json!({})))
            })
        }),
    );
    let db = new_db("t32-ops-sched-window");
    let (_apps, store) = apps(&db, 1).await;
    let now = Utc::now().timestamp();
    let mut t = task("win", "t_window", now - 1);
    t.only_in_window = true;
    store.insert_scheduled_task(&t).await.unwrap();
    wait_for("the skip", || async {
        store
            .get_scheduled_task("win")
            .await
            .unwrap()
            .is_some_and(|t| t.last_status.as_deref() == Some("skippedOutsideWindow"))
    })
    .await;
    assert_eq!(WINDOW_RUNS.load(Ordering::SeqCst), 0);
    let t = store.get_scheduled_task("win").await.unwrap().unwrap();
    assert!(t.next_run_at.unwrap() >= now + 3590);
    assert_eq!(t.last_run_at, None);
    assert!(runs_of(&store, "t_window").await.is_empty());
}

#[tokio::test]
async fn a_disabled_task_and_an_unscheduled_one_do_not_run() {
    jobs::register(
        "t_never",
        JobSpec::new(|_| Box::pin(async { Err("must not run".to_string()) })),
    );
    let db = new_db("t32-ops-sched-never");
    let (_apps, store) = apps(&db, 1).await;
    let now = Utc::now().timestamp();
    let mut off = task("off", "t_never", now - 1);
    off.enabled = false;
    store.insert_scheduled_task(&off).await.unwrap();
    // Enabled, no next run yet: the scheduler gives it its first run an hour away.
    let mut fresh = task("fresh", "t_never", 0);
    fresh.next_run_at = None;
    store.insert_scheduled_task(&fresh).await.unwrap();
    wait_for("the first run time", || async {
        store
            .get_scheduled_task("fresh")
            .await
            .unwrap()
            .is_some_and(|t| t.next_run_at.is_some())
    })
    .await;
    tokio::time::sleep(Duration::from_millis(500)).await;
    assert!(runs_of(&store, "t_never").await.is_empty());
    let fresh = store.get_scheduled_task("fresh").await.unwrap().unwrap();
    assert!(fresh.next_run_at.unwrap() >= now + 3590);
}
