//! Realtime JMAP push (plan §2.2): `/jmap/ws` (WebSocket, RFC 8887) and
//! `/jmap/eventsource` (SSE fallback), both authenticated by the same
//! `mw_session` cookie and both streaming the identical [`StateChange`] wire
//! object produced by [`mw_engine::StateChange::to_wire`].
//!
//! ## Where the frames come from
//! `mw-server` never invents state — it drains a [`broadcast`] channel. In
//! engine mode `build_app` bridges `Engine::subscribe()` (fed by e9's
//! `start_watch` loop) into that channel; tests inject synthetic changes via
//! [`PushHandle`]. Either way the socket loop below is identical, so the wire
//! contract is proven without a live engine.
//!
//! ## Getting through a reverse proxy
//! Both endpoints are long-lived streams, which is exactly the shape a stock
//! proxy default breaks. The SSE response therefore carries
//! `X-Accel-Buffering: no` (nginx reads it and disables response buffering for
//! that one response) and `Cache-Control: no-cache, no-transform` (`no-transform`
//! tells any intermediary not to recompress or otherwise rewrite the body — a
//! compressing proxy holds SSE frames until its buffer fills). The `/jmap/ws`
//! upgrade negotiates the RFC 8887 `jmap` subprotocol when the client offers it.
//!
//! ## Keepalives a client can actually see
//! Both legs emit their idle keepalive as an ordinary `{"@type":"ping"}` message
//! — a `data:` frame on SSE, a text frame on WS — because neither an SSE comment
//! line nor a protocol-level WebSocket Ping reaches browser JavaScript: the DOM
//! surfaces comments and Pings nowhere. The web client's own liveness timer only
//! advances on an observed message, so an invisible keepalive leaves an idle but
//! perfectly healthy session indistinguishable from a dead one. The WS leg still
//! sends the protocol Ping as well, for proxies and native clients that watch it.
//!
//! ## Whose changes a stream carries, and for how long (D-27)
//! The broadcast is process-wide: it carries every account's changes. Each stream
//! filters it at its own subscription ([`ScopedChanges`]) down to the accounts
//! [`stream_accounts`] names for the session that opened it — today the
//! session's own account — so one user never sees another account's id or state
//! tokens. The client has no say in that list.
//!
//! The session is checked once at the upgrade by `authed`, and then again every
//! [`recheck_period`] (`MW_REALTIME_RECHECK_SECS`, default 30 s) by
//! [`still_valid`]: the bearer token's native marker, the session row, its
//! idle/absolute timeouts and the account's flags. A revoked or signed-out
//! session, a revoked bearer token, an expired session, a disabled account, an
//! account held for a password change, or a store that cannot be read ends the
//! stream — a WS close with code 1008, or the end of the SSE body. The re-check
//! does not count as activity, so an open stream does not keep an idle session
//! alive. Between the session ending and the next re-check (at most one period)
//! the stream can still carry that account's own changes; nothing is written to
//! it after a failed re-check.
//!
//! ## Linked child mailboxes (t46)
//! A parent session's stream also carries the changes of its verified linked
//! mailboxes ([`linked_accounts::live_links`]), as a Mailwoman extension frame
//! a stock RFC 8887 client ignores:
//!
//! ```json
//! {"@type":"MailboxChange","link":"<link id>","storageAuthority":"msa1_…",
//!  "changed":{"Email":"…","Mailbox":"…",…},"inboxUnread":3}
//! ```
//!
//! The frame names the link (the selector the client already holds) and the
//! child's opaque storage authority, never the child's raw account id. While a
//! stream carries at least one link, each change of the parent's own account is
//! followed by the same frame with `"link":null`, so the mailbox picker can keep
//! the sign-in mailbox's count live while a child is open. When a link joins
//! the stream (at open, on a link/unlink anywhere, at each re-check) a frame
//! with an empty `changed` reports its current count. Each linked change is
//! verified again ([`linked_accounts::verify_link`]) before it is written.
//!
//! ## Gaps: a stream never skips changes silently (t35)
//! The broadcast holds a bounded backlog per receiver
//! (`MW_REALTIME_FANOUT_CAPACITY`, default 256). A stream that falls further
//! behind than that, for example while a slow client blocks its writes, gets
//! `Lagged` from its receiver: the changes in between are gone from the
//! channel. The stream then reads the current state of every account it
//! carries (its own, and each live link, re-verified as for any linked change)
//! from the store and sends it ([`resync`]). State tokens are absolute, so that
//! one frame per account covers everything the gap skipped. If the engine →
//! server bridge itself lags, [`PushHandle::resync_all`] asks every stream to do
//! the same. A read that fails closes the stream (WS close 1011, end of the SSE
//! body); the client reconnects and runs its own full resync.
//!
//! A frame byte-identical to the last one the stream sent for the same account
//! or link is not sent again (`MW_REALTIME_DEDUPE`), so a resync that finds
//! nothing new writes nothing. `MW_REALTIME_LAG_RESYNC=off` restores the
//! earlier behaviour (a lagged stream continues without catching up) for
//! comparison runs.
//!
//! ## Shutdown
//! [`PushHandle::begin_shutdown`] (or [`begin_shutdown_all`]) closes every WS
//! stream with code 1012 ("service restart") and ends every SSE body, including
//! streams opened after the call. Counters: [`metrics_snapshot`].

use std::collections::{HashMap, VecDeque};
use std::convert::Infallible;
use std::sync::{Arc, Mutex, OnceLock, Weak};
use std::time::Duration;

use axum::extract::State;
use axum::extract::ws::{CloseFrame, Message, WebSocket, WebSocketUpgrade, close_code};
use axum::http::HeaderMap;
use axum::http::header::{CACHE_CONTROL, HeaderName, HeaderValue};
use axum::response::sse::{Event, Sse};
use axum::response::{IntoResponse, Response};
use futures_util::Stream;
use futures_util::future::BoxFuture;
use tokio::sync::{broadcast, watch};
use tokio::time::{Interval, MissedTickBehavior};

use mw_engine::StateChange;
use mw_store::{Session, StoreError};

use crate::linked_accounts::{self, LiveLink};
use crate::{AppState, account_gate, authed, push_relay};

mod metrics;
pub(crate) mod resync;
#[cfg(test)]
mod resync_tests;

pub use self::metrics::ResyncReason;
use self::metrics::{Consumer, StreamGauge, Transport};
use self::resync::{Gap, Hub, ReadFn};

/// The realtime counters since process start (no identities): open streams,
/// lags, resyncs, suppressed duplicates, shutdown closes, backlog high-water.
pub fn metrics_snapshot() -> serde_json::Value {
    self::metrics::snapshot()
}

/// The environment variable that sizes each broadcast channel's backlog.
const FANOUT_ENV: &str = "MW_REALTIME_FANOUT_CAPACITY";
const FANOUT_DEFAULT: usize = 256;
const FANOUT_RANGE: (usize, usize) = (64, 65_536);
/// `on` (default) or `off`: catch a lagged stream up from the store.
const LAG_RESYNC_ENV: &str = "MW_REALTIME_LAG_RESYNC";
/// `on` (default) or `off`: skip a frame identical to the last one sent for the
/// same account or link.
const DEDUPE_ENV: &str = "MW_REALTIME_DEDUPE";
/// How many resync reads may run against the store at once, process-wide.
const RESYNC_CONCURRENCY_ENV: &str = "MW_REALTIME_RESYNC_CONCURRENCY";
const RESYNC_CONCURRENCY_DEFAULT: usize = 16;
const RESYNC_CONCURRENCY_RANGE: (usize, usize) = (1, 1024);

/// How long one stream's resync may take before the stream is closed instead.
const RESYNC_DEADLINE: Duration = Duration::from_secs(10);

/// The reason sent with the WS close frame when a resync could not read state.
const RESYNC_FAILED: &str = "resync failed";
/// The reason sent with the WS close frame at shutdown.
const SERVICE_RESTART: &str = "service restart";

/// The realtime switches, read once per process.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RealtimeConfig {
    pub(crate) fanout_capacity: usize,
    pub(crate) lag_resync: bool,
    pub(crate) dedupe: bool,
    pub(crate) resync_concurrency: usize,
}

impl Default for RealtimeConfig {
    fn default() -> Self {
        Self {
            fanout_capacity: FANOUT_DEFAULT,
            lag_resync: true,
            dedupe: true,
            resync_concurrency: RESYNC_CONCURRENCY_DEFAULT,
        }
    }
}

