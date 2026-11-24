//! Bounded-staleness mirror of authority epoch changes (t34, `MW_GUARD_SNAPSHOT=bounded`).
//!
//! The mirror lets a request reuse an earlier [`NativeAuthorityFence`] instead of
//! running the fence SELECT again, for at most a configured staleness bound. It is
//! DATA freshness bookkeeping, not an authentication result: callers still run every
//! clock, policy and process check on the reused fence.
//!
//! # Source of changes
//!
//! Every authority source trigger from 0111, 0112 and 0114 writes
//! `authority_epoch`, and two change-log triggers append one
//! `authority_change_log` row, in the writer's own transaction, for every INSERT,
//! UPDATE or DELETE of an `authority_epoch` row and of the `settings_revision` row
//! (SQLite: migration 0115; PostgreSQL: 0120, after 0118 removed the 0115 ones).
//! So every write that the existing epoch fence would notice also appends a log
//! row, including raw SQL writes that bypass the Store API.
//! A row records `(scope_kind, incarnation)`. An epoch row keeps one incarnation for
//! its whole life (0111 `authority_epoch_monotonic`), so this names exactly the
//! scope a cached fence observed, without copying account ids, subjects or session
//! ids into the log. On PostgreSQL a row also records `xid`, the top-level
//! transaction id of its writer (`pg_current_xact_id()`, which returns the
//! top-level id inside subtransactions too).
//!
//! # Poll
//!
//! A poll reads in ONE snapshot. It returns the log rows the previous poll had
//! not seen, and the publication schema stamp (`native_publication_schema.rs`).
//! The poll applies them: every row's `(kind, incarnation)` gets `changed_gen = N`,
//! where `N` is the number of this poll. Polls run every `bound / 4` (at least
//! 25 ms) on a background task, and at once after [`EpochMirror::expedite`].
//!
//! * SQLite: one SQL statement on a pooled connection, so one implicit read
//!   transaction (one WAL snapshot). It pages by `seq` after the cursor and also
//!   returns the newest `seq` and whether the cursor row still exists.
//! * PostgreSQL: `BEGIN ISOLATION LEVEL REPEATABLE READ READ ONLY`, one SELECT,
//!   `COMMIT`, on a dedicated connection outside the Store pool (so pool load
//!   cannot delay it; it is reopened after any error). The SELECT returns
//!   `pg_current_snapshot()` (call it `S`) and the rows the previous poll's
//!   snapshot `P` did not show: `xid >= pg_snapshot_xmax(P)` or `xid` in
//!   `pg_snapshot_xip(P)`. In REPEATABLE READ every statement of the transaction
//!   reads the transaction snapshot, and `pg_current_snapshot()` returns the
//!   active snapshot, which is that one (test
//!   `pg_poll_snapshot_is_the_transaction_snapshot`). Each returned row is also
//!   checked with `pg_visible_in_snapshot(xid, S)`; a row that fails the check
//!   fails the poll. `S` becomes `P` for the next poll.
//!
//! All cached observations go cold (`evicted_floor_gen = N + 1`), and the cursor
//! moves to the head of this poll's snapshot instead of paging, when: this is the
//! first poll or the SQLite generation changed; the page is full (4096 rows: the
//! poll did not reach the head); or the schema stamp changed.
//! SQLite only: the cursor is older than the retained horizon
//! (`max(seq) - 65536 > cursor`: the 0115 prune trigger may have deleted unread
//! rows); the cursor row disappeared although it is inside the horizon (rows
//! deleted outside the contract); `max(seq)` went below the cursor.
//! PostgreSQL only: more than `bound` passed between the start of the previous
//! complete poll and the end of this one (the gap rule, see Pruning); the
//! snapshot xmin went below the previous one. More than 65,536 tracked scopes
//! also flush everything except the global kinds.
//!
//! # Staleness bound
//!
//! A poll observes every change committed before it started. The mirror is fresh
//! while `Instant::now() < last_complete_poll_started + bound`, where a complete
//! poll succeeded and returned a schema stamp (it either applied every row up to
//! the head of its snapshot or flushed everything). So a
//! change committed at time C fails [`EpochMirror::still_valid`] for every check
//! made at or after C + bound, in every process that shares the database: either
//! a poll that started after C has applied it, or no complete poll started after C
//! and the mirror is no longer fresh. When polls fail or fall behind, nothing is
//! fresh, [`EpochMirror::stamp_gen`] returns `None` and callers take the existing
//! fresh SQL path. Freshness uses only this process's monotonic clock. The bound
//! only holds for changes that fire the authority triggers; a writer that
//! disables triggers is outside the contract (disabling or dropping a trigger on
//! a stamped table does change the schema stamp, so the next poll flushes).
//!
//! # Correctness of `still_valid(fence, G)`
//!
//! The caller takes `G = stamp_gen()` BEFORE it starts the fence statement, and
//! stores `G` with the fence. Poll G completed before the fence statement began, so
//! the fence's snapshot contains every commit that poll G's snapshot contained.
//! Take a change X that the fence does not reflect. Then poll G's snapshot does
//! not contain X either. It remains to show that a poll numbered G+1 or later
//! applies X's log row (or flushes everything), which sets `changed_gen > G` for
//! X's scope, so `still_valid` returns false. A change applied by poll G or
//! earlier was in poll G's snapshot, so the fence reflects it.
//!
//! * SQLite admits one writer at a time and `seq` is AUTOINCREMENT, so X's log
//!   row has a larger `seq` than every row poll G could see; the next poll that
//!   sees it pages past the cursor and applies it (or flushes).
//! * PostgreSQL: let poll K be the first poll after G whose snapshot shows X.
//!   Poll K-1 (number >= G) did not show X, so at its snapshot `P` X was either
//!   running, which puts X's top-level xid in `pg_snapshot_xip(P)`, or had no xid
//!   yet, which makes its xid at least `pg_snapshot_xmax(P)` (xids are assigned in
//!   increasing order, and every xid below `xmax` that is not in `xip` had
//!   finished when `P` was taken). Either way poll K selects X's row and applies
//!   it, or its page is full and it flushes. Sequence order plays no part.
//!
//! This argument depends on two facts:
//! 1. **Single read snapshot.** The fence and the poll must each read in ONE
//!    snapshot that begins after the call that precedes it. The fence is one SQL
//!    statement issued on a pooled connection in autocommit mode
//!    ([`Store::native_publication_fence`] and its siblings issue exactly one
//!    `fetch_all`; the 0111 contract test `primary_hint_revision_contract` asserts
//!    one statement), which on PostgreSQL takes its snapshot when the statement
//!    starts, in every isolation level. If the fence were ever read inside a
//!    transaction opened before `stamp_gen()`, the fence could be older than poll
//!    G and the argument would fail. The PostgreSQL poll's row scan and `S` are
//!    the same snapshot (REPEATABLE READ, above).
//! 2. **Snapshots only grow.** A transaction visible in one snapshot is visible
//!    in every later snapshot of the same database (SQLite WAL; one PostgreSQL
//!    primary). The PostgreSQL poll fails on a server in recovery.
//!
//! # Pruning (PostgreSQL)
//!
//! Writers never delete log rows. After a successful poll the poller records a
//! mark `(now, xmin(S))` at most every 30 s; once a mark is 60 s old it deletes up
//! to 5000 rows with `xid < mark` in one autocommit statement that first takes
//! `pg_try_advisory_xact_lock` (one pruner per schema at a time; it never waits
//! and holds no lock a writer needs). A row with `xid < xmin(S)` belonged to a
//! transaction that had finished before the mark was recorded, so every poll
//! whose snapshot was taken after the mark saw it. A poller that did not see it
//! took its last snapshot, and started that poll, before the mark; its next
//! successful poll ends after the delete, so at least 60 s after that start, and
//! the gap rule (more than `bound`, at most 5 s) flushes it. So no poll that
//! applies rows instead of flushing ever needed a deleted row.
//!
//! # Not covered by any trigger (callers must not cache these)
//!
//! * `sessions.created_at`, `activity_at` and `last_seen`: the 0111 sessions
//!   trigger deliberately ignores clocks. A reused fence carries the clocks it read.
//!   They may only feed expiry and touch-due checks that fall back to the fresh
//!   path when a check would trigger; see the coverage list in
//!   `docs/perf/guard-snapshot.md` (t34 e8) and `.orchestration/logs/t34-e1.md`.
//! * Mail DATA (`native_publication_fence_with_mail_data`): stays fresh in t34.
//! * `native_sessions`, admin tables, `request_protection_subject`,
//!   `sess-sec:v1:`/`sess-rot:v1:` watch state as read by `session_watch`, and
//!   `v6:admin:bans` as read by `source_bans` today: read fresh, outside the fence.
//! * TRUNCATE fires no row trigger. Every table the fence or the poll depends on
//!   is in the schema stamp's table list, whose PostgreSQL form includes each
//!   table's `relfilenode`; TRUNCATE assigns a new one, so the stamp changes and
//!   the next poll flushes.
use super::*;
use std::{
    collections::{HashMap, VecDeque},
    sync::{
        Arc, Mutex, Weak,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

/// Smallest accepted staleness bound.
pub const MIN_STALENESS: Duration = Duration::from_millis(100);
/// Largest accepted staleness bound.
pub const MAX_STALENESS: Duration = Duration::from_millis(5000);
/// Default staleness bound (user decision, t34).
pub const DEFAULT_STALENESS: Duration = Duration::from_millis(1000);
/// Must equal the horizon in the 0115 `authority_change_log_prune` trigger.
const RETAINED_SEQ: i64 = 65_536;
const PAGE: i64 = 4096;
const MAX_TRACKED: usize = 65_536;
const MAX_STORES: usize = 32;
const MIN_INTERVAL: Duration = Duration::from_millis(25);
const SETTINGS_REVISION: &str = "settings-revision";
/// PostgreSQL pruning: a mark at most this often, deleted below once this old.
const PRUNE_MARK_EVERY: Duration = Duration::from_secs(30);
const PRUNE_AFTER: Duration = Duration::from_secs(60);
const PRUNE_BATCH: i64 = 5000;
const MAX_PRUNE_MARKS: usize = 16;
/// Longest accepted `pg_snapshot` text (xmin:xmax:xip list).
const MAX_SNAPSHOT_TEXT: usize = 1 << 20;

/// The poll generation a caller recorded BEFORE it issued a fresh fence read.
/// It is bound to one mirror instance, so a value from another Store or from a
/// mirror that was dropped and recreated never validates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MirrorGen {
    instance: u64,
    generation: u64,
}

/// Monotonic counters for fixtures and diagnostics.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MirrorCounters {
    pub polls: u64,
    pub failures: u64,
    pub rows_applied: u64,
    pub overflow_flushes: u64,
    pub horizon_flushes: u64,
    pub schema_flushes: u64,
    pub generation_flushes: u64,
    pub capacity_flushes: u64,
    /// PostgreSQL: more than the bound between two complete polls.
    pub gap_flushes: u64,
    pub expedites: u64,
    pub hits: u64,
    pub misses: u64,
    /// PostgreSQL: dedicated poll connections opened.
    pub connects: u64,
    /// PostgreSQL: log rows deleted by this process's pruner.
    pub pruned_rows: u64,
    pub prune_failures: u64,
}

