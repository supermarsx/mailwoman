//! Offline SQLite repair. No migrations, decryption of mail, or automatic deletion.
use crate::encryption;
use serde_json::{Value, json};
use sqlx::{Connection, Row};
use std::path::Path;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicU64, Ordering},
};
use std::time::Instant;

/// SQLite exposes VM activity, not a reliable VACUUM completion percentage.
struct OfflineProgress {
    phase: Arc<Mutex<(&'static str, Instant)>>,
    vm_steps: Arc<AtomicU64>,
    cancel: Arc<AtomicBool>,
    heartbeat: tokio::task::JoinHandle<()>,
    timings: Vec<Value>,
    finished: bool,
}
impl OfflineProgress {
    fn new(path: &Path, cancel: Arc<AtomicBool>) -> Self {
        let phase = Arc::new(Mutex::new(("opening database", Instant::now())));
        let vm_steps = Arc::new(AtomicU64::new(0));
        let observed = phase.clone();
        let steps = vm_steps.clone();
        let cancelled = cancel.clone();
        let path = path.to_owned();
        eprintln!("[maintenance] opening database");
        let heartbeat = tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                let (phase, start) = *observed.lock().unwrap_or_else(|e| e.into_inner());
                let bytes = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                let wal = std::fs::metadata(format!("{}-wal", path.display()))
                    .map(|m| m.len())
                    .unwrap_or(0);
                eprintln!(
                    "[maintenance] {phase}: {}s elapsed; approximately {} SQLite VM steps; database {} MiB; WAL {} MiB{}",
                    start.elapsed().as_secs(),
                    steps.load(Ordering::Relaxed),
                    bytes / 1048576,
                    wal / 1048576,
                    if cancelled.load(Ordering::Relaxed) {
                        "; cancellation requested, waiting for SQLite rollback/close"
                    } else {
                        ""
                    }
                );
            }
        });
        Self {
            phase,
            vm_steps,
            cancel,
            heartbeat,
            timings: Vec::new(),
            finished: false,
        }
    }
    fn stage(&mut self, name: &'static str) {
        self.transition(name, true);
    }
    fn transition(&mut self, name: &'static str, completed: bool) {
        let mut phase = self.phase.lock().unwrap_or_else(|e| e.into_inner());
        self.timings.push(json!({"phase":phase.0,"completed":completed,"elapsedMs":phase.1.elapsed().as_millis() as u64,"approximateVmSteps":self.vm_steps.swap(0,Ordering::Relaxed)}));
        eprintln!(
            "[maintenance] {} {} after {}s",
            phase.0,
            if completed { "complete" } else { "stopped" },
            phase.1.elapsed().as_secs()
        );
        *phase = (name, Instant::now());
        eprintln!("[maintenance] {name}");
    }
    fn finish(&mut self) {
        self.stage("complete");
        self.finished = true;
        self.heartbeat.abort();
    }
}
impl Drop for OfflineProgress {
    fn drop(&mut self) {
        self.heartbeat.abort();
        if !self.finished {
            let phase = self.phase.lock().unwrap_or_else(|e| e.into_inner());
            eprintln!(
                "[maintenance] {} stopped after {}s{}; retain SQLite journal/WAL files for recovery",
                phase.0,
                phase.1.elapsed().as_secs(),
                if self.cancel.load(Ordering::Relaxed) {
                    " following cancellation"
                } else {
                    " with an error"
                }
            );
        }
    }
}

/// Preview is read-only. Applying requires an operator to stop every writer first.
pub async fn sqlite_maintenance(
    path: &Path,
    apply: bool,
    orphans: bool,
    log_days: Option<u32>,
    compact: bool,
) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
    sqlite_maintenance_with_incremental(path, apply, orphans, log_days, compact, false).await
}

