//! Offline SQLite repair. No migrations, decryption of mail, or automatic deletion.
use crate::encryption;
use serde_json::{Value, json};
use sqlx::{Connection, Row};
use std::path::Path;

/// Preview is read-only. Applying requires an operator to stop every writer first.
pub async fn sqlite_maintenance(
    path: &Path,
    apply: bool,
    orphans: bool,
    log_days: Option<u32>,
    compact: bool,
) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
    if !path.is_file() {
        return Err("database file does not exist".into());
    }
    encryption::source_key(path)?; // Fail cleanly before raw_options can panic.
    let options = encryption::raw_options(&path.to_string_lossy(), !apply);
    maintain_with_options(path, apply, orphans, log_days, compact, &options).await
}
async fn maintain_with_options(
    path: &Path,
    apply: bool,
    orphans: bool,
    log_days: Option<u32>,
    compact: bool,
    options: &sqlx::sqlite::SqliteConnectOptions,
) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
    let mut connection = sqlx::SqliteConnection::connect_with(options).await?;
    let page =
        crate::database_observability::sqlite_pragma_i64(&mut connection, "PRAGMA page_size")
            .await?;
    let pages =
        crate::database_observability::sqlite_pragma_i64(&mut connection, "PRAGMA page_count")
            .await?;
    let free =
        crate::database_observability::sqlite_pragma_i64(&mut connection, "PRAGMA freelist_count")
            .await?;
    let usage=sqlx::query("SELECT COUNT(*) AS n,COALESCE(SUM(length(sealed_bytes)),0) AS bytes FROM bodies WHERE blob_ref NOT IN (SELECT blob_ref FROM messages WHERE blob_ref IS NOT NULL)").fetch_one(&mut connection).await?;
    let orphan_count: i64 = usage.try_get("n")?;
    let orphan_bytes: i64 = usage.try_get("bytes")?;
    let mut logs = Vec::new();
    if let Some(days) = log_days {
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
        sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)")
            .execute(&mut connection)
            .await?;
        if compact {
            sqlx::query("VACUUM").execute(&mut connection).await?;
        }
        sqlx::query("PRAGMA optimize")
            .execute(&mut connection)
            .await?;
        sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)")
            .execute(&mut connection)
            .await?;
    }
    connection.close().await?;
    Ok(
        json!({"applied":apply,"bytesBefore":pages*page,"freeBytesBefore":free*page,"bytesAfter":std::fs::metadata(path)?.len(),
        "orphanBodies":orphan_count,"orphanBytes":orphan_bytes,"removedBodies":removed,"operationalLogs":logs,"compacted":apply&&compact}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
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
