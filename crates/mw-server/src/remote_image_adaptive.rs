//! Pure instance sizing. Per-message demand never changes replica-wide gates.
use super::resources::ImageResources;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Mode {
    Auto,
    Conservative,
    Balanced,
    Throughput,
    Manual,
}
impl Mode {
    pub(crate) fn parse(value: &str) -> Self {
        match value {
            "conservative" => Self::Conservative,
            "balanced" => Self::Balanced,
            "throughput" => Self::Throughput,
            "manual" => Self::Manual,
            _ => Self::Auto,
        }
    }
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Conservative => "conservative",
            Self::Balanced => "balanced",
            Self::Throughput => "throughput",
            Self::Manual => "manual",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Limits {
    pub per_message: usize,
    pub per_account: usize,
    pub global: usize,
    pub pipeline: usize,
    pub decoders: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Runtime {
    pub resources: ImageResources,
    pub effective: Limits,
    pub revision: u64,
}

/// The working-memory allowance the image pipeline may plan against: a bounded
/// share of the container limit, further capped by what is currently free.
pub(crate) fn memory_budget(resources: ImageResources) -> u64 {
    match (
        resources.memory_limit_bytes,
        resources.memory_available_bytes,
    ) {
        (Some(limit), Some(available)) => (limit / 4 * 3).min(available / 5 * 4),
        (Some(limit), None) => limit / 4 * 3,
        (None, Some(available)) => available / 5 * 4,
        (None, None) => 512 * 1024 * 1024,
    }
}

/// `auto` derives a value from capacity and load; `manual` uses the fixed
/// numbers exactly.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Sizing {
    Auto,
    Manual,
}
impl Sizing {
    pub(crate) fn parse(value: &str) -> Self {
        if value == "manual" {
            Self::Manual
        } else {
            Self::Auto
        }
    }
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Manual => "manual",
        }
    }
}

/// Per-account token bucket settings (`privacy.imageRate*`, `rateBurst`, `ratePerMin`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RateConfig {
    pub sizing: Sizing,
    pub burst_floor: usize,
    pub burst_ceiling: usize,
    pub refill_floor: usize,
    pub refill_ceiling: usize,
    /// Used exactly in manual mode.
    pub manual_burst: usize,
    pub manual_per_min: usize,
}
impl Default for RateConfig {
    fn default() -> Self {
        Self {
            sizing: Sizing::Auto,
            burst_floor: 64,
            burst_ceiling: 2048,
            refill_floor: 300,
            refill_ceiling: 6000,
            manual_burst: 1024,
            manual_per_min: 1200,
        }
    }
}

/// What the replica and this account are doing right now.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct RateLoad {
    /// Requests waiting for any image slot, and the queue size they wait in.
    pub waiting: usize,
    pub queue_capacity: usize,
    /// Accounts that used the proxy in the last minute.
    pub active_accounts: usize,
    /// Image count of the message being loaded, when known.
    pub demand: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RateBudget {
    /// Bucket size, tokens.
    pub burst: usize,
    /// Refill, tokens per minute.
    pub per_min: usize,
}

fn pressure_percent(load: RateLoad) -> usize {
    if load.queue_capacity == 0 {
        return 0;
    }
    (load.waiting.saturating_mul(100) / load.queue_capacity).min(100)
}

/// Pure per-account burst/refill sizing. Auto: burst follows the account's
/// concurrency slice (4x) shrunk by up to half under queue pressure, and never
/// below the message's own image count; refill is the account's share of the
/// replica's network capacity (one fetch per slot per second), scaled by idle
/// capacity (at least 20 percent). Both stay inside floor and ceiling. A rate
/// limit is an abuse cap; authentication, grants and the SSRF gate are not
/// affected by these numbers.
pub(crate) fn rate_budget(config: RateConfig, effective: Limits, load: RateLoad) -> RateBudget {
    if config.sizing == Sizing::Manual {
        return RateBudget {
            burst: config.manual_burst.max(1),
            per_min: config.manual_per_min.max(1),
        };
    }
    let (burst_lo, burst_hi) = (
        config.burst_floor.min(config.burst_ceiling).max(1),
        config.burst_floor.max(config.burst_ceiling).max(1),
    );
    let (refill_lo, refill_hi) = (
        config.refill_floor.min(config.refill_ceiling).max(1),
        config.refill_floor.max(config.refill_ceiling).max(1),
    );
    let pressure = pressure_percent(load);
    let capacity_burst = effective.per_account.saturating_mul(4);
    let shrunk = capacity_burst * (100 - pressure / 2) / 100;
    let burst = shrunk
        .max(load.demand.unwrap_or(0))
        .clamp(burst_lo, burst_hi);
    let idle = (100 - pressure).max(20);
    let share = effective.global.saturating_mul(60) / load.active_accounts.max(1);
    let per_min = (share * idle / 100).clamp(refill_lo, refill_hi);
    RateBudget { burst, per_min }
}

