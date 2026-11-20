//! Recover/open SQLite before applying the normal pool's acquisition deadline.
//! A worker doing crash recovery must finish/close before startup ownership ends.
use sqlx::{Connection, SqliteConnection, sqlite::SqliteConnectOptions};
use std::{future::Future, time::Duration};

const DEFAULT_SECONDS: u64 = 1800;

fn budget(value: Option<&str>) -> Result<Duration, sqlx::Error> {
    let seconds = value
        .map_or(Ok(DEFAULT_SECONDS), str::parse::<u64>)
        .map_err(|_| {
            sqlx::Error::Protocol(
                "MW_SQLITE_STARTUP_TIMEOUT_SECONDS must be an integer from 1 to 86400".into(),
            )
        })?;
    if !(1..=86400).contains(&seconds) {
        return Err(sqlx::Error::Protocol(
            "MW_SQLITE_STARTUP_TIMEOUT_SECONDS must be an integer from 1 to 86400".into(),
        ));
    }
    Ok(Duration::from_secs(seconds))
}

/// Only a new generation owner calls this. Do not probe an already live database
/// through unrelated file descriptors: POSIX SQLite locks belong to the process.
pub(crate) async fn prepare(options: &SqliteConnectOptions) -> Result<(), sqlx::Error> {
    let configured = std::env::var("MW_SQLITE_STARTUP_TIMEOUT_SECONDS").ok();
    observe(budget(configured.as_deref())?, async {
        let mut connection = SqliteConnection::connect_with(options).await?;
        let checked = sqlx::query("SELECT schema_version FROM pragma_schema_version")
            .execute(&mut connection)
            .await;
        // Always await worker teardown, even if schema/key verification failed.
        let closed = connection.close().await;
        checked?;
        closed
    })
    .await
}