#[derive(Default)]
struct Counters {
    polls: AtomicU64,
    failures: AtomicU64,
    rows_applied: AtomicU64,
    overflow_flushes: AtomicU64,
    horizon_flushes: AtomicU64,
    schema_flushes: AtomicU64,
    generation_flushes: AtomicU64,
    capacity_flushes: AtomicU64,
    gap_flushes: AtomicU64,
    expedites: AtomicU64,
    hits: AtomicU64,
    misses: AtomicU64,
    connects: AtomicU64,
    pruned_rows: AtomicU64,
    prune_failures: AtomicU64,
}
fn bump(counter: &AtomicU64) {
    counter.fetch_add(1, Ordering::Relaxed);
}

#[derive(Default)]
struct State {
    /// `None` until the first successful poll, then the generation it read.
    generation: Option<Option<PathBuf>>,
    /// Number of the last applied poll.
    poll_gen: u64,
    cursor: i64,
    last_complete_started: Option<Instant>,
    expedite_cleared: u64,
    evicted_floor_gen: u64,
    schema: Option<String>,
    revision_gen: u64,
    globals: HashMap<&'static str, u64>,
    changed: HashMap<(String, String), u64>,
    /// PostgreSQL: the snapshot of the last applied poll, and its xmin.
    pg_snapshot: Option<(String, u64)>,
}