/// RAM image cache settings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CacheConfig {
    pub sizing: Sizing,
    pub bytes_floor: u64,
    pub bytes_ceiling: u64,
    pub entries_ceiling: usize,
    pub entry_bytes_max: usize,
    pub ttl_floor_secs: u64,
    pub ttl_ceiling_secs: u64,
    pub manual_bytes: u64,
    pub manual_entries: usize,
    pub manual_ttl_secs: u64,
}
impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            sizing: Sizing::Auto,
            bytes_floor: 16 << 20,
            bytes_ceiling: 256 << 20,
            entries_ceiling: 4096,
            entry_bytes_max: 4 << 20,
            ttl_floor_secs: 300,
            ttl_ceiling_secs: 6 * 3600,
            manual_bytes: 32 << 20,
            manual_entries: 1024,
            manual_ttl_secs: 3600,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CacheBudget {
    pub bytes: u64,
    pub entries: usize,
    pub entry_bytes: usize,
    pub ttl_secs: u64,
}

/// Pure cache sizing. Auto: an eighth of the memory left after the decoder and
/// pipeline reservations (so it never competes with them), halved while free
/// memory is under a fifth of the limit; TTL runs from floor (low hit rate or
/// memory pressure) to ceiling (hit rate of 60 percent or more).
pub(crate) fn cache_budget(
    config: CacheConfig,
    effective: Limits,
    resources: ImageResources,
    max_image_bytes: usize,
    decoder_memory_bytes: usize,
    hit_percent: Option<u8>,
) -> CacheBudget {
    if config.sizing == Sizing::Manual {
        return CacheBudget {
            bytes: config.manual_bytes,
            entries: config.manual_entries.max(1),
            entry_bytes: config.entry_bytes_max,
            ttl_secs: config.manual_ttl_secs,
        };
    }
    let raw = (max_image_bytes as u64).clamp(65_536, 8 * 1024 * 1024);
    let reserved = (effective.decoders as u64)
        .saturating_mul(decoder_memory_bytes as u64)
        .saturating_add((effective.pipeline as u64).saturating_mul(raw));
    let spare = memory_budget(resources).saturating_sub(reserved);
    let pressured = match (
        resources.memory_limit_bytes,
        resources.memory_available_bytes,
    ) {
        (Some(limit), Some(available)) => available < limit / 5,
        _ => false,
    };
    let mut bytes = spare / 8;
    if pressured {
        bytes /= 2;
    }
    let (lo, hi) = (
        config.bytes_floor.min(config.bytes_ceiling),
        config.bytes_floor.max(config.bytes_ceiling),
    );
    // The floor is honoured, but never more than half of what is really spare.
    let bytes = bytes.clamp(lo.min(spare / 2), hi);
    let entries = ((bytes / (16 * 1024)) as usize).clamp(64, config.entries_ceiling.max(64));
    let entry_bytes = config
        .entry_bytes_max
        .min((bytes / 4).min(usize::MAX as u64) as usize)
        .max(1);
    let (tlo, thi) = (
        config.ttl_floor_secs.min(config.ttl_ceiling_secs),
        config.ttl_floor_secs.max(config.ttl_ceiling_secs),
    );
    let ttl_secs = if pressured {
        tlo
    } else {
        let hit = u64::from(hit_percent.unwrap_or(30).min(60));
        tlo + (thi - tlo) * hit / 60
    };
    CacheBudget {
        bytes,
        entries,
        entry_bytes,
        ttl_secs,
    }
}

