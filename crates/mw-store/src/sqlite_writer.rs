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
//! - The batch ends when no ticket is waiting, after [`MAX_BATCH_JOBS`] jobs,
//!   once [`MAX_BATCH_TIME`] has passed since `BEGIN`, or right after a
//!   [`Lane::Priority`] job. Then ONE `COMMIT`, and only after it succeeded is
//!   every caller of the batch told so: a caller sees `Ok` only for a committed
//!   job. There is no waiting window: a lone job commits at once, batches form
//!   only while jobs queue behind a running one.
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
//! - Bounded: [`QUEUE_JOBS`] normal and [`QUEUE_PRIORITY_JOBS`] priority
//!   tickets. A caller waits at most [`WAIT_LIMIT`] (5 s, the foreground
//!   `busy_timeout`) for its turn, then gets the same `database is locked`
//!   error a locked database gives.
//! - Priority lane delay: a priority ticket is granted when the job that holds
//!   the transaction now hands it back (priority tickets are taken first), and
//!   its batch commits right after it. Its delay is therefore one running job
//!   plus one commit, independent of how many normal tickets are queued. A
//!   writer that does not use the writer (`begin()`, autocommit writes) waits in
//!   the write gate for at most one batch: [`MAX_BATCH_TIME`] plus the job
//!   running at that moment plus the commit.
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

const SAVEPOINT: &str = "SAVEPOINT mw_job";
const ROLLBACK_TO: &str = "ROLLBACK TO mw_job";

/// Which queue a write job waits in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lane {
    /// Sign-in, revocation, audit, throttle transitions (A2 tier T0): served
    /// before every normal ticket, and its batch commits right after it.
    Priority,
    /// Acknowledged-after-commit writes (T1) and background flushes (T3).
    Normal,
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
    /// Longest wait of a priority ticket for its grant.
    pub priority_wait_micros_max: u64,
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
    priority_wait_micros_max: AtomicU64,
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
            priority_wait_micros_max: AtomicU64::new(0),
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
            priority_wait_micros_max: load(&self.priority_wait_micros_max),
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

