//! Background WAL checkpointing: request commits never checkpoint, the WAL stays
//! bounded under sustained writes, and close still checkpoints and removes it.
use super::*;
use crate::{Store, backend::{Backend, q}, encryption::EncryptionConfig};

async fn file_store(dir: &Path) -> (Store, PathBuf) {
    let path = dir.join("mail.db");
    let store = Store::open_with_encryption(
        path.to_str().unwrap(),
        crate::ServerKey::generate(),
        &EncryptionConfig::off(),
    )
    .await
    .unwrap();
    (store, path)
}
fn live(store: &Store) -> &LiveSqlite {
    let Backend::Sqlite(live) = &store.backend else {
        unreachable!()
    };
    live
}
fn wal_len(path: &Path) -> u64 {
    std::fs::metadata(wal_file(path)).map(|m| m.len()).unwrap_or(0)
}
fn db_len(path: &Path) -> u64 {
    std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
}
/// Wait until the checkpointer has been quiet for a full settle period.
async fn quiet(live: &LiveSqlite) -> u64 {
    let mut last = live.wal_checkpoint_stats().rounds;
    loop {
        tokio::time::sleep(WAL_MTIME_SETTLE + WAL_CHECKPOINT_TICK * 2).await;
        let now = live.wal_checkpoint_stats().rounds;
        if now == last {
            return now;
        }
        last = now;
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn request_connections_never_auto_checkpoint_and_background_bounds_the_wal() {
    let dir = tempfile::tempdir().unwrap();
    let (store, path) = file_store(dir.path()).await;
    let live = live(&store);
    let stats = live.wal_checkpoint_stats();
    assert!(stats.enabled && stats.running, "{stats:?}");
    {
        let mut connection = live.acquire().await.unwrap();
        let autocheckpoint: i64 = sqlx::query_scalar("PRAGMA wal_autocheckpoint")
            .fetch_one(&mut *connection)
            .await
            .unwrap();
        assert_eq!(autocheckpoint, 0, "a request commit must not run the WAL checkpoint");
        let synchronous: i64 = sqlx::query_scalar("PRAGMA synchronous")
            .fetch_one(&mut *connection)
            .await
            .unwrap();
        assert_eq!(synchronous, 1, "durability stays synchronous=NORMAL");
    }
    q("CREATE TABLE wal_load(id INTEGER PRIMARY KEY, payload BLOB NOT NULL)")
        .execute(&store.backend)
        .await
        .unwrap();
    // A low restart threshold so the written volume is several times the limit.
    let restart_frames = 512;
    live.set_wal_restart_frames(restart_frames);
    // Sustained writes: 5,000 transactions of 16 KiB each (about 80 MiB of row
    // data, over 20,000 WAL frames). Without any checkpoint the WAL file would
    // hold all of them; PASSIVE alone would not let it restart.
    let payload = vec![7u8; 16384];
    let mut written = 0u64;
    let mut largest = 0u64;
    for n in 0..5000i64 {
        let mut tx = store.backend.begin().await.unwrap();
        q("INSERT INTO wal_load(id,payload) VALUES(?1,?2)")
            .bind(n)
            .bind(payload.clone())
            .execute_tx(&mut tx)
            .await
            .unwrap();
        tx.commit().await.unwrap();
        written += payload.len() as u64;
        if n % 50 == 0 {
            largest = largest.max(wal_len(&path));
            // Give the background task scheduling room in a debug build.
            tokio::task::yield_now().await;
        }
    }
    largest = largest.max(wal_len(&path));
    let stats = live.wal_checkpoint_stats();
    assert!(stats.rounds > 0 && stats.restarts > 0, "{stats:?}");
    // The WAL holds about the restart threshold plus what is written during one
    // round, independent of the total volume. The round length of a debug
    // build on a busy host sets the slack, so compare with the volume.
    eprintln!("WAL high-water {largest} bytes for {written} bytes written; {stats:?}");
    assert!(
        largest < written / 3,
        "WAL reached {largest} bytes for {written} bytes written; {stats:?}"
    );
    // After the writes stop, later rounds copy the remaining frames into the
    // database file (no probe connection: it could checkpoint by itself).
    let mut settled = false;
    for _ in 0..40 {
        tokio::time::sleep(WAL_CHECKPOINT_TICK).await;
        if db_len(&path) >= written {
            settled = true;
            break;
        }
    }
    assert!(settled, "the database file never received the written rows");
    store.close().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn untracked_writes_are_found_by_the_wal_modification_time() {
    let dir = tempfile::tempdir().unwrap();
    let (store, path) = file_store(dir.path()).await;
    let live = live(&store);
    let before = quiet(live).await;
    let size = db_len(&path);
    // Raw SQLx writes bypass the store's write counter. 1 MiB stays in the WAL
    // until a checkpoint copies it into the database file.
    sqlx::query("CREATE TABLE raw_write(payload BLOB); INSERT INTO raw_write VALUES(zeroblob(1048576))")
        .execute(live)
        .await
        .unwrap();
    let mut settled = false;
    for _ in 0..40 {
        tokio::time::sleep(WAL_CHECKPOINT_TICK).await;
        if db_len(&path) >= size + 1048576 {
            settled = true;
            break;
        }
    }
    assert!(settled, "untracked WAL frames were never checkpointed");
    assert!(live.wal_checkpoint_stats().rounds > before);
    assert_eq!(live.wal_checkpoint_stats().failures, 0);
    store.close().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn close_checkpoints_removes_the_wal_and_stops_the_checkpointer() {
    let dir = tempfile::tempdir().unwrap();
    let (store, path) = file_store(dir.path()).await;
    q("CREATE TABLE kept(n INTEGER)")
        .execute(&store.backend)
        .await
        .unwrap();
    for n in 0..200i64 {
        q("INSERT INTO kept VALUES(?1)")
            .bind(n)
            .execute(&store.backend)
            .await
            .unwrap();
    }
    let live = live(&store).clone();
    store.close().await;
    assert_eq!(
        wal_len(&path),
        0,
        "closing the last connection checkpoints the WAL into the database"
    );
    let mut stopped = false;
    for _ in 0..20 {
        if !live.wal_checkpoint_stats().running {
            stopped = true;
            break;
        }
        tokio::time::sleep(WAL_CHECKPOINT_TICK).await;
    }
    assert!(stopped, "the checkpointer must end with its pool");
    drop(live);
    drop(store);
    // The database file alone holds every committed row.
    let options = SqliteConnectOptions::new().filename(&path).read_only(true);
    let mut connection = <SqliteConnection as sqlx::Connection>::connect_with(&options)
        .await
        .unwrap();
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM kept")
        .fetch_one(&mut connection)
        .await
        .unwrap();
    assert_eq!(count, 200);
    sqlx::Connection::close(connection).await.unwrap();
}

#[tokio::test]
async fn private_and_in_memory_pools_keep_sqlite_auto_checkpoint() {
    let store = Store::open_in_memory(crate::ServerKey::generate()).await.unwrap();
    let live = live(&store);
    assert_eq!(live.wal_checkpoint_stats(), WalCheckpointStats::default());
    let mut connection = live.acquire().await.unwrap();
    let autocheckpoint: i64 = sqlx::query_scalar("PRAGMA wal_autocheckpoint")
        .fetch_one(&mut *connection)
        .await
        .unwrap();
    assert_eq!(autocheckpoint, 1000);
}