/// Held for the whole poll, so polls never overlap.
struct PollSlot {
    /// PostgreSQL: the dedicated poll connection, outside the Store pool.
    conn: Option<sqlx::PgConnection>,
    /// PostgreSQL pruning marks `(recorded, snapshot xmin)`, oldest first.
    marks: VecDeque<(Instant, u64)>,
    mark_every: Duration,
    prune_after: Duration,
}

/// See the module documentation.
pub struct EpochMirror {
    instance: u64,
    store_id: uuid::Uuid,
    bound: Duration,
    interval: Duration,
    /// Held by live mirrors only. `None` means inert: never fresh.
    store: Option<Store>,
    state: Mutex<State>,
    poll_lock: tokio::sync::Mutex<PollSlot>,
    wake: Arc<tokio::sync::Notify>,
    expedite_requested: AtomicU64,
    counters: Counters,
}
impl fmt::Debug for EpochMirror {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EpochMirror")
            .field("bound", &self.bound)
            .field("inert", &self.store.is_none())
            .finish_non_exhaustive()
    }
}

/// Whether `bound` is inside the accepted range.
pub fn valid_staleness(bound: Duration) -> bool {
    (MIN_STALENESS..=MAX_STALENESS).contains(&bound)
}

fn poll_sql() -> &'static str {
    static SQL: OnceLock<String> = OnceLock::new();
    SQL.get_or_init(|| {
        let stamp = super::schema::stamp_expression(Dialect::Sqlite);
        format!(
            "SELECT 'row' AS tag,l.seq AS seq,l.scope_kind AS scope_kind,l.incarnation AS incarnation,CAST(NULL AS BIGINT) AS head,CAST(NULL AS BIGINT) AS present,CAST(NULL AS TEXT) AS publication_schema FROM (SELECT seq,scope_kind,incarnation FROM authority_change_log WHERE seq>?1 ORDER BY seq LIMIT ?2) l UNION ALL SELECT 'meta',CAST(NULL AS BIGINT),CAST(NULL AS TEXT),CAST(NULL AS TEXT),(SELECT COALESCE(MAX(seq),0) FROM authority_change_log),(SELECT COUNT(*) FROM authority_change_log WHERE seq=?1),{stamp}"
        )
    })
}

fn global_kind(kind: &str) -> Option<&'static str> {
    [
        AuthorityKind::Policy,
        AuthorityKind::Routing,
        AuthorityKind::Directory,
        AuthorityKind::Protection,
    ]
    .into_iter()
    .map(AuthorityKind::as_str)
    .find(|name| *name == kind)
}

