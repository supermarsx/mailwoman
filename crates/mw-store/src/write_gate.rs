//! In-process write gate for a managed SQLite database (t50hp stage 0).
//!
//! SQLite allows one writer at a time. Every store transaction starts with
//! `BEGIN IMMEDIATE`, and every autocommit write takes the same lock. Without
//! this gate, a second in-process writer gets a pooled connection first and then
//! sleeps in SQLite's busy handler (1, 2, 5, 10, ... 100 ms steps, up to
//! `busy_timeout`) on that connection's worker thread until the lock is free.
//!
//! With the gate, a writer first takes the single permit of a FIFO
//! [`tokio::sync::Semaphore`]; only then does it acquire a pool connection and
//! run `BEGIN IMMEDIATE` (or the autocommit statement). Waiting writers wait in
//! tokio, in arrival order, and hold no pool connection while they wait. The
//! permit is owned by the transaction (or by the statement's guarded
//! connection), so commit, rollback, an error, a cancelled future or a dropped
//! transaction all release it.
//!
//! What does not change: SQL text, transaction boundaries, `BEGIN IMMEDIATE`,
//! `synchronous`, and `busy_timeout`. Other processes (the CLI, a second server
//! on the same file) and the store's private maintenance connections do not
//! take this gate; they still meet in-process writers through SQLite's lock and
//! `busy_timeout`, as before.
//!
//! Bounded waiting: a writer that has not received the permit after
//! [`WAIT_LIMIT`] (the foreground pool's `busy_timeout`) gives up with the error a
//! locked database returns today: a database error with SQLite's primary code
//! `5` and the message `database is locked`.
//!
//! Which statements take the gate: those whose first word is `INSERT`, `UPDATE`,
//! `DELETE` or `REPLACE` (the write class of `pool_telemetry`), and every
//! `begin()`. Statements of other shapes (`WITH ... INSERT`, DDL, `VACUUM`,
//! `PRAGMA`) are not gated and keep waiting in the busy handler, as before;
//! that costs only the wait, never correctness, because `busy_timeout` stays.
//!
//! Switch: `MW_SQLITE_WRITER` = `gate` (default; also when empty) or `off`, read once per process
//! when the first managed SQLite database opens. PostgreSQL and unmanaged
//! (in-memory, private, test) SQLite pools never use the gate.
//!
//! API kept small for the group-commit writer that builds on it:
//! [`WriteGate::acquire`] returns a [`WritePermit`] (RAII; drop releases), and
//! [`WriteGate::stats`] reports the counters.
use std::{
    fmt,
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

/// Longest wait for the permit. Equal to the `busy_timeout` of the foreground
/// SQLite pool (`Store::open_with_encryption`), so an in-process writer gives up
/// after the same time a locked database makes it wait today.
pub(crate) const WAIT_LIMIT: Duration = Duration::from_secs(5);
/// A permit held longer than this is counted and reported (rate-limited).
pub(crate) const LONG_HOLD: Duration = Duration::from_secs(1);
/// At most one long-hold or same-task warning line per this interval and gate.
const WARN_EVERY: Duration = Duration::from_secs(60);

/// Effective `MW_SQLITE_WRITER` mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// In-process FIFO write gate in front of the SQLite write lock.
    Gate,
    /// No gate: in-process writers wait in SQLite's busy handler.
    Off,
}
impl Mode {
    /// The switch value.
    pub fn as_str(self) -> &'static str {
        match self {
            Mode::Gate => "gate",
            Mode::Off => "off",
        }
    }
}

/// The text reported for an invalid `MW_SQLITE_WRITER` value.
pub const INVALID_MODE: &str = "MW_SQLITE_WRITER must be gate or off";

/// Parse a `MW_SQLITE_WRITER` value. Unset or empty means the default (`gate`),
/// as for the t34 switches; any other value is refused.
pub fn parse_mode(raw: Option<&str>) -> Result<Mode, String> {
    match raw.map(str::trim) {
        None | Some("") => Ok(Mode::Gate),
        Some(value) if value.eq_ignore_ascii_case("gate") => Ok(Mode::Gate),
        Some(value) if value.eq_ignore_ascii_case("off") => Ok(Mode::Off),
        Some(_) => Err(INVALID_MODE.into()),
    }
}

/// The process mode, read from the environment once and then frozen.
pub fn configured_mode() -> Result<Mode, String> {
    static MODE: OnceLock<Result<Mode, String>> = OnceLock::new();
    MODE.get_or_init(|| parse_mode(std::env::var("MW_SQLITE_WRITER").ok().as_deref()))
        .clone()
}

