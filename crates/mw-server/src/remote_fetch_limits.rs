//! Remote-image fetch limits (t56): per-message and per-account concurrency, a
//! short negative cache for failed fetches, and the per-message load outcome
//! counts the reader's loading toast polls.
//!
//! All three are per replica and in memory, like the rate limiter and the image
//! cache they sit next to. None of them retains a URL, a session id or a message
//! id in the clear: keys are SHA-256 digests of length-prefixed parts.
//!
//! # Concurrency
//! The global ceiling (`MAX_CONCURRENT`, try-acquire, shed with `503`) bounds the
//! replica. These two bound one reader: a newsletter with sixty images may hold at
//! most `privacy.maxConcurrentPerMessage` upstream fetches at a time, and one
//! account across all its messages and tabs at most
//! `privacy.maxConcurrentPerAccount`. Over either cap a fetch **waits** for a slot
//! rather than failing, but only until the fetch's total deadline: an image
//! answers within `privacy.fetchTotalTimeoutMs` whether it waited, fetched or
//! both. The message slot is taken before the account slot, always in that order,
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

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use mw_egress::{FetchTimeouts, Refusal};
use sha2::{Digest, Sha256};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

/// How long a failed fetch is answered from memory.
pub(crate) const NEGATIVE_TTL: Duration = Duration::from_secs(60);
/// Most negative entries kept; the oldest go first.
const NEGATIVE_CAPACITY: usize = 2048;
/// Prune idle slot entries once a map holds this many.
const SLOT_PRUNE_AT: usize = 1024;
/// Most messages with load outcomes kept, and URLs per message.
const OUTCOME_MESSAGES: usize = 1024;
const OUTCOME_URLS: usize = 256;

/// The effective limits for one fetch, from `privacy.*` (see `remote_policy`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FetchLimits {
    pub timeouts: FetchTimeouts,
    pub per_message: usize,
    pub per_account: usize,
}

impl Default for FetchLimits {
    fn default() -> Self {
        Self {
            timeouts: FetchTimeouts::REMOTE_IMAGE,
            per_message: 6,
            per_account: 12,
        }
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

#[derive(Default)]
struct Slots(HashMap<[u8; 32], (usize, Arc<Semaphore>)>);

impl Slots {
    /// The semaphore for `key` sized `size`. A changed size starts a fresh
    /// semaphore; fetches holding the old one finish under it.
    fn get(&mut self, key: [u8; 32], size: usize) -> Arc<Semaphore> {
        if self.0.len() >= SLOT_PRUNE_AT {
            self.0.retain(|_, (size, sem)| {
                Arc::strong_count(sem) > 1 || sem.available_permits() != *size
            });
        }
        let size = size.max(1);
        let entry = self
            .0
            .entry(key)
            .or_insert_with(|| (size, Arc::new(Semaphore::new(size))));
        if entry.0 != size {
            *entry = (size, Arc::new(Semaphore::new(size)));
        }
        entry.1.clone()
    }
}

fn slots() -> &'static Mutex<Slots> {
    static SLOTS: OnceLock<Mutex<Slots>> = OnceLock::new();
    SLOTS.get_or_init(|| Mutex::new(Slots::default()))
}

/// A held message slot and account slot; both are released on drop.
pub(crate) struct FetchSlot {
    _message: OwnedSemaphorePermit,
    _account: OwnedSemaphorePermit,
}

/// Wait, until `deadline`, for a slot for `message` of `account`. `None` when the
/// deadline passed first.
pub(crate) async fn acquire(
    account: &str,
    message: &str,
    limits: &FetchLimits,
    deadline: Instant,
) -> Option<FetchSlot> {
    let (message_sem, account_sem) = {
        let mut slots = slots().lock().expect("fetch slot lock");
        (
            slots.get(digest(&["m", account, message]), limits.per_message),
            slots.get(digest(&["a", account]), limits.per_account),
        )
    };
    let deadline = tokio::time::Instant::from_std(deadline);
    let message = tokio::time::timeout_at(deadline, message_sem.acquire_owned())
        .await
        .ok()?
        .ok()?;
    let account = tokio::time::timeout_at(deadline, account_sem.acquire_owned())
        .await
        .ok()?
        .ok()?;
    Some(FetchSlot {
        _message: message,
        _account: account,
    })
}

// ── negative cache ────────────────────────────────────────────────────────────

/// A remembered failure. Only network failures are cached.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Failure {
    Timeout,
    Upstream,
    TooLarge,
}

impl Failure {
    /// The failure to remember for `refusal`, or `None` for one not to cache.
    pub(crate) fn of(refusal: &Refusal) -> Option<Failure> {
        match refusal {
            Refusal::Timeout => Some(Failure::Timeout),
            Refusal::Upstream | Refusal::Status(_) => Some(Failure::Upstream),
            Refusal::TooLarge => Some(Failure::TooLarge),
            Refusal::Blocked | Refusal::BadRequest(_) => None,
        }
    }

    /// The refusal this failure is answered as. A status collapses to
    /// `Upstream`, which `refusal_response` answers identically.
    pub(crate) fn refusal(self) -> Refusal {
        match self {
            Failure::Timeout => Refusal::Timeout,
            Failure::Upstream => Refusal::Upstream,
            Failure::TooLarge => Refusal::TooLarge,
        }
    }
}

#[derive(Default)]
struct Negative(HashMap<[u8; 32], (Instant, Failure)>);

impl Negative {
    fn get(&mut self, key: &[u8; 32], now: Instant) -> Option<Failure> {
        match self.0.get(key) {
            Some((until, failure)) if *until > now => Some(*failure),
            Some(_) => {
                self.0.remove(key);
                None
            }
            None => None,
        }
    }

