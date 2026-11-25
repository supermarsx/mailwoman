//! Write-behind request-protection accounting (`MW_REQUEST_PROTECTION_ACCOUNTING=write-behind`).
//!
//! Decisions are made in memory with `mw_store::request_protection::evaluate_protection`,
//! the same evaluator the transactional path uses. Observations are appended to one
//! ordered queue and written in the background through
//! `Store::request_protection_sqlite_batch_with_state` (at most 64 per transaction, in
//! queue order), after which each cached subject is re-seeded from the committed row.
//!
//! What is synchronous:
//! - any observation whose evaluation is a transition (throttle, block or stage change).
//!   That observation and everything queued before it are committed, with its audit
//!   event, before the decision is returned;
//! - the first observation of a subject without a row (or whose row has expired), so the
//!   subject-capacity check still runs before the request is answered;
//! - every `CredentialRejected` and `Malformed` observation (never deferred);
//! - loading a subject that is not cached (one read).
//!
//! When a cached subject is re-read (t50hp-e8):
//! - Ownership rule. While this instance is the only writer of the request-protection
//!   tables of its database, proven by the file witness in
//!   `mw_store::request_protection` (`ProtectionParticipant::check`, run on every flush
//!   tick), a cached subject is used until its row expires by its own policy window, it
//!   is evicted (bounded, below), or something invalidates it. A warm request then reads
//!   nothing. Ownership is checked again at every tick and lapses by itself two flush
//!   intervals after the instant taken before the last successful check (so a stall
//!   of this process after its check never extends it). Each check also reads the
//!   database marker of writes made outside the witness (one indexed read per tick,
//!   never on a request); a change re-reads every cached subject, so such a write (its
//!   writer could not join, or was not acknowledged in time) is seen within one
//!   interval. Whenever it is gained (again) or ends,
//!   or the check reports that another writer joined since the previous check, every
//!   cached subject without pending observations is re-read once on its next use.
//! - Re-read rule, without ownership (another participant exists, PostgreSQL never
//!   reaches this module, the witness is lost or unavailable): a subject with nothing
//!   pending is re-read when it was synced more than one flush interval ago.
//! - Writers outside the request path: administrative clear goes through [`WriteBehind::clear`];
//!   policy and settings changes call [`WriteBehind::invalidate`]; expiry pruning only
//!   deletes rows the evaluator already treats as absent; every other writer
//!   (a synchronous process, another write-behind process, a command-line tool using the
//!   store's request-protection functions) is a participant, which ends ownership
//!   before its first write.
//!
//! What can be lost on a crash: queued ordinary observations not yet flushed. Under
//! healthy storage they are at most `FLUSH_MS` old (plus one flush), and never more than
//! `MAX_PENDING`. A restarted process therefore under-counts each subject by at most
//! those observations. Transitions are never in that set: they are committed before
//! their decision is returned (a failed write-through is a storage failure, below).
//!
//! Full queue: at `MAX_PENDING` queued observations every further observation is written
//! through (backpressure: the request waits for the queue to drain up to it), so it is
//! counted and answered from the database. Only beyond `2 x MAX_PENDING`, which needs
//! write-throughs to keep failing, is an observation not recorded and the answer a storage
//! failure (as the synchronous mode answers when storage fails).
//!
//! Storage failure: a failed flush keeps its observations (bounded by `2 x MAX_PENDING`) and
//! marks the accounting degraded. The request is reported as a storage failure, so the
//! caller applies its configured failure policy (`storageFailure: closed` with an
//! enforcing mode answers 503), when the accounting has been degraded for longer than
//! `2 x FLUSH_MS`, when the queue is full, when a write-through fails, or when a cold
//! load fails. Otherwise the local decision is used.
//!
//! Several processes (no ownership). Each process decides from its own cache between
//! flushes. A process sees another's observations once that process has flushed them
//! (at most one interval after they were made) and it has re-read or reconciled the
//! subject (at most one interval later), so N processes sharing one SQLite file may
//! together admit up to about (N - 1) x a subject's admitted rate x 2 flush intervals
//! beyond its limit. Transitions are written through, so a block or throttle decided by
//! one process is seen by the others within one interval. The default `synchronous`
//! mode keeps exact serialization across processes.
use mw_store::{
    EscalationGuard, Store, StoreError, WriteEscalation, WriteLane, WriteLaneRequest,
    request_protection::{
        Ownership, PreparedProtection, ProtectionObservation, ProtectionOutcome,
        ProtectionParticipant, ProtectionSource, SUBJECT_CAPACITY, StoredProtection,
        SubjectKeyer, evaluate_protection,
    },
    request_protection_model::{Decision, Policy, Signal, State},
};
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock, PoisonError};
use std::time::{Duration, Instant};