fn parse_config(get: impl Fn(&str) -> Option<String>) -> Result<RealtimeConfig, String> {
    let number = |name: &str, (lo, hi): (usize, usize), default: usize| match get(name)
        .as_deref()
        .map(str::trim)
    {
        None | Some("") => Ok(default),
        Some(v) => v
            .parse::<usize>()
            .ok()
            .filter(|n| (lo..=hi).contains(n))
            .ok_or_else(|| format!("{name}={v:?} is not a whole number in {lo}..={hi}")),
    };
    let switch = |name: &str| match get(name).as_deref().map(str::trim) {
        None | Some("") | Some("on") => Ok(true),
        Some("off") => Ok(false),
        Some(v) => Err(format!("{name}={v:?} must be on or off")),
    };
    Ok(RealtimeConfig {
        fanout_capacity: number(FANOUT_ENV, FANOUT_RANGE, FANOUT_DEFAULT)?,
        lag_resync: switch(LAG_RESYNC_ENV)?,
        dedupe: switch(DEDUPE_ENV)?,
        resync_concurrency: number(
            RESYNC_CONCURRENCY_ENV,
            RESYNC_CONCURRENCY_RANGE,
            RESYNC_CONCURRENCY_DEFAULT,
        )?,
    })
}

fn env_config() -> Result<RealtimeConfig, String> {
    parse_config(|name| std::env::var(name).ok())
}

/// Refuse to start on an invalid `MW_REALTIME_*` switch; the error names the
/// variable and its range.
pub fn validate_startup() -> Result<(), String> {
    env_config().map(|_| ())
}

/// The process's realtime switches. An invalid value panics with the same
/// message [`validate_startup`] returns: [`PushHandle::new`] runs at start-up,
/// so the process does not start with a switch it cannot honour.
fn config() -> RealtimeConfig {
    static CONFIG: OnceLock<RealtimeConfig> = OnceLock::new();
    *CONFIG.get_or_init(|| env_config().unwrap_or_else(|e| panic!("refusing to start: {e}")))
}

/// How often an idle connection is nudged so proxies do not reap it (§2.2).
const HEARTBEAT: Duration = Duration::from_secs(30);

/// The environment variable that sets how often an open stream re-checks its
/// session, in whole seconds (D-27).
const RECHECK_ENV: &str = "MW_REALTIME_RECHECK_SECS";

/// [`RECHECK_ENV`]'s default, in seconds.
const RECHECK_DEFAULT_SECS: u64 = 30;

/// The reason sent with the WS close frame when a re-check ends a stream.
const SESSION_ENDED: &str = "session ended";

/// The RFC 8887 WebSocket subprotocol token. Echoed back only when the client
/// offered it; see [`negotiated_upgrade`].
const JMAP_SUBPROTOCOL: &str = "jmap";

/// nginx's per-response opt-out from `proxy_buffering on`, its stock default.
/// Other proxies ignore an unknown header, so emitting it unconditionally is
/// safe; the non-nginx recipes in `docs/deploy/proxy/` disable buffering in
/// their own config instead.
const X_ACCEL_BUFFERING: HeaderName = HeaderName::from_static("x-accel-buffering");

/// The idle keepalive payload, identical on both transports. Deliberately the
/// same object the web client already sends on the WS leg, and deliberately not
/// a `StateChange`: a client that does not know the type ignores it, while
/// having *received* it is the whole point.
const PING_FRAME: &str = r#"{"@type":"ping"}"#;

/// A keepalive ticker whose first tick is one full interval away. Ticking at t=0
/// would tell a client nothing it does not already know (it just connected) and
/// would put a keepalive ahead of the first real frame.
fn idle_beat(period: Duration) -> Interval {
    let mut beat = tokio::time::interval_at(tokio::time::Instant::now() + period, period);
    beat.set_missed_tick_behavior(MissedTickBehavior::Skip);
    beat
}

/// A cloneable sender end of the realtime push channel. `mw-server` holds one in
/// [`AppState`]; the engine-bridge and tests both feed it.
///
/// Two independent broadcast channels ride behind one handle: the `ws` channel
/// feeds the realtime `/jmap/ws` + `/jmap/eventsource` sessions, and the `relay`
/// channel feeds the V5 push dispatcher (plan §2.3 — the "second consumer" of the
/// engine `StateChange` broadcast that sends opaque WebPush/UnifiedPush wakes).
/// Keeping them separate means the dispatcher's always-on receiver does NOT
/// inflate the WS/SSE subscriber count [`send`](Self::send) reports — the
/// realtime wire behaviour (and its tests) is byte-identical.
#[derive(Clone)]
pub struct PushHandle {
    ws: broadcast::Sender<StateChange>,
    relay: broadcast::Sender<StateChange>,
    preferences: broadcast::Sender<Option<String>>,
    control: Arc<Control>,
}

/// A request that every stream read its accounts' current state again
/// ([`PushHandle::resync_all`]). `epoch` increases by one per request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResyncSignal {
    pub epoch: u64,
    pub reason: ResyncReason,
}

/// What every stream of one handle shares besides the channels.
struct Control {
    resync: watch::Sender<ResyncSignal>,
    shutdown: watch::Sender<bool>,
    hub: Hub,
    /// Account id → number of open streams carrying it.
    interest: Mutex<HashMap<String, usize>>,
    lag_resync: bool,
    dedupe: bool,
}

/// Every handle's control, for [`begin_shutdown_all`].
fn registry() -> &'static Mutex<Vec<Weak<Control>>> {
    static REGISTRY: OnceLock<Mutex<Vec<Weak<Control>>>> = OnceLock::new();
    REGISTRY.get_or_init(Default::default)
}

/// Close every realtime stream of every [`PushHandle`] in this process: WS with
/// 1012 "service restart", SSE by ending the body. Streams opened afterwards
/// close at once. Returns the number of handles told. For the shutdown path,
/// right after the termination signal and before HTTP graceful shutdown waits
/// for open connections (an SSE body is one).
pub fn begin_shutdown_all() -> usize {
    let mut handles = registry().lock().unwrap_or_else(|e| e.into_inner());
    handles.retain(|weak| weak.strong_count() > 0);
    for control in handles.iter().filter_map(Weak::upgrade) {
        control.shutdown.send_replace(true);
    }
    handles.len()
}

impl PushHandle {
    /// Create a fresh push channel with a bounded backlog (slow WS/SSE clients
    /// lag rather than stall the engine; a lagged stream catches up from the
    /// store, see the module header). Reads the `MW_REALTIME_*` switches.
    pub fn new() -> Self {
        Self::with_config(config())
    }

    pub(crate) fn with_config(config: RealtimeConfig) -> Self {
        let capacity = config.fanout_capacity;
        let (ws, _rx) = broadcast::channel(capacity);
        let (relay, _rx) = broadcast::channel(capacity);
        let (preferences, _) = broadcast::channel(256);
        let control = Arc::new(Control {
            resync: watch::Sender::new(ResyncSignal {
                epoch: 0,
                reason: ResyncReason::Bridge,
            }),
            shutdown: watch::Sender::new(false),
            hub: Hub::new(config.resync_concurrency),
            interest: Mutex::new(HashMap::new()),
            lag_resync: config.lag_resync,
            dedupe: config.dedupe,
        });
        registry()
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(Arc::downgrade(&control));
        Self {
            ws,
            relay,
            preferences,
            control,
        }
    }

    /// Ask every open stream to read its accounts' current state and send it
    /// (for a gap upstream of the streams: the engine bridge lagged, or a
    /// cross-process transport lost its place). Relay consumers can watch the
    /// same signal ([`subscribe_resync`](Self::subscribe_resync)).
    pub fn resync_all(&self, reason: ResyncReason) {
        self::metrics::resync_signal(reason);
        self.control.resync.send_modify(|signal| {
            signal.epoch = signal.epoch.wrapping_add(1);
            signal.reason = reason;
        });
    }

    /// Wakes on each later [`resync_all`](Self::resync_all).
    pub fn subscribe_resync(&self) -> watch::Receiver<ResyncSignal> {
        self.control.resync.subscribe()
    }

    /// Close this handle's streams (WS 1012, SSE end), now and for any stream
    /// opened later.
    pub fn begin_shutdown(&self) {
        self.control.shutdown.send_replace(true);
    }

    /// Whether [`begin_shutdown`](Self::begin_shutdown) was called.
    pub fn shutting_down(&self) -> bool {
        *self.control.shutdown.borrow()
    }

    /// The account ids at least one open stream carries (its own accounts and
    /// its live links), for a transport that only needs those accounts'
    /// changes. Sorted.
    pub fn interest_accounts(&self) -> Vec<String> {
        let interest = self
            .control
            .interest
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let mut accounts: Vec<String> = interest.keys().cloned().collect();
        accounts.sort();
        accounts
    }

    /// Mark a gap detected now: [`current_state`](Self::current_state) with it
    /// answers only from a read that started afterwards.
    #[allow(dead_code)] // for the relay consumers' sweep (t35-e2)
    pub(crate) fn resync_gap(&self) -> Gap {
        self.control.hub.gap()
    }

