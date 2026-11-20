use super::*;
use crate::{AccountKind, Credentials, NewAccount, ServerKey};

async fn account(store: &Store) -> String {
    let name = format!("chunk-{}", uuid::Uuid::new_v4());
    store
        .create_account(
            &NewAccount {
                kind: AccountKind::Imap,
                host: "fixture.invalid",
                port: 993,
                tls: "implicit",
                username: &name,
                sync_policy_json: "{}",
            },
            &Credentials {
                username: name.clone(),
                password: "fixture".into(),
            },
        )
        .await
        .unwrap()
}
async fn age(store: &Store, account: &str) {
    q("UPDATE changes SET at='2000-01-01T00:00:00Z' WHERE account_id=?1")
        .bind(account)
        .execute(&store.backend)
        .await
        .unwrap();
}
async fn prune(store: &Store, account: &str, rows: u32) -> ChangeLogRetentionReport {
    let mut report = ChangeLogRetentionReport {
        at: chrono::Utc::now().timestamp(),
        cursor: ChangeLogRetentionCursor {
            account_id: account.into(),
            kind: String::new(),
        },
        ..Default::default()
    };
    collect(
        &store.backend,
        &ChangeLogRetentionOptions {
            batch_rows: rows,
            ..Default::default()
        },
        Instant::now() + Duration::from_secs(10),
        &mut report,
    )
    .await
    .unwrap();
    report
}
async fn clean(store: &Store, account: &str) {
    q("DELETE FROM accounts WHERE id=?1")
        .bind(account)
        .execute(&store.backend)
        .await
        .unwrap();
}