    fn put(&mut self, key: [u8; 32], failure: Failure, now: Instant) {
        if self.0.len() >= NEGATIVE_CAPACITY {
            self.0.retain(|_, (until, _)| *until > now);
        }
        if self.0.len() >= NEGATIVE_CAPACITY
            && let Some(oldest) = self
                .0
                .iter()
                .min_by_key(|(_, (until, _))| *until)
                .map(|(k, _)| *k)
        {
            self.0.remove(&oldest);
        }
        self.0.insert(key, (now + NEGATIVE_TTL, failure));
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
pub(crate) fn remembered(account: &str, url: &str) -> Option<Failure> {
    negative()
        .lock()
        .expect("negative cache lock")
        .get(&negative_key(account, url), Instant::now())
}

/// Remember `refusal` for `url` of `account` when it is a network failure.
pub(crate) fn remember(account: &str, url: &str, refusal: &Refusal) {
    if let Some(failure) = Failure::of(refusal) {
        negative().lock().expect("negative cache lock").put(
            negative_key(account, url),
            failure,
            Instant::now(),
        );
    }
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
}

#[derive(Default)]
struct Outcomes(HashMap<[u8; 32], Message>);

impl Outcomes {
    fn prune(&mut self, now: i64) {
        self.0.retain(|_, m| {
            m.urls.retain(|_, (_, expires)| *expires > now);
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

/// `(loaded, failed)` distinct images for the session's message.
pub(crate) fn read_outcomes(account: &str, session: &str, message: &str) -> (usize, usize) {
    outcomes()
        .lock()
        .expect("outcome lock")
        .read(&digest(&["o", account, session, message]), chrono::Utc::now().timestamp())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn limits(per_message: usize, per_account: usize) -> FetchLimits {
        FetchLimits {
            per_message,
            per_account,
            ..FetchLimits::default()
        }
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
        assert!(waited >= Duration::from_millis(140) && waited < Duration::from_secs(2), "{waited:?}");
        // Another message of the same account is not held up.
        let other = acquire("t56-acct-m", "other", &l, soon()).await;
        assert!(other.is_some());
        drop(a);
        assert!(acquire("t56-acct-m", "msg", &l, soon()).await.is_some(), "a freed slot is reused");
        drop(b);
    }

    #[tokio::test]
    async fn the_per_account_cap_spans_messages_and_not_accounts() {
        let l = limits(5, 2);
        let soon = || Instant::now() + Duration::from_millis(100);
        let _a = acquire("t56-acct-a", "m1", &l, soon()).await.unwrap();
        let _b = acquire("t56-acct-a", "m2", &l, soon()).await.unwrap();
        assert!(acquire("t56-acct-a", "m3", &l, soon()).await.is_none());
        assert!(acquire("t56-acct-b", "m3", &l, soon()).await.is_some(), "per account");
    }

    #[tokio::test]
    async fn a_waiting_fetch_takes_the_slot_as_soon_as_it_frees() {
        let l = limits(1, 1);
        let held = acquire("t56-acct-w", "m", &l, Instant::now() + Duration::from_secs(1))
            .await
            .unwrap();
        let release = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            drop(held);
        });
        let started = Instant::now();
        let got = acquire("t56-acct-w", "m", &l, Instant::now() + Duration::from_secs(2)).await;
        assert!(got.is_some());
        assert!(started.elapsed() < Duration::from_secs(1));
        release.await.unwrap();
    }

    #[test]
    fn only_network_failures_are_remembered_and_they_expire() {
        assert_eq!(Failure::of(&Refusal::Timeout), Some(Failure::Timeout));
        assert_eq!(Failure::of(&Refusal::Status(404)), Some(Failure::Upstream));
        assert_eq!(Failure::of(&Refusal::Upstream), Some(Failure::Upstream));
        assert_eq!(Failure::of(&Refusal::TooLarge), Some(Failure::TooLarge));
        assert_eq!(Failure::of(&Refusal::Blocked), None);
        assert_eq!(Failure::of(&Refusal::BadRequest("x")), None);

        let mut n = Negative::default();
        let now = Instant::now();
        let k = negative_key("a", "https://img.example/x.png");
        n.put(k, Failure::Timeout, now);
        assert_eq!(n.get(&k, now + Duration::from_secs(59)), Some(Failure::Timeout));
        assert_eq!(n.get(&k, now + NEGATIVE_TTL + Duration::from_millis(1)), None);
        assert!(n.0.is_empty(), "an expired entry is dropped on read");
    }

    #[test]
    fn negative_entries_are_per_account_and_forgotten_on_retry() {
        let url = "https://t56-retry.example/a.png";
        remember("t56-n-a", url, &Refusal::Timeout);
        assert_eq!(remembered("t56-n-a", url), Some(Failure::Timeout));
        assert_eq!(remembered("t56-n-b", url), None, "another account never sees it");
        remember("t56-n-a", "https://t56-retry.example/blocked", &Refusal::Blocked);
        assert_eq!(remembered("t56-n-a", "https://t56-retry.example/blocked"), None);
        forget("t56-n-a", [url]);
        assert_eq!(remembered("t56-n-a", url), None);
    }

    #[test]
    fn the_negative_cache_is_bounded() {
        let mut n = Negative::default();
        let now = Instant::now();
        for i in 0..NEGATIVE_CAPACITY + 10 {
            n.put(negative_key("a", &i.to_string()), Failure::Upstream, now + Duration::from_millis(i as u64));
        }
        assert_eq!(n.0.len(), NEGATIVE_CAPACITY);
        assert!(n.get(&negative_key("a", "0"), now).is_none(), "the oldest went first");
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
