#!/usr/bin/env bash
# Build the browser WASM crypto bundle (plan §2.5 / §3 e8, top V4 risk §6#2).
#
# Compiles `mw-crypto` to `wasm32-unknown-unknown` via wasm-pack (target `web`) into
# `apps/web/src/wasm/mw-crypto`, where the crypto Web Worker
# (`apps/web/src/crypto/worker.entry.ts`) imports the wasm-pack glue + module
# (loaded via `vite-plugin-wasm` + `vite-plugin-top-level-await`, off the
# login→inbox critical path — plan risk #12).
#
# e0 scaffolded this; e1 filled the crypto; e8 wired the worker; e8b (this) added the
# `mw-sanitize` wasm surface + build so decrypted E2EE HTML is sanitized IN-WORKER
# (plan §1.3). Both bundles are pruned of wasm-pack's `package.json`/`.gitignore` so
# they sit cleanly under `src/` (committed so a Rust-less `pnpm build/typecheck/test`
# stays green; e9 rebuilds on Win + Linux CI). The companion `build-wasm.ps1` is the
# Windows-dev twin (plan §1.13).
#
# §1.3 (in-worker sanitize): decrypted E2EE plaintext is sanitized in the browser
# crypto worker via the `mw-sanitize` wasm build BELOW — it never round-trips to the
# server sanitizer (which would defeat end-to-end encryption). HTML decrypted mail is
# then rendered as sanitized HTML in the existing no-scripts/no-same-origin sandboxed
# iframe; non-HTML plaintext keeps rendering as escaped text.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT_DIR="${ROOT}/apps/web/src/wasm"

# rPGP/RustCrypto reach getrandom's JS backend on wasm32 via plain crate features
# (mw-crypto's Cargo.toml wasm target deps), so no `--cfg getrandom_backend` is
# strictly required; we still export it for older getrandom generations' safety.
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

# wasm-pack drops a publish `package.json` + a `.gitignore` (`*`) into the out
# dir; prune both so the module imports cleanly from `src/` and is committed.
rm -f "${OUT_DIR}/mw-crypto/package.json" "${OUT_DIR}/mw-crypto/.gitignore"

echo "wasm bundle built into ${OUT_DIR}/mw-crypto"

# mw-sanitize → in-worker sanitize of decrypted E2EE HTML (plan §1.3 / risk #5). Pure
# ammonia; small bundle. Same `--target web` + prune as mw-crypto.
echo "building mw-sanitize → ${OUT_DIR}/mw-sanitize"
wasm-pack build "${ROOT}/crates/mw-sanitize" \
  --target web --out-dir "${OUT_DIR}/mw-sanitize" --out-name mw_sanitize
rm -f "${OUT_DIR}/mw-sanitize/package.json" "${OUT_DIR}/mw-sanitize/.gitignore"

echo "wasm bundle built into ${OUT_DIR}/mw-sanitize"
