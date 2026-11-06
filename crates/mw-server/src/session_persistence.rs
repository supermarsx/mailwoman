//! What a server restart does to signed-in people, decided once at start-up.
//!
//! Mailbox sessions, native bearer sessions and admin console sessions are rows in
//! the database with their creation time, last activity and absolute deadline, and
//! the upstream credentials in a mailbox session are sealed with the server key. A
//! fresh process therefore needs nothing from the old one to keep a session valid;
//! `docs/admin/sessions.md` lists the little that is deliberately in memory only.
//!
//! The settings read here apply at start-up:
//! * `session.persistAcrossRestarts` (default on): off ends every session when the
//!   process starts.
//! * `session.lastActivityFlushSeconds`: how often a request writes `last_seen`.
//! * `server.shutdownGraceSeconds`: how long a stopping server waits for requests.

use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU64, Ordering};
use std::time::Duration;

use mw_store::Store;
use mw_store::settings_policy::SettingSubject;

static FLUSH_SECS: AtomicI64 = AtomicI64::new(15);
static GRACE_SECS: AtomicU64 = AtomicU64::new(0);
static PERSISTING: AtomicBool = AtomicBool::new(true);
static PURGED_AT_START: AtomicU64 = AtomicU64::new(0);

async fn global(store: &Store, key: &str) -> Option<serde_json::Value> {
    crate::settings_registry::effective_for(store, key, &SettingSubject::default())
        .await
        .ok()
        .flatten()
        .map(|effective| effective.value)
}

/// Read the start-up settings and apply `session.persistAcrossRestarts`.
pub(crate) async fn apply_at_startup(store: &Store) {
    if let Some(n) = global(store, "session.lastActivityFlushSeconds")
        .await
        .and_then(|v| v.as_i64())
    {
        FLUSH_SECS.store(n.clamp(0, 300), Ordering::Relaxed);
    }
    if let Some(n) = global(store, "server.shutdownGraceSeconds")
        .await
        .and_then(|v| v.as_u64())
    {
        GRACE_SECS.store(n.clamp(1, 120), Ordering::Relaxed);
    }
    let persist = global(store, "session.persistAcrossRestarts")
        .await
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    PERSISTING.store(persist, Ordering::Relaxed);
    if !persist {
        match store.purge_all_sessions().await {
            Ok(n) => {
                PURGED_AT_START.store(n, Ordering::Relaxed);
                tracing::info!(
                    "session.persistAcrossRestarts is off: ended {n} persisted session(s) at start-up"
                );
            }
            Err(error) => tracing::warn!("could not end sessions at start-up: {error}"),
        }
    }
}

/// Minimum seconds between two `last_seen` writes for one session.
pub(crate) fn flush_seconds() -> i64 {
    FLUSH_SECS.load(Ordering::Relaxed)
}

/// The operator's shutdown grace, when the setting was read.
pub fn shutdown_grace() -> Option<Duration> {
    match GRACE_SECS.load(Ordering::Relaxed) {
        0 => None,
        n => Some(Duration::from_secs(n)),
    }
}

/// Resolves on SIGTERM (Unix) or Ctrl+C. systemd, Docker and Kubernetes stop a
/// service with SIGTERM, so handling only Ctrl+C would skip the drain.
pub async fn termination_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        if let Ok(mut term) = signal(SignalKind::terminate()) {
            tokio::select! {
                _ = term.recv() => {}
                _ = tokio::signal::ctrl_c() => {}
            }
            return;
        }
    }
    let _ = tokio::signal::ctrl_c().await;
}

/// The "Sessions survive restarts" posture for the admin status line.
pub(crate) async fn posture(store: &Store, key_persistent: bool) -> serde_json::Value {
    let persisting = PERSISTING.load(Ordering::Relaxed);
    let (checked, unopenable) = store.session_key_check(50).await.unwrap_or((0, 0));
    let (survive, reason) = if unopenable > 0 {
        (false, "server-key-changed")
    } else if !persisting {
        (false, "setting-off")
    } else if !key_persistent {
        (false, "server-key-ephemeral")
    } else {
        (true, "persisted")
    };
    let counts = store.session_counts().await.unwrap_or_default();
    serde_json::json!({
        "survive": survive,
        "reason": reason,
        "persistAcrossRestarts": persisting,
        "serverKeyPersistent": key_persistent,
        "mailbox": counts.mailbox,
        "native": counts.native,
        "admin": counts.admin,
        "sessionsChecked": checked,
        "sessionsUnopenable": unopenable,
        "endedAtStart": PURGED_AT_START.load(Ordering::Relaxed),
        "lastActivityFlushSeconds": flush_seconds(),
    })
}