    /// `account`'s current state through this handle's shared, bounded resync
    /// reads (the read started after `since`).
    #[allow(dead_code)] // for the relay consumers' sweep (t35-e2)
    pub(crate) async fn current_state(
        &self,
        store: &mw_store::Store,
        account: &str,
        since: Gap,
    ) -> Result<StateChange, String> {
        let source = resync::store_reader(store.clone());
        self.control
            .hub
            .read(&source, account, since, RESYNC_DEADLINE)
            .await
    }

    /// Opaque invalidation after a committed preference write. None denotes an
    /// administrator policy change; values are re-read through authenticated APIs.
    pub(crate) fn preferences_changed(&self, account: Option<&str>) {
        let _ = self.preferences.send(account.map(str::to_owned));
    }

    /// Publish a [`StateChange`] to every connected session. Returns the number
    /// of live WS/SSE receivers (0 when nobody is listening — not an error). The
    /// change is also fanned out to the push-relay dispatcher (a separate channel,
    /// so it never changes the reported WS/SSE count).
    pub fn send(&self, change: StateChange) -> usize {
        // Fan out to the opaque-wake dispatcher (plan §2.3). Ignore its receiver
        // count / absence — it is an independent consumer.
        let _ = self.relay.send(change.clone());
        self.ws.send(change).unwrap_or(0)
    }

    /// A new receiver for one WS/SSE session.
    pub fn subscribe(&self) -> broadcast::Receiver<StateChange> {
        self.ws.subscribe()
    }

    /// A new receiver for the push-relay dispatcher (plan §2.3). Distinct from
    /// [`subscribe`](Self::subscribe) so the dispatcher does not count as a
    /// realtime WS/SSE subscriber.
    pub fn subscribe_relay(&self) -> broadcast::Receiver<StateChange> {
        self.relay.subscribe()
    }
}

impl Default for PushHandle {
    fn default() -> Self {
        Self::new()
    }
}

/// The account ids a session's realtime stream may carry changes for. Today that
/// is the session's own account; accounts a session is later authorised for
/// beyond its own join here, so every stream is filtered through this one
/// function. An empty account id names no account (a proxy-mode upstream that
/// reports none) and such a session sees no changes at all.
pub(crate) fn stream_accounts(session: &Session) -> Vec<String> {
    if session.account_id.is_empty() {
        Vec::new()
    } else {
        vec![session.account_id.clone()]
    }
}

/// One stream's subscription to the process-wide broadcast, narrowed to the
/// changes for `accounts`. Changes for any other account are dropped here and
/// never reach the socket.
pub(crate) struct ScopedChanges {
    rx: broadcast::Receiver<StateChange>,
    preferences: Option<broadcast::Receiver<Option<String>>>,
    accounts: Vec<String>,
    /// The parent's linked mailboxes, when the stream carries any (t46).
    linked: Option<LinkedScope>,
    /// Wakes when some user's links changed; taken by the stream loop.
    links_epoch: Option<watch::Receiver<u64>>,
    /// Frames ready to write ahead of the next change.
    pending: VecDeque<String>,
    /// Where a resync reads current state; `None` makes a gap close the stream.
    resync: Option<StreamResync>,
    /// Wakes on [`PushHandle::resync_all`]; taken by the stream loop.
    resync_epoch: Option<watch::Receiver<ResyncSignal>>,
    /// Becomes `true` at shutdown; taken by the stream loop.
    shutdown: Option<watch::Receiver<bool>>,
    lag_resync: bool,
    dedupe: bool,
    /// The last `StateChange` wire string sent per own account, and the last
    /// `changed` object sent per link id (bounded by the stream's accounts and
    /// links), for duplicate suppression.
    last_own: HashMap<String, String>,
    last_link: HashMap<String, String>,
    /// Registers the stream's accounts with [`PushHandle::interest_accounts`].
    interest: Option<InterestGuard>,
}

/// The store reader and shared read hub one stream resyncs through.
struct StreamResync {
    control: Arc<Control>,
    source: ReadFn,
}

/// Holds one stream's accounts in its handle's interest set until dropped.
struct InterestGuard {
    control: Arc<Control>,
    accounts: Vec<String>,
}

impl InterestGuard {
    fn new(control: Arc<Control>, accounts: Vec<String>) -> Self {
        let mut guard = Self {
            control,
            accounts: Vec::new(),
        };
        guard.set(accounts);
        guard
    }

    fn set(&mut self, mut next: Vec<String>) {
        next.sort();
        next.dedup();
        if next == self.accounts {
            return;
        }
        let mut interest = self
            .control
            .interest
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        for account in &next {
            *interest.entry(account.clone()).or_default() += 1;
        }
        for account in &self.accounts {
            release(&mut interest, account);
        }
        drop(interest);
        self.accounts = next;
    }
}

fn release(interest: &mut HashMap<String, usize>, account: &str) {
    if let Some(n) = interest.get_mut(account) {
        *n -= 1;
        if *n == 0 {
            interest.remove(account);
        }
    }
}

impl Drop for InterestGuard {
    fn drop(&mut self) {
        let mut interest = self
            .control
            .interest
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        for account in &self.accounts {
            release(&mut interest, account);
        }
    }
}

/// What one stream needs to deliver its parent's linked mailboxes.
struct LinkedScope {
    state: AppState,
    parent: Session,
    /// The parent's own storage authority, for its `"link":null` frames.
    parent_authority: Option<String>,
    links: Vec<LiveLink>,
    metadata: std::collections::BTreeMap<String, String>,
    personal_owner: Option<String>,
}

/// A change taken off the broadcast for this stream, not yet written.
#[derive(Debug, PartialEq)]
enum Incoming {
    /// A change of one of the session's own accounts.
    Own(StateChange),
    Preferences(Option<String>),
    /// A change of a linked mailbox, still to be re-verified.
    Linked(LiveLink, StateChange),
}

impl ScopedChanges {
    /// A stream with no state source: a gap closes it (see [`attach`](Self::attach)).
    pub(crate) fn new(rx: broadcast::Receiver<StateChange>, accounts: Vec<String>) -> Self {
        let defaults = RealtimeConfig::default();
        Self {
            rx,
            preferences: None,
            accounts,
            linked: None,
            links_epoch: None,
            pending: VecDeque::new(),
            resync: None,
            resync_epoch: None,
            shutdown: None,
            lag_resync: defaults.lag_resync,
            dedupe: defaults.dedupe,
            last_own: HashMap::new(),
            last_link: HashMap::new(),
            interest: None,
        }
    }

    /// Join `push`'s resync and shutdown signals and its interest set, and
    /// resync through `source` after a gap.
    pub(crate) fn attach(mut self, push: &PushHandle, source: Option<ReadFn>) -> Self {
        let control = &push.control;
        self.resync_epoch = Some(control.resync.subscribe());
        self.shutdown = Some(control.shutdown.subscribe());
        self.lag_resync = control.lag_resync;
        self.dedupe = control.dedupe;
        self.resync = source.map(|source| StreamResync {
            control: control.clone(),
            source,
        });
        self.interest = Some(InterestGuard::new(control.clone(), self.interest_set()));
        self
    }

    /// The accounts this stream carries: its own and its live links'.
    fn interest_set(&self) -> Vec<String> {
        let mut accounts = self.accounts.clone();
        if let Some(scope) = &self.linked {
            accounts.extend(scope.links.iter().map(|link| link.account_id.clone()));
        }
        accounts
    }

    /// A stream for `parent` that also carries its verified linked mailboxes.
    /// The link set is resolved here, and the current count of each link is
    /// queued as the first frames.
    pub(crate) async fn with_links(
        rx: broadcast::Receiver<StateChange>,
        accounts: Vec<String>,
        state: AppState,
        parent: Session,
    ) -> Self {
        let mut changes = Self::new(rx, accounts)
            .attach(&state.push, Some(resync::store_reader(state.store.clone())));
        // Subscribed before the first resolution, so a link made meanwhile is
        // not missed.
        changes.links_epoch = Some(linked_accounts::subscribe_links());
        changes.preferences = Some(state.push.preferences.subscribe());
        let parent_authority = state
            .store
            .session_storage_authority(&parent.id)
            .await
            .ok()
            .flatten();
        let personal_owner = state
            .store
            .session_personal_subject(&parent.id)
            .await
            .ok()
            .flatten();
        changes.linked = Some(LinkedScope {
            state,
            parent,
            parent_authority,
            links: Vec::new(),
            metadata: Default::default(),
            personal_owner,
        });
        changes.refresh_links().await;
        changes
    }

