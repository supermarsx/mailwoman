#!/usr/bin/env bash
# Build the Mailwoman thin shells (plan §3 e7 — mount/wire + build glue).
#
# The ONE UI bundle is built once (apps/web) and BOTH the server (rust-embed) and
# the shells (Tauri frontendDist) consume it — no forked UI. This orchestrates:
#   1. build the shared SPA           (pnpm -C apps/web build)
#   2. emit the UI-bundle hash (§7.4) (node scripts/emit-bundle-hash.mjs)
#   3. bundle mw-server for self-contained mode (scripts/bundle-server.sh)
#   4. build the desktop shell        (tauri build)
#   5. assert the §16 size budgets    (thin < 27 MB, self-contained < 204 MB)
#   6. build the Android APK          (tauri android build)  [needs SDK/NDK+JDK]
#
# The ORDER of 1→2→4 matters: the shell compiles the emitted bundle-hash.json into
# the binary (`include_str!`) AND embeds the same dist, then verifies them at launch
# (the §7.4 tamper gate). Re-emitting after the web build keeps the two consistent.
#
# Per deployment (t32, docs/deploy/shells.md):
#   MW_SHELL_SERVER_URL    the Mailwoman server the app talks to, compiled in:
#                          https://host[:port][/path] (http only to localhost,
#                          127.x or [::1], for testing). Required. The user cannot
#                          enter or change it. A desktop provisioning file
#                          (server.json) takes precedence at run time.
#   MW_SHELL_NO_SERVER_URL=1  build without one instead: the app says it has not
#                          been set up unless a desktop provisioning file names a
#                          server (fleets provisioned only by file, CI size checks).
#   MW_SHELL_PRODUCT_NAME  the app name (window title, bundle name); default
#                          Mailwoman.
#   MW_SHELL_IDENTIFIER    the desktop bundle identifier (reverse DNS); default
#                          com.mailwoman.desktop.
#   MW_SHELL_UPDATER_URL   the desktop update feed, https, normally
#                          https://host/path/{{target}}/{{arch}}/{{current_version}}
#   MW_SHELL_UPDATER_PUBKEY  the minisign public key the update bundles are signed
#                          for, as `tauri signer generate` prints it. With both set
#                          the desktop shell checks that feed; with either missing
#                          it has no updater (scripts/shells/config-overlay.mjs).
set -euo pipefail
cd "$(dirname "$0")/.."

fail() { echo "[build-shells] $*" >&2; exit 1; }

# A first check before the long SPA build; the shell crates check the value again
# at compile time and refuse anything else.
if [ -n "${MW_SHELL_SERVER_URL:-}" ]; then
  case "$MW_SHELL_SERVER_URL" in
    *[[:space:]@?#\\]*) fail "MW_SHELL_SERVER_URL must not contain spaces, @, ?, # or a backslash: $MW_SHELL_SERVER_URL" ;;
    https://[!/:]*) ;;
    http://localhost | http://localhost[:/]* | http://127.[0-9]* | "http://[::1]" | "http://[::1]"[:/]*) ;;
    *) fail "MW_SHELL_SERVER_URL must be https://host[:port][/path] (http only to localhost, 127.x or [::1]): $MW_SHELL_SERVER_URL" ;;
  esac
  export MW_SHELL_SERVER_URL
  echo "[build-shells] server address: $MW_SHELL_SERVER_URL"
elif [ "${MW_SHELL_NO_SERVER_URL:-0}" = "1" ]; then
  unset MW_SHELL_SERVER_URL
  echo "[build-shells] no server address (MW_SHELL_NO_SERVER_URL=1): the app needs a provisioning file"
else
  fail "set MW_SHELL_SERVER_URL=https://mail.example.org (the server this build talks to), or MW_SHELL_NO_SERVER_URL=1 for a build provisioned only by server.json (docs/deploy/shells.md)"
fi

# Branding and the updater: passed to tauri as a config overlay, and the branding
# to the shell crates (window title) through the environment.
TAURI_CONFIG_ARGS=()
if [ -n "${MW_SHELL_PRODUCT_NAME:-}" ]; then
  if [ "${#MW_SHELL_PRODUCT_NAME}" -gt 64 ] || printf '%s' "$MW_SHELL_PRODUCT_NAME" | LC_ALL=C grep -q '[[:cntrl:]/\]'; then
    fail "MW_SHELL_PRODUCT_NAME must be at most 64 characters with no control characters, / or a backslash"
  fi
  export MW_SHELL_PRODUCT_NAME
fi
if [ -n "${MW_SHELL_IDENTIFIER:-}" ] && ! printf '%s' "$MW_SHELL_IDENTIFIER" | grep -Eq '^[A-Za-z][A-Za-z0-9-]*(\.[A-Za-z][A-Za-z0-9-]*)+$'; then
  fail "MW_SHELL_IDENTIFIER must be reverse DNS, such as org.example.mail: $MW_SHELL_IDENTIFIER"
fi
TAURI_CONFIG_FILE="$(mktemp)"
trap 'rm -f "$TAURI_CONFIG_FILE"' EXIT
# The updater inputs are checked here, before the long SPA build.
node scripts/shells/config-overlay.mjs > "$TAURI_CONFIG_FILE" ||
  fail "fix the updater inputs above, or unset both MW_SHELL_UPDATER_URL and MW_SHELL_UPDATER_PUBKEY for a build without an updater"
if [ "$(cat "$TAURI_CONFIG_FILE")" != "{}" ]; then
  TAURI_CONFIG_ARGS=(--config "$TAURI_CONFIG_FILE")
  echo "[build-shells] config overlay: $(cat "$TAURI_CONFIG_FILE")"
fi

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
# (The ${...+...} form: an empty array is "unbound" under set -u in bash 3.2, macOS.)
pnpm -C apps/desktop exec tauri build --no-bundle ${TAURI_CONFIG_ARGS[@]+"${TAURI_CONFIG_ARGS[@]}"}

echo "[build-shells] 5/5 checking §16 bundle-size budgets…"
node scripts/check-bundle-size.mjs

# Android (needs Android SDK+NDK+JDK). This dev/CI machine has the SDK+NDK but no JDK
# on PATH (plan §1.11 / risk #1), so the APK build is the e8 CI gate, documented here
# (MW_SHELL_SERVER_URL and MW_SHELL_PRODUCT_NAME apply the same way; the Android
# identifier stays com.mailwoman.mobile, see docs/deploy/shells.md):
#   pnpm -C apps/mobile install
#   pnpm -C apps/mobile exec tauri android init --ci
#   pnpm -C apps/mobile exec tauri android build --apk
echo "[build-shells] done (Android APK is the CI gate — see scripts + docs/deploy)."