pub(crate) const MODE_VAR: &str = "MW_REQUEST_PROTECTION_ACCOUNTING";
pub(crate) const FLUSH_VAR: &str = "MW_REQUEST_PROTECTION_FLUSH_MS";
pub(crate) const MAX_PENDING_VAR: &str = "MW_REQUEST_PROTECTION_MAX_PENDING";
pub(crate) const REREAD_VAR: &str = "MW_REQUEST_PROTECTION_REREAD_MS";
/// Over-admission fraction stated for the multi-process bound: with N processes,
/// a subject at its limit is over-admitted by at most (N - 1) x L x (F + S) / W
/// (L limit, W window, F flush interval, S re-read interval). Start-up refuses
/// F + S above `W_DEFAULT / BOUND_DIVISOR`; a policy window shorter than
/// `BOUND_DIVISOR x (F + S)` is accounted synchronously while not owned.
pub(crate) const BOUND_DIVISOR: u64 = 6;
/// The default rate window (`security.protection.rateWindowSecs`).
const DEFAULT_WINDOW: Duration = Duration::from_secs(60);
/// S in milliseconds (0: equal to the flush interval), set once at start-up.
static REREAD_MS: AtomicU64 = AtomicU64::new(0);
pub(crate) const PG_OWNERSHIP_VAR: &str = "MW_REQUEST_PROTECTION_PG_OWNERSHIP";
/// Ownership (the advisory-lock witness) on PostgreSQL: off unless
/// `MW_REQUEST_PROTECTION_PG_OWNERSHIP=on` (e15a C1-C3). Off, write-behind on
/// PostgreSQL uses the re-read rule with the compare-and-set flush.
pub(crate) static PG_OWNERSHIP: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
/// An owner trusts its cache again only after the writers marker has not
/// changed for this long.
const MARKER_QUIET: Duration = Duration::from_secs(2);
/// The batch API's group limit.
const BATCH: usize = 64;
const SHARDS: usize = 64;
const SHARD_CAPACITY: usize = SUBJECT_CAPACITY as usize / SHARDS;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AccountingMode {
    Synchronous,
    WriteBehind,
}
impl AccountingMode {
    /// Both backends support write-behind (PostgreSQL since t50hp-e8: advisory
    /// lock witness, compare-and-set flush).
    pub(crate) fn effective(self, _sqlite: bool) -> Self {
        self
    }
}
/// Process-immutable accounting switches, read once at startup.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct AccountingConfig {
    pub mode: AccountingMode,
    pub flush_every: Duration,
    pub max_pending: usize,
}
impl Default for AccountingConfig {
    fn default() -> Self {
        Self {
            mode: AccountingMode::Synchronous,
            flush_every: Duration::from_millis(250),
            max_pending: 4096,
        }
    }
}
impl AccountingConfig {
    pub(crate) fn from_env() -> Result<Self, String> {
        #[cfg(test)]
        if let Some(config) = TEST_CONFIG.with(std::cell::Cell::get) {
            return Ok(config);
        }
        let read = |name| std::env::var(name).ok();
        let config = Self::parse(
            read(MODE_VAR).as_deref(),
            read(FLUSH_VAR).as_deref(),
            read(MAX_PENDING_VAR).as_deref(),
        )?;
        let reread = parse_reread(read(REREAD_VAR).as_deref(), config.flush_every)?;
        let pg_ownership = match read(PG_OWNERSHIP_VAR).as_deref().map(str::trim) {
            None | Some("") | Some("off") => false,
            Some("on") => true,
            Some(other) => {
                return Err(format!("{PG_OWNERSHIP_VAR}={other:?} is invalid; use \"off\" or \"on\""));
            }
        };
        PG_OWNERSHIP.store(pg_ownership, Ordering::Relaxed);
        let mut config = config;
        let http_requests = crate::inbound_http::HttpLimits::from_env_without_probe()
            .map(|limits| limits.requests)
            .unwrap_or_else(|_| crate::inbound_http::HttpLimits::default().requests);
        let explicit = read(MAX_PENDING_VAR).filter(|raw| !raw.trim().is_empty()).map(|_| config.max_pending);
        config.max_pending = max_pending_for(explicit, http_requests)?;
        REREAD_MS.store(u64::try_from(reread.as_millis()).unwrap_or(u64::MAX), Ordering::Relaxed);
        Ok(config)
    }
    /// S, the re-read interval of an idle cached subject while not owned.
    pub(crate) fn reread(&self) -> Duration {
        match REREAD_MS.load(Ordering::Relaxed) {
            0 => self.flush_every,
            ms => Duration::from_millis(ms),
        }
    }
    /// Unset or empty values take the default; anything else must be valid.
    pub(crate) fn parse(
        mode: Option<&str>,
        flush_ms: Option<&str>,
        max_pending: Option<&str>,
    ) -> Result<Self, String> {
        fn set(value: Option<&str>) -> Option<&str> {
            value.map(str::trim).filter(|value| !value.is_empty())
        }
        let mut config = Self::default();
        match set(mode) {
            None | Some("synchronous") => {}
            Some("write-behind") => config.mode = AccountingMode::WriteBehind,
            Some(other) => {
                return Err(format!(
                    "{MODE_VAR}={other:?} is invalid; use \"synchronous\" or \"write-behind\""
                ));
            }
        }
        let ranged = |name: &str, value: Option<&str>, range: std::ops::RangeInclusive<u64>| {
            set(value)
                .map(|raw| {
                    raw.parse::<u64>()
                        .ok()
                        .filter(|value| range.contains(value))
                        .ok_or_else(|| {
                            format!(
                                "{name}={raw:?} is invalid; use an integer from {} to {}",
                                range.start(),
                                range.end()
                            )
                        })
                })
                .transpose()
        };
        if let Some(ms) = ranged(FLUSH_VAR, flush_ms, 50..=5000)? {
            config.flush_every = Duration::from_millis(ms);
        }
        // The ownership window and a joiner's wait derive from the interval.
        mw_store::request_protection::ownership_window(config.flush_every)
            .map_err(|error| format!("{FLUSH_VAR}: {error}"))?;
        if let Some(max) = ranged(MAX_PENDING_VAR, max_pending, 64..=65536)? {
            config.max_pending = max as usize;
        }
        Ok(config)
    }
}

#[cfg(test)]
thread_local! {
    /// Test-only: the configuration `from_env` returns on this thread.
    pub(crate) static TEST_CONFIG: std::cell::Cell<Option<AccountingConfig>> =
        const { std::cell::Cell::new(None) };
}

/// `MW_REQUEST_PROTECTION_REREAD_MS`: 50 to 5000, default the flush interval.
/// Refused when F + S exceeds the stated bound (see [`BOUND_DIVISOR`]).
pub(crate) fn parse_reread(raw: Option<&str>, flush_every: Duration) -> Result<Duration, String> {
    let reread = match raw.map(str::trim).filter(|value| !value.is_empty()) {
        None => flush_every,
        Some(raw) => raw
            .parse::<u64>()
            .ok()
            .filter(|ms| (50..=5000).contains(ms))
            .map(Duration::from_millis)
            .ok_or_else(|| format!("{REREAD_VAR}={raw:?} is invalid; use an integer from 50 to 5000"))?,
    };
    let limit = DEFAULT_WINDOW / u32::try_from(BOUND_DIVISOR).unwrap_or(1);
    if flush_every + reread > limit {
        return Err(format!(
            "{FLUSH_VAR} + {REREAD_VAR} = {} ms exceeds {} ms: with two processes a subject could be admitted more than 1/{BOUND_DIVISOR} of its limit beyond it at the default {} s window",
            (flush_every + reread).as_millis(),
            limit.as_millis(),
            DEFAULT_WINDOW.as_secs()
        ));
    }
    Ok(reread)
}

/// The refusal for a request whose observation could not be queued with
/// healthy storage (the queue stayed at its hard cap): wait one second.
const LOAD_REFUSAL: Decision = Decision {
    action: mw_store::request_protection_model::Action::Throttle,
    reason: Some(mw_store::request_protection_model::Reason::RequestRate),
    enforced: true,
    retry_after_secs: 1,
};

/// The smallest `MW_REQUEST_PROTECTION_MAX_PENDING` for an HTTP in-flight
/// request limit: a request records at most one observation at a time, so with
/// the queue at least twice the in-flight limit, the hard cap (twice the queue)
/// cannot be reached by in-flight requests alone. Unset: the larger of the
/// default and this floor (capped at 65536). Set below it: start-up is refused.
pub(crate) fn max_pending_for(configured: Option<usize>, http_requests: usize) -> Result<usize, String> {
    let floor = http_requests.saturating_mul(2).min(65536);
    match configured {
        None => Ok(AccountingConfig::default().max_pending.max(floor)),
        Some(value) if value >= floor => Ok(value),
        Some(value) => Err(format!(
            "{MAX_PENDING_VAR}={value} is below twice the HTTP in-flight request limit ({http_requests}); use at least {floor}, or leave it unset"
        )),
    }
}