fn next_instance() -> u64 {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    NEXT.fetch_add(1, Ordering::Relaxed)
}

impl EpochMirror {
    fn new(store: &Store, bound: Duration, live: bool) -> Self {
        Self {
            instance: next_instance(),
            store_id: store.cache_scope(),
            bound,
            interval: (bound / 4).max(MIN_INTERVAL),
            store: live.then(|| store.clone()),
            state: Mutex::new(State::default()),
            poll_lock: tokio::sync::Mutex::new(PollSlot {
                conn: None,
                marks: VecDeque::new(),
                mark_every: PRUNE_MARK_EVERY,
                prune_after: PRUNE_AFTER,
            }),
            wake: Arc::new(tokio::sync::Notify::new()),
            expedite_requested: AtomicU64::new(0),
            counters: Counters::default(),
        }
    }

    /// The process mirror for this Store, created and started on first use.
    ///
    /// The registry holds only a weak handle, keyed by `store.cache_scope()`, for
    /// at most 32 Stores. The CALLER must keep the returned `Arc` (for example on
    /// the native publication worker that AppState owns); when the last `Arc` is
    /// dropped the poll task exits within one interval and the next call starts a
    /// cold mirror. An existing mirror is returned as is, even if `bound` differs:
    /// the bound is process-immutable. The returned mirror is inert (never fresh)
    /// outside a Tokio runtime, when `bound` is outside
    /// [`MIN_STALENESS`]..=[`MAX_STALENESS`], or when the registry is full. On
    /// PostgreSQL a live mirror opens one dedicated connection (with the Store
    /// pool's connect options) on its first poll and keeps it while it lives.
    pub fn for_store(store: &Store, bound: Duration) -> Arc<Self> {
        if !valid_staleness(bound) || tokio::runtime::Handle::try_current().is_err() {
            return Arc::new(Self::new(store, bound, false));
        }
        static REGISTRY: OnceLock<Mutex<HashMap<uuid::Uuid, Weak<EpochMirror>>>> = OnceLock::new();
        let Ok(mut registry) = REGISTRY.get_or_init(|| Mutex::new(HashMap::new())).lock() else {
            return Arc::new(Self::new(store, bound, false));
        };
        registry.retain(|_, mirror| mirror.strong_count() > 0);
        if let Some(mirror) = registry.get(&store.cache_scope()).and_then(Weak::upgrade) {
            return mirror;
        }
        if registry.len() >= MAX_STORES {
            return Arc::new(Self::new(store, bound, false));
        }
        let mirror = Arc::new(Self::new(store, bound, true));
        registry.insert(store.cache_scope(), Arc::downgrade(&mirror));
        drop(registry);
        let weak = Arc::downgrade(&mirror);
        let wake = mirror.wake.clone();
        let interval = mirror.interval;
        tokio::spawn(async move {
            loop {
                let Some(mirror) = weak.upgrade() else { break };
                let flushed = mirror.poll_now().await.unwrap_or(false);
                drop(mirror);
                if !flushed {
                    tokio::select! {
                        _ = tokio::time::sleep(interval) => {}
                        _ = wake.notified() => {}
                    }
                }
            }
        });
        mirror
    }

    /// A live mirror without a background task, for deterministic tests.
    #[cfg(test)]
    pub(crate) fn manual(store: &Store, bound: Duration) -> Arc<Self> {
        Arc::new(Self::new(store, bound, true))
    }

    /// Test hook: pruning timing (production: 30 s marks, 60 s retention).
    #[cfg(test)]
    pub(crate) async fn set_prune_timing(&self, mark_every: Duration, prune_after: Duration) {
        let mut slot = self.poll_lock.lock().await;
        slot.mark_every = mark_every;
        slot.prune_after = prune_after;
    }

    /// Test hook: backend pid of the dedicated PostgreSQL poll connection.
    #[cfg(test)]
    pub(crate) async fn poll_backend_pid(&self) -> Option<i32> {
        let mut slot = self.poll_lock.lock().await;
        let conn = slot.conn.as_mut()?;
        sqlx::query_scalar("SELECT pg_catalog.pg_backend_pid()")
            .fetch_one(conn)
            .await
            .ok()
    }

    pub fn bound(&self) -> Duration {
        self.bound
    }

    /// False for an inert mirror (no runtime, invalid bound, full registry).
    pub fn is_live(&self) -> bool {
        self.store.is_some()
    }

    fn fresh(&self, state: &State, now: Instant) -> bool {
        self.store.is_some()
            && state.schema.is_some()
            && state.generation.is_some()
            && state
                .last_complete_started
                .and_then(|started| started.checked_add(self.bound))
                .is_some_and(|until| now < until)
    }

    /// Whether a complete poll started less than `bound` ago.
    pub fn is_fresh(&self) -> bool {
        self.state
            .lock()
            .is_ok_and(|state| self.fresh(&state, Instant::now()))
    }

