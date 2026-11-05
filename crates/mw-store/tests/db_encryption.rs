//! Whole-database encryption at rest (SQLCipher with the Rust crypto provider).
//!
//! These tests read the raw bytes of the files the store leaves on disk.

#[path = "../src/test_db.rs"]
mod test_db;

use mw_store::encryption::{
    self, DbKey, EncryptionConfig, EncryptionError, EncryptionMode, FileState, TransformOptions,
};
use mw_store::{ServerKey, Store, StoreError};
use std::path::{Path, PathBuf};

const MARKER: &str = "PLAINTEXT-MARKER-ZEBRA-7731";

fn key(n: u8) -> DbKey {
    DbKey::from_bytes([n; 32])
}

fn cfg(mode: EncryptionMode, k: Option<DbKey>) -> EncryptionConfig {
    EncryptionConfig {
        mode,
        key: k,
        source: None,
    }
}

async fn open(path: &Path, c: &EncryptionConfig) -> Result<Store, StoreError> {
    Store::open_with_encryption(path.to_str().unwrap(), ServerKey::generate(), c).await
}

/// All files SQLite may have left for `db`.
fn artefacts(db: &Path) -> Vec<PathBuf> {
    let mut v = vec![db.to_path_buf()];
    for s in ["-wal", "-shm", "-journal"] {
        let mut p = db.as_os_str().to_owned();
        p.push(s);
        v.push(PathBuf::from(p));
    }
    v
}

