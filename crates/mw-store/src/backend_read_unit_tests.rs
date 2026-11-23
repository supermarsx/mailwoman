//! Read units: one acquisition (and on SQLite one worker command) for several
//! independent reads, with the same rows as separate statements. Also BEGIN
//! IMMEDIATE cancellation without an owning task.
use super::*;
use crate::pool_telemetry::measure_queries;

async fn fixture(backend: &Backend) {
    q("CREATE TEMP TABLE unit_probe(id BIGINT PRIMARY KEY, label TEXT NOT NULL, owner TEXT)")
        .execute(backend)
        .await
        .unwrap();
    for (id, label, owner) in [(1i64, "a", Some("x")), (2, "b", None), (3, "c", Some("x")), (4, "d", Some("y"))] {
        q("INSERT INTO unit_probe(id,label,owner) VALUES(?1,?2,?3)")
            .bind(id)
            .bind(label)
            .bind(owner.map(str::to_owned))
            .execute(backend)
            .await
            .unwrap();
    }
}
fn reads() -> Vec<Sql> {
    vec![
        q("SELECT id,label FROM unit_probe WHERE owner=?1 OR id=?2 ORDER BY id")
            .bind("x")
            .bind(4i64),
        // No rows: the alignment of later results must hold.
        q("SELECT id,label FROM unit_probe WHERE label=?1").bind("missing"),
        q("SELECT COUNT(*) AS n FROM unit_probe WHERE id>?1 AND (owner IS NULL OR owner<>?2)")
            .bind(1i64)
            .bind("z"),
        q("SELECT label FROM unit_probe WHERE id=?1;").bind(2i64),
    ]
}
fn cell(row: &Row, i: usize) -> String {
    match row {
        Row::Sqlite(r) => r
            .try_get::<Option<String>, _>(i)
            .ok()
            .flatten()
            .or_else(|| r.try_get::<i64, _>(i).ok().map(|n| n.to_string()))
            .unwrap_or_default(),
        Row::Postgres(r) => r
            .try_get::<Option<String>, _>(i)
            .ok()
            .flatten()
            .or_else(|| r.try_get::<i64, _>(i).ok().map(|n| n.to_string()))
            .unwrap_or_default(),
    }
}
fn width(row: &Row) -> usize {
    match row {
        Row::Sqlite(r) => r.len(),
        Row::Postgres(r) => r.len(),
    }
}
fn flatten(rows: &[Vec<Row>]) -> Vec<Vec<Vec<String>>> {
    rows.iter()
        .map(|set| {
            set.iter()
                .map(|row| (0..width(row)).map(|i| cell(row, i)).collect())
                .collect()
        })
        .collect()
}
async fn exercise(backend: Backend) {
    fixture(&backend).await;
    let (separate, separate_counts) = measure_queries(async {
        let mut out = Vec::new();
        for read in reads() {
            out.push(read.fetch_all(&backend).await.unwrap());
        }
        out
    })
    .await;
    let (unit, unit_counts) = measure_queries(backend.fetch_all_unit(reads())).await;
    let unit = unit.unwrap();
    assert_eq!(flatten(&unit), flatten(&separate));
    assert_eq!(unit.len(), 4);
    assert_eq!(unit[0].len(), 3);
    assert!(unit[1].is_empty());
    assert_eq!(unit[2][0].get_i64("n"), 3);
    assert_eq!(unit[3][0].get_string("label"), "b");
    // Each statement is still counted; the unit takes one connection.
    assert_eq!(separate_counts.read.calls, 4);
    assert_eq!(separate_counts.acquire.calls, 4);
    assert_eq!(unit_counts.read.calls, 4);
    assert_eq!(unit_counts.acquire.calls, 1);
    assert!(
        backend
            .fetch_all_unit(vec![q("SELECT 1"), q("DELETE FROM unit_probe")])
            .await
            .is_err()
    );
    assert_eq!(
        q("SELECT COUNT(*) AS n FROM unit_probe")
            .fetch_one(&backend)
            .await
            .unwrap()
            .get_i64("n"),
        4,
        "a rejected unit runs nothing"
    );
    assert!(backend.fetch_all_unit(Vec::new()).await.unwrap().is_empty());
    match backend {
        Backend::Sqlite(pool) => pool.close().await,
        Backend::Postgres(pool) => pool.close().await,
    }
}

