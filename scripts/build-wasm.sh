#!/usr/bin/env bash
# Build the browser WASM crypto bundle (plan §2.5 / §3 e8, top V4 risk §6#2).
#
# Compiles `mw-crypto` (+ `mw-sanitize`, for in-worker decrypt sanitize, §1.3) to
# `wasm32-unknown-unknown` via wasm-pack into `apps/web/src/wasm/`, where the crypto
# Web Worker (`apps/web/src/crypto/worker.ts`) imports it (wired by
# `vite-plugin-wasm` + `vite-plugin-top-level-await`).
#
# e0 authors this scaffold (the toolchain contract); the wasm-pack module bodies
# are `todo!()` until e1 fills the crypto and e8 wires the worker — so a run now
# produces a loadable-but-inert bundle proving the toolchain end-to-end. The
# companion `build-wasm.ps1` is the Windows-dev twin (plan §1.13: Win + Linux both
# build). e9 runs BOTH in CI.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT_DIR="${ROOT}/apps/web/src/wasm"

# getrandom's JS backend on wasm32 is selected by this cfg (getrandom 0.3+/0.4),
# NOT a cargo feature — required once e1 pulls in real RNG (rPGP keygen).
export RUSTFLAGS="${RUSTFLAGS:-} --cfg getrandom_backend=\"wasm_js\""

if ! command -v wasm-pack >/dev/null 2>&1; then
  echo "wasm-pack not found. Install: cargo install wasm-pack (or https://rustwasm.github.io/wasm-pack/)" >&2
  exit 1
fi
rustup target add wasm32-unknown-unknown >/dev/null 2>&1 || true

echo "building mw-crypto → ${OUT_DIR}/mw-crypto"
wasm-pack build "${ROOT}/crates/mw-crypto" \
  --target web --out-dir "${OUT_DIR}/mw-crypto" --out-name mw_crypto \
  -- --features wasm

echo "building mw-sanitize → ${OUT_DIR}/mw-sanitize"
wasm-pack build "${ROOT}/crates/mw-sanitize" \
  --target web --out-dir "${OUT_DIR}/mw-sanitize" --out-name mw_sanitize

echo "wasm bundle built into ${OUT_DIR}"
