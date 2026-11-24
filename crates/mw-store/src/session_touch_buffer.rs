//! Coalesced session `last_seen` writes (t50hp plan A2 T3, A3 W-1).
//!
//! `sessions.last_seen` is request telemetry, not the idle clock: idle and
//! absolute expiry are decided from `activity_at` and `created_at`
//! ([`Store::touch_session_throttled`]). Without this buffer every request whose
//! `last_seen` is older than the throttle (`session.lastActivityFlushSeconds`)
//! writes the row before it answers. With it, such a request records the time
//! in process memory and continues; a background task writes the recorded
//! times in batches.
//!
//! Rules:
//! * Coalesced per session, keeping the latest time. A session is recorded at
//!   most once per throttle interval, as the UPDATE it replaces.
//! * Flushed every [`TouchBufferConfig::flush_every`] (default 5 s) by one
//!   statement per batch of at most [`FLUSH_BATCH`] sessions, a max-merge:
//!   `SET last_seen = v WHERE id = … AND last_seen < v` (byte order on both
//!   backends: SQLite's BINARY collation, `COLLATE "C"` on PostgreSQL). It never
//!   creates a row, so a session deleted in the meantime (sign-out, revoke,
//!   disable, expiry) stays deleted, and an older value never overwrites a newer
//!   one written by another process.
//! * Bounded: at most [`TouchBufferConfig::max_sessions`] sessions are held. At
//!   the cap a new session's touch is written through synchronously, as without
//!   the buffer.
//! * The buffered time is the read source in this process wherever a read
//!   depends on `last_seen`: the touch-due test
//!   ([`Store::session_touch_due`]) and the session lists
//!   ([`Store::buffered_last_seen`]).
//! * Flushed at shutdown ([`Store::flush_session_touches`], called by the
//!   server's drain).
//!
//! Loss on a crash: the touches recorded since the last flush, at most one
//! flush interval (plus the existing throttle). No session clock, authority
//! or other column is buffered.
//!
//! A flush that fails keeps its entries (merged with newer ones) for the next
//! flush. Buffers are per [`Store`] instance (shared by its clones) and live in
//! a process registry keyed by [`Store::instance_id`]; the Store struct is not
//! changed.
use std::collections::HashMap;
use std::sync::{
    Arc, Mutex, RwLock,
    atomic::{AtomicU64, AtomicUsize, Ordering},
};
use std::time::Duration;

use chrono::Utc;

use crate::backend::q;
use crate::{Store, StoreError};

/// Sessions per flush statement.
pub const FLUSH_BATCH: usize = 500;
const SHARDS: usize = 16;

/// How a store's touch buffer behaves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TouchBufferConfig {
    /// Interval of the background flush.
    pub flush_every: Duration,
    /// Most sessions held; a new session's touch is written through above it.
    pub max_sessions: usize,
    /// The touch throttle (`session.lastActivityFlushSeconds`). A flushed entry
    /// is kept as a not-due hint until it is this old.
    pub throttle_secs: i64,
}
impl Default for TouchBufferConfig {
    fn default() -> Self {
        Self {
            flush_every: Duration::from_secs(5),
            max_sessions: 65_536,
            throttle_secs: 15,
        }
    }
}

/// Counters of one store's touch buffer (process lifetime).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TouchBufferStats {
    /// Touches recorded in memory instead of an UPDATE.
    pub recorded: u64,
    /// Touches found not due against the buffered time (no write at all).
    pub not_due: u64,
    /// Touches written through because the buffer was at its cap.
    pub write_through: u64,
    /// Flush statements run, rows they changed, and failed statements.
    pub flush_statements: u64,
    pub flushed_rows: u64,
    pub flush_failures: u64,
    /// Sessions waiting for a flush, and sessions held (pending + hints).
    pub pending: u64,
    pub held: u64,
}

struct Entry {
    at: String,
    dirty: bool,
}

