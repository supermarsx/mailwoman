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
    let budget = match (
        resources.memory_limit_bytes,
        resources.memory_available_bytes,
    ) {
        (Some(limit), Some(available)) => (limit / 4 * 3).min(available / 5 * 4),
        (Some(limit), None) => limit / 4 * 3,
        (None, Some(available)) => available / 5 * 4,
        (None, None) => 512 * 1024 * 1024,
    };
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
}
