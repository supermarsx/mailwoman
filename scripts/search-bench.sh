#!/usr/bin/env sh
# mw-search p95 latency gate (SPEC §23, plan §3 e1/e11): build the synthetic
# 100k-document index and assert p95 query latency < 50 ms.
#
# The gate itself lives in the Rust test `crates/mw-search/tests/bench.rs`
# (`p95_under_50ms_over_100k`, `#[ignore]`), which `assert!`s p95 < 50 ms — so a
# regression fails the test and therefore this script (non-zero cargo exit).
# This wrapper just runs it in release with output captured, echoes the p95
# number prominently for the CI log/trend, and propagates the exit code.
#
# Usage: scripts/search-bench.sh
set -eu

OUT="$(mktemp)"
trap 'rm -f "$OUT"' EXIT

echo "[search-bench] building + running the 100k p95 gate (release)..."
set +e
cargo test -p mw-search --release --test bench -- --ignored --nocapture 2>&1 | tee "$OUT"
STATUS=$?
set -e

# Surface the timing line so the run's p95 is visible at a glance / trendable.
P95_LINE="$(grep -E '^\s*p95\s*:' "$OUT" || true)"
if [ -n "$P95_LINE" ]; then
  echo "[search-bench] measured${P95_LINE#*p95}"  # -> "measured : 2.825 ms"
fi

if [ "$STATUS" -ne 0 ]; then
  echo "[search-bench] FAIL: p95 gate (<50 ms over 100k) not met — see above" >&2
  exit "$STATUS"
fi
echo "[search-bench] OK: p95 < 50 ms over 100k (SPEC §23)"
