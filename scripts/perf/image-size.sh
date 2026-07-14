#!/usr/bin/env sh
# Container-image size gate (SPEC §23 / plan §6 t8-e5-perf): the production
# runtime image must be < 30 MB.
#
# MEASURED, not asserted: builds the existing Dockerfile `runtime` target and
# reads the REAL on-disk image size via `docker image inspect`, prints it, and
# FAILS on over-budget. Needs Docker (a Linux CI runner); locally skippable.
#
# HONESTY NOTE (task directive): if the runtime image legitimately cannot hit
# 30 MB, this gate MEASURES the real number and FAILS loudly rather than faking a
# pass — the Dockerfile itself flags the `FROM scratch` + musl-static build as a
# deferred hardening step (it currently ships on distroless/cc-debian12, ~20+ MB
# base, plus two Rust binaries). If the measured size is over, this is a real
# finding for a product decision (do the musl-static work, or revise the budget),
# NOT something to hide. The coordinator can set the agreed ceiling via
# PERF_IMAGE_BUDGET_MB without editing this script.
#
# Budget override: PERF_IMAGE_BUDGET_MB (default 30).
# Reuse an already-built image (skip docker build): PERF_IMAGE_TAG + PERF_SKIP_BUILD=1.
#
# Usage: scripts/perf/image-size.sh
set -eu

BUDGET_MB="${PERF_IMAGE_BUDGET_MB:-30}"
BUDGET_BYTES=$((BUDGET_MB * 1024 * 1024))
TAG="${PERF_IMAGE_TAG:-mailwoman-perf:runtime}"

if [ "${PERF_SKIP_BUILD:-0}" != "1" ]; then
  echo "[image-size] building runtime image ($TAG) from the existing Dockerfile..."
  DOCKER_BUILDKIT=1 docker build --target runtime -t "$TAG" .
fi

# Primary metric: `docker image inspect .Size` (uncompressed on-disk bytes,
# standard + correct on Linux CI runners). Cross-check against `docker history`
# layer sizes: some Docker Desktop builds under-report .Size, and we must NOT
# fake-pass on a bogus small number — so gate on the LARGER of the two.
INSPECT_BYTES="$(docker image inspect "$TAG" --format '{{.Size}}' 2>/dev/null || echo 0)"
HISTORY_BYTES="$(docker history "$TAG" --no-trunc --format '{{.Size}}' 2>/dev/null \
  | awk '
    /[0-9]/ {
      v=$0; sub(/[A-Za-z]+$/,"",v); u=$0; sub(/^[0-9.]+/,"",u);
      m=1; if(u=="kB")m=1000; else if(u=="MB")m=1000000; else if(u=="GB")m=1000000000;
      total += v*m;
    }
    END { printf "%d", total }')"
BYTES="$INSPECT_BYTES"
if [ "${HISTORY_BYTES:-0}" -gt "${BYTES:-0}" ] 2>/dev/null; then BYTES="$HISTORY_BYTES"; fi
if [ -z "$BYTES" ] || [ "$BYTES" -eq 0 ] 2>/dev/null; then
  echo "[image-size] FAIL: could not read image size for $TAG" >&2
  exit 1
fi
MB="$(awk "BEGIN{printf \"%.2f\", $BYTES/1048576}")"
echo "[image-size] $TAG = ${MB} MB (${BYTES} B) — budget ${BUDGET_MB} MB"
echo "[image-size]   (inspect .Size=$(awk "BEGIN{printf \"%.1f\", ${INSPECT_BYTES:-0}/1048576}")MB, history-sum=$(awk "BEGIN{printf \"%.1f\", ${HISTORY_BYTES:-0}/1048576}")MB — gating on the larger)"

if [ "$BYTES" -gt "$BUDGET_BYTES" ]; then
  echo "[image-size] FAIL: runtime image exceeds ${BUDGET_MB} MB (SPEC §23)." >&2
  echo "[image-size] Real finding for a decision: switch the runtime stage to a" >&2
  echo "[image-size] FROM scratch + musl-static build (Dockerfile TODO), or revise the budget." >&2
  exit 1
fi
echo "[image-size] OK: runtime image < ${BUDGET_MB} MB (SPEC §23)"
