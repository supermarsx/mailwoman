//! Group-commit writer for a managed SQLite database (t50hp A2 stage 1,
//! `MW_SQLITE_WRITER=group`).
//!
//! SQLite has one writer at a time, and every store write transaction pays
//! for its own `BEGIN IMMEDIATE` … `COMMIT`. The commit is the expensive part
//! under load: it appends every page the transaction dirtied to the WAL (with
//! the 0125 projection triggers about 0.7 ms per single-message flag write,
//! against about 40 µs per row inside one bulk transaction), and every commit
//! makes the readers' page caches start again. This writer lets several
//! independent write jobs share one transaction and one commit.
//!
//! How a job runs
//! - A caller asks for a [`WriteLease`] on a [`Lane`]. Its ticket goes into a
//!   bounded queue; one writer task per database serves the queue in order.
//! - The writer opens one transaction for a batch (through
//!   `LiveSqlite::begin_at`: generation pin, write-gate permit, pooled
//!   connection, `BEGIN IMMEDIATE`, exactly as an ordinary transaction does).
//!   For each ticket it runs `SAVEPOINT mw_job` and hands the transaction to the
//!   caller. The caller runs its statements on its own task (so per-request
//!   statement accounting and borrowed inputs keep working) and hands the
//!   transaction back with [`WriteLease::finish`].
//! - A job that failed, or whose lease was dropped while it held the
//!   transaction (cancelled future, panic), is undone with `ROLLBACK TO
//!   mw_job`; the other jobs of the batch are not touched. A ticket whose caller
//!   went away before its turn is skipped: it never starts.
//! - Idle fast path: when nothing is queued or running, a job runs as an
//!   ordinary transaction of its own (no savepoint, no hand-over), exactly as
//!   with `gate`; it counts as in flight, so jobs arriving meanwhile queue and
//!   form the next batch.
//! - Lanes are served by weighted alternation: priority tickets first, but
//!   after [`PRIORITY_RUN`] priority grants in a row a waiting normal ticket gets
//!   the next turn, so neither lane can starve the other.
//! - The batch ends when no ticket is waiting, after [`MAX_BATCH_JOBS`] jobs,
//!   or once [`MAX_BATCH_TIME`] has passed since `BEGIN`. A batch that holds a
//!   priority job takes no further normal tickets except the guaranteed share,
//!   and ends as soon as no priority ticket is waiting: priority work is grouped
//!   with other priority work and committed promptly. Then ONE `COMMIT`, and
//!   only after it succeeded is every caller of the batch told so: a caller
//!   sees `Ok` only for a committed job. There is no waiting window: a lone job
//!   commits at once, batches form only while jobs queue behind a running one.
//! - If anything fails at batch level (the `COMMIT` itself, for example a
//!   deferred foreign key of one job, or a savepoint statement after SQLite
//!   aborted the transaction), the whole batch is rolled back and every caller
//!   of it runs its job again alone, in an ordinary transaction. So a job body
//!   must be re-runnable: SQL only, no side effect before its result is
//!   returned (true of every converted call site).
//!
//! Guarantees
//! - Durability: unchanged (`synchronous` as configured on the pool, NORMAL in
//!   WAL). A caller is answered after the commit that contains its job.
//! - Visibility: readers on other connections see the job once the caller has
//!   its answer (the WAL commit happened before it).
//! - Order: FIFO per lane; a later job of a batch sees the earlier ones.
//! - Runtime: the writer task is spawned on the runtime that created the
//!   writer (the store's), not on the first caller's.
//! - Bounded: [`QUEUE_JOBS`] normal and [`QUEUE_PRIORITY_JOBS`] priority
//!   tickets. A caller waits at most [`WAIT_LIMIT`] (5 s, the foreground
//!   `busy_timeout`) for its turn, then gets the same `database is locked`
//!   error a locked database gives.
//! - Priority lane delay: a priority ticket is granted after the job that holds
//!   the transaction now (and at most one normal job of the guaranteed share);
//!   its batch then ends when no priority ticket is waiting, at the latest
//!   [`MAX_BATCH_TIME`] after `BEGIN`. Bound: [`MAX_BATCH_TIME`] + two jobs +
//!   one COMMIT, independent of how many normal tickets are queued.
//! - Normal lane under a sustained priority stream: at least one grant in
//!   every `PRIORITY_RUN + 1` goes to a waiting normal ticket, so the normal
//!   lane keeps at least that share of the writer.
//! - A normal-lane wait can be escalated ([`WriteEscalation`]): while anyone
//!   raises it, the waiter moves to the priority lane (its normal ticket is
//!   withdrawn before its turn, never half-run). Used by a lock holder that
//!   others with priority work queue behind.
//! - A writer that does not use the writer (`begin()`, autocommit writes) waits
//!   in the write gate for at most one batch per batch ahead of it:
//!   [`MAX_BATCH_TIME`] plus the job running at that moment plus the commit.
//!
//! Rules for job bodies (every future conversion must follow them)
//! - SQL on the given transaction only: no pool access, no `begin()`, no second
//!   lease, no network or file I/O, no other side effect before the result is
//!   returned. A body can run twice (a failed batch re-runs it alone).
//! - Never swallow a statement error (`let _ =`, `.ok()`, `unwrap_or`): return
//!   it. If SQLite aborted the transaction (SQLITE_FULL, IOERR, ...) and a body
//!   continued, its later statements would run in autocommit outside the batch.
//!   The writer's next statement after such a job (`RELEASE` of its savepoint,
//!   or the batch `COMMIT`) fails with "no such savepoint" / "no transaction is
//!   active"; the writer then fails that job (its caller gets an error, it is
//!   not re-run, so its stray statements are not applied twice), re-runs the
//!   other members alone, closes the connection instead of pooling it, and
//!   panics in debug builds so tests catch the body. Reviewers of a conversion
//!   check the body against these rules.
//! - Post-commit bookkeeping that must happen once the job committed belongs in
//!   a task that owns its completion (`tokio::spawn`), not in a request future
//!   that a disconnecting client can drop between the commit and the bookkeeping.
//!
//! Writes that do not use the writer still work and stay correct: `begin()`
//! and autocommit writes take the write gate, and the writer takes the same
//! gate for each batch, so at any time one batch or one ordinary transaction
//! holds the SQLite write lock. A nested write from inside a job (a pool
//! write, `begin()`, or a second lease) waits for the gate or the queue and
//! fails after [`WAIT_LIMIT`], as the same code fails today.
use std::{
    fmt,
    sync::{
        Arc, Mutex, Weak,
        atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};
use tokio::sync::{Notify, mpsc, oneshot};

use super::{Inner, LiveSqlite};
use crate::backend::Tx;

pub(crate) use super::write_gate::WAIT_LIMIT;

/// Most jobs in one batch (one commit).
pub(crate) const MAX_BATCH_JOBS: usize = 64;
/// No new job joins a batch once this much time has passed since its `BEGIN`.
/// Measured (t50hp-e19, encrypted file, single-message flag writes, release):
/// derived from the ladder, a job costs about 0.6-0.7 ms inside a batch and a
/// commit about 1.5-2 ms (not timed separately), so
/// 2 ms batches held 1.8 jobs at 64 callers, 10 ms held 13, 20 ms held 32. The
/// window only matters under load (an empty queue ends a batch at once); it
/// bounds how long a writer outside the group writer waits in the gate.
pub(crate) const MAX_BATCH_TIME: Duration = Duration::from_millis(10);
/// Queue bound of the normal lane (tickets, not bytes: a ticket is a few
/// pointers; a job's data stays with its caller).
pub(crate) const QUEUE_JOBS: usize = 1024;
/// Queue bound of the priority lane.
pub(crate) const QUEUE_PRIORITY_JOBS: usize = 256;

/// Priority grants in a row after which a waiting normal ticket is served.
pub(crate) const PRIORITY_RUN: u32 = 3;

const SAVEPOINT: &str = "SAVEPOINT mw_job";
/// The previous job's savepoint is released before the next one opens (one
/// statement round trip): nested savepoints would make every page write of a
/// later job do work per open savepoint (measured: a 1,024-job move batch at
/// 13 ms per job nested, against about 1.3 ms at 64).
const RELEASE_AND_SAVEPOINT: &str = "RELEASE mw_job; SAVEPOINT mw_job";
const ROLLBACK_TO: &str = "ROLLBACK TO mw_job";

/// Which queue a write job waits in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lane {
    /// Real T0 writes only (throttle and block transitions, administrative
    /// clears): served before normal tickets (with the normal lane's guaranteed
    /// share), grouped with other priority work and committed promptly.
    Priority,
    /// Acknowledged-after-commit writes (T1) and background flushes (T3).
    Normal,
}