    /// The next change for one of this stream's accounts or links. Cancel-safe,
    /// as [`broadcast::Receiver::recv`] is: a change for another account is
    /// dropped whether or not the caller is still waiting.
    async fn recv(&mut self) -> Result<Incoming, broadcast::error::RecvError> {
        loop {
            let change = tokio::select! {
                change = self.rx.recv() => {
                    let change = change?;
                    self::metrics::observe_backlog(self.rx.len());
                    change
                }
                event = async {match self.preferences.as_mut(){Some(rx)=>rx.recv().await,None=>std::future::pending().await}} => {
                    // A slow receiver catches up its own settings; no foreign
                    // account id or setting value ever goes onto its wire.
                    let target=match event {
                        Ok(target)=>target,
                        Err(broadcast::error::RecvError::Lagged(_))=>None,
                        Err(broadcast::error::RecvError::Closed)=>{self.preferences=None;continue;}
                    };
                    if target.as_ref().is_none_or(|account|self.accounts.contains(account)||self.linked.as_ref().is_some_and(|scope|scope.links.iter().any(|link|&link.account_id==account))) {
                        return Ok(Incoming::Preferences(target));
                    }
                    continue;
                }
            };
            if self.accounts.contains(&change.account_id) {
                return Ok(Incoming::Own(change));
            }
            if let Some(link) = self.linked.as_ref().and_then(|scope| {
                scope
                    .links
                    .iter()
                    .find(|link| link.account_id == change.account_id)
            }) {
                return Ok(Incoming::Linked(link.clone(), change));
            }
        }
    }

    /// Queue the frames one incoming change produces. A linked change is
    /// re-verified first; one that no longer verifies is dropped and its link
    /// leaves the stream until a later resolution admits it again.
    async fn admit(&mut self, incoming: Incoming) {
        match incoming {
            Incoming::Preferences(target) => {
                let token = uuid::Uuid::new_v4().simple().to_string();
                if target
                    .as_ref()
                    .is_none_or(|account| self.accounts.contains(account))
                {
                    for account in &self.accounts {
                        self.pending.push_back(serde_json::json!({"@type":"StateChange","changed":{account:{"Settings":token}}}).to_string());
                    }
                }
                if let Some(scope) = &self.linked {
                    for link in &scope.links {
                        if target
                            .as_ref()
                            .is_some_and(|account| account != &link.account_id)
                        {
                            continue;
                        }
                        if linked_accounts::verify_link(&scope.state, &scope.parent, &link.link)
                            .await
                            .as_ref()
                            != Some(link)
                        {
                            continue;
                        }
                        self.pending.push_back(serde_json::json!({"@type":"MailboxChange","link":link.link,"storageAuthority":link.storage_authority,"changed":{"Settings":token}}).to_string());
                    }
                }
            }
            Incoming::Own(change) => {
                let wire = change.to_wire().to_string();
                if self.dedupe {
                    if self.last_own.get(&change.account_id) == Some(&wire) {
                        self::metrics::duplicate_suppressed();
                        return;
                    }
                    self.last_own
                        .insert(change.account_id.clone(), wire.clone());
                }
                self.pending.push_back(wire);
                if let Some(scope) = &self.linked
                    && !scope.links.is_empty()
                    && let Some(authority) = &scope.parent_authority
                {
                    let unread =
                        linked_accounts::inbox_unread(&scope.state, &change.account_id).await;
                    self.pending
                        .push_back(mailbox_change(None, authority, Some(&change), unread));
                }
            }
            Incoming::Linked(link, change) => {
                let Some(scope) = self.linked.as_mut() else {
                    return;
                };
                let changed = change.to_wire()["changed"][&change.account_id].to_string();
                if self.dedupe && self.last_link.get(&link.link) == Some(&changed) {
                    self::metrics::duplicate_suppressed();
                    return;
                }
                let current =
                    linked_accounts::verify_link(&scope.state, &scope.parent, &link.link).await;
                if current.as_ref() != Some(&link) {
                    scope.links.retain(|l| l.link != link.link);
                    self.last_link.remove(&link.link);
                    return;
                }
                if self.dedupe {
                    self.last_link.insert(link.link.clone(), changed);
                }
                let unread = linked_accounts::inbox_unread(&scope.state, &link.account_id).await;
                self.pending.push_back(mailbox_change(
                    Some(&link.link),
                    &link.storage_authority,
                    Some(&change),
                    unread,
                ));
            }
        }
    }

    /// Resolve the link set again. Each link that joins queues a counts-only
    /// frame; when the first link joins, so does the parent's own mailbox.
    async fn refresh_links(&mut self) {
        let Some(scope) = self.linked.as_mut() else {
            return;
        };
        // Only opaque state tokens leave this authenticated account's stream.
        // Seed the initial baseline without emitting spurious changes.
        let mut changed = serde_json::Map::new();
        if !scope.metadata.contains_key("Settings") {
            let token = uuid::Uuid::new_v4().simple().to_string();
            scope.metadata.insert("Settings".into(), token.clone());
            changed.insert("Settings".into(), token.into());
        }
        // A reconnect also carries the current web build so an old open tab
        // notices a server restart/deploy without waiting for its slow poll.
        let build = crate::client_policy::web_build_id(scope.state.web_dir.as_deref());
        if scope
            .metadata
            .insert("WebVersion".into(), build.clone())
            .as_deref()
            != Some(build.as_str())
        {
            changed.insert("WebVersion".into(), build.into());
        }
        if let Ok(mut rows) = scope
            .state
            .store
            .list_security_sessions(Some(&scope.parent.account_id), 100)
            .await
        {
            rows.sort_by(|a, b| a.handle.cmp(&b.handle));
            let snapshot: Vec<_> = rows.iter().map(|row| (&row.handle, &row.device)).collect();
            if let Ok(json) = serde_json::to_string(&snapshot) {
                let token = crate::push_relay::hash_token(&json);
                if scope
                    .metadata
                    .insert("Session".into(), token.clone())
                    .is_some_and(|old| old != token)
                {
                    changed.insert("Session".into(), token.into());
                }
            }
        }
        if let Some(owner) = &scope.personal_owner {
            if let Ok(mut rows) = scope
                .state
                .store
                .personal_records(owner, "notification-center:v1:")
                .await
            {
                rows.sort();
                if let Ok(json) = serde_json::to_string(&rows) {
                    let token = crate::push_relay::hash_token(&json);
                    if scope
                        .metadata
                        .insert("Notification".into(), token.clone())
                        .is_some_and(|old| old != token)
                    {
                        changed.insert("Notification".into(), token.into());
                    }
                }
            }
        }
        if !changed.is_empty() {
            self.pending.push_back(serde_json::json!({"@type":"StateChange", "changed":{scope.parent.account_id.clone():changed}}).to_string());
        }
        let next = linked_accounts::live_links(&scope.state, &scope.parent).await;
        let had_none = scope.links.is_empty();
        if had_none
            && !next.is_empty()
            && let Some(authority) = &scope.parent_authority
        {
            let unread =
                linked_accounts::inbox_unread(&scope.state, &scope.parent.account_id).await;
            self.pending
                .push_back(mailbox_change(None, authority, None, unread));
        }
        for link in &next {
            if !scope.links.contains(link) {
                let unread = linked_accounts::inbox_unread(&scope.state, &link.account_id).await;
                self.pending.push_back(mailbox_change(
                    Some(&link.link),
                    &link.storage_authority,
                    None,
                    unread,
                ));
            }
        }
        scope.links = next;
        let live: Vec<&str> = scope.links.iter().map(|l| l.link.as_str()).collect();
        self.last_link
            .retain(|link, _| live.contains(&link.as_str()));
        let accounts = self.interest_set();
        if let Some(interest) = self.interest.as_mut() {
            interest.set(accounts);
        }
    }

    /// Read the current state of every account this stream carries (own
    /// accounts and live links) and queue it, as if each had just changed: a
    /// linked one is re-verified first and identical frames are suppressed. The
    /// reads start after this call begins, so they cover every change skipped
    /// before it. An error means the stream must close.
    async fn resync(&mut self, reason: ResyncReason) -> Result<(), String> {
        self::metrics::resync(reason);
        let Some((control, source)) = self
            .resync
            .as_ref()
            .map(|r| (r.control.clone(), r.source.clone()))
        else {
            return Err("this stream has no state source".into());
        };
        // Everything still queued for this stream was committed before the
        // reads below start, so the reads cover it: drop it rather than send it
        // after the newer state (a stale token would make the client refetch
        // for nothing). Bounded, so a busy channel cannot hold the stream here.
        for _ in 0..self.rx.len().saturating_add(1).min(1 << 17) {
            match self.rx.try_recv() {
                Ok(_) | Err(broadcast::error::TryRecvError::Lagged(_)) => {}
                Err(_) => break,
            }
        }
        let gap = control.hub.gap();
        let deadline = tokio::time::Instant::now() + RESYNC_DEADLINE;
        let links = self
            .linked
            .as_ref()
            .map(|scope| scope.links.clone())
            .unwrap_or_default();
        let wanted: Vec<String> = self
            .accounts
            .iter()
            .cloned()
            .chain(links.iter().map(|link| link.account_id.clone()))
            .collect();
        let reads = wanted.iter().map(|account| {
            let left = deadline.saturating_duration_since(tokio::time::Instant::now());
            control.hub.read(&source, account, gap, left)
        });
        let mut current = futures_util::future::try_join_all(reads).await?.into_iter();
        for _ in 0..self.accounts.len() {
            if let Some(change) = current.next() {
                self.admit(Incoming::Own(change)).await;
            }
        }
        for (link, change) in links.into_iter().zip(current) {
            self.admit(Incoming::Linked(link, change)).await;
        }
        self::metrics::observe_pending(self.pending.len());
        Ok(())
    }

