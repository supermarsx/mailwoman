//! Remote-image fetch limits (t56): per-message and per-account concurrency, a
//! short negative cache for failed fetches, and the per-message load outcome
//! counts the reader's loading toast polls.
//!
//! All three are per replica and in memory, like the rate limiter and the image
//! cache they sit next to. None of them retains a URL, a session id or a message
//! id in the clear: keys are SHA-256 digests of length-prefixed parts.
//!
//! # Concurrency
//! The global ceiling (`MAX_CONCURRENT`, with a bounded waiting queue) bounds the
//! replica. These two bound one reader: a newsletter with sixty images may hold at
//! most `privacy.maxConcurrentPerMessage` upstream fetches at a time, and one
//! account across all its messages and tabs at most
//! `privacy.maxConcurrentPerAccount`. Over either cap a fetch **waits** for a slot
//! rather than failing, within a separate queue allowance plus the configured
//! network total. Network work still has `privacy.fetchTotalTimeoutMs` at most.
//! The message slot is taken before the account slot, always in that order,
//! so two fetches cannot hold one each and wait on the other.
//!
//! # Negative cache
//! A fetch that went to the network and failed — timed out, failed upstream,
//! answered a non-success status, or exceeded the size cap — is remembered for
//! [`NEGATIVE_TTL`] per (account, URL). A repeat inside that window answers the
//! same failure at once, without a slot, a rate token or a connection, so a frame
//! re-rendering, a second tab or a reload does not retry a dead host in a loop.
//! Policy refusals (blocked address, bad URL) are not cached: they cost no
//! network and are already immediate. `POST /api/rc/sign` with `"retry": true`
//! forgets the entries for the URLs it signs.

use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use super::super::capacity::Permit;
use mw_egress::{FetchTimeouts, Refusal};
use sha2::{Digest, Sha256};

/// How long a failed fetch is answered from memory.
pub(crate) const NEGATIVE_TTL: Duration = Duration::from_secs(10);
/// Most negative entries kept; the oldest go first.
const NEGATIVE_CAPACITY: usize = 2048;
/// Most messages with load outcomes kept, and URLs per message.
const OUTCOME_MESSAGES: usize = 1024;
const OUTCOME_URLS: usize = 256;

/// The effective limits for one fetch, from `privacy.*` (see `remote_policy`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FetchLimits {
    pub concurrency_mode: super::super::adaptive::Mode,
    pub image_bytes: usize,
    pub runtime: Option<super::super::adaptive::Runtime>,
    pub timeouts: FetchTimeouts,
    pub queue_timeout: Duration,
    pub decode_timeout: Duration,
    pub global: usize,
    pub pipeline: usize,
    pub decodes: usize,
    pub queued: usize,
    pub per_message: usize,
    pub per_account: usize,
}

impl Default for FetchLimits {
    fn default() -> Self {
        Self {
            concurrency_mode: super::super::adaptive::Mode::Auto,
            image_bytes: 8 * 1024 * 1024,
            runtime: None,
            timeouts: FetchTimeouts::REMOTE_IMAGE,
            queue_timeout: Duration::from_secs(60),
            decode_timeout: Duration::from_secs(10),
            global: 128,
            pipeline: 128,
            decodes: 10,
            queued: 2048,
            per_message: 30,
            per_account: 128,
        }
    }
}

impl FetchLimits {
    pub(crate) fn ceilings(self) -> super::super::adaptive::Limits {
        super::super::adaptive::Limits {
            per_message: self.per_message,
            per_account: self.per_account,
            global: self.global,
            pipeline: self.pipeline,
            decoders: self.decodes,
        }
    }
    /// One absolute bound for admission, fetching and image processing. Queue
    /// time adds headroom; it never widens an individual network phase.
    pub(crate) fn overall_budget(self) -> Duration {
        self.queue_timeout
            .saturating_add(self.timeouts.total)
            .saturating_add(self.decode_timeout)
    }

    /// Admission ends before the reserved network budget begins. Derive this
    /// from the original overall deadline, never from the time a phase starts.
    pub(crate) fn queue_deadline(self, overall_deadline: Instant) -> Instant {
        overall_deadline
            .checked_sub(self.timeouts.total.saturating_add(self.decode_timeout))
            .unwrap_or_else(Instant::now)
    }

