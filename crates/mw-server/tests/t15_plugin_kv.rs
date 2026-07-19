//! t15-E-e2e — LEG 5: persistent plugin KV storage across a HOST RESTART, over REAL
//! infrastructure.
//!
//! 26.15 replaces the former non-persistent `HostKv` stub with a sealed, quota-bounded,
//! per-(plugin, account)-isolated store over the 0013 `plugin_kv` table. "unit-green !=
//! wired": the unit tests in `mw-store/src/plugin_kv.rs` prove the store methods on a
//! SINGLE in-memory SQLite handle. THIS leg proves the value is genuinely PERSISTED and
//! sealed — not held in RAM — by writing it through one `Store`, DROPPING that store
//! (the "restart"), opening a FRESH `Store` over the SAME database with the SAME
//! `ServerKey`, and reading the value back. It runs over a real on-disk SQLite file
//! (always) AND a real Postgres (via `MW_E14_PG_DSN`), so the 0013 migration + sealed
//! round-trip are exercised in both dialects across a process-equivalent restart.
//!
//! Also proven live: per-(plugin, account) isolation (a second plugin / a second account
//! cannot read the value), a visible quota rejection, and whole-namespace purge on
//! uninstall.
//!
//! ## Running
//!   cargo test -p mw-server --test t15_plugin_kv                       # on-disk SQLite
//!   docker compose -f docker-compose.ci.yml up -d --wait postgres
//!   MW_E14_PG_DSN=postgres://mailwoman:mailwoman@localhost:5432/mailwoman \
//!     cargo test -p mw-server --test t15_plugin_kv -- --nocapture --test-threads=1

use std::path::PathBuf;

use mw_store::{PluginKvError, PluginKvLimits, ServerKey, Store};

fn unique() -> String {
    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    format!(
        "{}_{}",
        std::process::id(),
        SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    )
}

fn temp_db_path(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("mw-t15-kv-{tag}-{}-{nanos}.db", unique()))
}

/// How the two `Store` instances (the "before" and "after" the restart) are opened over
/// the SAME backing store. The `ServerKey` is carried by hex so the second host can
/// UNSEAL what the first sealed — proving the value survived at rest, sealed.
enum Backing {
    /// A real on-disk SQLite file (persists across `Store` instances).
    SqliteFile(PathBuf),
    /// A live Postgres DSN.
    Postgres(String),
}

impl Backing {
    async fn open(&self, key: ServerKey) -> Store {
        match self {
            Backing::SqliteFile(p) => Store::open(&p.to_string_lossy(), key)
                .await
                .expect("open on-disk sqlite store"),
            Backing::Postgres(dsn) => Store::open(dsn, key)
                .await
                .expect("open live postgres store"),
        }
    }
}

