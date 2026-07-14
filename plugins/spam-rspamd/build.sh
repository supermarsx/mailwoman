#!/usr/bin/env sh
# Reproducibly (re)build the spam-rspamd `wasm32-wasip2` COMPONENT and refresh the
# committed fixture the host jail load test (`tests/jail_load.rs`) + e13/e14 load. Run
# this when `crates/mw-plugin/wit/plugin.wit` or `src/*.rs` changes.
#
# The `wasm32-wasip2` linker (`wasm-component-ld`) componentizes the cdylib
# automatically from wit-bindgen's embedded `component-type` sections — the output is
# already a real component, no `wasm-tools component new` step required.
set -eu
cd "$(dirname "$0")"

rustup target add wasm32-wasip2 >/dev/null 2>&1 || true
cargo build -p spam-rspamd --target wasm32-wasip2 --release
mkdir -p tests/fixtures
cp ../../target/wasm32-wasip2/release/spam_rspamd.wasm tests/fixtures/spam-rspamd.wasm
echo "refreshed plugins/spam-rspamd/tests/fixtures/spam-rspamd.wasm"
