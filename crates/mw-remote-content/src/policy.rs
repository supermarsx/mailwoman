//! Pure reader and cache policy models; effective values are supplied by the host.
use serde::Serialize;
use serde_json::Value;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PrivacyPolicy {
    pub mode: String,
    pub trackers: bool,
    pub tracker_override_allowed: bool,
    pub clean_links: bool,
    pub allow_direct: bool,
    pub inline_parts: bool,
    pub remote_image_downloads: bool,
    /// Browser HTTP cache only; independent of the server's image tiers.
    pub body_image_cache_enabled: bool,
    pub body_image_cache_ttl_secs: usize,
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
    pub image_rate: crate::adaptive::RateConfig,
    /// RAM + disk tier configuration for the image cache (`privacy.image*`).
    #[serde(skip)]
    pub image_tier: crate::tier::TierConfig,
    #[serde(skip)]
    pub negative_cache_ttl: std::time::Duration,
    /// Record image proxy latency histograms (`privacy.imageLatencyTracking`).
    #[serde(skip)]
    pub track_latency: bool,
    pub image_load_budget_ms: u64,
    pub image_retry_attempts: usize,
    pub image_retry_delay_ms: usize,
    /// The widest image the reader's frame may ask for with a `w=` hint; absent when
    /// the resize is off, which is also the client's cue not to send one.
    pub image_resize_max_width: Option<u32>,
    /// Optional downscale of oversized images (`privacy.imageResize*`).
    #[serde(skip)]
    pub image_resize: crate::resize::ResizeConfig,
    /// Timeouts, redirect cap and per-message/per-account concurrency (t56).
    #[serde(skip)]
    pub fetch: crate::FetchLimits,
}

/// The t56 fetch keys, read in one pass (`effective_all`) rather than one query
/// set per key: they are read on every remote image request.
pub const FETCH_KEYS: [&str; 13] = [
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
/// the [`FetchLimits`](crate::FetchLimits) default.
/// Values are already bounded by the registry; the clamps here only keep a value
/// that bypassed it (a hand-edited row) from producing a zero-length clock or a
/// zero-sized semaphore.
pub fn fetch_limits(value: impl Fn(&str) -> Option<u64>) -> crate::FetchLimits {
    use std::time::Duration;
    let defaults = crate::FetchLimits::default();
    let ms = |key: &str, fallback: Duration, min: u64, max: u64| {
        value(key).map_or(fallback, |v| Duration::from_millis(v.clamp(min, max)))
    };
    let count = |key: &str, fallback: usize, min: u64, max: u64| {
        value(key).map_or(fallback, |v| v.clamp(min, max) as usize)
    };
    let t = defaults.timeouts;
    crate::FetchLimits {
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
        let Some(host) = reqwest::Url::parse(url).ok().and_then(|u| {
            u.host_str()
                .map(|h| h.trim_end_matches('.').to_ascii_lowercase())
        }) else {
            return false;
        };
        list.iter()
            .any(|entry| host == *entry || host.ends_with(&format!(".{entry}")))
    }
    /// The user blocked this image's host. Wins over every grant and trust.
    pub fn host_blocked(&self, url: &str) -> bool {
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
    pub fn host_trusted(&self, url: &str) -> bool {
        self.mode != "block"
            && !self.host_blocked(url)
            && Self::host_listed(&self.trusted_hosts, url)
    }
}

pub const POLICY_KEYS: &[&str] = &[
    "privacy.bodyImageCacheEnabled",
    "privacy.bodyImageCacheTtlSecs",
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
    "privacy.imageCacheStaleSecs",
    "privacy.imageCacheMaxStaleSecs",
    "privacy.imageResizeEnabled",
    "privacy.imageResizeMaxWidth",
    "privacy.imageResizeMaxPixels",
];

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Locks {
    pub mode: bool,
    pub trackers: bool,
    pub clean_links: bool,
}

/// Every image-cache tunable. A registry test checks that each is registered,
/// documented, labelled in both shipped admin locales and read into the policy, and
/// that no other `privacy.image*` or `privacy.cache*` cache key exists without being
/// listed here.
pub const IMAGE_CACHE_KEYS: &[&str] = &[
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
    "privacy.imageCacheStaleSecs",
    "privacy.imageCacheMaxStaleSecs",
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
pub fn tier_config(
    get: &dyn Fn(&str) -> Option<Value>,
    budget: &crate::adaptive::CacheBudget,
) -> crate::tier::TierConfig {
    use crate::tier::{DiskConfig, TierConfig};
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
        recent_pct: n("privacy.imageContextRecentPercent", u64::from(d.recent_pct)) as u32,
        account_share_pct: n(
            "privacy.imageAccountSharePercent",
            u64::from(d.account_share_pct),
        ) as u32,
        sweep_secs: n("privacy.imageSweepSecs", d.sweep_secs),
        snapshot_secs: n("privacy.imageSnapshotSecs", d.snapshot_secs),
        sweep_batch: n("privacy.imageSweepBatch", d.sweep_batch as u64) as usize,
        stale_secs: n("privacy.imageCacheStaleSecs", d.stale_secs),
        max_stale_secs: n("privacy.imageCacheMaxStaleSecs", d.max_stale_secs)
            .max(n("privacy.imageCacheStaleSecs", d.stale_secs)),
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

/// The optional image resize, from its `privacy.imageResize*` keys.
pub fn resize_config(get: &dyn Fn(&str) -> Option<Value>) -> crate::resize::ResizeConfig {
    use crate::resize::ResizeConfig;
    let d = ResizeConfig::default();
    let n = |key: &str, fallback: u64| get(key).and_then(|v| v.as_u64()).unwrap_or(fallback);
    ResizeConfig {
        enabled: get("privacy.imageResizeEnabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(d.enabled),
        max_width: n("privacy.imageResizeMaxWidth", u64::from(d.max_width)) as u32,
        max_pixels: n("privacy.imageResizeMaxPixels", d.max_pixels),
        // PNG is the only encoder in the build, so the format is not a setting.
        format: d.format,
    }
}

/// Rate and cache sizing configuration from the `privacy.*` keys. `number`
/// reads an integer key with a fallback; `sizing` reads an auto/manual key.
pub fn image_configs(
    number: &dyn Fn(&str, usize) -> usize,
    sizing: &dyn Fn(&str) -> crate::adaptive::Sizing,
) -> (crate::adaptive::RateConfig, crate::adaptive::CacheConfig) {
    use crate::adaptive::{CacheConfig, RateConfig};
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
        manual_bytes: number("privacy.cacheBytes", crate::MAX_CACHE_BYTES) as u64,
        manual_entries: entries,
        manual_ttl_secs: number("privacy.cacheTtlSecs", crate::CACHE_TTL.as_secs() as usize) as u64,
    };
    (rate, cache)
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
        assert_eq!(
            P::rule("ask", &t, &b, &url("x.ads.news.test")),
            (true, false)
        );
        assert_eq!(
            P::rule("ask", &t, &b, &url("evilshop.test")),
            (false, false)
        );
        assert_eq!(
            P::rule("ask", &t, &b, &url("shop.test.evil.example")),
            (false, false)
        );
        assert_eq!(P::rule("ask", &t, &b, &url("SHOP.test.")), (false, true));
        // The administrator's block mode is a cap trust cannot lift.
        assert_eq!(P::rule("block", &t, &b, &url("shop.test")), (false, false));
        assert_eq!(
            P::rule("proxy", &t, &b, &url("ads.news.test")),
            (true, false)
        );
        assert_eq!(P::rule("ask", &t, &b, "not a url"), (false, false));
    }
}