/// A request for a lane, optionally escalatable (see [`WriteEscalation`]).
#[derive(Clone, Debug)]
pub struct LaneRequest {
    pub(crate) lane: Lane,
    pub(crate) escalation: Option<WriteEscalation>,
}
impl From<Lane> for LaneRequest {
    fn from(lane: Lane) -> Self {
        Self { lane, escalation: None }
    }
}
impl LaneRequest {
    /// A normal-lane wait that moves to the priority lane while `escalation`
    /// is raised.
    pub fn escalatable(escalation: &WriteEscalation) -> Self {
        Self { lane: Lane::Normal, escalation: Some(escalation.clone()) }
    }
}

/// Moves normal-lane waits to the priority lane while raised. For a lock
/// holder (for example the accounting timer flush) that callers with priority
/// work queue behind: they raise it before taking the lock, so the holder's
/// wait no longer depends on the normal lane's backlog.
#[derive(Clone, Default)]
pub struct WriteEscalation(Arc<EscalationInner>);
#[derive(Default)]
struct EscalationInner {
    raised: AtomicUsize,
    notify: Notify,
}
impl fmt::Debug for WriteEscalation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WriteEscalation").field("raised", &self.0.raised.load(Ordering::Relaxed)).finish()
    }
}
impl WriteEscalation {
    pub fn new() -> Self {
        Self::default()
    }
    /// Raise until the returned guard drops.
    pub fn raise(&self) -> EscalationGuard {
        self.0.raised.fetch_add(1, Ordering::AcqRel);
        self.0.notify.notify_waiters();
        EscalationGuard(self.clone())
    }
    pub fn raised(&self) -> bool {
        self.0.raised.load(Ordering::Acquire) > 0
    }
}
/// Keeps a [`WriteEscalation`] raised.
pub struct EscalationGuard(WriteEscalation);
impl Drop for EscalationGuard {
    fn drop(&mut self) {
        self.0.0.raised.fetch_sub(1, Ordering::AcqRel);
    }
}