async fn large_contract(store: &Store) {
    let foreign = account(store).await;
    store
        .record_change(&foreign, "Email", "foreign-first", "updated")
        .await
        .unwrap();
    let a = account(store).await;
    let ids = (0..12001).map(|n| format!("batch-{n}")).collect::<Vec<_>>();
    store
        .record_changes(&a, "Email", &ids, "updated")
        .await
        .unwrap();
    store
        .record_change(&a, "Email", "later", "updated")
        .await
        .unwrap();
    age(store, &a).await;
    store
        .record_change(&foreign, "Email", "foreign-last", "updated")
        .await
        .unwrap();
    if let Backend::Postgres(pool) = &store.backend {
        use sqlx::Row as _;
        let ids=q("SELECT CAST(ctid AS TEXT) AS locator FROM changes WHERE account_id=?1 AND type='Email' LIMIT 64").bind(&a).fetch_all(&store.backend).await.unwrap().into_iter().map(|row|row.get_string("locator")).collect::<Vec<_>>();
        let sql =
            crate::dialect::placeholders(delete_chunk_sql(Dialect::Postgres), Dialect::Postgres);
        let plan = sqlx::query(&format!("EXPLAIN {sql}"))
            .bind(&a)
            .bind("Email")
            .bind(serde_json::to_string(&ids).unwrap())
            .fetch_all(pool)
            .await
            .unwrap()
            .into_iter()
            .map(|row| row.get::<String, _>(0))
            .collect::<Vec<_>>();
        assert!(
            plan.iter().filter(|line| line.contains("Tid Scan")).count() >= 2,
            "locator deletion must use bounded tuple lookups: {plan:?}"
        );
        assert!(
            !plan.iter().any(|line| line.contains("Seq Scan on changes")
                || line.contains("Index Scan using idx_changes")),
            "{plan:?}"
        );
    }
    if let Backend::Sqlite(pool) = &store.backend {
        use sqlx::Row as _;
        let pin = pool.pin().await;
        let ids=q("SELECT CAST(rowid AS TEXT) AS locator FROM changes WHERE account_id=?1 AND type='Email' LIMIT 64").bind(&a).fetch_all(&store.backend).await.unwrap().into_iter().map(|row|row.get_string("locator")).collect::<Vec<_>>();
        let plan = sqlx::query(&format!(
            "EXPLAIN QUERY PLAN {}",
            delete_chunk_sql(Dialect::Sqlite)
        ))
        .bind(&a)
        .bind("Email")
        .bind(serde_json::to_string(&ids).unwrap())
        .fetch_all(pin.pool())
        .await
        .unwrap()
        .into_iter()
        .map(|row| row.get::<String, _>("detail"))
        .collect::<Vec<_>>();
        assert!(plan.iter().any(|line| line.contains("rowid=?")), "{plan:?}");
        assert!(
            !plan.iter().any(|line| line.contains("SCAN changes")),
            "{plan:?}"
        );
    }
    let (first, queries) = crate::pool_telemetry::measure_queries(prune(store, &a, 10000)).await;
    assert_eq!(first.removed_rows, 10000);
    assert_eq!(first.chunked_states, 1);
    assert_eq!(
        queries.write.calls, 2,
        "a chunk must use a floor write and one set-based DELETE: {queries:?}"
    );
    assert!(matches!(
        store.changes_since(&a, "Email", 0).await,
        Err(StoreError::ChangeHistoryExpired)
    ));
    let delta = store.changes_since(&a, "Email", 1).await.unwrap();
    assert_eq!(delta.len(), 1);
    assert_eq!(delta[0].stable_id, "later");
    assert_eq!(store.current_state(&a, "Email").await.unwrap(), 2);
    assert_eq!(
        store
            .record_change(&a, "Email", "fresh", "updated")
            .await
            .unwrap(),
        3
    );
    assert_eq!(prune(store, &a, 10000).await.removed_rows, 2002);
    let delta = store.changes_since(&a, "Email", 2).await.unwrap();
    assert_eq!(delta.len(), 1);
    assert_eq!(delta[0].stable_id, "fresh");
    assert_eq!(store.current_state(&a, "Email").await.unwrap(), 3);
    assert_eq!(
        store
            .changes_since(&foreign, "Email", 0)
            .await
            .unwrap()
            .len(),
        2,
        "other accounts must remain untouched"
    );
    clean(store, &a).await;
    clean(store, &foreign).await;
}
async fn timestamp_contract(store: &Store) {
    for timestamp in ["invalid-timestamp", "2999-01-01T00:00:00Z"] {
        let a = account(store).await;
        store
            .record_changes(
                &a,
                "Email",
                &["old-a".into(), "bad".into(), "old-b".into()],
                "updated",
            )
            .await
            .unwrap();
        age(store, &a).await;
        q("UPDATE changes SET at=?1 WHERE account_id=?2 AND stable_id='bad'")
            .bind(timestamp)
            .bind(&a)
            .execute(&store.backend)
            .await
            .unwrap();
        assert_eq!(prune(store, &a, 10).await.removed_rows, 0);
        assert_eq!(store.changes_since(&a, "Email", 0).await.unwrap().len(), 3);
        clean(store, &a).await;
    }
    let a = account(store).await;
    store
        .record_changes(
            &a,
            "Email",
            &(0..6).map(|n| n.to_string()).collect::<Vec<_>>(),
            "updated",
        )
        .await
        .unwrap();
    age(store, &a).await;
    assert_eq!(prune(store, &a, 2).await.removed_rows, 2);
    // Simulate discovering a bad remainder after an earlier old chunk expired.
    q("UPDATE changes SET at='invalid-timestamp' WHERE account_id=?1")
        .bind(&a)
        .execute(&store.backend)
        .await
        .unwrap();
    assert_eq!(prune(store, &a, 10).await.removed_rows, 0);
    assert_eq!(
        q("SELECT COUNT(*) FROM changes WHERE account_id=?1")
            .bind(&a)
            .fetch_scalar_i64(&store.backend)
            .await
            .unwrap(),
        4
    );
    assert!(matches!(
        store.changes_since(&a, "Email", 0).await,
        Err(StoreError::ChangeHistoryExpired)
    ));
    assert!(
        store
            .changes_since(&a, "Email", 1)
            .await
            .unwrap()
            .is_empty()
    );
    clean(store, &a).await;
}
async fn cancellation_contract(store: &Store) {
    let a = account(store).await;
    store
        .record_change(&a, "Email", "kept", "updated")
        .await
        .unwrap();
    age(store, &a).await;
    let backend = store.backend.clone();
    let target = a.clone();
    let (ready, started) = tokio::sync::oneshot::channel();
    let task = tokio::spawn(async move {
        let mut tx = backend.begin().await.unwrap();
        persist_floor(&mut tx, &target, "Email", 1).await.unwrap();
        ready.send(()).unwrap();
        std::future::pending::<()>().await;
        drop(tx);
    });
    started.await.unwrap();
    task.abort();
    assert!(task.await.unwrap_err().is_cancelled());
    assert_eq!(store.changes_since(&a, "Email", 0).await.unwrap().len(), 1);
    assert_eq!(prune(store, &a, 1).await.removed_rows, 1);
    assert_eq!(store.current_state(&a, "Email").await.unwrap(), 1);
    assert_eq!(
        store
            .record_change(&a, "Email", "after-cancel", "updated")
            .await
            .unwrap(),
        2
    );
    clean(store, &a).await;
}
async fn concurrent_contract(store: &Store) {
    let a = account(store).await;
    store
        .record_changes(
            &a,
            "Email",
            &(0..2000).map(|n| n.to_string()).collect::<Vec<_>>(),
            "updated",
        )
        .await
        .unwrap();
    age(store, &a).await;
    let (report, state, concurrent_read) = tokio::join!(
        prune(store, &a, 256),
        store.record_change(&a, "Email", "new-append", "updated"),
        store.changes_since(&a, "Email", 0)
    );
    match concurrent_read {
        Ok(rows) => assert_eq!(
            rows.iter().filter(|row| row.state == 1).count(),
            2000,
            "a reader must see the whole pre-cleanup state, never a partial chunk"
        ),
        Err(StoreError::ChangeHistoryExpired) => {}
        Err(error) => panic!("unexpected concurrent read: {error}"),
    }
    assert_eq!(report.removed_rows, 256);
    assert_eq!(state.unwrap(), 2);
    let tail = store.changes_since(&a, "Email", 1).await.unwrap();
    assert_eq!(tail.len(), 1);
    assert_eq!(tail[0].stable_id, "new-append");
    assert_eq!(store.current_state(&a, "Email").await.unwrap(), 2);
    if matches!(store.backend, Backend::Postgres(_)) {
        let mut held = store.backend.begin().await.unwrap();
        q("SELECT id FROM accounts WHERE id=?1 FOR NO KEY UPDATE")
            .bind(&a)
            .execute_tx(&mut held)
            .await
            .unwrap();
        let appended = tokio::time::timeout(
            Duration::from_secs(2),
            store.record_change(&a, "Email", "append-under-parent-lock", "updated"),
        )
        .await
        .expect("maintenance parent lock must allow FK KEY SHARE")
        .unwrap();
        assert_eq!(appended, 3);
        drop(held);
    }
    clean(store, &a).await;
}
#[tokio::test]
async fn large_state_chunks_preserve_tokens_and_monotonic_heads() {
    large_contract(&Store::open_in_memory(ServerKey::generate()).await.unwrap()).await;
}
#[tokio::test]
async fn visible_and_later_discovered_bad_timestamps_remain_protected() {
    timestamp_contract(&Store::open_in_memory(ServerKey::generate()).await.unwrap()).await;
}
#[tokio::test]
async fn cancelled_floor_transaction_rolls_back_before_deletion() {
    cancellation_contract(&Store::open_in_memory(ServerKey::generate()).await.unwrap()).await;
}
#[tokio::test]
async fn foreground_append_survives_chunk_cleanup() {
    concurrent_contract(&Store::open_in_memory(ServerKey::generate()).await.unwrap()).await;
}
#[tokio::test]
async fn postgres_large_state_timestamp_cancellation_and_append_contracts() {
    let Ok(dsn) = std::env::var("MW_BG_TEST_PG") else {
        eprintln!("SKIP PostgreSQL chunk retention: MW_BG_TEST_PG is unset");
        return;
    };
    let store = Store::open_postgres(&dsn, ServerKey::generate())
        .await
        .unwrap();
    large_contract(&store).await;
    timestamp_contract(&store).await;
    cancellation_contract(&store).await;
    concurrent_contract(&store).await;
    store.close().await;
}

