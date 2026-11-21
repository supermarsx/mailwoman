//! Owned diagnostic only: app-cache misses, pager misses and physical disk reads
//! are distinct. These counters establish neither disk throughput nor all-page eviction.
use serde::Serialize;
use sha2::{Digest, Sha256};
use sqlx::{Connection, SqliteConnection};
use std::time::Duration;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, Error>;
const ROWS: usize = 128;
const ROW_BYTES: usize = 4096;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Observation {
    cycle: usize,
    control: &'static str,
    hits: u64,
    misses: u64,
    writes: u64,
    spills: u64,
    resident_before: u64,
    resident_after: u64,
}
fn known(value: Option<u64>) -> Result<u64> {
    value.ok_or_else(|| "owned db_status counter unavailable".into())
}
fn delta(before: Option<u64>, after: Option<u64>) -> Result<u64> {
    known(after)?.checked_sub(known(before)?)
        .ok_or_else(|| "owned counter reset/wrap: observation invalid".into())
}
async fn read_payload(reader: &mut SqliteConnection, expected: &[u8; 32]) -> Result<()> {
    let rows: Vec<Vec<u8>> = sqlx::query_scalar("SELECT payload FROM pager_payload ORDER BY id")
        .fetch_all(reader).await?;
    if rows.len() != ROWS || rows.iter().any(|row| row.len() != ROW_BYTES) {
        return Err("payload shape changed".into());
    }
    let mut digest = Sha256::new();
    for row in rows { digest.update(row); }
    let actual: [u8; 32] = digest.finalize().into();
    if &actual != expected { return Err("payload checksum changed".into()); }
    Ok(())
}
async fn observe(reader: &mut SqliteConnection, writer: &mut SqliteConnection, expected: &[u8;32]) -> Result<Vec<Observation>> {
    for connection in [&mut *reader, &mut *writer] {
        let cipher: String = sqlx::query_scalar("PRAGMA cipher_version").fetch_one(&mut *connection).await?;
        let hmac: i64 = crate::database_observability::sqlite_pragma_i64(connection, "PRAGMA cipher_use_hmac").await?;
        let journal: String = sqlx::query_scalar("PRAGMA journal_mode").fetch_one(&mut *connection).await?;
        let synchronous: i64 = crate::database_observability::sqlite_pragma_i64(connection, "PRAGMA synchronous").await?;
        let target: i64 = crate::database_observability::sqlite_pragma_i64(connection, "PRAGMA main.cache_size").await?;
        if cipher.is_empty() || hmac != 1 || !journal.eq_ignore_ascii_case("wal") || synchronous != 1 || target != -16384 {
            return Err("encryption/durability/cache fixture prerequisite failed".into());
        }
    }
    let mut observations = Vec::with_capacity(9);
    for cycle in 0..3 {
        for control in ["no-write", "rollback", "commit"] {
            // Fully materialized reads end their autocommit read transaction.
            // A second warm read verifies the working set fits, without reset=1.
            read_payload(reader, expected).await?;
            let warm_before = mw_sqlite_status::cache_status(reader).await?;
            read_payload(reader, expected).await?;
            let before = mw_sqlite_status::cache_status(reader).await?;
            if delta(warm_before.misses, before.misses)? != 0 || delta(warm_before.hits, before.hits)? == 0 {
                return Err("reader pages did not reach stable warm state".into());
            }
            if control != "no-write" {
                let mut transaction = writer.begin().await?;
                let changed = sqlx::query("UPDATE pager_control SET value=value+1 WHERE id=1")
                    .execute(&mut *transaction).await?.rows_affected();
                if changed != 1 { return Err("control update missed owned row".into()); }
                if control == "commit" { transaction.commit().await?; }
                else { transaction.rollback().await?; }
            }
            read_payload(reader, expected).await?;
            let after = mw_sqlite_status::cache_status(reader).await?;
            observations.push(Observation {
                cycle, control,
                hits: delta(before.hits, after.hits)?,
                misses: delta(before.misses, after.misses)?,
                writes: delta(before.writes, after.writes)?,
                spills: delta(before.spills, after.spills)?,
                resident_before: known(before.used_bytes)?,
                resident_after: known(after.used_bytes)?,
            });
        }
    }
    let commits: i64 = sqlx::query_scalar("SELECT value FROM pager_control WHERE id=1").fetch_one(writer).await?;
    if commits != 3 { return Err("rollback/commit isolation contract failed".into()); }
    Ok(observations)
}

#[tokio::test]
async fn sqlcipher_same_reader_unrelated_commit_pager_diagnostic() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("owned-pager.db");
    let key = crate::encryption::DbKey::generate();
    let options = crate::encryption::connect_options(&path, Some(&key), true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
        .pragma("recursive_triggers", "ON")
        .pragma("cache_size", "-16384")
        .busy_timeout(Duration::from_secs(2));
    let mut writer = SqliteConnection::connect_with(&options).await.unwrap();
    sqlx::query("CREATE TABLE pager_payload(id INTEGER PRIMARY KEY, payload BLOB NOT NULL)").execute(&mut writer).await.unwrap();
    sqlx::query("CREATE TABLE pager_control(id INTEGER PRIMARY KEY, value INTEGER NOT NULL)").execute(&mut writer).await.unwrap();
    sqlx::query("INSERT INTO pager_control VALUES(1,0)").execute(&mut writer).await.unwrap();
    let mut expected = Sha256::new();
    let mut transaction = writer.begin().await.unwrap();
    for id in 0..ROWS {
        let payload = vec![(id % 251) as u8; ROW_BYTES];
        expected.update(&payload);
        sqlx::query("INSERT INTO pager_payload VALUES(?,?)").bind(id as i64).bind(payload)
            .execute(&mut *transaction).await.unwrap();
    }
    transaction.commit().await.unwrap();
    let expected: [u8;32] = expected.finalize().into();
    let mut reader = SqliteConnection::connect_with(&options.create_if_missing(false)).await.unwrap();
    // Both handles are retained, never returned/reopened between samples.
    // Timeout drops only the borrowed diagnostic future; close both actual
    // connections before propagating any timeout/error/assertion to the test.
    let outcome = tokio::time::timeout(Duration::from_secs(30), observe(&mut reader, &mut writer, &expected)).await;
    let reader_closed = reader.close().await;
    let writer_closed = writer.close().await;
    reader_closed.unwrap(); writer_closed.unwrap();
    let observations = outcome.expect("bounded owned diagnostic timed out").expect("diagnostic contract failed");
    assert_eq!(observations.len(), 9);
    let controls_clean = observations.iter().filter(|row| row.control != "commit").all(|row| row.misses == 0);
    let commit_misses = observations.iter().filter(|row| row.control == "commit").all(|row| row.misses > 0);
    let finding = if controls_clean && commit_misses { "commit-associated-pager-misses" } else { "inconclusive" };
    eprintln!("owned-sqlcipher-pager-observations {}", serde_json::json!({
        "version":1,"finding":finding,"readerReopened":false,"counterReset":false,
        "payloadBytes":ROWS*ROW_BYTES,"cacheTargetKiB":16384,
        "cipherVerified":true,"hmacVerified":true,"walVerified":true,
        "physicalDiskClaim":false,"allPagesEvictedClaim":false,"observations":observations
    }));
}

#[test]
fn pager_counter_delta_rejects_unknown_and_reset_instead_of_inventing_zero() {
    assert_eq!(delta(Some(4),Some(9)).unwrap(),5);
    assert!(delta(None,Some(9)).is_err());
    assert!(delta(Some(4),None).is_err());
    assert!(delta(Some(9),Some(4)).is_err());
}