/// Counters of one writer. Times in microseconds.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SqliteWriterStats {
    /// Committed batches (one `COMMIT` each).
    pub commits: u64,
    /// Jobs answered after a successful commit (succeeded or failed alone).
    pub jobs: u64,
    pub priority_jobs: u64,
    /// Jobs that returned an error and were rolled back to their savepoint.
    pub job_errors: u64,
    /// Leases dropped while they held the transaction (cancelled or panicked):
    /// rolled back to their savepoint.
    pub abandoned: u64,
    /// Tickets whose caller went away before their turn (never started).
    pub skipped: u64,
    /// Batches rolled back as a whole; their callers ran again alone.
    pub batch_failures: u64,
    /// Callers that waited longer than the wait limit (answered locked).
    pub refusals: u64,
    /// Largest batch.
    pub batch_jobs_max: u64,
    /// Tickets queued or running now, and the most at once.
    pub in_flight: u64,
    pub in_flight_max: u64,
    pub batch_micros_total: u64,
    pub batch_micros_max: u64,
    /// Longest time one job held the batch transaction.
    pub job_hold_micros_max: u64,
    /// Time split of the batches (writer task's view): the jobs holding the
    /// transaction (their statements plus the hand-over to and from the
    /// caller's task), the `SAVEPOINT` / `ROLLBACK TO` statements, `BEGIN`
    /// (including the gate and pool wait) and `COMMIT`.
    pub job_hold_micros_total: u64,
    pub savepoint_micros_total: u64,
    pub begin_micros_total: u64,
    pub commit_micros_total: u64,
    /// Longest wait of a priority ticket for its grant.
    pub priority_wait_micros_max: u64,
    /// Normal tickets served as the guaranteed share while priority waited.
    pub normal_share_grants: u64,
    /// Normal-lane waits moved to the priority lane by an escalation.
    pub escalations: u64,
    /// Callers that ran alone because the batch broke before their turn.
    pub solo_fallbacks: u64,
    /// Jobs that returned `Ok` with the connection outside the batch
    /// transaction (a swallowed fatal error); failed, not re-run.
    pub left_transaction: u64,
    /// Writer tasks started (1 per database unless its runtime ended).
    pub writer_starts: u64,
    /// Jobs run as their own transaction because the writer was idle.
    pub solo_idle: u64,
}

#[derive(Default)]
struct Counters {
    commits: AtomicU64,
    jobs: AtomicU64,
    priority_jobs: AtomicU64,
    job_errors: AtomicU64,
    abandoned: AtomicU64,
    skipped: AtomicU64,
    batch_failures: AtomicU64,
    refusals: AtomicU64,
    batch_jobs_max: AtomicU64,
    in_flight: AtomicU64,
    in_flight_max: AtomicU64,
    batch_micros_total: AtomicU64,
    batch_micros_max: AtomicU64,
    job_hold_micros_max: AtomicU64,
    job_hold_micros_total: AtomicU64,
    savepoint_micros_total: AtomicU64,
    begin_micros_total: AtomicU64,
    commit_micros_total: AtomicU64,
    priority_wait_micros_max: AtomicU64,
    normal_share_grants: AtomicU64,
    escalations: AtomicU64,
    solo_fallbacks: AtomicU64,
    left_transaction: AtomicU64,
    writer_starts: AtomicU64,
    solo_idle: AtomicU64,
}
impl Counters {
    const fn new() -> Self {
        Self {
            commits: AtomicU64::new(0),
            jobs: AtomicU64::new(0),
            priority_jobs: AtomicU64::new(0),
            job_errors: AtomicU64::new(0),
            abandoned: AtomicU64::new(0),
            skipped: AtomicU64::new(0),
            batch_failures: AtomicU64::new(0),
            refusals: AtomicU64::new(0),
            batch_jobs_max: AtomicU64::new(0),
            in_flight: AtomicU64::new(0),
            in_flight_max: AtomicU64::new(0),
            batch_micros_total: AtomicU64::new(0),
            batch_micros_max: AtomicU64::new(0),
            job_hold_micros_max: AtomicU64::new(0),
            job_hold_micros_total: AtomicU64::new(0),
            savepoint_micros_total: AtomicU64::new(0),
            begin_micros_total: AtomicU64::new(0),
            commit_micros_total: AtomicU64::new(0),
            priority_wait_micros_max: AtomicU64::new(0),
            normal_share_grants: AtomicU64::new(0),
            escalations: AtomicU64::new(0),
            solo_fallbacks: AtomicU64::new(0),
            left_transaction: AtomicU64::new(0),
            writer_starts: AtomicU64::new(0),
            solo_idle: AtomicU64::new(0),
        }
    }
    fn snapshot(&self) -> SqliteWriterStats {
        let load = |value: &AtomicU64| value.load(Ordering::Relaxed);
        SqliteWriterStats {
            commits: load(&self.commits),
            jobs: load(&self.jobs),
            priority_jobs: load(&self.priority_jobs),
            job_errors: load(&self.job_errors),
            abandoned: load(&self.abandoned),
            skipped: load(&self.skipped),
            batch_failures: load(&self.batch_failures),
            refusals: load(&self.refusals),
            batch_jobs_max: load(&self.batch_jobs_max),
            in_flight: load(&self.in_flight),
            in_flight_max: load(&self.in_flight_max),
            batch_micros_total: load(&self.batch_micros_total),
            batch_micros_max: load(&self.batch_micros_max),
            job_hold_micros_max: load(&self.job_hold_micros_max),
            job_hold_micros_total: load(&self.job_hold_micros_total),
            savepoint_micros_total: load(&self.savepoint_micros_total),
            begin_micros_total: load(&self.begin_micros_total),
            commit_micros_total: load(&self.commit_micros_total),
            priority_wait_micros_max: load(&self.priority_wait_micros_max),
            normal_share_grants: load(&self.normal_share_grants),
            escalations: load(&self.escalations),
            solo_fallbacks: load(&self.solo_fallbacks),
            left_transaction: load(&self.left_transaction),
            writer_starts: load(&self.writer_starts),
            solo_idle: load(&self.solo_idle),
        }
    }
}

/// Sum over every writer of the process (maxima: the largest of any writer).
static PROCESS: Counters = Counters::new();
static PROCESS_WRITERS: AtomicU64 = AtomicU64::new(0);