    pub(crate) fn network_timeouts(self, deadline: Instant) -> Option<FetchTimeouts> {
        let remaining = deadline
            .saturating_duration_since(Instant::now())
            .saturating_sub(self.decode_timeout);
        if remaining.is_zero() {
            return None;
        }
        Some(FetchTimeouts {
            total: self.timeouts.total.min(remaining),
            ..self.timeouts
        })
    }
}

fn digest(parts: &[&str]) -> [u8; 32] {
    let mut hash = Sha256::new();
    for part in parts {
        hash.update((part.len() as u64).to_be_bytes());
        hash.update(part.as_bytes());
    }
    hash.finalize().into()
}

// ── concurrency ───────────────────────────────────────────────────────────────

use super::super::slots::Slots;

fn slots() -> &'static Mutex<Slots> {
    static SLOTS: OnceLock<Mutex<Slots>> = OnceLock::new();
    SLOTS.get_or_init(|| Mutex::new(Slots::default()))
}

/// A held message slot and account slot; both are released on drop.
pub(crate) struct FetchSlot {
    _message: Permit,
    _account: Permit,
}

/// Wait, until `deadline`, for a slot for `message` of `account`. `None` when the
/// deadline passed first.
pub(crate) async fn acquire(
    account: &str,
    message: &str,
    limits: &FetchLimits,
    deadline: Instant,
) -> Option<FetchSlot> {
    let revision = limits.runtime.map_or(0, |runtime| runtime.revision);
    let message_limit = super::super::adaptive::message_limit(
        if limits.runtime.is_some() {
            usize::MAX
        } else {
            limits.per_message
        },
        super::super::demand::count(&digest(&["image-count", account, message])),
    );
    let account_limit = if limits.runtime.is_some() {
        usize::MAX
    } else {
        limits.per_account
    };
    let (message_sem, account_sem) = {
        let mut slots = slots().lock().expect("fetch slot lock");
        (
            slots.get(
                digest(&["m", account, message]),
                message_limit,
                false,
                revision,
            ),
            slots.get(digest(&["a", account]), account_limit, true, revision),
        )
    };
    let deadline = tokio::time::Instant::from_std(deadline);
    let message = tokio::time::timeout_at(deadline, message_sem.acquire())
        .await
        .ok()?
        .ok()?;
    let account = tokio::time::timeout_at(deadline, account_sem.acquire())
        .await
        .ok()?
        .ok()?;
    Some(FetchSlot {
        _message: message,
        _account: account,
    })
}

pub(crate) fn register_images(account: &str, message: &str, count: usize) {
    super::super::demand::record(digest(&["image-count", account, message]), count);
}

pub(crate) fn advance(runtime: super::super::adaptive::Runtime) {
    slots().lock().expect("fetch slot lock").advance(runtime);
}

pub(crate) fn waiting() -> usize {
    slots().lock().expect("fetch slot lock").waiting()
}

// ── negative cache ────────────────────────────────────────────────────────────

/// A remembered failure. Only network failures are cached.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Failure {
    Timeout,
    Upstream,
    Status(u16),
    TooLarge,
}

impl Failure {
    /// The failure to remember for `refusal`, or `None` for one not to cache.
    pub(crate) fn of(refusal: &Refusal) -> Option<Failure> {
        match refusal {
            Refusal::Timeout => Some(Failure::Timeout),
            Refusal::Upstream => Some(Failure::Upstream),
            Refusal::Status(status) => Some(Failure::Status(*status)),
            Refusal::TooLarge => Some(Failure::TooLarge),
            Refusal::Blocked | Refusal::BadRequest(_) => None,
        }
    }

    /// Preserve upstream status so cached permanent failures never auto-retry.
    pub(crate) fn refusal(self) -> Refusal {
        match self {
            Failure::Timeout => Refusal::Timeout,
            Failure::Upstream => Refusal::Upstream,
            Failure::Status(status) => Refusal::Status(status),
            Failure::TooLarge => Refusal::TooLarge,
        }
    }
}

#[derive(Default)]
struct Negative(HashMap<[u8; 32], (Instant, Instant, Failure)>);

