//! Audit-log retention and pruning on SQLite and, when `DATABASE_URL_PG` or
//! `MW_TEST_PG` is set, on a live PostgreSQL (t39 L5).
use mw_store::audit_identity::retention::AuditRetentionRow;
use mw_store::{AuditRow, ServerKey, Store};

fn key() -> ServerKey {
    ServerKey::from_bytes(&[9u8; 32]).unwrap()
}

fn ts(now: i64, days_ago: i64) -> String {
    chrono::DateTime::from_timestamp(now - days_ago * 86_400, 0)
        .unwrap()
        .to_rfc3339()
}

async fn row(s: &Store, id: &str, at: String, category: Option<&str>) {
    let r = AuditRow {
        id: id.into(),
        ts: at,
        actor: "root".into(),
        actor_kind: "admin".into(),
        action: "x".into(),
        target: Some("account:someone".into()),
        detail_json: "{}".into(),
        ip: None,
    };
    match category {
        Some(c) => s.append_categorized_audit(&r, c, None).await.unwrap(),
        None => s.append_audit(&r).await.unwrap(),
    }
}

fn policy(category: &str, days: Option<i64>, hold: bool) -> AuditRetentionRow {
    AuditRetentionRow {
        category: category.into(),
        days,
        legal_hold: hold,
        updated_by: "root".into(),
        updated_at: 1,
    }
}

async fn ids(s: &Store) -> Vec<String> {
    let mut v: Vec<String> = s
        .list_audit(1000)
        .await
        .unwrap()
        .into_iter()
        .map(|r| r.id)
        .collect();
    v.sort();
    v
}

async fn scenario(s: &Store) {
    let now = 1_800_000_000;
    // Nothing set: nothing is pruned.
    row(s, "admin-old", ts(now, 400), None).await;
    row(s, "admin-new", ts(now, 10), None).await;
    row(s, "sec-old", ts(now, 100), Some("security")).await;
    row(s, "alert-old", ts(now, 100), Some("alert")).await;
    row(s, "other-old", ts(now, 100), Some("future")).await;
    assert_eq!(s.prune_audit_log(now).await.unwrap().total, 0);
    assert!(s.last_audit_prune().await.unwrap().is_none());
    // Bounds and unknown categories are refused and write nothing.
    for bad in [Some(0), Some(29), Some(3651)] {
        assert!(
            !s.put_audit_retention(&policy("*", bad, false))
                .await
                .unwrap()
        );
    }
    assert!(
        !s.put_audit_retention(&policy("nope", Some(30), false))
            .await
            .unwrap()
    );
    assert!(s.audit_retention().await.unwrap().is_empty());
    // Default 365 days, security keeps 30, alerts forever.
    assert!(
        s.put_audit_retention(&policy("*", Some(365), false))
            .await
            .unwrap()
    );
    assert!(
        s.put_audit_retention(&policy("security", Some(30), false))
            .await
            .unwrap()
    );
    assert!(
        s.put_audit_retention(&policy("alert", None, false))
            .await
            .unwrap()
    );
    let report = s.prune_audit_log(now).await.unwrap();
    assert_eq!(report.total, 2, "{report:?}");
    assert_eq!(
        report.removed,
        vec![("admin".to_string(), 1), ("security".to_string(), 1)]
    );
    assert_eq!(ids(s).await, ["admin-new", "alert-old", "other-old"]);
    let cp = s.last_audit_prune().await.unwrap().unwrap();
    assert_eq!(Some(cp.id), report.checkpoint_id);
    assert_eq!(cp.ts, now);
    // A hold on a category keeps it; a hold on the default keeps everything.
    assert!(
        s.put_audit_retention(&policy("alert", Some(30), true))
            .await
            .unwrap()
    );
    assert!(
        s.put_audit_retention(&policy("*", Some(30), true))
            .await
            .unwrap()
    );
    assert_eq!(s.prune_audit_log(now).await.unwrap().total, 0);
    assert!(
        s.put_audit_retention(&policy("*", Some(30), false))
            .await
            .unwrap()
    );
    let report = s.prune_audit_log(now).await.unwrap();
    assert_eq!(report.removed, vec![("other".to_string(), 1)]);
    assert_eq!(ids(s).await, ["admin-new", "alert-old"]);
    // Removing the category row returns it to the default.
    assert!(s.delete_audit_retention("alert").await.unwrap());
    assert!(!s.delete_audit_retention("alert").await.unwrap());
    assert_eq!(s.prune_audit_log(now).await.unwrap().total, 1);
    assert_eq!(ids(s).await, ["admin-new"]);
    let rows = s.audit_retention().await.unwrap();
    assert_eq!(
        rows.iter().map(|r| r.category.as_str()).collect::<Vec<_>>(),
        ["*", "security"]
    );
    // More rows than one batch are all removed.
    for i in 0..4500 {
        row(s, &format!("bulk-{i:05}"), ts(now, 60), None).await;
    }
    let report = s.prune_audit_log(now).await.unwrap();
    assert_eq!(report.removed, vec![("admin".to_string(), 4500)]);
    assert_eq!(ids(s).await, ["admin-new"]);
}

#[tokio::test]
async fn sqlite_audit_retention_prunes_by_category_default_and_hold() {
    let s = Store::open_in_memory(key()).await.unwrap();
    scenario(&s).await;
}

#[tokio::test]
async fn postgres_audit_retention_prunes_by_category_default_and_hold() {
    let Some(dsn) = std::env::var("DATABASE_URL_PG")
        .ok()
        .or_else(|| std::env::var("MW_TEST_PG").ok())
        .filter(|s| !s.trim().is_empty())
    else {
        eprintln!(
            "[mw-store] t39 audit retention: DATABASE_URL_PG and MW_TEST_PG unset; the \
             PostgreSQL path was not driven."
        );
        return;
    };
    let s = Store::open_postgres(&dsn, key()).await.unwrap();
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(&dsn)
        .await
        .unwrap();
    sqlx::query("TRUNCATE audit_log, audit_retention, audit_checkpoint")
        .execute(&pool)
        .await
        .unwrap();
    scenario(&s).await;
    eprintln!("[mw-store] t39 audit retention: PostgreSQL path RAN.");
}