/// The full persistence-across-restart + isolation + quota + purge proof against a real
/// persistent backing store.
async fn drive(backing: Backing, dialect: &str) {
    // A stable server key, carried by hex across the "restart".
    let key_hex = ServerKey::generate().to_hex();
    // Unique ids so a shared Postgres never collides across tests/legs.
    let plugin_a = format!("plugin-a-{}", unique());
    let plugin_b = format!("plugin-b-{}", unique());
    let account = format!("acct-{}", unique());
    let other_account = format!("acct-other-{}", unique());
    let secret = b"persist-me-across-a-restart-sealed-value";

    // ── Host #1: a granted plugin persists a value, then the host goes away. ──────────
    {
        let store = backing.open(ServerKey::from_hex(&key_hex).unwrap()).await;
        store
            .plugin_kv_set(
                &plugin_a,
                &account,
                "greeting",
                secret,
                &PluginKvLimits::default(),
            )
            .await
            .expect("set persists");
        // The value is readable within the same host.
        assert_eq!(
            store
                .plugin_kv_get(&plugin_a, &account, "greeting")
                .await
                .unwrap()
                .as_deref(),
            Some(&secret[..]),
            "[{dialect}] value reads back within host #1"
        );
        // store drops here → the "restart".
    }

    // ── Host #2: a FRESH store over the SAME backing + SAME key reads it back. ────────
    let store2 = backing.open(ServerKey::from_hex(&key_hex).unwrap()).await;
    assert_eq!(
        store2
            .plugin_kv_get(&plugin_a, &account, "greeting")
            .await
            .unwrap()
            .as_deref(),
        Some(&secret[..]),
        "[{dialect}] the value SURVIVED the restart, sealed + persisted (not in-memory)"
    );

    // Isolation: another plugin, or the same plugin under another account, cannot read it.
    assert!(
        store2
            .plugin_kv_get(&plugin_b, &account, "greeting")
            .await
            .unwrap()
            .is_none(),
        "[{dialect}] a different plugin cannot read the key"
    );
    assert!(
        store2
            .plugin_kv_get(&plugin_a, &other_account, "greeting")
            .await
            .unwrap()
            .is_none(),
        "[{dialect}] the same plugin under another account cannot read the key"
    );

    // Quota is enforced VISIBLY (never a silent drop) on the real store.
    let tiny = PluginKvLimits {
        max_key_bytes: 256,
        max_value_bytes: 16,
        max_total_bytes: 32,
        max_keys: 1000,
    };
    let over_value = store2
        .plugin_kv_set(&plugin_a, &account, "big", &[0u8; 17], &tiny)
        .await;
    assert!(
        matches!(over_value, Err(PluginKvError::ValueTooLarge { .. })),
        "[{dialect}] an oversize value is rejected visibly: {over_value:?}"
    );
    assert!(
        store2
            .plugin_kv_get(&plugin_a, &account, "big")
            .await
            .unwrap()
            .is_none(),
        "[{dialect}] the rejected over-quota put wrote nothing"
    );

    // Uninstall purge: the whole namespace is reclaimed; a second plugin's data survives.
    store2
        .plugin_kv_set(
            &plugin_b,
            &account,
            "keep",
            b"survives",
            &PluginKvLimits::default(),
        )
        .await
        .unwrap();
    let purged = store2.plugin_kv_purge(&plugin_a).await.unwrap();
    assert!(
        purged >= 1,
        "[{dialect}] uninstall purges plugin-a's namespace ({purged} rows)"
    );
    assert!(
        store2
            .plugin_kv_get(&plugin_a, &account, "greeting")
            .await
            .unwrap()
            .is_none(),
        "[{dialect}] plugin-a's keys are gone after purge"
    );
    assert_eq!(
        store2
            .plugin_kv_get(&plugin_b, &account, "keep")
            .await
            .unwrap()
            .as_deref(),
        Some(&b"survives"[..]),
        "[{dialect}] a different plugin's KV survives the purge"
    );

    // Clean up the SQLite temp file (+ WAL/SHM sidecars); Postgres rows are ephemeral.
    if let Backing::SqliteFile(p) = &backing {
        drop(store2);
        for suffix in ["", "-wal", "-shm"] {
            let _ = std::fs::remove_file(format!("{}{suffix}", p.to_string_lossy()));
        }
    }
}

// ── On-disk SQLite (always) ──────────────────────────────────────────────────────────

#[tokio::test]
async fn plugin_kv_persists_across_restart_sqlite() {
    drive(Backing::SqliteFile(temp_db_path("restart")), "sqlite").await;
}

// ── Postgres (live via MW_E14_PG_DSN, else loud-skip) ────────────────────────────────

#[tokio::test]
async fn plugin_kv_persists_across_restart_postgres() {
    let Ok(dsn) = std::env::var("MW_E14_PG_DSN") else {
        eprintln!(
            "\n[t15 plugin-kv SKIP] MW_E14_PG_DSN unset — live Postgres KV persistence not driven.\n"
        );
        return;
    };
    drive(Backing::Postgres(dsn), "postgres").await;
}