pub(crate) struct TouchBuffer {
    config: TouchBufferConfig,
    shards: [Mutex<HashMap<String, Entry>>; SHARDS],
    held: AtomicUsize,
    flushing: tokio::sync::Mutex<()>,
    recorded: AtomicU64,
    not_due: AtomicU64,
    write_through: AtomicU64,
    flush_statements: AtomicU64,
    flushed_rows: AtomicU64,
    flush_failures: AtomicU64,
}

enum Recorded {
    NotDue,
    Buffered,
    Full,
}

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

impl TouchBuffer {
    fn new(config: TouchBufferConfig) -> Self {
        Self {
            config,
            shards: std::array::from_fn(|_| Mutex::new(HashMap::new())),
            held: AtomicUsize::new(0),
            flushing: tokio::sync::Mutex::new(()),
            recorded: AtomicU64::new(0),
            not_due: AtomicU64::new(0),
            write_through: AtomicU64::new(0),
            flush_statements: AtomicU64::new(0),
            flushed_rows: AtomicU64::new(0),
            flush_failures: AtomicU64::new(0),
        }
    }

    fn shard(&self, id: &str) -> &Mutex<HashMap<String, Entry>> {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        id.hash(&mut hasher);
        &self.shards[(hasher.finish() as usize) % SHARDS]
    }

    /// Record `now` for `id` unless the held time is at or after `cutoff`.
    fn record(&self, id: &str, now: &str, cutoff: &str) -> Recorded {
        let mut shard = lock(self.shard(id));
        if let Some(entry) = shard.get_mut(id) {
            if entry.at.as_str() >= cutoff {
                self.not_due.fetch_add(1, Ordering::Relaxed);
                return Recorded::NotDue;
            }
            if entry.at.as_str() < now {
                entry.at = now.to_owned();
            }
            entry.dirty = true;
            self.recorded.fetch_add(1, Ordering::Relaxed);
            return Recorded::Buffered;
        }
        if self.held.load(Ordering::Relaxed) >= self.config.max_sessions {
            // Hints are only an optimisation: drop every flushed entry before
            // refusing a new session (one shard lock at a time).
            drop(shard);
            for other in &self.shards {
                let mut other = lock(other);
                let before = other.len();
                other.retain(|_, entry| entry.dirty);
                self.held.fetch_sub(before - other.len(), Ordering::Relaxed);
            }
            shard = lock(self.shard(id));
            if shard.contains_key(id) {
                drop(shard);
                return self.record(id, now, cutoff);
            }
            if self.held.load(Ordering::Relaxed) >= self.config.max_sessions {
                self.write_through.fetch_add(1, Ordering::Relaxed);
                return Recorded::Full;
            }
        }
        shard.insert(
            id.to_owned(),
            Entry {
                at: now.to_owned(),
                dirty: true,
            },
        );
        self.held.fetch_add(1, Ordering::Relaxed);
        self.recorded.fetch_add(1, Ordering::Relaxed);
        Recorded::Buffered
    }

    fn latest(&self, id: &str) -> Option<String> {
        lock(self.shard(id)).get(id).map(|entry| entry.at.clone())
    }

    /// Take every pending time (entries stay as clean hints) and drop hints
    /// older than `stale_before`.
    fn take_pending(&self, stale_before: &str) -> Vec<(String, String)> {
        let mut pending = Vec::new();
        for shard in &self.shards {
            let mut shard = lock(shard);
            let before = shard.len();
            shard.retain(|id, entry| {
                if entry.dirty {
                    entry.dirty = false;
                    pending.push((id.clone(), entry.at.clone()));
                    true
                } else {
                    entry.at.as_str() >= stale_before
                }
            });
            self.held.fetch_sub(before - shard.len(), Ordering::Relaxed);
        }
        pending
    }