/// The answer for one observation or probe read.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Answer {
    /// The decision from the cached state (or from the database after a write-through).
    pub decision: Decision,
    /// Apply the caller's storage-failure policy. `decision` is still the best local answer.
    pub storage_failed: bool,
}
impl Answer {
    fn ok(decision: Decision) -> Self {
        Self { decision, storage_failed: false }
    }
    fn failed(decision: Decision) -> Self {
        Self { decision, storage_failed: true }
    }
}

/// Counters for the status view and tests.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WriteBehindStatus {
    pub pending: usize,
    pub flushed: u64,
    pub batches: u64,
    pub write_throughs: u64,
    pub write_through_failures: u64,
    pub flush_failures: u64,
    /// Observations the database refused deterministically (for example subject capacity).
    pub rejected: u64,
    /// Observations not recorded because the queue was full.
    pub dropped: u64,
    pub loads: u64,
    /// Answers reported as storage failures (the caller's failure policy applied).
    pub storage_failures: u64,
    pub degraded: bool,
    pub cache_size: usize,
    pub evictions: u64,
    /// This instance currently holds ownership (cached subjects are not re-read).
    pub owned: bool,
    pub ownership_gained: u64,
    pub ownership_lost: u64,
    /// Changes of the database marker of writes outside the witness seen by checks.
    pub unjoined_writes_seen: u64,
    /// Observations written through because the queue was full.
    pub backpressure_writes: u64,
    /// Which regime this instance runs: whether ownership may be used at all
    /// (SQLite, or PostgreSQL with MW_REQUEST_PROTECTION_PG_OWNERSHIP=on).
    pub ownership_enabled: bool,
    /// Observations accounted synchronously because their window is shorter
    /// than six times F + S + 2D while not owned (the bound's write-through).
    pub short_window_write_throughs: u64,
    /// Requests that waited for a drain at the hard cap (healthy storage).
    pub full_waits: u64,
    /// The longest flush so far, in milliseconds (D in the multi-process bound).
    pub flush_max_ms: u64,
    /// Requests refused because the queue stayed at its hard cap with healthy storage.
    pub load_refusals: u64,
}
#[derive(Default)]
struct Counters {
    flushed: AtomicU64,
    batches: AtomicU64,
    write_throughs: AtomicU64,
    write_through_failures: AtomicU64,
    flush_failures: AtomicU64,
    rejected: AtomicU64,
    dropped: AtomicU64,
    loads: AtomicU64,
    evictions: AtomicU64,
    storage_failures: AtomicU64,
    ownership_gained: AtomicU64,
    ownership_lost: AtomicU64,
    unjoined_writes_seen: AtomicU64,
    backpressure_writes: AtomicU64,
    short_window_write_throughs: AtomicU64,
    full_waits: AtomicU64,
    flush_max_ms: AtomicU64,
    flush_last_ms: AtomicU64,
    load_refusals: AtomicU64,
}
fn bump(counter: &AtomicU64, by: u64) {
    counter.fetch_add(by, Ordering::Relaxed);
}

/// The graceful-shutdown flush did not finish before its deadline, or storage failed.
#[derive(Debug)]
pub(crate) struct FlushIncomplete {
    pub remaining: usize,
}

/// One queued observation, owned so it can outlive the request.
struct Pending {
    seq: u64,
    kind: &'static str,
    value: String,
    prepared: PreparedProtection,
    policy: Policy,
    now: i64,
    signal: Option<Signal>,
    country: Option<String>,
    ip: Option<String>,
    /// The committed decision, once flushed.
    durable: OnceLock<Decision>,
}
impl Pending {
    fn source(&self) -> ProtectionSource<'_> {
        ProtectionSource { country: self.country.as_deref(), ip: self.ip.as_deref() }
    }
    fn observation(&self) -> ProtectionObservation<'_> {
        ProtectionObservation {
            kind: self.kind,
            value: &self.value,
            policy: &self.policy,
            now: self.now,
            signal: self.signal,
            source: self.source(),
        }
    }
    fn replay(&self, state: &mut State, expires_at: &mut i64) {
        evaluate_protection(
            state, expires_at, &self.prepared, &self.policy, self.now, self.signal, self.source(),
        );
    }
}

/// A cached subject: the committed row plus every not-yet-committed observation.
struct Entry {
    state: State,
    expires_at: i64,
    /// A row is known to exist.
    persisted: bool,
    /// Re-read before the next use unless observations are pending.
    stale: bool,
    pending: VecDeque<Arc<Pending>>,
    synced_at: Instant,
    last_used: Instant,
}
impl Entry {
    fn loaded(row: Option<StoredProtection>, at: Instant) -> Self {
        let persisted = row.is_some();
        let (state, expires_at) = row.map_or((State::default(), 0), |row| (row.state, row.expires_at));
        Self {
            state,
            expires_at,
            persisted,
            stale: false,
            pending: VecDeque::new(),
            synced_at: at,
            last_used: at,
        }
    }
    /// An entry with pending observations is the database plus those observations,
    /// which is the most current view this process has; it is never re-read. Without
    /// pending observations: valid while owned, else for one flush interval.
    fn usable(&self, reread: Duration, owned: bool) -> bool {
        !self.pending.is_empty()
            || (!self.stale && (owned || self.synced_at.elapsed() < reread))
    }
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

/// A group handed to its own task (see `WriteBehind::store_group`).
struct Orphan {
    batch: Vec<Arc<Pending>>,
    /// `stale_epoch` taken before the task's database read (N3).
    epoch: u64,
    task: tokio::task::JoinHandle<Result<Vec<(ProtectionOutcome, Option<StoredProtection>)>, StoreError>>,
}
/// Keeps a group whose caller is cancelled mid-wait for the next drain.
struct OrphanGuard<'a> {
    owner: &'a WriteBehind,
    orphan: Option<Orphan>,
}
impl Drop for OrphanGuard<'_> {
    fn drop(&mut self) {
        if let Some(orphan) = self.orphan.take() {
            *lock(&self.owner.orphan) = Some(orphan);
        }
    }
}

enum Recorded {
    Full(Decision),
    Queued { pending: Arc<Pending>, decision: Decision, through: bool, transition: bool, full: bool, tight: bool, queued: usize },
}

