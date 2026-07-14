#!/usr/bin/env sh
# Reproducibly (re)build the host-test guest COMPONENT and refresh the committed
# fixture. Run this when `crates/mw-plugin/wit/plugin.wit` changes. e15 wires this
# into the `wasm32-wasip2` plugin-build CI job so the committed `guest.wasm` stays
# in sync with the WIT.
#
# The `wasm32-wasip2` target's linker (`wasm-component-ld`) componentizes the
# cdylib automatically using wit-bindgen's embedded `component-type` sections — the
# output is already a real component (magic `00 61 73 6d 0d 00 01 00`), no
# `wasm-tools component new` step required.
set -eu
cd "$(dirname "$0")"

rustup target add wasm32-wasip2 >/dev/null 2>&1 || true
cargo build --target wasm32-wasip2 --release
cp target/wasm32-wasip2/release/mw_plugin_guest_fixture.wasm ../fixtures/guest.wasm
echo "refreshed crates/mw-plugin/tests/fixtures/guest.wasm"