    /// Call strictly BEFORE issuing the fresh fence statement and keep the result
    /// with that fence. `None` means the mirror is not fresh: do not cache.
    pub fn stamp_gen(&self) -> Option<MirrorGen> {
        let state = self.state.lock().ok()?;
        // After a flush nothing stamped at the current generation can validate.
        (self.fresh(&state, Instant::now()) && state.evicted_floor_gen <= state.poll_gen).then_some(
            MirrorGen {
                instance: self.instance,
                generation: state.poll_gen,
            },
        )
    }

    fn valid_locked(&self, state: &State, built: MirrorGen, now: Instant) -> bool {
        built.instance == self.instance
            && self.fresh(state, now)
            && self.expedite_requested.load(Ordering::SeqCst) <= state.expedite_cleared
            && built.generation <= state.poll_gen
            && state.evicted_floor_gen <= built.generation
            && state.revision_gen <= built.generation
            && state
                .globals
                .values()
                .all(|changed| *changed <= built.generation)
    }

    /// True iff `fence`, read after `built` was stamped, still reflects every
    /// authority change that this mirror has applied, the mirror is fresh, no
    /// expedite is pending, and the Store, SQLite generation and schema stamp are
    /// the ones the mirror polls. Counts a hit or a miss.
    pub fn still_valid(&self, fence: &NativeAuthorityFence, built: MirrorGen) -> bool {
        let valid = self.state.lock().is_ok_and(|state| {
            let versions = fence.versions();
            self.valid_locked(&state, built, Instant::now())
                && versions.store == self.store_id
                && state.generation.as_ref() == Some(&versions.generation)
                && fence
                    .publication_schema()
                    .is_none_or(|schema| state.schema.as_deref() == Some(schema))
                && versions.stamps.values().all(|stamp| {
                    state
                        .changed
                        .get(&(
                            stamp.scope.kind.as_str().to_owned(),
                            stamp.incarnation.clone(),
                        ))
                        .is_none_or(|changed| *changed <= built.generation)
                })
        });
        bump(if valid {
            &self.counters.hits
        } else {
            &self.counters.misses
        });
        valid
    }

    /// For data keyed only by the global epochs and `settings_revision` (for
    /// example the settings revision or parsed source bans): true iff the mirror
    /// is fresh, no expedite is pending and no global epoch or settings revision
    /// change was applied after `built`. Does not count hits or misses.
    pub fn globals_unchanged(&self, built: MirrorGen) -> bool {
        self.state
            .lock()
            .is_ok_and(|state| self.valid_locked(&state, built, Instant::now()))
    }

    /// Make every check fail until a poll that starts after this call completes,
    /// and start that poll now. Call after committing a local revocation.
    pub fn expedite(&self) {
        self.expedite_requested.fetch_add(1, Ordering::SeqCst);
        bump(&self.counters.expedites);
        self.wake.notify_one();
    }

    pub fn counters(&self) -> MirrorCounters {
        let c = &self.counters;
        let get = |counter: &AtomicU64| counter.load(Ordering::Relaxed);
        MirrorCounters {
            polls: get(&c.polls),
            failures: get(&c.failures),
            rows_applied: get(&c.rows_applied),
            overflow_flushes: get(&c.overflow_flushes),
            horizon_flushes: get(&c.horizon_flushes),
            schema_flushes: get(&c.schema_flushes),
            generation_flushes: get(&c.generation_flushes),
            capacity_flushes: get(&c.capacity_flushes),
            gap_flushes: get(&c.gap_flushes),
            expedites: get(&c.expedites),
            hits: get(&c.hits),
            misses: get(&c.misses),
            connects: get(&c.connects),
            pruned_rows: get(&c.pruned_rows),
            prune_failures: get(&c.prune_failures),
        }
    }

    /// Run one poll now (the background task calls this too; polls never
    /// overlap). Returns `Ok(true)` when this poll flushed every cached
    /// observation, so nothing stamps until the next poll (the background task
    /// then polls again at once). An inert mirror returns `Ok(false)` without SQL.
    pub async fn poll_now(&self) -> Result<bool, StoreError> {
        let Some(store) = &self.store else {
            return Ok(false);
        };
        let mut slot = self.poll_lock.lock().await;
        let result = match &store.backend {
            Backend::Sqlite(_) => self.poll_sqlite(store).await,
            Backend::Postgres(pool) => self.poll_postgres(pool, &mut slot).await,
        };
        bump(&self.counters.polls);
        if result.is_err() {
            bump(&self.counters.failures);
        }
        result
    }