/// Process-wide writer counters, `None` while no writer exists.
pub(crate) fn process_stats() -> Option<SqliteWriterStats> {
    (PROCESS_WRITERS.load(Ordering::Relaxed) > 0).then(|| PROCESS.snapshot())
}

fn micros(duration: Duration) -> u64 {
    duration.as_micros().min(u128::from(u64::MAX)) as u64
}

/// Batch limits (constants in production; tests shrink or widen them).
#[derive(Clone, Copy, Debug)]
pub(crate) struct Limits {
    pub(crate) max_jobs: usize,
    pub(crate) max_time: Duration,
    pub(crate) queue: usize,
    pub(crate) queue_priority: usize,
    pub(crate) wait_limit: Duration,
    /// An idle writer (nothing queued or running) runs a job as an ordinary
    /// transaction of its own: no savepoint, no hand-over, exactly the `gate`
    /// path, so a lone write is never slower than without the writer.
    pub(crate) solo_when_idle: bool,
}
/// Test builds only: a batch-time override (microseconds, 0 = the constant)
/// for writers created after it is set; the benchmark compares windows.
#[cfg(test)]
pub(crate) static TEST_MAX_BATCH_MICROS: AtomicU64 = AtomicU64::new(0);
impl Default for Limits {
    fn default() -> Self {
        #[cfg(test)]
        let max_time = match TEST_MAX_BATCH_MICROS.load(Ordering::Relaxed) {
            0 => MAX_BATCH_TIME,
            micros => Duration::from_micros(micros),
        };
        #[cfg(not(test))]
        let max_time = MAX_BATCH_TIME;
        Self {
            max_jobs: MAX_BATCH_JOBS,
            max_time,
            queue: QUEUE_JOBS,
            queue_priority: QUEUE_PRIORITY_JOBS,
            wait_limit: WAIT_LIMIT,
            solo_when_idle: true,
        }
    }
}

struct Shared {
    counters: Counters,
    limits: Limits,
    /// Tickets submitted and not yet answered (or skipped).
    in_flight: AtomicUsize,
    idle: Notify,
    closing: AtomicBool,
    /// The runtime the writer task runs on: the store's (G6).
    runtime: Mutex<Option<tokio::runtime::Handle>>,
    /// Test hook: the next SAVEPOINT statement fails as if SQLite had ended
    /// the transaction.
    #[cfg(test)]
    fail_next_savepoint: AtomicBool,
}
impl Shared {
    fn each(&self, update: impl Fn(&Counters)) {
        update(&self.counters);
        update(&PROCESS);
    }
    fn max(&self, pick: impl Fn(&Counters) -> &AtomicU64, value: u64) {
        self.each(|c| {
            pick(c).fetch_max(value, Ordering::Relaxed);
        });
    }
    fn add(&self, pick: impl Fn(&Counters) -> &AtomicU64, value: u64) {
        self.each(|c| {
            pick(c).fetch_add(value, Ordering::Relaxed);
        });
    }
}

/// Counts a ticket from submission until its caller has been answered.
struct InFlight(Arc<Shared>);
impl InFlight {
    fn enter(shared: &Arc<Shared>) -> Self {
        let now = shared.in_flight.fetch_add(1, Ordering::AcqRel) as u64 + 1;
        shared.add(|c| &c.in_flight, 1);
        shared.max(|c| &c.in_flight_max, now);
        Self(shared.clone())
    }
}
impl Drop for InFlight {
    fn drop(&mut self) {
        self.0.each(|c| {
            c.in_flight.fetch_sub(1, Ordering::Relaxed);
        });
        if self.0.in_flight.fetch_sub(1, Ordering::AcqRel) == 1 {
            self.0.idle.notify_waiters();
        }
    }
}

/// Why a ticket got no grant.
enum Refused {
    /// The batch could not start (gate wait limit, closed pool).
    Error(sqlx::Error),
    /// The batch broke before this ticket's turn: run the job alone.
    Solo,
}