pub(crate) struct WriteBehind {
    store: Store,
    /// Subject keys without re-deriving the HMAC subkey per key.
    keyer: SubjectKeyer,
    flush_every: Duration,
    /// S: an idle cached subject is re-read after this while not owned.
    reread: Duration,
    max_pending: usize,
    shards: Vec<Mutex<HashMap<String, Entry>>>,
    /// Global observation order; the batch contract applies observations in this order.
    queue: Mutex<VecDeque<Arc<Pending>>>,
    next_seq: AtomicU64,
    /// One flusher at a time, so committed order is queue order.
    flushing: tokio::sync::Mutex<()>,
    /// SQLite group writer (`MW_SQLITE_WRITER=group`): raised by a transition
    /// write-through or an admin clear before it takes `flushing`, so a timer
    /// flush holding `flushing` moves its normal-lane wait to the priority lane
    /// instead of keeping them behind the normal backlog (review G1).
    escalation: WriteEscalation,
    /// A group whose store call was handed to a task and whose caller was
    /// cancelled before it could book the result (review G3): the next drain
    /// books it first, so no observation is written twice.
    orphan: Mutex<Option<Orphan>>,
    /// This instance's witness participant (joined on first use), shared by the
    /// flusher and the ownership checker, which never waits for a flush.
    participant: Mutex<Option<Arc<ProtectionParticipant>>>,
    joining: tokio::sync::Mutex<()>,
    /// Whether the ownership rule may be used at all.
    ownership: std::sync::atomic::AtomicBool,
    /// Compare-and-set conflicts seen so far (PostgreSQL): one while owned means
    /// an unknown writer, and ownership is switched off for good.
    cas_conflicts_seen: AtomicU64,
    /// Milliseconds since `epoch` until which ownership holds; 0 = not owned.
    owned_until_ms: AtomicU64,
    epoch: Instant,
    /// Two flush intervals: how long one successful check is trusted.
    window: Duration,
    /// Bumped by every `mark_all_stale`. A load or a flush batch takes it before
    /// its database read and installs its row as stale when it changed, so a
    /// mark is never overwritten by a row read before it (e15a N3).
    stale_epoch: AtomicU64,
    /// Test hook: run `mark_all_stale` once inside `committed`, just before a
    /// subject's shard lock is taken.
    #[cfg(test)]
    mark_before_install: Mutex<bool>,
    /// Test hook: the hard-cap wait's drains make no room.
    #[cfg(test)]
    drain_makes_no_room: Mutex<bool>,
    /// When the writers marker last changed (a real change, not the first read).
    marker_changed_at: Mutex<Option<Instant>>,
    /// The writers marker as of the previous check (`None`: no check yet).
    marker: Mutex<Option<Option<String>>>,
    /// Test hook: a delay between a check and storing its deadline.
    #[cfg(test)]
    stall_after_check: Mutex<Option<Duration>>,
    wake: Arc<tokio::sync::Notify>,
    degraded_since: Mutex<Option<Instant>>,
    counters: Counters,
}

impl WriteBehind {
    pub(crate) fn new(store: Store, config: AccountingConfig) -> Self {
        let ownership = store.uses_sqlite() || PG_OWNERSHIP.load(Ordering::Relaxed);
        Self {
            keyer: store.request_protection_keyer(),
            store,
            flush_every: config.flush_every,
            reread: config.reread(),
            max_pending: config.max_pending,
            shards: (0..SHARDS).map(|_| Mutex::new(HashMap::new())).collect(),
            queue: Mutex::new(VecDeque::new()),
            next_seq: AtomicU64::new(1),
            flushing: tokio::sync::Mutex::new(()),
            escalation: WriteEscalation::new(),
            orphan: Mutex::new(None),
            participant: Mutex::new(None),
            joining: tokio::sync::Mutex::new(()),
            ownership: std::sync::atomic::AtomicBool::new(ownership),
            cas_conflicts_seen: AtomicU64::new(mw_store::request_protection::request_protection_witness_status().cas_conflicts),
            owned_until_ms: AtomicU64::new(0),
            epoch: Instant::now(),
            window: mw_store::request_protection::ownership_window(config.flush_every)
                .unwrap_or(config.flush_every * 2),
            stale_epoch: AtomicU64::new(0),
            #[cfg(test)]
            mark_before_install: Mutex::new(false),
            #[cfg(test)]
            drain_makes_no_room: Mutex::new(false),
            marker_changed_at: Mutex::new(None),
            marker: Mutex::new(None),
            #[cfg(test)]
            stall_after_check: Mutex::new(None),
            wake: Arc::new(tokio::sync::Notify::new()),
            degraded_since: Mutex::new(None),
            counters: Counters::default(),
        }
    }
    /// Construct and spawn the background flusher, which stops when the last
    /// strong reference is dropped.
    pub(crate) fn start(store: Store, config: AccountingConfig) -> Arc<Self> {
        let accounting = Arc::new(Self::new(store, config));
        let weak = Arc::downgrade(&accounting);
        *lock(&ACTIVE) = Some(weak.clone());
        let wake = accounting.wake.clone();
        let every = accounting.flush_every;
        tokio::spawn({
            let weak = weak.clone();
            async move {
                loop {
                    tokio::select! {
                        () = tokio::time::sleep(every) => {}
                        () = wake.notified() => {}
                    }
                    let Some(accounting) = weak.upgrade() else { break };
                    if accounting.flush_pending().await.is_err() {
                        super::warning();
                    }
                }
            }
        });
        // Ownership is checked on its own timer, twice per flush interval
        // (four times per ownership window), so a slow or blocked flush (write
        // gate, storage failure, a long drain) never lets ownership lapse; a
        // lapse would mark every cached subject for re-reading.
        let check_every = every / 2;
        tokio::spawn(async move {
            loop {
                let Some(accounting) = weak.upgrade() else { break };
                accounting.check_ownership().await;
                drop(accounting);
                tokio::time::sleep(check_every).await;
            }
        });
        accounting
    }

    /// A policy window shorter than `BOUND_DIVISOR x (F + S)`: rate window or
    /// short adaptive window.
    fn window_too_short(&self, policy: &Policy) -> bool {
        let window = u64::from(policy.rate_window_secs.min(policy.short.seconds)).saturating_mul(1000);
        // F + S + 2D with D the latest flush time (e15a C4): a slow database
        // moves more windows to the synchronous path instead of exceeding the bound.
        let flush = self.counters.flush_last_ms.load(Ordering::Relaxed);
        let staleness = u64::try_from((self.flush_every + self.reread).as_millis())
            .unwrap_or(u64::MAX)
            .saturating_add(flush.saturating_mul(2));
        window < staleness.saturating_mul(BOUND_DIVISOR)
    }

    /// Test hook: never use the ownership rule.
    #[cfg(test)]
    pub(crate) fn without_ownership(self) -> Self {
        self.ownership.store(false, Ordering::Relaxed);
        self
    }

