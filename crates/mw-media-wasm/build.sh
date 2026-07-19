#!/usr/bin/env sh
# Reproducibly (re)build the second-layer media-jail guest and refresh the
# committed `media.wasm`. Run this when `src/lib.rs` changes. The committed module
# is what `mw-render` loads via `include_bytes!`, so `cargo test -p mw-render` is
# green everywhere without a wasm toolchain.
#
# Target `wasm32-unknown-unknown`: a pure CORE module with NO host imports — the
# strongest jail posture (the guest cannot do I/O at all). No component tooling.
set -eu
cd "$(dirname "$0")"

rustup target add wasm32-unknown-unknown >/dev/null 2>&1 || true
cargo build --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/mw_media_wasm.wasm media.wasm
echo "refreshed crates/mw-media-wasm/media.wasm"