    /// [`resync`](Self::resync), logging and counting a failure. `false`: close.
    async fn catch_up(&mut self, reason: ResyncReason) -> bool {
        match self.resync(reason).await {
            Ok(()) => true,
            Err(why) => {
                self::metrics::resync_failure();
                tracing::info!(
                    "realtime stream closed: {} resync failed: {why}",
                    reason.as_str()
                );
                false
            }
        }
    }

    /// What the stream loop does after its receiver reported a gap of
    /// `skipped` changes: `false` means close.
    async fn lagged(&mut self, consumer: Consumer, skipped: u64) -> bool {
        self::metrics::lagged(consumer, skipped);
        !self.lag_resync || self.catch_up(ResyncReason::Lag).await
    }

    /// The same for a [`PushHandle::resync_all`] signal.
    async fn resync_requested(&mut self, reason: ResyncReason) -> bool {
        !self.lag_resync || self.catch_up(reason).await
    }

    /// The next queued frame, if any.
    fn pop_pending(&mut self) -> Option<String> {
        self.pending.pop_front()
    }
}

/// The `MailboxChange` extension frame (see the module header). `change` is
/// `None` for a counts-only frame.
fn mailbox_change(
    link: Option<&str>,
    storage_authority: &str,
    change: Option<&StateChange>,
    inbox_unread: Option<u32>,
) -> String {
    let changed = change
        .map(|c| c.to_wire()["changed"][&c.account_id].clone())
        .unwrap_or_else(|| serde_json::json!({}));
    let mut frame = serde_json::json!({
        "@type": "MailboxChange",
        "link": link,
        "storageAuthority": storage_authority,
        "changed": changed,
    });
    if let Some(n) = inbox_unread {
        frame["inboxUnread"] = n.into();
    }
    frame.to_string()
}

/// Resolves when `epoch` reports that some user's links changed; never when the
/// stream has no epoch (no linked delivery) or the sender is gone.
async fn links_moved(epoch: &mut Option<watch::Receiver<u64>>) {
    if let Some(rx) = epoch
        && rx.changed().await.is_ok()
    {
        return;
    }
    std::future::pending().await
}

/// Resolves with the reason of the next [`PushHandle::resync_all`]; never when
/// the stream has no signal.
async fn resync_signalled(signal: &mut Option<watch::Receiver<ResyncSignal>>) -> ResyncReason {
    if let Some(rx) = signal
        && rx.changed().await.is_ok()
    {
        return rx.borrow_and_update().reason;
    }
    std::future::pending().await
}

/// Resolves once shutdown has begun, at once if it already has.
async fn shutdown_begun(shutdown: &mut Option<watch::Receiver<bool>>) {
    if let Some(rx) = shutdown
        && rx.wait_for(|down| *down).await.is_ok()
    {
        return;
    }
    std::future::pending().await
}

/// How often an open stream re-checks its session: [`RECHECK_ENV`] read once
/// per process, at least one second, [`RECHECK_DEFAULT_SECS`] when unset or not
/// a whole number.
fn recheck_period() -> Duration {
    static PERIOD: OnceLock<Duration> = OnceLock::new();
    *PERIOD.get_or_init(|| parse_recheck(std::env::var(RECHECK_ENV).ok().as_deref()))
}

fn parse_recheck(raw: Option<&str>) -> Duration {
    let secs = match raw.map(str::trim) {
        None | Some("") => RECHECK_DEFAULT_SECS,
        Some(v) => v.parse::<u64>().map(|n| n.max(1)).unwrap_or_else(|_| {
            tracing::warn!(
                "{RECHECK_ENV}={v:?} is not a whole number of seconds; using {RECHECK_DEFAULT_SECS}"
            );
            RECHECK_DEFAULT_SECS
        }),
    };
    Duration::from_secs(secs)
}

/// Decides, every `period`, whether an open stream may stay open.
#[derive(Clone)]
struct StreamGuard {
    period: Duration,
    check: Arc<dyn Fn() -> BoxFuture<'static, bool> + Send + Sync>,
}

impl StreamGuard {
    /// The guard for a stream opened by `session`; `bearer` says whether it was
    /// presented as a native bearer token rather than a cookie.
    fn for_session(state: AppState, session: Session, bearer: bool) -> Self {
        Self {
            period: recheck_period(),
            check: Arc::new(move || {
                let (state, session) = (state.clone(), session.clone());
                Box::pin(async move { still_valid(&state, &session, bearer).await })
            }),
        }
    }

    async fn holds(&self) -> bool {
        (self.check)().await
    }
}

/// Is `session` still good for a realtime stream? Re-reads what `authed` read at
/// the upgrade, without recording activity. Any read that fails says no (fail
/// closed).
async fn still_valid(state: &AppState, session: &Session, bearer: bool) -> bool {
    match recheck(state, session, bearer).await {
        Ok(()) => true,
        Err(why) => {
            tracing::info!("realtime stream for {} ended: {why}", session.username);
            false
        }
    }
}

async fn recheck(state: &AppState, session: &Session, bearer: bool) -> Result<(), &'static str> {
    const UNREADABLE: &str = "the store could not be read";
    if bearer {
        match state
            .store
            .get_native_session(&push_relay::hash_token(&session.id))
            .await
        {
            Ok(Some(_)) => {}
            Ok(None) => return Err("bearer token revoked"),
            Err(_) => return Err(UNREADABLE),
        }
    }
    let current = match state.store.get_session(&session.id).await {
        Ok(current) => current,
        Err(StoreError::NotFound) => return Err("session revoked"),
        Err(_) => return Err(UNREADABLE),
    };
    if current.account_id != session.account_id {
        return Err("session changed account");
    }
    // Not `SessionGuard::check`: that records activity, and an open stream is
    // not activity. The next request with the session deletes it, as `authed`
    // does for any expired session.
    if state
        .sessions
        .expired(&session.id, &state.hardening)
        .is_some()
    {
        return Err("session expired");
    }
    // Request telemetry and a process-local clock cannot extend the persisted
    // foreground-activity deadline, including after restarts or policy changes.
    crate::security_center::enforce_session(state, &current)
        .await
        .map_err(|_| "session expired or policy unavailable")?;
    let gate = account_gate::for_session(&state.store, &current)
        .await
        .map_err(|_| UNREADABLE)?;
    if gate.disabled {
        return Err("account disabled");
    }
    if gate.password_change_required {
        return Err("password change required");
    }
    Ok(())
}

/// The scoped subscription and the guard for a stream `session` is opening.
async fn open_stream(
    state: &AppState,
    headers: &HeaderMap,
    session: Session,
) -> (ScopedChanges, StreamGuard) {
    // The same test `authed_with_gate` uses to pick the bearer path.
    let bearer = push_relay::bearer_token(headers).is_some();
    let changes = ScopedChanges::with_links(
        state.push.subscribe(),
        stream_accounts(&session),
        state.clone(),
        session.clone(),
    )
    .await;
    (
        changes,
        StreamGuard::for_session(state.clone(), session, bearer),
    )
}

/// Forward every `Engine` broadcast into the server push channel for the life of
/// the process. Spawned once by `build_app` in engine mode. When the bridge
/// itself falls behind, every stream is asked to resync
/// ([`PushHandle::resync_all`]), since the lost changes never reached any of
/// them.
pub(crate) async fn bridge_engine(mut src: broadcast::Receiver<StateChange>, out: PushHandle) {
    loop {
        match src.recv().await {
            Ok(change) => {
                out.send(change);
            }
            Err(broadcast::error::RecvError::Lagged(n)) => {
                self::metrics::lagged(Consumer::Bridge, n);
                tracing::warn!("push bridge lagged {n} engine changes; streams resync");
                if out.control.lag_resync {
                    out.resync_all(ResyncReason::Bridge);
                }
            }
            Err(broadcast::error::RecvError::Closed) => break,
        }
    }
}

// ---------------------------------------------------------------------------
// WebSocket (RFC 8887)
// ---------------------------------------------------------------------------

