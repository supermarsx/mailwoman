#!/usr/bin/env sh
# Reproducibly (re)build the EWS bridge COMPONENT and refresh the committed fixture
# `fixtures/bridge-ews.wasm` that the host integration test (`tests/bridge.rs`)
# loads through `mw-plugin`. Run this when `src/guest.rs` or the shared
# `crates/mw-plugin/wit/plugin.wit` ABI changes. e15 wires this into the
# `wasm32-wasip2` plugin-build CI job.
#
# The `wasm32-wasip2` target's linker (`wasm-component-ld`) componentizes the cdylib
# automatically from wit-bindgen's embedded `component-type` sections — the output is
# already a real component (magic `00 61 73 6d 0d 00 01 00`); no `wasm-tools
# component new` step is needed.
set -eu
cd "$(dirname "$0")"

rustup target add wasm32-wasip2 >/dev/null 2>&1 || true
# Build just this package for wasm (dev-deps like mw-plugin/wasmtime are NOT built
# for the wasm target — they are host-only test deps).
cargo build -p bridge-ews --target wasm32-wasip2 --release
cp ../../target/wasm32-wasip2/release/bridge_ews.wasm fixtures/bridge-ews.wasm
echo "refreshed plugins/bridge-ews/fixtures/bridge-ews.wasm"