#[test]
fn unit_text_maps_every_placeholder_to_its_argument() {
    static A: &str = "SELECT ?1, ?2, ?1";
    static B: &str = "  SELECT x FROM t WHERE y = ?1 AND z = ?10 ;  ";
    static C: &str = "SELECT '?' || ?1";
    let unit = sqlite_unit_text(&[A, B, C]);
    assert_eq!(
        unit.text,
        "SELECT ?, ?, ?;
SELECT x FROM t WHERE y = ? AND z = ?;
SELECT '?' || ?;
"
    );
    assert_eq!(unit.slots, vec![vec![0, 1, 0], vec![0, 9], vec![0]]);
    // Cached: the same text is returned for the same statements.
    assert_eq!(sqlite_unit_text(&[A, B, C]).text.as_ptr(), unit.text.as_ptr());
}

/// Reused, gapped and out-of-order numbers bind the right values when a
/// statement follows others (SQLx counts unused numbered slots positionally).
#[tokio::test]
async fn sqlite_unit_binds_reused_and_late_numbers() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    let backend = Backend::Sqlite(pool.into());
    let set = || {
        vec![
            q("SELECT ?2 AS a, ?1 AS b, ?2 AS c").bind(1i64).bind(2i64),
            q("SELECT ?3 AS a, ?1 AS b").bind(10i64).bind(20i64).bind(30i64),
            q("SELECT ?1 AS a").bind(7i64),
            // ?2 has no argument: NULL, as for a separate statement.
            q("SELECT ?2 IS NULL AS a").bind(1i64),
        ]
    };
    let rows = backend.fetch_all_unit(set()).await.unwrap();
    let mut separate = Vec::new();
    for read in set() {
        separate.push(read.fetch_all(&backend).await.unwrap());
    }
    assert_eq!(flatten(&rows), flatten(&separate));
    assert_eq!(
        flatten(&rows),
        vec![
            vec![vec!["2".to_owned(), "1".to_owned(), "2".to_owned()]],
            vec![vec!["30".to_owned(), "10".to_owned()]],
            vec![vec!["7".to_owned()]],
            vec![vec!["1".to_owned()]],
        ]
    );
}

#[tokio::test]
async fn sqlite_read_unit_matches_separate_reads_with_one_acquisition() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    exercise(Backend::Sqlite(pool.into())).await;
}

#[tokio::test]
async fn postgres_read_unit_matches_separate_reads_with_one_acquisition() {
    let Ok(url) = std::env::var("MW_TEST_PG") else {
        mw_test_gate::skip("read unit PostgreSQL leg: MW_TEST_PG not configured");
        return;
    };
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(&url)
        .await
        .unwrap();
    exercise(Backend::Postgres(pool)).await;
}

/// BEGIN IMMEDIATE without an owning task: cancelling while the BEGIN waits
/// for the write lock leaves no transaction behind, and commit/rollback work.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cancelled_begin_waiting_for_the_write_lock_leaves_no_transaction() {
    use sqlx::TransactionManager;
    use std::time::Duration;
    let dir = tempfile::tempdir().unwrap();
    let options = sqlx::sqlite::SqliteConnectOptions::new()
        .filename(dir.path().join("begin.db"))
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .busy_timeout(Duration::from_secs(5));
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(2)
        .connect_with(options)
        .await
        .unwrap();
    sqlx::query("CREATE TABLE t(n INTEGER)")
        .execute(&pool)
        .await
        .unwrap();
    let backend = Backend::Sqlite(crate::sqlite_live::LiveSqlite::unmanaged(pool.clone()));
    for _ in 0..4 {
        let holder = backend.begin().await.unwrap();
        let waiting = tokio::time::timeout(Duration::from_millis(60), backend.begin()).await;
        assert!(waiting.is_err(), "the second BEGIN IMMEDIATE must wait for the lock");
        drop(holder);
        // The cancelled waiter must not keep or later take the write lock.
        let next = tokio::time::timeout(Duration::from_secs(2), backend.begin())
            .await
            .expect("write lock released")
            .unwrap();
        drop(next);
        for _ in 0..2 {
            let connection = tokio::time::timeout(Duration::from_secs(2), pool.acquire())
                .await
                .unwrap()
                .unwrap();
            assert_eq!(
                sqlx::sqlite::SqliteTransactionManager::get_transaction_depth(&connection),
                0
            );
            drop(connection);
        }
    }
    let mut tx = backend.begin().await.unwrap();
    q("INSERT INTO t(n) VALUES(?1)")
        .bind(1i64)
        .execute_tx(&mut tx)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    let mut tx = backend.begin().await.unwrap();
    q("INSERT INTO t(n) VALUES(?1)")
        .bind(2i64)
        .execute_tx(&mut tx)
        .await
        .unwrap();
    tx.rollback().await.unwrap();
    assert_eq!(
        q("SELECT COUNT(*) AS n FROM t")
            .fetch_one(&backend)
            .await
            .unwrap()
            .get_i64("n"),
        1
    );
    pool.close().await;
}

