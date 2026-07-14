#!/usr/bin/env sh
# Reproducibly (re)build the Microsoft Graph bridge COMPONENT and refresh the
# committed fixture the in-jail test loads. Run this when the guest wiring
# (`src/guest.rs`) or the frozen WIT (`crates/mw-plugin/wit/plugin.wit`) changes.
# e15 wires this into the `wasm32-wasip2` plugin-build CI job.
#
# The `wasm32-wasip2` linker (`wasm-component-ld`) componentizes the cdylib
# automatically via wit-bindgen's embedded `component-type` sections — the output is
# already a real component (magic `00 61 73 6d 0d 00 01 00`), no `wasm-tools
# component new` step required.
set -eu
cd "$(dirname "$0")/../.."

rustup target add wasm32-wasip2 >/dev/null 2>&1 || true
cargo build -p bridge-graph --target wasm32-wasip2 --release
cp target/wasm32-wasip2/release/bridge_graph.wasm \
   plugins/bridge-graph/tests/fixtures/bridge-graph.wasm
echo "refreshed plugins/bridge-graph/tests/fixtures/bridge-graph.wasm"