    /// A failed batch is pending again (merged with anything newer).
    fn restore(&self, failed: &[(String, String)]) {
        for (id, at) in failed {
            let mut shard = lock(self.shard(id));
            match shard.get_mut(id) {
                Some(entry) => {
                    if entry.at < *at {
                        entry.at = at.clone();
                    }
                    entry.dirty = true;
                }
                None => {
                    shard.insert(
                        id.clone(),
                        Entry {
                            at: at.clone(),
                            dirty: true,
                        },
                    );
                    self.held.fetch_add(1, Ordering::Relaxed);
                }
            }
        }
    }

    fn stats(&self) -> TouchBufferStats {
        let pending = self
            .shards
            .iter()
            .map(|shard| lock(shard).values().filter(|entry| entry.dirty).count() as u64)
            .sum();
        TouchBufferStats {
            recorded: self.recorded.load(Ordering::Relaxed),
            not_due: self.not_due.load(Ordering::Relaxed),
            write_through: self.write_through.load(Ordering::Relaxed),
            flush_statements: self.flush_statements.load(Ordering::Relaxed),
            flushed_rows: self.flushed_rows.load(Ordering::Relaxed),
            flush_failures: self.flush_failures.load(Ordering::Relaxed),
            pending,
            held: self.held.load(Ordering::Relaxed) as u64,
        }
    }
}

/// The later of a row's `last_seen` and a buffered time (byte order, as the
/// flush compares them).
pub fn later(row: String, buffered: Option<String>) -> String {
    match buffered {
        Some(held) if held > row => held,
        _ => row,
    }
}

fn registry() -> &'static RwLock<HashMap<uuid::Uuid, Arc<TouchBuffer>>> {
    static BUFFERS: std::sync::OnceLock<RwLock<HashMap<uuid::Uuid, Arc<TouchBuffer>>>> =
        std::sync::OnceLock::new();
    BUFFERS.get_or_init(|| RwLock::new(HashMap::new()))
}

impl Store {
    fn touch_buffer(&self) -> Option<Arc<TouchBuffer>> {
        registry()
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(&self.instance_id())
            .cloned()
    }

    /// Buffer this store's session touches (module docs). Idempotent: a second
    /// call keeps the first configuration. Starts the background flush on the
    /// current Tokio runtime; returns false (and buffers nothing) outside one,
    /// or when the configuration is unusable.
    pub fn enable_session_touch_buffer(&self, config: TouchBufferConfig) -> bool {
        if config.max_sessions == 0 || config.throttle_secs <= 0 || config.flush_every.is_zero() {
            return false;
        }
        let Ok(runtime) = tokio::runtime::Handle::try_current() else {
            return false;
        };
        let buffer = {
            let mut buffers = registry()
                .write()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if buffers.contains_key(&self.instance_id()) {
                return true;
            }
            let buffer = Arc::new(TouchBuffer::new(config));
            buffers.insert(self.instance_id(), buffer.clone());
            buffer
        };
        let store = self.clone();
        let weak = Arc::downgrade(&buffer);
        drop(buffer);
        runtime.spawn(async move {
            let mut tick = tokio::time::interval(config.flush_every);
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            tick.tick().await;
            loop {
                tick.tick().await;
                // Disabled (tests) or replaced: this task ends.
                let Some(buffer) = weak.upgrade() else { break };
                if !store
                    .touch_buffer()
                    .is_some_and(|current| Arc::ptr_eq(&current, &buffer))
                {
                    break;
                }
                drop(buffer);
                // A failure is counted and its entries stay pending.
                let _ = store.flush_session_touches().await;
            }
        });
        true
    }

    /// Test and maintenance hook: stop buffering for this store, flushing
    /// what is pending first.
    pub async fn disable_session_touch_buffer(&self) -> Result<u64, StoreError> {
        let flushed = self.flush_session_touches().await?;
        registry()
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .remove(&self.instance_id());
        Ok(flushed)
    }

    /// Whether this store's session touches go through the buffer.
    pub fn session_touch_buffered(&self) -> bool {
        self.touch_buffer().is_some()
    }