struct Ticket {
    grant: oneshot::Sender<Result<Grant, Refused>>,
    lane: Lane,
    queued_at: Instant,
    flight: InFlight,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Status {
    /// The job finished with `Ok`: keep its statements.
    Done,
    /// The job returned an error: undo it.
    Failed,
    /// The lease was dropped while it held the transaction: undo it.
    Abandoned,
}
struct Returned {
    tx: Tx,
    status: Status,
}
#[derive(Debug)]
enum Outcome {
    Committed,
    /// The batch was rolled back as a whole: run the job again alone.
    Rerun,
    /// The job left the batch transaction (a swallowed fatal error): it is
    /// not re-run; its statements outside the batch may have been applied.
    LeftTransaction,
}

struct Member {
    done: oneshot::Sender<Outcome>,
    _flight: InFlight,
}

/// The batch transaction while one job holds it.
pub(crate) struct Grant {
    tx: Option<Tx>,
    back: Option<oneshot::Sender<Returned>>,
    done: Option<oneshot::Receiver<Outcome>>,
}
impl Drop for Grant {
    fn drop(&mut self) {
        // Cancelled, panicked, or never picked up: the writer undoes this job
        // and continues with the next one.
        if let (Some(tx), Some(back)) = (self.tx.take(), self.back.take()) {
            let _ = back.send(Returned {
                tx,
                status: Status::Abandoned,
            });
        }
    }
}

/// Exclusive use of a write transaction for one job.
///
/// Solo: an ordinary transaction of its own (PostgreSQL, `gate`/`off`, or a
/// re-run after a failed batch). Grouped: a savepoint inside the writer's
/// batch transaction. Use [`crate::backend::write_job!`] rather than this type
/// directly: it re-runs the job alone when the batch fails.
pub(crate) struct WriteLease(LeaseKind);
enum LeaseKind {
    /// The transaction, and (fast path) the in-flight mark that makes later
    /// arrivals queue for a batch while it runs.
    Solo(Option<Tx>, Option<InFlight>),
    Grouped(Grant),
}

/// What [`WriteLease::finish`] returns.
pub(crate) enum Finished<T, E> {
    /// The job's own result; for `Ok`, its statements are committed.
    Done(Result<T, E>),
    /// The batch failed as a whole; nothing of the job is stored. Run it again
    /// with a solo lease.
    Rerun,
}

impl WriteLease {
    pub(crate) fn solo(tx: Tx) -> Self {
        Self(LeaseKind::Solo(Some(tx), None))
    }
    pub(crate) fn tx(&mut self) -> &mut Tx {
        match &mut self.0 {
            LeaseKind::Solo(tx, _) => tx.as_mut().expect("lease holds its transaction"),
            LeaseKind::Grouped(grant) => grant.tx.as_mut().expect("lease holds its transaction"),
        }
    }
    /// End the job with its result. `Ok` is returned only after the commit
    /// that contains the job; an `Err` result rolls the job back.
    pub(crate) async fn finish<T, E: From<sqlx::Error>>(mut self, result: Result<T, E>) -> Finished<T, E> {
        match &mut self.0 {
            LeaseKind::Solo(tx, _) => {
                let tx = tx.take().expect("lease holds its transaction");
                match result {
                    Ok(value) => match tx.commit().await {
                        Ok(()) => Finished::Done(Ok(value)),
                        Err(error) => Finished::Done(Err(error.into())),
                    },
                    Err(error) => {
                        // The job's error wins over a rollback failure; a failed
                        // rollback still ends the transaction when it drops.
                        let _ = tx.rollback().await;
                        Finished::Done(Err(error))
                    }
                }
            }
            LeaseKind::Grouped(grant) => {
                let tx = grant.tx.take().expect("lease holds its transaction");
                let back = grant.back.take().expect("lease holds its return channel");
                let done = grant.done.take().expect("lease holds its outcome channel");
                let status = if result.is_ok() { Status::Done } else { Status::Failed };
                // A writer that is gone drops the returned transaction, which
                // rolls the whole batch back: nothing of this job is stored.
                if back.send(Returned { tx, status }).is_err() {
                    return Finished::Done(Err(writer_stopped().into()));
                }
                match done.await {
                    Ok(Outcome::Committed) => Finished::Done(result),
                    Ok(Outcome::Rerun) => Finished::Rerun,
                    Ok(Outcome::LeftTransaction) => Finished::Done(Err(sqlx::Error::Protocol(
                        "write job returned Ok after its statements left the batch transaction (a swallowed SQLite error)".into(),
                    )
                    .into())),
                    Err(_) => Finished::Done(Err(writer_stopped().into())),
                }
            }
        }
    }
}

/// The error of a caller whose writer stopped before the batch committed:
/// nothing of the job is stored.
fn writer_stopped() -> sqlx::Error {
    sqlx::Error::WorkerCrashed
}

struct Channels {
    priority: mpsc::Sender<Ticket>,
    normal: mpsc::Sender<Ticket>,
    /// Used by tests to stop the task as a crash would.
    #[cfg_attr(not(test), allow(dead_code))]
    task: tokio::task::AbortHandle,
}

/// The group-commit writer of one managed SQLite database.
pub(crate) struct GroupWriter {
    shared: Arc<Shared>,
    channels: Mutex<Option<Channels>>,
}
impl fmt::Debug for GroupWriter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GroupWriter").field("stats", &self.stats()).finish()
    }
}

impl GroupWriter {
    pub(crate) fn new() -> Arc<Self> {
        Self::with_limits(Limits::default())
    }
    pub(crate) fn with_limits(limits: Limits) -> Arc<Self> {
        PROCESS_WRITERS.fetch_add(1, Ordering::Relaxed);
        Arc::new(Self {
            shared: Arc::new(Shared {
                counters: Counters::new(),
                limits,
                in_flight: AtomicUsize::new(0),
                idle: Notify::new(),
                closing: AtomicBool::new(false),
                runtime: Mutex::new(tokio::runtime::Handle::try_current().ok()),
                #[cfg(test)]
                fail_next_savepoint: AtomicBool::new(false),
            }),
            channels: Mutex::new(None),
        })
    }

    pub(crate) fn stats(&self) -> SqliteWriterStats {
        self.shared.counters.snapshot()
    }

    /// Wait for a turn in the batch transaction. Cancel-safe: a dropped future
    /// either never gets a turn (its ticket is skipped) or, if the grant was
    /// already sent, returns it unused (the writer undoes the empty savepoint).
    /// A ticket whose batch broke before its turn runs alone (`Refused::Solo`).
    pub(crate) async fn lease(
        &self,
        live: &LiveSqlite,
        request: LaneRequest,
        at: &'static std::panic::Location<'static>,
    ) -> Result<WriteLease, sqlx::Error> {
        if self.shared.closing.load(Ordering::Acquire) {
            return Err(sqlx::Error::PoolClosed);
        }
        // Idle fast path: nothing queued or running, so there is nothing to
        // group with. Counted in flight until it ends, so a job arriving
        // meanwhile queues for a batch (which starts after this commit).
        if self.shared.limits.solo_when_idle && self.shared.in_flight.load(Ordering::Acquire) == 0 {
            let flight = InFlight::enter(&self.shared);
            if self.shared.in_flight.load(Ordering::Acquire) == 1 {
                self.shared.add(|c| &c.solo_idle, 1);
                let tx = Tx::Sqlite(live.begin_at(at).await?);
                return Ok(WriteLease(LeaseKind::Solo(Some(tx), Some(flight))));
            }
            drop(flight);
        }
        let waited = tokio::time::timeout(self.shared.limits.wait_limit, self.wait_turn(live, request)).await;
        match waited {
            Ok(Ok(grant)) => Ok(WriteLease(LeaseKind::Grouped(grant))),
            Ok(Err(Refused::Solo)) => {
                self.shared.add(|c| &c.solo_fallbacks, 1);
                Ok(WriteLease::solo(Tx::Sqlite(live.begin_at(at).await?)))
            }
            Ok(Err(Refused::Error(error))) => Err(error),
            Err(_) => {
                self.shared.add(|c| &c.refusals, 1);
                Err(super::write_gate::locked())
            }
        }
    }

