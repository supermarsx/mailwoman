//! Stale-while-revalidate for the image proxy cache.
//!
//! An entry past its freshness is, for `privacy.imageCacheStaleSecs`, still answered
//! at once; the same request starts a background revalidation that asks the origin
//! with `If-None-Match` / `If-Modified-Since`. A `304` renews the entry, a `200`
//! replaces it, and an error leaves the old copy in place (it keeps answering for
//! `privacy.imageCacheMaxStaleSecs`, including when a foreground fetch fails).
//!
//! * One revalidation per image at a time (single flight, keyed like the tier).
//! * At most [`MAX_REVALIDATIONS`] run at once; past that a stale answer simply does
//!   not start one, and the next request will.
//! * A revalidation goes through the same guarded fetch as any other: SSRF gate,
//!   egress route, per-account and per-message slots, timeouts. It is not charged
//!   against the account rate limit (the reader did not ask for it).
//! * The origin `Cache-Control` is honoured by the tier: `no-store` is not cached,
//!   `no-cache` and `max-age=0` are stale at once, `max-age` bounds freshness.
use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
use std::sync::{Arc, Mutex, OnceLock};

use bytes::Bytes;
use mw_egress::{Request, TrafficClass};

use super::remote_content::{self, Fetched, FetchedBody};
use super::tier::{self, Hash, ImageTier};
use crate::AppState;

/// Revalidations running at once on this replica.
pub(crate) const MAX_REVALIDATIONS: usize = 4;

#[derive(Clone, Copy, Debug)]
pub(crate) enum Kind {
    /// Served fresh from the cache.
    Hit,
    /// Served past its freshness, revalidation started.
    Stale,
    /// Not in the cache; fetched.
    Miss,
}

static HIT: AtomicU64 = AtomicU64::new(0);
static STALE: AtomicU64 = AtomicU64::new(0);
static MISS: AtomicU64 = AtomicU64::new(0);
static NOT_MODIFIED: AtomicU64 = AtomicU64::new(0);
static REPLACED: AtomicU64 = AtomicU64::new(0);
static FAILED: AtomicU64 = AtomicU64::new(0);
static STALE_ON_ERROR: AtomicU64 = AtomicU64::new(0);

pub(crate) fn note(kind: Kind) {
    match kind {
        Kind::Hit => HIT.fetch_add(1, Relaxed),
        Kind::Stale => STALE.fetch_add(1, Relaxed),
        Kind::Miss => MISS.fetch_add(1, Relaxed),
    };
}

/// Cumulative counters since start.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Counters {
    pub hit: u64,
    pub stale: u64,
    pub miss: u64,
    /// Revalidations the origin answered `304`.
    pub revalidated: u64,
    /// Revalidations that brought a new body.
    pub replaced: u64,
    pub failed: u64,
    /// Answers served from a kept copy because the origin failed.
    pub stale_on_error: u64,
}

pub(crate) fn counters() -> Counters {
    Counters {
        hit: HIT.load(Relaxed),
        stale: STALE.load(Relaxed),
        miss: MISS.load(Relaxed),
        revalidated: NOT_MODIFIED.load(Relaxed),
        replaced: REPLACED.load(Relaxed),
        failed: FAILED.load(Relaxed),
        stale_on_error: STALE_ON_ERROR.load(Relaxed),
    }
}

/// A kept copy for a foreground request whose fetch failed, if one is within
/// `max_stale_secs` of its freshness.
pub(crate) async fn stale_on_error(
    tier: &Arc<ImageTier>,
    key: &super::CacheKey,
    max_stale_secs: u64,
) -> Option<(String, Bytes)> {
    if max_stale_secs == 0 {
        return None;
    }
    let found = tier.lookup_swr(&key.route, &key.url, key.ctx, max_stale_secs).await?;
    STALE_ON_ERROR.fetch_add(1, Relaxed);
    Some((found.etag, found.png))
}

// ── single flight and the concurrency bound ────────────────────────────────────

fn running() -> &'static Mutex<HashSet<Hash>> {
    static RUNNING: OnceLock<Mutex<HashSet<Hash>>> = OnceLock::new();
    RUNNING.get_or_init(Default::default)
}

fn slots() -> &'static Arc<tokio::sync::Semaphore> {
    static SLOTS: OnceLock<Arc<tokio::sync::Semaphore>> = OnceLock::new();
    SLOTS.get_or_init(|| Arc::new(tokio::sync::Semaphore::new(MAX_REVALIDATIONS)))
}

/// Releases the single-flight claim when the revalidation ends, however it ends.
struct Claim(Hash);
impl Drop for Claim {
    fn drop(&mut self) {
        running().lock().expect("revalidation set").remove(&self.0);
    }
}

fn claim(hash: Hash) -> Option<Claim> {
    // `then`, not `then_some`: a refused claim must not build (and so drop) a `Claim`,
    // whose drop would release the running revalidation's entry and re-lock the set.
    let inserted = running().lock().expect("revalidation set").insert(hash);
    inserted.then(|| Claim(hash))
}

/// Revalidations in progress now.
pub(crate) fn in_progress() -> usize {
    running().lock().map_or(0, |set| set.len())
}