async fn observe<F>(budget: Duration, work: F) -> Result<(), sqlx::Error>
where
    F: Future<Output = Result<(), sqlx::Error>>,
{
    let started = tokio::time::Instant::now();
    eprintln!(
        "[mw-store] SQLite startup: opening/recovering database before pool initialization; recovery budget {}s",
        budget.as_secs()
    );
    let mut work = std::pin::pin!(work);
    let mut deadline = std::pin::pin!(tokio::time::sleep(budget));
    let mut ticks = tokio::time::interval(Duration::from_secs(5));
    ticks.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    ticks.tick().await;
    let mut expired = false;
    loop {
        tokio::select! {
            result = &mut work => {
                let elapsed = started.elapsed();
                if let Err(error) = &result {
                    eprintln!("[mw-store] SQLite startup: opening/recovery failed after {}s: {error}", elapsed.as_secs());
                    return result;
                }
                if expired || elapsed > budget {
                    eprintln!("[mw-store] SQLite startup: recovery worker closed after {}s; startup budget exceeded", elapsed.as_secs());
                    return Err(sqlx::Error::Protocol(format!("SQLite startup recovery exceeded its {}s budget; worker has finished and closed. Keep database journal/WAL files intact; review disk activity and MW_SQLITE_STARTUP_TIMEOUT_SECONDS", budget.as_secs())));
                }
                eprintln!("[mw-store] SQLite startup: recovery/open check complete in {}s; initializing normal connection pool", elapsed.as_secs());
                return Ok(());
            }
            _ = &mut deadline, if !expired => {
                expired = true;
                // Dropping this future could strand SQLite's blocking worker in
                // recovery after the generation lease is released. Keep polling.
                eprintln!("[mw-store] SQLite startup: recovery budget exceeded; waiting for SQLite to finish and close safely before failing startup. Do not delete journal/WAL files");
            }
            _ = ticks.tick() => eprintln!("[mw-store] SQLite startup: opening/recovering database; {}s elapsed; waiting on SQLite recovery/locks/I/O, not serving requests yet", started.elapsed().as_secs()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };

    #[test]
    fn startup_budget_is_separate_and_bounded() {
        assert_eq!(budget(None).unwrap(), Duration::from_secs(1800));
        assert_eq!(budget(Some("3600")).unwrap(), Duration::from_secs(3600));
        for value in ["0", "86401", "-1", "garbage", ""] {
            assert!(budget(Some(value)).is_err());
        }
    }

    #[tokio::test]
    async fn slow_initialization_can_look_like_pool_exhaustion_without_borrowers() {
        let result = SqlitePoolOptions::new()
            .max_connections(1)
            .acquire_timeout(Duration::from_millis(10))
            .after_connect(|_, _| {
                Box::pin(async {
                    tokio::time::sleep(Duration::from_millis(80)).await;
                    Ok(())
                })
            })
            .connect("sqlite::memory:")
            .await;
        assert!(matches!(result, Err(sqlx::Error::PoolTimedOut)));
    }

    #[tokio::test]
    async fn elapsed_budget_does_not_abandon_active_recovery_worker() {
        let closed = Arc::new(AtomicBool::new(false));
        let done = closed.clone();
        let result = observe(Duration::from_millis(10), async move {
            tokio::time::sleep(Duration::from_millis(60)).await;
            done.store(true, Ordering::SeqCst);
            Ok(())
        })
        .await;
        assert!(closed.load(Ordering::SeqCst));
        assert!(
            matches!(result, Err(sqlx::Error::Protocol(message)) if message.contains("startup recovery exceeded"))
        );
    }

    #[tokio::test]
    async fn recovery_check_preserves_normal_pool_acquisition_budget() {
        let directory = std::env::temp_dir().join(format!("mw-startup-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&directory).unwrap();
        let options = SqliteConnectOptions::new()
            .filename(directory.join("test.db"))
            .create_if_missing(true);
        prepare(&options).await.unwrap();
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .acquire_timeout(Duration::from_millis(25))
            .connect_with(options)
            .await
            .unwrap();
        let held = pool.acquire().await.unwrap();
        assert_eq!(
            pool.options().get_acquire_timeout(),
            Duration::from_millis(25)
        );
        assert!(matches!(
            pool.acquire().await,
            Err(sqlx::Error::PoolTimedOut)
        ));
        drop(held);
        pool.close().await;
        std::fs::remove_file(directory.join("test.db")).unwrap();
        std::fs::remove_dir(directory).unwrap();
    }

    #[tokio::test]
    #[ignore = "private subprocess used only by the recovery regression"]
    async fn crash_recovery_child() {
        let path =
            std::path::PathBuf::from(std::env::var_os("MW_TEST_STARTUP_CRASH_PATH").unwrap());
        assert_eq!(path.file_name().unwrap(), "test.db");
        assert!(
            path.parent()
                .unwrap()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("mw-startup-recovery-")
        );
        assert_eq!(
            path.parent().unwrap().parent().unwrap(),
            std::env::temp_dir()
        );
        let options = SqliteConnectOptions::new()
            .filename(&path)
            .pragma("cache_size", "2");
        let mut connection = SqliteConnection::connect_with(&options).await.unwrap();
        sqlx::query("BEGIN IMMEDIATE")
            .execute(&mut connection)
            .await
            .unwrap();
        sqlx::query("UPDATE samples SET value=randomblob(131072)")
            .execute(&mut connection)
            .await
            .unwrap();
        // Deliberately skip destructors: leave a real hot rollback journal after
        // dirty pages spill. This child owns only its freshly created test DB.
        std::process::exit(17);
    }

    #[tokio::test]
    async fn opening_recovers_a_hot_journal_and_preserves_committed_rows() {
        let directory =
            std::env::temp_dir().join(format!("mw-startup-recovery-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&directory).unwrap();
        let path = directory.join("test.db");
        let options = SqliteConnectOptions::new()
            .filename(&path)
            .create_if_missing(true);
        let mut connection = SqliteConnection::connect_with(&options).await.unwrap();
        sqlx::query("CREATE TABLE samples(n INTEGER PRIMARY KEY,value BLOB)")
            .execute(&mut connection)
            .await
            .unwrap();
        sqlx::query("WITH RECURSIVE n(x) AS (VALUES(1) UNION ALL SELECT x+1 FROM n WHERE x<12) INSERT INTO samples SELECT x,zeroblob(131072) FROM n").execute(&mut connection).await.unwrap();
        connection.close().await.unwrap();
        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "sqlite_startup::tests::crash_recovery_child",
                "--exact",
                "--ignored",
            ])
            .env("MW_TEST_STARTUP_CRASH_PATH", &path)
            .status()
            .unwrap();
        assert_eq!(status.code(), Some(17));
        assert!(directory.join("test.db-journal").exists());
        let recovered_options = options.journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);
        prepare(&recovered_options).await.unwrap();
        let mut connection = SqliteConnection::connect_with(&recovered_options)
            .await
            .unwrap();
        let preserved: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM samples WHERE value=zeroblob(131072)")
                .fetch_one(&mut connection)
                .await
                .unwrap();
        assert_eq!(preserved, 12);
        let integrity: String = sqlx::query_scalar("PRAGMA integrity_check")
            .fetch_one(&mut connection)
            .await
            .unwrap();
        assert_eq!(integrity, "ok");
        connection.close().await.unwrap();
        std::fs::remove_file(path).unwrap();
        std::fs::remove_dir(directory).unwrap();
    }
}
