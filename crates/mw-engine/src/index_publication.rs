//! One bounded, ordered index publication owner. SQL remains authoritative.
//! Fresh optimistic witnesses are rechecked after publication; no cross-DB
//! atomicity is claimed.
//!
//! Acknowledgement modes ([`PublicationAck`]):
//! * `commit` (default): callers that await their waiter return after the index
//!   commit. No journal rows are captured or acknowledged.
//! * `journal` (SQLite only): store triggers (migration 0116) write a journal row
//!   in the same transaction as every indexed source change. `Email/set` may
//!   return after its SQL commit without awaiting the index. Durability comes
//!   from the journal: the owner acknowledges rows only after a successful
//!   index commit, startup replays what is left, and a sweeper retries rows no
//!   publication covered. Read-your-writes comes from [`Publisher::read_fence`]:
//!   every index-backed read waits until the account's publications handed off
//!   before the read started are applied, and returns an error (never a stale
//!   result) on timeout, poison, pending repair, erasure or an unfinished
//!   startup replay.
use crate::{
    backend::{EngineError, Result},
    v6::V6Hooks,
};
use mw_store::index_publication::{IndexPublicationMessage, IndexPublicationRequest};
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, VecDeque},
    sync::{
        Arc, Mutex, OnceLock, RwLock, Weak,
        atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
    },
    time::Duration,
};
use tokio::sync::{Notify, OwnedSemaphorePermit, Semaphore, mpsc, oneshot};

/// When a metadata mutation (`Email/set`) is acknowledged to the client.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PublicationAck {
    /// After the sealed index commit (today's behaviour).
    #[default]
    Commit,
    /// After the SQL commit that also wrote the journal row; reads are fenced.
    Journal,
}
impl PublicationAck {
    /// `commit` or `journal` (case-insensitive, surrounding space ignored).
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "commit" => Some(Self::Commit),
            "journal" => Some(Self::Journal),
            _ => None,
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Commit => "commit",
            Self::Journal => "journal",
        }
    }
}