    async fn poll_sqlite(&self, store: &Store) -> Result<bool, StoreError> {
        let expedite_seen = self.expedite_requested.load(Ordering::SeqCst);
        let started = Instant::now();
        let cursor = self
            .state
            .lock()
            .map_err(|_| corrupt("authority mirror state poisoned"))?
            .cursor;
        let generation = store.authority_generation().await;
        let rows = q(poll_sql())
            .bind(cursor)
            .bind(PAGE)
            .fetch_all(&store.backend)
            .await?;
        if generation != store.authority_generation().await {
            return Err(StoreError::SourceChanged);
        }
        if rows.len() as i64 > PAGE + 1 {
            return Err(corrupt("authority change log page bound exceeded"));
        }
        let mut meta = None;
        let mut page = Vec::with_capacity(rows.len());
        for row in &rows {
            match text(row, "tag")?.as_str() {
                "row" => {
                    let seq = number(row, "seq")?
                        .ok_or_else(|| corrupt("authority change log seq is missing"))?;
                    page.push((seq, text(row, "scope_kind")?, text(row, "incarnation")?));
                }
                "meta" => {
                    let head = number(row, "head")?
                        .ok_or_else(|| corrupt("authority change log head is missing"))?;
                    let present = number(row, "present")?.unwrap_or(0) > 0;
                    let schema = row
                        .try_get_opt_string("publication_schema")?
                        .filter(|stamp| stamp.len() <= 128);
                    if meta.replace((head, present, schema)).is_some() {
                        return Err(corrupt("duplicate authority change log head"));
                    }
                }
                _ => return Err(corrupt("unknown authority change log section")),
            }
        }
        let (head, present, schema) =
            meta.ok_or_else(|| corrupt("authority change log head is missing"))?;
        page.sort_unstable_by_key(|(seq, _, _)| *seq);
        if page.windows(2).any(|pair| pair[0].0 == pair[1].0)
            || page.first().is_some_and(|(seq, _, _)| *seq <= cursor)
        {
            return Err(corrupt("authority change log page out of order"));
        }
        let truncated = page.len() as i64 == PAGE;
        let mut state = self
            .state
            .lock()
            .map_err(|_| corrupt("authority mirror state poisoned"))?;
        if state.cursor != cursor {
            return Err(corrupt(
                "authority mirror cursor moved during a serialized poll",
            ));
        }
        let number = state.poll_gen + 1;
        let mut flush = false;
        if state.generation.as_ref() != Some(&generation) {
            // First poll or a new SQLite generation: start at the head. Nothing
            // cached before this poll can validate (generation or instance differ).
            if state.generation.is_some() {
                bump(&self.counters.generation_flushes);
            }
            state.generation = Some(generation);
            state.changed.clear();
            state.globals.clear();
            state.cursor = head;
            flush = true;
        } else {
            if head < cursor
                || head - RETAINED_SEQ > cursor
                || (cursor > 0 && !present && cursor > head - RETAINED_SEQ)
            {
                bump(&self.counters.horizon_flushes);
                flush = true;
            }
            if truncated {
                bump(&self.counters.overflow_flushes);
                flush = true;
            }
            if schema != state.schema {
                bump(&self.counters.schema_flushes);
                flush = true;
            }
        }
        if flush {
            // Every fence stamped at or before this poll goes cold, and a fence
            // stamped after this poll was read after this snapshot, so it already
            // reflects every row up to `head`: skip them instead of paging.
            state.cursor = head;
        } else {
            if let Some((seq, _, _)) = page.last() {
                state.cursor = *seq;
            }
            self.apply(
                &mut state,
                page.into_iter()
                    .map(|(_, kind, incarnation)| (kind, incarnation)),
                number,
            );
        }
        self.finish(&mut state, number, flush, schema, started, expedite_seen);
        Ok(flush)
    }

    /// Gives every row's scope `changed_gen = number`.
    fn apply(&self, state: &mut State, rows: impl Iterator<Item = (String, String)>, number: u64) {
        let mut applied = 0u64;
        for (kind, incarnation) in rows {
            if kind == SETTINGS_REVISION {
                state.revision_gen = number;
            } else if let Some(global) = global_kind(&kind) {
                state.globals.insert(global, number);
            }
            state.changed.insert((kind, incarnation), number);
            applied += 1;
        }
        self.counters
            .rows_applied
            .fetch_add(applied, Ordering::Relaxed);
        if state.changed.len() > MAX_TRACKED {
            // Entries carry generations <= number; dropping them is safe only
            // if every fence stamped before `number` goes cold.
            state
                .changed
                .retain(|(kind, _), _| global_kind(kind).is_some());
            state.evicted_floor_gen = state.evicted_floor_gen.max(number);
            bump(&self.counters.capacity_flushes);
        }
    }

    fn finish(
        &self,
        state: &mut State,
        number: u64,
        flush: bool,
        schema: Option<String>,
        started: Instant,
        expedite_seen: u64,
    ) {
        if flush {
            state.evicted_floor_gen = number + 1;
        }
        state.poll_gen = number;
        state.schema = schema;
        // Either every row up to the head of this poll's snapshot was applied,
        // or this poll flushed: both account for every change committed before
        // `started`.
        if state.schema.is_some() {
            state.last_complete_started = Some(started);
            state.expedite_cleared = state.expedite_cleared.max(expedite_seen);
        }
    }

    fn poll_timeout(&self) -> Duration {
        (self.bound * 4).max(Duration::from_secs(2))
    }

