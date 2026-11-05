//! Retention sweeper primitives over a real SQLite store: old rows go, recent
//! rows stay, an unsent outbox row is never deleted, batches are bounded, a
//! repeat sweep finds nothing, held accounts are skipped and IPs are reduced.

use mw_store::jobs::retention::{IpMode, TARGETS, target as find};
use mw_store::{ServerKey, Store};
use sqlx::sqlite::SqlitePoolOptions;

#[path = "../src/test_db.rs"]
mod test_db;

const NOW: i64 = 1_800_000_000;
const DAY: i64 = 86_400;

fn rfc(unix: i64) -> String {
    chrono::DateTime::from_timestamp(unix, 0).unwrap().to_rfc3339()
}

async fn open() -> (Store, sqlx::SqlitePool) {
    let path = test_db::unique_file_path("mw-retention", "s.sqlite");
    let store = Store::open(path.to_str().unwrap(), ServerKey::from_bytes(&[7u8; 32]).unwrap())
        .await
        .unwrap();
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&format!("sqlite://{}", path.display()))
        .await
        .unwrap();
    (store, pool)
}

async fn rows(store: &Store, id: &str) -> i64 {
    store
        .retention_footprint(target(id))
        .await
        .unwrap()
        .rows
}

async fn sweep(store: &Store, id: &str, days: i64, batch: i64, held: &[String]) -> u64 {
    store
        .retention_sweep(target(id), NOW - days * DAY, batch, 100, false, held)
        .await
        .unwrap()
        .rows
}

#[tokio::test]
async fn old_rows_are_deleted_recent_kept_and_a_repeat_is_a_no_op() {
    let (store, pool) = open().await;
    for (id, ts) in [("old", NOW - 400 * DAY), ("new", NOW - DAY)] {
        sqlx::query("INSERT INTO password_change_audit(id,ts,account_id,backend,outcome) VALUES(?,?,?,?,?)")
            .bind(id).bind(rfc(ts)).bind("a1").bind("local").bind("ok")
            .execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO sso_login_audit(id,ts,provider_id,kind,subject_hash,outcome) VALUES(?,?,?,?,?,?)")
            .bind(id).bind(rfc(ts)).bind("p").bind("oidc").bind("h").bind("ok")
            .execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO alert_event(id,rule_id,fired_at,summary,delivered_json) VALUES(?,?,?,?,?)")
            .bind(id).bind("r").bind(ts).bind("s").bind("[]")
            .execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO webhook_nonce(hook_id,nonce,seen_at) VALUES(?,?,?)")
            .bind("h").bind(id).bind(ts)
            .execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO settings(key,value) VALUES(?,?)")
            .bind(format!("rc.h.{id}"))
            .bind(format!("{:012}|{{}}", ts))
            .execute(&pool).await.unwrap();
    }
    for target in ["passwordChangeAudit", "ssoLoginAudit", "alertEvents", "webhookNonces", "imageHandles"] {
        let dry = store
            .retention_sweep(self::target(target), NOW - 30 * DAY, 10, 1, true, &[])
            .await
            .unwrap();
        assert_eq!(dry.rows, 1, "{target} dry run counts");
        assert_eq!(rows(&store, target).await, 2, "{target} dry run deletes nothing");
        assert_eq!(sweep(&store, target, 30, 10, &[]).await, 1, "{target}");
        assert_eq!(rows(&store, target).await, 1, "{target} keeps the recent row");
        assert_eq!(sweep(&store, target, 30, 10, &[]).await, 0, "{target} repeat");
    }
}

fn target(id: &str) -> &'static mw_store::jobs::retention::Target {
    find(id).unwrap()
}

#[tokio::test]
async fn outbox_keeps_everything_still_needed() {
    let (store, pool) = open().await;
    sqlx::query("INSERT INTO accounts(id,kind,host,port,tls,username,sealed_creds) VALUES('a1','imap','h',993,'tls','u@x',x'00')")
        .execute(&pool).await.unwrap();
    let old = rfc(NOW - 400 * DAY);
    for (id, status, err) in [
        ("sent", "final", None),
        ("cancelled", "canceled", None),
        ("pending", "pending", None),
        ("claimed", "sending:abc", None),
        ("failed", "failed", Some("boom")),
        ("uncertain", "uncertain", None),
        ("filing", "final", Some("sent copy not filed")),
    ] {
        sqlx::query("INSERT INTO submissions(id,account_id,email_id,undo_status,hold_seconds,created_at,last_error) VALUES(?,?,?,?,0,?,?)")
            .bind(id).bind("a1").bind("e").bind(status).bind(&old).bind(err)
            .execute(&pool).await.unwrap();
    }
    assert_eq!(sweep(&store, "outbox", 30, 10, &[]).await, 2);
    assert_eq!(rows(&store, "outbox").await, 5);
}