#[derive(Clone, Debug)]
pub struct PublicationOptions {
    pub max_pending_jobs: usize,
    pub max_pending_bytes: usize,
    pub max_batch_jobs: usize,
    pub max_batch_documents: usize,
    pub max_source_bytes: usize,
    pub max_prepared_bytes: usize,
    pub max_pending_repair_keys: usize,
    pub collection_window: Duration,
    pub admission_timeout: Duration,
    pub revalidation_attempts: usize,
    /// Acknowledgement mode. `Journal` takes effect only after
    /// [`crate::Engine::start_index_publication`] has enabled trigger capture.
    pub ack: PublicationAck,
    /// Longest an index-backed read waits on its account's fence (journal mode).
    pub read_fence_timeout: Duration,
    /// Default deadline of [`crate::Engine::drain_index_publication`].
    pub drain_timeout: Duration,
}
impl Default for PublicationOptions {
    fn default() -> Self {
        Self {
            max_pending_jobs: 128,
            max_pending_bytes: 16 * 1024 * 1024,
            max_batch_jobs: 64,
            max_batch_documents: 4096,
            max_source_bytes: 32 * 1024 * 1024,
            max_prepared_bytes: 32 * 1024 * 1024,
            max_pending_repair_keys: 4096,
            collection_window: Duration::from_millis(1),
            admission_timeout: Duration::from_secs(30),
            revalidation_attempts: 2,
            ack: PublicationAck::Commit,
            read_fence_timeout: Duration::from_secs(5),
            drain_timeout: Duration::from_secs(10),
        }
    }
}
impl PublicationOptions {
    pub fn validate(&self) -> Result<()> {
        if !(1..=1024).contains(&self.max_pending_jobs)
            || !(4096..=128 * 1024 * 1024).contains(&self.max_pending_bytes)
            || self.max_batch_jobs == 0
            || self.max_batch_jobs > self.max_pending_jobs
            || !(1..=4096).contains(&self.max_batch_documents)
            || !(65536..=256 * 1024 * 1024).contains(&self.max_source_bytes)
            || !(65536..=256 * 1024 * 1024).contains(&self.max_prepared_bytes)
            || !(1..=65536).contains(&self.max_pending_repair_keys)
            || self.collection_window > Duration::from_millis(10)
            || self.admission_timeout.is_zero()
            || self.admission_timeout > Duration::from_secs(60)
            || !(1..=4).contains(&self.revalidation_attempts)
            || self.read_fence_timeout < Duration::from_millis(100)
            || self.read_fence_timeout > Duration::from_secs(30)
            || self.drain_timeout > Duration::from_secs(60)
        {
            return Err(error("invalid bounded index publication options"));
        }
        Ok(())
    }
}
fn error(message: &str) -> EngineError {
    EngineError::Protocol(message.into())
}
fn search_error(error: mw_search::SearchError) -> EngineError {
    EngineError::Protocol(format!("index publication: {error}"))
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PublicationMode {
    Reconcile,
    Rebuild,
    DeleteIfAbsent,
    Metadata,
    AttachmentMetadata,
    MetadataAndAttachment,
    ReconcileAndAttachment,
}
impl PublicationMode {
    // Source reconciliation and attachment projection are separate obligations.
    fn source_strength(self) -> u8 {
        match self {
            Self::AttachmentMetadata => 0,
            Self::Metadata | Self::MetadataAndAttachment => 1,
            Self::DeleteIfAbsent => 2,
            Self::Reconcile | Self::ReconcileAndAttachment => 3,
            Self::Rebuild => 4,
        }
    }
    fn needs_attachment(self) -> bool {
        matches!(
            self,
            Self::AttachmentMetadata
                | Self::MetadataAndAttachment
                | Self::ReconcileAndAttachment
                | Self::Rebuild
        )
    }
    // Only a deterministic ordering for grouping equal recovery intents.
    fn rank(self) -> u8 {
        self.source_strength() * 2 + u8::from(self.needs_attachment())
    }
    fn strongest(self, other: Self) -> Self {
        let source = self.source_strength().max(other.source_strength());
        let attachment = self.needs_attachment() || other.needs_attachment();
        match (source, attachment) {
            (4, _) => Self::Rebuild,
            (3, true) | (2, true) => Self::ReconcileAndAttachment,
            (3, false) => Self::Reconcile,
            (2, false) => Self::DeleteIfAbsent,
            (1, true) => Self::MetadataAndAttachment,
            (1, false) => Self::Metadata,
            _ => Self::AttachmentMetadata,
        }
    }
    fn covers(self, pending: Self) -> bool {
        self.source_strength() >= pending.source_strength()
            && (!pending.needs_attachment() || self.needs_attachment())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct IndexPublicationReceipt {
    pub publication: Option<mw_search::PublicationReceipt>,
    pub considered_messages: usize,
    pub removed_account_documents: u64,
    pub suppressed: bool,
    pub erasure_epoch: Option<u64>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicationStatus {
    pub running: bool,
    pub poisoned: bool,
    pub pending_jobs: usize,
    pub pending_bytes: usize,
    pub queued_jobs: usize,
    pub submitted_jobs: u64,
    pub completed_jobs: u64,
    pub failed_jobs: u64,
    pub published_batches: u64,
    pub published_changes: u64,
    pub revalidations: u64,
    pub pending_repair_messages: usize,
    pub requires_full_rebuild: bool,
    pub erasing_accounts: usize,
    pub max_pending_jobs: usize,
    pub max_pending_bytes: usize,
    pub max_batch_documents: usize,
    /// Configured acknowledgement mode.
    pub ack_mode: PublicationAck,
    /// Whether `Email/set` currently returns before the index commit (journal
    /// mode after trigger capture was enabled).
    pub early_ack: bool,
    /// Whether index reads are held back until the startup journal replay ends.
    pub journal_replay_pending: bool,
    /// Journal rows at the last observation (startup or sweeper pass); `None`
    /// when the journal has not been inspected in this process.
    pub journal_rows: Option<u64>,
    /// Journal rows removed after successful publications.
    pub journal_acknowledged_rows: u64,
    /// Failed journal acknowledgement statements (rows stay for replay).
    pub journal_ack_failures: u64,
    /// Keys resubmitted from the journal by startup replay or the sweeper.
    pub journal_replayed_keys: u64,
    /// Reads that had to wait on their account's fence.
    pub read_fence_waits: u64,
    /// Reads refused because the fence did not open in time.
    pub read_fence_timeouts: u64,
    /// Reads refused because publication was unhealthy for the account.
    pub read_fence_refusals: u64,
    /// Admission closed by [`Publisher::drain`].
    pub draining: bool,
}
#[derive(Default)]
struct Stats {
    running: AtomicBool,
    poisoned: AtomicBool,
    pending_jobs: AtomicUsize,
    pending_bytes: AtomicUsize,
    queued_jobs: AtomicUsize,
    submitted: AtomicU64,
    completed: AtomicU64,
    failed: AtomicU64,
    batches: AtomicU64,
    changes: AtomicU64,
    revalidations: AtomicU64,
    requires_full_rebuild: AtomicBool,
    repair_epoch: AtomicU64,
}
/// Journal-mode state. All flags are inert in commit mode unless startup found
/// leftover rows from an earlier journal-mode run.
#[derive(Default)]
struct JournalState {
    /// The owner captures `index_journal_max_seq` and acknowledges rows.
    active: AtomicBool,
    /// `Email/set` does not await its waiter. Set only after capture is enabled.
    early_ack: AtomicBool,
    /// Reads wait (bounded) until startup replay has completed cleanly.
    replay_pending: AtomicBool,
    replay_done: Notify,
    draining: AtomicBool,
    sweeper_started: AtomicBool,
    rows: AtomicU64,
    rows_known: AtomicBool,
    acknowledged: AtomicU64,
    ack_failures: AtomicU64,
    replayed: AtomicU64,
    fence_waits: AtomicU64,
    fence_timeouts: AtomicU64,
    fence_refusals: AtomicU64,
}
/// Per-account publication watermark. `submitted` is assigned under its lock
/// together with the channel send, so channel order equals sequence order and
/// the FIFO owner advances `published` monotonically.
#[derive(Default)]
struct Watermark {
    submitted: Mutex<u64>,
    published: AtomicU64,
    advanced: Notify,
}
struct AccountGate {
    epoch: AtomicU64,
    closed: AtomicBool,
}
struct Core {
    options: PublicationOptions,
    jobs: Arc<Semaphore>,
    bytes: Arc<Semaphore>,
    stats: Stats,
    journal: JournalState,
    watermarks: Mutex<HashMap<String, Arc<Watermark>>>,
    gates: Mutex<HashMap<String, Weak<AccountGate>>>,
    erasing: Mutex<HashMap<String, Arc<AccountGate>>>,
    dirty: Mutex<BTreeMap<(String, String), PublicationMode>>,
    projection_clock: crate::mail_projection_proof::ProjectionClock,
    #[cfg(test)]
    hook: Mutex<Option<PublicationTestHook>>,
}
struct Context {
    store: mw_store::Store,
    search: Arc<mw_search::Index>,
    hooks: Arc<RwLock<V6Hooks>>,
    claim: OnceLock<Arc<mw_search::PublicationOwnerClaim>>,
}
impl Context {
    fn owner(&self) -> Result<&mw_search::PublicationOwnerClaim> {
        self.claim.get().map(Arc::as_ref).ok_or_else(|| error("index publication owner unavailable"))
    }
}
#[derive(Clone)]
pub(crate) struct ProjectionObservation {
    local: crate::mail_projection_proof::ProjectionProof,
    unowned: u64,
    gate: Arc<AccountGate>,
    epoch: u64,
}
struct Producer {
    core: Arc<Core>,
    context: Arc<Context>,
    sender: Mutex<Option<mpsc::Sender<Job>>>,
}
#[derive(Clone)]
pub(crate) struct Publisher(Arc<Producer>);
struct Admission {
    _job: OwnedSemaphorePermit,
    _bytes: OwnedSemaphorePermit,
    core: Arc<Core>,
    charged: usize,
    // Follows the owned job through waiter cancellation, not the waiter.
    _projection_write: Option<crate::mail_projection_proof::ProjectionWrite>,
}
impl Drop for Admission {
    fn drop(&mut self) {
        self.core.stats.pending_jobs.fetch_sub(1, Ordering::Relaxed);
        self.core
            .stats
            .pending_bytes
            .fetch_sub(self.charged, Ordering::Relaxed);
    }
}
struct TicketState {
    sender: mpsc::Sender<Job>,
    admission: Admission,
    account: String,
    gate: Arc<AccountGate>,
    epoch: u64,
    slots: usize,
    tracked_ids: Vec<String>,
    tracked_mode: PublicationMode,
}
pub(crate) struct PublicationTicket {
    state: Option<TicketState>,
}
pub(crate) struct PublicationWaiter(oneshot::Receiver<Result<IndexPublicationReceipt>>);
impl std::future::Future for PublicationWaiter {
    type Output = Result<IndexPublicationReceipt>;
    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        use std::future::Future;
        match std::pin::Pin::new(&mut self.0).poll(cx) {
            std::task::Poll::Ready(Ok(result)) => std::task::Poll::Ready(result),
            std::task::Poll::Ready(Err(_)) => std::task::Poll::Ready(Err(error(
                "index publication owner stopped before acknowledgement",
            ))),
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }
}
enum Kind {
    Reconcile {
        ids: Vec<String>,
        mode: PublicationMode,
    },
    Erase,
    FinishErase {
        expected_epoch: u64,
    },
    Recover,
    FinishFullRebuild {
        expected_epoch: u64,
    },
}
struct Job {
    account: String,
    gate: Arc<AccountGate>,
    epoch: u64,
    kind: Kind,
    ack: oneshot::Sender<Result<IndexPublicationReceipt>>,
    _admission: Option<Admission>,
    watermark: Option<(Arc<Watermark>, u64)>,
}
impl Job {
    fn ids(&self) -> &[String] {
        match &self.kind {
            Kind::Reconcile { ids, .. } => ids,
            _ => &[],
        }
    }
}
/// Interval of the journal-mode sweeper.
const JOURNAL_SWEEP_INTERVAL: Duration = Duration::from_secs(2);

/// Outcome of [`crate::Engine::start_index_publication`].
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JournalStartup {
    pub mode: PublicationAck,
    /// Keys resubmitted from leftover journal rows.
    pub replayed_keys: u64,
    /// Keys whose replay failed (their rows remain).
    pub failed_keys: u64,
    /// Journal rows after replay, when they could be counted.
    pub remaining_rows: Option<u64>,
    /// Whether fenced index reads are served. In journal mode `false` means the
    /// sweeper keeps retrying and reads are refused until a clean pass.
    pub ready: bool,
}
#[derive(Default)]
struct ReplayPass {
    keys: u64,
    failed: u64,
}

pub struct AccountIndexErasure {
    publisher: Publisher,
    account: String,
    gate: Arc<AccountGate>,
    epoch: u64,
    pub removed_documents: u64,
}
impl AccountIndexErasure {
    /// Close the erasure fence only after authoritative account removal. Drop
    /// deliberately leaves it closed so cancellation cannot repopulate data.
    pub async fn complete(self) -> Result<()> {
        let ticket = self.publisher.reserve_inner(&self.account, 0, true).await?;
        if !Arc::ptr_eq(&ticket.state.as_ref().unwrap().gate, &self.gate) {
            return Err(error("account erasure generation changed"));
        }
        // The caller has removed the account authoritatively. Purge its journal
        // rows before the fence reopens, so no replay can recreate documents.
        // Replay reconciles to current source anyway; this keeps the journal
        // from holding an erased account's keys at all.
        if self.publisher.0.core.journal.active.load(Ordering::Acquire) {
            self.publisher
                .0
                .context
                .store
                .index_journal_purge_account(&self.account)
                .await?;
        }
        ticket
            .send(Kind::FinishErase {
                expected_epoch: self.epoch,
            })?
            .await?;
        Ok(())
    }
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PublicationTestPhase {
    BeforeExecution,
    AfterValidation,
    BeforeAcknowledgement,
}
#[cfg(test)]
pub(crate) type PublicationTestHook = Arc<
    dyn Fn(PublicationTestPhase) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
        + Send
        + Sync,
>;

impl Publisher {
    pub(crate) fn new(
        store: mw_store::Store,
        search: Arc<mw_search::Index>,
        hooks: Arc<RwLock<V6Hooks>>,
        options: PublicationOptions,
    ) -> Result<Self> {
        options.validate()?;
        let journal = JournalState::default();
        if options.ack == PublicationAck::Journal {
            // Until startup enables capture and replays leftovers, early
            // acknowledgement stays off and fenced reads are held back.
            journal.active.store(true, Ordering::Release);
            journal.replay_pending.store(true, Ordering::Release);
        }
        Ok(Self(Arc::new(Producer {
            core: Arc::new(Core {
                jobs: Arc::new(Semaphore::new(options.max_pending_jobs)),
                bytes: Arc::new(Semaphore::new(options.max_pending_bytes)),
                options,
                stats: Stats::default(),
                journal,
                watermarks: Mutex::new(HashMap::new()),
                gates: Mutex::new(HashMap::new()),
                erasing: Mutex::new(HashMap::new()),
                dirty: Mutex::new(BTreeMap::new()),
                projection_clock: Default::default(),
                #[cfg(test)]
                hook: Mutex::new(None),
            }),
            context: Arc::new(Context {
                store,
                search,
                hooks,
                claim: OnceLock::new(),
            }),
            sender: Mutex::new(None),
        })))
    }
    fn sender(&self) -> Result<mpsc::Sender<Job>> {
        let mut sender = self
            .0
            .sender
            .lock()
            .map_err(|_| error("index producer lock poisoned"))?;
        if let Some(sender) = sender.as_ref() {
            if sender.is_closed() || !self.0.core.stats.running.load(Ordering::Acquire) {
                self.0.core.stats.poisoned.store(true, Ordering::Release);
                return Err(error(
                    "index publication owner stopped; admission is closed",
                ));
            }
            return Ok(sender.clone());
        }
        let runtime = tokio::runtime::Handle::try_current()
            .map_err(|_| error("index publication requires an active runtime"))?;
        let claim = Arc::new(self
            .0
            .context
            .search
            .claim_publication_owner()
            .map_err(search_error)?);
        self.0.context.claim.set(claim.clone())
            .map_err(|_| error("index publication owner already initialized"))?;
        let (tx, rx) = mpsc::channel(self.0.core.options.max_pending_jobs);
        let context = Arc::clone(&self.0.context);
        let core = Arc::clone(&self.0.core);
        core.stats.running.store(true, Ordering::Release);
        let stopped = Arc::clone(&core);
        if std::thread::Builder::new()
            .name("mw-index-publisher".into())
            .spawn(move || {
                struct Running(Arc<Core>);
                impl Drop for Running {
                    fn drop(&mut self) {
                        self.0.stats.running.store(false, Ordering::Release);
                    }
                }
                let _running = Running(core.clone());
                let _claim = claim;
                runtime.block_on(run_owner(context, core, rx));
            })
            .is_err()
        {
            stopped.stats.running.store(false, Ordering::Release);
            return Err(error("index publication thread could not start"));
        }
        *sender = Some(tx.clone());
        Ok(tx)
    }
    fn gate(&self, account: &str) -> Result<Arc<AccountGate>> {
        if account.is_empty() || account.len() > 1024 {
            return Err(error("invalid publication account"));
        }
        if let Some(gate) = self
            .0
            .core
            .erasing
            .lock()
            .map_err(|_| error("index erasure lock poisoned"))?
            .get(account)
        {
            return Ok(gate.clone());
        }
        let mut gates = self
            .0
            .core
            .gates
            .lock()
            .map_err(|_| error("index gate lock poisoned"))?;
        gates.retain(|_, gate| gate.strong_count() > 0);
        if let Some(gate) = gates.get(account).and_then(Weak::upgrade) {
            return Ok(gate);
        }
        if gates.len() >= 10000 {
            return Err(error("publication account gate limit reached"));
        }
        let gate = Arc::new(AccountGate {
            epoch: AtomicU64::new(0),
            closed: AtomicBool::new(false),
        });
        gates.insert(account.into(), Arc::downgrade(&gate));
        Ok(gate)
    }
    pub(crate) async fn reserve(&self, account: &str, slots: usize) -> Result<PublicationTicket> {
        self.reserve_inner(account, slots, false).await
    }
    pub(crate) async fn reserve_rebuild(
        &self,
        account: &str,
        slots: usize,
    ) -> Result<PublicationTicket> {
        self.reserve_inner(account, slots, true).await
    }
    async fn reserve_inner(
        &self,
        account: &str,
        slots: usize,
        barrier: bool,
    ) -> Result<PublicationTicket> {
        let core = &self.0.core;
        if slots > core.options.max_batch_documents {
            return Err(error("index publication document limit exceeded"));
        }
        if core.journal.draining.load(Ordering::Acquire) {
            return Err(error("index publication is shutting down; admission is closed"));
        }
        if !barrier
            && (core.stats.poisoned.load(Ordering::Acquire)
                || self.0.context.search.publication_is_poisoned())
        {
            return Err(error("index publication is paused pending repair"));
        }
        let gate = self.gate(account)?;
        if !barrier && gate.closed.load(Ordering::Acquire) {
            return Err(error("account index erasure is in progress"));
        }
        let charge = slots
            .checked_mul(1088)
            .and_then(|bytes| bytes.checked_add(account.len() + 256))
            .ok_or_else(|| error("index admission byte overflow"))?;
        if charge > core.options.max_pending_bytes {
            return Err(error("index admission byte limit exceeded"));
        }
        let deadline = tokio::time::Instant::now() + core.options.admission_timeout;
        let job = tokio::time::timeout_at(deadline, core.jobs.clone().acquire_owned())
            .await
            .map_err(|_| error("index publication admission timed out"))?
            .map_err(|_| error("index publication admission closed"))?;
        let bytes = tokio::time::timeout_at(
            deadline,
            core.bytes.clone().acquire_many_owned(charge as u32),
        )
        .await
        .map_err(|_| error("index publication admission timed out"))?
        .map_err(|_| error("index publication admission closed"))?;
        if !barrier
            && (gate.closed.load(Ordering::Acquire) || core.stats.poisoned.load(Ordering::Acquire))
        {
            return Err(error("index publication admission changed"));
        }
        let sender = self.sender()?;
        let projection_write = core.projection_clock.begin(account);
        if projection_write.is_none() {
            // Cache observation limits must not add a new mutation failure.
            core.projection_clock.invalidate_all();
        }
        core.stats.pending_jobs.fetch_add(1, Ordering::Relaxed);
        core.stats
            .pending_bytes
            .fetch_add(charge, Ordering::Relaxed);
        Ok(PublicationTicket {
            state: Some(TicketState {
                sender,
                admission: Admission {
                    _job: job,
                    _bytes: bytes,
                    core: core.clone(),
                    charged: charge,
                    _projection_write: projection_write,
                },
                account: account.into(),
                epoch: gate.epoch.load(Ordering::Acquire),
                gate,
                slots,
                tracked_ids: Vec::new(),
                tracked_mode: PublicationMode::Reconcile,
            }),
        })
    }
    pub(crate) async fn begin_erasure(&self, account: &str) -> Result<AccountIndexErasure> {
        let ticket = self.reserve_inner(account, 0, true).await?;
        let gate = ticket.state.as_ref().unwrap().gate.clone();
        let receipt = ticket.send(Kind::Erase)?.await?;
        Ok(AccountIndexErasure {
            publisher: self.clone(),
            account: account.into(),
            gate,
            epoch: receipt
                .erasure_epoch
                .ok_or_else(|| error("missing erasure epoch"))?,
            removed_documents: receipt.removed_account_documents,
        })
    }
    pub(crate) async fn recover(&self) -> Result<()> {
        self.reserve_inner("__publication_recovery__", 0, true)
            .await?
            .send(Kind::Recover)?
            .await?;
        Ok(())
    }
    pub(crate) fn begin_full_rebuild(&self) -> u64 {
        self.0.core.projection_clock.invalidate_all();
        self.0.core.stats.poisoned.store(true, Ordering::Release);
        self.0.core.stats.repair_epoch.load(Ordering::Acquire)
    }
    pub(crate) async fn finish_full_rebuild(&self, expected_epoch: u64) -> Result<()> {
        self.reserve_inner("__publication_recovery__", 0, true)
            .await?
            .send(Kind::FinishFullRebuild { expected_epoch })?
            .await?;
        Ok(())
    }
    pub(crate) fn status(&self) -> PublicationStatus {
        let core = &self.0.core;
        let s = &core.stats;
        PublicationStatus {
            running: s.running.load(Ordering::Acquire),
            poisoned: s.poisoned.load(Ordering::Acquire)
                || self.0.context.search.publication_is_poisoned(),
            pending_jobs: s.pending_jobs.load(Ordering::Relaxed),
            pending_bytes: s.pending_bytes.load(Ordering::Relaxed),
            queued_jobs: s.queued_jobs.load(Ordering::Relaxed),
            submitted_jobs: s.submitted.load(Ordering::Relaxed),
            completed_jobs: s.completed.load(Ordering::Relaxed),
            failed_jobs: s.failed.load(Ordering::Relaxed),
            published_batches: s.batches.load(Ordering::Relaxed),
            published_changes: s.changes.load(Ordering::Relaxed),
            revalidations: s.revalidations.load(Ordering::Relaxed),
            pending_repair_messages: core.dirty.lock().map_or(0, |keys| keys.len()),
            requires_full_rebuild: s.requires_full_rebuild.load(Ordering::Acquire),
            erasing_accounts: core.erasing.lock().map_or(0, |gates| gates.len()),
            max_pending_jobs: core.options.max_pending_jobs,
            max_pending_bytes: core.options.max_pending_bytes,
            max_batch_documents: core.options.max_batch_documents,
            ack_mode: core.options.ack,
            early_ack: core.journal.early_ack.load(Ordering::Acquire),
            journal_replay_pending: core.journal.replay_pending.load(Ordering::Acquire),
            journal_rows: core
                .journal
                .rows_known
                .load(Ordering::Acquire)
                .then(|| core.journal.rows.load(Ordering::Acquire)),
            journal_acknowledged_rows: core.journal.acknowledged.load(Ordering::Relaxed),
            journal_ack_failures: core.journal.ack_failures.load(Ordering::Relaxed),
            journal_replayed_keys: core.journal.replayed.load(Ordering::Relaxed),
            read_fence_waits: core.journal.fence_waits.load(Ordering::Relaxed),
            read_fence_timeouts: core.journal.fence_timeouts.load(Ordering::Relaxed),
            read_fence_refusals: core.journal.fence_refusals.load(Ordering::Relaxed),
            draining: core.journal.draining.load(Ordering::Acquire),
        }
    }
    /// Whether `Email/set` may return without awaiting its publication: journal
    /// mode, after trigger capture was enabled durably.
    pub(crate) fn acknowledges_after_journal(&self) -> bool {
        self.0.core.journal.early_ack.load(Ordering::Acquire)
    }
    /// Read-your-writes fence for an index-backed read of `account`.
    ///
    /// In commit mode (and no leftover startup replay) it returns at once and
    /// changes nothing. In journal mode it waits until every publication of
    /// this account handed off before the call has been applied by the owner,
    /// bounded by `read_fence_timeout`, and then refuses (an error, never a
    /// stale result) while the startup replay is unfinished, the owner is
    /// poisoned or stopped, a full rebuild is required, the account is being
    /// erased, or a repair of one of the account's keys is pending.
    pub(crate) async fn read_fence(&self, account: &str) -> Result<()> {
        let core = &self.0.core;
        if core.options.ack != PublicationAck::Journal
            && !core.journal.replay_pending.load(Ordering::Acquire)
        {
            return Ok(());
        }
        let deadline = tokio::time::Instant::now() + core.options.read_fence_timeout;
        let mut waited = false;
        loop {
            let notified = core.journal.replay_done.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            if !core.journal.replay_pending.load(Ordering::Acquire) {
                break;
            }
            waited = true;
            if tokio::time::timeout_at(deadline, notified).await.is_err() {
                core.journal.fence_waits.fetch_add(1, Ordering::Relaxed);
                core.journal.fence_timeouts.fetch_add(1, Ordering::Relaxed);
                return Err(error(
                    "search index is replaying its publication journal; retry",
                ));
            }
        }
        let watermark = core
            .watermarks
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(account)
            .cloned();
        if let Some(watermark) = watermark {
            let target = *watermark
                .submitted
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            loop {
                let notified = watermark.advanced.notified();
                tokio::pin!(notified);
                notified.as_mut().enable();
                if watermark.published.load(Ordering::Acquire) >= target {
                    break;
                }
                if let Err(reason) = self.fence_health(account) {
                    if waited {
                        core.journal.fence_waits.fetch_add(1, Ordering::Relaxed);
                    }
                    return Err(reason);
                }
                waited = true;
                if tokio::time::timeout_at(deadline, notified).await.is_err() {
                    core.journal.fence_waits.fetch_add(1, Ordering::Relaxed);
                    core.journal.fence_timeouts.fetch_add(1, Ordering::Relaxed);
                    return Err(error(
                        "search index publication is behind for this account; retry",
                    ));
                }
            }
        }
        if waited {
            core.journal.fence_waits.fetch_add(1, Ordering::Relaxed);
        }
        self.fence_health(account)
    }
    fn fence_health(&self, account: &str) -> Result<()> {
        let core = &self.0.core;
        let refuse = |message: &str| {
            core.journal.fence_refusals.fetch_add(1, Ordering::Relaxed);
            Err(error(message))
        };
        if core.stats.poisoned.load(Ordering::Acquire)
            || self.0.context.search.publication_is_poisoned()
        {
            return refuse("search index publication is paused pending repair");
        }
        if core.stats.requires_full_rebuild.load(Ordering::Acquire) {
            return refuse("search index requires a full rebuild");
        }
        let started = self
            .0
            .sender
            .lock()
            .map(|sender| sender.as_ref().map(mpsc::Sender::is_closed))
            .unwrap_or(Some(true));
        if started == Some(true)
            || (started.is_some() && !core.stats.running.load(Ordering::Acquire))
        {
            return refuse("search index publication owner stopped");
        }
        if core
            .erasing
            .lock()
            .map_or(true, |erasing| erasing.contains_key(account))
        {
            return refuse("account index erasure is in progress");
        }
        let dirty = core.dirty.lock().map_or(true, |dirty| {
            dirty
                .range((account.to_owned(), String::new())..)
                .next()
                .is_some_and(|((owner, _), _)| owner == account)
        });
        if dirty {
            return refuse("search index repair is pending for this account");
        }
        Ok(())
    }
    /// Close admission and wait until every accepted job has been processed,
    /// or `timeout` elapses. Returns whether the queue drained. Rows of jobs
    /// still queued at the deadline stay in the journal for the next start.
    pub(crate) async fn drain(&self, timeout: Duration) -> bool {
        let core = &self.0.core;
        core.journal.draining.store(true, Ordering::Release);
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            if core.stats.pending_jobs.load(Ordering::Acquire) == 0 {
                return true;
            }
            if tokio::time::Instant::now() >= deadline {
                return false;
            }
            tokio::time::sleep(Duration::from_millis(2)).await;
        }
    }
    pub(crate) fn drain_timeout(&self) -> Duration {
        self.0.core.options.drain_timeout
    }
    /// Journal sequence to trim up to after a full rebuild; `None` when the
    /// journal is not in use in this process. Capture BEFORE the source scan.
    pub(crate) async fn journal_capture(&self) -> Result<Option<i64>> {
        if !self.0.core.journal.active.load(Ordering::Acquire) {
            return Ok(None);
        }
        Ok(Some(self.0.context.store.index_journal_max_seq().await?))
    }
    /// After a successful full rebuild: delete rows `<= upto`.
    pub(crate) async fn journal_trim(&self, upto: i64) -> Result<()> {
        let removed = self.0.context.store.index_journal_complete_all(upto).await?;
        self.0
            .core
            .journal
            .acknowledged
            .fetch_add(removed, Ordering::Relaxed);
        self.refresh_journal_rows().await;
        Ok(())
    }
    pub(crate) async fn journal_rows(&self) -> Result<u64> {
        let rows = self.0.context.store.index_journal_len().await?;
        self.0.core.journal.rows.store(rows, Ordering::Release);
        self.0.core.journal.rows_known.store(true, Ordering::Release);
        Ok(rows)
    }
    async fn refresh_journal_rows(&self) {
        if let Err(reason) = self.journal_rows().await {
            tracing::warn!("index publication journal: row count unavailable: {reason}");
        }
    }
    /// Startup: reconcile the journal with the configured mode.
    ///
    /// * journal: enable trigger capture (refused on PostgreSQL), turn on early
    ///   acknowledgement, replay every row present, then open the read fence and
    ///   start the sweeper. A replay with failures leaves the fence closed; the
    ///   sweeper retries until a pass is clean.
    /// * commit: if capture is enabled or rows exist (an earlier journal-mode
    ///   run), disable capture, replay the rows and report. With neither, this is
    ///   two reads and nothing else.
    pub(crate) async fn start_journal(&self) -> Result<JournalStartup> {
        let core = &self.0.core;
        let store = &self.0.context.store;
        match core.options.ack {
            PublicationAck::Journal => {
                store.set_index_journal_enabled(true).await?;
                core.journal.early_ack.store(true, Ordering::Release);
                let upto = store.index_journal_max_seq().await?;
                let pass = self.replay_pass(upto).await;
                let clean = matches!(&pass, Ok(pass) if pass.failed == 0);
                if clean {
                    core.journal.replay_pending.store(false, Ordering::Release);
                    core.journal.replay_done.notify_waiters();
                }
                self.spawn_sweeper();
                let rows = self.journal_rows().await.ok();
                let pass = pass?;
                Ok(JournalStartup {
                    mode: PublicationAck::Journal,
                    replayed_keys: pass.keys,
                    failed_keys: pass.failed,
                    remaining_rows: rows,
                    ready: clean,
                })
            }
            PublicationAck::Commit => {
                let enabled = store.index_journal_enabled().await?;
                let rows = store.index_journal_len().await?;
                if !enabled && rows == 0 {
                    return Ok(JournalStartup {
                        mode: PublicationAck::Commit,
                        replayed_keys: 0,
                        failed_keys: 0,
                        remaining_rows: Some(0),
                        ready: true,
                    });
                }
                core.journal.active.store(true, Ordering::Release);
                core.journal.replay_pending.store(true, Ordering::Release);
                let outcome = async {
                    if enabled {
                        // A control row left enabled makes migrate-store refuse
                        // this file; commit mode must not keep capturing.
                        store.set_index_journal_enabled(false).await?;
                    }
                    let upto = store.index_journal_max_seq().await?;
                    self.replay_pass(upto).await
                }
                .await;
                let remaining = self.journal_rows().await.ok();
                // Commit mode keeps today's best-effort read semantics: the
                // fence opens after one pass whatever its outcome; leftover
                // rows are retried at the next start.
                core.journal.replay_pending.store(false, Ordering::Release);
                core.journal.replay_done.notify_waiters();
                if remaining == Some(0) {
                    core.journal.active.store(false, Ordering::Release);
                }
                let pass = outcome?;
                Ok(JournalStartup {
                    mode: PublicationAck::Commit,
                    replayed_keys: pass.keys,
                    failed_keys: pass.failed,
                    remaining_rows: remaining,
                    ready: true,
                })
            }
        }
    }
    /// Resubmit every journal row with `seq <= upto` as a reconcile of the
    /// current source and wait for the owner. The owner acknowledges the rows.
    async fn replay_pass(&self, upto: i64) -> Result<ReplayPass> {
        let core = &self.0.core;
        let store = &self.0.context.store;
        let chunk = core.options.max_batch_documents.min(256);
        let mut pass = ReplayPass::default();
        let mut after = 0i64;
        while after < upto {
            let page = store.index_journal_page(after, 1024).await?;
            let Some(last) = page.last() else { break };
            after = last.seq;
            let mut groups: BTreeMap<String, BTreeMap<String, PublicationMode>> = BTreeMap::new();
            for entry in page.iter().filter(|entry| entry.seq <= upto) {
                // The mode is a hint; every mode below reconciles against current
                // source. Metadata keeps legacy documents patchable.
                let mode = match entry.mode {
                    mw_store::index_publication::IndexJournalMode::Metadata => {
                        PublicationMode::Metadata
                    }
                    mw_store::index_publication::IndexJournalMode::Reconcile => {
                        PublicationMode::Reconcile
                    }
                    mw_store::index_publication::IndexJournalMode::Delete => {
                        PublicationMode::DeleteIfAbsent
                    }
                };
                groups
                    .entry(entry.account_id.clone())
                    .or_default()
                    .entry(entry.stable_id.clone())
                    .and_modify(|previous| *previous = previous.strongest(mode))
                    .or_insert(mode);
            }
            let mut waiters = Vec::new();
            for (account, keys) in groups {
                if self.is_erasing(&account) {
                    // Erasure completion purges these rows.
                    continue;
                }
                let mut by_mode: BTreeMap<u8, (PublicationMode, Vec<String>)> = BTreeMap::new();
                for (id, mode) in keys {
                    by_mode.entry(mode.rank()).or_insert((mode, Vec::new())).1.push(id);
                }
                for (mode, ids) in by_mode.into_values() {
                    for ids in ids.chunks(chunk) {
                        pass.keys += ids.len() as u64;
                        core.journal
                            .replayed
                            .fetch_add(ids.len() as u64, Ordering::Relaxed);
                        let submitted = match self.reserve(&account, ids.len()).await {
                            Ok(ticket) => ticket.submit(ids.to_vec(), mode),
                            Err(reason) => Err(reason),
                        };
                        match submitted {
                            Ok(waiter) => waiters.push((ids.len() as u64, waiter)),
                            Err(reason) => {
                                if !self.is_erasing(&account) {
                                    pass.failed += ids.len() as u64;
                                    tracing::warn!(
                                        "index publication journal: replay deferred: {reason}"
                                    );
                                }
                            }
                        }
                    }
                }
            }
            for (keys, waiter) in waiters {
                if let Err(reason) = waiter.await {
                    pass.failed += keys;
                    tracing::warn!("index publication journal: replay failed: {reason}");
                }
            }
        }
        Ok(pass)
    }
    fn is_erasing(&self, account: &str) -> bool {
        self.0
            .core
            .erasing
            .lock()
            .map_or(true, |erasing| erasing.contains_key(account))
    }
    /// Background retry of journal rows that no publication acknowledged
    /// (writers without a publication, failed publications). Each tick
    /// resubmits rows that already existed at the previous tick, so in-flight
    /// publications get one interval to acknowledge their own rows. While the
    /// startup replay is unfinished it retries every row and opens the fence
    /// after a clean pass. Holds only a weak reference to the producer.
    fn spawn_sweeper(&self) {
        if self.0.core.journal.sweeper_started.swap(true, Ordering::AcqRel) {
            return;
        }
        let Ok(runtime) = tokio::runtime::Handle::try_current() else {
            return;
        };
        let weak = Arc::downgrade(&self.0);
        runtime.spawn(async move {
            let mut previous = 0i64;
            loop {
                tokio::time::sleep(JOURNAL_SWEEP_INTERVAL).await;
                let Some(producer) = weak.upgrade() else { break };
                let publisher = Publisher(producer);
                if publisher.0.core.journal.draining.load(Ordering::Acquire) {
                    break;
                }
                publisher.sweep_once(&mut previous).await;
            }
        });
    }
    async fn sweep_once(&self, previous: &mut i64) {
        let core = &self.0.core;
        let current = match self.0.context.store.index_journal_max_seq().await {
            Ok(seq) => seq,
            Err(reason) => {
                tracing::warn!("index publication journal: sweep skipped: {reason}");
                return;
            }
        };
        let pending = core.journal.replay_pending.load(Ordering::Acquire);
        let upto = if pending { current } else { *previous };
        *previous = current;
        if upto <= 0 && !pending {
            return;
        }
        match self.replay_pass(upto).await {
            Ok(pass) if pass.failed == 0 && pending => {
                core.journal.replay_pending.store(false, Ordering::Release);
                core.journal.replay_done.notify_waiters();
            }
            Ok(_) => {}
            Err(reason) => tracing::warn!("index publication journal: sweep failed: {reason}"),
        }
        self.refresh_journal_rows().await;
    }
    #[cfg(test)]
    pub(crate) async fn sweep_for_test(&self, upto_previous: i64) {
        let mut previous = upto_previous;
        self.sweep_once(&mut previous).await;
    }
    /// Account-local DATA observation; never establishes ownership or auth.
    /// The result must also match a fresh joint Store/auth data observation.
    pub(crate) fn projection_proof(
        &self,
        account: &str,
        ids: &[String],
    ) -> Option<ProjectionObservation> {
        self.sender().ok()?;
        let core = &self.0.core;
        let gate=self.gate(account).ok()?;
        let epoch=gate.epoch.load(Ordering::Acquire);
        let unowned = self.0.context.search.unowned_projection_stamp()?;
        let clean = || {
            if core.stats.poisoned.load(Ordering::Acquire)
                || core.stats.requires_full_rebuild.load(Ordering::Acquire)
                || self.0.context.search.publication_is_poisoned()
                || !core.stats.running.load(Ordering::Acquire)
            { return false; }
            let Ok(erasing) = core.erasing.lock() else { return false; };
            if erasing.contains_key(account) { return false; }
            drop(erasing);
            let Ok(dirty) = core.dirty.lock() else { return false; };
            !dirty.keys().any(|(owner, _)| owner == account)
        };
        if !clean() { return None; }
        let proof = core.projection_clock.observe_selected(account,ids)?;
        (clean() && proof.is_current() && !gate.closed.load(Ordering::Acquire)
            && gate.epoch.load(Ordering::Acquire)==epoch
            && self.0.context.search.unowned_projection_stamp() == Some(unowned))
            .then_some(ProjectionObservation { local: proof, unowned,gate,epoch })
    }
    pub(crate) fn projection_is_current(
        &self,
        account: &str,
        ids: &[String],
        proof: &ProjectionObservation,
    ) -> bool {
        self.projection_proof(account,ids).is_some_and(|current|
            proof.unowned == current.unowned && proof.epoch==current.epoch
                && Arc::ptr_eq(&proof.gate,&current.gate) && proof.local.matches(&current.local))
    }
    #[cfg(test)]
    pub(crate) fn set_test_hook(&self, hook: Option<PublicationTestHook>) {
        *self.0.core.hook.lock().unwrap() = hook;
    }
}
impl PublicationTicket {
    /// Arm a bounded cancellation handoff before a multi-step source mutation.
    /// Drop then reconciles current authoritative rows, never tentative patches.
    pub(crate) fn track_ids(&mut self, ids: Vec<String>) -> Result<()> {
        let state = self
            .state
            .as_mut()
            .ok_or_else(|| error("publication ticket already handed off"))?;
        if ids.len() > state.slots || ids.iter().any(|id| id.is_empty() || id.len() > 1024) {
            return Err(error("publication exceeds reserved input"));
        }
        state.tracked_ids = ids;
        if let Some(write)=&mut state.admission._projection_write {write.touched(&state.tracked_ids);}
        Ok(())
    }
    pub(crate) fn track_metadata_ids(&mut self, ids: Vec<String>) -> Result<()> {
        self.track_ids(ids)?;
        self.state.as_mut().unwrap().tracked_mode = PublicationMode::Metadata;
        Ok(())
    }
    pub(crate) fn track_attachment_ids(&mut self, ids: Vec<String>) -> Result<()> {
        self.track_ids(ids)?;
        self.state.as_mut().unwrap().tracked_mode = PublicationMode::AttachmentMetadata;
        Ok(())
    }
    pub(crate) fn submit(
        self,
        ids: Vec<String>,
        mode: PublicationMode,
    ) -> Result<PublicationWaiter> {
        let state = self
            .state
            .as_ref()
            .ok_or_else(|| error("publication ticket already handed off"))?;
        if ids.len() > state.slots || ids.iter().any(|id| id.is_empty() || id.len() > 1024) {
            return Err(error("publication exceeds reserved input"));
        }
        self.send(Kind::Reconcile { ids, mode })
    }
    fn send(mut self, kind: Kind) -> Result<PublicationWaiter> {
        Self::handoff(
            self.state
                .take()
                .ok_or_else(|| error("publication ticket already handed off"))?,
            kind,
        )
    }
    fn handoff(state: TicketState, kind: Kind) -> Result<PublicationWaiter> {
        let mut state=state;
        if let Kind::Reconcile {ids,..}=&kind {
            if let Some(write)=&mut state.admission._projection_write {write.touched(ids);}
        }
        let (tx, rx) = oneshot::channel();
        let core = state.admission.core.clone();
        let watermark = core.watermark(&state.account);
        let mut job = Job {
            account: state.account,
            gate: state.gate,
            epoch: state.epoch,
            kind,
            ack: tx,
            _admission: Some(state.admission),
            watermark: None,
        };
        core.stats.submitted.fetch_add(1, Ordering::Relaxed);
        core.stats.queued_jobs.fetch_add(1, Ordering::Relaxed);
        let sent = {
            // Sequence assignment and send under one per-account lock: channel
            // order is sequence order, so the FIFO owner never publishes a
            // higher sequence of this account before a lower one.
            let mut submitted = watermark
                .submitted
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let seq = *submitted + 1;
            job.watermark = Some((watermark.clone(), seq));
            let sent = state.sender.try_send(job);
            if sent.is_ok() {
                *submitted = seq;
            }
            sent
        };
        if let Err(failed) = sent {
            let job = failed.into_inner();
            core.remember_dirty(std::slice::from_ref(&job));
            core.stats.queued_jobs.fetch_sub(1, Ordering::Relaxed);
            core.stats.failed.fetch_add(1, Ordering::Relaxed);
            core.stats.poisoned.store(true, Ordering::Release);
            return Err(error(
                "publication owner stopped; committed source needs repair",
            ));
        }
        Ok(PublicationWaiter(rx))
    }
}
impl Drop for PublicationTicket {
    fn drop(&mut self) {
        if let Some(mut state) = self.state.take() {
            if !state.tracked_ids.is_empty() {
                let ids = std::mem::take(&mut state.tracked_ids);
                let mode = state.tracked_mode;
                let _ = Self::handoff(state, Kind::Reconcile { ids, mode });
            }
        }
    }
}
impl Core {
    fn watermark(&self, account: &str) -> Arc<Watermark> {
        let mut marks = self
            .watermarks
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(mark) = marks.get(account) {
            return mark.clone();
        }
        if marks.len() >= 10000 {
            // Only settled marks nobody holds can go: a fence that finds no mark
            // has nothing to wait for, which is exactly a settled mark's answer.
            marks.retain(|_, mark| {
                Arc::strong_count(mark) > 1
                    || *mark
                        .submitted
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        != mark.published.load(Ordering::Acquire)
            });
        }
        let mark = Arc::new(Watermark::default());
        marks.insert(account.to_owned(), mark.clone());
        mark
    }
    fn remember_dirty(&self, jobs: &[Job]) {
        self.remember_dirty_as(jobs, None);
    }
    fn remember_dirty_as(&self, jobs: &[Job], force: Option<PublicationMode>) {
        self.stats.repair_epoch.fetch_add(1, Ordering::AcqRel);
        if let Ok(mut dirty) = self.dirty.lock() {
            for job in jobs {
                for id in job.ids() {
                    if dirty.len() >= self.options.max_pending_repair_keys
                        && !dirty.contains_key(&(job.account.clone(), id.clone()))
                    {
                        self.stats
                            .requires_full_rebuild
                            .store(true, Ordering::Release);
                        self.stats.poisoned.store(true, Ordering::Release);
                        break;
                    }
                    let mode = force.unwrap_or_else(|| match job.kind {
                        Kind::Reconcile { mode, .. } => mode,
                        _ => PublicationMode::Rebuild,
                    });
                    dirty
                        .entry((job.account.clone(), id.clone()))
                        .and_modify(|previous| *previous = previous.strongest(mode))
                        .or_insert(mode);
                }
            }
        }
    }
    fn complete_dirty(&self, jobs: &[Job], full_source: bool) {
        if let Ok(mut dirty) = self.dirty.lock() {
            for job in jobs {
                for id in job.ids() {
                    let key = (job.account.clone(), id.clone());
                    let completed = if full_source {
                        PublicationMode::Rebuild
                    } else {
                        intent_mode(jobs, &job.account, id)
                    };
                    if dirty
                        .get(&key)
                        .is_some_and(|pending| completed.covers(*pending))
                    {
                        dirty.remove(&key);
                    }
                }
            }
        }
    }
    #[cfg(test)]
    async fn test_phase(&self, phase: PublicationTestPhase) {
        let hook = self.hook.lock().unwrap().clone();
        if let Some(hook) = hook {
            hook(phase).await;
        }
    }
}

async fn run_owner(context: Arc<Context>, core: Arc<Core>, mut receiver: mpsc::Receiver<Job>) {
    let mut pending = VecDeque::new();
    loop {
        let first = match pending.pop_front() {
            Some(job) => job,
            None => match receiver.recv().await {
                Some(job) => job,
                None => break,
            },
        };
        core.stats.queued_jobs.fetch_sub(1, Ordering::Relaxed);
        let mut jobs = vec![first];
        if matches!(
            jobs[0].kind,
            Kind::Reconcile {
                mode: PublicationMode::Reconcile
                    | PublicationMode::DeleteIfAbsent
                    | PublicationMode::Metadata
                    | PublicationMode::AttachmentMetadata
                    | PublicationMode::MetadataAndAttachment
                    | PublicationMode::ReconcileAndAttachment,
                ..
            }
        ) {
            let deadline = tokio::time::Instant::now() + core.options.collection_window;
            let mut documents = jobs[0].ids().len();
            let mut accounts = BTreeSet::from([jobs[0].account.clone()]);
            while jobs.len() < core.options.max_batch_jobs {
                let next = if let Some(next) = pending.pop_front() {
                    Some(next)
                } else if core.options.collection_window.is_zero() {
                    receiver.try_recv().ok()
                } else {
                    tokio::time::timeout_at(deadline, receiver.recv())
                        .await
                        .ok()
                        .flatten()
                };
                let Some(next) = next else { break };
                if !matches!(
                    next.kind,
                    Kind::Reconcile {
                        mode: PublicationMode::Reconcile
                            | PublicationMode::DeleteIfAbsent
                            | PublicationMode::Metadata
                            | PublicationMode::AttachmentMetadata
                            | PublicationMode::MetadataAndAttachment
                            | PublicationMode::ReconcileAndAttachment,
                        ..
                    }
                ) || documents + next.ids().len() > core.options.max_batch_documents
                    || (!accounts.contains(&next.account) && accounts.len() >= 64)
                {
                    pending.push_front(next);
                    break;
                }
                core.stats.queued_jobs.fetch_sub(1, Ordering::Relaxed);
                documents += next.ids().len();
                accounts.insert(next.account.clone());
                jobs.push(next);
            }
        }
        #[cfg(test)]
        core.test_phase(PublicationTestPhase::BeforeExecution).await;
        let result = match &jobs[0].kind {
            Kind::Reconcile {
                mode: PublicationMode::Rebuild,
                ..
            } => {
                if context.search.publication_is_poisoned() {
                    match context.owner().and_then(|owner| context.search.recover_publication_owned(owner).map_err(search_error)) {
                        Ok(()) => reconcile(&context, &core, &jobs, true).await,
                        Err(reason) => Err(reason),
                    }
                } else {
                    reconcile(&context, &core, &jobs, true).await
                }
            }
            Kind::Reconcile { .. } => reconcile(&context, &core, &jobs, false).await,
            Kind::Erase => erase(&context, &core, &jobs[0]).await,
            Kind::FinishErase { expected_epoch } => {
                finish_erasure(&context, &core, &jobs[0], *expected_epoch).await
            }
            Kind::Recover => recover(&context, &core).await,
            Kind::FinishFullRebuild { expected_epoch } => {
                finish_full_rebuild(&context, &core, *expected_epoch).await
            }
        };
        if result.is_err() {
            core.remember_dirty(&jobs);
            core.stats
                .failed
                .fetch_add(jobs.len() as u64, Ordering::Relaxed);
        } else {
            core.stats
                .completed
                .fetch_add(jobs.len() as u64, Ordering::Relaxed);
        }
        #[cfg(test)]
        core.test_phase(PublicationTestPhase::BeforeAcknowledgement)
            .await;
        for mut job in jobs {
            let outcome = match &result {
                Ok(receipt) => Ok(*receipt),
                Err(reason) => Err(error(&reason.to_string())),
            };
            let watermark = job.watermark.take();
            let _ = job.ack.send(outcome);
            // Released after the outcome is final: success is query-visible,
            // and a failure is already recorded as dirty (or poison), which the
            // read fence refuses on.
            drop(job._admission.take());
            if let Some((mark, seq)) = watermark {
                mark.published.fetch_max(seq, Ordering::AcqRel);
                mark.advanced.notify_waiters();
            }
        }
    }
}
fn requests(jobs: &[Job]) -> Vec<IndexPublicationRequest> {
    let mut groups: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for job in jobs {
        groups
            .entry(job.account.clone())
            .or_default()
            .extend(job.ids().iter().cloned());
    }
    groups
        .into_iter()
        .map(|(account_id, ids)| IndexPublicationRequest {
            account_id,
            stable_ids: ids.into_iter().collect(),
        })
        .collect()
}
fn intent_mode(jobs: &[Job], account: &str, id: &str) -> PublicationMode {
    jobs.iter()
        .filter(|job| job.account == account && job.ids().iter().any(|value| value == id))
        .filter_map(|job| match job.kind {
            Kind::Reconcile { mode, .. } => Some(mode),
            _ => None,
        })
        .reduce(PublicationMode::strongest)
        .unwrap_or(PublicationMode::Rebuild)
}
async fn source_snapshot(
    context: &Context,
    requests: &[IndexPublicationRequest],
    metadata_only: bool,
) -> Result<mw_store::index_publication::IndexPublicationSnapshot> {
    let now = chrono::Utc::now().timestamp();
    Ok(if metadata_only {
        context
            .store
            .index_publication_metadata_snapshot(requests, now)
            .await?
    } else {
        context
            .store
            .index_publication_snapshot(requests, now)
            .await?
    })
}
async fn live_postures(
    context: &Context,
    requests: &[IndexPublicationRequest],
) -> BTreeMap<String, mw_cache::AccountPosture> {
    let source = context
        .hooks
        .read()
        .expect("v6 hooks lock")
        .publication_posture_source();
    let mut postures = BTreeMap::new();
    for request in requests {
        postures.insert(
            request.account_id.clone(),
            source.posture_live(&request.account_id).await,
        );
    }
    postures
}
fn current_jobs(jobs: &[Job]) -> Result<()> {
    if jobs.iter().any(|job| {
        job.gate.closed.load(Ordering::Acquire)
            || job.epoch != job.gate.epoch.load(Ordering::Acquire)
    }) {
        return Err(error("publication invalidated by account erasure"));
    }
    Ok(())
}
async fn reconcile(
    context: &Context,
    core: &Core,
    jobs: &[Job],
    recovering: bool,
) -> Result<IndexPublicationReceipt> {
    if !recovering
        && (core.stats.poisoned.load(Ordering::Acquire) || context.search.publication_is_poisoned())
    {
        return Err(error("index owner requires source repair"));
    }
    let mut published = false;
    let outcome = async {
        current_jobs(jobs)?;
        let requests = requests(jobs);
        let mut force_rebuild = jobs.iter().any(|job| {
            matches!(
                job.kind,
                Kind::Reconcile {
                    mode: PublicationMode::Rebuild,
                    ..
                }
            )
        });
        for attempt in 0..core.options.revalidation_attempts {
            current_jobs(jobs)?;
            let metadata_snapshot = !force_rebuild
                && jobs.iter().all(|job| {
                    matches!(
                        job.kind,
                        Kind::Reconcile {
                            mode: PublicationMode::AttachmentMetadata,
                            ..
                        }
                    )
                });
            // Journal protocol step 1: the bound is taken BEFORE the source read,
            // so any change committed later has a higher seq and survives the
            // acknowledgement below (SQLite: seq order is commit order).
            let journal_upto = if core.journal.active.load(Ordering::Acquire) {
                match context.store.index_journal_max_seq().await {
                    Ok(seq) => Some(seq),
                    Err(reason) => {
                        tracing::warn!("index publication journal: bound unavailable, rows kept: {reason}");
                        None
                    }
                }
            } else {
                None
            };
            let snapshot = source_snapshot(context, &requests, metadata_snapshot).await?;
            let postures = live_postures(context, &requests).await;
            let mut changes = Vec::new();
            let mut source_bytes = 0usize;
            let mut prepared_bytes = 0usize;
            let mut considered = 0usize;
            let mut removed = 0u64;
            let mut suppressed = false;
            for request in &requests {
                let account = snapshot
                    .account(&request.account_id)
                    .ok_or_else(|| error("missing publication account observation"))?;
                if !account.exists
                    || account.zero_access
                    || postures.get(&request.account_id)
                        != Some(&mw_cache::AccountPosture::Standard)
                {
                    core.projection_clock.focus_changed(&request.account_id);
                    published = true;
                    removed += context
                        .search
                        .delete_account_physical_owned(context.owner()?, &request.account_id)
                        .map_err(|reason| {
                            context.search.poison_publication();
                            core.stats.poisoned.store(true, Ordering::Release);
                            search_error(reason)
                        })?;
                    suppressed = true;
                    continue;
                }
                if account.indexing_restricted {
                    suppressed = true;
                    continue;
                }
                for id in &request.stable_ids {
                    let observation = snapshot
                        .message(&request.account_id, id)
                        .ok_or_else(|| error("missing publication source observation"))?;
                    let Some(message) = observation.message.as_ref() else {
                        core.projection_clock.focus_changed(&request.account_id);
                        changes.push(mw_search::PublicationChange::Delete {
                            account_id: request.account_id.clone(),
                            stable_id: id.clone(),
                        });
                        continue;
                    };
                    considered += 1;
                    let current = context
                        .search
                        .publication_document(
                            &request.account_id,
                            id,
                            64 * 1024 * 1024,
                            core.options.max_prepared_bytes,
                        )
                        .map_err(search_error)?;
                    let mode = intent_mode(jobs, &request.account_id, id);
                    if mode == PublicationMode::AttachmentMetadata && !force_rebuild {
                        // Repair only this existing index field; do not create an index or
                        // read raw MIME merely because a display projection was upgraded.
                        if let Some(mut doc) = current {
                            let attachment = attachment_flag(
                                context,
                                core,
                                message,
                                observation,
                                &mut source_bytes,
                            )
                            .await?;
                            if doc.has_attachment != attachment {
                                doc.has_attachment = attachment;
                                let bytes = serde_json::to_vec(&doc)
                                    .map_err(|_| error("index document could not be sized"))?;
                                prepared_bytes = prepared_bytes
                                    .checked_add(bytes.len())
                                    .ok_or_else(|| error("index document budget overflow"))?;
                                if prepared_bytes > core.options.max_prepared_bytes {
                                    return Err(error("index prepared-document budget exceeded"));
                                }
                                changes.push(mw_search::PublicationChange::Upsert(Arc::new(doc)));
                            }
                        }
                        continue;
                    }
                    let content = observation
                        .content_blob_ref
                        .clone()
                        .unwrap_or_else(|| format!("envelope:{}", message.cache_revision));
                    let legacy_metadata = matches!(
                        mode,
                        PublicationMode::Metadata | PublicationMode::MetadataAndAttachment
                    ) && !force_rebuild
                        && current
                            .as_ref()
                            .is_some_and(|doc| doc.source_content_ref.is_none());
                    let mut doc = if legacy_metadata
                        || (!force_rebuild
                            && message.internaldate.is_some()
                            && current.as_ref().is_some_and(|doc| {
                                doc.source_content_ref.as_deref() == Some(content.as_str())
                            })) {
                        current.as_ref().unwrap().clone()
                    } else {
                        source_bytes = source_bytes
                            .checked_add(observation.envelope_bytes.unwrap_or(0) as usize)
                            .and_then(|bytes| {
                                bytes.checked_add(observation.raw_body_bytes.unwrap_or(0) as usize)
                            })
                            .ok_or_else(|| error("index source budget overflow"))?;
                        if source_bytes > core.options.max_source_bytes {
                            return Err(error(
                                "index source working-set budget exceeded; source retained",
                            ));
                        }
                        build_document(
                            context,
                            observation,
                            core.options.max_source_bytes as u64,
                            current.as_ref(),
                            &content,
                        )
                        .await?
                    };
                    patch_current(&mut doc, message, observation.meta.as_ref())?;
                    if mode.needs_attachment() && !force_rebuild {
                        doc.has_attachment =
                            attachment_flag(context, core, message, observation, &mut source_bytes)
                                .await?;
                    }
                    if !legacy_metadata {
                        doc.source_content_ref = Some(content);
                    }
                    let encoded = serde_json::to_vec(&doc)
                        .map_err(|_| error("index document could not be sized"))?;
                    prepared_bytes = prepared_bytes
                        .checked_add(encoded.len())
                        .ok_or_else(|| error("index document budget overflow"))?;
                    if prepared_bytes > core.options.max_prepared_bytes {
                        return Err(error(
                            "index prepared-document budget exceeded; source retained",
                        ));
                    }
                    if force_rebuild || current.as_ref() != Some(&doc) {
                        if current.as_ref().is_none_or(|old|old.to!=doc.to || old.cc!=doc.cc
                            || old.mailbox_id!=doc.mailbox_id || old.account_id!=doc.account_id)
                        {core.projection_clock.focus_changed(&request.account_id);}
                        changes.push(mw_search::PublicationChange::Upsert(Arc::new(doc)));
                    }
                }
            }
            #[cfg(test)]
            core.test_phase(PublicationTestPhase::AfterValidation).await;
            current_jobs(jobs)?;
            let publication = if changes.is_empty() {
                None
            } else {
                let receipt = context.search.publish_batch_owned(context.owner()?, &changes).map_err(|reason| {
                    if context.search.publication_is_poisoned() {
                        core.stats.poisoned.store(true, Ordering::Release);
                    }
                    search_error(reason)
                })?;
                published = true;
                core.stats.batches.fetch_add(1, Ordering::Relaxed);
                core.stats
                    .changes
                    .fetch_add(changes.len() as u64, Ordering::Relaxed);
                Some(receipt)
            };
            let after = source_snapshot(context, &requests, metadata_snapshot).await?;
            let after_postures = live_postures(context, &requests).await;
            if snapshot.matches(&after)? && postures == after_postures {
                current_jobs(jobs)?;
                if let Some(upto) = journal_upto {
                    acknowledge_journal(context, core, &requests, upto).await;
                }
                core.complete_dirty(jobs, force_rebuild);
                return Ok(IndexPublicationReceipt {
                    publication,
                    considered_messages: considered,
                    removed_account_documents: removed,
                    suppressed,
                    erasure_epoch: None,
                });
            }
            core.stats.revalidations.fetch_add(1, Ordering::Relaxed);
            // Once an observed source has changed, even canceled recovery must
            // reconstruct it rather than treating the operation as metadata-only.
            core.remember_dirty_as(jobs, Some(PublicationMode::Rebuild));
            force_rebuild = true;
            if attempt + 1 == core.options.revalidation_attempts {
                context.search.poison_publication();
                core.stats.poisoned.store(true, Ordering::Release);
                return Err(error(
                    "source changed during index publication; owner paused pending repair",
                ));
            }
        }
        Err(error("index publication did not converge"))
    }
    .await;
    if outcome.is_err() && published {
        context.search.poison_publication();
        core.stats.poisoned.store(true, Ordering::Release);
    }
    outcome
}
/// Journal protocol step 3, on the owner after a validated index commit. A
/// failure only leaves rows to replay, which reconcile to current source.
async fn acknowledge_journal(
    context: &Context,
    core: &Core,
    requests: &[IndexPublicationRequest],
    upto: i64,
) {
    let keys: Vec<(&str, &str, i64)> = requests
        .iter()
        .flat_map(|request| {
            request
                .stable_ids
                .iter()
                .map(move |id| (request.account_id.as_str(), id.as_str(), upto))
        })
        .collect();
    for chunk in keys.chunks(mw_store::index_publication::MAX_INDEX_JOURNAL_BATCH) {
        match context.store.index_journal_complete(chunk).await {
            Ok(removed) => {
                core.journal.acknowledged.fetch_add(removed, Ordering::Relaxed);
            }
            Err(reason) => {
                core.journal.ack_failures.fetch_add(1, Ordering::Relaxed);
                tracing::warn!("index publication journal: acknowledgement failed, rows kept: {reason}");
            }
        }
    }
}
async fn attachment_flag(
    context: &Context,
    core: &Core,
    message: &mw_store::Message,
    observation: &IndexPublicationMessage,
    source_bytes: &mut usize,
) -> Result<bool> {
    *source_bytes = source_bytes
        .checked_add(observation.envelope_bytes.unwrap_or(0) as usize)
        .ok_or_else(|| error("attachment metadata source budget overflow"))?;
    if *source_bytes > core.options.max_source_bytes {
        return Err(error("attachment metadata source budget exceeded"));
    }
    let envelope = context
        .store
        .index_publication_envelope(
            message,
            core.options.max_source_bytes.min(64 * 1024 * 1024) as u64,
        )
        .await?
        .ok_or_else(|| error("attachment metadata source changed"))?;
    let value: serde_json::Value = serde_json::from_slice(&envelope)
        .map_err(|_| error("attachment metadata envelope is malformed"))?;
    value
        .get("hasAttachment")
        .and_then(serde_json::Value::as_bool)
        .ok_or_else(|| error("attachment metadata flag is malformed"))
}
fn patch_current(
    doc: &mut mw_search::IndexDoc,
    message: &mw_store::Message,
    meta: Option<&mw_store::StoredMeta>,
) -> Result<()> {
    let derived: Vec<String> = doc
        .keywords
        .iter()
        .filter(|keyword| {
            keyword.starts_with("$mw-mention:")
                || keyword.starts_with("$list:")
                || keyword.starts_with("$supersedes:")
                || keyword.as_str() == mw_search::KW_PRIORITY_HIGH
                || keyword.as_str() == mw_search::KW_PRIORITY_LOW
        })
        .cloned()
        .collect();
    let flags = serde_json::from_str::<Vec<crate::backend::Flag>>(&message.flags_json)
        .map_err(|_| error("current source flags are malformed"))?;
    doc.keywords = crate::mapping::flags_to_keywords(&flags)
        .into_keys()
        .collect();
    doc.keywords.extend(derived);
    if meta.is_some_and(|meta| meta.follow_up_at.is_some()) {
        doc.keywords.push(mw_search::KW_FOLLOWUP_ACTIVE.into());
    }
    doc.keywords.sort();
    doc.keywords.dedup();
    doc.pinned = meta.is_some_and(|meta| meta.pinned);
    doc.mailbox_id = message.mailbox_id.clone();
    doc.size = message.size;
    if let Some(date) = message.internaldate.as_deref() {
        doc.date = chrono::DateTime::parse_from_rfc3339(date)
            .map(|date| date.timestamp())
            .unwrap_or(0);
    }
    Ok(())
}
async fn build_document(
    context: &Context,
    observation: &IndexPublicationMessage,
    limit: u64,
    current: Option<&mw_search::IndexDoc>,
    content: &str,
) -> Result<mw_search::IndexDoc> {
    let message = observation
        .message
        .as_ref()
        .ok_or_else(|| error("source is no longer owned"))?;
    let source = context
        .store
        .index_publication_source(message, limit.min(64 * 1024 * 1024))
        .await?
        .ok_or_else(|| error("source changed before index materialization"))?;
    let raw = source.raw;
    let mut email = if let Some(raw) = raw.as_ref() {
        mw_mime::parse(raw)
            .map_err(|_| error("current source MIME is invalid"))?
            .email
    } else if let Some(envelope) = source.envelope.as_ref() {
        serde_json::from_slice::<mw_jmap::Email>(envelope)
            .map_err(|_| error("current source envelope is invalid"))?
    } else {
        return Err(error(
            "index source bytes are unavailable; repair remains pending",
        ));
    };
    email.received_at = message.internaldate.clone().or(email.received_at);
    email.size = message.size;
    if raw.is_none() && email.body_values.is_empty() {
        if let Some(doc) = current.filter(|doc| doc.source_content_ref.as_deref() == Some(content))
        {
            let mut doc = doc.clone();
            doc.has_attachment = email.has_attachment;
            return Ok(doc);
        }
        return Err(error(
            "legacy index content witness needs current MIME; source retained for repair",
        ));
    }
    let mut keywords: Vec<String> =
        crate::mapping::flags_to_keywords(&crate::mapping::flags_from_json(&message.flags_json))
            .into_keys()
            .collect();
    let (filenames, text, calendar, list_signal) = if let Some(raw) = raw.as_ref() {
        keywords.extend(crate::focus::list_id_keyword(raw));
        keywords.extend(crate::focus::supersedes_keyword(raw));
        (
            crate::search_index::attachment_filenames(raw),
            crate::search_index::attachment_text(raw),
            crate::calendar_hint::calendar_hint(raw),
            crate::focus::list_signal(raw),
        )
    } else {
        (Vec::new(), String::new(), None, None)
    };
    Ok(crate::search_index::build_index_doc(
        &message.stable_id,
        &message.account_id,
        &message.mailbox_id,
        &email,
        keywords,
        filenames,
        text,
        observation.meta.as_ref().is_some_and(|meta| meta.pinned),
        calendar,
        list_signal,
    ))
}
async fn erase(context: &Context, core: &Core, job: &Job) -> Result<IndexPublicationReceipt> {
    {
        let mut erasing = core
            .erasing
            .lock()
            .map_err(|_| error("index erasure lock poisoned"))?;
        if erasing.len() >= 10000 && !erasing.contains_key(&job.account) {
            return Err(error("index erasure account limit reached"));
        }
        job.gate.closed.store(true, Ordering::Release);
        job.gate.epoch.fetch_add(1, Ordering::AcqRel);
        erasing.insert(job.account.clone(), job.gate.clone());
    }
    if context.search.publication_is_poisoned() {
        context.search.recover_publication_owned(context.owner()?).map_err(search_error)?;
    }
    let removed = context
        .search
        .delete_account_physical_owned(context.owner()?, &job.account)
        .map_err(|reason| {
            context.search.poison_publication();
            core.stats.poisoned.store(true, Ordering::Release);
            search_error(reason)
        })?;
    Ok(IndexPublicationReceipt {
        publication: None,
        considered_messages: 0,
        removed_account_documents: removed,
        suppressed: true,
        erasure_epoch: Some(job.gate.epoch.load(Ordering::Acquire)),
    })
}
async fn finish_erasure(
    context: &Context,
    core: &Core,
    job: &Job,
    expected_epoch: u64,
) -> Result<IndexPublicationReceipt> {
    if !job.gate.closed.load(Ordering::Acquire)
        || job.gate.epoch.load(Ordering::Acquire) != expected_epoch
    {
        return Err(error("account erasure fence generation changed"));
    }
    let state = context
        .store
        .index_publication_snapshot(
            &[IndexPublicationRequest {
                account_id: job.account.clone(),
                stable_ids: vec![],
            }],
            chrono::Utc::now().timestamp(),
        )
        .await?;
    if state
        .account(&job.account)
        .is_none_or(|account| account.exists)
    {
        return Err(error(
            "authoritative account still exists; erasure fence remains closed",
        ));
    }
    job.gate.epoch.fetch_add(1, Ordering::AcqRel);
    job.gate.closed.store(false, Ordering::Release);
    core.erasing
        .lock()
        .map_err(|_| error("index erasure lock poisoned"))?
        .remove(&job.account);
    Ok(IndexPublicationReceipt {
        publication: None,
        considered_messages: 0,
        removed_account_documents: 0,
        suppressed: true,
        erasure_epoch: None,
    })
}
async fn recover(context: &Context, core: &Core) -> Result<IndexPublicationReceipt> {
    core.projection_clock.invalidate_all();
    core.stats.poisoned.store(true, Ordering::Release);
    if core.stats.requires_full_rebuild.load(Ordering::Acquire) {
        return Err(error(
            "index dirty-key overflow requires a full source rebuild",
        ));
    }
    let receipt = repair_dirty(context, core).await?;
    core.stats.poisoned.store(false, Ordering::Release);
    Ok(receipt)
}
async fn finish_full_rebuild(
    context: &Context,
    core: &Core,
    expected_epoch: u64,
) -> Result<IndexPublicationReceipt> {
    core.projection_clock.invalidate_all();
    core.stats.poisoned.store(true, Ordering::Release);
    if core.stats.repair_epoch.load(Ordering::Acquire) != expected_epoch {
        return Err(error(
            "new index repair obligations arose during full rebuild; repeat reconciliation",
        ));
    }
    let receipt = repair_dirty(context, core).await?;
    if core.stats.repair_epoch.load(Ordering::Acquire) != expected_epoch
        || !core
            .dirty
            .lock()
            .map_err(|_| error("index repair lock poisoned"))?
            .is_empty()
    {
        return Err(error("full rebuild repair changed during completion"));
    }
    core.stats
        .requires_full_rebuild
        .store(false, Ordering::Release);
    core.stats.poisoned.store(false, Ordering::Release);
    Ok(receipt)
}
async fn repair_dirty(context: &Context, core: &Core) -> Result<IndexPublicationReceipt> {
    context.search.recover_publication_owned(context.owner()?).map_err(search_error)?;
    let mut dirty: Vec<((String, String), PublicationMode)> = core
        .dirty
        .lock()
        .map_err(|_| error("index repair lock poisoned"))?
        .iter()
        .map(|(key, mode)| (key.clone(), *mode))
        .collect();
    // Do not let one strict content repair turn a separate legacy metadata
    // obligation into a rebuild. Group only equal intents during recovery.
    dirty.sort_by_key(|(_, mode)| mode.rank());
    let mut remaining = dirty.as_slice();
    while !remaining.is_empty() {
        let mode = remaining[0].1;
        let count = remaining
            .iter()
            .take(core.options.max_batch_documents.min(64))
            .take_while(|(_, candidate)| *candidate == mode)
            .count();
        let (chunk, next) = remaining.split_at(count);
        remaining = next;
        let mut jobs = Vec::with_capacity(chunk.len());
        for ((account, id), mode) in chunk {
            let gate = if let Some(_gate) = core
                .erasing
                .lock()
                .map_err(|_| error("index erasure lock poisoned"))?
                .get(account)
                .cloned()
            {
                context
                    .search
                    .delete_account_physical_owned(context.owner()?, account)
                    .map_err(search_error)?;
                core.dirty
                    .lock()
                    .map_err(|_| error("index repair lock poisoned"))?
                    .remove(&(account.clone(), id.clone()));
                continue;
            } else {
                let mut gates = core
                    .gates
                    .lock()
                    .map_err(|_| error("index gate lock poisoned"))?;
                match gates.get(account).and_then(Weak::upgrade) {
                    Some(gate) => gate,
                    None => {
                        let gate = Arc::new(AccountGate {
                            epoch: AtomicU64::new(0),
                            closed: AtomicBool::new(false),
                        });
                        gates.insert(account.clone(), Arc::downgrade(&gate));
                        gate
                    }
                }
            };
            let (ack, _receiver) = oneshot::channel();
            jobs.push(Job {
                account: account.clone(),
                epoch: gate.epoch.load(Ordering::Acquire),
                gate,
                kind: Kind::Reconcile {
                    ids: vec![id.clone()],
                    mode: match mode {
                        PublicationMode::Metadata
                        | PublicationMode::AttachmentMetadata
                        | PublicationMode::MetadataAndAttachment => *mode,
                        _ => PublicationMode::Rebuild,
                    },
                },
                ack,
                _admission: None,
                watermark: None,
            });
        }
        if !jobs.is_empty() {
            if let Err(reason) = reconcile(context, core, &jobs, true).await {
                context.search.poison_publication();
                return Err(reason);
            }
        }
    }
    Ok(IndexPublicationReceipt {
        publication: None,
        considered_messages: 0,
        removed_account_documents: 0,
        suppressed: false,
        erasure_epoch: None,
    })
}

#[cfg(test)]
#[path = "index_publication_journal_tests.rs"]
mod journal_tests;

#[cfg(test)]
mod intent_contract_tests {
    use super::PublicationMode as Mode;

    #[test]
    fn publication_intents_preserve_both_obligations_in_union_and_completion() {
        let modes = [
            Mode::AttachmentMetadata,
            Mode::Metadata,
            Mode::MetadataAndAttachment,
            Mode::DeleteIfAbsent,
            Mode::Reconcile,
            Mode::ReconcileAndAttachment,
            Mode::Rebuild,
        ];
        for left in modes {
            assert_eq!(left.strongest(left), left);
            for right in modes {
                let union = left.strongest(right);
                assert_eq!(union, right.strongest(left));
                assert!(
                    union.covers(left) && union.covers(right),
                    "{left:?} + {right:?} lost an obligation"
                );
                for third in modes {
                    assert_eq!(
                        left.strongest(right).strongest(third),
                        left.strongest(right.strongest(third))
                    );
                }
            }
        }
        for plain in [Mode::Metadata, Mode::DeleteIfAbsent, Mode::Reconcile] {
            assert!(!plain.covers(Mode::AttachmentMetadata));
            assert!(plain.strongest(Mode::AttachmentMetadata).needs_attachment());
        }
        assert!(!Mode::AttachmentMetadata.covers(Mode::Metadata));
        assert!(!Mode::ReconcileAndAttachment.covers(Mode::Rebuild));
        assert_eq!(
            Mode::Reconcile.strongest(Mode::MetadataAndAttachment),
            Mode::ReconcileAndAttachment
        );
    }
}
