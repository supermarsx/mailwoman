//! Bounded online cache collection and incremental SQLite reclamation.
//! Dedicated connections never change the foreground pool's lock/deadline policy.
use crate::{
    backend::{Backend, Tx},
    q, Store, StoreError,
};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MaintenanceOptions {
    pub collect_bodies: bool,
    pub incremental_vacuum: bool,
    pub batch_rows: u32,
    pub batch_bytes: u64,
    pub grace_seconds: u64,
    pub batch_pages: u32,
    pub max_run_ms: u64,
    pub min_free_bytes: u64,
    pub max_wal_bytes: u64,
}
impl Default for MaintenanceOptions {
    fn default() -> Self {
        Self {
            collect_bodies: true,
            incremental_vacuum: true,
            batch_rows: 64,
            batch_bytes: 16 * 1024 * 1024,
            grace_seconds: 86400,
            batch_pages: 1024,
            max_run_ms: 250,
            min_free_bytes: 16 * 1024 * 1024,
            max_wal_bytes: 32 * 1024 * 1024,
        }
    }
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MaintenanceReport {
    pub at: i64,
    pub status: String,
    pub cursor: String,
    pub scanned: u64,
    pub marked: u64,
    pub removed: u64,
    pub removed_bytes: u64,
    pub reclaimed_pages: u64,
    pub auto_vacuum: Option<i64>,
    pub vacuum_status: String,
    pub elapsed_ms: u64,
}
/// SQLite BEGIN IMMEDIATE already serializes writers. PostgreSQL needs an
/// account lifecycle lock shared by body insertion, reference attachment and GC.
pub(crate) async fn lock_body_account(tx: &mut Tx, account: &str) -> Result<(), sqlx::Error> {
    if matches!(tx, Tx::Postgres(_)) {
        q("SELECT pg_advisory_xact_lock(hashtextextended(?1,719304))")
            .bind(account)
            .execute_tx(tx)
            .await?;
    }
    Ok(())
}
async fn try_lock(tx: &mut Tx, account: &str) -> Result<bool, sqlx::Error> {
    if matches!(tx, Tx::Postgres(_)) {
        let row=q("SELECT CASE WHEN pg_try_advisory_xact_lock(hashtextextended(?1,719304)) THEN CAST(1 AS BIGINT) ELSE CAST(0 AS BIGINT) END AS acquired").bind(account).fetch_all_tx(tx).await?;
        return Ok(row[0].get_i64("acquired") == 1);
    }
    Ok(true)
}
impl Store {
    /// Cancellation cannot leave a modified connection in the foreground pool or
    /// interrupt the short interval between BEGIN and its rollback guard.
    pub async fn background_maintenance(
        &self,
        options: MaintenanceOptions,
        cursor: String,
    ) -> Result<MaintenanceReport, StoreError> {
        let backend = self.backend.clone();
        tokio::spawn(async move { run(backend, options, cursor).await })
            .await
            .map_err(|e| StoreError::from(sqlx::Error::Protocol(e.to_string())))?
    }
}
async fn run(
    original: Backend,
    mut options: MaintenanceOptions,
    cursor: String,
) -> Result<MaintenanceReport, StoreError> {
    options.batch_rows = options.batch_rows.clamp(1, 512);
    options.batch_pages = options.batch_pages.clamp(8, 16384);
    options.max_run_ms = options.max_run_ms.clamp(25, 2000);
    options.batch_bytes = options.batch_bytes.clamp(1024 * 1024, 256 * 1024 * 1024);
    options.grace_seconds = options.grace_seconds.max(3600);
    let started = Instant::now();
    let source_pin = match &original {
        Backend::Sqlite(pool) => Some(pool.pin().await),
        Backend::Postgres(_) => None,
    };
    let backend = match original {
        Backend::Sqlite(_) => Backend::Sqlite(
            sqlx::sqlite::SqlitePoolOptions::new()
                .max_connections(1)
                .acquire_timeout(Duration::from_secs(2))
                .connect_with(
                    (*source_pin.as_ref().expect("SQLite source pin").active_options())
                        .clone()
                        .busy_timeout(Duration::from_millis(50))
                        .pragma("wal_autocheckpoint", "0"),
                )
                .await?.into(),
        ),
        Backend::Postgres(pool) => Backend::Postgres(
            sqlx::postgres::PgPoolOptions::new()
                .max_connections(1)
                .acquire_timeout(Duration::from_secs(2))
                .after_connect(|connection, _| {
                    Box::pin(async move {
                        sqlx::query("SET lock_timeout='50ms'")
                            .execute(&mut *connection)
                            .await?;
                        sqlx::query("SET statement_timeout='500ms'")
                            .execute(&mut *connection)
                            .await?;
                        Ok(())
                    })
                })
                .connect_with((*pool.connect_options()).clone())
                .await?,
        ),
    };
    let deadline = Instant::now() + Duration::from_millis(options.max_run_ms);
    if let Backend::Sqlite(pool) = &backend {
        let mut connection = pool.acquire().await?;
        connection
            .lock_handle()
            .await?
            .set_progress_handler(1000, move || Instant::now() < deadline);
    }
    let mut report = MaintenanceReport {
        at: chrono::Utc::now().timestamp(),
        status: "complete".into(),
        cursor,
        vacuum_status: "disabled".into(),
        ..Default::default()
    };
    let result = async {
        if wal_bytes(&backend).is_some_and(|bytes| bytes > options.max_wal_bytes) {
            report.status = "wal-pressure".into();
            report.vacuum_status = "wal-pressure".into();
            return Ok(());
        }
        if options.collect_bodies {
            collect(&backend, &options, deadline, &mut report).await?;
        }
        if options.incremental_vacuum && Instant::now() < deadline {
            vacuum(&backend, &options, deadline, &mut report).await?;
        }
        Ok::<(), StoreError>(())
    }
    .await;
    // Checkpoint only bounded WAL input, even when the SQL work budget expired.
    // Otherwise idle-only GC could grow a WAL without any foreground writer to checkpoint it.
    if let Backend::Sqlite(pool) = &backend {
        if let Ok(mut connection) = pool.acquire().await {
            if let Ok(mut handle) = connection.lock_handle().await {
                handle.remove_progress_handler();
            }
            if (options.collect_bodies || options.incremental_vacuum)
                && wal_bytes(&backend).is_some_and(|bytes| bytes <= options.max_wal_bytes)
            {
                let _ = sqlx::query("PRAGMA wal_checkpoint(PASSIVE)")
                    .execute(&mut *connection)
                    .await;
            }
        }
    }
    match &backend {
        Backend::Sqlite(pool) => pool.close().await,
        Backend::Postgres(pool) => pool.close().await,
    };
    report.elapsed_ms = started.elapsed().as_millis() as u64;
    if let Err(error) = result {
        let text = error.to_string().to_ascii_lowercase();
        if Instant::now() >= deadline
            || text.contains("locked")
            || text.contains("busy")
            || text.contains("statement timeout")
            || text.contains("lock timeout")
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
    options: &MaintenanceOptions,
    deadline: Instant,
    report: &mut MaintenanceReport,
) -> Result<(), StoreError> {
    let mut tx = backend.begin().await?;
    let rows =
        q("SELECT blob_ref,account_id FROM bodies WHERE blob_ref>?1 ORDER BY blob_ref LIMIT ?2")
            .bind(report.cursor.as_str())
            .bind(i64::from(options.batch_rows))
            .fetch_all_tx(&mut tx)
            .await?;
    let mut next = report.cursor.clone();
    let mut scanned = 0;
    let mut marked = 0;
    let mut removed = 0;
    let mut bytes = 0;
    for row in &rows {
        if Instant::now() >= deadline {
            break;
        }
        let blob = row.get_string("blob_ref");
        let account = row.get_string("account_id");
        next = blob.clone();
        scanned += 1;
        if !try_lock(&mut tx, &account).await? {
            continue;
        }
        marked+=q("INSERT INTO body_gc_candidate(blob_ref,first_seen) SELECT ?1,?2 WHERE NOT EXISTS(SELECT 1 FROM messages WHERE blob_ref=?1) ON CONFLICT(blob_ref) DO NOTHING").bind(blob).bind(report.at).execute_tx(&mut tx).await?;
    }
    let cutoff = report.at - options.grace_seconds.min(i64::MAX as u64) as i64;
    let candidates=q("SELECT b.blob_ref,b.account_id,CAST(octet_length(b.sealed_bytes) AS BIGINT) AS bytes FROM body_gc_candidate c JOIN bodies b ON b.blob_ref=c.blob_ref WHERE c.first_seen<=?1 AND octet_length(b.sealed_bytes)<=?2 ORDER BY c.first_seen,c.blob_ref LIMIT ?3").bind(cutoff).bind(options.batch_bytes as i64).bind(i64::from(options.batch_rows)).fetch_all_tx(&mut tx).await?;
    for row in candidates {
        let size = row.get_i64("bytes").max(0) as u64;
        if Instant::now() >= deadline || bytes + size > options.batch_bytes {
            break;
        }
        let blob = row.get_string("blob_ref");
        let account = row.get_string("account_id");
        if !try_lock(&mut tx, &account).await? {
            continue;
        }
        let n=q("DELETE FROM bodies WHERE blob_ref=?1 AND account_id=?2 AND NOT EXISTS(SELECT 1 FROM messages WHERE blob_ref=?1) AND EXISTS(SELECT 1 FROM body_gc_candidate WHERE blob_ref=?1 AND first_seen<=?3)").bind(&blob).bind(account).bind(cutoff).execute_tx(&mut tx).await?;
        if n == 0 {
            q("DELETE FROM body_gc_candidate WHERE blob_ref=?1 AND EXISTS(SELECT 1 FROM messages WHERE blob_ref=?1)").bind(blob).execute_tx(&mut tx).await?;
        }
        removed += n;
        bytes += n * size;
    }
    tx.commit().await?;
    report.scanned = scanned;
    report.marked = marked;
    report.removed = removed;
    report.removed_bytes = bytes;
    report.cursor = if rows.len() < options.batch_rows as usize && scanned == rows.len() as u64 {
        String::new()
    } else {
        next
    };
    Ok(())
}
fn wal_bytes(backend: &Backend) -> Option<u64> {
    let Backend::Sqlite(pool) = backend else {
        return None;
    };
    let path = pool.logical_path().to_path_buf();
    Some(
        std::fs::metadata(format!("{}-wal", path.display()))
            .map(|meta| meta.len())
            .unwrap_or(0),
    )
}
async fn vacuum(
    backend: &Backend,
    options: &MaintenanceOptions,
    deadline: Instant,
    report: &mut MaintenanceReport,
) -> Result<(), StoreError> {
    let Backend::Sqlite(pool) = backend else {
        report.vacuum_status = "postgres-autovacuum".into();
        return Ok(());
    };
    let path = pool.logical_path().to_path_buf();
    let wal = std::fs::metadata(format!("{}-wal", path.display()))
        .map(|m| m.len())
        .unwrap_or(0);
    if wal > options.max_wal_bytes {
        report.vacuum_status = "wal-pressure".into();
        return Ok(());
    }
    let mut connection = pool.acquire().await?;
    let mode =
        crate::database_observability::sqlite_pragma_i64(&mut connection, "PRAGMA auto_vacuum")
            .await?;
    report.auto_vacuum = Some(mode);
    if mode != 2 {
        report.vacuum_status = if mode == 0 {
            "offline-conversion-required"
        } else {
            "full-auto-vacuum"
        }
        .into();
        return Ok(());
    }
    let before =
        crate::database_observability::sqlite_pragma_i64(&mut connection, "PRAGMA freelist_count")
            .await?;
    let page =
        crate::database_observability::sqlite_pragma_i64(&mut connection, "PRAGMA page_size")
            .await?;
    if before.saturating_mul(page) < options.min_free_bytes as i64 {
        report.vacuum_status = "below-threshold".into();
        return Ok(());
    }
    report.vacuum_status = "complete".into();
    let start =
        crate::database_observability::sqlite_pragma_i64(&mut connection, "PRAGMA page_count")
            .await?;
    for _ in 0..options.batch_pages / 8 {
        if Instant::now() >= deadline {
            report.vacuum_status = "budget".into();
            break;
        }
        match sqlx::query("PRAGMA incremental_vacuum(8)")
            .execute(&mut *connection)
            .await
        {
            Ok(_) => {}
            Err(_) if Instant::now() >= deadline => {
                report.vacuum_status = "budget".into();
                break;
            }
            Err(error) => return Err(error.into()),
        }
    }
    connection.lock_handle().await?.remove_progress_handler();
    let end =
        crate::database_observability::sqlite_pragma_i64(&mut connection, "PRAGMA page_count")
            .await?;
    report.reclaimed_pages = start.saturating_sub(end).max(0) as u64;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cache::{AccountKind, MailboxUpsert, MessageUpsert, NewAccount},
        Credentials, ServerKey,
    };
    async fn seed(store: &Store) -> (String, String) {
        let username = format!("maintenance-{}@example.test", uuid::Uuid::new_v4());
        let account = store
            .create_account(
                &NewAccount {
                    kind: AccountKind::Imap,
                    host: "mail.example.test",
                    port: 993,
                    tls: "implicit",
                    username: &username,
                    sync_policy_json: "{}",
                },
                &Credentials {
                    username: "test@example.test".into(),
                    password: "secret".into(),
                },
            )
            .await
            .unwrap();
        let mailbox = store
            .upsert_mailbox(&MailboxUpsert {
                account_id: &account,
                name: "INBOX",
                role: Some("inbox"),
                uidvalidity: 1,
                uidnext: 1,
                highestmodseq: 0,
                total: 0,
                unread: 0,
                parent_id: None,
            })
            .await
            .unwrap();
        (account, mailbox)
    }
    fn message<'a>(account: &'a str, mailbox: &'a str, blob: Option<&'a str>) -> MessageUpsert<'a> {
        MessageUpsert {
            account_id: account,
            mailbox_id: mailbox,
            uid: 1,
            uidvalidity: 1,
            message_id: None,
            thread_id: None,
            internaldate: None,
            size: 4,
            flags_json: "[]",
            envelope: None,
            blob_ref: blob,
        }
    }
    fn gc() -> MaintenanceOptions {
        MaintenanceOptions {
            incremental_vacuum: false,
            max_run_ms: 2000,
            grace_seconds: 3600,
            ..Default::default()
        }
    }
    #[tokio::test]
    async fn legacy_orphans_receive_grace_and_referenced_bodies_survive() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let (account, mailbox) = seed(&store).await;
        let live = store
            .upsert_message_with_body(&message(&account, &mailbox, None), b"live")
            .await
            .unwrap()
            .0;
        let orphan = store.put_body(&account, b"orphan").await.unwrap();
        q("DELETE FROM body_gc_candidate")
            .execute(&store.backend)
            .await
            .unwrap();
        let first = store
            .background_maintenance(gc(), String::new())
            .await
            .unwrap();
        assert_eq!(first.marked, 1);
        assert_eq!(first.removed, 0);
        q("UPDATE body_gc_candidate SET first_seen=0")
            .execute(&store.backend)
            .await
            .unwrap();
        let second = store
            .background_maintenance(gc(), String::new())
            .await
            .unwrap();
        assert_eq!(second.removed, 1);
        assert!(second.removed_bytes > 0);
        assert!(store.get_body(&orphan).await.unwrap().is_none());
        let reference = store.get_message(&live).await.unwrap().blob_ref.unwrap();
        assert_eq!(store.get_body(&reference).await.unwrap().unwrap(), b"live");
    }
    #[tokio::test]
    async fn refreshed_standalone_body_lease_and_missing_references_are_safe() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let (account, mailbox) = seed(&store).await;
        let blob = store.put_body(&account, b"pending").await.unwrap();
        q("UPDATE body_gc_candidate SET first_seen=0")
            .execute(&store.backend)
            .await
            .unwrap();
        assert_eq!(store.put_body(&account, b"pending").await.unwrap(), blob);
        assert_eq!(
            store
                .background_maintenance(gc(), String::new())
                .await
                .unwrap()
                .removed,
            0
        );
        assert!(matches!(
            store
                .upsert_message(&message(&account, &mailbox, Some("missing")))
                .await,
            Err(StoreError::NotFound)
        ));
    }
    #[tokio::test]
    async fn bounded_batches_collect_only_the_requested_number() {
        let store = Store::open_in_memory(ServerKey::generate()).await.unwrap();
        let (account, _) = seed(&store).await;
        for n in 0..5 {
            store.put_body(&account, &[n]).await.unwrap();
        }
        q("UPDATE body_gc_candidate SET first_seen=0")
            .execute(&store.backend)
            .await
            .unwrap();
        let report = store
            .background_maintenance(
                MaintenanceOptions {
                    batch_rows: 2,
                    ..gc()
                },
                String::new(),
            )
            .await
            .unwrap();
        assert_eq!(report.removed, 2);
        assert_eq!(report.scanned, 2);
        assert!(!report.cursor.is_empty());
    }
    async fn clean(path: std::path::PathBuf) {
        for attempt in 0..20 {
            match std::fs::remove_dir_all(&path) {
                Ok(()) => return,
                Err(error) if attempt == 19 => panic!("fixture cleanup: {error}"),
                Err(_) => tokio::time::sleep(Duration::from_millis(25)).await,
            }
        }
    }
    async fn file_store(mode: bool) -> (Store, std::path::PathBuf) {
        let dir = std::env::temp_dir().join(format!("mw-online-maint-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&dir).unwrap();
        let path = dir.join("data.db");
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(4)
            .connect_with(
                sqlx::sqlite::SqliteConnectOptions::new()
                    .filename(&path)
                    .create_if_missing(true)
                    .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
                    .busy_timeout(Duration::from_secs(2)),
            )
            .await
            .unwrap();
        if !mode {
            sqlx::query("CREATE TABLE legacy(id INTEGER)")
                .execute(&pool)
                .await
                .unwrap();
        }
        (
            Store::init_sqlite(pool, ServerKey::generate())
                .await
                .unwrap(),
            dir,
        )
    }
    #[tokio::test]
    async fn incremental_vacuum_is_bounded_and_never_converts_existing_databases() {
        for mode in [true, false] {
            let (store, dir) = file_store(mode).await;
            let Backend::Sqlite(pool) = &store.backend else {
                unreachable!()
            };
            sqlx::query("CREATE TABLE junk(data BLOB);INSERT INTO junk VALUES(zeroblob(2097152));DELETE FROM junk").execute(pool).await.unwrap();
            let report = store
                .background_maintenance(
                    MaintenanceOptions {
                        collect_bodies: false,
                        batch_pages: 16,
                        min_free_bytes: 0,
                        max_run_ms: 2000,
                        max_wal_bytes: 128 * 1024 * 1024,
                        ..Default::default()
                    },
                    String::new(),
                )
                .await
                .unwrap();
            if mode {
                assert_eq!(report.auto_vacuum, Some(2));
                assert!(
                    report.reclaimed_pages > 0 && report.reclaimed_pages <= 16,
                    "{report:?}"
                );
            } else {
                assert_eq!(report.vacuum_status, "offline-conversion-required");
                assert_eq!(report.reclaimed_pages, 0);
            }
            store.close().await;
            clean(dir).await;
        }
    }
    #[tokio::test]
    async fn concurrent_body_reattachment_and_gc_never_leave_a_dangling_message() {
        let (store, dir) = file_store(true).await;
        let (account, mailbox) = seed(&store).await;
        for _ in 0..10 {
            store.put_body(&account, b"live after race").await.unwrap();
            q("UPDATE body_gc_candidate SET first_seen=0")
                .execute(&store.backend)
                .await
                .unwrap();
            let input = message(&account, &mailbox, None);
            let (ingest, maintenance) = tokio::join!(
                store.upsert_message_with_body(&input, b"live after race"),
                store.background_maintenance(gc(), String::new())
            );
            let (id, _) = ingest.unwrap();
            maintenance.unwrap();
            let reference = store.get_message(&id).await.unwrap().blob_ref.unwrap();
            assert_eq!(
                store.get_body(&reference).await.unwrap().unwrap(),
                b"live after race"
            );
        }
        store.close().await;
        clean(dir).await;
    }
    #[tokio::test]
    async fn busy_writer_is_skipped_instead_of_waiting_for_foreground_timeout() {
        let (store, dir) = file_store(true).await;
        seed(&store).await;
        let held = store.backend.begin().await.unwrap();
        let started = Instant::now();
        let report = store
            .background_maintenance(gc(), String::new())
            .await
            .unwrap();
        assert_eq!(report.status, "busy");
        assert!(started.elapsed() < Duration::from_secs(1));
        drop(held);
        store.close().await;
        clean(dir).await;
    }
    #[tokio::test]
    async fn postgres_collector_respects_account_locks_and_preserves_live_bodies() {
        let Ok(url) = std::env::var("MW_BG_TEST_PG") else {
            return;
        };
        let store = Store::open_postgres(&url, ServerKey::generate())
            .await
            .unwrap();
        let (account, mailbox) = seed(&store).await;
        let message = message(&account, &mailbox, None);
        let (id, _) = store
            .upsert_message_with_body(&message, b"live pg")
            .await
            .unwrap();
        let orphan = store.put_body(&account, b"orphan pg").await.unwrap();
        q("UPDATE body_gc_candidate SET first_seen=0 WHERE blob_ref=?1")
            .bind(&orphan)
            .execute(&store.backend)
            .await
            .unwrap();
        let mut held = store.backend.begin().await.unwrap();
        lock_body_account(&mut held, &account).await.unwrap();
        let _first = store
            .background_maintenance(gc(), String::new())
            .await
            .unwrap();
        assert!(store.get_body(&orphan).await.unwrap().is_some());
        drop(held);
        let report = store
            .background_maintenance(gc(), String::new())
            .await
            .unwrap();
        assert!(report.removed >= 1);
        assert!(store.get_body(&orphan).await.unwrap().is_none());
        let reference = store.get_message(&id).await.unwrap().blob_ref.unwrap();
        assert_eq!(
            store.get_body(&reference).await.unwrap().unwrap(),
            b"live pg"
        );
        store.close().await;
    }
    #[tokio::test]
    async fn wal_pressure_defers_both_collection_and_vacuum() {
        let (store, dir) = file_store(true).await;
        let (account, _) = seed(&store).await;
        let blob = store
            .put_body(&account, b"protected under WAL pressure")
            .await
            .unwrap();
        q("UPDATE body_gc_candidate SET first_seen=0")
            .execute(&store.backend)
            .await
            .unwrap();
        let mut options = gc();
        options.max_wal_bytes = 0;
        options.incremental_vacuum = true;
        let report = store
            .background_maintenance(options, String::new())
            .await
            .unwrap();
        assert_eq!(report.status, "wal-pressure");
        assert_eq!(report.removed, 0);
        assert!(store.get_body(&blob).await.unwrap().is_some());
        store.close().await;
        clean(dir).await;
    }
    #[tokio::test]
    async fn private_worker_connection_preserves_sqlcipher_encryption() {
        use crate::encryption::{self, DbKey, EncryptionConfig, EncryptionMode};
        let dir = std::env::temp_dir().join(format!("mw-bg-encrypted-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&dir).unwrap();
        let path = dir.join("data.db");
        let config = EncryptionConfig {
            mode: EncryptionMode::Required,
            key: Some(DbKey::generate()),
            source: None,
        };
        let store =
            Store::open_with_encryption(path.to_str().unwrap(), ServerKey::generate(), &config)
                .await
                .unwrap();
        let (account, mailbox) = seed(&store).await;
        store
            .upsert_message_with_body(&message(&account, &mailbox, None), b"keep encrypted")
            .await
            .unwrap();
        let orphan = store
            .put_body(&account, b"collect encrypted")
            .await
            .unwrap();
        q("UPDATE body_gc_candidate SET first_seen=0")
            .execute(&store.backend)
            .await
            .unwrap();
        let report = store
            .background_maintenance(gc(), String::new())
            .await
            .unwrap();
        assert_eq!(report.removed, 1);
        assert!(store.get_body(&orphan).await.unwrap().is_none());
        store.close().await;
        assert_eq!(
            encryption::probe_file(&path).unwrap(),
            encryption::FileState::NotPlaintext
        );
        clean(dir).await;
    }
}