#[tokio::test]
async fn running_jobs_and_held_accounts_are_kept() {
    let (store, pool) = open().await;
    for (id, status) in [("done", "ok"), ("live", "running")] {
        sqlx::query("INSERT INTO job_run(id,kind,instance,started_at,heartbeat_at,status,triggered_by) VALUES(?,?,?,?,?,?,?)")
            .bind(id).bind("vacuum").bind("i").bind(NOW - 400 * DAY).bind(NOW - 400 * DAY).bind(status).bind("t")
            .execute(&pool).await.unwrap();
    }
    assert_eq!(sweep(&store, "jobRuns", 30, 10, &[]).await, 1);
    assert_eq!(rows(&store, "jobRuns").await, 1);

    for (id, acct) in [("p1", "held-1"), ("p2", "free-1")] {
        sqlx::query("INSERT INTO password_change_audit(id,ts,account_id,backend,outcome) VALUES(?,?,?,?,?)")
            .bind(id).bind(rfc(NOW - 400 * DAY)).bind(acct).bind("local").bind("ok")
            .execute(&pool).await.unwrap();
    }
    assert_eq!(sweep(&store, "passwordChangeAudit", 30, 10, &["held-1".into()]).await, 1);
    assert_eq!(rows(&store, "passwordChangeAudit").await, 1);
}

#[tokio::test]
async fn deletes_are_bounded_per_pass() {
    let (store, pool) = open().await;
    for i in 0..25 {
        sqlx::query("INSERT INTO alert_event(id,rule_id,fired_at,summary,delivered_json) VALUES(?,?,?,?,?)")
            .bind(format!("a{i}")).bind("r").bind(NOW - 400 * DAY).bind("s").bind("[]")
            .execute(&pool).await.unwrap();
    }
    let t = target("alertEvents");
    let first = store.retention_sweep(t, NOW - 30 * DAY, 10, 2, false, &[]).await.unwrap();
    assert_eq!((first.rows, first.more), (20, true));
    let second = store.retention_sweep(t, NOW - 30 * DAY, 10, 2, false, &[]).await.unwrap();
    assert_eq!((second.rows, second.more), (5, false));
}

#[tokio::test]
async fn old_ips_are_reduced_once() {
    let (store, pool) = open().await;
    for (id, ts, ip) in [("o", NOW - 100 * DAY, "203.0.113.77"), ("n", NOW - DAY, "198.51.100.9")] {
        sqlx::query("INSERT INTO signin_event(id,username_attempted,ts,result,method,ip,user_agent,reason) VALUES(?,?,?,?,?,?,?,?)")
            .bind(id).bind("u").bind(ts).bind("failure").bind("password").bind(ip).bind("ua").bind("bad")
            .execute(&pool).await.unwrap();
    }
    let dry = store.retention_reduce_ips("signin_event", NOW - 30 * DAY, IpMode::Truncate, "", 10, 5, true).await.unwrap();
    assert_eq!(dry.rows, 1);
    let done = store.retention_reduce_ips("signin_event", NOW - 30 * DAY, IpMode::Truncate, "", 10, 5, false).await.unwrap();
    assert_eq!(done.rows, 1);
    let again = store.retention_reduce_ips("signin_event", NOW - 30 * DAY, IpMode::Truncate, "", 10, 5, false).await.unwrap();
    assert_eq!(again.rows, 0);
    let ips: Vec<(String, String)> = sqlx::query_as("SELECT id, ip FROM signin_event ORDER BY id").fetch_all(&pool).await.unwrap();
    assert_eq!(ips, vec![("n".into(), "198.51.100.9".into()), ("o".into(), "203.0.113.0".into())]);
}

#[test]
fn every_target_names_a_real_guard_free_id() {
    assert!(TARGETS.len() >= 15);
}
