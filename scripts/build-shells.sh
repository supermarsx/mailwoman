#!/usr/bin/env bash
# Build the Mailwoman thin shells (plan §3 e0 scaffold → e7 fills).
#
# The ONE UI bundle is built once (apps/web) and BOTH the server (rust-embed) and
# the shells (Tauri frontendDist) consume it — no forked UI. This orchestrates:
#   1. build the shared SPA           (pnpm -C apps/web build)
#   2. emit the UI-bundle hash (§7.4) (node scripts/emit-bundle-hash.mjs)
#   3. build the desktop shell        (tauri build)  [+ self-contained variant, e3]
#   4. build the Android APK          (tauri android build)  [e7, needs SDK/NDK+JDK]
#
# e0 ships this as a stub with the steps sequenced; e7 fills the self-contained
# bundling (scripts/bundle-server.sh) + the size gate + the Android/updater wiring.
set -euo pipefail
cd "$(dirname "$0")/.."

echo "[build-shells] 1/3 building shared SPA (apps/web)…"
pnpm -C apps/web install --frozen-lockfile
pnpm -C apps/web build

echo "[build-shells] 2/3 emitting UI-bundle integrity hash (§7.4)…"
node scripts/emit-bundle-hash.mjs

echo "[build-shells] 3/3 building desktop shell…"
pnpm -C apps/desktop install
# e3 bundles the release mw-server as a resource for self-contained mode first:
#   ./scripts/bundle-server.sh
pnpm -C apps/desktop exec tauri build --no-bundle   # e7/e8: enable installer bundling

# Android (e7): needs Android SDK+NDK+JDK; documented CI-only gate on this machine.
#   pnpm -C apps/mobile install
#   pnpm -C apps/mobile exec tauri android init --ci
#   pnpm -C apps/mobile exec tauri android build --apk

echo "[build-shells] done."