/// Everything a background revalidation needs; owned, because it outlives the request.
pub(crate) struct Revalidate {
    pub state: AppState,
    pub policy: crate::remote_policy::PrivacyPolicy,
    pub route_id: String,
    pub domain: Option<String>,
    /// The cache key the stale entry is filed under.
    pub key: super::CacheKey,
    /// The URL to ask the origin for (the key may carry a resize suffix).
    pub fetch_url: String,
    pub account: String,
    pub message: String,
    pub validators: Option<tier::Upstream>,
    pub accept: &'static str,
    pub width: Option<u32>,
}

/// Start a revalidation for the stale image `key` unless one is already running or
/// all slots are busy. Never waits. The job is built here, behind `inline(never)`,
/// from references: the policy and app state are large, and building them inside
/// the request handlers' futures grew those futures past a debug test thread stack.
#[inline(never)]
#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_for(
    state: &AppState,
    policy: &crate::remote_policy::PrivacyPolicy,
    route_id: &str,
    domain: Option<&str>,
    key: &super::CacheKey,
    fetch_url: &str,
    (account, message): (&str, &str),
    validators: Option<tier::Upstream>,
    accept: &'static str,
    width: Option<u32>,
) {
    let hash = tier::key_hash(&key.route, &key.url);
    if running().lock().expect("revalidation set").contains(&hash) {
        return;
    }
    spawn(Box::new(Revalidate {
        state: state.clone(),
        policy: policy.clone(),
        route_id: route_id.to_string(),
        domain: domain.map(str::to_owned),
        key: key.clone(),
        fetch_url: fetch_url.to_string(),
        account: account.to_string(),
        message: message.to_string(),
        validators,
        accept,
        width,
    }));
}

fn spawn(job: Box<Revalidate>) {
    let hash = tier::key_hash(&job.key.route, &job.key.url);
    let Some(claim) = claim(hash) else { return };
    let Ok(permit) = slots().clone().try_acquire_owned() else {
        return;
    };
    tokio::spawn(async move {
        let _claim = claim;
        let _permit = permit;
        run(*job).await;
    });
}

async fn run(job: Revalidate) {
    let Ok(url) = reqwest::Url::parse(&job.fetch_url) else {
        FAILED.fetch_add(1, Relaxed);
        return;
    };
    let Ok(plan) = super::load_plan(&job.state.store).await else {
        FAILED.fetch_add(1, Relaxed);
        return;
    };
    let policy = &job.policy;
    let deadline = std::time::Instant::now() + policy.fetch.overall_budget();
    let validators = job.validators.clone().unwrap_or_default();
    let mut request = Request::get(TrafficClass::RemoteContent)
        .response_limit(policy.max_bytes)
        .accept(job.accept);
    if !validators.is_empty() {
        request = request.conditional(validators.etag.as_deref(), validators.last_modified.as_deref());
    }
    let fetched = remote_content::guarded_fetch(
        &job.state,
        &plan,
        &job.route_id,
        url,
        request,
        job.domain.as_deref(),
        (&job.account, &job.message, &job.fetch_url),
        policy,
        deadline,
    )
    .await;
    let tier = tier::prepare(&policy.image_tier).await;
    match fetched {
        Ok(Fetched::NotModified { upstream, directives }) => {
            tier.renew(&job.key.route, &job.key.url, upstream, directives);
            NOT_MODIFIED.fetch_add(1, Relaxed);
        }
        Ok(Fetched::Body(FetchedBody { raw, content_type, admission, upstream, directives })) => {
            if raw.len() > policy.max_bytes
                || remote_content::type_refusal(content_type.as_deref(), &raw).is_some()
            {
                FAILED.fetch_add(1, Relaxed);
                return;
            }
            let stored = remote_content::reencode_and_cache(
                raw,
                Some(job.key.clone()),
                policy,
                deadline,
                admission,
                Some((upstream, directives)),
                job.width,
            )
            .await;
            match stored {
                Ok(_) => REPLACED.fetch_add(1, Relaxed),
                Err(_) => FAILED.fetch_add(1, Relaxed),
            };
        }
        // The old copy stays and keeps answering until `max_stale_secs`.
        Err(_) => {
            FAILED.fetch_add(1, Relaxed);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_revalidation_per_image_and_the_claim_is_released() {
        let h: Hash = 0xfeed_0001;
        let first = claim(h).expect("first claim");
        assert!(claim(h).is_none(), "a second revalidation of the same image joins the first");
        assert!(claim(h + 1).is_some(), "another image is independent");
        drop(first);
        assert!(claim(h).is_some(), "released on drop");
        let _ = claim(h + 1);
        running().lock().unwrap().remove(&h);
        running().lock().unwrap().remove(&(h + 1));
    }

    #[test]
    fn revalidation_slots_are_bounded() {
        let sem = Arc::new(tokio::sync::Semaphore::new(MAX_REVALIDATIONS));
        let held: Vec<_> = (0..MAX_REVALIDATIONS).map(|_| sem.clone().try_acquire_owned().unwrap()).collect();
        assert!(sem.clone().try_acquire_owned().is_err());
        drop(held);
        assert!(sem.clone().try_acquire_owned().is_ok());
    }
}