/// Enable incremental auto-vacuum only during acknowledged offline maintenance.
pub async fn sqlite_maintenance_with_incremental(
    path: &Path,
    apply: bool,
    orphans: bool,
    log_days: Option<u32>,
    compact: bool,
    enable_incremental: bool,
) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
    sqlite_maintenance_with_control(
        path,
        apply,
        orphans,
        log_days,
        compact,
        enable_incremental,
        Arc::new(AtomicBool::new(false)),
    )
    .await
}

/// Request cancellation without dropping the running SQL future. SQLite's
/// progress handler interrupts at a VM boundary; rollback/close must finish
/// before the caller releases its offline lease or restarts the application.
pub async fn sqlite_maintenance_with_control(
    path: &Path,
    apply: bool,
    orphans: bool,
    log_days: Option<u32>,
    compact: bool,
    enable_incremental: bool,
    cancel: Arc<AtomicBool>,
) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
    if !path.is_file() {
        return Err("database file does not exist".into());
    }
    encryption::source_key(path)?; // Fail cleanly before raw_options can panic.
    let options = encryption::raw_options(&path.to_string_lossy(), !apply);
    maintain_with_options_control(
        path,
        apply,
        orphans,
        log_days,
        compact,
        &options,
        enable_incremental,
        cancel,
    )
    .await
}
#[cfg(test)]
async fn maintain_with_options(
    path: &Path,
    apply: bool,
    orphans: bool,
    log_days: Option<u32>,
    compact: bool,
    options: &sqlx::sqlite::SqliteConnectOptions,
) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
    maintain_with_options_incremental(path, apply, orphans, log_days, compact, options, false).await
}
async fn maintain_with_options_incremental(
    path: &Path,
    apply: bool,
    orphans: bool,
    log_days: Option<u32>,
    compact: bool,
    options: &sqlx::sqlite::SqliteConnectOptions,
    enable_incremental: bool,
) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
    maintain_with_options_control(
        path,
        apply,
        orphans,
        log_days,
        compact,
        options,
        enable_incremental,
        Arc::new(AtomicBool::new(false)),
    )
    .await
}
async fn maintain_with_options_control(
    path: &Path,
    apply: bool,
    orphans: bool,
    log_days: Option<u32>,
    compact: bool,
    options: &sqlx::sqlite::SqliteConnectOptions,
    enable_incremental: bool,
    cancel: Arc<AtomicBool>,
) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
    let mut progress = OfflineProgress::new(path, cancel.clone());
    let mut connection = sqlx::SqliteConnection::connect_with(options).await?;
    let steps = progress.vm_steps.clone();
    connection
        .lock_handle()
        .await?
        .set_progress_handler(10000, move || {
            steps.fetch_add(10000, Ordering::Relaxed);
            !cancel.load(Ordering::Relaxed)
        });
    let result:Result<Value,Box<dyn std::error::Error+Send+Sync>>=async {
    progress.stage("reading database layout");
    let page =
        crate::database_observability::sqlite_pragma_i64(&mut connection, "PRAGMA page_size")
            .await?;
    let pages =
        crate::database_observability::sqlite_pragma_i64(&mut connection, "PRAGMA page_count")
            .await?;
    let free =
        crate::database_observability::sqlite_pragma_i64(&mut connection, "PRAGMA freelist_count")
            .await?;
    let auto_vacuum=crate::database_observability::sqlite_pragma_i64(&mut connection,"PRAGMA auto_vacuum").await?;
    // A body scan can read a huge table. It is unrelated to enabling incremental
    // vacuum and must not delay that operation when cleanup was not requested.
    let (orphan_count,orphan_bytes)=if orphans {
        progress.stage("counting orphan bodies");
        let usage=sqlx::query("SELECT COUNT(*) AS n,COALESCE(SUM(length(sealed_bytes)),0) AS bytes FROM bodies WHERE blob_ref NOT IN (SELECT blob_ref FROM messages WHERE blob_ref IS NOT NULL)").fetch_one(&mut connection).await?;
        (Some(usage.try_get::<i64,_>("n")?),Some(usage.try_get::<i64,_>("bytes")?))
    }else{(None,None)};
    let mut logs = Vec::new();
    if let Some(days) = log_days {
        progress.stage("counting and cleaning selected operational logs");
        let cutoff = chrono::Utc::now().timestamp() - i64::from(days) * 86400;
        // Operational diagnostics only. Audit/security/compliance records, mail and
        // queued jobs are deliberately excluded from this explicit target list.
        for (table, age, guard) in [
            ("client_error_event", "time", "1=1"),
            (
                "client_error_group",
                "last_seen",
                "NOT EXISTS (SELECT 1 FROM client_error_event WHERE group_id=client_error_group.id)",
            ),
            ("job_run", "started_at", "status<>'running'"),
            ("diag_run", "started_at", "status<>'running'"),
        ] {
            let exists: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM sqlite_schema WHERE type='table' AND name=?",
            )
            .bind(table)
            .fetch_one(&mut connection)
            .await?;
            if exists == 0 {
                continue;
            }
            let clause = format!("{age}<? AND {guard}");
            let count: i64 =
                sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {table} WHERE {clause}"))
                    .bind(cutoff)
                    .fetch_one(&mut connection)
                    .await?;
            let mut removed = 0;
            if apply {
                loop {
                    let n=sqlx::query(&format!("DELETE FROM {table} WHERE rowid IN (SELECT rowid FROM {table} WHERE {clause} LIMIT 1000)")).bind(cutoff).execute(&mut connection).await?.rows_affected();
                    removed += n;
                    if n == 0 {
                        break;
                    }
                }
            }
            logs.push(json!({"table":table,"eligibleRows":count,"removedRows":removed}));
        }
    }
    let mut removed = 0;
    if apply && orphans {
        progress.stage("removing orphan bodies");
        loop {
            let n=sqlx::query("DELETE FROM bodies WHERE blob_ref IN (SELECT blob_ref FROM bodies WHERE blob_ref NOT IN (SELECT blob_ref FROM messages WHERE blob_ref IS NOT NULL) LIMIT 128)").execute(&mut connection).await?.rows_affected();
            removed += n;
            if n == 0 {
                break;
            }
        }
    }
    if apply {
        // Checkpoint before VACUUM to avoid carrying a huge old WAL throughout it.
        progress.stage("checkpointing WAL before rebuild");
        if crate::database_observability::sqlite_pragma_i64(&mut connection,"PRAGMA wal_checkpoint(TRUNCATE)").await?!=0 {return Err("WAL checkpoint is busy; stop all other database users before offline maintenance".into());}
        if enable_incremental { sqlx::query("PRAGMA auto_vacuum=INCREMENTAL").execute(&mut connection).await?; }
        if compact || enable_incremental && auto_vacuum==0 {
            progress.stage("rebuilding database with VACUUM (no reliable percentage)");
            sqlx::query("VACUUM").execute(&mut connection).await?;
        }
        progress.stage("optimizing database");
        sqlx::query("PRAGMA optimize")
            .execute(&mut connection)
            .await?;
        progress.stage("checkpointing WAL after maintenance");
        if crate::database_observability::sqlite_pragma_i64(&mut connection,"PRAGMA wal_checkpoint(TRUNCATE)").await?!=0 {return Err("final WAL checkpoint is busy; retain the WAL and close other database users before retrying".into());}
    }
    Ok(
        json!({"applied":apply,"bytesBefore":pages*page,"freeBytesBefore":free*page,"bytesAfter":std::fs::metadata(path)?.len(),
        "orphanBodies":orphan_count,"orphanBytes":orphan_bytes,"orphanStatisticsMeasured":orphans,"removedBodies":removed,"operationalLogs":logs,"compacted":apply&&(compact||enable_incremental&&auto_vacuum==0),"autoVacuumBefore":auto_vacuum,"incrementalVacuumEnabled":apply&&enable_incremental,"offlineConversionRequired":enable_incremental&&auto_vacuum==0&&!apply}),
    )
    }.await;
    // Never return from interruption/error while a SQLite worker might still
    // own the offline database. Await statement completion above, then close.
    progress.transition(
        "closing database after statement completion",
        result.is_ok(),
    );
    if let Ok(mut handle) = connection.lock_handle().await {
        handle.remove_progress_handler();
    }
    connection.close().await?;
    let mut report = result?;
    report["bytesAfter"] = json!(std::fs::metadata(path)?.len());
    progress.finish();
    report["phaseTimings"] = json!(&progress.timings);
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn incremental_conversion_does_not_scan_unrequested_body_tables() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("without-body-tables.db");
        let options = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(&path)
            .create_if_missing(true);
        let mut connection = sqlx::SqliteConnection::connect_with(&options)
            .await
            .unwrap();
        sqlx::query("CREATE TABLE retained(id INTEGER PRIMARY KEY,value TEXT); INSERT INTO retained VALUES(1,'keep')").execute(&mut connection).await.unwrap();
        connection.close().await.unwrap();
        let report =
            maintain_with_options_incremental(&path, true, false, None, false, &options, true)
                .await
                .unwrap();
        assert_eq!(report["orphanStatisticsMeasured"], false);
        assert!(report["orphanBodies"].is_null());
        assert!(
            report["phaseTimings"]
                .as_array()
                .unwrap()
                .iter()
                .any(|phase| phase["phase"]
                    == "rebuilding database with VACUUM (no reliable percentage)")
        );
        let mut connection = sqlx::SqliteConnection::connect_with(&options)
            .await
            .unwrap();
        assert_eq!(
            crate::database_observability::sqlite_pragma_i64(&mut connection, "PRAGMA auto_vacuum")
                .await
                .unwrap(),
            2
        );
        assert_eq!(
            sqlx::query_scalar::<_, String>("SELECT value FROM retained")
                .fetch_one(&mut connection)
                .await
                .unwrap(),
            "keep"
        );
        connection.close().await.unwrap();
    }
    #[tokio::test]
    async fn cancelled_vacuum_closes_worker_and_preserves_original_rows() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cancel.db");
        let options = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(&path)
            .create_if_missing(true);
        let mut connection = sqlx::SqliteConnection::connect_with(&options)
            .await
            .unwrap();
        sqlx::query("CREATE TABLE retained(id INTEGER PRIMARY KEY,value BLOB); WITH RECURSIVE numbers(n) AS (VALUES(1) UNION ALL SELECT n+1 FROM numbers WHERE n<12000) INSERT INTO retained SELECT n,zeroblob(1024) FROM numbers").execute(&mut connection).await.unwrap();
        connection.close().await.unwrap();
        let result = maintain_with_options_control(
            &path,
            true,
            false,
            None,
            true,
            &options,
            false,
            Arc::new(AtomicBool::new(true)),
        )
        .await;
        assert!(
            result.is_err(),
            "VACUUM must observe cancellation through its VM callback"
        );
        // An exclusive transaction proves the previous worker released its locks.
        let mut connection = sqlx::SqliteConnection::connect_with(&options)
            .await
            .unwrap();
        sqlx::query("BEGIN EXCLUSIVE")
            .execute(&mut connection)
            .await
            .unwrap();
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM retained")
                .fetch_one(&mut connection)
                .await
                .unwrap(),
            12000
        );
        sqlx::query("ROLLBACK")
            .execute(&mut connection)
            .await
            .unwrap();
        assert_eq!(
            sqlx::query_scalar::<_, String>("PRAGMA integrity_check")
                .fetch_one(&mut connection)
                .await
                .unwrap(),
            "ok"
        );
        connection.close().await.unwrap();
    }
    #[test]
    fn index_purge_previews_and_refuses_unknown_files_before_removing_anything() {
        let dir = std::env::temp_dir().join(format!("mw-index-maint-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&dir).unwrap();
        let db = dir.join("mailwoman.db");
        std::fs::write(&db, b"fixture").unwrap();
        let index = dir.join("mailwoman.db.search-index");
        std::fs::create_dir(&index).unwrap();
        std::fs::write(index.join("meta.json"), b"sealed index").unwrap();
        assert_eq!(purge_default_search_index(&db, false).unwrap()["files"], 1);
        assert!(index.join("meta.json").exists());
        std::fs::write(index.join("unrelated.txt"), b"keep").unwrap();
        assert!(purge_default_search_index(&db, true).is_err());
        assert!(index.join("meta.json").exists());
        std::fs::remove_file(index.join("unrelated.txt")).unwrap();
        assert_eq!(purge_default_search_index(&db, true).unwrap()["files"], 1);
        assert!(!index.exists());
        assert!(db.is_file());
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[tokio::test]
    async fn offline_incremental_conversion_preserves_whole_database_encryption() {
        encryption::init();
        let dir =
            std::env::temp_dir().join(format!("mw-convert-incremental-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&dir).unwrap();
        let path = dir.join("data.db");
        let key = encryption::DbKey::generate();
        let options = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(&path)
            .create_if_missing(true)
            .pragma("key", encryption::key_pragma(&key));
        let mut connection = sqlx::SqliteConnection::connect_with(&options)
            .await
            .unwrap();
        sqlx::query("CREATE TABLE bodies(blob_ref TEXT PRIMARY KEY,sealed_bytes BLOB);CREATE TABLE messages(blob_ref TEXT);INSERT INTO bodies VALUES('live',zeroblob(4096));INSERT INTO messages VALUES('live')").execute(&mut connection).await.unwrap();
        connection.close().await.unwrap();
        let preview =
            maintain_with_options_incremental(&path, false, false, None, false, &options, true)
                .await
                .unwrap();
        assert_eq!(preview["offlineConversionRequired"], true);
        let result =
            maintain_with_options_incremental(&path, true, false, None, false, &options, true)
                .await
                .unwrap();
        assert_eq!(result["incrementalVacuumEnabled"], true);
        assert_eq!(
            encryption::probe_file(&path).unwrap(),
            encryption::FileState::NotPlaintext
        );
        let mut connection = sqlx::SqliteConnection::connect_with(&options)
            .await
            .unwrap();
        assert_eq!(
            crate::database_observability::sqlite_pragma_i64(&mut connection, "PRAGMA auto_vacuum")
                .await
                .unwrap(),
            2
        );
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM bodies")
                .fetch_one(&mut connection)
                .await
                .unwrap(),
            1
        );
        connection.close().await.unwrap();
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[tokio::test]
    async fn compaction_preserves_whole_database_encryption() {
        encryption::init();
        let dir = std::env::temp_dir().join(format!("mw-encrypted-maint-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&dir).unwrap();
        let path = dir.join("store.db");
        let key = encryption::DbKey::generate();
        let options = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(&path)
            .create_if_missing(true)
            .pragma("key", encryption::key_pragma(&key));
        let mut connection = sqlx::SqliteConnection::connect_with(&options)
            .await
            .unwrap();
        sqlx::query("CREATE TABLE bodies(blob_ref TEXT PRIMARY KEY,sealed_bytes BLOB);CREATE TABLE messages(blob_ref TEXT);INSERT INTO bodies VALUES('live',zeroblob(1024)),('orphan',zeroblob(1048576));INSERT INTO messages VALUES('live')").execute(&mut connection).await.unwrap();
        connection.close().await.unwrap();
        let report = maintain_with_options(&path, true, true, None, true, &options)
            .await
            .unwrap();
        assert_eq!(report["removedBodies"], 1);
        assert_eq!(
            encryption::probe_file(&path).unwrap(),
            encryption::FileState::NotPlaintext
        );
        let mut connection = sqlx::SqliteConnection::connect_with(&options)
            .await
            .unwrap();
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM bodies")
                .fetch_one(&mut connection)
                .await
                .unwrap(),
            1
        );
        connection.close().await.unwrap();
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[tokio::test]
    async fn preview_preserves_everything_apply_keeps_referenced_mail() {
        let dir = std::env::temp_dir().join(format!("mw-maint-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&dir).unwrap();
        let path = dir.join("store.db");
        let mut c = sqlx::SqliteConnection::connect_with(
            &sqlx::sqlite::SqliteConnectOptions::new()
                .filename(&path)
                .create_if_missing(true),
        )
        .await
        .unwrap();
        sqlx::query("CREATE TABLE bodies(blob_ref TEXT PRIMARY KEY,sealed_bytes BLOB); CREATE TABLE messages(blob_ref TEXT); CREATE TABLE metrics_bucket(bucket_start BIGINT); INSERT INTO metrics_bucket VALUES(1); CREATE TABLE audit_log(id TEXT); INSERT INTO audit_log VALUES('evidence'); CREATE TABLE client_error_event(id TEXT,time BIGINT); INSERT INTO client_error_event VALUES('old',1); INSERT INTO bodies VALUES('live',zeroblob(1024)),('orphan',zeroblob(1048576)); INSERT INTO messages VALUES('live')").execute(&mut c).await.unwrap();
        c.close().await.unwrap();
        let report = sqlite_maintenance(&path, false, true, None, true)
            .await
            .unwrap();
        assert_eq!(report["orphanBodies"], 1);
        assert_eq!(report["removedBodies"], 0);
        let report = sqlite_maintenance(&path, true, true, Some(0), true)
            .await
            .unwrap();
        assert_eq!(report["removedBodies"], 1);
        assert!(report["bytesAfter"].as_u64().unwrap() < report["bytesBefore"].as_u64().unwrap());
        let mut c = sqlx::SqliteConnection::connect_with(&encryption::raw_options(
            &path.to_string_lossy(),
            true,
        ))
        .await
        .unwrap();
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM bodies WHERE blob_ref='live'")
                .fetch_one(&mut c)
                .await
                .unwrap(),
            1
        );
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM metrics_bucket")
                .fetch_one(&mut c)
                .await
                .unwrap(),
            1
        );
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM audit_log")
                .fetch_one(&mut c)
                .await
                .unwrap(),
            1
        );
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM client_error_event")
                .fetch_one(&mut c)
                .await
                .unwrap(),
            0
        );
        c.close().await.unwrap();
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
/// Purge the default sibling search directory only, refusing symlinks and unknown
/// files. Custom index locations continue to use the online rebuild facility.
pub fn purge_default_search_index(
    database: &Path,
    apply: bool,
) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
    let database = database.canonicalize()?;
    let index = std::path::PathBuf::from(format!("{}.search-index", database.display()));
    if !index.exists() {
        return Ok(json!({"path":index,"files":0,"bytes":0,"applied":apply}));
    }
    let parent = database
        .parent()
        .ok_or("database has no parent directory")?;
    if std::fs::symlink_metadata(&index)?.file_type().is_symlink()
        || index.canonicalize()?.parent() != Some(parent)
    {
        return Err("refusing a redirected search-index directory".into());
    }
    let mut files = Vec::new();
    let mut bytes = 0u64;
    for entry in std::fs::read_dir(&index)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        let name = entry.file_name();
        let name = name.to_str().ok_or("invalid index filename")?;
        let stem = name.split('.').next().unwrap_or("");
        let known = matches!(
            name,
            "meta.json" | ".managed.json" | ".tantivy-meta.lock" | ".tantivy-writer.lock"
        ) || (stem.len() == 32
            && stem.bytes().all(|b| b.is_ascii_hexdigit())
            && name.len() > 33);
        if !kind.is_file() || !known {
            return Err("unexpected file or directory in search index; nothing removed".into());
        }
        bytes = bytes.saturating_add(entry.metadata()?.len());
        files.push(entry.path());
    }
    if apply {
        for file in &files {
            std::fs::remove_file(file)?
        }
        std::fs::remove_dir(&index)?;
    }
    Ok(
        json!({"path":index,"files":files.len(),"bytes":bytes,"applied":apply,"rebuildRequired":apply}),
    )
}