    /// One ticket on the requested lane; a normal-lane wait moves to the
    /// priority lane when its escalation is raised (before or while waiting).
    async fn wait_turn(&self, live: &LiveSqlite, request: LaneRequest) -> Result<Grant, Refused> {
        let escalation = match (request.lane, &request.escalation) {
            (Lane::Normal, Some(escalation)) => Some(escalation.clone()),
            _ => None,
        };
        let mut lane = request.lane;
        loop {
            if lane == Lane::Normal && escalation.as_ref().is_some_and(WriteEscalation::raised) {
                self.shared.add(|c| &c.escalations, 1);
                lane = Lane::Priority;
            }
            let (grant, mut granted) = oneshot::channel();
            let ticket = Ticket {
                grant,
                lane,
                queued_at: Instant::now(),
                flight: InFlight::enter(&self.shared),
            };
            self.submit(live, ticket).await.map_err(Refused::Error)?;
            let Some(escalation) = escalation.as_ref().filter(|_| lane == Lane::Normal) else {
                return granted.await.map_err(|_| Refused::Error(writer_stopped()))?;
            };
            let raised = escalation.0.notify.notified();
            tokio::pin!(raised);
            raised.as_mut().enable();
            if !escalation.raised() {
                tokio::select! {
                    biased;
                    result = &mut granted => return result.map_err(|_| Refused::Error(writer_stopped()))?,
                    _ = &mut raised => {}
                }
            }
            // Withdraw the normal ticket: dropping its receiver makes the writer
            // skip it, or (if its grant was already sent) return the unused
            // grant, whose empty savepoint is undone. Then wait on priority.
            drop(granted);
        }
    }

    async fn submit(&self, live: &LiveSqlite, mut ticket: Ticket) -> Result<(), sqlx::Error> {
        // A writer task ends with the runtime that spawned it (tests, and a
        // store reused by a later runtime); its closed queue is replaced once,
        // then on the caller's runtime.
        for attempt in 0..2 {
            let sender = self.sender(live, ticket.lane, attempt > 0);
            match sender.send(ticket).await {
                Ok(()) => return Ok(()),
                Err(mpsc::error::SendError(returned)) => {
                    ticket = returned;
                    self.reset(&sender);
                }
            }
        }
        Err(writer_stopped())
    }

    fn sender(&self, live: &LiveSqlite, lane: Lane, on_current_runtime: bool) -> mpsc::Sender<Ticket> {
        let mut channels = self.channels.lock().unwrap_or_else(|p| p.into_inner());
        if channels
            .as_ref()
            .is_none_or(|c| c.normal.is_closed() || c.priority.is_closed())
        {
            let (priority, priority_rx) = mpsc::channel(self.shared.limits.queue_priority.max(1));
            let (normal, normal_rx) = mpsc::channel(self.shared.limits.queue.max(1));
            let task = run(Arc::downgrade(&live.inner), self.shared.clone(), priority_rx, normal_rx);
            // The store's runtime (captured when the writer was created), so a
            // caller on a short-lived secondary runtime cannot take the writer
            // down with it. If that runtime has ended, the caller's runtime.
            let mut runtime = self.shared.runtime.lock().unwrap_or_else(|p| p.into_inner());
            if on_current_runtime || runtime.is_none() {
                *runtime = tokio::runtime::Handle::try_current().ok();
            }
            let handle = runtime.clone().unwrap_or_else(tokio::runtime::Handle::current);
            drop(runtime);
            self.shared.add(|c| &c.writer_starts, 1);
            let task = handle.spawn(task).abort_handle();
            *channels = Some(Channels { priority, normal, task });
        }
        let channels = channels.as_ref().expect("channels just ensured");
        match lane {
            Lane::Priority => channels.priority.clone(),
            Lane::Normal => channels.normal.clone(),
        }
    }

    fn reset(&self, failed: &mpsc::Sender<Ticket>) {
        let mut channels = self.channels.lock().unwrap_or_else(|p| p.into_inner());
        if channels
            .as_ref()
            .is_some_and(|c| c.normal.same_channel(failed) || c.priority.same_channel(failed))
        {
            *channels = None;
        }
    }

    /// Wait until every submitted ticket has been answered, at most until
    /// `deadline`. New tickets are still admitted (used by the shutdown drain
    /// before the last flushes). Returns whether the writer was idle in time.
    pub(crate) async fn drain(&self, deadline: tokio::time::Instant) -> bool {
        loop {
            let idle = self.shared.idle.notified();
            tokio::pin!(idle);
            idle.as_mut().enable();
            if self.shared.in_flight.load(Ordering::Acquire) == 0 {
                return true;
            }
            if tokio::time::timeout_at(deadline, idle).await.is_err() {
                return self.shared.in_flight.load(Ordering::Acquire) == 0;
            }
        }
    }

    /// Stop admitting tickets (later callers get `PoolClosed`), then drain.
    pub(crate) async fn close(&self, deadline: tokio::time::Instant) -> bool {
        self.shared.closing.store(true, Ordering::Release);
        self.drain(deadline).await
    }