    /// The latest touch this process recorded for `id` and has not yet seen
    /// age past the throttle, flushed or not. Callers merge it with the row's
    /// `last_seen` by taking the later of the two.
    pub fn buffered_last_seen(&self, id: &str) -> Option<String> {
        self.touch_buffer()?.latest(id)
    }

    /// [`Store::session_touch_may_be_due_at`] on the later of the row's
    /// `last_seen` (read earlier in the request) and this process's buffered
    /// time for the session.
    pub fn session_touch_due(
        &self,
        id: &str,
        last_seen: Option<&str>,
        min_age_secs: i64,
        now: chrono::DateTime<Utc>,
    ) -> bool {
        let buffered = self.buffered_last_seen(id);
        let latest = match (last_seen, buffered.as_deref()) {
            (Some(row), Some(held)) => Some(if held > row { held } else { row }),
            (row, held) => row.or(held),
        };
        self.session_touch_may_be_due_at(latest, min_age_secs, now)
    }

    /// Record a due touch in the buffer. `Ok(true)`: handled in memory (or
    /// not due against the buffered time); `Ok(false)`: no buffer, or the
    /// buffer is at its cap and the caller writes through.
    pub(crate) fn buffer_session_touch(&self, id: &str, now: &str, cutoff: &str) -> bool {
        match self
            .touch_buffer()
            .map(|buffer| buffer.record(id, now, cutoff))
        {
            Some(Recorded::NotDue | Recorded::Buffered) => true,
            Some(Recorded::Full) | None => false,
        }
    }

    /// Write every pending touch now (one max-merge statement per batch).
    /// Returns the rows changed. A failed batch stays pending; the first error
    /// is returned after the remaining batches were attempted.
    pub async fn flush_session_touches(&self) -> Result<u64, StoreError> {
        let Some(buffer) = self.touch_buffer() else {
            return Ok(0);
        };
        let _one_flush = buffer.flushing.lock().await;
        let stale_before =
            (Utc::now() - chrono::Duration::seconds(buffer.config.throttle_secs)).to_rfc3339();
        let pending = buffer.take_pending(&stale_before);
        let sql = match self.backend.dialect() {
            crate::backend::Dialect::Sqlite => {
                "UPDATE sessions SET last_seen = v.t FROM (SELECT json_extract(value, '$[0]') AS id, json_extract(value, '$[1]') AS t FROM json_each(?1)) AS v WHERE sessions.id = v.id AND sessions.last_seen < v.t"
            }
            crate::backend::Dialect::Postgres => {
                "UPDATE sessions SET last_seen = v.t FROM (SELECT e->>0 AS id, e->>1 AS t FROM json_array_elements(CAST(?1 AS json)) AS e) AS v WHERE sessions.id = v.id AND sessions.last_seen < v.t COLLATE \"C\""
            }
        };
        let mut changed = 0;
        let mut first_error = None;
        for batch in pending.chunks(FLUSH_BATCH) {
            let rows = serde_json::to_string(
                &batch
                    .iter()
                    .map(|(id, at)| [id.as_str(), at.as_str()])
                    .collect::<Vec<_>>(),
            )
            .map_err(|_| StoreError::Invalid("session touch batch"))?;
            buffer.flush_statements.fetch_add(1, Ordering::Relaxed);
            match q(sql).bind(rows).execute(&self.backend).await {
                Ok(n) => {
                    changed += n;
                    buffer.flushed_rows.fetch_add(n, Ordering::Relaxed);
                }
                Err(error) => {
                    buffer.flush_failures.fetch_add(1, Ordering::Relaxed);
                    buffer.restore(batch);
                    first_error.get_or_insert(StoreError::from(error));
                }
            }
        }
        match first_error {
            Some(error) => Err(error),
            None => Ok(changed),
        }
    }

    /// Counters of this store's touch buffer, `None` when it is not buffered.
    pub fn session_touch_buffer_stats(&self) -> Option<TouchBufferStats> {
        self.touch_buffer().map(|buffer| buffer.stats())
    }
}

#[cfg(test)]
#[path = "session_touch_buffer_tests.rs"]
mod tests;
