//! Ignored, release-mode old/new benchmarks. No synthetic timing claims.
use std::{
    fmt::Debug,
    hint::black_box,
    time::{Duration, Instant},
};
fn timed<R>(inputs: &[String], rounds: usize, operation: &mut impl FnMut(&str) -> R) -> Duration {
    let start = Instant::now();
    for _ in 0..rounds {
        for input in inputs {
            black_box(operation(black_box(input.as_str())));
        }
    }
    start.elapsed()
}
pub fn compare<R: PartialEq + Debug>(
    name: &str,
    inputs: &[String],
    mut reference: impl FnMut(&str) -> R,
    mut optimized: impl FnMut(&str) -> R,
) {
    assert!(!inputs.is_empty());
    for input in inputs {
        assert_eq!(
            reference(input),
            optimized(input),
            "benchmark parity: {input:?}"
        );
    }
    let warm = timed(inputs, 1, &mut reference).as_nanos().max(1);
    let rounds = (50_000_000u128 / warm).clamp(1, 100_000) as usize;
    let (mut old, mut new) = (Vec::new(), Vec::new());
    for sample in 0..7 {
        if sample % 2 == 0 {
            old.push(timed(inputs, rounds, &mut reference));
            new.push(timed(inputs, rounds, &mut optimized));
        } else {
            new.push(timed(inputs, rounds, &mut optimized));
            old.push(timed(inputs, rounds, &mut reference));
        }
    }
    old.sort_unstable();
    new.sort_unstable();
    let operations = (rounds * inputs.len()) as f64;
    let old_ns = old[3].as_nanos() as f64 / operations;
    let new_ns = new[3].as_nanos() as f64 / operations;
    println!(
        "{name}: reference={old_ns:.2} ns/op optimized={new_ns:.2} ns/op ratio={:.3}x operations/sample={} samples=7",
        old_ns / new_ns,
        rounds * inputs.len()
    );
}