impl Negative {
    #[cfg(test)]
    fn get(&mut self, key: &[u8; 32], now: Instant) -> Option<Failure> {
        self.get_for(key, now, NEGATIVE_TTL)
    }
    fn get_for(&mut self, key: &[u8; 32], now: Instant, ttl: Duration) -> Option<Failure> {
        match self.0.get(key) {
            Some((created, until, failure))
                if *until > now && now.saturating_duration_since(*created) < ttl =>
            {
                Some(*failure)
            }
            Some(_) => {
                self.0.remove(key);
                None
            }
            None => None,
        }
    }

    #[cfg(test)]
    fn put(&mut self, key: [u8; 32], failure: Failure, now: Instant) {
        self.put_for(key, failure, now, NEGATIVE_TTL);
    }
    fn put_for(&mut self, key: [u8; 32], failure: Failure, now: Instant, ttl: Duration) {
        if ttl.is_zero() {
            self.0.remove(&key);
            return;
        }
        if self.0.len() >= NEGATIVE_CAPACITY {
            self.0.retain(|_, (_, until, _)| *until > now);
        }
        if self.0.len() >= NEGATIVE_CAPACITY
            && let Some(oldest) = self
                .0
                .iter()
                .min_by_key(|(_, (_, until, _))| *until)
                .map(|(k, _)| *k)
        {
            self.0.remove(&oldest);
        }
        self.0.insert(key, (now, now + ttl, failure));
    }
}

fn negative() -> &'static Mutex<Negative> {
    static NEGATIVE: OnceLock<Mutex<Negative>> = OnceLock::new();
    NEGATIVE.get_or_init(|| Mutex::new(Negative::default()))
}

fn negative_key(account: &str, url: &str) -> [u8; 32] {
    digest(&["n", account, url])
}

/// A failure remembered for `url` of `account`, if one is live.
#[cfg(test)]
pub(crate) fn remembered(account: &str, url: &str) -> Option<Failure> {
    remembered_for(account, url, NEGATIVE_TTL)
}
pub(crate) fn remembered_for(account: &str, url: &str, ttl: Duration) -> Option<Failure> {
    negative().lock().expect("negative cache lock").get_for(
        &negative_key(account, url),
        Instant::now(),
        ttl,
    )
}

/// Remember `refusal` for `url` of `account` when it is a network failure.
#[cfg(test)]
pub(crate) fn remember(account: &str, url: &str, refusal: &Refusal) {
    remember_for(account, url, refusal, NEGATIVE_TTL);
}
pub(crate) fn remember_for(account: &str, url: &str, refusal: &Refusal, ttl: Duration) {
    if let Some(failure) = Failure::of(refusal) {
        negative().lock().expect("negative cache lock").put_for(
            negative_key(account, url),
            failure,
            Instant::now(),
            ttl,
        );
    }
}

/// Forget every remembered failure (the egress routes changed).
pub(crate) fn forget_all() {
    negative().lock().expect("negative cache lock").0.clear();
}

/// Forget the failures remembered for `urls` of `account` (the retry path).
pub(crate) fn forget<'a>(account: &str, urls: impl IntoIterator<Item = &'a str>) {
    let mut negative = negative().lock().expect("negative cache lock");
    for url in urls {
        negative.0.remove(&negative_key(account, url));
    }
}

// ── load outcomes (the reader's loading toast, t55) ───────────────────────────

#[derive(Default)]
struct Message {
    /// URL digest -> (loaded?, expires).
    urls: HashMap<[u8; 32], (bool, i64)>,
    retryable: HashSet<[u8; 32]>,
}

#[derive(Default)]
struct Outcomes(HashMap<[u8; 32], Message>);

impl Outcomes {
    fn prune(&mut self, now: i64) {
        self.0.retain(|_, m| {
            m.urls.retain(|_, (_, expires)| *expires > now);
            m.retryable.retain(|url| m.urls.contains_key(url));
            !m.urls.is_empty()
        });
    }

    fn record(&mut self, key: [u8; 32], url: [u8; 32], loaded: bool, expires: i64, now: i64) {
        self.prune(now);
        if expires <= now {
            return;
        }
        if !self.0.contains_key(&key)
            && self.0.len() >= OUTCOME_MESSAGES
            && let Some(oldest) = self
                .0
                .iter()
                .min_by_key(|(_, m)| m.urls.values().map(|(_, e)| *e).max().unwrap_or(0))
                .map(|(k, _)| *k)
        {
            self.0.remove(&oldest);
        }
        let message = self.0.entry(key).or_default();
        if message.urls.contains_key(&url) || message.urls.len() < OUTCOME_URLS {
            message.urls.insert(url, (loaded, expires));
            message.retryable.remove(&url);
        }
    }