    /// Stop the writer task at once, as a crash would (tests).
    #[cfg(test)]
    pub(crate) fn abort_task(&self) {
        let channels = self.channels.lock().unwrap_or_else(|p| p.into_inner());
        if let Some(channels) = channels.as_ref() {
            channels.task.abort();
        }
    }
}

impl Drop for GroupWriter {
    fn drop(&mut self) {
        if let Some(channels) = self.channels.get_mut().unwrap_or_else(|p| p.into_inner()).take() {
            // Senders drop here; the task ends after the batch it is running.
            drop(channels.priority);
            drop(channels.normal);
        }
    }
}

/// The writer task: holds no strong reference to the database between
/// batches, and ends when its queues close.
async fn run(
    weak: Weak<Inner>,
    shared: Arc<Shared>,
    mut priority: mpsc::Receiver<Ticket>,
    mut normal: mpsc::Receiver<Ticket>,
) {
    let mut turns = Turns::default();
    loop {
        let first = match turns.pick(&mut priority, &mut normal, false) {
            Some(ticket) => Some(ticket),
            None => {
                let ticket = tokio::select! {
                    biased;
                    ticket = priority.recv() => ticket,
                    ticket = normal.recv() => ticket,
                };
                if let Some(ticket) = &ticket {
                    turns.granted(ticket.lane);
                }
                ticket
            }
        };
        let Some(first) = first else { break };
        let Some(inner) = weak.upgrade() else {
            let _ = first.grant.send(Err(Refused::Error(sqlx::Error::PoolClosed)));
            break;
        };
        let live = LiveSqlite { inner };
        batch(&live, &shared, first, &mut priority, &mut normal, &mut turns).await;
    }
}

/// Weighted alternation between the lanes (G2): priority first, but after
/// [`PRIORITY_RUN`] priority grants in a row a waiting normal ticket is next.
#[derive(Default)]
struct Turns {
    priority_run: u32,
    shared_turn: bool,
}
impl Turns {
    fn granted(&mut self, lane: Lane) {
        self.shared_turn = false;
        match lane {
            Lane::Priority => self.priority_run += 1,
            Lane::Normal => self.priority_run = 0,
        }
    }
    /// The next ticket without waiting. `priority_only`: the batch already
    /// holds priority work, so normal tickets come only as the guaranteed share.
    fn pick(
        &mut self,
        priority: &mut mpsc::Receiver<Ticket>,
        normal: &mut mpsc::Receiver<Ticket>,
        priority_only: bool,
    ) -> Option<Ticket> {
        if self.priority_run >= PRIORITY_RUN {
            if let Ok(ticket) = normal.try_recv() {
                self.granted(Lane::Normal);
                self.shared_turn = true;
                return Some(ticket);
            }
        }
        if let Ok(ticket) = priority.try_recv() {
            self.granted(Lane::Priority);
            return Some(ticket);
        }
        if priority_only {
            return None;
        }
        normal.try_recv().ok().inspect(|_| self.granted(Lane::Normal))
    }
}

async fn execute(tx: &mut Tx, sql: &'static str) -> Result<(), sqlx::Error> {
    match tx {
        Tx::Sqlite(live) => sqlx::query(sql).execute(&mut **live).await.map(|_| ()),
        Tx::Postgres(_) => Err(sqlx::Error::Protocol("group writer is SQLite only".into())),
    }
}

/// G5: the error SQLite gives the writer's next statement when the previous
/// job swallowed a fatal error and returned `Ok` (the batch transaction, and
/// with it the job savepoint, no longer exists).
fn left_transaction(error: &sqlx::Error) -> bool {
    let text = error.to_string();
    text.contains("no such savepoint") || text.contains("no transaction is active")
}

fn note_left(shared: &Shared) {
    shared.add(|c| &c.left_transaction, 1);
    debug_assert!(
        cfg!(test) && TOLERATE_LEFT_TRANSACTION.load(Ordering::Relaxed),
        "group writer: a write job returned Ok after its statements left the batch transaction (a swallowed SQLite error in the job body)"
    );
}
/// Test builds: the G5 test provokes the condition on purpose.
static TOLERATE_LEFT_TRANSACTION: AtomicBool = AtomicBool::new(false);

#[track_caller]
fn here() -> &'static std::panic::Location<'static> {
    std::panic::Location::caller()
}