    fn now_ms(&self) -> u64 {
        u64::try_from(self.epoch.elapsed().as_millis()).unwrap_or(u64::MAX)
    }
    /// Ownership holds now (see the module docs).
    pub(crate) fn owned(&self) -> bool {
        let until = self.owned_until_ms.load(Ordering::Acquire);
        until != 0 && self.now_ms() < until
    }
    /// Ends ownership. Every cached subject without pending observations is
    /// re-read on its next use: another writer may write right after this
    /// check acknowledged it, and an entry synced moments ago would otherwise
    /// still be used for up to one interval.
    fn lose_ownership(&self) {
        if self.owned_until_ms.swap(0, Ordering::AcqRel) != 0 {
            bump(&self.counters.ownership_lost, 1);
            self.mark_all_stale();
        }
    }
    #[cfg(test)]
    pub(crate) fn stall_after_check(&self, delay: Option<Duration>) {
        *lock(&self.stall_after_check) = delay;
    }
    /// Every cached subject without pending observations is re-read on its next use.
    fn mark_all_stale(&self) {
        // Before the entries: a reader that installs after this sees the change.
        self.stale_epoch.fetch_add(1, Ordering::SeqCst);
        for shard in &self.shards {
            for entry in lock(shard).values_mut() {
                if entry.pending.is_empty() {
                    entry.stale = true;
                }
            }
        }
    }
    /// This instance's participant, joining on first use.
    async fn participant(&self) -> Result<Arc<ProtectionParticipant>, StoreError> {
        if let Some(participant) = lock(&self.participant).clone() {
            return Ok(participant);
        }
        let _joining = self.joining.lock().await;
        if let Some(participant) = lock(&self.participant).clone() {
            return Ok(participant);
        }
        let participant = Arc::new(self.store.request_protection_participant().await?);
        *lock(&self.participant) = Some(participant.clone());
        Ok(participant)
    }
    /// One ownership check (its own timer, independent of flushing): file
    /// system calls and one indexed read of the writers marker.
    pub(crate) async fn check_ownership(&self) {
        if !self.ownership.load(Ordering::Relaxed) {
            return;
        }
        // A row changed under an owner's flush (PostgreSQL CAS conflict) means a
        // writer outside the witness, such as an older binary (e15a C3): stop
        // owning for the life of this process.
        let conflicts = mw_store::request_protection::request_protection_witness_status().cas_conflicts;
        if conflicts > self.cas_conflicts_seen.swap(conflicts, Ordering::Relaxed) && self.owned() {
            self.ownership.store(false, Ordering::Relaxed);
            self.lose_ownership();
            tracing::warn!(target: "request_protection", "a request-protection row changed under this owner's flush (another writer outside the witness, an older version?); ownership is off until restart");
            return;
        }
        let Ok(participant) = self.participant().await else {
            self.lose_ownership();
            return;
        };
        let checked = participant.check(self.window).await;
        match checked.ownership {
            Ownership::Alone { changed } => {
                // Writes made outside the witness carry the database marker.
                let Ok(marker) = self.store.request_protection_writers_marker().await else {
                    self.lose_ownership();
                    return;
                };
                let (marker_changed, marker_moved) = {
                    let mut seen = lock(&self.marker);
                    let changed = seen.as_ref() != Some(&marker);
                    let moved = changed && seen.is_some();
                    if moved {
                        bump(&self.counters.unjoined_writes_seen, 1);
                    }
                    *seen = Some(marker);
                    (changed, moved)
                };
                // While the marker moves (writers outside the witness are active)
                // and for MARKER_QUIET after, nobody owns: entries follow the re-read rule.
                {
                    let mut changed_at = lock(&self.marker_changed_at);
                    if marker_moved {
                        *changed_at = Some(Instant::now());
                    }
                    if changed_at.is_some_and(|at| at.elapsed() < MARKER_QUIET) {
                        drop(changed_at);
                        self.lose_ownership();
                        if marker_moved {
                            self.mark_all_stale();
                        }
                        return;
                    }
                }
                let was_owned = self.owned();
                if changed || marker_changed || !was_owned {
                    // Before ownership is (re)asserted: anything cached may predate
                    // another writer's rows.
                    self.mark_all_stale();
                }
                #[cfg(test)]
                {
                    let delay = *lock(&self.stall_after_check);
                    if let Some(delay) = delay {
                        tokio::time::sleep(delay).await;
                    }
                }
                // Counted from before the check took the admission lock: a joiner
                // that entered after the check waits longer than this window.
                let until = checked.started.saturating_duration_since(self.epoch) + self.window;
                let until = u64::try_from(until.as_millis()).unwrap_or(u64::MAX).max(1);
                self.owned_until_ms.store(until, Ordering::Release);
                if !was_owned {
                    bump(&self.counters.ownership_gained, 1);
                }
            }
            Ownership::Shared | Ownership::Busy => self.lose_ownership(),
            Ownership::Lost => {
                self.lose_ownership();
                // Join again (new lock handle) before the next write.
                let mut slot = lock(&self.participant);
                if slot.as_ref().is_some_and(|current| Arc::ptr_eq(current, &participant)) {
                    *slot = None;
                }
            }
        }
    }

    fn shard(&self, key: &str) -> &Mutex<HashMap<String, Entry>> {
        use std::hash::{BuildHasher, BuildHasherDefault, DefaultHasher};
        let hash = BuildHasherDefault::<DefaultHasher>::default().hash_one(key);
        &self.shards[(hash as usize) % SHARDS]
    }

    fn degraded_too_long(&self) -> bool {
        lock(&self.degraded_since).is_some_and(|since| since.elapsed() > self.flush_every * 2)
    }

    /// Run `f` on the subject's entry under its shard lock, loading the row first
    /// when it is not cached (or is stale and has nothing pending). Concurrent
    /// requests for one subject always evaluate the same entry, one at a time.
    async fn with_entry<R>(
        &self,
        key: &str,
        kind: &str,
        now: i64,
        f: impl FnOnce(&mut Entry) -> R,
    ) -> Result<R, StoreError> {
        {
            let mut entries = lock(self.shard(key));
            if let Some(entry) = entries.get_mut(key)
                && entry.usable(self.reread, self.owned())
            {
                entry.last_used = Instant::now();
                return Ok(f(entry));
            }
        }
        let started = Instant::now();
        let epoch = self.stale_epoch.load(Ordering::SeqCst);
        let row = self.store.request_protection_load(key, kind, now).await?;
        bump(&self.counters.loads, 1);
        let mut entries = lock(self.shard(key));
        // A mark made while the row was read may concern a write after the read.
        let marked = self.stale_epoch.load(Ordering::SeqCst) != epoch;
        let loaded = |row| {
            let mut entry = Entry::loaded(row, started);
            entry.stale = marked;
            entry
        };
        match entries.get_mut(key) {
            // A concurrent request loaded (and possibly charged) it first, or a
            // flush re-seeded it after our read began: keep the newer view.
            Some(entry) if !entry.pending.is_empty() || entry.synced_at > started => {}
            Some(entry) => *entry = loaded(row),
            None => {
                self.evict(&mut entries);
                entries.insert(key.to_owned(), loaded(row));
            }
        }
        let entry = entries.get_mut(key).expect("entry present under its shard lock");
        entry.last_used = Instant::now();
        Ok(f(entry))
    }