    async fn connect_postgres(
        &self,
        pool: &sqlx::PgPool,
    ) -> Result<sqlx::PgConnection, StoreError> {
        use sqlx::ConnectOptions as _;
        bump(&self.counters.connects);
        let options = (*pool.connect_options())
            .clone()
            .application_name("mailwoman-guard-mirror");
        let mut conn = timeout(self.poll_timeout(), options.connect()).await??;
        // Freshness is measured by this process's clock, not by these limits; they
        // end a stuck statement (poll or prune) instead of waiting on it.
        let limit = format!(
            "SET statement_timeout = {}",
            (self.bound * 2).max(Duration::from_secs(2)).as_millis()
        );
        let set = async {
            sqlx::query(&limit).execute(&mut conn).await?;
            sqlx::query("SET lock_timeout = 100")
                .execute(&mut conn)
                .await?;
            sqlx::query("SET idle_in_transaction_session_timeout = 10000")
                .execute(&mut conn)
                .await
        };
        timeout(self.poll_timeout(), set).await??;
        Ok(conn)
    }

    async fn poll_postgres(
        &self,
        pool: &sqlx::PgPool,
        slot: &mut PollSlot,
    ) -> Result<bool, StoreError> {
        let expedite_seen = self.expedite_requested.load(Ordering::SeqCst);
        let started = Instant::now();
        let previous = self
            .state
            .lock()
            .map_err(|_| corrupt("authority mirror state poisoned"))?
            .pg_snapshot
            .clone();
        // Taken out for the poll: an error drops it and the next poll reconnects.
        let mut conn = match slot.conn.take() {
            Some(conn) => conn,
            None => self.connect_postgres(pool).await?,
        };
        let rows = timeout(
            self.poll_timeout(),
            read_postgres(&mut conn, previous.as_ref().map(|(text, _)| text.as_str())),
        )
        .await??;
        slot.conn = Some(conn);
        let completed = Instant::now();
        if rows.len() as i64 > PAGE + 1 {
            return Err(corrupt("authority change log page bound exceeded"));
        }
        let mut meta = None;
        let mut page = Vec::with_capacity(rows.len());
        for row in rows {
            let row = Row::Postgres(row);
            match text(&row, "tag")?.as_str() {
                "row" => {
                    if number(&row, "visible")? != Some(1) {
                        return Err(corrupt(
                            "authority change log row outside the poll snapshot",
                        ));
                    }
                    page.push((text(&row, "scope_kind")?, text(&row, "incarnation")?));
                }
                "meta" => {
                    if number(&row, "recovery")? != Some(0) {
                        return Err(corrupt(
                            "authority change log poll reached a server in recovery",
                        ));
                    }
                    let snapshot = text(&row, "snapshot")?;
                    let xmin = text(&row, "xmin")?
                        .parse::<u64>()
                        .map_err(|_| corrupt("authority change log snapshot xmin"))?;
                    if snapshot.len() > MAX_SNAPSHOT_TEXT {
                        return Err(corrupt("authority change log snapshot too large"));
                    }
                    let schema = row
                        .try_get_opt_string("publication_schema")?
                        .filter(|stamp| stamp.len() <= 128);
                    if meta.replace((snapshot, xmin, schema)).is_some() {
                        return Err(corrupt("duplicate authority change log head"));
                    }
                }
                _ => return Err(corrupt("unknown authority change log section")),
            }
        }
        let (snapshot, xmin, schema) =
            meta.ok_or_else(|| corrupt("authority change log head is missing"))?;
        let truncated = page.len() as i64 == PAGE;
        let (flush, complete) = {
            let mut state = self
                .state
                .lock()
                .map_err(|_| corrupt("authority mirror state poisoned"))?;
            if state.pg_snapshot != previous {
                return Err(corrupt(
                    "authority mirror cursor moved during a serialized poll",
                ));
            }
            let number = state.poll_gen + 1;
            let mut flush = false;
            if state.generation.is_none() || previous.is_none() {
                // First poll: start at the head of this snapshot.
                state.generation = Some(None);
                state.changed.clear();
                state.globals.clear();
                flush = true;
            } else {
                if previous.as_ref().is_some_and(|(_, before)| xmin < *before) {
                    bump(&self.counters.horizon_flushes);
                    flush = true;
                }
                // The gap rule (module doc, Pruning).
                if state
                    .last_complete_started
                    .is_none_or(|last| completed.saturating_duration_since(last) > self.bound)
                {
                    bump(&self.counters.gap_flushes);
                    flush = true;
                }
                if truncated {
                    bump(&self.counters.overflow_flushes);
                    flush = true;
                }
                if schema != state.schema {
                    bump(&self.counters.schema_flushes);
                    flush = true;
                }
            }
            if !flush {
                self.apply(&mut state, page.into_iter(), number);
            }
            // After a flush every fence stamped at or before this poll goes cold, and
            // a fence stamped after it reflects every row this snapshot shows.
            state.pg_snapshot = Some((snapshot, xmin));
            let complete = schema.is_some();
            self.finish(&mut state, number, flush, schema, started, expedite_seen);
            (flush, complete)
        };
        if complete {
            self.prune(slot, xmin, completed).await;
        }
        Ok(flush)
    }

