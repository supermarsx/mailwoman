#!/usr/bin/env sh
# Reproducibly (re)build the LanguageTool `wasm32-wasip2` COMPONENT and refresh the
# committed fixture the host jail test (`tests/jail_load.rs`) + e16 load. Run this
# when `crates/mw-plugin/wit/plugin.wit` or `src/component.rs` changes; e15 wires it
# into the `wasm32-wasip2` plugin-build CI job.
#
# The `wasm32-wasip2` linker (`wasm-component-ld`) componentizes the cdylib
# automatically from wit-bindgen's embedded `component-type` sections — the output is
# already a real component, no `wasm-tools component new` step required.
set -eu
cd "$(dirname "$0")"

rustup target add wasm32-wasip2 >/dev/null 2>&1 || true
cargo build -p languagetool --target wasm32-wasip2 --release
mkdir -p tests/fixtures
cp ../../target/wasm32-wasip2/release/languagetool.wasm tests/fixtures/languagetool.wasm
echo "refreshed plugins/languagetool/tests/fixtures/languagetool.wasm"