    /// Bounded cache. First drop entries that would be re-read anyway (no pending
    /// observation and older than one flush), which loses nothing. Only if every
    /// entry is fresh, drop the least recently used tenth of the idle ones. Entries
    /// with pending observations are never evicted, so a shard can exceed its share
    /// only by subjects that have queued observations (bounded by `MAX_PENDING`).
    fn evict(&self, entries: &mut HashMap<String, Entry>) {
        if entries.len() < SHARD_CAPACITY {
            return;
        }
        let before = entries.len();
        let owned = self.owned();
        entries.retain(|_, entry| entry.usable(self.reread, owned));
        if entries.len() >= SHARD_CAPACITY {
            let mut idle: Vec<_> = entries
                .iter()
                .filter(|(_, entry)| entry.pending.is_empty())
                .map(|(key, entry)| (entry.last_used, key.clone()))
                .collect();
            idle.sort();
            for (_, key) in idle.into_iter().take((SHARD_CAPACITY / 10).max(1)) {
                entries.remove(&key);
            }
        }
        bump(&self.counters.evictions, (before - entries.len()) as u64);
    }

    fn counted(&self, answer: Answer) -> Answer {
        if answer.storage_failed {
            bump(&self.counters.storage_failures, 1);
        }
        answer
    }

    /// Charge (`signal == None`) or observe one request, deciding from the cache.
    pub(crate) async fn record(
        &self,
        kind: &'static str,
        value: &str,
        policy: &Policy,
        now: i64,
        signal: Option<Signal>,
        source: ProtectionSource<'_>,
    ) -> Answer {
        let answer = self.record_inner(kind, value, policy, now, signal, source).await;
        self.counted(answer)
    }

    async fn record_inner(
        &self,
        kind: &'static str,
        value: &str,
        policy: &Policy,
        now: i64,
        signal: Option<Signal>,
        source: ProtectionSource<'_>,
    ) -> Answer {
        let prepared = match self.keyer.prepare(kind, value, policy, now, signal, source) {
            Ok(Some(prepared)) => prepared,
            Ok(None) => return Answer::ok(Decision::default()),
            Err(_) => return Answer::failed(Decision::default()),
        };
        let key = prepared.subject_key().to_owned();
        // At the hard cap with healthy storage (concurrency alone filled the
        // queue), wait for the queue to drain and record then (e15a N2); only
        // a degraded store or a drain that does not make room drops.
        let mut attempts = 0;
        loop {
        let owned = self.owned();
        let recorded = self
            .with_entry(&key, kind, now, |entry| {
                let mut queue = lock(&self.queue);
                // At MAX_PENDING the observation is still recorded and written
                // through (backpressure). Only beyond twice that, which needs
                // write-throughs to keep failing, is it not recorded.
                let full = queue.len() >= self.max_pending;
                if queue.len() >= self.max_pending.saturating_mul(2) {
                    // Refusals still apply; nothing is recorded.
                    let (mut state, mut expires_at) = (entry.state.clone(), entry.expires_at);
                    let step = evaluate_protection(
                        &mut state, &mut expires_at, &prepared, policy, now, signal, source,
                    );
                    return Recorded::Full(step.decision);
                }
                let new_row = !entry.persisted || entry.expires_at <= now;
                let step = evaluate_protection(
                    &mut entry.state, &mut entry.expires_at, &prepared, policy, now, signal, source,
                );
                let pending = Arc::new(Pending {
                    seq: self.next_seq.fetch_add(1, Ordering::Relaxed),
                    kind,
                    value: value.to_owned(),
                    prepared: prepared.clone(),
                    policy: policy.clone(),
                    now,
                    signal,
                    country: source.country.map(str::to_owned),
                    ip: source.ip.map(str::to_owned),
                    durable: OnceLock::new(),
                });
                entry.pending.push_back(pending.clone());
                queue.push_back(pending.clone());
                // Typed rejection signals are written through: never deferred.
                let typed = matches!(signal, Some(Signal::CredentialRejected | Signal::Malformed));
                // Not owned: a window shorter than BOUND_DIVISOR x (F + S) is
                // accounted synchronously, so the stated bound holds for it too.
                let tight = !owned && self.window_too_short(policy);
                Recorded::Queued {
                    pending,
                    decision: step.decision,
                    through: step.transition || new_row || typed || full || tight,
                    tight,
                    transition: step.transition,
                    full,
                    queued: queue.len(),
                }
            })
            .await;
        let (pending, decision, through, transition, queued) = match recorded {
            Err(_) => return Answer::failed(Decision::default()),
            Ok(Recorded::Full(decision)) => {
                let healthy = lock(&self.degraded_since).is_none();
                #[cfg(test)]
                let drained = if *lock(&self.drain_makes_no_room) { Ok(()) } else { self.flush_pending().await };
                #[cfg(not(test))]
                let drained = self.flush_pending().await;
                if healthy && attempts < 3 && drained.is_ok() {
                    attempts += 1;
                    bump(&self.counters.full_waits, 1);
                    continue;
                }
                bump(&self.counters.dropped, 1);
                if healthy {
                    // Load alone filled the queue: never admit uncounted, whatever
                    // `storageFailure` says (it is for failing storage).
                    bump(&self.counters.load_refusals, 1);
                    return Answer::ok(if decision.enforced { decision } else { LOAD_REFUSAL });
                }
                return Answer::failed(decision);
            }
            Ok(Recorded::Queued { pending, decision, through, transition, full, tight, queued }) => {
                if full {
                    bump(&self.counters.backpressure_writes, 1);
                }
                if tight {
                    bump(&self.counters.short_window_write_throughs, 1);
                }
                (pending, decision, through, transition, queued)
            }
        };
        if queued >= BATCH {
            self.wake.notify_one();
        }
        if through {
            bump(&self.counters.write_throughs, 1);
            // Only a transition (throttle, block, stage change) is T0 and uses
            // the group writer's priority lane; first rows and typed rejections,
            // which anonymous traffic can produce at will, wait on the normal
            // lane (review G2). Without the group writer the lane changes nothing.
            let (lane, _urgent): (WriteLaneRequest, Option<EscalationGuard>) = if transition {
                (WriteLane::Priority.into(), Some(self.escalation.raise()))
            } else {
                // Escalatable too: a transition that queues behind this
                // drain's `flushing` moves it to the priority lane (re-check G1).
                (WriteLaneRequest::escalatable(&self.escalation), None)
            };
            let flushed = self.drain(pending.seq, lane).await;
            return match (flushed, pending.durable.get()) {
                (Ok(()), Some(durable)) => Answer::ok(*durable),
                _ => {
                    bump(&self.counters.write_through_failures, 1);
                    Answer::failed(decision)
                }
            };
        }
        return Answer { decision, storage_failed: self.degraded_too_long() };
        }
    }