/// `GET /jmap/ws` — authenticate via cookie or native bearer *before* upgrading,
/// then stream the session's own `StateChange` frames until the session ends.
/// An unauthenticated request never upgrades (401).
pub(crate) async fn jmap_ws(
    State(state): State<AppState>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> Response {
    let session = match authed(&state, &headers).await {
        Ok(session) => session,
        Err(resp) => return resp,
    };
    let (changes, guard) = open_stream(&state, &headers, session).await;
    negotiated_upgrade(ws, changes, HEARTBEAT, guard)
}

/// Complete the upgrade, offering the RFC 8887 `jmap` subprotocol.
///
/// [`WebSocketUpgrade::protocols`] selects from what the *client* offered, so
/// the echo happens only for a client that asked for `jmap`. A client that
/// offers nothing, or offers only tokens we do not speak, still upgrades with no
/// `Sec-WebSocket-Protocol` in the response — which is what every browser and
/// the pre-existing in-tree clients expect. Making the subprotocol mandatory
/// would break all of them.
fn negotiated_upgrade(
    ws: WebSocketUpgrade,
    changes: ScopedChanges,
    heartbeat: Duration,
    guard: StreamGuard,
) -> Response {
    ws.protocols([JMAP_SUBPROTOCOL])
        // t63: explicit limits instead of tungstenite's defaults (64 MiB message,
        // 16 MiB frame, unbounded write buffer). The loop reads nothing but
        // control frames (ping/pong/close, <= 125 bytes); a text frame such as
        // an RFC 8887 WebSocketPushEnable (a few hundred bytes) is read and
        // ignored. A larger message ends the socket (`recv` error -> break).
        .max_message_size(PUSH_WS_MAX_MESSAGE)
        .max_frame_size(PUSH_WS_MAX_MESSAGE)
        // The write buffer grows past its 128 KiB default only while writes
        // fail; past this bound a send errors, which ends the loop.
        .max_write_buffer_size(PUSH_WS_MAX_WRITE_BUFFER)
        .on_upgrade(move |socket| ws_loop(socket, changes, heartbeat, guard))
}

/// Largest client message on /jmap/ws (see `negotiated_upgrade`).
pub(crate) const PUSH_WS_MAX_MESSAGE: usize = 4 * 1024;
/// Most bytes queued for one slow client before the socket is closed.
pub(crate) const PUSH_WS_MAX_WRITE_BUFFER: usize = 1024 * 1024;

/// Pump the stream's changes to the socket, answer nothing but pings/closes from
/// the client, and keep alive every `heartbeat`. Exits cleanly on any I/O error,
/// and with close code 1008 when a re-check finds the session no longer valid.
async fn ws_loop(
    mut socket: WebSocket,
    mut changes: ScopedChanges,
    heartbeat: Duration,
    guard: StreamGuard,
) {
    let _gauge = StreamGauge::open(Transport::Ws);
    let mut beat = idle_beat(heartbeat);
    let mut recheck = idle_beat(guard.period);
    let mut links_epoch = changes.links_epoch.take();
    let mut resync_epoch = changes.resync_epoch.take();
    let mut shutdown = changes.shutdown.take();
    let resync_failed = || CloseFrame {
        code: close_code::ERROR,
        reason: RESYNC_FAILED.into(),
    };
    loop {
        // Queued frames first: linked counts at open, and the frames a change
        // or a link resolution produced.
        while let Some(frame) = changes.pop_pending() {
            if socket.send(Message::Text(frame.into())).await.is_err() {
                return;
            }
            beat.reset(); // a real frame is itself proof of life
        }
        tokio::select! {
            _ = shutdown_begun(&mut shutdown) => {
                self::metrics::shutdown_close();
                let frame = CloseFrame {
                    code: close_code::RESTART,
                    reason: SERVICE_RESTART.into(),
                };
                let _ = socket.send(Message::Close(Some(frame))).await;
                return;
            }
            incoming = socket.recv() => match incoming {
                Some(Ok(Message::Close(_))) | None => break,
                Some(Ok(_)) => {}            // pong/ping/text/binary: ignore (axum auto-pongs)
                Some(Err(_)) => break,
            },
            change = changes.recv() => match change {
                Ok(incoming) => {
                    changes.admit(incoming).await;
                    self::metrics::observe_pending(changes.pending.len());
                }
                Err(broadcast::error::RecvError::Lagged(skipped)) => {
                    if !changes.lagged(Consumer::Ws, skipped).await {
                        let _ = socket.send(Message::Close(Some(resync_failed()))).await;
                        return;
                    }
                }
                Err(broadcast::error::RecvError::Closed) => break,
            },
            reason = resync_signalled(&mut resync_epoch) => {
                if !changes.resync_requested(reason).await {
                    let _ = socket.send(Message::Close(Some(resync_failed()))).await;
                    return;
                }
            }
            _ = links_moved(&mut links_epoch) => changes.refresh_links().await,
            _ = beat.tick() => {
                // Protocol Ping first (proxies and native clients watch for it),
                // then the text frame that is the only half browser JS can see.
                if socket.send(Message::Ping(Vec::new().into())).await.is_err() {
                    break;
                }
                if socket.send(Message::Text(PING_FRAME.into())).await.is_err() {
                    break;
                }
            }
            _ = recheck.tick() => {
                if !guard.holds().await {
                    let frame = CloseFrame {
                        code: close_code::POLICY,
                        reason: SESSION_ENDED.into(),
                    };
                    let _ = socket.send(Message::Close(Some(frame))).await;
                    return;
                }
                changes.refresh_links().await;
            }
        }
    }
    let _ = socket.send(Message::Close(None)).await;
}

// ---------------------------------------------------------------------------
// EventSource / SSE fallback
// ---------------------------------------------------------------------------

/// `GET /jmap/eventsource` — the SSE fallback. Same auth, same scope and the
/// same re-check as the WS leg, the session's `StateChange` JSON emitted as
/// `data:` frames, plus a `data:` keepalive every [`HEARTBEAT`].
pub(crate) async fn jmap_eventsource(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Response {
    let session = match authed(&state, &headers).await {
        Ok(session) => session,
        Err(resp) => return resp,
    };
    let (changes, guard) = open_stream(&state, &headers, session).await;
    sse_response(changes, HEARTBEAT, guard)
}

/// Build the SSE response, overriding the two headers a stock reverse proxy
/// otherwise gets wrong for a long-lived stream.
///
/// `axum` already sets `Cache-Control: no-cache`; the `insert` widens it to
/// `no-cache, no-transform` so an intermediary is also told not to recompress
/// the body, which is the other common way SSE frames get held back.
fn sse_response(changes: ScopedChanges, heartbeat: Duration, guard: StreamGuard) -> Response {
    let mut resp = Sse::new(change_stream(changes, heartbeat, guard)).into_response();
    let headers = resp.headers_mut();
    headers.insert(
        CACHE_CONTROL,
        HeaderValue::from_static("no-cache, no-transform"),
    );
    headers.insert(X_ACCEL_BUFFERING, HeaderValue::from_static("no"));
    resp
}

/// Turn the stream's changes into an SSE `Event` stream, catching up after a
/// gap (see the module header) and ending when the channel closes, a resync
/// cannot read state, shutdown begins, or a re-check finds the session no
/// longer valid.
///
/// The keepalive is emitted here rather than through [`axum::response::sse::KeepAlive`]
/// because that helper writes an SSE *comment* (`:keep-alive`), which the
/// `EventSource` API drops on the floor — see the module header. A `data:` frame
/// costs the same bytes and is observable.
fn change_stream(
    changes: ScopedChanges,
    heartbeat: Duration,
    guard: StreamGuard,
) -> impl Stream<Item = Result<Event, Infallible>> + Send {
    struct Sse {
        changes: ScopedChanges,
        beat: Interval,
        recheck: Interval,
        guard: StreamGuard,
        links_epoch: Option<watch::Receiver<u64>>,
        resync_epoch: Option<watch::Receiver<ResyncSignal>>,
        shutdown: Option<watch::Receiver<bool>>,
        _gauge: StreamGauge,
    }
    let mut changes = changes;
    let state = Sse {
        recheck: idle_beat(guard.period),
        beat: idle_beat(heartbeat),
        guard,
        links_epoch: changes.links_epoch.take(),
        resync_epoch: changes.resync_epoch.take(),
        shutdown: changes.shutdown.take(),
        changes,
        _gauge: StreamGauge::open(Transport::Sse),
    };
    futures_util::stream::unfold(state, |mut s| async move {
        loop {
            // Queued frames first: linked counts at open, and the frames a
            // change or a link resolution produced.
            if let Some(frame) = s.changes.pop_pending() {
                s.beat.reset(); // a real frame is itself proof of life
                let ev = Event::default().data(frame);
                return Some((Ok(ev), s));
            }
            tokio::select! {
                _ = shutdown_begun(&mut s.shutdown) => {
                    self::metrics::shutdown_close();
                    return None;
                }
                // `ScopedChanges::recv` is cancel-safe, so losing this branch
                // of the select drops no change for this stream.
                change = s.changes.recv() => match change {
                    Ok(incoming) => {
                        s.changes.admit(incoming).await;
                        self::metrics::observe_pending(s.changes.pending.len());
                    }
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        if !s.changes.lagged(Consumer::Sse, skipped).await {
                            return None;
                        }
                    }
                    Err(broadcast::error::RecvError::Closed) => return None,
                },
                reason = resync_signalled(&mut s.resync_epoch) => {
                    if !s.changes.resync_requested(reason).await {
                        return None;
                    }
                }
                _ = links_moved(&mut s.links_epoch) => s.changes.refresh_links().await,
                _ = s.beat.tick() => {
                    let ev = Event::default().data(PING_FRAME);
                    return Some((Ok(ev), s));
                }
                _ = s.recheck.tick() => {
                    if !s.guard.holds().await {
                        return None;
                    }
                    s.changes.refresh_links().await;
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use axum::Router;
    use axum::routing::get;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;

    use super::*;

    /// A keepalive period short enough to observe inside a test. The production
    /// value is [`HEARTBEAT`]; both endpoints take it as a parameter precisely so
    /// the idle behaviour is testable without a 30 s wait.
    const TEST_BEAT: Duration = Duration::from_millis(60);

    /// Serve `app` on an ephemeral loopback port. These routers deliberately omit
    /// `AppState` and the `authed` guard: what is under test is the wire
    /// behaviour of the two response builders, which the real handlers call
    /// unchanged once authentication has passed.
    async fn spawn(app: Router) -> SocketAddr {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        addr
    }

    /// A guard that always holds. The re-check is the integration tests' subject
    /// (`tests/t32_h13_realtime_scope.rs`); here it must not interfere.
    fn guard_holding() -> StreamGuard {
        StreamGuard {
            period: Duration::from_secs(3600),
            check: Arc::new(|| Box::pin(async { true })),
        }
    }

    /// A guard that holds `n` times and then fails, re-checking every 50 ms.
    fn guard_failing_after(n: usize) -> StreamGuard {
        let left = Arc::new(AtomicUsize::new(n));
        StreamGuard {
            period: Duration::from_millis(50),
            check: Arc::new(move || {
                let left = left.clone();
                Box::pin(async move {
                    left.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |v| v.checked_sub(1))
                        .is_ok()
                })
            }),
        }
    }

    /// The stream the test endpoints serve: scoped to [`change`]'s account.
    fn scoped(push: &PushHandle) -> ScopedChanges {
        ScopedChanges::new(push.subscribe(), vec!["acct1".into()])
    }

    async fn spawn_sse(push: PushHandle) -> SocketAddr {
        spawn_sse_guarded(push, guard_holding()).await
    }

    async fn spawn_sse_guarded(push: PushHandle, guard: StreamGuard) -> SocketAddr {
        let app = Router::new().route(
            "/jmap/eventsource",
            get(move || {
                let (push, guard) = (push.clone(), guard.clone());
                async move { sse_response(scoped(&push), TEST_BEAT, guard) }
            }),
        );
        spawn(app).await
    }

    async fn spawn_ws(push: PushHandle) -> SocketAddr {
        spawn_ws_guarded(push, guard_holding()).await
    }

    async fn spawn_ws_guarded(push: PushHandle, guard: StreamGuard) -> SocketAddr {
        let app = Router::new().route(
            "/jmap/ws",
            get(move |ws: WebSocketUpgrade| {
                let (push, guard) = (push.clone(), guard.clone());
                async move { negotiated_upgrade(ws, scoped(&push), TEST_BEAT, guard) }
            }),
        );
        spawn(app).await
    }

    /// Send one raw HTTP/1.1 request head and return the (lowercased) response
    /// head plus the still-open socket. Raw bytes rather than a client library so
    /// the assertions are about what actually goes on the wire.
    async fn raw_request(addr: SocketAddr, head: &str) -> (String, TcpStream) {
        let mut sock = TcpStream::connect(addr).await.unwrap();
        sock.write_all(head.as_bytes()).await.unwrap();
        let mut buf = Vec::new();
        let mut byte = [0u8; 1];
        while !buf.ends_with(b"\r\n\r\n") {
            let n = tokio::time::timeout(Duration::from_secs(5), sock.read(&mut byte))
                .await
                .expect("response head arrives")
                .unwrap();
            assert_eq!(n, 1, "connection closed mid response head");
            buf.push(byte[0]);
        }
        (String::from_utf8(buf).unwrap().to_ascii_lowercase(), sock)
    }

    /// A well-formed RFC 6455 upgrade head, optionally offering subprotocols.
    fn upgrade_head(addr: SocketAddr, protocols: Option<&str>) -> String {
        let offer = protocols
            .map(|p| format!("Sec-WebSocket-Protocol: {p}\r\n"))
            .unwrap_or_default();
        format!(
            "GET /jmap/ws HTTP/1.1\r\n\
             Host: {addr}\r\n\
             Connection: Upgrade\r\n\
             Upgrade: websocket\r\n\
             Sec-WebSocket-Version: 13\r\n\
             Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n\
             {offer}\r\n"
        )
    }

    /// Read from `sock` until `needle` shows up in the accumulated body.
    async fn read_until(sock: &mut TcpStream, needle: &str) -> String {
        let mut body = String::new();
        let mut chunk = [0u8; 2048];
        let outcome = tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                let n = sock.read(&mut chunk).await.unwrap();
                assert!(n > 0, "stream closed before {needle:?} arrived");
                body.push_str(&String::from_utf8_lossy(&chunk[..n]));
                if body.contains(needle) {
                    return;
                }
            }
        })
        .await;
        assert!(outcome.is_ok(), "timed out waiting for {needle:?}");
        body
    }

    fn change() -> StateChange {
        change_for("acct1")
    }

    fn change_for(account_id: &str) -> StateChange {
        StateChange {
            account_id: account_id.into(),
            email: "7".into(),
            mailbox: "3".into(),
            submission: "1".into(),
            thread: "7".into(),
            crypto_key: "2".into(),
            mail_rule: "1".into(),
        }
    }

    #[tokio::test]
    async fn push_handle_delivers_to_subscribers() {
        let h = PushHandle::new();
        let mut a = h.subscribe();
        let mut b = h.subscribe();
        assert_eq!(h.send(change()), 2);
        assert_eq!(a.recv().await.unwrap(), change());
        assert_eq!(b.recv().await.unwrap(), change());
    }

    #[tokio::test]
    async fn send_with_no_subscribers_is_not_an_error() {
        let h = PushHandle::new();
        assert_eq!(h.send(change()), 0);
    }

    #[tokio::test]
    async fn preference_changes_are_private_opaque_and_survive_receiver_lag() {
        let h = PushHandle::new();
        let mut mine = ScopedChanges::new(h.subscribe(), vec!["acct1".into()]);
        mine.preferences = Some(h.preferences.subscribe());
        h.preferences_changed(Some("other-account"));
        h.preferences_changed(Some("acct1"));
        let event = mine.recv().await.unwrap();
        assert_eq!(event, Incoming::Preferences(Some("acct1".into())));
        mine.admit(event).await;
        let frame: serde_json::Value =
            serde_json::from_str(&mine.pending.pop_front().unwrap()).unwrap();
        assert!(frame["changed"]["acct1"]["Settings"].is_string());
        assert_eq!(frame["changed"]["acct1"].as_object().unwrap().len(), 1);
        assert!(!frame.to_string().contains("other-account"));
        h.preferences_changed(None);
        assert_eq!(mine.recv().await.unwrap(), Incoming::Preferences(None));
        for _ in 0..300 {
            h.preferences_changed(Some("other-account"));
        }
        assert_eq!(mine.recv().await.unwrap(), Incoming::Preferences(None));
    }

    #[tokio::test]
    async fn scoped_changes_drop_other_accounts() {
        let h = PushHandle::new();
        let mut mine = ScopedChanges::new(h.subscribe(), vec!["acct1".into()]);
        h.send(change_for("acct2"));
        h.send(change_for(""));
        h.send(change_for("acct1"));
        assert_eq!(
            mine.recv().await.unwrap(),
            Incoming::Own(change_for("acct1"))
        );
    }

    #[test]
    fn a_mailbox_change_names_the_link_and_authority_not_the_account() {
        let frame: serde_json::Value = serde_json::from_str(&mailbox_change(
            Some("link-1"),
            "msa1_x",
            Some(&change_for("raw-child-account")),
            Some(4),
        ))
        .unwrap();
        assert_eq!(frame["@type"], "MailboxChange");
        assert_eq!(frame["link"], "link-1");
        assert_eq!(frame["storageAuthority"], "msa1_x");
        assert_eq!(frame["changed"]["Email"], "7");
        assert_eq!(frame["changed"]["Mailbox"], "3");
        assert_eq!(frame["inboxUnread"], 4);
        assert!(
            !frame.to_string().contains("raw-child-account"),
            "the raw account id never reaches the wire: {frame}"
        );
        let counts: serde_json::Value =
            serde_json::from_str(&mailbox_change(None, "msa1_p", None, None)).unwrap();
        assert_eq!(counts["link"], serde_json::Value::Null);
        assert_eq!(counts["changed"], serde_json::json!({}));
        assert!(counts.get("inboxUnread").is_none());
    }

    #[test]
    fn an_empty_account_id_is_scoped_to_nothing() {
        let session = |account_id: &str| Session {
            id: "s".into(),
            account_id: account_id.into(),
            username: "u".into(),
            jmap_url: String::new(),
            api_url: String::new(),
            credentials: mw_store::Credentials {
                username: "u".into(),
                password: "p".into(),
            },
        };
        assert!(stream_accounts(&session("")).is_empty());
        assert_eq!(
            stream_accounts(&session("acct1")),
            vec!["acct1".to_string()]
        );
    }

    #[test]
    fn recheck_period_is_whole_seconds_with_a_floor_of_one() {
        let secs = |raw| parse_recheck(raw).as_secs();
        assert_eq!(secs(None), RECHECK_DEFAULT_SECS);
        assert_eq!(secs(Some("")), RECHECK_DEFAULT_SECS);
        assert_eq!(secs(Some("5")), 5);
        assert_eq!(secs(Some(" 7 ")), 7);
        assert_eq!(secs(Some("0")), 1);
        assert_eq!(secs(Some("-3")), RECHECK_DEFAULT_SECS);
        assert_eq!(secs(Some("1.5")), RECHECK_DEFAULT_SECS);
    }

    #[tokio::test]
    async fn a_failed_recheck_ends_the_sse_body() {
        let addr = spawn_sse_guarded(PushHandle::new(), guard_failing_after(1)).await;
        let (_head, mut sock) = raw_request(
            addr,
            &format!("GET /jmap/eventsource HTTP/1.1\r\nHost: {addr}\r\n\r\n"),
        )
        .await;
        // A chunked body ends with its zero-length last chunk.
        read_until(&mut sock, "\r\n0\r\n\r\n").await;
    }

    #[tokio::test]
    async fn a_failed_recheck_closes_the_ws_with_policy_violation() {
        use futures_util::StreamExt;
        use tokio_tungstenite::tungstenite::Message as Frame;

        let addr = spawn_ws_guarded(PushHandle::new(), guard_failing_after(1)).await;
        let (mut ws, _) = tokio_tungstenite::connect_async(format!("ws://{addr}/jmap/ws"))
            .await
            .unwrap();
        let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
        loop {
            let msg = tokio::time::timeout_at(deadline, ws.next())
                .await
                .expect("the close arrives");
            match msg {
                Some(Ok(Frame::Close(Some(frame)))) => {
                    assert_eq!(u16::from(frame.code), 1008);
                    assert_eq!(frame.reason.as_str(), SESSION_ENDED);
                    return;
                }
                Some(Ok(Frame::Ping(_) | Frame::Text(_))) => continue,
                other => panic!("expected a policy close, got {other:?}"),
            }
        }
    }

    #[tokio::test]
    async fn bridge_forwards_engine_changes() {
        let (engine_tx, engine_rx) = broadcast::channel(8);
        let out = PushHandle::new();
        let mut sink = out.subscribe();
        tokio::spawn(bridge_engine(engine_rx, out.clone()));
        engine_tx.send(change()).unwrap();
        assert_eq!(sink.recv().await.unwrap(), change());
    }

    // -----------------------------------------------------------------------
    // SSE: proxy headers + observable keepalive
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn sse_carries_the_proxy_buffering_headers() {
        let addr = spawn_sse(PushHandle::new()).await;
        let (head, _sock) = raw_request(
            addr,
            &format!("GET /jmap/eventsource HTTP/1.1\r\nHost: {addr}\r\n\r\n"),
        )
        .await;

        assert!(head.starts_with("http/1.1 200"), "head was: {head}");
        assert!(
            head.contains("content-type: text/event-stream"),
            "head was: {head}"
        );
        // Nginx's stock `proxy_buffering on` batches SSE frames without this.
        assert!(head.contains("x-accel-buffering: no"), "head was: {head}");
        // `no-transform` on top of axum's `no-cache`: a recompressing
        // intermediary holds frames until its buffer fills.
        assert!(
            head.contains("cache-control: no-cache, no-transform"),
            "head was: {head}"
        );
        assert!(
            !head.contains("cache-control: no-cache\r\n"),
            "the narrower axum default must have been replaced, not appended: {head}"
        );
    }

    #[tokio::test]
    async fn sse_idle_keepalive_is_a_data_frame_not_a_comment() {
        let addr = spawn_sse(PushHandle::new()).await;
        let (_head, mut sock) = raw_request(
            addr,
            &format!("GET /jmap/eventsource HTTP/1.1\r\nHost: {addr}\r\n\r\n"),
        )
        .await;

        // Nothing is broadcast; the only traffic can be the idle keepalive.
        let body = read_until(&mut sock, PING_FRAME).await;
        assert!(
            body.lines().any(|l| l == format!("data: {PING_FRAME}")),
            "keepalive must be a `data:` line an EventSource can see, got: {body:?}"
        );
        assert!(
            !body.contains(":keep-alive"),
            "the invisible comment keepalive should be gone, got: {body:?}"
        );
    }

    #[tokio::test]
    async fn sse_still_streams_state_changes() {
        let push = PushHandle::new();
        let addr = spawn_sse(push.clone()).await;
        let (_head, mut sock) = raw_request(
            addr,
            &format!("GET /jmap/eventsource HTTP/1.1\r\nHost: {addr}\r\n\r\n"),
        )
        .await;

        // The handler subscribes before it returns the head, so the subscriber
        // is live by now.
        assert_eq!(push.send(change()), 1);
        let body = read_until(&mut sock, "StateChange").await;
        assert!(
            body.lines()
                .any(|l| l.starts_with("data: {") && l.contains("StateChange")),
            "change must still arrive as a data: frame, got: {body:?}"
        );
    }

    // -----------------------------------------------------------------------
    // WebSocket: RFC 8887 subprotocol negotiation + observable keepalive
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn ws_echoes_the_jmap_subprotocol_and_pings_observably() {
        use futures_util::StreamExt;
        use tokio_tungstenite::tungstenite::Message as Frame;
        use tokio_tungstenite::tungstenite::client::IntoClientRequest;

        let addr = spawn_ws(PushHandle::new()).await;
        let mut req = format!("ws://{addr}/jmap/ws")
            .into_client_request()
            .unwrap();
        req.headers_mut()
            .insert("sec-websocket-protocol", JMAP_SUBPROTOCOL.parse().unwrap());

        // tungstenite enforces RFC 6455 §4.1: a client that offered a
        // subprotocol and gets none back fails the connection. So reaching
        // `unwrap` at all is the negotiation assertion; the header check below
        // pins which token was chosen.
        let (mut ws, resp) = tokio_tungstenite::connect_async(req).await.unwrap();
        assert_eq!(
            resp.headers().get("sec-websocket-protocol").unwrap(),
            JMAP_SUBPROTOCOL
        );

        let mut saw_protocol_ping = false;
        let mut saw_text_ping = false;
        let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
        while !(saw_protocol_ping && saw_text_ping) {
            let msg = tokio::time::timeout_at(deadline, ws.next())
                .await
                .expect("keepalives arrive")
                .expect("stream stays open")
                .expect("no ws error");
            match msg {
                Frame::Ping(_) => saw_protocol_ping = true,
                Frame::Text(t) => {
                    assert_eq!(t.as_str(), PING_FRAME, "unexpected text frame");
                    saw_text_ping = true;
                }
                other => panic!("unexpected frame: {other:?}"),
            }
        }
    }

    #[tokio::test]
    async fn ws_upgrade_without_a_subprotocol_offer_is_accepted() {
        let addr = spawn_ws(PushHandle::new()).await;
        let (head, _sock) = raw_request(addr, &upgrade_head(addr, None)).await;

        assert!(head.starts_with("http/1.1 101"), "head was: {head}");
        // Echoing a protocol the client never offered would make a browser fail
        // the connection.
        assert!(!head.contains("sec-websocket-protocol"), "head was: {head}");
    }

    #[tokio::test]
    async fn ws_upgrade_offering_only_an_unknown_subprotocol_still_connects() {
        let addr = spawn_ws(PushHandle::new()).await;
        let (head, _sock) = raw_request(addr, &upgrade_head(addr, Some("soap, wamp"))).await;

        assert!(head.starts_with("http/1.1 101"), "head was: {head}");
        assert!(
            !head.contains("sec-websocket-protocol"),
            "we must not claim a protocol we do not speak: {head}"
        );
    }
}