/// Presets narrow existing numeric ceilings; they never silently rewrite them.
pub(crate) fn calculate(
    mode: Mode,
    configured: Limits,
    resources: ImageResources,
    max_image_bytes: usize,
    decoder_memory_bytes: usize,
) -> Limits {
    let mut limits = configured;
    let (message, global, decoders, pipeline, account, io_per_core, io_floor, cpu_weight) =
        match mode {
            Mode::Conservative => (8, 32, 2, 48, 32, 8, 16, 500),
            Mode::Balanced => (20, 96, 6, 96, 96, 12, 24, 750),
            Mode::Throughput => (30, 128, 10, 128, 128, 24, 48, 1500),
            Mode::Auto => (
                usize::MAX,
                usize::MAX,
                usize::MAX,
                usize::MAX,
                usize::MAX,
                16,
                32,
                1000,
            ),
            Mode::Manual => return limits,
        };
    limits.per_message = limits.per_message.min(message);
    limits.per_account = limits.per_account.min(account);
    limits.global = limits.global.min(global);
    limits.pipeline = limits.pipeline.min(pipeline);
    limits.decoders = limits.decoders.min(decoders);

    let cpu = u64::from(resources.cpu_millis).clamp(100, 256_000);
    let cpu_network =
        (cpu * io_per_core)
            .div_ceil(1000)
            .max(if cpu >= 1000 { io_floor } else { 1 });
    let cpu_decoders = (cpu * cpu_weight).div_ceil(1_000_000).max(1);
    // Use container headroom as well as its size. A loaded instance can reduce
    // capacity without waiting for a failed allocation. Unknown RAM uses a
    // bounded working allowance, rather than assuming host-sized memory.
    let budget = memory_budget(resources);
    let raw = (max_image_bytes as u64).clamp(65_536, 8 * 1024 * 1024);
    // Reserve raw buffers before filling the remaining allowance with decoder
    // jobs. Use the real sandbox bound, including codec working memory.
    let decoder_allowance = (decoder_memory_bytes as u64).max(1);
    let io_target = limits.global.min(limits.pipeline).min(cpu_network as usize) as u64;
    let raw_reserve = raw.saturating_mul(io_target);
    let memory_decoders = (budget.saturating_sub(raw_reserve) / decoder_allowance).max(1);
    limits.decoders = limits
        .decoders
        .min(cpu_decoders as usize)
        .min(memory_decoders as usize)
        .max(1);
    let buffered = budget.saturating_sub(limits.decoders as u64 * decoder_allowance);
    let memory_pipeline = (buffered / raw).max(1) as usize;
    limits.pipeline = limits.pipeline.min(memory_pipeline).max(1);
    limits.global = limits
        .global
        .min(cpu_network as usize)
        .min(limits.pipeline)
        .max(1);
    limits.per_account = limits.per_account.min(limits.pipeline).max(1);
    limits.per_message = limits
        .per_message
        .min(limits.global)
        .min(limits.per_account)
        .max(1);
    limits
}

