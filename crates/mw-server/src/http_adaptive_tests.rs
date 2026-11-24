//! Synthetic latency traces. The model server has `capacity` parallel slots and a
//! no-load time of 10 ms; with `n` requests in flight each takes
//! 10 ms × max(1, n / capacity) × jitter (jitter 0.9..1.3, deterministic).
use super::*;

const BASE_US: f64 = 10_000.0;

struct Model {
    capacity: f64,
    seed: u64,
}
impl Model {
    fn jitter(&mut self) -> f64 {
        self.seed = self
            .seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        0.9 + ((self.seed >> 33) % 1000) as f64 / 1000.0 * 0.4
    }
    /// One window at `demand` requests offered; returns the limit after it.
    fn window(&mut self, limiter: &mut Limiter, demand: usize, slowdown: f64) -> usize {
        for _ in 0..WINDOW {
            let n = demand.min(limiter.limit()).max(1);
            let us = BASE_US * (n as f64 / self.capacity).max(1.0) * slowdown * self.jitter();
            limiter.sample(Duration::from_micros(us as u64), n);
        }
        limiter.limit()
    }
}

#[test]
fn converges_under_saturation_and_stays_in_band_without_ratcheting() {
    let mut limiter = Limiter::new(256, 4096);
    let mut model = Model {
        capacity: 800.0,
        seed: 7,
    };
    for _ in 0..40 {
        assert_eq!(
            model.window(&mut limiter, 100, 1.0),
            4096,
            "light load leaves it"
        );
    }
    let baseline = limiter.baseline().unwrap();
    // Saturated: demand far above capacity.
    let mut windows_to_converge = None;
    for i in 0..60 {
        let limit = model.window(&mut limiter, 100_000, 1.0);
        if windows_to_converge.is_none() && limit <= 1800 {
            windows_to_converge = Some(i + 1);
        }
    }
    assert!(
        windows_to_converge.is_some_and(|w| w <= 20),
        "{windows_to_converge:?}"
    );
    // Long saturation: the limit holds a band and does not climb back up.
    let limits: Vec<usize> = (0..1000)
        .map(|_| model.window(&mut limiter, 100_000, 1.0))
        .collect();
    let (low, high) = (*limits.iter().min().unwrap(), *limits.iter().max().unwrap());
    // In-server time at most ~2x the 10 ms baseline: n up to ~2 x 800 / 1.3 jitter.
    assert!(low >= 900 && high <= 1800, "band {low}..{high}");
    assert!(
        (high - low) as f64 <= 0.35 * high as f64,
        "oscillation {low}..{high} beyond 35% of the top"
    );
    assert_eq!(
        limiter.baseline(),
        Some(baseline),
        "baseline unchanged by saturation"
    );
}

#[test]
fn recovers_to_the_ceiling_after_a_latency_spike() {
    let mut limiter = Limiter::new(256, 1024);
    let mut model = Model {
        capacity: 100_000.0,
        seed: 11,
    };
    for _ in 0..40 {
        model.window(&mut limiter, 300, 1.0);
    }
    for _ in 0..5 {
        assert_eq!(
            model.window(&mut limiter, 100_000, 1.0),
            1024,
            "no congestion"
        );
    }
    // A 10x spike (a slow upstream, a stall) for 30 windows.
    let mut lowest = usize::MAX;
    for _ in 0..30 {
        lowest = lowest.min(model.window(&mut limiter, 100_000, 10.0));
    }
    assert_eq!(lowest, 256, "fell to the floor, never below it");
    let mut back = None;
    for i in 0..60 {
        if model.window(&mut limiter, 100_000, 1.0) == 1024 {
            back = Some(i + 1);
            break;
        }
    }
    assert!(
        back.is_some_and(|w| w <= 40),
        "recovered after {back:?} windows"
    );
}

#[test]
fn never_leaves_floor_and_ceiling_on_an_erratic_trace() {
    let mut limiter = Limiter::new(300, 2000);
    let mut model = Model {
        capacity: 500.0,
        seed: 3,
    };
    for i in 0..2000u64 {
        let demand = [10, 5000, 400, 100_000, 1][(i % 5) as usize];
        let slowdown = if i % 97 < 5 { 20.0 } else { 1.0 };
        let limit = model.window(&mut limiter, demand, slowdown);
        assert!((300..=2000).contains(&limit), "{limit}");
    }
    assert!(limiter.decreases() > 0 && limiter.increases() > 0);
}

#[test]
fn low_demand_neither_raises_nor_lowers() {
    let mut limiter = Limiter::new(256, 1024);
    let mut model = Model {
        capacity: 50.0,
        seed: 5,
    };
    // 40 in flight against capacity 50: no queueing, demand below 80%.
    for _ in 0..200 {
        assert_eq!(model.window(&mut limiter, 40, 1.0), 1024);
    }
    assert_eq!((limiter.decreases(), limiter.increases()), (0, 0));
}

#[test]
fn floor_and_ceiling_are_normalised() {
    let limiter = Limiter::new(5000, 1000);
    assert_eq!(
        (limiter.floor(), limiter.ceiling(), limiter.limit()),
        (1000, 1000, 1000)
    );
    let limiter = Limiter::new(0, 0);
    assert_eq!((limiter.floor(), limiter.ceiling()), (1, 1));
}