    /// Read an existing probe-path action without charging anything.
    pub(crate) async fn probe_decision(
        &self,
        kind: &'static str,
        value: &str,
        policy: &Policy,
        now: i64,
    ) -> Answer {
        if !policy.valid() || now < 0 {
            return Answer::failed(Decision::default());
        }
        if policy.adaptive_mode == mw_store::request_protection_model::Mode::Off {
            return Answer::ok(Decision::default());
        }
        let Ok(key) = self.keyer.subject(kind, value) else {
            return Answer::failed(Decision::default());
        };
        let answer = match self
            .with_entry(&key, kind, now, |entry| {
                if entry.expires_at <= now {
                    Decision::default()
                } else {
                    entry.state.probe_decision(policy, now)
                }
            })
            .await
        {
            Ok(decision) => Answer::ok(decision),
            Err(_) => Answer::failed(Decision::default()),
        };
        self.counted(answer)
    }

    /// Commit every observation up to and including `target`, in queue order.
    /// `lane` is the queue of the SQLite group writer (`MW_SQLITE_WRITER=group`):
    /// priority for a write-through (a transition answered to a client) and an
    /// administrative clear, normal for the timer and shutdown flushes. Without
    /// the writer it changes nothing.
    async fn drain(&self, target: u64, lane: WriteLaneRequest) -> Result<(), StoreError> {
        let _flushing = self.flushing.lock().await;
        self.drain_locked(target, lane).await
    }
    /// `drain` for a caller that already holds `flushing`.
    async fn drain_locked(&self, target: u64, lane: WriteLaneRequest) -> Result<(), StoreError> {
        self.settle_orphan().await;
        loop {
            let front = lock(&self.queue).front().map(|pending| pending.seq);
            match front {
                Some(seq) if seq <= target => {
                    let participant = match self.participant().await {
                        Ok(participant) => participant,
                        Err(error) => return Err(self.degrade(error)),
                    };
                    self.flush_batch(&participant, lane.clone()).await?
                }
                _ => return Ok(()),
            }
        }
    }

    /// Commit everything queued now. Used by the background flusher.
    pub(crate) async fn flush_pending(&self) -> Result<(), StoreError> {
        let Some(last) = lock(&self.queue).back().map(|pending| pending.seq) else {
            return Ok(());
        };
        // Normal lane, escalated to priority while a transition or an admin
        // clear waits for `flushing` (review G1).
        let started = Instant::now();
        let result = self.drain(last, WriteLaneRequest::escalatable(&self.escalation)).await;
        // D in the multi-process bound: the longest flush so far.
        let took = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
        self.counters.flush_max_ms.fetch_max(took, Ordering::Relaxed);
        self.counters.flush_last_ms.store(took, Ordering::Relaxed);
        result
    }

    /// Graceful shutdown: commit everything queued before `deadline`.
    pub(crate) async fn flush_all(&self, deadline: tokio::time::Instant) -> Result<(), FlushIncomplete> {
        let done = tokio::time::timeout_at(deadline, self.flush_pending()).await;
        let remaining = lock(&self.queue).len();
        match done {
            Ok(Ok(())) if remaining == 0 => Ok(()),
            _ => Err(FlushIncomplete { remaining }),
        }
    }

    /// One group of at most 64 from the front of the queue, one writer transaction
    /// (set-based in the store when every subject has a row). Caller holds `flushing`.
    async fn flush_batch(&self, participant: &Arc<ProtectionParticipant>, lane: WriteLaneRequest) -> Result<(), StoreError> {
        let batch: Vec<Arc<Pending>> = lock(&self.queue).iter().take(BATCH).cloned().collect();
        if batch.is_empty() {
            return Ok(());
        }
        match self.store_group(participant, batch.clone(), lane.clone()).await {
            Ok((results, epoch)) => {
                self.committed(&batch, results, epoch);
                Ok(())
            }
            // A deterministic refusal (corrupt row, subject capacity) would fail the
            // same group forever. Isolate it so the rest of the queue still commits.
            Err(StoreError::Corrupt(_)) => {
                for pending in &batch {
                    match self.store_group(participant, vec![pending.clone()], lane.clone()).await {
                        Ok((results, epoch)) => self.committed(std::slice::from_ref(pending), results, epoch),
                        Err(StoreError::Corrupt(_)) => self.rejected(pending),
                        Err(error) => return Err(self.degrade(error)),
                    }
                }
                Ok(())
            }
            Err(error) => Err(self.degrade(error)),
        }
    }

    /// The store call for one group, run by a task that owns it (review G3):
    /// if this caller is cancelled while the group waits in the group writer,
    /// the task still finishes, and its result is booked by the next drain
    /// (`settle_orphan`) instead of being lost, so the group is not written
    /// again. Caller holds `flushing`.
    async fn store_group(
        &self,
        participant: &Arc<ProtectionParticipant>,
        batch: Vec<Arc<Pending>>,
        lane: WriteLaneRequest,
    ) -> Result<(Vec<(ProtectionOutcome, Option<StoredProtection>)>, u64), StoreError> {
        let (store, participant) = (self.store.clone(), participant.clone());
        let group = batch.clone();
        // N3: before the task's database read, so a mark made while it reads
        // is not overwritten when its rows are booked (here or by settle_orphan).
        let epoch = self.stale_epoch.load(Ordering::SeqCst);
        // The request's statement measurement follows the group into its task.
        let task = tokio::spawn(mw_store::pool_telemetry::in_current_measurement(async move {
            let observations: Vec<_> = group.iter().map(|pending| pending.observation()).collect();
            store.request_protection_batch_in(&participant, &observations, lane).await
        }));
        let mut guard = OrphanGuard { owner: self, orphan: Some(Orphan { batch, epoch, task }) };
        let orphan = guard.orphan.as_mut().expect("orphan present");
        let joined = (&mut orphan.task).await;
        guard.orphan = None;
        joined.unwrap_or_else(|_| Err(StoreError::Db(sqlx::Error::WorkerCrashed))).map(|results| (results, epoch))
    }

    /// Book a group whose caller was cancelled after handing it to its task.
    /// Caller holds `flushing`, so no other group of the queue is in flight.
    async fn settle_orphan(&self) {
        let orphan = lock(&self.orphan).take();
        let Some(Orphan { batch, epoch, task }) = orphan else { return };
        match task.await {
            Ok(Ok(results)) => self.committed(&batch, results, epoch),
            // Not stored: the observations stay queued and the next group
            // writes them, as after any failed flush.
            Ok(Err(error)) => {
                self.degrade(error);
            }
            Err(_) => {}
        }
    }

    fn degrade(&self, error: StoreError) -> StoreError {
        bump(&self.counters.flush_failures, 1);
        lock(&self.degraded_since).get_or_insert_with(Instant::now);
        error
    }

