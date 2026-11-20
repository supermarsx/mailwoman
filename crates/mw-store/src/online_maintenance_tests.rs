use super::*;
use crate::{
    ServerKey,
    backend::q,
    encryption::{self, DbKey, EncryptionConfig, EncryptionMode},
};

struct Fixture {
    store: Store,
    dir: PathBuf,
    path: PathBuf,
    config: EncryptionConfig,
}
impl Fixture {
    async fn new(encrypted: bool, incremental: bool) -> Self {
        let dir =
            std::env::temp_dir().join(format!("mw-conversion-fixture-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&dir).unwrap();
        let path = dir.join("mailwoman.db");
        let config = if encrypted {
            EncryptionConfig::with_key(EncryptionMode::Required, DbKey::generate())
        } else {
            EncryptionConfig::off()
        };
        // Install the real schema before Store opens it, preserving legacy NONE.
        let raw = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(
                encryption::connect_options(&path, config.key.as_ref(), true)
                    .journal_mode(SqliteJournalMode::Wal)
                    .pragma("recursive_triggers", "ON"),
            )
            .await
            .unwrap();
        if incremental {
            sqlx::query("PRAGMA auto_vacuum=INCREMENTAL")
                .execute(&raw)
                .await
                .unwrap();
            sqlx::query("VACUUM").execute(&raw).await.unwrap();
        }
        sqlx::migrate!("./migrations").run(&raw).await.unwrap();
        sqlx::query("CREATE TABLE conversion_probe(id INTEGER PRIMARY KEY,value BLOB NOT NULL)")
            .execute(&raw)
            .await
            .unwrap();
        raw.close().await;
        let store =
            Store::open_with_encryption(path.to_str().unwrap(), ServerKey::generate(), &config)
                .await
                .unwrap();
        Self {
            store,
            dir,
            path,
            config,
        }
    }
    fn live(&self) -> LiveSqlite {
        let Backend::Sqlite(live) = &self.store.backend else {
            unreachable!()
        };
        live.clone()
    }
    async fn finish(&self) -> ConversionReport {
        tokio::time::timeout(Duration::from_secs(90), async {
            loop {
                let report = self.store.online_conversion_status().unwrap();
                if terminal(&report.phase) {
                    return report;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .expect("conversion terminated within fixture deadline")
    }
    async fn clean(self) {
        self.store.close().await;
        drop(self.store);
        for attempt in 0..100 {
            match std::fs::remove_dir_all(&self.dir) {
                Ok(()) => return,
                Err(error) if attempt == 99 => panic!("fixture cleanup: {error}"),
                Err(_) => tokio::time::sleep(Duration::from_millis(10)).await,
            }
        }
    }
}
fn fast_options() -> ConversionOptions {
    ConversionOptions {
        batch_rows: 128,
        pause_ms: 0,
        max_cutover_ms: 10_000,
        ..Default::default()
    }
}
async fn mode(store: &Store) -> i64 {
    q("PRAGMA auto_vacuum")
        .fetch_scalar_i64(&store.backend)
        .await
        .unwrap()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn full_schema_conversion_preserves_concurrent_writes_and_restart_generation() {
    let fixture = Fixture::new(false, false).await;
    assert_eq!(mode(&fixture.store).await, 0);
    let pool_before = fixture.store.database_pool_snapshot().await;
    let clone = fixture.store.clone();
    fixture
        .store
        .start_online_conversion(fast_options())
        .await
        .unwrap();
    let writer = tokio::spawn(async move {
        for id in 0..80i64 {
            q("INSERT INTO conversion_probe(id,value) VALUES(?1,?2)")
                .bind(id)
                .bind(vec![id as u8, 0, 255])
                .execute(&clone.backend)
                .await
                .unwrap();
            if id % 3 == 0 {
                q("UPDATE conversion_probe SET value=?1 WHERE id=?2")
                    .bind(vec![42u8, id as u8])
                    .bind(id)
                    .execute(&clone.backend)
                    .await
                    .unwrap();
            }
            tokio::time::sleep(Duration::from_millis(2)).await;
        }
        q("DELETE FROM conversion_probe WHERE id=17")
            .execute(&clone.backend)
            .await
            .unwrap();
    });
    let report = fixture.finish().await;
    writer.await.unwrap();
    assert_eq!(report.phase, "completed", "{report:?}");
    assert!(report.timing.finished_at.is_some());
    assert!(report.timing.elapsed_ms.is_some_and(|value|value>0));
    assert!(report.timing.copy_elapsed_ms.is_some_and(|value|value>0));
    assert!(report.timing.replay_elapsed_ms.is_some());
    assert!(report.timing.average_copy_bytes_per_second.is_some_and(|value|value.is_finite()&&value>=0.0));
    let persisted=crate::online_control::online_conversion_status_path(&fixture.path).unwrap();
    assert_eq!(persisted.timing.finished_at,report.timing.finished_at);
    assert_eq!(persisted.timing.elapsed_ms,report.timing.elapsed_ms);
    assert_eq!(persisted.timing.copy_elapsed_ms,report.timing.copy_elapsed_ms);
    assert_eq!(persisted.timing.replay_elapsed_ms,report.timing.replay_elapsed_ms);
    // JSON parser round trips can differ by one floating-point ULP.
    for (stored,observed) in [
        (persisted.timing.average_copy_bytes_per_second,report.timing.average_copy_bytes_per_second),
        (persisted.timing.average_replay_keys_per_second,report.timing.average_replay_keys_per_second),
    ] {
        match (stored,observed) {
            (Some(stored),Some(observed))=>assert!((stored-observed).abs()<=observed.abs().max(1.0)*1e-12),
            (None,None)=>{},_=>panic!("persisted rate availability changed"),
        }
    }
    tokio::time::sleep(Duration::from_millis(5)).await;
    assert_eq!(serde_json::to_value(fixture.store.online_conversion_status().unwrap().timing).unwrap(),serde_json::to_value(&report.timing).unwrap(),"terminal duration must not grow during polling");
    assert!(report.retained_previous);
    assert_eq!(mode(&fixture.store).await, 2);
    let pool_after = fixture.store.database_pool_snapshot().await;
    for key in [
        "minimum",
        "maximum",
        "acquireTimeoutMs",
        "idleTimeoutMs",
        "maxLifetimeMs",
    ] {
        assert_eq!(
            pool_after[key], pool_before[key],
            "conversion preserves pool policy: {key}"
        );
    }
    assert_eq!(
        q("SELECT COUNT(*) FROM conversion_probe")
            .fetch_scalar_i64(&fixture.store.backend)
            .await
            .unwrap(),
        79
    );
    let selected = fixture.live().files().unwrap().resolve_active().unwrap();
    assert_ne!(selected.id, "legacy");
    assert_ne!(selected.path, fixture.path);
    let expected = q("SELECT id,value FROM conversion_probe ORDER BY id")
        .fetch_all(&fixture.store.backend)
        .await
        .unwrap();
    fixture.store.close().await;
    let reopened = Store::open_with_encryption(
        fixture.path.to_str().unwrap(),
        fixture.store.key.clone(),
        &fixture.config,
    )
    .await
    .unwrap();
    assert_eq!(mode(&reopened).await, 2);
    let actual = q("SELECT id,value FROM conversion_probe ORDER BY id")
        .fetch_all(&reopened.backend)
        .await
        .unwrap();
    assert_eq!(actual.len(), expected.len());
    for (a, e) in actual.iter().zip(&expected) {
        assert_eq!(a.get_i64("id"), e.get_i64("id"));
        assert_eq!(a.get_blob("value"), e.get_blob("value"));
    }
    reopened.close().await;
    drop(reopened);
    fixture.clean().await;
}

#[tokio::test]
async fn cancellation_keeps_source_authoritative_and_writable() {
    let fixture = Fixture::new(false, false).await;
    fixture
        .store
        .start_online_conversion(fast_options())
        .await
        .unwrap();
    assert!(fixture.store.cancel_online_conversion());
    let report = fixture.finish().await;
    assert_eq!(report.phase, "cancelled", "{report:?}");
    assert!(
        !fixture.live().files().unwrap().conversion_active(),
        "cancellation must permit another conversion without restarting"
    );
    assert_eq!(
        fixture.live().files().unwrap().resolve_active().unwrap().id,
        "legacy"
    );
    assert_eq!(mode(&fixture.store).await, 0);
    q("INSERT INTO conversion_probe VALUES(1,?1)")
        .bind(vec![1u8, 0, 255])
        .execute(&fixture.store.backend)
        .await
        .unwrap();
    assert_eq!(
        q("SELECT COUNT(*) FROM conversion_probe")
            .fetch_scalar_i64(&fixture.store.backend)
            .await
            .unwrap(),
        1
    );
    fixture.clean().await;
}

#[tokio::test]
async fn invalid_options_never_start_and_incremental_database_is_noop() {
    let fixture = Fixture::new(false, true).await;
    let invalid = ConversionOptions {
        batch_rows: 0,
        ..Default::default()
    };
    assert!(
        fixture
            .store
            .start_online_conversion(invalid)
            .await
            .is_err()
    );
    assert!(fixture.store.online_conversion_status().is_none());
    fixture
        .store
        .start_online_conversion(fast_options())
        .await
        .unwrap();
    let report = fixture.finish().await;
    assert_eq!(report.phase, "completed", "{report:?}");
    assert!(!report.retained_previous);
    assert_eq!(
        fixture.live().files().unwrap().resolve_active().unwrap().id,
        "legacy"
    );
    assert_eq!(mode(&fixture.store).await, 2);
    fixture.clean().await;
}

#[tokio::test]
async fn busy_generation_aborts_cutover_without_closing_source() {
    let fixture = Fixture::new(false, false).await;
    let pin = fixture.live().pin().await;
    fixture
        .store
        .start_online_conversion(ConversionOptions {
            max_cutover_ms: 100,
            ..fast_options()
        })
        .await
        .unwrap();
    let report = fixture.finish().await;
    assert_eq!(report.phase, "failed", "{report:?}");
    assert!(
        report
            .error
            .as_deref()
            .is_some_and(|error| error.contains("Busy operations")),
        "{report:?}"
    );
    drop(pin);
    assert_eq!(mode(&fixture.store).await, 0);
    q("INSERT INTO conversion_probe VALUES(1,?1)")
        .bind(vec![7u8])
        .execute(&fixture.store.backend)
        .await
        .unwrap();
    assert_eq!(
        fixture.live().files().unwrap().resolve_active().unwrap().id,
        "legacy"
    );
    fixture.clean().await;
}

#[tokio::test]
async fn sqlcipher_conversion_preserves_encryption_and_reopens_with_same_key() {
    let fixture = Fixture::new(true, false).await;
    q("INSERT INTO conversion_probe VALUES(1,?1)")
        .bind(vec![0u8, 255, 17])
        .execute(&fixture.store.backend)
        .await
        .unwrap();
    fixture
        .store
        .start_online_conversion(fast_options())
        .await
        .unwrap();
    let report = fixture.finish().await;
    assert_eq!(report.phase, "completed", "{report:?}");
    assert_eq!(mode(&fixture.store).await, 2);
    let selected = fixture.live().files().unwrap().resolve_active().unwrap();
    fixture.store.close().await;
    assert_eq!(
        encryption::probe_file(&selected.path).unwrap(),
        encryption::FileState::NotPlaintext
    );
    let reopened = Store::open_with_encryption(
        fixture.path.to_str().unwrap(),
        fixture.store.key.clone(),
        &fixture.config,
    )
    .await
    .unwrap();
    assert_eq!(
        q("SELECT value FROM conversion_probe WHERE id=1")
            .fetch_one(&reopened.backend)
            .await
            .unwrap()
            .get_blob("value"),
        vec![0u8, 255, 17]
    );
    assert_eq!(mode(&reopened).await, 2);
    reopened.close().await;
    drop(reopened);
    fixture.clean().await;
}

#[tokio::test]
async fn initial_status_failure_is_terminal_in_cache_and_can_be_retried() {
    let fixture = Fixture::new(false, true).await;
    let status = status_path(fixture.live().logical_path());
    std::fs::create_dir(&status).unwrap();
    assert!(
        fixture
            .store
            .start_online_conversion(fast_options())
            .await
            .is_err()
    );
    let failed = fixture.store.online_conversion_status().unwrap();
    assert_eq!(failed.phase, "failed");
    assert!(failed.error.is_some());
    assert!(!fixture.store.cancel_online_conversion());
    std::fs::remove_dir(&status).unwrap();
    fixture
        .store
        .start_online_conversion(fast_options())
        .await
        .unwrap();
    assert_eq!(fixture.finish().await.phase, "completed");
    fixture.clean().await;
}

#[tokio::test]
async fn retained_conversion_ownership_preserves_the_last_recovery_record() {
    let fixture = Fixture::new(false, true).await;
    fixture
        .store
        .start_online_conversion(fast_options())
        .await
        .unwrap();
    let previous = fixture.finish().await;
    let files = fixture.live().files().unwrap();
    let exclusive = files.acquire_conversion().unwrap();
    assert!(
        fixture
            .store
            .start_online_conversion(fast_options())
            .await
            .is_err()
    );
    let current = fixture.store.online_conversion_status().unwrap();
    assert_eq!(current.id, previous.id);
    assert_eq!(current.phase, previous.phase);
    drop(exclusive);
    drop(files);
    fixture.clean().await;
}

#[tokio::test]
async fn same_process_recovery_updates_cached_nonterminal_report() {
    let fixture = Fixture::new(false, false).await;
    let path = fixture.live().logical_path().to_path_buf();
    let report = ConversionReport {
        id: uuid::Uuid::new_v4().to_string(),
        phase: "copying".into(),
        started_at: now(),
        updated_at: now(),
        progress: None,
        error: None,
        retained_previous: false,
        source_generation: Some("legacy".into()),
        target_generation: Some(uuid::Uuid::new_v4().to_string()),
        timing: ConversionTiming::default(),
    };
    JOBS.lock().unwrap_or_else(|p| p.into_inner()).insert(
        path.clone(),
        Arc::new(Job {
            report: Mutex::new(report),
            cancel: Arc::new(AtomicBool::new(false)),
            running: AtomicBool::new(false),
            last_write: Mutex::new(Instant::now()),
            timing: Mutex::new(JobTiming::new(Instant::now())),
        }),
    );
    fixture.store.recover_online_conversion().await.unwrap();
    let cached = fixture.store.online_conversion_status().unwrap();
    assert_eq!(cached.phase, "failed");
    assert_eq!(cached.timing.finished_at,None,"recovery cannot infer the interrupted worker's completion time");
    assert_eq!(cached.timing.elapsed_ms,None,"legacy status has no measured duration");
    assert!(!fixture.store.cancel_online_conversion());
    let persisted: ConversionReport = serde_json::from_slice(
        &generation_files::bounded_read(&status_path(&path), 65536).unwrap(),
    )
    .unwrap();
    assert_eq!(persisted.phase, cached.phase);
    assert_eq!(persisted.id, cached.id);
    fixture.clean().await;
}

#[tokio::test]
async fn startup_recovery_removes_only_closed_abandoned_shadow_and_capture() {
    let fixture = Fixture::new(false, false).await;
    q("INSERT INTO conversion_probe VALUES(1,?1)")
        .bind(vec![7u8, 0, 255])
        .execute(&fixture.store.backend)
        .await
        .unwrap();
    let live = fixture.live();
    let files = live.files().unwrap();
    let source = files.resolve_active().unwrap();
    let target = files.reserve_generation().unwrap();
    let source_options = {
        let pin = live.pin().await;
        (*pin.active_options()).clone()
    };
    let mut session = CopySession::prepare(
        source_options.clone(),
        source_options.filename(&target.path),
        CopyOptions::default(),
    )
    .await
    .unwrap();
    let progress = session.seed_batch().await.unwrap();
    // Model process termination only after worker shutdown is confirmed. Leave
    // source capture installed, as a genuinely interrupted copy would do.
    session.close_target().await.unwrap();
    session.close_source().await.unwrap();
    drop(session);
    assert!(target.path.exists());
    assert!(
        q("SELECT COUNT(*) FROM sqlite_schema WHERE name GLOB '__mw_copy_*'")
            .fetch_scalar_i64(&fixture.store.backend)
            .await
            .unwrap()
            > 0
    );
    // SQLite may remove its sidecars during a clean close. Closed stand-ins
    // ensure startup also exercises cleanup of all exact-owned sidecar names.
    for suffix in ["-wal", "-shm", "-journal"] {
        std::fs::write(sidecar(&target.path, suffix), b"abandoned sidecar").unwrap();
    }
    let report = ConversionReport {
        id: uuid::Uuid::new_v4().to_string(),
        phase: "copying".into(),
        started_at: now(),
        updated_at: now(),
        progress: Some(progress),
        error: None,
        retained_previous: false,
        source_generation: Some(source.id.clone()),
        target_generation: Some(target.id.clone()),
        timing: ConversionTiming::default(),
    };
    generation_files::private_atomic_write(
        &status_path(live.logical_path()),
        &serde_json::to_vec(&report).unwrap(),
    )
    .unwrap();
    assert!(
        !JOBS
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .contains_key(live.logical_path())
    );
    fixture.store.recover_online_conversion().await.unwrap();
    assert!(!target.path.exists());
    for suffix in ["-wal", "-shm", "-journal"] {
        assert!(!sidecar(&target.path, suffix).exists());
    }
    assert_eq!(files.resolve_active().unwrap(), source);
    assert_eq!(source.id, "legacy");
    assert_eq!(mode(&fixture.store).await, 0);
    assert_eq!(
        q("SELECT COUNT(*) FROM sqlite_schema WHERE name GLOB '__mw_copy_*'")
            .fetch_scalar_i64(&fixture.store.backend)
            .await
            .unwrap(),
        0
    );
    q("UPDATE conversion_probe SET value=?1 WHERE id=1")
        .bind(vec![42u8])
        .execute(&fixture.store.backend)
        .await
        .unwrap();
    assert_eq!(
        q("SELECT value FROM conversion_probe WHERE id=1")
            .fetch_one(&fixture.store.backend)
            .await
            .unwrap()
            .get_blob("value"),
        vec![42]
    );
    let recovered = fixture.store.online_conversion_status().unwrap();
    assert_eq!(recovered.phase, "failed");
    assert_eq!(recovered.id, report.id);
    assert!(recovered.error.is_some());
    drop(files);
    drop(live);
    fixture.clean().await;
}

#[tokio::test]
async fn startup_recovery_keeps_already_published_target_and_its_new_writes() {
    let fixture = Fixture::new(false, false).await;
    fixture
        .store
        .start_online_conversion(fast_options())
        .await
        .unwrap();
    let mut report = fixture.finish().await;
    assert_eq!(report.phase, "completed", "{report:?}");
    let live = fixture.live();
    let selected = live.files().unwrap().resolve_active().unwrap();
    q("INSERT INTO conversion_probe VALUES(123,?1)")
        .bind(vec![99u8])
        .execute(&fixture.store.backend)
        .await
        .unwrap();
    // A crash after publication but before completion status must not remove
    // CURRENT's target or fall back to the now-stale legacy database.
    report.phase = "publishing".into();
    JOBS.lock()
        .unwrap_or_else(|p| p.into_inner())
        .remove(live.logical_path());
    generation_files::private_atomic_write(
        &status_path(live.logical_path()),
        &serde_json::to_vec(&report).unwrap(),
    )
    .unwrap();
    fixture.store.recover_online_conversion().await.unwrap();
    assert_eq!(live.files().unwrap().resolve_active().unwrap(), selected);
    assert!(selected.path.exists());
    assert_eq!(mode(&fixture.store).await, 2);
    assert_eq!(
        q("SELECT value FROM conversion_probe WHERE id=123")
            .fetch_one(&fixture.store.backend)
            .await
            .unwrap()
            .get_blob("value"),
        vec![99]
    );
    assert_eq!(
        fixture.store.online_conversion_status().unwrap().phase,
        "completed"
    );
    drop(live);
    fixture.clean().await;
}

#[tokio::test]
async fn expired_copy_deadline_can_cleanup_and_confirm_workers_before_shadow_removal() {
    let fixture = Fixture::new(false, false).await;
    q("INSERT INTO conversion_probe VALUES(1,?1)")
        .bind(vec![7u8, 0, 255])
        .execute(&fixture.store.backend)
        .await
        .unwrap();
    let live = fixture.live();
    let pin = live.pin().await;
    let files = live.files().unwrap();
    let source = files.resolve_active().unwrap();
    let target = files.reserve_generation().unwrap();
    let options = (*pin.active_options()).clone();
    let mut session = CopySession::prepare(
        options.clone(),
        options.filename(&target.path),
        CopyOptions::default(),
    )
    .await
    .unwrap();
    while !session.seed_batch().await.unwrap().seed_complete {}
    session.verify_bulk().await.unwrap();
    session
        .set_deadline(Instant::now() - Duration::from_millis(1))
        .await
        .unwrap();
    assert!(
        session.catch_up_batch().await.is_err(),
        "expired VM deadline interrupts schema validation"
    );
    session.clear_deadline().await.unwrap();
    session.cleanup_source().await.unwrap();
    // These awaits, outside any timeout, confirm worker shutdown before unlink.
    session.close_target().await.unwrap();
    session.close_source().await.unwrap();
    files.discard_closed_generation(&source, &target).unwrap();
    assert!(!target.path.exists());
    drop(pin);
    let captures = q("SELECT COUNT(*) FROM sqlite_schema WHERE name GLOB '__mw_copy_*'")
        .fetch_scalar_i64(&fixture.store.backend)
        .await
        .unwrap();
    assert_eq!(captures, 0);
    assert_eq!(mode(&fixture.store).await, 0);
    q("UPDATE conversion_probe SET value=?1 WHERE id=1")
        .bind(vec![42u8])
        .execute(&fixture.store.backend)
        .await
        .unwrap();
    assert_eq!(
        q("SELECT value FROM conversion_probe WHERE id=1")
            .fetch_one(&fixture.store.backend)
            .await
            .unwrap()
            .get_blob("value"),
        vec![42]
    );
    drop(session);
    drop(files);
    drop(live);
    fixture.clean().await;
}