/// Whether a statement takes the gate: its first word is a write verb, the
/// same classification as `pool_telemetry::QueryTimer`.
pub(crate) fn is_gated_statement(sql: &str) -> bool {
    let verb = sql.split_ascii_whitespace().next().unwrap_or("");
    ["INSERT", "UPDATE", "DELETE", "REPLACE"]
        .iter()
        .any(|candidate| verb.eq_ignore_ascii_case(candidate))
}

/// What a permit is for (diagnostics only).
#[derive(Clone, Copy, Debug)]
pub(crate) enum Site {
    /// A transaction; the location of the `begin()` call.
    Transaction(&'static std::panic::Location<'static>),
    /// One autocommit write statement.
    Statement,
}
impl fmt::Display for Site {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Site::Transaction(at) => write!(f, "transaction begun at {}:{}", at.file(), at.line()),
            Site::Statement => f.write_str("autocommit write statement"),
        }
    }
}

/// Counters of one gate. Times in microseconds.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteGateStats {
    /// Permits granted.
    pub granted: u64,
    /// Permits granted after waiting behind another holder.
    pub waited: u64,
    /// Writers waiting now.
    pub queued: u64,
    /// Most writers waiting at once.
    pub queued_max: u64,
    pub wait_micros_total: u64,
    pub wait_micros_max: u64,
    pub hold_micros_total: u64,
    pub hold_micros_max: u64,
    /// Waiters that gave up after [`WAIT_LIMIT`] (answered "database is locked").
    pub timeouts: u64,
    /// Waiters whose future was dropped while queued.
    pub cancelled: u64,
    /// Permits held longer than [`LONG_HOLD`].
    pub long_holds: u64,
    /// Waits that started on the tokio task that holds the permit. Such a wait
    /// ends only when the holder finishes in another branch of the same task
    /// (`join!`), or after [`WAIT_LIMIT`]; without the gate the same code waits
    /// in the busy handler for the same time.
    pub same_task_waits: u64,
}

struct Counters {
    granted: AtomicU64,
    waited: AtomicU64,
    queued: AtomicU64,
    queued_max: AtomicU64,
    wait_micros_total: AtomicU64,
    wait_micros_max: AtomicU64,
    hold_micros_total: AtomicU64,
    hold_micros_max: AtomicU64,
    timeouts: AtomicU64,
    cancelled: AtomicU64,
    long_holds: AtomicU64,
    same_task_waits: AtomicU64,
}
impl Counters {
    const fn new() -> Self {
        Self {
            granted: AtomicU64::new(0),
            waited: AtomicU64::new(0),
            queued: AtomicU64::new(0),
            queued_max: AtomicU64::new(0),
            wait_micros_total: AtomicU64::new(0),
            wait_micros_max: AtomicU64::new(0),
            hold_micros_total: AtomicU64::new(0),
            hold_micros_max: AtomicU64::new(0),
            timeouts: AtomicU64::new(0),
            cancelled: AtomicU64::new(0),
            long_holds: AtomicU64::new(0),
            same_task_waits: AtomicU64::new(0),
        }
    }
    fn snapshot(&self) -> WriteGateStats {
        let load = |value: &AtomicU64| value.load(Ordering::Relaxed);
        WriteGateStats {
            granted: load(&self.granted),
            waited: load(&self.waited),
            queued: load(&self.queued),
            queued_max: load(&self.queued_max),
            wait_micros_total: load(&self.wait_micros_total),
            wait_micros_max: load(&self.wait_micros_max),
            hold_micros_total: load(&self.hold_micros_total),
            hold_micros_max: load(&self.hold_micros_max),
            timeouts: load(&self.timeouts),
            cancelled: load(&self.cancelled),
            long_holds: load(&self.long_holds),
            same_task_waits: load(&self.same_task_waits),
        }
    }
}

/// Sum over every gate of the process (`queued_max` and the maxima are the
/// largest seen by any one gate). Reported in `pool_telemetry::process_snapshot`.
static PROCESS: Counters = Counters::new();
static PROCESS_GATES: AtomicU64 = AtomicU64::new(0);

/// Process-wide gate counters, `None` while no gate exists in this process.
pub(crate) fn process_stats() -> Option<WriteGateStats> {
    (PROCESS_GATES.load(Ordering::Relaxed) > 0).then(|| PROCESS.snapshot())
}