struct Ticket {
    grant: oneshot::Sender<Result<Grant, sqlx::Error>>,
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
    Solo(Option<Tx>),
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
        Self(LeaseKind::Solo(Some(tx)))
    }
    pub(crate) fn tx(&mut self) -> &mut Tx {
        match &mut self.0 {
            LeaseKind::Solo(tx) => tx.as_mut().expect("lease holds its transaction"),
            LeaseKind::Grouped(grant) => grant.tx.as_mut().expect("lease holds its transaction"),
        }
    }
    /// End the job with its result. `Ok` is returned only after the commit
    /// that contains the job; an `Err` result rolls the job back.
    pub(crate) async fn finish<T, E: From<sqlx::Error>>(mut self, result: Result<T, E>) -> Finished<T, E> {
        match &mut self.0 {
            LeaseKind::Solo(tx) => {
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
    pub(crate) async fn lease(&self, live: &LiveSqlite, lane: Lane) -> Result<WriteLease, sqlx::Error> {
        if self.shared.closing.load(Ordering::Acquire) {
            return Err(sqlx::Error::PoolClosed);
        }
        let (grant, granted) = oneshot::channel();
        let ticket = Ticket {
            grant,
            lane,
            queued_at: Instant::now(),
            flight: InFlight::enter(&self.shared),
        };
        let waited = tokio::time::timeout(self.shared.limits.wait_limit, async {
            self.submit(live, ticket).await?;
            granted.await.map_err(|_| writer_stopped())?
        })
        .await;
        match waited {
            Ok(Ok(grant)) => Ok(WriteLease(LeaseKind::Grouped(grant))),
            Ok(Err(error)) => Err(error),
            Err(_) => {
                self.shared.add(|c| &c.refusals, 1);
                Err(super::write_gate::locked())
            }
        }
    }

    async fn submit(&self, live: &LiveSqlite, mut ticket: Ticket) -> Result<(), sqlx::Error> {
        // A writer task ends with the runtime that spawned it (tests, and a
        // store reused by a later runtime); its closed queue is replaced once.
        for _ in 0..2 {
            let sender = self.sender(live, ticket.lane);
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

    fn sender(&self, live: &LiveSqlite, lane: Lane) -> mpsc::Sender<Ticket> {
        let mut channels = self.channels.lock().unwrap_or_else(|p| p.into_inner());
        if channels
            .as_ref()
            .is_none_or(|c| c.normal.is_closed() || c.priority.is_closed())
        {
            let (priority, priority_rx) = mpsc::channel(self.shared.limits.queue_priority.max(1));
            let (normal, normal_rx) = mpsc::channel(self.shared.limits.queue.max(1));
            let task = tokio::spawn(run(
                Arc::downgrade(&live.inner),
                self.shared.clone(),
                priority_rx,
                normal_rx,
            ))
            .abort_handle();
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
    loop {
        let first = tokio::select! {
            biased;
            ticket = priority.recv() => ticket,
            ticket = normal.recv() => ticket,
        };
        let Some(first) = first else { break };
        let Some(inner) = weak.upgrade() else {
            let _ = first.grant.send(Err(sqlx::Error::PoolClosed));
            break;
        };
        let live = LiveSqlite { inner };
        batch(&live, &shared, first, &mut priority, &mut normal).await;
    }
}

fn next_ticket(priority: &mut mpsc::Receiver<Ticket>, normal: &mut mpsc::Receiver<Ticket>) -> Option<Ticket> {
    priority.try_recv().ok().or_else(|| normal.try_recv().ok())
}

async fn execute(tx: &mut Tx, sql: &'static str) -> Result<(), sqlx::Error> {
    match tx {
        Tx::Sqlite(live) => sqlx::query(sql).execute(&mut **live).await.map(|_| ()),
        Tx::Postgres(_) => Err(sqlx::Error::Protocol("group writer is SQLite only".into())),
    }
}

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
) {
    // Skip callers that left before BEGIN: an empty batch is not opened.
    let mut next = Some(first);
    while let Some(ticket) = next.take() {
        if !ticket.grant.is_closed() {
            next = Some(ticket);
            break;
        }
        shared.add(|c| &c.skipped, 1);
        next = next_ticket(priority, normal);
    }
    let Some(first) = next else { return };

    let begun = Instant::now();
    let mut tx: Option<Tx> = match live.begin_at(here()).await {
        Ok(tx) => Some(Tx::Sqlite(tx)),
        Err(error) => {
            // Gate wait limit, closed pool: this caller alone hears it; the
            // queued tickets get their own attempt in the next round.
            let _ = first.grant.send(Err(error));
            return;
        }
    };
    let mut members: Vec<Member> = Vec::new();
    let mut broken = false;
    let mut next = Some(first);
    while let Some(ticket) = next.take() {
        let Ticket {
            grant,
            lane,
            queued_at,
            flight,
        } = ticket;
        if grant.is_closed() {
            shared.add(|c| &c.skipped, 1);
        } else {
            let mut current = tx.take().expect("batch transaction present");
            if execute(&mut current, SAVEPOINT).await.is_err() {
                // SQLite ended the transaction: nothing of this batch is kept.
                broken = true;
                let _ = grant.send(Err(writer_stopped()));
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
                            shared.max(|c| &c.job_hold_micros_max, micros(granted_at.elapsed()));
                            let mut back_tx = back_tx;
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
                                if execute(&mut back_tx, ROLLBACK_TO).await.is_err() {
                                    broken = true;
                                    members.push(member);
                                    drop(back_tx);
                                    break;
                                }
                            }
                            tx = Some(back_tx);
                            if lane == Lane::Priority {
                                shared.add(|c| &c.priority_jobs, 1);
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
            if lane == Lane::Priority {
                break;
            }
        }
        if members.len() >= shared.limits.max_jobs || begun.elapsed() >= shared.limits.max_time {
            break;
        }
        next = next_ticket(priority, normal);
    }

    let committed = match (broken, tx) {
        (false, Some(tx)) => tx.commit().await.is_ok(),
        // A broken batch: dropping the transaction rolls it back.
        (_, tx) => {
            drop(tx);
            false
        }
    };
    let elapsed = micros(begun.elapsed());
    shared.add(|c| &c.batch_micros_total, elapsed);
    shared.max(|c| &c.batch_micros_max, elapsed);
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
