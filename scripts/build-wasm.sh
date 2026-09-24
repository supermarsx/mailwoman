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
#
# ── Toolchain pin + what reproducibility is actually achievable (t24-e13) ────
#
# `wasm-pack` is PINNED below. It is not cosmetic: wasm-pack chooses the flags
# handed to `wasm-bindgen`, and a different wasm-pack emits a different glue ABI
# for the SAME crate version. The guests committed before 26.20 carry the newer
# multi-value/externref ABI; 0.15.0 emits the older stack-pointer one. Both work
# — glue and `.wasm` are generated together and are internally consistent — but
# the artefact changes wholesale when this version moves, so moving it is an
# explicit, reviewed change, exactly like rust-toolchain.toml.
#
# The wasm-bindgen CLI needs NO separate pin. wasm-pack resolves it from the
# build's own dependency graph and downloads that exact version (verified: with
# `wasm-bindgen 0.2.126` in Cargo.lock it fetched CLI 0.2.126), and a CLI/crate
# mismatch fails loudly with the "rust wasm file schema version" error rather
# than silently producing a bad artefact. So Cargo.lock IS the CLI pin.
#
# MEASURED, so nobody builds a byte-comparison gate on a false premise:
#   * Same machine, twice → BYTE-IDENTICAL.
#   * Two Linux hosts with DIFFERENT `CARGO_HOME` and different workspace paths →
#     BYTE-IDENTICAL, once the remap below is applied. This is what makes a digest
#     comparison a usable gate rather than a machine-specific accident.
#   * Windows host vs a Linux container, same pinned rustc AND wasm-pack →
#     still NOT byte-identical, and the remap does not fix it (nor does `--no-opt`,
#     so it is not `wasm-opt` either): rustc's own codegen differs by HOST, and
#     `--remap-path-prefix` replaces only the prefix — the remainder keeps the
#     host's separators (`a\b` vs `a/b`). Same honest limit e15 records for the
#     plugins.
#
# So **Linux is canonical**: it is what the Dockerfile and the CI runner build
# with, and the committed guests are Linux builds. To reproduce them off Linux:
#
#     docker run --rm -v "$PWD:/w" -w /w rust:1.98.1-bookworm bash scripts/build-wasm.sh
#
# A digest gate is therefore sound on the runner. Keep the behavioural checks
# beside it, because they catch a different failure: the digest catches a STALE
# guest, the tests catch one that is fresh but broken, and only the tests survive
# a deliberate toolchain bump. They live in apps/web/src/crypto/sanitize.test.ts
# and wasm-crypto.test.ts (both drive the committed bytes directly) and
# apps/web/e2e/crypto-pgp.spec.ts (drives them through the real worker).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT_DIR="${ROOT}/apps/web/src/wasm"

# Pinned; see the note above. Bump deliberately, and re-commit both guests in the
# same change (the ABI moves with it).
WASM_PACK_VERSION="0.15.0"

# Build-input normalisation, shared with the first-party plugins (t24-e15). It
# remaps `$CARGO_HOME/registry/src` and the workspace root to constants, so the
# bytes stop being a function of WHERE the build ran. That is strictly stronger
# than pinning `CARGO_HOME`: pinning only makes two builds agree when both happen
# to use the same value, whereas remapping makes them agree across different ones
# — verified for these guests with `CARGO_HOME=/usr/local/cargo` at `/tmp/ws` vs
# `CARGO_HOME=/tmp/ch-alt` at `/other/path/deep`, identical digests, and zero
# embedded cargo paths in either (they had 40 in mw-sanitize and 302 in mw-crypto).
#
# It discovers the workspace root by walking up rather than assuming a depth,
# because a wrong prefix does not fail — it silently matches nothing and leaves the
# paths embedded.
. "${ROOT}/plugins/reproducible-env.sh"

# CAREFUL: reproducible-env.sh sets CARGO_ENCODED_RUSTFLAGS, and that variable
# OVERRIDES RUSTFLAGS entirely — cargo reads one or the other, never both. The
# `--cfg` below therefore has to be appended to the encoded form; setting RUSTFLAGS
# here (as this script used to) would silently drop it.
#
# rPGP/RustCrypto reach getrandom's JS backend on wasm32 via plain crate features
# (mw-crypto's Cargo.toml wasm target deps), so no `--cfg getrandom_backend` is
# strictly required; we still pass it for older getrandom generations' safety.
# `\037` is the 0x1f separator the encoded form uses, and `--cfg` and its value are
# two separate arguments to rustc, hence two elements.
CARGO_ENCODED_RUSTFLAGS="${CARGO_ENCODED_RUSTFLAGS}$(printf '\037')--cfg$(printf '\037')getrandom_backend=\"wasm_js\""
export CARGO_ENCODED_RUSTFLAGS

if ! command -v wasm-pack >/dev/null 2>&1; then
  echo "wasm-pack not found. Install: cargo install wasm-pack --version ${WASM_PACK_VERSION} --locked" >&2
  exit 1
fi
have="$(wasm-pack --version 2>/dev/null | awk '{print $2}')"
if [ "${have}" != "${WASM_PACK_VERSION}" ]; then
  echo "wasm-pack ${have:-unknown} found, but these artefacts are pinned to ${WASM_PACK_VERSION}." >&2
  echo "A different wasm-pack emits a different glue ABI, so the committed guests would" >&2
  echo "change wholesale. Install the pin:" >&2
  echo "  cargo install wasm-pack --version ${WASM_PACK_VERSION} --locked --force" >&2
  echo "Set MW_WASM_PACK_ANY=1 to override deliberately (e.g. when bumping the pin)." >&2
  [ "${MW_WASM_PACK_ANY:-}" = "1" ] || exit 1
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