/// A fair FIFO single-writer gate. One per managed SQLite database (shared by
/// every clone of its `LiveSqlite`, across pool generations).
pub(crate) struct WriteGate {
    semaphore: Arc<Semaphore>,
    wait_limit: Duration,
    counters: Counters,
    holder: Mutex<Option<tokio::task::Id>>,
    started: Instant,
    /// Seconds since `started` of the last warning line, plus one (0 = never).
    last_warning: AtomicU64,
    #[cfg(test)]
    pub(crate) allow_same_task_wait: std::sync::atomic::AtomicBool,
}
impl fmt::Debug for WriteGate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WriteGate").field("stats", &self.stats()).finish()
    }
}

fn micros(duration: Duration) -> u64 {
    duration.as_micros().min(u128::from(u64::MAX)) as u64
}

impl WriteGate {
    pub(crate) fn new() -> Arc<Self> {
        Self::with_wait_limit(WAIT_LIMIT)
    }
    pub(crate) fn with_wait_limit(wait_limit: Duration) -> Arc<Self> {
        PROCESS_GATES.fetch_add(1, Ordering::Relaxed);
        Arc::new(Self {
            semaphore: Arc::new(Semaphore::new(1)),
            wait_limit,
            counters: Counters::new(),
            holder: Mutex::new(None),
            started: Instant::now(),
            last_warning: AtomicU64::new(0),
            #[cfg(test)]
            allow_same_task_wait: std::sync::atomic::AtomicBool::new(false),
        })
    }

    /// Wait for the write permit, in arrival order, for at most the wait limit.
    /// Cancel-safe: dropping the future leaves the queue and never leaks the
    /// permit.
    pub(crate) async fn acquire(self: &Arc<Self>, site: Site) -> Result<WritePermit, sqlx::Error> {
        if let Ok(permit) = self.semaphore.clone().try_acquire_owned() {
            return Ok(self.grant(permit, site, None));
        }
        let task = tokio::task::try_id();
        if task.is_some() && task == *self.holder.lock().unwrap_or_else(|p| p.into_inner()) {
            self.each(|c| c.same_task_waits.fetch_add(1, Ordering::Relaxed));
            #[cfg(test)]
            if !self.allow_same_task_wait.load(Ordering::Relaxed)
                && std::env::var("MW_TEST_WRITE_GATE_STRICT").as_deref() == Ok("1")
            {
                panic!("write gate: {site} waits for a permit held by its own task");
            }
            self.warn(format_args!(
                "a write ({site}) waits for the SQLite write permit held by its own task; it proceeds only if the holder finishes in another branch of the task, otherwise it fails after {} ms",
                self.wait_limit.as_millis()
            ));
        }
        let waiting = Waiting::enter(self);
        let started = Instant::now();
        match tokio::time::timeout(self.wait_limit, self.semaphore.clone().acquire_owned()).await {
            Ok(Ok(permit)) => {
                waiting.leave(false);
                Ok(self.grant(permit, site, Some(started.elapsed())))
            }
            // The semaphore is never closed.
            Ok(Err(_)) => {
                waiting.leave(false);
                Err(sqlx::Error::PoolClosed)
            }
            Err(_) => {
                waiting.leave(false);
                self.each(|c| c.timeouts.fetch_add(1, Ordering::Relaxed));
                Err(locked())
            }
        }
    }

    fn grant(self: &Arc<Self>, permit: OwnedSemaphorePermit, site: Site, waited: Option<Duration>) -> WritePermit {
        self.each(|c| {
            c.granted.fetch_add(1, Ordering::Relaxed);
            if let Some(waited) = waited {
                let waited = micros(waited);
                c.waited.fetch_add(1, Ordering::Relaxed);
                c.wait_micros_total.fetch_add(waited, Ordering::Relaxed);
                c.wait_micros_max.fetch_max(waited, Ordering::Relaxed);
            }
        });
        *self.holder.lock().unwrap_or_else(|p| p.into_inner()) = tokio::task::try_id();
        WritePermit {
            gate: self.clone(),
            site,
            granted_at: Instant::now(),
            permit: Some(permit),
        }
    }