    /// Re-seed each flushed subject from its committed row, then replay the
    /// observations it queued after this group.
    /// `epoch`: `stale_epoch` taken before the batch's database read.
    fn committed(
        &self,
        batch: &[Arc<Pending>],
        results: Vec<(ProtectionOutcome, Option<StoredProtection>)>,
        epoch: u64,
    ) {
        let mut last: HashMap<&str, (u64, Option<StoredProtection>)> = HashMap::new();
        for (pending, (outcome, stored)) in batch.iter().zip(results) {
            let _ = pending.durable.set(outcome.decision);
            last.insert(pending.prepared.subject_key(), (pending.seq, stored));
        }
        let synced = Instant::now();
        for (key, (seq, stored)) in last {
            #[cfg(test)]
            if std::mem::take(&mut *lock(&self.mark_before_install)) {
                self.mark_all_stale();
            }
            let mut entries = lock(self.shard(key));
            // Under the subject's shard lock: `mark_all_stale` bumps the epoch
            // before it visits any shard, so a mark that has not yet reached
            // this shard is seen here (e15a N3).
            let marked = self.stale_epoch.load(Ordering::SeqCst) != epoch;
            let Some(entry) = entries.get_mut(key) else { continue };
            while entry.pending.front().is_some_and(|pending| pending.seq <= seq) {
                entry.pending.pop_front();
            }
            match stored {
                Some(stored) => {
                    entry.state = stored.state;
                    entry.expires_at = stored.expires_at;
                    entry.persisted = true;
                    // Re-read on the next use if a mark happened since the read.
                    entry.stale = marked;
                    for pending in &entry.pending {
                        pending.replay(&mut entry.state, &mut entry.expires_at);
                    }
                    entry.synced_at = synced;
                }
                None => entry.stale = true,
            }
        }
        let mut queue = lock(&self.queue);
        for pending in batch {
            if queue.front().is_some_and(|front| front.seq == pending.seq) {
                queue.pop_front();
            }
        }
        drop(queue);
        bump(&self.counters.flushed, batch.len() as u64);
        bump(&self.counters.batches, 1);
        *lock(&self.degraded_since) = None;
    }

    fn rejected(&self, pending: &Arc<Pending>) {
        bump(&self.counters.rejected, 1);
        let key = pending.prepared.subject_key();
        if let Some(entry) = lock(self.shard(key)).get_mut(key) {
            entry.pending.retain(|queued| queued.seq != pending.seq);
            entry.stale = true;
        }
        let mut queue = lock(&self.queue);
        if queue.front().is_some_and(|front| front.seq == pending.seq) {
            queue.pop_front();
        }
    }

    /// Policy or settings changed: drop every cached subject with nothing pending,
    /// and re-read the rest after their next flush.
    pub(crate) fn invalidate(&self) {
        for shard in &self.shards {
            let mut entries = lock(shard);
            entries.retain(|_, entry| !entry.pending.is_empty());
            for entry in entries.values_mut() {
                entry.stale = true;
            }
        }
        self.wake.notify_one();
    }

    /// Administrative clear: commit what is pending, clear the row with its
    /// expected revision, then restart the cached subject from an absent row.
    pub(crate) async fn clear(&self, key: &str, expected: &str, now: i64) -> Result<bool, StoreError> {
        // An admin clear is T0: a timer flush holding `flushing` escalates.
        let _urgent = self.escalation.raise();
        // Hold the flusher out until the cached subject matches the cleared row.
        let _flushing = self.flushing.lock().await;
        let last = lock(&self.queue).back().map(|pending| pending.seq);
        if let Some(last) = last {
            self.drain_locked(last, WriteLane::Priority.into()).await?;
        } else {
            self.settle_orphan().await;
        }
        let participant = self.participant().await?;
        let cleared = self.store.clear_request_protection_as(&participant, key, expected, now).await?;
        let mut entries = lock(self.shard(key));
        if cleared && let Some(entry) = entries.get_mut(key) {
            if entry.pending.is_empty() {
                entries.remove(key);
            } else {
                // Observations recorded after the flush land on the absent row,
                // exactly as the next flush will apply them.
                entry.state = State::default();
                entry.expires_at = 0;
                entry.persisted = false;
                for pending in &entry.pending {
                    pending.replay(&mut entry.state, &mut entry.expires_at);
                }
            }
        }
        Ok(cleared)
    }

    pub(crate) fn status(&self) -> WriteBehindStatus {
        let counter = |value: &AtomicU64| value.load(Ordering::Relaxed);
        WriteBehindStatus {
            pending: lock(&self.queue).len(),
            flushed: counter(&self.counters.flushed),
            batches: counter(&self.counters.batches),
            write_throughs: counter(&self.counters.write_throughs),
            write_through_failures: counter(&self.counters.write_through_failures),
            flush_failures: counter(&self.counters.flush_failures),
            rejected: counter(&self.counters.rejected),
            dropped: counter(&self.counters.dropped),
            loads: counter(&self.counters.loads),
            storage_failures: counter(&self.counters.storage_failures),
            degraded: lock(&self.degraded_since).is_some(),
            cache_size: self.shards.iter().map(|shard| lock(shard).len()).sum(),
            evictions: counter(&self.counters.evictions),
            owned: self.owned(),
            ownership_gained: counter(&self.counters.ownership_gained),
            ownership_lost: counter(&self.counters.ownership_lost),
            unjoined_writes_seen: counter(&self.counters.unjoined_writes_seen),
            backpressure_writes: counter(&self.counters.backpressure_writes),
            ownership_enabled: self.ownership.load(Ordering::Relaxed),
            short_window_write_throughs: counter(&self.counters.short_window_write_throughs),
            full_waits: counter(&self.counters.full_waits),
            flush_max_ms: counter(&self.counters.flush_max_ms),
            load_refusals: counter(&self.counters.load_refusals),
        }
    }
}

/// The most recently started accounting in this process, for the load-fixture
/// witness. Weak: it never keeps a stopped server's accounting alive.
static ACTIVE: Mutex<Option<std::sync::Weak<WriteBehind>>> = Mutex::new(None);

/// Load-fixture witness: process-global write-behind counters, counts only (no
/// subject keys, addresses or account ids). `accounting` is `synchronous` with
/// zero counters when no write-behind accounting is running in this process.
/// Counters are monotonic per accounting instance; sample before and after.
#[cfg(any(test, feature = "load-test-fixtures"))]
pub fn fixture_snapshot() -> serde_json::Value {
    let active = lock(&ACTIVE).as_ref().and_then(std::sync::Weak::upgrade);
    let (accounting, status) = match active {
        Some(active) => ("write-behind", active.status()),
        None => ("synchronous", WriteBehindStatus::default()),
    };
    let mut value = serde_json::to_value(status).expect("status serializes");
    if let Some(object) = value.as_object_mut() {
        object.insert("supported".into(), true.into());
        object.insert("accounting".into(), accounting.into());
        object.insert(
            "writerJoins".into(),
            serde_json::to_value(mw_store::request_protection::request_protection_witness_status())
                .expect("witness status serializes"),
        );
    }
    value
}

#[cfg(test)]
#[path = "write_behind_tests.rs"]
pub(crate) mod tests;

#[cfg(test)]
#[path = "write_behind_writer_tests.rs"]
mod writer_tests;
