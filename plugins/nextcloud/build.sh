#!/usr/bin/env sh
# Reproducibly (re)build the Nextcloud `wasm32-wasip2` COMPONENT and refresh the
# committed fixture the host test (`tests/share_link.rs`) loads. Run this when
# `crates/mw-plugin/wit/plugin.wit` or `src/component.rs` changes; e15 wires it into
# the `wasm32-wasip2` plugin-build CI job.
set -eu
cd "$(dirname "$0")"

rustup target add wasm32-wasip2 >/dev/null 2>&1 || true
cargo build -p nextcloud-plugin --target wasm32-wasip2 --release
mkdir -p tests/fixtures
cp ../../target/wasm32-wasip2/release/nextcloud_plugin.wasm tests/fixtures/nextcloud.wasm
echo "refreshed plugins/nextcloud/tests/fixtures/nextcloud.wasm"