    fn release(&self, held: Duration, site: Site) {
        *self.holder.lock().unwrap_or_else(|p| p.into_inner()) = None;
        let held_micros = micros(held);
        self.each(|c| {
            c.hold_micros_total.fetch_add(held_micros, Ordering::Relaxed);
            c.hold_micros_max.fetch_max(held_micros, Ordering::Relaxed);
        });
        if held >= LONG_HOLD {
            PROCESS.long_holds.fetch_add(1, Ordering::Relaxed);
            let count = self.counters.long_holds.fetch_add(1, Ordering::Relaxed) + 1;
            self.warn(format_args!(
                "SQLite write permit held {} ms by a {site} (threshold {} ms; {count} long holds since start); other in-process writers waited behind it",
                held.as_millis(),
                LONG_HOLD.as_millis()
            ));
        }
    }

    fn warn(&self, message: fmt::Arguments<'_>) {
        let now = self.started.elapsed().as_secs() + 1;
        let last = self.last_warning.load(Ordering::Relaxed);
        if (last == 0 || now >= last + WARN_EVERY.as_secs())
            && self
                .last_warning
                .compare_exchange(last, now, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
        {
            // No SQL, paths, identities or bind values.
            eprintln!("[mw-store] write gate: {message}");
        }
    }

    pub(crate) fn stats(&self) -> WriteGateStats {
        self.counters.snapshot()
    }

    /// Apply one counter update to this gate and to the process aggregate.
    fn each<T>(&self, update: impl Fn(&Counters) -> T) {
        update(&self.counters);
        update(&PROCESS);
    }
}

/// Queue-depth bookkeeping that survives cancellation of the waiting future.
struct Waiting<'a> {
    gate: &'a WriteGate,
    done: bool,
}
impl<'a> Waiting<'a> {
    fn enter(gate: &'a WriteGate) -> Self {
        let depth = gate.counters.queued.fetch_add(1, Ordering::Relaxed) + 1;
        gate.counters.queued_max.fetch_max(depth, Ordering::Relaxed);
        let process = PROCESS.queued.fetch_add(1, Ordering::Relaxed) + 1;
        PROCESS.queued_max.fetch_max(process, Ordering::Relaxed);
        Self { gate, done: false }
    }
    fn leave(mut self, cancelled: bool) {
        self.finish(cancelled);
    }
    fn finish(&mut self, cancelled: bool) {
        if !self.done {
            self.done = true;
            self.gate.each(|c| c.queued.fetch_sub(1, Ordering::Relaxed));
            if cancelled {
                self.gate.each(|c| c.cancelled.fetch_add(1, Ordering::Relaxed));
            }
        }
    }
}
impl Drop for Waiting<'_> {
    fn drop(&mut self) {
        self.finish(true);
    }
}

/// The write permit. Dropping it releases the gate to the next waiter.
pub(crate) struct WritePermit {
    gate: Arc<WriteGate>,
    site: Site,
    granted_at: Instant,
    permit: Option<OwnedSemaphorePermit>,
}
impl fmt::Debug for WritePermit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WritePermit").field("site", &self.site).finish_non_exhaustive()
    }
}
impl Drop for WritePermit {
    fn drop(&mut self) {
        // Account (and clear the holder) before the next waiter can be woken.
        self.gate.release(self.granted_at.elapsed(), self.site);
        drop(self.permit.take());
    }
}

/// The error a writer gets when the gate's wait limit passes: the same class
/// SQLite returns for a lock not released within `busy_timeout`.
#[derive(Debug)]
pub(crate) struct DatabaseLocked;
impl fmt::Display for DatabaseLocked {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("database is locked")
    }
}
impl std::error::Error for DatabaseLocked {}
impl sqlx::error::DatabaseError for DatabaseLocked {
    fn message(&self) -> &str {
        "database is locked"
    }
    fn code(&self) -> Option<std::borrow::Cow<'_, str>> {
        // SQLITE_BUSY
        Some("5".into())
    }
    fn as_error(&self) -> &(dyn std::error::Error + Send + Sync + 'static) {
        self
    }
    fn as_error_mut(&mut self) -> &mut (dyn std::error::Error + Send + Sync + 'static) {
        self
    }
    fn into_error(self: Box<Self>) -> Box<dyn std::error::Error + Send + Sync + 'static> {
        self
    }
    fn kind(&self) -> sqlx::error::ErrorKind {
        sqlx::error::ErrorKind::Other
    }
}
pub(crate) fn locked() -> sqlx::Error {
    sqlx::Error::Database(Box::new(DatabaseLocked))
}

#[cfg(test)]
#[path = "write_gate_tests.rs"]
mod tests;
