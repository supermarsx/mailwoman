#!/usr/bin/env bash
# Build the Mailwoman thin shells (plan §3 e7 — mount/wire + build glue).
#
# The ONE UI bundle is built once (apps/web) and BOTH the server (rust-embed) and
# the shells (Tauri frontendDist) consume it — no forked UI. This orchestrates:
#   1. build the shared SPA           (pnpm -C apps/web build)
#   2. emit the UI-bundle hash (§7.4) (node scripts/emit-bundle-hash.mjs)
#   3. bundle mw-server for self-contained mode (scripts/bundle-server.sh)
#   4. build the desktop shell        (tauri build)
#   5. assert the §16 size budgets    (thin < 10 MB, self-contained < 40 MB)
#   6. build the Android APK          (tauri android build)  [needs SDK/NDK+JDK]
#
# The ORDER of 1→2→4 matters: the shell compiles the emitted bundle-hash.json into
# the binary (`include_str!`) AND embeds the same dist, then verifies them at launch
# (the §7.4 tamper gate). Re-emitting after the web build keeps the two consistent.
set -euo pipefail
cd "$(dirname "$0")/.."

# Include the self-contained mw-server in the bundle unless MW_SELF_CONTAINED=0.
SELF_CONTAINED="${MW_SELF_CONTAINED:-1}"

echo "[build-shells] 1/5 building shared SPA (apps/web)…"
pnpm -C apps/web install --frozen-lockfile
pnpm -C apps/web build

echo "[build-shells] 2/5 emitting UI-bundle integrity hash (§7.4)…"
node scripts/emit-bundle-hash.mjs

echo "[build-shells] 3/5 bundling mw-server for self-contained mode…"
if [ "$SELF_CONTAINED" = "1" ]; then
  ./scripts/bundle-server.sh
else
  echo "  (skipped: MW_SELF_CONTAINED=0 — thin shell without the bundled engine)"
fi

echo "[build-shells] 4/5 building desktop shell…"
pnpm -C apps/desktop install
# --no-bundle keeps CI fast + avoids installer tooling; e8 flips it on for installers.
pnpm -C apps/desktop exec tauri build --no-bundle

echo "[build-shells] 5/5 checking §16 bundle-size budgets…"
node scripts/check-bundle-size.mjs

# Android (needs Android SDK+NDK+JDK). This dev/CI machine has the SDK+NDK but no JDK
# on PATH (plan §1.11 / risk #1), so the APK build is the e8 CI gate, documented here:
#   pnpm -C apps/mobile install
#   pnpm -C apps/mobile exec tauri android init --ci
#   pnpm -C apps/mobile exec tauri android build --apk
echo "[build-shells] done (Android APK is the CI gate — see scripts + docs/deploy)."