    fn read(&mut self, key: &[u8; 32], now: i64) -> (usize, usize) {
        self.prune(now);
        self.0.get(key).map_or((0, 0), |m| {
            let loaded = m.urls.values().filter(|(ok, _)| *ok).count();
            (loaded, m.urls.len() - loaded)
        })
    }
}

fn outcomes() -> &'static Mutex<Outcomes> {
    static OUTCOMES: OnceLock<Mutex<Outcomes>> = OnceLock::new();
    OUTCOMES.get_or_init(|| Mutex::new(Outcomes::default()))
}

/// Record that `url` of the session's message loaded (`true`) or failed. A URL
/// counts once; its latest outcome wins, so a retried image moves to loaded.
pub(crate) fn record_outcome(
    account: &str,
    session: &str,
    message: &str,
    url: &str,
    loaded: bool,
    expires: i64,
) {
    outcomes().lock().expect("outcome lock").record(
        digest(&["o", account, session, message]),
        Sha256::digest(url.as_bytes()).into(),
        loaded,
        expires,
        chrono::Utc::now().timestamp(),
    );
}

/// Opaque reference shared with the signed-image list; never retains a clear URL.
pub(crate) fn image_reference(url: &str) -> String {
    Sha256::digest(url.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub(crate) fn retryable_refusal(refusal: &Refusal) -> bool {
    matches!(
        refusal,
        Refusal::Timeout | Refusal::Upstream | Refusal::Status(408 | 429 | 500..=599)
    )
}

pub(crate) fn record_retryable_failure(
    account: &str,
    session: &str,
    message: &str,
    url: &str,
    expires: i64,
) {
    let key = digest(&["o", account, session, message]);
    let url = Sha256::digest(url.as_bytes()).into();
    let mut all = outcomes().lock().expect("outcome lock");
    all.record(key, url, false, expires, chrono::Utc::now().timestamp());
    if let Some(m) = all.0.get_mut(&key)
        && m.urls.contains_key(&url)
    {
        m.retryable.insert(url);
    }
}

/// Snapshot of distinct outcomes and eligible failed references, scoped as counts are.
pub(crate) fn outcome_details(
    account: &str,
    session: &str,
    message: &str,
) -> (usize, usize, Vec<String>, Vec<String>) {
    let key = digest(&["o", account, session, message]);
    let mut all = outcomes().lock().expect("outcome lock");
    let (loaded, failed) = all.read(&key, chrono::Utc::now().timestamp());
    let mut failures = Vec::new();
    let mut retryable = Vec::new();
    if let Some(m) = all.0.get(&key) {
        for (url, (ok, _)) in &m.urls {
            if !ok {
                let reference = url.iter().map(|b| format!("{b:02x}")).collect::<String>();
                if m.retryable.contains(url) {
                    retryable.push(reference.clone());
                }
                failures.push(reference);
            }
        }
    }
    failures.sort();
    retryable.sort();
    (loaded, failed, failures, retryable)
}

/// Forget the recorded outcomes of `urls` for the session's message (t67).
/// Only explicit retry starts a new load for these accepted targets. Ordinary
/// signing retains outcomes: a browser may keep the same image URLs without
/// fetching them again. Existing token expiry and authority scoping still apply.
pub(crate) fn forget_outcomes<'a>(
    account: &str,
    session: &str,
    message: &str,
    urls: impl IntoIterator<Item = &'a str>,
) {
    let mut all = outcomes().lock().expect("outcome lock");
    if let Some(m) = all.0.get_mut(&digest(&["o", account, session, message])) {
        for url in urls {
            let key: [u8; 32] = Sha256::digest(url.as_bytes()).into();
            m.urls.remove(&key);
            m.retryable.remove(&key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retry_references_are_scoped_distinct_and_only_transient_failures_auto_retry() {
        for status in [400, 401, 403, 404, 413] {
            assert!(!retryable_refusal(&Refusal::Status(status)));
        }
        for status in [408, 429, 500, 502, 503, 504] {
            assert!(retryable_refusal(&Refusal::Status(status)));
        }
        assert!(!retryable_refusal(&Refusal::Blocked));
        assert!(!retryable_refusal(&Refusal::TooLarge));
        let expiry = chrono::Utc::now().timestamp() + 60;
        record_retryable_failure(
            "retry-test",
            "session",
            "message",
            "https://example.test/transient",
            expiry,
        );
        record_outcome(
            "retry-test",
            "session",
            "message",
            "https://example.test/permanent",
            false,
            expiry,
        );
        let (loaded, failed, refs, retryable) = outcome_details("retry-test", "session", "message");
        assert_eq!((loaded, failed), (0, 2));
        assert_eq!(refs.len(), 2);
        assert_eq!(
            retryable,
            vec![image_reference("https://example.test/transient")]
        );
        assert_eq!(outcome_details("other", "session", "message").1, 0);
        assert_eq!(outcome_details("retry-test", "other", "message").1, 0);
        record_outcome(
            "retry-test",
            "session",
            "message",
            "https://example.test/transient",
            true,
            expiry,
        );
        let (loaded, failed, refs, retryable) = outcome_details("retry-test", "session", "message");
        assert_eq!((loaded, failed), (1, 1));
        assert_eq!(
            refs,
            vec![image_reference("https://example.test/permanent")]
        );
        assert!(retryable.is_empty());
        let permanent = Failure::of(&Refusal::Status(403)).unwrap();
        assert!(
            !retryable_refusal(&permanent.refusal()),
            "negative cache must retain a permanent status"
        );
    }

    fn limits(per_message: usize, per_account: usize) -> FetchLimits {
        FetchLimits {
            per_message,
            per_account,
            ..FetchLimits::default()
        }
    }

    #[test]
    fn admission_headroom_never_extends_network_or_phase_timeouts() {
        let limits = FetchLimits::default();
        let started = Instant::now();
        let deadline = started + limits.overall_budget();
        assert_eq!(
            limits.queue_deadline(deadline),
            started + limits.queue_timeout
        );
        assert_eq!(
            deadline.duration_since(limits.queue_deadline(deadline)),
            limits.timeouts.total + limits.decode_timeout
        );
        assert_eq!(
            limits.overall_budget(),
            Duration::from_secs(60) + limits.timeouts.total + limits.decode_timeout
        );
        let fresh = limits
            .network_timeouts(Instant::now() + limits.overall_budget())
            .unwrap();
        assert_eq!(fresh, limits.timeouts);
        let short = limits
            .network_timeouts(Instant::now() + limits.decode_timeout + Duration::from_millis(100))
            .unwrap();
        assert!(short.total <= Duration::from_millis(100));
        assert_eq!(short.connect, limits.timeouts.connect);
        assert_eq!(short.headers, limits.timeouts.headers);
        assert_eq!(short.idle, limits.timeouts.idle);
        assert_eq!(short.max_redirects, limits.timeouts.max_redirects);
        assert!(limits.network_timeouts(Instant::now()).is_none());
    }

    #[tokio::test]
    async fn the_per_message_cap_queues_and_releases_on_drop() {
        let l = limits(2, 10);
        let soon = || Instant::now() + Duration::from_millis(150);
        let a = acquire("t56-acct-m", "msg", &l, soon()).await.expect("1st");
        let b = acquire("t56-acct-m", "msg", &l, soon()).await.expect("2nd");
        let started = Instant::now();
        assert!(
            acquire("t56-acct-m", "msg", &l, soon()).await.is_none(),
            "a third fetch of one message waits, then gives up at the deadline"
        );
        let waited = started.elapsed();
        assert!(
            waited >= Duration::from_millis(140) && waited < Duration::from_secs(2),
            "{waited:?}"
        );
        // Another message of the same account is not held up.
        let other = acquire("t56-acct-m", "other", &l, soon()).await;
        assert!(other.is_some());
        drop(a);
        assert!(
            acquire("t56-acct-m", "msg", &l, soon()).await.is_some(),
            "a freed slot is reused"
        );
        drop(b);
    }

    #[tokio::test]
    async fn the_per_account_cap_spans_messages_and_not_accounts() {
        let l = limits(5, 2);
        let soon = || Instant::now() + Duration::from_millis(100);
        let _a = acquire("t56-acct-a", "m1", &l, soon()).await.unwrap();
        let _b = acquire("t56-acct-a", "m2", &l, soon()).await.unwrap();
        assert!(acquire("t56-acct-a", "m3", &l, soon()).await.is_none());
        assert!(
            acquire("t56-acct-b", "m3", &l, soon()).await.is_some(),
            "per account"
        );
    }

    #[tokio::test]
    async fn a_waiting_fetch_takes_the_slot_as_soon_as_it_frees() {
        let l = limits(1, 1);
        let held = acquire(
            "t56-acct-w",
            "m",
            &l,
            Instant::now() + Duration::from_secs(1),
        )
        .await
        .unwrap();
        let release = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            drop(held);
        });
        let started = Instant::now();
        let got = acquire(
            "t56-acct-w",
            "m",
            &l,
            Instant::now() + Duration::from_secs(2),
        )
        .await;
        assert!(got.is_some());
        assert!(started.elapsed() < Duration::from_secs(1));
        release.await.unwrap();
    }

    #[test]
    fn only_network_failures_are_remembered_and_they_expire() {
        assert_eq!(Failure::of(&Refusal::Timeout), Some(Failure::Timeout));
        assert_eq!(
            Failure::of(&Refusal::Status(404)),
            Some(Failure::Status(404))
        );
        assert_eq!(Failure::of(&Refusal::Upstream), Some(Failure::Upstream));
        assert_eq!(Failure::of(&Refusal::TooLarge), Some(Failure::TooLarge));
        assert_eq!(Failure::of(&Refusal::Blocked), None);
        assert_eq!(Failure::of(&Refusal::BadRequest("x")), None);

        let mut n = Negative::default();
        let now = Instant::now();
        let k = negative_key("a", "https://img.example/x.png");
        n.put(k, Failure::Timeout, now);
        assert_eq!(
            n.get(&k, now + Duration::from_secs(9)),
            Some(Failure::Timeout)
        );
        assert_eq!(
            n.get(&k, now + NEGATIVE_TTL + Duration::from_millis(1)),
            None
        );
        assert!(n.0.is_empty(), "an expired entry is dropped on read");
    }

    #[test]
    fn negative_entries_are_per_account_and_forgotten_on_retry() {
        let url = "https://t56-retry.example/a.png";
        remember("t56-n-a", url, &Refusal::Timeout);
        assert_eq!(remembered("t56-n-a", url), Some(Failure::Timeout));
        assert_eq!(
            remembered("t56-n-b", url),
            None,
            "another account never sees it"
        );
        remember(
            "t56-n-a",
            "https://t56-retry.example/blocked",
            &Refusal::Blocked,
        );
        assert_eq!(
            remembered("t56-n-a", "https://t56-retry.example/blocked"),
            None
        );
        forget("t56-n-a", [url]);
        assert_eq!(remembered("t56-n-a", url), None);
    }

    #[test]
    fn the_negative_cache_is_bounded() {
        let mut n = Negative::default();
        let now = Instant::now();
        for i in 0..NEGATIVE_CAPACITY + 10 {
            n.put(
                negative_key("a", &i.to_string()),
                Failure::Upstream,
                now + Duration::from_millis(i as u64),
            );
        }
        assert_eq!(n.0.len(), NEGATIVE_CAPACITY);
        assert!(
            n.get(&negative_key("a", "0"), now).is_none(),
            "the oldest went first"
        );
    }

    #[test]
    fn outcomes_count_distinct_urls_and_the_latest_outcome_wins() {
        let mut o = Outcomes::default();
        let k = digest(&["o", "a", "s", "m"]);
        o.record(k, [1; 32], false, 100, 1);
        o.record(k, [2; 32], true, 100, 1);
        o.record(k, [2; 32], true, 100, 1);
        assert_eq!(o.read(&k, 1), (1, 1));
        o.record(k, [1; 32], true, 100, 1);
        assert_eq!(o.read(&k, 1), (2, 0), "a retried image moves to loaded");
        assert_eq!(o.read(&digest(&["o", "a", "s2", "m"]), 1), (0, 0));
        assert_eq!(o.read(&k, 100), (0, 0), "expired with the token window");
    }
}
