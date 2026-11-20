//! Bounded prefix retention for mail change logs. Checkpoints outlive deleted
//! rows, so old tokens are refused and future state counters never go backwards.
use crate::{
    Store, StoreError,
    backend::{Backend, Dialect, Row, Tx},
    q,
};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeLogRetentionOptions {
    pub enabled: bool,
    pub retention_days: u32,
    pub batch_rows: u32,
    pub max_run_ms: u64,
    pub max_wal_bytes: u64,
}
impl Default for ChangeLogRetentionOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            retention_days: 60,
            batch_rows: 1000,
            max_run_ms: 250,
            max_wal_bytes: 32 * 1024 * 1024,
        }
    }
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeLogRetentionCursor {
    pub account_id: String,
    pub kind: String,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeLogRetentionReport {
    pub at: i64,
    pub status: String,
    pub cursor: ChangeLogRetentionCursor,
    pub scanned_rows: u64,
    pub removed_rows: u64,
    /// Legacy report field retained for old persisted reports.
    pub oversized_batches: u64,
    #[serde(default)]
    pub chunked_states: u64,
    pub elapsed_ms: u64,
}
impl Store {
    /// Owned task keeps the generation lease and rollback guard alive even if
    /// the caller disconnects. No foreground connection policy is changed.
    pub async fn expire_change_log(
        &self,
        options: ChangeLogRetentionOptions,
        cursor: ChangeLogRetentionCursor,
    ) -> Result<ChangeLogRetentionReport, StoreError> {
        let backend = self.backend.clone();
        tokio::spawn(async move { run(backend, options, cursor, false).await })
            .await
            .map_err(|e| StoreError::Db(sqlx::Error::Protocol(e.to_string())))?
    }

    /// Background scheduling resumes across process/store restarts. Manual
    /// callers of `expire_change_log` retain their explicit cursor semantics.
    pub async fn expire_change_log_resumable(
        &self,
        options: ChangeLogRetentionOptions,
    ) -> Result<ChangeLogRetentionReport, StoreError> {
        let backend = self.backend.clone();
        tokio::spawn(async move {
            run(backend, options, ChangeLogRetentionCursor::default(), true).await
        }).await.map_err(|e| StoreError::Db(sqlx::Error::Protocol(e.to_string())))?
    }
}
async fn run(
    original: Backend,
    mut options: ChangeLogRetentionOptions,
    cursor: ChangeLogRetentionCursor,
    resumable: bool,
) -> Result<ChangeLogRetentionReport, StoreError> {
    let started = Instant::now();
    let mut report = ChangeLogRetentionReport {
        at: chrono::Utc::now().timestamp(),
        status: "complete".into(),
        cursor,
        ..Default::default()
    };
    if !options.enabled || options.retention_days == 0 {
        report.status = "disabled".into();
        return Ok(report);
    }
    options.retention_days = options.retention_days.min(3650);
    options.batch_rows = options.batch_rows.clamp(1, 10000);
    options.max_run_ms = options.max_run_ms.clamp(25, 2000);
    let mut pin = match &original {
        Backend::Sqlite(p) => Some(p.pin().await),
        _ => None,
    };
    let mut source = match &pin {
        Some(p) => match crate::background_maintenance::idle_keepalive(p).await {
            Some(c) => Some(c),
            None => {
                report.status = "busy".into();
                return Ok(report);
            }
        },
        None => None,
    };
    if let Some(p) = &pin {
        let wal = std::fs::metadata(format!(
            "{}-wal",
            p.active_options().get_filename().display()
        ))
        .map(|m| m.len())
        .unwrap_or(0);
        if wal > options.max_wal_bytes {
            report.status = "wal-pressure".into();
            return Ok(report);
        }
    }
    let mut lease = None;
    let backend = match original {
        Backend::Sqlite(_) => {
            let connect = (*pin.as_ref().unwrap().active_options())
                .clone()
                .busy_timeout(Duration::from_millis(50))
                .pragma("wal_autocheckpoint", "0");
            let pool = sqlx::sqlite::SqlitePoolOptions::new()
                .max_connections(1)
                .acquire_timeout(Duration::from_secs(2))
                .connect_lazy_with(connect);
            lease = Some(crate::background_maintenance::WorkerLease::new(
                pool.clone(),
                pin.take().unwrap(),
                source.take().unwrap(),
            ));
            Backend::Sqlite(pool.into())
        }
        Backend::Postgres(pool) => Backend::Postgres(
            sqlx::postgres::PgPoolOptions::new()
                .max_connections(1)
                .acquire_timeout(Duration::from_secs(2))
                .after_connect(|c, _| {
                    Box::pin(async move {
                        sqlx::query("SET lock_timeout='50ms'")
                            .execute(&mut *c)
                            .await?;
                        sqlx::query("SET statement_timeout='500ms'")
                            .execute(&mut *c)
                            .await?;
                        Ok(())
                    })
                })
                .connect_with((*pool.connect_options()).clone())
                .await?,
        ),
    };
    let deadline = Instant::now() + Duration::from_millis(options.max_run_ms);
    let outcome = async {
        if let Backend::Sqlite(pool) = &backend {
            pool.acquire()
                .await?
                .lock_handle()
                .await?
                .set_progress_handler(1000, move || Instant::now() < deadline);
        }
        if resumable {
            let group = reserve_next_group(&backend, report.at).await?;
            report.cursor = group.clone().unwrap_or_default();
            match group {
                Some(group) => collect_group(&backend, &options, deadline, &mut report, &group.account_id, &group.kind).await,
                None => Ok(()),
            }
        } else {
            collect(&backend, &options, deadline, &mut report).await
        }
    }
    .await;
    if let Backend::Sqlite(pool) = &backend {
        if let Ok(mut c) = pool.acquire().await {
            if let Ok(mut handle) = c.lock_handle().await {
                handle.remove_progress_handler();
            }
            // Bounded input only: never checkpoint a large pre-existing WAL.
            let bytes = std::fs::metadata(format!("{}-wal", pool.logical_path().display()))
                .map(|m| m.len())
                .unwrap_or(0);
            if bytes <= options.max_wal_bytes {
                let _ = sqlx::query("PRAGMA wal_checkpoint(PASSIVE)")
                    .execute(&mut *c)
                    .await;
            }
        }
    }
    match &backend {
        Backend::Sqlite(p) => p.close().await,
        Backend::Postgres(p) => p.close().await,
    }
    if let Some(lease) = &mut lease {
        lease.close().await;
    }
    report.elapsed_ms = started.elapsed().as_millis() as u64;
    if let Err(error) = outcome {
        let message = error.to_string().to_ascii_lowercase();
        if Instant::now() >= deadline
            || message.contains("locked")
            || message.contains("busy")
            || message.contains("timeout")
        {
            report.status = if Instant::now() >= deadline {
                "budget"
            } else {
                "busy"
            }
            .into();
        } else {
            return Err(error);
        }
    }
    Ok(report)
}
async fn collect(
    backend: &Backend,
    options: &ChangeLogRetentionOptions,
    deadline: Instant,
    report: &mut ChangeLogRetentionReport,
) -> Result<(), StoreError> {
    // Skip directly to the next indexed account/type, without DISTINCT over the
    // whole history. Visit each account/type once per round, even with backlog.
    let groups=q("SELECT account_id,type FROM changes WHERE (account_id,type)>(?1,?2) ORDER BY account_id,type LIMIT 1")
        .bind(&report.cursor.account_id).bind(&report.cursor.kind).fetch_all(backend).await?;
    let Some(group) = groups.first() else {
        report.cursor = Default::default();
        return Ok(());
    };
    let account = group.get_string("account_id");
    let kind = group.get_string("type");
    // Advance before attempting SQL so a busy or oversized group cannot
    // monopolize every run. A later round safely resumes its remaining prefix.
    report.cursor = ChangeLogRetentionCursor {
        account_id: account.clone(),
        kind: kind.clone(),
    };
    collect_group(backend, options, deadline, report, &account, &kind).await
}

/// Reservation commits separately from deletion: a busy group or a crash during
/// cleanup must not reset round-robin progress after every process restart.
/// Both transactions use the same pinned worker generation and execution budget.
async fn reserve_next_group(
    backend: &Backend,
    at: i64,
) -> Result<Option<ChangeLogRetentionCursor>, StoreError> {
    let mut tx = backend.begin().await?;
    let select = match backend.dialect() {
        Dialect::Sqlite => "SELECT account_id,type FROM change_log_maintenance_cursor WHERE id=1",
        Dialect::Postgres => "SELECT account_id,type FROM change_log_maintenance_cursor WHERE id=1 FOR UPDATE",
    };
    let rows = q(select).fetch_all_tx(&mut tx).await?;
    let previous = rows.first().ok_or_else(|| StoreError::Corrupt("change-log maintenance cursor missing".into()))?;
    let groups = q("SELECT account_id,type FROM changes WHERE (account_id,type)>(?1,?2) ORDER BY account_id,type LIMIT 1")
        .bind(previous.get_string("account_id")).bind(previous.get_string("type"))
        .fetch_all_tx(&mut tx).await?;
    let next = groups.first().map(|group| ChangeLogRetentionCursor {
        account_id: group.get_string("account_id"), kind: group.get_string("type"),
    });
    let saved = next.clone().unwrap_or_default();
    q("UPDATE change_log_maintenance_cursor SET account_id=?1,type=?2,updated_at=?3 WHERE id=1")
        .bind(saved.account_id).bind(saved.kind).bind(at).execute_tx(&mut tx).await?;
    tx.commit().await?;
    Ok(next)
}

async fn collect_group(
    backend: &Backend,
    options: &ChangeLogRetentionOptions,
    deadline: Instant,
    report: &mut ChangeLogRetentionReport,
    account: &str,
    kind: &str,
) -> Result<(), StoreError> {
    let cutoff = report.at - i64::from(options.retention_days) * 86400;
    let mut tx = backend.begin().await?;
    // Lock the parent first, matching account erasure's parent/child order.
    if matches!(backend, Backend::Postgres(_)) {
        q("SELECT id FROM accounts WHERE id=?1 FOR NO KEY UPDATE")
            .bind(account)
            .execute_tx(&mut tx)
            .await?;
    }
    let sql = match backend.dialect() {
        Dialect::Sqlite => {
            "SELECT CAST(rowid AS TEXT) AS locator,state,at FROM changes WHERE account_id=?1 AND type=?2 ORDER BY state ASC LIMIT ?3"
        }
        Dialect::Postgres => {
            "SELECT CAST(ctid AS TEXT) AS locator,state,at FROM changes WHERE account_id=?1 AND type=?2 ORDER BY state ASC LIMIT ?3 FOR UPDATE SKIP LOCKED"
        }
    };
    let rows = q(sql)
        .bind(account)
        .bind(kind)
        .bind(i64::from(options.batch_rows) + 1)
        .fetch_all_tx(&mut tx)
        .await?;
    report.scanned_rows = rows.len() as u64;
    let mut floor = 0;
    let mut locators = Vec::new();
    let mut chunked = false;
    let mut index = 0;
    while index < rows.len() && index < (options.batch_rows as usize) && Instant::now() < deadline {
        let state = rows[index].get_i64("state");
        let start = index;
        while index < rows.len() && rows[index].get_i64("state") == state {
            index += 1;
        }
        // A known recent/malformed row protects every selected row of its state,
        // including when that row is the bounded lookahead. Unseen rows can only
        // be checked by later chunks; the durable floor makes that sync-safe.
        if !rows[start..index].iter().all(|row| older_than(row, cutoff)) {
            break;
        }
        let end = index.min(options.batch_rows as usize);
        locators.extend(rows[start..end].iter().map(|row| row.get_string("locator")));
        floor = state;
        chunked = index > end;
    }
    if !locators.is_empty() && Instant::now() < deadline {
        persist_floor(&mut tx, account, kind, floor).await?;
        let removed = delete_chunk(&mut tx, backend.dialect(), account, kind, &locators).await?;
        tx.commit().await?;
        report.removed_rows = removed;
        report.chunked_states = u64::from(chunked && removed > 0);
    } else {
        tx.commit().await?;
    }

    if Instant::now() >= deadline {
        report.status = "budget".into();
    }
    Ok(())
}
async fn persist_floor(
    tx: &mut Tx,
    account: &str,
    kind: &str,
    floor: i64,
) -> Result<(), StoreError> {
    q("INSERT INTO change_log_checkpoint(account_id,type,head_state,pruned_through)
       SELECT ?1,?2,COALESCE(MAX(state),0),?3 FROM changes WHERE account_id=?1 AND type=?2
       ON CONFLICT(account_id,type) DO UPDATE SET
       head_state=CASE WHEN change_log_checkpoint.head_state>excluded.head_state THEN change_log_checkpoint.head_state ELSE excluded.head_state END,
       pruned_through=CASE WHEN change_log_checkpoint.pruned_through>excluded.pruned_through THEN change_log_checkpoint.pruned_through ELSE excluded.pruned_through END")
        .bind(account).bind(kind).bind(floor).execute_tx(tx).await?;
    Ok(())
}
async fn delete_chunk(
    tx: &mut Tx,
    dialect: Dialect,
    account: &str,
    kind: &str,
    locators: &[String],
) -> Result<u64, StoreError> {
    let sql = delete_chunk_sql(dialect);
    let ids = serde_json::to_string(locators).expect("row locator strings serialize");
    Ok(q(sql)
        .bind(account)
        .bind(kind)
        .bind(ids)
        .execute_tx(tx)
        .await?)
}

fn delete_chunk_sql(dialect: Dialect) -> &'static str {
    match dialect {
        Dialect::Sqlite => {
            "DELETE FROM changes NOT INDEXED WHERE account_id=?1 AND type=?2 AND rowid IN(SELECT CAST(value AS INTEGER) FROM json_each(?3))"
        }
        Dialect::Postgres => {
            "DELETE FROM changes WHERE ctid = ANY(ARRAY(
                 WITH targets AS MATERIALIZED (
                   SELECT ctid,account_id,type FROM changes
                   WHERE ctid = ANY(ARRAY(SELECT CAST(value AS pg_catalog.tid) FROM jsonb_array_elements_text(?3::jsonb)))
                 ) SELECT ctid FROM targets WHERE account_id=?1 AND type=?2
               ))"
        }
    }
}