    /// Background pruning on the poll connection; never fails the poll.
    async fn prune(&self, slot: &mut PollSlot, xmin: u64, now: Instant) {
        if slot
            .marks
            .back()
            .is_none_or(|(at, _)| now.saturating_duration_since(*at) >= slot.mark_every)
        {
            slot.marks.push_back((now, xmin));
            while slot.marks.len() > MAX_PRUNE_MARKS {
                slot.marks.pop_front();
            }
        }
        let Some(&(at, below)) = slot.marks.front() else {
            return;
        };
        if now.saturating_duration_since(at) < slot.prune_after {
            return;
        }
        let Some(conn) = slot.conn.as_mut() else {
            return;
        };
        let deleted = timeout(
            self.poll_timeout(),
            sqlx::query(PRUNE_SQL)
                .bind(below.to_string())
                .bind(PRUNE_BATCH)
                .execute(conn),
        )
        .await;
        match deleted {
            Ok(Ok(done)) => {
                self.counters
                    .pruned_rows
                    .fetch_add(done.rows_affected(), Ordering::Relaxed);
                // A full batch keeps the mark for the next poll.
                if (done.rows_affected() as i64) < PRUNE_BATCH {
                    slot.marks.pop_front();
                }
            }
            Ok(Err(_)) => bump(&self.counters.prune_failures),
            Err(_) => {
                bump(&self.counters.prune_failures);
                slot.conn = None;
            }
        }
    }
}

/// Deletes at most `$2` rows below the mark, only while holding this schema's
/// prune lock. Autocommit: the transaction-level lock ends with the statement.
const PRUNE_SQL: &str = "WITH held AS MATERIALIZED (SELECT pg_catalog.pg_try_advisory_xact_lock(pg_catalog.hashtextextended('mw_authority_change_log_prune:'||pg_catalog.current_schema(),0)) AS held),doomed AS (SELECT seq FROM authority_change_log WHERE xid<CAST(CAST($1 AS TEXT) AS xid8) AND (SELECT held FROM held) LIMIT $2) DELETE FROM authority_change_log WHERE seq IN (SELECT seq FROM doomed)";

fn postgres_poll_sql() -> &'static str {
    static SQL: OnceLock<String> = OnceLock::new();
    SQL.get_or_init(|| {
        let stamp = super::schema::stamp_expression(Dialect::Postgres);
        // $1: the previous poll's snapshot (NULL on the first poll: no rows).
        format!(
            "WITH s AS MATERIALIZED (SELECT pg_catalog.pg_current_snapshot() AS snap),p AS MATERIALIZED (SELECT CAST(CAST($1 AS TEXT) AS pg_catalog.pg_snapshot) AS snap) SELECT 'row' AS tag,l.scope_kind AS scope_kind,l.incarnation AS incarnation,CAST(CASE WHEN pg_catalog.pg_visible_in_snapshot(l.xid,(SELECT snap FROM s)) THEN 1 ELSE 0 END AS BIGINT) AS visible,CAST(NULL AS TEXT) AS snapshot,CAST(NULL AS TEXT) AS xmin,CAST(NULL AS BIGINT) AS recovery,CAST(NULL AS TEXT) AS publication_schema FROM (SELECT seq,xid,scope_kind,incarnation FROM authority_change_log WHERE (SELECT snap FROM p) IS NOT NULL AND (xid>=pg_catalog.pg_snapshot_xmax((SELECT snap FROM p)) OR xid IN (SELECT pg_catalog.pg_snapshot_xip((SELECT snap FROM p)))) ORDER BY xid,seq LIMIT $2) l UNION ALL SELECT 'meta',CAST(NULL AS TEXT),CAST(NULL AS TEXT),CAST(NULL AS BIGINT),CAST(s.snap AS TEXT),CAST(pg_catalog.pg_snapshot_xmin(s.snap) AS TEXT),CAST(CASE WHEN pg_catalog.pg_is_in_recovery() THEN 1 ELSE 0 END AS BIGINT),{stamp} FROM s"
        )
    })
}

/// One poll: one SELECT inside a REPEATABLE READ READ ONLY transaction.
async fn read_postgres(
    conn: &mut sqlx::PgConnection,
    previous: Option<&str>,
) -> Result<Vec<sqlx::postgres::PgRow>, StoreError> {
    sqlx::query("BEGIN ISOLATION LEVEL REPEATABLE READ READ ONLY")
        .execute(&mut *conn)
        .await?;
    let rows = sqlx::query(postgres_poll_sql())
        .bind(previous)
        .bind(PAGE)
        .fetch_all(&mut *conn)
        .await?;
    sqlx::query("COMMIT").execute(&mut *conn).await?;
    Ok(rows)
}

async fn timeout<T>(
    limit: Duration,
    future: impl std::future::Future<Output = T>,
) -> Result<T, StoreError> {
    tokio::time::timeout(limit, future).await.map_err(|_| {
        StoreError::Db(sqlx::Error::Io(std::io::Error::new(
            std::io::ErrorKind::TimedOut,
            "authority mirror poll timed out",
        )))
    })
}

#[cfg(test)]
#[path = "mirror_tests.rs"]
mod tests;
