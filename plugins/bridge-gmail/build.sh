#!/usr/bin/env sh
# Reproducibly (re)build the Gmail bridge COMPONENT and refresh the committed
# artifact at `fixtures/bridge-gmail.wasm`. Run this when the bridge source or the
# frozen `crates/mw-plugin/wit/plugin.wit` changes. e15 wires this into the
# `wasm32-wasip2` plugin-build CI job so the committed component stays in sync.
#
# The `wasm32-wasip2` target's linker (`wasm-component-ld`) componentizes the
# cdylib automatically using wit-bindgen's embedded `component-type` sections — the
# output is already a real component, no `wasm-tools component new` step required.
set -eu
cd "$(dirname "$0")/../.."   # workspace root (bridge is a workspace member)

rustup target add wasm32-wasip2 >/dev/null 2>&1 || true
cargo build -p bridge-gmail --target wasm32-wasip2 --release
cp target/wasm32-wasip2/release/bridge_gmail.wasm \
   plugins/bridge-gmail/fixtures/bridge-gmail.wasm
echo "refreshed plugins/bridge-gmail/fixtures/bridge-gmail.wasm"
