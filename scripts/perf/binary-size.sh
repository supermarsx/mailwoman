#!/usr/bin/env sh
# Server release-binary size gate (SPEC §23 / t9-e5-size 26.9 revision): the
# shipped `mailwoman` server binary must stay < 91 MB (measured + ~15% headroom).
#
# MEASURED, not asserted: builds the release binary the Docker `runtime` stage
# ships (`cargo build --release -p mw-server --bin mailwoman`), stats the exact
# bytes, prints the number, and FAILS on over-budget. Run on a Linux runner —
# the gate targets the Linux release artifact that goes into the container
# (Windows/MSVC builds are ~2× larger and are NOT the shipped artifact, so do
# not gate on them).
#
# 26.9 REVISED BUDGET (GREEN at the revised budget, fail-on-regression — NOT an
# honest-red-against-an-unreachable-number gate any more). The original SPEC §23
# 45 MB assumed a core-only build; the full V7 feature set (wasmtime JIT + every
# protocol + crypto + embedded SPA) statically links to ~79 MB stripped on Linux
# AFTER the first-party `.wasm` components were externalized out of the binary
# (t9-e5, digest-pinned). 91 = ceil(79 × 1.15): ~15% regression headroom so a new
# heavy dep still trips the gate. Rationale + measurement:
# docs/perf/size-budget-revision.md.
#
# Budget override (for a coordinator-agreed ceiling): PERF_BINARY_BUDGET_MB.
# Skip the build if the binary already exists: PERF_SKIP_BUILD=1.
#
# Usage: scripts/perf/binary-size.sh
set -eu

BUDGET_MB="${PERF_BINARY_BUDGET_MB:-91}"
BUDGET_BYTES=$((BUDGET_MB * 1024 * 1024))
BIN="target/release/mailwoman"

if [ "${PERF_SKIP_BUILD:-0}" != "1" ] || [ ! -f "$BIN" ]; then
  echo "[binary-size] building release binary (cargo build --release -p mw-server --bin mailwoman)..."
  cargo build --release -p mw-server --bin mailwoman
fi

if [ ! -f "$BIN" ]; then
  echo "[binary-size] FAIL: $BIN not found after build" >&2
  exit 1
fi

BYTES="$(wc -c < "$BIN" | tr -d ' ')"
MB="$(awk "BEGIN{printf \"%.2f\", $BYTES/1048576}")"
echo "[binary-size] $BIN = ${MB} MB (${BYTES} B) — budget ${BUDGET_MB} MB"

if [ "$BYTES" -gt "$BUDGET_BYTES" ]; then
  echo "[binary-size] FAIL: server binary exceeds ${BUDGET_MB} MB (SPEC §23)" >&2
  echo "[binary-size] Consider: strip symbols (profile.release strip=true), or opt-level=\"z\"/lto." >&2
  exit 1
fi
echo "[binary-size] OK: server binary < ${BUDGET_MB} MB (SPEC §23)"