pub(crate) fn message_limit(ceiling: usize, approved_images: Option<usize>) -> usize {
    approved_images
        .map_or(ceiling, |images| ceiling.min(images.max(1)))
        .max(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn ceilings() -> Limits {
        Limits {
            per_message: 30,
            per_account: 128,
            global: 128,
            pipeline: 128,
            decoders: 10,
        }
    }
    fn resources(cpu: u32, gib: u64) -> ImageResources {
        ImageResources {
            cpu_millis: cpu,
            memory_limit_bytes: Some(gib * 1024 * 1024 * 1024),
            memory_available_bytes: Some(gib * 1024 * 1024 * 1024),
        }
    }
    #[test]
    fn large_instances_can_use_requested_ceilings_and_small_messages_use_their_count() {
        let effective = calculate(
            Mode::Auto,
            ceilings(),
            resources(16_000, 32),
            8 * 1024 * 1024,
            512 * 1024 * 1024,
        );
        assert_eq!(effective, ceilings());
        assert_eq!(message_limit(effective.per_message, Some(3)), 3);
        assert_eq!(message_limit(effective.per_message, Some(200)), 30);
        assert_eq!(message_limit(effective.per_message, None), 30);
    }
    #[test]
    fn small_instances_scale_down_and_recover_with_real_headroom() {
        let small = calculate(
            Mode::Auto,
            ceilings(),
            resources(1000, 1),
            8 * 1024 * 1024,
            512 * 1024 * 1024,
        );
        assert!(small.global < 128);
        assert_eq!(small.decoders, 1);
        assert!(small.global >= 16);
        let mut busy = resources(16_000, 32);
        busy.memory_available_bytes = Some(64 * 1024 * 1024);
        let pressured = calculate(
            Mode::Auto,
            ceilings(),
            busy,
            8 * 1024 * 1024,
            512 * 1024 * 1024,
        );
        assert_eq!(
            (pressured.global, pressured.pipeline, pressured.decoders),
            (1, 1, 1)
        );
        assert_eq!(
            calculate(
                Mode::Auto,
                ceilings(),
                resources(16_000, 32),
                8 * 1024 * 1024,
                512 * 1024 * 1024
            ),
            ceilings()
        );
    }
    #[test]
    fn presets_and_explicit_limits_remain_ceilings() {
        let hardware = resources(16_000, 32);
        let low = calculate(
            Mode::Conservative,
            ceilings(),
            hardware,
            8 * 1024 * 1024,
            512 * 1024 * 1024,
        );
        let balanced = calculate(
            Mode::Balanced,
            ceilings(),
            hardware,
            8 * 1024 * 1024,
            512 * 1024 * 1024,
        );
        let fast = calculate(
            Mode::Throughput,
            ceilings(),
            hardware,
            8 * 1024 * 1024,
            512 * 1024 * 1024,
        );
        assert_eq!((low.per_message, low.global, low.decoders), (8, 32, 2));
        assert_eq!(
            (balanced.per_message, balanced.global, balanced.decoders),
            (20, 96, 6)
        );
        assert_eq!(fast, ceilings());
        let explicit = Limits {
            per_message: 5,
            per_account: 9,
            global: 12,
            pipeline: 16,
            decoders: 2,
        };
        for mode in [
            Mode::Auto,
            Mode::Conservative,
            Mode::Balanced,
            Mode::Throughput,
            Mode::Manual,
        ] {
            let result = calculate(mode, explicit, hardware, 8 * 1024 * 1024, 512 * 1024 * 1024);
            assert!(
                result.per_message <= 5
                    && result.per_account <= 9
                    && result.global <= 12
                    && result.pipeline <= 16
                    && result.decoders <= 2
            );
        }
    }
    #[test]
    fn manual_mode_preserves_numeric_settings_even_on_small_hardware() {
        assert_eq!(
            calculate(
                Mode::Manual,
                ceilings(),
                resources(500, 1),
                8 * 1024 * 1024,
                512 * 1024 * 1024
            ),
            ceilings()
        );
        assert_eq!(Mode::parse("unknown"), Mode::Auto);
        assert_eq!(Mode::Auto.as_str(), "auto");
    }
    #[test]
    fn rate_budget_manual_is_exact_and_auto_honours_bounds() {
        let manual = RateConfig {
            sizing: Sizing::Manual,
            manual_burst: 7,
            manual_per_min: 9,
            ..RateConfig::default()
        };
        assert_eq!(
            rate_budget(manual, ceilings(), RateLoad::default()),
            RateBudget {
                burst: 7,
                per_min: 9
            }
        );
        let config = RateConfig::default();
        for waiting in [0, 100, 2048, 9999] {
            for accounts in [0, 1, 50, 5000] {
                for demand in [None, Some(3), Some(100_000)] {
                    let b = rate_budget(
                        config,
                        ceilings(),
                        RateLoad {
                            waiting,
                            queue_capacity: 2048,
                            active_accounts: accounts,
                            demand,
                        },
                    );
                    assert!((64..=2048).contains(&b.burst), "{b:?}");
                    assert!((300..=6000).contains(&b.per_min), "{b:?}");
                }
            }
        }
    }
    #[test]
    fn rate_budget_is_monotonic_in_demand_pressure_and_accounts() {
        let config = RateConfig {
            burst_floor: 1,
            refill_floor: 1,
            ..RateConfig::default()
        };
        let at = |waiting, accounts, demand| {
            rate_budget(
                config,
                ceilings(),
                RateLoad {
                    waiting,
                    queue_capacity: 1000,
                    active_accounts: accounts,
                    demand,
                },
            )
        };
        assert!(at(0, 1, Some(900)).burst >= at(0, 1, Some(10)).burst);
        assert!(at(0, 1, None).burst >= at(900, 1, None).burst);
        assert!(at(0, 1, None).per_min >= at(900, 1, None).per_min);
        assert!(at(0, 1, None).per_min >= at(0, 40, None).per_min);
        assert!(at(0, 1, Some(500)).burst >= 500);
    }
    #[test]
    fn cache_budget_scales_with_memory_and_stays_in_bounds() {
        let cfg = CacheConfig::default();
        let size = |res, hit| cache_budget(cfg, ceilings(), res, 8 << 20, 64 << 20, hit);
        let big = size(resources(16_000, 32), Some(60));
        let small = size(resources(1000, 1), Some(60));
        assert!(big.bytes >= small.bytes);
        assert!(big.bytes <= cfg.bytes_ceiling && big.entries <= cfg.entries_ceiling);
        assert_eq!(big.ttl_secs, cfg.ttl_ceiling_secs);
        let cold = size(resources(16_000, 32), Some(0));
        assert_eq!(cold.ttl_secs, cfg.ttl_floor_secs);
        let mut tight = resources(16_000, 32);
        tight.memory_available_bytes = Some(1 << 30);
        let pressured = size(tight, Some(60));
        assert_eq!(pressured.ttl_secs, cfg.ttl_floor_secs);
        assert!(pressured.bytes <= big.bytes);
        let manual = CacheConfig {
            sizing: Sizing::Manual,
            manual_bytes: 5,
            manual_entries: 2,
            manual_ttl_secs: 9,
            ..cfg
        };
        let m = cache_budget(
            manual,
            ceilings(),
            resources(1000, 1),
            8 << 20,
            64 << 20,
            None,
        );
        assert_eq!((m.bytes, m.entries, m.ttl_secs), (5, 2, 9));
    }
}
