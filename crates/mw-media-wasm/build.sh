#!/usr/bin/env sh
# Reproducibly (re)build the second-layer media-jail guest and refresh the
# committed `media.wasm`. Run this when `src/lib.rs` changes. The committed module
# is what `mw-render` loads via `include_bytes!`, so `cargo test -p mw-render` is
# green everywhere without a wasm toolchain.
#
# Target `wasm32-unknown-unknown`: a pure CORE module with NO host imports — the
# strongest jail posture (the guest cannot do I/O at all). No component tooling.
#
# ── Why the path remapping below (t24-e15) ───────────────────────────────────
# "Reproducibly" was aspirational until 26.20. Two inputs made the output differ
# per builder, and both are fixed here:
#
#   1. The toolchain. `rust-toolchain.toml` said `channel = "stable"` until t24-e2
#      pinned it, so "stable" meant whatever the builder had last updated to. The
#      committed guest was built by a rustc <= 1.95, which exported two extra
#      linker-internal globals (`__data_end`, `__heap_base`); 1.96 stopped
#      exporting them by default, so a 1.98.1 rebuild produced a different symbol
#      set and the `wasm-plugin-build` CI gate went red. Verified by bisection:
#      1.88.0 and 1.95.0 emit 6 exports, 1.97.0 and 1.98.1 emit 4. Nothing in this
#      crate's own build inputs changed — no wasm-opt runs on this target, no
#      RUSTFLAGS, and Cargo.lock pins every dependency.
#
#   2. Absolute paths baked into the artifact. `strip = true` removes debuginfo
#      but NOT `core::panic::Location` file strings, so the guest embedded 50
#      absolute paths from the builder's Cargo registry — the committed module
#      literally shipped `C:\Users\<name>\.cargo\registry\...` to every user. That
#      made the bytes a function of the builder's home directory, which is why the
#      CI job could only ever gate on the symbol set and had to downgrade the
#      digest difference to a warning.
#
# With `--remap-path-prefix` for both the registry and this crate's own directory,
# two builds from different source paths and different CARGO_HOMEs produce a
# byte-identical module, so the digest becomes a real supply-chain gate.
#
# HONEST LIMIT: reproducible **per platform**, not across them. Remapping replaces
# the prefix; the remainder keeps the host's path separators (`a\b` vs `a/b`), so a
# Windows build and a Linux build still differ. Linux is canonical here — it is
# what `Dockerfile` (`rust:1.98.1-bookworm`) and the `wasm-plugin-build` CI job
# build with, and the committed `media.wasm` is a Linux build. Rebuild it the way
# CI does if you are not on Linux:
#
#   docker run --rm -v "$PWD:/w" -w /w rust:1.98.1-bookworm sh build.sh
set -eu
cd "$(dirname "$0")"

rustup target add wasm32-unknown-unknown >/dev/null 2>&1 || true

# Native-form absolute paths, because rustc compares the remap prefix against the
# paths IT sees. Under MSYS/git-bash `pwd` yields `/f/...` while rustc is handed
# `F:\...`, so convert when cygpath is available or the remap silently no-ops.
native_path() {
  if command -v cygpath >/dev/null 2>&1; then cygpath -w "$1"; else printf '%s' "$1"; fi
}
CRATE_DIR="$(native_path "$(pwd)")"
REGISTRY="$(native_path "${CARGO_HOME:-$HOME/.cargo}/registry/src")"

# CARGO_ENCODED_RUSTFLAGS rather than RUSTFLAGS: it is separated by 0x1f, so a
# builder whose home directory contains a space does not silently get the flag
# split in half. It overrides RUSTFLAGS, which is intended — the artifact must not
# depend on the caller's ambient flags either.
CARGO_ENCODED_RUSTFLAGS="$(printf -- '--remap-path-prefix=%s=/cargo\037--remap-path-prefix=%s=/src' \
  "$REGISTRY" "$CRATE_DIR")"
export CARGO_ENCODED_RUSTFLAGS

# --locked: Cargo.lock is one of the reproducibility inputs. Without it a fresh
# minor release of `image` or `cfb` would change the bytes with nothing in the
# repository recording that it had.
cargo build --locked --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/mw_media_wasm.wasm media.wasm
echo "refreshed crates/mw-media-wasm/media.wasm"
