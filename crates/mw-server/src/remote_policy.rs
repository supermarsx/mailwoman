//! Effective reader privacy controls, resolved from the shared settings policy.
use crate::{AppState, settings_registry};
use axum::{
    Json,
    extract::{Query, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PrivacyPolicy {
    pub mode: String,
    pub trackers: bool,
    pub tracker_override_allowed: bool,
    pub clean_links: bool,
    pub allow_direct: bool,
    pub inline_parts: bool,
    pub remote_image_downloads: bool,
    /// The user's `images.trustedHosts` / `images.blockedHosts` (canonical
    /// lowercase domains). Blocked wins; trusted never overrides `block` mode.
    #[serde(skip)]
    pub trusted_hosts: Vec<String>,
    #[serde(skip)]
    pub blocked_hosts: Vec<String>,
    pub locked: Locks,
    #[serde(skip)]
    pub max_bytes: usize,
    #[serde(skip)]
    pub rate_per_min: usize,
    #[serde(skip)]
    pub rate_burst: usize,
    /// Per-account image fetch bucket sizing (`privacy.imageRate*`).
    #[serde(skip)]
    pub image_rate: crate::image_proxy::adaptive::RateConfig,
    /// RAM + disk tier configuration for the image cache (`privacy.image*`).
    #[serde(skip)]
    pub image_tier: crate::image_proxy::tier::TierConfig,
    #[serde(skip)]
    pub negative_cache_ttl: std::time::Duration,
    /// Record image proxy latency histograms (`privacy.imageLatencyTracking`).
    #[serde(skip)]
    pub track_latency: bool,
    pub image_load_budget_ms: u64,
    pub image_retry_attempts: usize,
    pub image_retry_delay_ms: usize,
    /// Timeouts, redirect cap and per-message/per-account concurrency (t56).
    #[serde(skip)]
    pub fetch: crate::image_proxy::remote_content::FetchLimits,
}

/// The t56 fetch keys, read in one pass (`effective_all`) rather than one query
/// set per key: they are read on every remote image request.
pub(crate) const FETCH_KEYS: [&str; 13] = [
    "privacy.fetchConnectTimeoutMs",
    "privacy.fetchHeadersTimeoutMs",
    "privacy.fetchIdleTimeoutMs",
    "privacy.fetchTotalTimeoutMs",
    "privacy.fetchQueueTimeoutMs",
    "privacy.imageDecodeTimeoutMs",
    "privacy.maxRedirects",
    "privacy.maxConcurrentGlobal",
    "privacy.maxConcurrentPipeline",
    "privacy.maxConcurrentDecodes",
    "privacy.maxQueuedRequests",
    "privacy.maxConcurrentPerMessage",
    "privacy.maxConcurrentPerAccount",
];

/// The fetch limits from resolved `privacy.*` values; a key with no value keeps
/// the [`FetchLimits`](crate::image_proxy::remote_content::FetchLimits) default.
/// Values are already bounded by the registry; the clamps here only keep a value
/// that bypassed it (a hand-edited row) from producing a zero-length clock or a
/// zero-sized semaphore.
pub(crate) fn fetch_limits(
    value: impl Fn(&str) -> Option<u64>,
) -> crate::image_proxy::remote_content::FetchLimits {
    use std::time::Duration;
    let defaults = crate::image_proxy::remote_content::FetchLimits::default();
    let ms = |key: &str, fallback: Duration, min: u64, max: u64| {
        value(key).map_or(fallback, |v| Duration::from_millis(v.clamp(min, max)))
    };
    let count = |key: &str, fallback: usize, min: u64, max: u64| {
        value(key).map_or(fallback, |v| v.clamp(min, max) as usize)
    };
    let t = defaults.timeouts;
    crate::image_proxy::remote_content::FetchLimits {
        concurrency_mode: defaults.concurrency_mode,
        image_bytes: defaults.image_bytes,
        runtime: None,
        queue_timeout: ms(
            "privacy.fetchQueueTimeoutMs",
            defaults.queue_timeout,
            1_000,
            180_000,
        ),
        decode_timeout: ms(
            "privacy.imageDecodeTimeoutMs",
            defaults.decode_timeout,
            1_000,
            60_000,
        ),
        timeouts: mw_egress::FetchTimeouts {
            connect: ms("privacy.fetchConnectTimeoutMs", t.connect, 500, 30_000),
            headers: ms("privacy.fetchHeadersTimeoutMs", t.headers, 500, 60_000),
            idle: ms("privacy.fetchIdleTimeoutMs", t.idle, 500, 60_000),
            total: ms("privacy.fetchTotalTimeoutMs", t.total, 1_000, 120_000),
            max_redirects: count("privacy.maxRedirects", t.max_redirects, 0, 10),
        },
        global: count("privacy.maxConcurrentGlobal", defaults.global, 1, 256),
        pipeline: count("privacy.maxConcurrentPipeline", defaults.pipeline, 1, 256),
        decodes: count("privacy.maxConcurrentDecodes", defaults.decodes, 1, 16),
        queued: count("privacy.maxQueuedRequests", defaults.queued, 16, 16384),
        per_message: count(
            "privacy.maxConcurrentPerMessage",
            defaults.per_message,
            1,
            128,
        ),
        per_account: count(
            "privacy.maxConcurrentPerAccount",
            defaults.per_account,
            1,
            256,
        ),
    }
}
impl PrivacyPolicy {
    /// `host` is `entry` or a subdomain of it (whole labels only).
    fn host_listed(list: &[String], url: &str) -> bool {
        let Some(host) = reqwest::Url::parse(url)
            .ok()
            .and_then(|u| u.host_str().map(|h| h.trim_end_matches('.').to_ascii_lowercase()))
        else {
            return false;
        };
        list.iter()
            .any(|entry| host == *entry || host.ends_with(&format!(".{entry}")))
    }
    /// The user blocked this image's host. Wins over every grant and trust.
    pub(crate) fn host_blocked(&self, url: &str) -> bool {
        Self::host_listed(&self.blocked_hosts, url)
    }
    /// Pure rule behind [`Self::host_trusted`], for tests.
    #[cfg(test)]
    fn rule(mode: &str, trusted: &[&str], blocked: &[&str], url: &str) -> (bool, bool) {
        let own = |list: &[&str]| list.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        let (trusted, blocked) = (own(trusted), own(blocked));
        let is_blocked = Self::host_listed(&blocked, url);
        (
            is_blocked,
            mode != "block" && !is_blocked && Self::host_listed(&trusted, url),
        )
    }
    /// The user trusts this image's host: it loads without a grant. Never in
    /// `block` mode (the administrator's cap) and never when blocked.
    pub(crate) fn host_trusted(&self, url: &str) -> bool {
        self.mode != "block" && !self.host_blocked(url) && Self::host_listed(&self.trusted_hosts, url)
    }
}

pub(crate) const POLICY_KEYS: &[&str] = &[
    "privacy.imageConcurrencyMode",
    "privacy.remoteContent",
    "images.trustedHosts",
    "images.blockedHosts",
    "privacy.remoteImageDownloads",
    "privacy.trackers",
    "privacy.trackerOverrideAllowed",
    "privacy.cleanLinks",
    "privacy.allowDirect",
    "cap.remote_content.direct",
    "privacy.maxBytes",
    "privacy.cacheBytes",
    "privacy.ratePerMin",
    "privacy.rateBurst",
    "privacy.imageRateMode",
    "privacy.imageRateBurstFloor",
    "privacy.imageRateBurstCeiling",
    "privacy.imageRateRefillFloor",
    "privacy.imageRateRefillCeiling",
    "privacy.imageCacheMode",
    "privacy.imageCacheBytesFloor",
    "privacy.imageCacheBytesCeiling",
    "privacy.cacheEntries",
    "privacy.cacheEntryBytes",
    "privacy.imageCacheTtlFloorSecs",
    "privacy.imageCacheTtlCeilingSecs",
    "privacy.cacheTtlSecs",
    "privacy.imageRamMaxMiB",
    "privacy.imageDiskEnabled",
    "privacy.imageDiskPath",
    "privacy.imageDiskMode",
    "privacy.imageDiskMaxMiB",
    "privacy.imageDiskFloorMiB",
    "privacy.imageDiskCeilingMiB",
    "privacy.imageDiskEntries",
    "privacy.imageDiskEntryBytes",
    "privacy.imageDiskMinHits",
    "privacy.imageDiskWriteQueueMiB",
    "privacy.imageTtlStepSecs",
    "privacy.imageTtlMaxSecs",
    "privacy.imageMaxAgeSecs",
    "privacy.imageScoreHalfLifeSecs",
    "privacy.imageScoreDemoteMilli",
    "privacy.imageScoreHysteresisPercent",
    "privacy.imageContextOpenPercent",
    "privacy.imageContextRecentPercent",
    "privacy.imageAccountSharePercent",
    "privacy.imageSweepSecs",
    "privacy.imageSnapshotSecs",
    "privacy.imageSweepBatch",
    "privacy.negativeCacheTtlSecs",
    "privacy.imageLatencyTracking",
    "privacy.imageRetryAttempts",
    "privacy.imageRetryDelayMs",
];

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Locks {
    pub mode: bool,
    pub trackers: bool,
    pub clean_links: bool,
}

fn unavailable() -> Response {
    (
        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        "privacy policy unavailable",
    )
        .into_response()
}

/// Mail aliases inherit the canonical domain route; an existing bare username
/// never acquires the preferred domain's route.
pub(crate) async fn canonical_domain(
    state: &AppState,
    session: &mw_store::Session,
) -> Result<Option<String>, Response> {
    match crate::upstream_resolve::domain_of(&session.username) {
        Some(domain) => state
            .store
            .canonical_policy_domain(&domain)
            .await
            .map(Some)
            .map_err(|_| unavailable()),
        None => Ok(None),
    }
}

pub(crate) async fn effective(
    state: &AppState,
    session: &mw_store::Session,
) -> Result<PrivacyPolicy, Response> {
    let identity = crate::settings_routes::UserSubject::resolved(&state.store, session)
        .await
        .map_err(|_| unavailable())?;
    let subject = identity.as_subject();
    // Resolve one current policy snapshot for the entire image request. Repeated
    // per-key reads used to reload the same subject/profile rows ten times.
    let resolved = settings_registry::effective_all(&state.store, &subject, |d| {
        FETCH_KEYS.contains(&d.key) || POLICY_KEYS.contains(&d.key)
    })
    .await
    .map_err(|_| unavailable())?;
    let values: std::collections::BTreeMap<_, _> = resolved
        .into_iter()
        .map(|value| (value.key.clone(), value))
        .collect();
    let boolean = |key| {
        values
            .get(key)
            .and_then(|v| v.value.as_bool())
            .unwrap_or(false)
    };
    let locked = |key| values.get(key).is_none_or(|v| v.locked());
    let hosts = |key: &str| -> Vec<String> {
        values
            .get(key)
            .and_then(|v| v.value.as_array())
            .map(|list| {
                list.iter()
                    .filter_map(|item| item.as_str().map(str::to_owned))
                    .take(100)
                    .collect()
            })
            .unwrap_or_default()
    };
    let number = |key: &str, fallback: usize| {
        values
            .get(key)
            .and_then(|v| v.value.as_u64())
            .map(|n| n as usize)
            .unwrap_or(fallback)
    };
    let sizing = |key: &str| {
        crate::image_proxy::adaptive::Sizing::parse(
            values
                .get(key)
                .and_then(|v| v.value.as_str())
                .unwrap_or("auto"),
        )
    };
    let mut fetch = fetch_limits(|key| values.get(key).and_then(|e| e.value.as_u64()));
    fetch.concurrency_mode = crate::image_proxy::adaptive::Mode::parse(
        values
            .get("privacy.imageConcurrencyMode")
            .and_then(|e| e.value.as_str())
            .unwrap_or("auto"),
    );
    fetch.image_bytes = number("privacy.maxBytes", 8 * 1024 * 1024);
    let allow_direct = boolean("privacy.allowDirect") && boolean("cap.remote_content.direct");
    let mut mode = values
        .get("privacy.remoteContent")
        .and_then(|v| Value::as_str(&v.value))
        .unwrap_or("ask")
        .to_owned();
    if mode == "direct" && !allow_direct {
        mode = "ask".into();
    }
    let (image_rate, image_cache) = image_configs(&number, &sizing);
    let resources = crate::image_proxy::resources::snapshot();
    let budget = crate::image_proxy::adaptive::cache_budget(
        image_cache,
        crate::image_proxy::adaptive::calculate(
            fetch.concurrency_mode,
            fetch.ceilings(),
            resources,
            fetch.image_bytes,
            mw_render::media_jail::IMAGE_DECODE_TYPICAL_BYTES,
        ),
        resources,
        fetch.image_bytes,
        mw_render::media_jail::IMAGE_DECODE_TYPICAL_BYTES,
        crate::image_proxy::hit_percent(),
    );
    Ok(PrivacyPolicy {
        mode,
        trackers: boolean("privacy.trackers"),
        tracker_override_allowed: boolean("privacy.trackerOverrideAllowed")
            && !(boolean("privacy.trackers") && locked("privacy.trackers")),
        clean_links: boolean("privacy.cleanLinks"),
        allow_direct,
        inline_parts: state.engine.is_some(),
        remote_image_downloads: boolean("privacy.remoteImageDownloads"),
        trusted_hosts: hosts("images.trustedHosts"),
        blocked_hosts: hosts("images.blockedHosts"),
        locked: Locks {
            mode: locked("privacy.remoteContent"),
            trackers: locked("privacy.trackers"),
            clean_links: locked("privacy.cleanLinks"),
        },
        max_bytes: number("privacy.maxBytes", 8 * 1024 * 1024),
        rate_per_min: number("privacy.ratePerMin", 1200),
        rate_burst: number("privacy.rateBurst", 1024),
        image_rate,
        image_tier: tier_config(&|key| values.get(key).map(|v| v.value.clone()), &budget),
        negative_cache_ttl: std::time::Duration::from_secs(number(
            "privacy.negativeCacheTtlSecs",
            crate::image_proxy::remote_content::fetch_limits::NEGATIVE_TTL.as_secs() as usize,
        ) as u64),
        track_latency: values
            .get("privacy.imageLatencyTracking")
            .and_then(|v| v.value.as_bool())
            .unwrap_or(true),
        image_load_budget_ms: fetch.overall_budget().as_millis() as u64,
        image_retry_attempts: number("privacy.imageRetryAttempts", 3),
        image_retry_delay_ms: number("privacy.imageRetryDelayMs", 5000),
        fetch,
    })
}
/// Every image-cache tunable. A registry test checks that each is registered,
/// documented, labelled in both shipped admin locales and read into the policy, and
/// that no other `privacy.image*` or `privacy.cache*` cache key exists without being
/// listed here.
#[cfg(test)]
pub(crate) const IMAGE_CACHE_KEYS: &[&str] = &[
    "privacy.cacheBytes",
    "privacy.cacheEntries",
    "privacy.cacheEntryBytes",
    "privacy.cacheTtlSecs",
    "privacy.imageCacheMode",
    "privacy.imageCacheBytesFloor",
    "privacy.imageCacheBytesCeiling",
    "privacy.imageCacheTtlFloorSecs",
    "privacy.imageCacheTtlCeilingSecs",
    "privacy.imageRamMaxMiB",
    "privacy.imageDiskEnabled",
    "privacy.imageDiskPath",
    "privacy.imageDiskMode",
    "privacy.imageDiskMaxMiB",
    "privacy.imageDiskFloorMiB",
    "privacy.imageDiskCeilingMiB",
    "privacy.imageDiskEntries",
    "privacy.imageDiskEntryBytes",
    "privacy.imageDiskMinHits",
    "privacy.imageDiskWriteQueueMiB",
    "privacy.imageTtlStepSecs",
    "privacy.imageTtlMaxSecs",
    "privacy.imageMaxAgeSecs",
    "privacy.imageScoreHalfLifeSecs",
    "privacy.imageScoreDemoteMilli",
    "privacy.imageScoreHysteresisPercent",
    "privacy.imageContextOpenPercent",
    "privacy.imageContextRecentPercent",
    "privacy.imageAccountSharePercent",
    "privacy.imageSweepSecs",
    "privacy.imageSnapshotSecs",
    "privacy.imageSweepBatch",
];

/// The cache directory: the configured absolute path, or an `image-cache` folder
/// beside the database file. Relative configured paths and `..` give `None`
/// (the tier stays off and the readout says why).
fn tier_path(configured: &str) -> Option<std::path::PathBuf> {
    use std::path::{Component, PathBuf};
    let configured = configured.trim();
    if !configured.is_empty() {
        let path = PathBuf::from(configured);
        let clean =
            path.is_absolute() && !path.components().any(|c| matches!(c, Component::ParentDir));
        return clean.then_some(path);
    }
    let db = PathBuf::from(std::env::var_os("MW_DB_PATH")?);
    let db = if db.is_absolute() {
        db
    } else {
        std::env::current_dir().ok()?.join(db)
    };
    Some(db.parent()?.join("image-cache"))
}

/// Tier configuration from the `privacy.image*` keys and the RAM budget that the
/// existing automatic or manual sizing produced.
pub(crate) fn tier_config(
    get: &dyn Fn(&str) -> Option<Value>,
    budget: &crate::image_proxy::adaptive::CacheBudget,
) -> crate::image_proxy::tier::TierConfig {
    use crate::image_proxy::tier::{DiskConfig, TierConfig};
    let (d, dd) = (TierConfig::default(), DiskConfig::default());
    let n = |key: &str, fallback: u64| get(key).and_then(|v| v.as_u64()).unwrap_or(fallback);
    let ram_cap = n("privacy.imageRamMaxMiB", 0) << 20;
    let ram_bytes = if ram_cap > 0 {
        budget.bytes.min(ram_cap)
    } else {
        budget.bytes
    };
    let enabled = get("privacy.imageDiskEnabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let text = get("privacy.imageDiskPath")
        .and_then(|v| v.as_str().map(str::to_owned))
        .unwrap_or_default();
    let auto = get("privacy.imageDiskMode")
        .and_then(|v| v.as_str().map(|m| m != "manual"))
        .unwrap_or(true);
    TierConfig {
        ram_bytes: ram_bytes as usize,
        ram_entries: budget.entries,
        ram_entry_bytes: budget.entry_bytes,
        ttl_base_secs: budget.ttl_secs,
        ttl_step_secs: n("privacy.imageTtlStepSecs", d.ttl_step_secs),
        ttl_max_secs: n("privacy.imageTtlMaxSecs", d.ttl_max_secs),
        max_age_secs: n("privacy.imageMaxAgeSecs", d.max_age_secs),
        half_life_secs: n("privacy.imageScoreHalfLifeSecs", d.half_life_secs),
        demote_milli: n("privacy.imageScoreDemoteMilli", u64::from(d.demote_milli)) as u32,
        hysteresis_pct: n(
            "privacy.imageScoreHysteresisPercent",
            u64::from(d.hysteresis_pct),
        ) as u32,
        open_pct: n("privacy.imageContextOpenPercent", u64::from(d.open_pct)) as u32,
        recent_pct: n(
            "privacy.imageContextRecentPercent",
            u64::from(d.recent_pct),
        ) as u32,
        account_share_pct: n(
            "privacy.imageAccountSharePercent",
            u64::from(d.account_share_pct),
        ) as u32,
        sweep_secs: n("privacy.imageSweepSecs", d.sweep_secs),
        snapshot_secs: n("privacy.imageSnapshotSecs", d.snapshot_secs),
        sweep_batch: n("privacy.imageSweepBatch", d.sweep_batch as u64) as usize,
        disk: DiskConfig {
            enabled,
            path: if enabled { tier_path(&text) } else { None },
            auto,
            manual_bytes: n("privacy.imageDiskMaxMiB", dd.manual_bytes >> 20) << 20,
            floor_bytes: n("privacy.imageDiskFloorMiB", dd.floor_bytes >> 20) << 20,
            ceiling_bytes: n("privacy.imageDiskCeilingMiB", dd.ceiling_bytes >> 20) << 20,
            entries: n("privacy.imageDiskEntries", dd.entries as u64) as usize,
            entry_bytes: n("privacy.imageDiskEntryBytes", dd.entry_bytes as u64) as usize,
            min_hits: n("privacy.imageDiskMinHits", u64::from(dd.min_hits)) as u32,
            write_queue_bytes: (n(
                "privacy.imageDiskWriteQueueMiB",
                (dd.write_queue_bytes >> 20) as u64,
            ) as usize)
                << 20,
        },
    }
}

/// Instance-wide tier configuration for the admin readout and diagnostics.
pub(crate) async fn instance_tier_config(
    state: &AppState,
    budget: &crate::image_proxy::adaptive::CacheBudget,
) -> Result<crate::image_proxy::tier::TierConfig, mw_store::StoreError> {
    let subject = mw_store::settings_policy::SettingSubject::default();
    let values =
        settings_registry::effective_all(&state.store, &subject, |d| POLICY_KEYS.contains(&d.key))
            .await?;
    Ok(tier_config(
        &|key| values.iter().find(|v| v.key == key).map(|v| v.value.clone()),
        budget,
    ))
}

/// Rate and cache sizing configuration from the `privacy.*` keys. `number`
/// reads an integer key with a fallback; `sizing` reads an auto/manual key.
pub(crate) fn image_configs(
    number: &dyn Fn(&str, usize) -> usize,
    sizing: &dyn Fn(&str) -> crate::image_proxy::adaptive::Sizing,
) -> (
    crate::image_proxy::adaptive::RateConfig,
    crate::image_proxy::adaptive::CacheConfig,
) {
    use crate::image_proxy::adaptive::{CacheConfig, RateConfig};
    let rate_defaults = RateConfig::default();
    let rate = RateConfig {
        sizing: sizing("privacy.imageRateMode"),
        burst_floor: number("privacy.imageRateBurstFloor", rate_defaults.burst_floor),
        burst_ceiling: number("privacy.imageRateBurstCeiling", rate_defaults.burst_ceiling),
        refill_floor: number("privacy.imageRateRefillFloor", rate_defaults.refill_floor),
        refill_ceiling: number(
            "privacy.imageRateRefillCeiling",
            rate_defaults.refill_ceiling,
        ),
        manual_burst: number("privacy.rateBurst", rate_defaults.manual_burst),
        manual_per_min: number("privacy.ratePerMin", rate_defaults.manual_per_min),
    };
    let defaults = CacheConfig::default();
    let entries = number("privacy.cacheEntries", defaults.entries_ceiling);
    let cache = CacheConfig {
        sizing: sizing("privacy.imageCacheMode"),
        bytes_floor: number(
            "privacy.imageCacheBytesFloor",
            defaults.bytes_floor as usize,
        ) as u64,
        bytes_ceiling: number(
            "privacy.imageCacheBytesCeiling",
            defaults.bytes_ceiling as usize,
        ) as u64,
        entries_ceiling: entries,
        entry_bytes_max: number("privacy.cacheEntryBytes", defaults.entry_bytes_max),
        ttl_floor_secs: number(
            "privacy.imageCacheTtlFloorSecs",
            defaults.ttl_floor_secs as usize,
        ) as u64,
        ttl_ceiling_secs: number(
            "privacy.imageCacheTtlCeilingSecs",
            defaults.ttl_ceiling_secs as usize,
        ) as u64,
        manual_bytes: number("privacy.cacheBytes", crate::image_proxy::MAX_CACHE_BYTES) as u64,
        manual_entries: entries,
        manual_ttl_secs: number(
            "privacy.cacheTtlSecs",
            crate::image_proxy::CACHE_TTL.as_secs() as usize,
        ) as u64,
    };
    (rate, cache)
}

/// Global sizing for administrative diagnostics. No account/domain preference
/// can raise these replica-wide resource ceilings.
pub(crate) async fn instance_limits(
    state: &AppState,
) -> Result<crate::image_proxy::remote_content::FetchLimits, mw_store::StoreError> {
    let subject = mw_store::settings_policy::SettingSubject::default();
    let values = settings_registry::effective_all(&state.store, &subject, |d| {
        FETCH_KEYS.contains(&d.key) || POLICY_KEYS.contains(&d.key)
    })
    .await?;
    let mut fetch = fetch_limits(|key| {
        values
            .iter()
            .find(|value| value.key == key)
            .and_then(|value| value.value.as_u64())
    });
    fetch.concurrency_mode = crate::image_proxy::adaptive::Mode::parse(
        values
            .iter()
            .find(|value| value.key == "privacy.imageConcurrencyMode")
            .and_then(|value| value.value.as_str())
            .unwrap_or("auto"),
    );
    fetch.image_bytes = values
        .iter()
        .find(|value| value.key == "privacy.maxBytes")
        .and_then(|value| value.value.as_u64())
        .unwrap_or(8 * 1024 * 1024) as usize;
    Ok(fetch)
}

/// A tracker exception is independent from image display grants and applies to
/// exactly one message. Re-read permission and grant on every capability use,
/// before cache access, so revocation or a new administrator lock takes effect.
pub(crate) async fn effective_for_message(
    state: &AppState,
    session: &mw_store::Session,
    email_id: &str,
) -> Result<PrivacyPolicy, Response> {
    let mut policy = effective(state, session).await?;
    if policy.tracker_override_allowed
        && policy.mode != "block"
        && state
            .store
            .is_remote_image_granted(&session.account_id, "trackers-single", email_id)
            .await
            .map_err(|_| unavailable())?
    {
        policy.trackers = false;
        // Explicit tracker consent always retains the server proxy: later
        // revocation must not leave direct-origin URLs in the message frame.
        if policy.mode == "direct" {
            policy.mode = "proxy".into();
        }
    }
    Ok(policy)
}
pub(crate) async fn get(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let session = match crate::authed(&state, &headers).await {
        Ok(s) => s,
        Err(r) => return r,
    };
    match effective(&state, &session).await {
        Ok(policy) => (
            [(axum::http::header::CACHE_CONTROL, "no-store")],
            Json(policy),
        )
            .into_response(),
        Err(r) => r,
    }
}

/// Static, public wrapper: message HTML is delivered only through browser
/// postMessage and never enters this endpoint or a server log.
#[derive(Clone)]
pub(crate) struct ReaderFrameResponse {
    pub compatible: bool,
}

#[derive(Default, Deserialize)]
pub(crate) struct ReaderFrameQuery {
    coep: Option<String>,
    #[serde(rename = "parentOrigin")]
    parent_origin: Option<String>,
}

fn reader_parent_origin(config: &crate::push_relay::NativeAuthConfig, value: &str) -> bool {
    if value.is_empty()
        || value == "null"
        || value.chars().any(|c| {
            c.is_whitespace()
                || c.is_control()
                || matches!(c, '*' | '\'' | '"' | ';' | '<' | '>' | '\\')
        })
        || !config.allows(value)
    {
        return false;
    }
    let Ok(url) = reqwest::Url::parse(value) else {
        return false;
    };
    if url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || !matches!(url.path(), "" | "/")
    {
        return false;
    }
    // Accept canonical origins only, including configured non-special shell
    // schemes whose URL.origin() is opaque. No CSP source grammar is accepted.
    url.as_str().trim_end_matches('/') == value && !value.ends_with('/')
}

pub(crate) async fn frame(
    State(state): State<AppState>,
    Query(query): Query<ReaderFrameQuery>,
) -> Response {
    use base64::Engine as _;
    use sha2::Digest;
    let parent_origin = match query.parent_origin.as_deref() {
        Some(origin) if reader_parent_origin(&state.native_auth, origin) => Some(origin),
        Some(_) => {
            return (
                axum::http::StatusCode::FORBIDDEN,
                "reader parent origin refused",
            )
                .into_response();
        }
        None => None,
    };
    let handler = include_str!("reader-address-handler.js")
        .replace("\r\n", "\n")
        .replace('\r', "\n");
    let handler_hash =
        base64::engine::general_purpose::STANDARD.encode(sha2::Sha256::digest(handler.as_bytes()));
    let script = include_str!("remote-frame.js")
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .replace(
            "__MW_PARENT_ORIGIN__",
            &serde_json::to_string(&parent_origin).expect("origin JSON"),
        )
        .replace(
            "__MW_ADDRESS_HANDLER_SOURCE__",
            &serde_json::to_string(&handler).expect("handler JSON"),
        )
        .replace(
            "__MW_ADDRESS_HANDLER_HASH__",
            &serde_json::to_string(&handler_hash).expect("hash JSON"),
        );
    let hash =
        base64::engine::general_purpose::STANDARD.encode(sha2::Sha256::digest(script.as_bytes()));
    let compatible = query.coep.as_deref() == Some("require-corp");
    // Unlike credentialless, require-corp does not omit credentials on direct
    // third-party images. Keep that fallback limited to our image proxy/parts
    // and embedded data, rather than silently sending third-party cookies.
    let images = if compatible {
        "'self' data: blob:"
    } else {
        "'self' https: data: blob:"
    };
    let ancestor = parent_origin.unwrap_or("'self'");
    let csp = format!(
        "default-src 'none'; script-src 'sha256-{hash}' 'sha256-{handler_hash}'; style-src 'unsafe-inline'; frame-src 'self'; img-src {images}; frame-ancestors {ancestor}; base-uri 'none'; form-action 'none'"
    );
    let html = format!(
        "<!doctype html><html><head><meta charset=utf-8><meta name=referrer content=no-referrer><style>html,body{{margin:0;width:100%;height:100%;overflow:hidden}}</style></head><body><script>{script}</script></body></html>"
    );
    let mut response = (
        [
            (
                axum::http::header::CONTENT_TYPE,
                "text/html; charset=utf-8".to_string(),
            ),
            (axum::http::header::CONTENT_SECURITY_POLICY, csp),
            (
                axum::http::header::CACHE_CONTROL,
                "public, max-age=300".to_string(),
            ),
            (
                axum::http::header::X_CONTENT_TYPE_OPTIONS,
                "nosniff".to_string(),
            ),
            (
                axum::http::header::REFERRER_POLICY,
                "no-referrer".to_string(),
            ),
        ],
        html,
    )
        .into_response();
    response
        .extensions_mut()
        .insert(ReaderFrameResponse { compatible });
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn reader_parent_origin_is_canonical_and_exactly_native_allowlisted() {
        let config = crate::push_relay::NativeAuthConfig {
            extra_origins: vec![
                "app://mailwoman".into(),
                "https://native.example:8443".into(),
                "https://bad.example/path".into(),
            ],
        };
        for value in [
            "tauri://localhost",
            "http://tauri.localhost",
            "https://tauri.localhost",
            "app://mailwoman",
            "https://native.example:8443",
        ] {
            assert!(reader_parent_origin(&config, value), "{value}");
        }
        for value in [
            "null",
            "*",
            "https://unknown.example",
            "http://tauri.localhost/",
            "http://tauri.localhost?x",
            "http://tauri.localhost#x",
            "https://bad.example/path",
            "http://tauri.localhost\n",
            "https://native.example:8443.evil",
        ] {
            assert!(!reader_parent_origin(&config, value), "{value}");
        }
    }

    #[test]
    fn fetch_limits_default_then_honour_values_and_clamp_bypassed_rows() {
        let none = fetch_limits(|_| None);
        assert_eq!(
            none,
            crate::image_proxy::remote_content::FetchLimits::default()
        );
        assert_eq!(none.timeouts, mw_egress::FetchTimeouts::REMOTE_IMAGE);
        assert_eq!(none.queue_timeout, Duration::from_secs(60));

        let set = fetch_limits(|key| match key {
            "privacy.fetchConnectTimeoutMs" => Some(1500),
            "privacy.fetchHeadersTimeoutMs" => Some(2500),
            "privacy.fetchIdleTimeoutMs" => Some(3500),
            "privacy.fetchTotalTimeoutMs" => Some(4500),
            "privacy.fetchQueueTimeoutMs" => Some(6500),
            "privacy.maxRedirects" => Some(0),
            "privacy.maxConcurrentPerMessage" => Some(2),
            "privacy.maxConcurrentPerAccount" => Some(3),
            _ => None,
        });
        assert_eq!(set.timeouts.connect, Duration::from_millis(1500));
        assert_eq!(set.timeouts.headers, Duration::from_millis(2500));
        assert_eq!(set.timeouts.idle, Duration::from_millis(3500));
        assert_eq!(set.timeouts.total, Duration::from_millis(4500));
        assert_eq!(set.queue_timeout, Duration::from_millis(6500));
        assert_eq!(set.overall_budget(), Duration::from_secs(21));
        assert_eq!(set.timeouts.max_redirects, 0);
        assert_eq!((set.per_message, set.per_account), (2, 3));

        let wild = fetch_limits(|key| Some(if key.ends_with("Ms") { 0 } else { u64::MAX }));
        assert_eq!(wild.timeouts.connect, Duration::from_millis(500));
        assert_eq!(wild.timeouts.total, Duration::from_secs(1));
        assert_eq!(wild.queue_timeout, Duration::from_secs(1));
        assert_eq!(
            fetch_limits(|key| (key == "privacy.fetchQueueTimeoutMs").then_some(u64::MAX))
                .queue_timeout,
            Duration::from_secs(180)
        );
        assert_eq!(wild.timeouts.max_redirects, 10);
        assert_eq!((wild.per_message, wild.per_account), (128, 256));
    }
}

/// Global (no user override) rate and cache configuration, for the admin readout.
pub(crate) async fn instance_image_configs(
    state: &AppState,
) -> Result<
    (
        crate::image_proxy::adaptive::RateConfig,
        crate::image_proxy::adaptive::CacheConfig,
    ),
    mw_store::StoreError,
> {
    let subject = mw_store::settings_policy::SettingSubject::default();
    let values =
        settings_registry::effective_all(&state.store, &subject, |d| POLICY_KEYS.contains(&d.key))
            .await?;
    let number = |key: &str, fallback: usize| {
        values
            .iter()
            .find(|v| v.key == key)
            .and_then(|v| v.value.as_u64())
            .map_or(fallback, |n| n as usize)
    };
    let sizing = |key: &str| {
        crate::image_proxy::adaptive::Sizing::parse(
            values
                .iter()
                .find(|v| v.key == key)
                .and_then(|v| v.value.as_str())
                .unwrap_or("auto"),
        )
    };
    Ok(image_configs(&number, &sizing))
}

#[cfg(test)]
mod host_rule_tests {
    use super::PrivacyPolicy as P;
    #[test]
    fn blocked_wins_trusted_needs_a_mode_that_loads_and_labels_are_whole() {
        let url = |h: &str| format!("https://{h}/a.png");
        let t = ["shop.test", "news.test"];
        let b = ["ads.news.test"];
        assert_eq!(P::rule("ask", &t, &b, &url("cdn.shop.test")), (false, true));
        assert_eq!(P::rule("ask", &t, &b, &url("ads.news.test")), (true, false));
        assert_eq!(P::rule("ask", &t, &b, &url("x.ads.news.test")), (true, false));
        assert_eq!(P::rule("ask", &t, &b, &url("evilshop.test")), (false, false));
        assert_eq!(P::rule("ask", &t, &b, &url("shop.test.evil.example")), (false, false));
        assert_eq!(P::rule("ask", &t, &b, &url("SHOP.test.")), (false, true));
        // The administrator's block mode is a cap trust cannot lift.
        assert_eq!(P::rule("block", &t, &b, &url("shop.test")), (false, false));
        assert_eq!(P::rule("proxy", &t, &b, &url("ads.news.test")), (true, false));
        assert_eq!(P::rule("ask", &t, &b, "not a url"), (false, false));
    }
}