#[tokio::test]
async fn locator_chunk_work_is_bounded_despite_large_target_and_unrelated_histories() {
    use std::sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    };
    let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
    let target = account(&store).await;
    let unrelated = account(&store).await;
    let ids = (0..25000).map(|n| n.to_string()).collect::<Vec<_>>();
    store
        .record_changes(&target, "Email", &ids, "updated")
        .await
        .unwrap();
    store
        .record_changes(&unrelated, "Email", &ids, "updated")
        .await
        .unwrap();
    age(&store, &target).await;
    let Backend::Sqlite(pool) = &store.backend else {
        unreachable!()
    };
    let pin = pool.pin().await;
    let count = Arc::new(AtomicU64::new(0));
    let observed = count.clone();
    let mut connection = pin.pool().acquire().await.unwrap();
    connection
        .lock_handle()
        .await
        .unwrap()
        .set_progress_handler(1000, move || {
            observed.fetch_add(1, Ordering::Relaxed);
            true
        });
    drop(connection);
    let report = prune(&store, &target, 64).await;
    let chunk_steps = count.swap(0, Ordering::Relaxed);
    let control = store.current_states(&target).await.unwrap();
    let scan_steps = count.load(Ordering::Relaxed);
    pin.pool()
        .acquire()
        .await
        .unwrap()
        .lock_handle()
        .await
        .unwrap()
        .remove_progress_handler();
    assert_eq!(report.removed_rows, 64);
    assert_eq!(control["Email"], 1);
    assert!(
        scan_steps > 100,
        "control must observe retained-history scan: {scan_steps}"
    );
    assert!(
        chunk_steps < 50,
        "a 64-row chunk must not scan the large tail: {chunk_steps}"
    );
    assert_eq!(
        q("SELECT COUNT(*) FROM changes WHERE account_id=?1")
            .bind(&unrelated)
            .fetch_scalar_i64(&store.backend)
            .await
            .unwrap(),
        25000
    );
}