fn older_than(row: &Row, cutoff: i64) -> bool {
    chrono::DateTime::parse_from_rfc3339(&row.get_string("at"))
        .is_ok_and(|t| t.timestamp() < cutoff)
}

#[cfg(test)]
#[path = "change_log_retention_chunk_tests.rs"]
mod chunk_tests;

#[cfg(test)]
#[path = "change_log_retention_cursor_tests.rs"]
mod cursor_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AccountKind, Credentials, NewAccount, ServerKey};
    async fn account(store: &Store, user: &str) -> String {
        store
            .create_account(
                &NewAccount {
                    kind: AccountKind::Imap,
                    host: "fixture.invalid",
                    port: 993,
                    tls: "implicit",
                    username: user,
                    sync_policy_json: "{}",
                },
                &Credentials {
                    username: user.into(),
                    password: "fixture".into(),
                },
            )
            .await
            .unwrap()
    }
    async fn aged(store: &Store) {
        q("UPDATE changes SET at='2000-01-01T00:00:00Z'")
            .execute(&store.backend)
            .await
            .unwrap();
    }
    async fn prune(store: &Store, rows: u32) -> ChangeLogRetentionReport {
        let mut report = ChangeLogRetentionReport {
            at: chrono::Utc::now().timestamp(),
            status: "complete".into(),
            ..Default::default()
        };
        collect(
            &store.backend,
            &ChangeLogRetentionOptions {
                batch_rows: rows,
                ..Default::default()
            },
            Instant::now() + Duration::from_secs(5),
            &mut report,
        )
        .await
        .unwrap();
        report
    }
    #[tokio::test]
    async fn complete_prune_preserves_head_resume_boundary_and_next_append() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let a = account(&store, "a").await;
        store
            .record_change(&a, "Email", "a", "created")
            .await
            .unwrap();
        store
            .record_changes(&a, "Email", &["b".into(), "c".into()], "updated")
            .await
            .unwrap();
        aged(&store).await;
        assert_eq!(prune(&store, 10).await.removed_rows, 3);
        assert_eq!(store.current_state(&a, "Email").await.unwrap(), 2);
        assert_eq!(store.current_states(&a).await.unwrap()["Email"], 2);
        assert!(matches!(
            store.changes_since(&a, "Email", 0).await,
            Err(StoreError::ChangeHistoryExpired)
        ));
        assert!(matches!(
            store.changes_since_limited(&a, "Email", 1, 0).await,
            Err(StoreError::ChangeHistoryExpired)
        ));
        assert!(
            store
                .changes_since(&a, "Email", 2)
                .await
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            store
                .record_change(&a, "Email", "d", "updated")
                .await
                .unwrap(),
            3
        );
        assert_eq!(
            store
                .record_changes(&a, "Email", &["e".into(), "f".into()], "updated")
                .await
                .unwrap(),
            4
        );
        let delta = store.changes_since(&a, "Email", 2).await.unwrap();
        assert_eq!(delta.len(), 3);
        assert_eq!(delta.last().unwrap().state, 4);
    }
    #[tokio::test]
    async fn chunked_states_are_fenced_from_deltas_and_other_groups_progress() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let a = account(&store, "a").await;
        let ids = (0..500).map(|i| format!("m{i}")).collect::<Vec<_>>();
        store
            .record_changes(&a, "Email", &ids, "updated")
            .await
            .unwrap();
        store
            .record_change(&a, "Mailbox", "box", "updated")
            .await
            .unwrap();
        aged(&store).await;
        let small = prune(&store, 64).await;
        assert_eq!(small.removed_rows, 64);
        assert_eq!(small.chunked_states, 1);
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
        let mut next = ChangeLogRetentionReport {
            at: chrono::Utc::now().timestamp(),
            cursor: small.cursor,
            ..Default::default()
        };
        collect(
            &store.backend,
            &ChangeLogRetentionOptions::default(),
            Instant::now() + Duration::from_secs(5),
            &mut next,
        )
        .await
        .unwrap();
        assert_eq!(next.removed_rows, 1);
        assert_eq!(prune(&store, 500).await.removed_rows, 436);
        assert_eq!(store.current_state(&a, "Email").await.unwrap(), 1);
    }
    #[tokio::test]
    async fn recent_or_malformed_rows_protect_the_entire_state_and_later_prefix() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let a = account(&store, "a").await;
        store
            .record_change(&a, "Email", "old", "updated")
            .await
            .unwrap();
        store
            .record_changes(&a, "Email", &["x".into(), "y".into()], "updated")
            .await
            .unwrap();
        store
            .record_change(&a, "Email", "later", "updated")
            .await
            .unwrap();
        aged(&store).await;
        q("UPDATE changes SET at='not-a-date' WHERE stable_id='y'")
            .execute(&store.backend)
            .await
            .unwrap();
        assert_eq!(prune(&store, 10).await.removed_rows, 1);
        let tail = store.changes_since(&a, "Email", 1).await.unwrap();
        assert_eq!(tail.len(), 3);
        assert_eq!(store.current_state(&a, "Email").await.unwrap(), 3);
        assert!(matches!(
            store.changes_since(&a, "Email", 0).await,
            Err(StoreError::ChangeHistoryExpired)
        ));
        assert_eq!(
            store.changes_since(&a, "Mailbox", 0).await.unwrap().len(),
            0
        );
    }
    #[tokio::test]
    async fn foreground_appends_and_retention_preserve_monotonic_history() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let a = account(&store, "a").await;
        for i in 0..20 {
            store
                .record_change(&a, "Email", &i.to_string(), "updated")
                .await
                .unwrap();
        }
        aged(&store).await;
        let append = async {
            for i in 20..40 {
                assert_eq!(
                    store
                        .record_change(&a, "Email", &i.to_string(), "updated")
                        .await
                        .unwrap(),
                    i + 1
                );
            }
        };
        let retention = async {
            for _ in 0..10 {
                prune(&store, 3).await;
            }
        };
        tokio::join!(append, retention);
        assert_eq!(store.current_state(&a, "Email").await.unwrap(), 40);
        assert_eq!(
            store.changes_since(&a, "Email", 20).await.unwrap().len(),
            20
        );
        let floor = q(
            "SELECT pruned_through FROM change_log_checkpoint WHERE account_id=?1 AND type='Email'",
        )
        .bind(&a)
        .fetch_scalar_i64(&store.backend)
        .await
        .unwrap();
        assert_eq!(floor, 20);
    }
    #[tokio::test]
    async fn dedicated_worker_prunes_and_reopens_plaintext_and_encrypted_databases() {
        use crate::encryption::{DbKey, EncryptionConfig, EncryptionMode};
        for encrypted in [false, true] {
            let dir =
                std::env::temp_dir().join(format!("mw-change-retention-{}", uuid::Uuid::new_v4()));
            std::fs::create_dir(&dir).unwrap();
            let path = dir.join("fixture.db");
            let key = ServerKey::generate();
            let config = EncryptionConfig {
                mode: if encrypted {
                    EncryptionMode::Required
                } else {
                    EncryptionMode::Off
                },
                key: encrypted.then(DbKey::generate),
                source: None,
            };
            let store = Store::open_with_encryption(path.to_str().unwrap(), key.clone(), &config)
                .await
                .unwrap();
            let a = account(&store, "a").await;
            store
                .record_change(&a, "Email", "old", "created")
                .await
                .unwrap();
            aged(&store).await;
            let disabled = store
                .expire_change_log(
                    ChangeLogRetentionOptions {
                        enabled: false,
                        ..Default::default()
                    },
                    Default::default(),
                )
                .await
                .unwrap();
            assert_eq!(disabled.status, "disabled");
            let report = store
                .expire_change_log(
                    ChangeLogRetentionOptions {
                        max_run_ms: 2000,
                        ..Default::default()
                    },
                    Default::default(),
                )
                .await
                .unwrap();
            assert_eq!(report.removed_rows, 1, "{report:?}");
            store.close().await;
            drop(store);
            let reopened = Store::open_with_encryption(path.to_str().unwrap(), key, &config)
                .await
                .unwrap();
            assert_eq!(reopened.current_state(&a, "Email").await.unwrap(), 1);
            assert_eq!(
                reopened
                    .record_change(&a, "Email", "new", "updated")
                    .await
                    .unwrap(),
                2
            );
            reopened.close().await;
            drop(reopened);
            for attempt in 0..20 {
                match std::fs::remove_dir_all(&dir) {
                    Ok(()) => break,
                    Err(error) if attempt == 19 => panic!("fixture cleanup: {error}"),
                    Err(_) => tokio::time::sleep(Duration::from_millis(25)).await,
                }
            }
        }
    }
    #[tokio::test]
    async fn a_hot_first_account_cannot_starve_the_next_account() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let mut accounts = [
            account(&store, "first").await,
            account(&store, "second").await,
        ];
        accounts.sort();
        let hot = &accounts[0];
        let next = &accounts[1];
        store
            .record_changes(
                hot,
                "Email",
                &["a".into(), "b".into(), "c".into()],
                "updated",
            )
            .await
            .unwrap();
        store
            .record_change(next, "Email", "other", "updated")
            .await
            .unwrap();
        aged(&store).await;
        let first = prune(&store, 1).await;
        assert_eq!(first.removed_rows, 1);
        assert_eq!(first.chunked_states, 1);
        assert_eq!(first.cursor.account_id, *hot);
        let mut second = ChangeLogRetentionReport {
            at: chrono::Utc::now().timestamp(),
            cursor: first.cursor,
            ..Default::default()
        };
        collect(
            &store.backend,
            &ChangeLogRetentionOptions {
                batch_rows: 1,
                ..Default::default()
            },
            Instant::now() + Duration::from_secs(5),
            &mut second,
        )
        .await
        .unwrap();
        assert_eq!(second.removed_rows, 1);
        assert_eq!(second.cursor.account_id, *next);
        assert!(matches!(
            store.changes_since(next, "Email", 0).await,
            Err(StoreError::ChangeHistoryExpired)
        ));
        assert_eq!(
            q("SELECT COUNT(*) FROM changes WHERE account_id=?1")
                .bind(hot)
                .fetch_scalar_i64(&store.backend)
                .await
                .unwrap(),
            2
        );
        let mut wrapped = ChangeLogRetentionReport {
            at: chrono::Utc::now().timestamp(),
            cursor: second.cursor,
            ..Default::default()
        };
        collect(
            &store.backend,
            &ChangeLogRetentionOptions::default(),
            Instant::now() + Duration::from_secs(5),
            &mut wrapped,
        )
        .await
        .unwrap();
        assert_eq!(wrapped.cursor.account_id, "");
        collect(
            &store.backend,
            &ChangeLogRetentionOptions {
                batch_rows: 1,
                ..Default::default()
            },
            Instant::now() + Duration::from_secs(5),
            &mut wrapped,
        )
        .await
        .unwrap();
        assert_eq!(wrapped.cursor.account_id, *hot);
        assert_eq!(wrapped.removed_rows, 1);
    }
    #[tokio::test]
    async fn expired_tokens_do_not_materialize_the_retained_tail() {
        use std::sync::{
            Arc,
            atomic::{AtomicU64, Ordering},
        };
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let a = account(&store, "tail-probe").await;
        store
            .record_change(&a, "Email", "old", "updated")
            .await
            .unwrap();
        aged(&store).await;
        let ids = (0..10000)
            .map(|n| format!("retained-{n}"))
            .collect::<Vec<_>>();
        store
            .record_changes(&a, "Email", &ids, "updated")
            .await
            .unwrap();
        assert_eq!(prune(&store, 1).await.removed_rows, 1);
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
        assert!(matches!(
            store
                .changes_since_limited(&a, "Email", 0, i64::MAX - 1)
                .await,
            Err(StoreError::ChangeHistoryExpired)
        ));
        let expired_steps = count.swap(0, Ordering::Relaxed);
        let tail = store
            .changes_since_limited(&a, "Email", 1, i64::MAX - 1)
            .await
            .unwrap();
        let valid_steps = count.load(Ordering::Relaxed);
        pin.pool()
            .acquire()
            .await
            .unwrap()
            .lock_handle()
            .await
            .unwrap()
            .remove_progress_handler();
        assert_eq!(tail.0.len(), 10000);
        assert!(
            valid_steps > 50,
            "control must prove the SQLite VM probe is active: {valid_steps}"
        );
        assert!(
            expired_steps <= 2,
            "expired state must not scan/materialize the retained tail: {expired_steps}"
        );
    }
    #[tokio::test]
    async fn postgres_checkpoint_and_floor_preserve_resume_contract() {
        let Ok(dsn) = std::env::var("MW_BG_TEST_PG") else {
            eprintln!("SKIP live PostgreSQL retention: MW_BG_TEST_PG is unset");
            return;
        };
        let store = Store::open_postgres(&dsn, ServerKey::generate())
            .await
            .unwrap();
        let a = account(&store, &format!("retention-{}", uuid::Uuid::new_v4())).await;
        store
            .record_change(&a, "Email", "old", "updated")
            .await
            .unwrap();
        q("UPDATE changes SET at='2000-01-01T00:00:00Z' WHERE account_id=?1")
            .bind(&a)
            .execute(&store.backend)
            .await
            .unwrap();
        let mut report = ChangeLogRetentionReport {
            at: chrono::Utc::now().timestamp(),
            cursor: ChangeLogRetentionCursor {
                account_id: a.clone(),
                kind: String::new(),
            },
            ..Default::default()
        };
        collect(
            &store.backend,
            &ChangeLogRetentionOptions::default(),
            Instant::now() + Duration::from_secs(5),
            &mut report,
        )
        .await
        .unwrap();
        assert_eq!(report.removed_rows, 1);
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
        assert_eq!(store.current_states(&a).await.unwrap()["Email"], 1);
        assert_eq!(
            store
                .record_changes(&a, "Email", &["new".into()], "updated")
                .await
                .unwrap(),
            2
        );
        q("DELETE FROM accounts WHERE id=?1")
            .bind(&a)
            .execute(&store.backend)
            .await
            .unwrap();
        store.close().await;
    }
}