async fn batch(
    live: &LiveSqlite,
    shared: &Arc<Shared>,
    first: Ticket,
    priority: &mut mpsc::Receiver<Ticket>,
    normal: &mut mpsc::Receiver<Ticket>,
    turns: &mut Turns,
) {
    // Skip callers that left before BEGIN: an empty batch is not opened.
    let mut next = Some(first);
    while let Some(ticket) = next.take() {
        if !ticket.grant.is_closed() {
            next = Some(ticket);
            break;
        }
        shared.add(|c| &c.skipped, 1);
        next = turns.pick(priority, normal, false);
    }
    let Some(first) = next else { return };

    let begun = Instant::now();
    let mut tx: Option<Tx> = match live.begin_at(here()).await {
        Ok(tx) => Some(Tx::Sqlite(tx)),
        Err(error) => {
            // Gate wait limit, closed pool: this caller alone hears it; the
            // queued tickets get their own attempt in the next round.
            let _ = first.grant.send(Err(Refused::Error(error)));
            return;
        }
    };
    shared.add(|c| &c.begin_micros_total, micros(begun.elapsed()));
    let mut members: Vec<Member> = Vec::new();
    // The member whose job left the transaction (failed, never re-run).
    let mut left: Option<Member> = None;
    let mut broken = false;
    let mut holds_priority = false;
    // Whether the last member's job returned Ok (G5 attribution).
    let mut last_done = false;
    // Whether a job's savepoint is still open (released before the next opens;
    // COMMIT releases the last one).
    let mut open = false;
    let mut next = Some(first);
    while let Some(ticket) = next.take() {
        let Ticket {
            grant,
            lane,
            queued_at,
            flight,
        } = ticket;
        if turns.shared_turn {
            shared.add(|c| &c.normal_share_grants, 1);
        }
        if grant.is_closed() {
            shared.add(|c| &c.skipped, 1);
        } else {
            let mut current = tx.take().expect("batch transaction present");
            let savepoint_at = Instant::now();
            #[cfg(test)]
            let injected = shared.fail_next_savepoint.swap(false, Ordering::AcqRel);
            #[cfg(not(test))]
            let injected = false;
            let opened = if injected {
                Err(sqlx::Error::Protocol("injected savepoint failure".into()))
            } else {
                execute(&mut current, if open { RELEASE_AND_SAVEPOINT } else { SAVEPOINT }).await
            };
            open = true;
            shared.add(|c| &c.savepoint_micros_total, micros(savepoint_at.elapsed()));
            if let Err(error) = &opened {
                // The batch is lost; this caller did nothing wrong: it runs
                // alone (G4), the earlier members re-run alone. If the previous
                // job left the transaction (G5), it alone is failed, not re-run.
                if last_done && left_transaction(error) {
                    note_left(shared);
                    left = members.pop();
                }
                broken = true;
                let _ = grant.send(Err(Refused::Solo));
                current.close_on_drop();
                drop(current);
                break;
            }
            let (back, returned) = oneshot::channel();
            let (done, outcome) = oneshot::channel();
            let granted_at = Instant::now();
            if lane == Lane::Priority {
                shared.max(
                    |c| &c.priority_wait_micros_max,
                    micros(granted_at.duration_since(queued_at)),
                );
            }
            let sent = grant.send(Ok(Grant {
                tx: Some(current),
                back: Some(back),
                done: Some(outcome),
            }));
            match sent {
                Err(Ok(mut unused)) => {
                    // The caller left after the check: take the transaction back,
                    // its savepoint is empty.
                    tx = unused.tx.take();
                    unused.back = None;
                    shared.add(|c| &c.skipped, 1);
                }
                Err(Err(_)) => unreachable!("only grants are sent"),
                Ok(()) => {
                    let member = Member { done, _flight: flight };
                    match returned.await {
                        Ok(Returned { tx: back_tx, status }) => {
                            let held = micros(granted_at.elapsed());
                            shared.max(|c| &c.job_hold_micros_max, held);
                            shared.add(|c| &c.job_hold_micros_total, held);
                            let mut back_tx = back_tx;
                            last_done = status == Status::Done;
                            if status != Status::Done {
                                shared.add(
                                    |c| {
                                        if status == Status::Failed {
                                            &c.job_errors
                                        } else {
                                            &c.abandoned
                                        }
                                    },
                                    1,
                                );
                                let undo_at = Instant::now();
                                let undone = execute(&mut back_tx, ROLLBACK_TO).await;
                                shared.add(|c| &c.savepoint_micros_total, micros(undo_at.elapsed()));
                                if undone.is_err() {
                                    broken = true;
                                    members.push(member);
                                    back_tx.close_on_drop();
                                    drop(back_tx);
                                    break;
                                }
                            }
                            tx = Some(back_tx);
                            if lane == Lane::Priority {
                                shared.add(|c| &c.priority_jobs, 1);
                                holds_priority = true;
                            }
                            members.push(member);
                        }
                        // The grant was lost without being returned (its drop
                        // always returns it, so only a panic inside drop): the
                        // transaction is gone and rolled back.
                        Err(_) => {
                            broken = true;
                            members.push(member);
                            break;
                        }
                    }
                }
            }
        }
        if members.len() >= shared.limits.max_jobs || begun.elapsed() >= shared.limits.max_time {
            break;
        }
        next = turns.pick(priority, normal, holds_priority);
    }

    let committed = match (broken, tx) {
        // One COMMIT per batch.
        (false, Some(tx)) => {
            let commit_at = Instant::now();
            // A failed COMMIT leaves SQLite's state unknown to SQLx (G5: no
            // transaction at all): the connection is then closed, not pooled.
            let result = tx.commit_or_close().await;
            shared.add(|c| &c.commit_micros_total, micros(commit_at.elapsed()));
            if let Err(error) = &result
                && last_done
                && left_transaction(error)
            {
                note_left(shared);
                left = members.pop();
            }
            result.is_ok()
        }
        // A broken batch: dropping the transaction rolls it back; the
        // connection is closed, not pooled (its state is unknown).
        (_, mut tx) => {
            if let Some(tx) = tx.as_mut() {
                tx.close_on_drop();
            }
            drop(tx);
            false
        }
    };
    let elapsed = micros(begun.elapsed());
    shared.add(|c| &c.batch_micros_total, elapsed);
    shared.max(|c| &c.batch_micros_max, elapsed);
    if let Some(member) = left {
        let _ = member.done.send(Outcome::LeftTransaction);
    }
    if committed {
        shared.add(|c| &c.commits, 1);
        shared.add(|c| &c.jobs, members.len() as u64);
        shared.max(|c| &c.batch_jobs_max, members.len() as u64);
        for member in members {
            let _ = member.done.send(Outcome::Committed);
        }
    } else {
        shared.add(|c| &c.batch_failures, 1);
        for member in members {
            let _ = member.done.send(Outcome::Rerun);
        }
    }
}

#[cfg(test)]
#[path = "sqlite_writer_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "sqlite_writer_benchmark.rs"]
mod benchmark;