/// `Store::close` returns before SQLite has finished its close-time checkpoint; reading
/// the files in that window can miss data that is moving from the WAL to the main file.
fn settle(db: &Path) {
    let wal = artefacts(db).remove(1);
    for _ in 0..40 {
        if !wal.exists() {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}

fn any_contains(db: &Path, needle: &str) -> bool {
    settle(db);
    artefacts(db).iter().any(|p| match std::fs::read(p) {
        Ok(b) => b.windows(needle.len()).any(|w| w == needle.as_bytes()),
        // A file SQLite never created.
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => false,
        // Anything else would make a "not found" result meaningless.
        Err(e) => panic!("cannot read {}: {e}", p.display()),
    })
}

#[tokio::test(flavor = "multi_thread")]
async fn encrypted_file_hides_marker_and_magic_including_wal() {
    let db = test_db::unique_db_path("enc-bytes");
    let c = cfg(EncryptionMode::Required, Some(key(7)));
    let store = open(&db, &c).await.unwrap();
    store.set_setting("probe.marker", MARKER).await.unwrap();
    // Leave the WAL un-checkpointed: the marker must be unreadable there too.
    assert!(
        std::fs::metadata(format!("{}-wal", db.display()))
            .map(|m| m.len() > 0)
            .unwrap_or(false),
        "expected a non-empty WAL while the store is open"
    );
    assert!(!any_contains(&db, MARKER), "marker readable in db or WAL");
    assert_eq!(encryption::probe_file(&db).unwrap(), FileState::NotPlaintext);
    let head = std::fs::read(&db).unwrap();
    assert_ne!(&head[..16], b"SQLite format 3\0");
    assert_eq!(
        store.get_setting("probe.marker").await.unwrap().as_deref(),
        Some(MARKER)
    );
    store.close().await;
    assert!(!any_contains(&db, MARKER));
}

#[tokio::test(flavor = "multi_thread")]
async fn plaintext_control_shows_marker() {
    // Sanity check of the byte scan itself: the same data in a plain file is visible.
    let db = test_db::unique_db_path("plain-bytes");
    let store = open(&db, &cfg(EncryptionMode::Off, None)).await.unwrap();
    store.set_setting("probe.marker", MARKER).await.unwrap();
    store.close().await;
    assert!(any_contains(&db, MARKER));
    assert_eq!(encryption::probe_file(&db).unwrap(), FileState::Plaintext);
}

#[tokio::test(flavor = "multi_thread")]
async fn wrong_or_missing_key_fails_clearly() {
    let db = test_db::unique_db_path("enc-wrong");
    let store = open(&db, &cfg(EncryptionMode::Required, Some(key(1))))
        .await
        .unwrap();
    store.set_setting("a", "b").await.unwrap();
    store.close().await;

    let wrong = open(&db, &cfg(EncryptionMode::Required, Some(key(2))))
        .await
        .err()
        .unwrap();
    assert!(
        matches!(wrong, StoreError::Encryption(EncryptionError::WrongKey(_))),
        "{wrong}"
    );
    let none = open(&db, &cfg(EncryptionMode::Optional, None))
        .await
        .err()
        .unwrap();
    assert!(
        matches!(none, StoreError::Encryption(EncryptionError::EncryptedNoKey(_))),
        "{none}"
    );
    let off = open(&db, &cfg(EncryptionMode::Off, None))
        .await
        .err()
        .unwrap();
    assert!(
        matches!(off, StoreError::Encryption(EncryptionError::Config(_))),
        "{off}"
    );
    let required_nokey = open(&db, &cfg(EncryptionMode::Required, None))
        .await
        .err()
        .unwrap();
    assert!(matches!(
        required_nokey,
        StoreError::Encryption(EncryptionError::KeyMissingButRequired)
    ));
    // The right key still works afterwards.
    let ok = open(&db, &cfg(EncryptionMode::Required, Some(key(1))))
        .await
        .unwrap();
    assert_eq!(ok.get_setting("a").await.unwrap().as_deref(), Some("b"));
    ok.close().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn required_refuses_a_plaintext_database() {
    let db = test_db::unique_db_path("enc-required");
    let store = open(&db, &cfg(EncryptionMode::Off, None)).await.unwrap();
    store.set_setting("a", "b").await.unwrap();
    store.close().await;
    let e = open(&db, &cfg(EncryptionMode::Required, Some(key(3))))
        .await
        .err()
        .unwrap();
    assert!(
        matches!(e, StoreError::Encryption(EncryptionError::PlaintextButRequired(_))),
        "{e}"
    );
    // optional + key keeps opening an existing plaintext file (and does not touch it).
    let s = open(&db, &cfg(EncryptionMode::Optional, Some(key(3))))
        .await
        .unwrap();
    assert_eq!(s.get_setting("a").await.unwrap().as_deref(), Some("b"));
    s.close().await;
    assert_eq!(encryption::probe_file(&db).unwrap(), FileState::Plaintext);
}

async fn seeded_plaintext(prefix: &str) -> PathBuf {
    let db = test_db::unique_db_path(prefix);
    let store = open(&db, &cfg(EncryptionMode::Off, None)).await.unwrap();
    for i in 0..50 {
        store
            .set_setting(&format!("k.{i}"), &format!("{MARKER}-{i}"))
            .await
            .unwrap();
    }
    store.close().await;
    db
}

#[tokio::test(flavor = "multi_thread")]
async fn migrate_plain_to_encrypted_keeps_every_row_then_rekey_then_decrypt() {
    let db = seeded_plaintext("enc-migrate").await;
    let k1 = key(11);
    let k2 = key(22);

    // Dry run changes nothing.
    let dry = encryption::transform(
        &db,
        None,
        Some(&k1),
        &TransformOptions {
            dry_run: true,
            keep_rollback_copy: true,
        },
    )
    .await
    .unwrap();
    assert!(dry.dry_run && dry.rows_total > 50);
    assert_eq!(encryption::probe_file(&db).unwrap(), FileState::Plaintext);
    assert!(!PathBuf::from(format!("{}.mw-new", db.display())).exists());

    // Encrypt.
    let rep = encryption::transform(&db, None, Some(&k1), &TransformOptions::default())
        .await
        .unwrap();
    assert_eq!(rep.operation, "encrypt");
    assert!(
        rep.tables
            .iter()
            .any(|(t, n)| t == "_sqlx_migrations" && *n > 50),
        "{:?}",
        rep.tables
    );
    assert!(rep.tables.iter().any(|(t, n)| t == "settings" && *n >= 50));
    assert_eq!(encryption::probe_file(&db).unwrap(), FileState::NotPlaintext);
    assert!(!any_contains(&db, MARKER));
    let rollback = PathBuf::from(rep.rollback_copy.clone().unwrap());
    assert_eq!(encryption::probe_file(&rollback).unwrap(), FileState::Plaintext);

    // The store opens with all rows, and the migrations table means no re-run.
    let s = open(&db, &cfg(EncryptionMode::Required, Some(k1.clone())))
        .await
        .unwrap();
    for i in 0..50 {
        assert_eq!(
            s.get_setting(&format!("k.{i}")).await.unwrap().as_deref(),
            Some(format!("{MARKER}-{i}").as_str())
        );
    }
    s.close().await;
    encryption::purge_file(&rollback).unwrap();

    // Rotate to k2: k1 stops working, k2 works.
    let rep = encryption::transform(&db, Some(&k1), Some(&k2), &TransformOptions::default())
        .await
        .unwrap();
    assert_eq!(rep.operation, "rekey");
    assert!(
        open(&db, &cfg(EncryptionMode::Required, Some(k1.clone())))
            .await
            .is_err()
    );
    let s = open(&db, &cfg(EncryptionMode::Required, Some(k2.clone())))
        .await
        .unwrap();
    assert_eq!(
        s.get_setting("k.49").await.unwrap().as_deref(),
        Some(format!("{MARKER}-49").as_str())
    );
    s.close().await;
    let meta = encryption::read_meta(&db).unwrap();
    assert_eq!(
        meta.key_fingerprint.as_deref(),
        Some(k2.fingerprint().as_str())
    );
    assert!(meta.last_rotated_at.is_some());

    // Decrypt back to plaintext.
    let rep = encryption::transform(
        &db,
        Some(&k2),
        None,
        &TransformOptions {
            dry_run: false,
            keep_rollback_copy: false,
        },
    )
    .await
    .unwrap();
    assert_eq!(rep.operation, "decrypt");
    assert_eq!(encryption::probe_file(&db).unwrap(), FileState::Plaintext);
    assert!(any_contains(&db, MARKER));
    let s = open(&db, &cfg(EncryptionMode::Off, None)).await.unwrap();
    assert_eq!(
        s.get_setting("k.0").await.unwrap().as_deref(),
        Some(format!("{MARKER}-0").as_str())
    );
    s.close().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn failed_transform_leaves_original_untouched_and_rollback_restores() {
    let db = seeded_plaintext("enc-rollback").await;
    let before = std::fs::read(&db).unwrap();
    // A source key on a plaintext file is refused before anything is written.
    let e = encryption::transform(&db, Some(&key(1)), Some(&key(2)), &TransformOptions::default())
        .await
        .err()
        .unwrap();
    assert!(matches!(e, EncryptionError::Migration(_)), "{e}");
    assert_eq!(std::fs::read(&db).unwrap(), before);

    let rep = encryption::transform(&db, None, Some(&key(5)), &TransformOptions::default())
        .await
        .unwrap();
    let rb = PathBuf::from(rep.rollback_copy.unwrap());
    encryption::rollback(&db, &rb).unwrap();
    assert_eq!(encryption::probe_file(&db).unwrap(), FileState::Plaintext);
    let s = open(&db, &cfg(EncryptionMode::Off, None)).await.unwrap();
    assert_eq!(
        s.get_setting("k.1").await.unwrap().as_deref(),
        Some(format!("{MARKER}-1").as_str())
    );
    s.close().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn encrypted_backup_and_restore_with_a_separate_key() {
    let db = test_db::unique_db_path("enc-backup");
    let live = key(40);
    let bk = key(41);
    let s = open(&db, &cfg(EncryptionMode::Required, Some(live.clone())))
        .await
        .unwrap();
    s.set_setting("backup.marker", MARKER).await.unwrap();
    s.close().await;

    let dst = db.with_file_name("backup.db");
    encryption::backup(&db, Some(&live), &dst, Some(&bk), false)
        .await
        .unwrap();
    assert!(!any_contains(&dst, MARKER));
    assert_eq!(encryption::probe_file(&dst).unwrap(), FileState::NotPlaintext);
    // A plaintext backup of an encrypted DB is refused unless explicitly allowed.
    let refused =
        encryption::backup(&db, Some(&live), &db.with_file_name("p.db"), None, false).await;
    assert!(refused.is_err());
    // The live key cannot open the backup; the backup key can.
    assert!(
        encryption::open_verified(&dst, Some(&live), false)
            .await
            .is_err()
    );

    let restored = db.with_file_name("restored.db");
    encryption::restore(&dst, Some(&bk), &restored, Some(&live))
        .await
        .unwrap();
    let s = open(&restored, &cfg(EncryptionMode::Required, Some(live)))
        .await
        .unwrap();
    assert_eq!(
        s.get_setting("backup.marker").await.unwrap().as_deref(),
        Some(MARKER)
    );
    s.close().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn verification_proves_the_file_is_encrypted_and_flags_plaintext() {
    let db = test_db::unique_db_path("enc-verify");
    let k = key(60);
    let s = open(&db, &cfg(EncryptionMode::Required, Some(k.clone())))
        .await
        .unwrap();
    s.close().await;
    let r = encryption::verify(&db, Some(&k)).await;
    assert!(r.passed, "{:#?}", r.checks);
    let names: Vec<_> = r.checks.iter().map(|c| c.name).collect();
    for n in [
        "header-not-sqlite",
        "wrong-key-rejected",
        "right-key-opens",
        "row-round-trip",
        "marker-not-in-file",
    ] {
        assert!(names.contains(&n), "missing check {n}");
    }

    let plain = seeded_plaintext("enc-verify-plain").await;
    let r = encryption::verify(&plain, Some(&k)).await;
    assert!(!r.passed);
    assert!(
        r.checks
            .iter()
            .any(|c| c.name == "header-not-sqlite" && !c.passed)
    );
}

#[test]
fn key_parsing() {
    // 64 hex characters are used as the key itself.
    let hex = "ab".repeat(32);
    assert_eq!(DbKey::from_secret(&hex).unwrap().to_hex().as_str(), hex);
    // Passphrases are stretched, deterministically.
    let a = DbKey::from_secret("correct horse battery staple").unwrap();
    let b = DbKey::from_secret("correct horse battery staple").unwrap();
    assert_eq!(a.fingerprint(), b.fingerprint());
    assert!(DbKey::from_secret("short").is_err());
    assert!(DbKey::from_secret("   ").is_err());
    assert_eq!(
        EncryptionMode::parse("Required").unwrap(),
        EncryptionMode::Required
    );
    assert!(EncryptionMode::parse("sometimes").is_err());
    // Debug output never contains the key.
    assert!(!format!("{a:?}").contains(a.to_hex().as_str()));
}

/// Opt-in: a key supplied only through the environment, as production resolves it.
#[tokio::test(flavor = "multi_thread")]
async fn env_key_toggle_is_honoured_when_set() {
    if std::env::var("MW_TEST_DB_KEY").is_err() {
        return;
    }
    let cfg = EncryptionConfig::from_env().unwrap();
    assert!(cfg.key.is_some(), "MW_TEST_DB_KEY set but no key resolved");
    let db = test_db::unique_db_path("enc-env");
    let s = Store::open_with_encryption(db.to_str().unwrap(), ServerKey::generate(), &cfg)
        .await
        .unwrap();
    s.close().await;
    assert_eq!(encryption::probe_file(&db).unwrap(), FileState::NotPlaintext);
}

#[tokio::test(flavor = "multi_thread")]
async fn refuses_to_convert_a_database_that_is_open() {
    let db = test_db::unique_db_path("enc-inuse");
    let s = open(&db, &cfg(EncryptionMode::Off, None)).await.unwrap();
    s.set_setting("a", "b").await.unwrap();
    let e = encryption::transform(&db, None, Some(&key(9)), &TransformOptions::default())
        .await
        .err()
        .unwrap();
    assert!(matches!(e, EncryptionError::Migration(_)), "{e}");
    assert_eq!(encryption::probe_file(&db).unwrap(), FileState::Plaintext);
    s.close().await;
}
