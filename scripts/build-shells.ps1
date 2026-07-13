# Build the Mailwoman thin shells (plan §3 e7 — mount/wire + build glue). Windows
# counterpart of build-shells.sh. See that file for the full rationale: ONE UI
# bundle, built once (apps/web), consumed by both the server and the shells. The
# 1→2→4 order matters: the shell compiles the emitted bundle-hash.json into the
# binary AND embeds the same dist, then verifies them at launch (the §7.4 gate).
$ErrorActionPreference = 'Stop'
Set-Location (Join-Path $PSScriptRoot '..')

# Include the self-contained mw-server unless MW_SELF_CONTAINED=0.
$selfContained = if ($env:MW_SELF_CONTAINED) { $env:MW_SELF_CONTAINED } else { '1' }

Write-Host '[build-shells] 1/5 building shared SPA (apps/web)...'
pnpm -C apps/web install --frozen-lockfile
pnpm -C apps/web build

Write-Host '[build-shells] 2/5 emitting UI-bundle integrity hash (SPEC 7.4)...'
node scripts/emit-bundle-hash.mjs

Write-Host '[build-shells] 3/5 bundling mw-server for self-contained mode...'
if ($selfContained -eq '1') {
  & "$PSScriptRoot/bundle-server.ps1"
} else {
  Write-Host '  (skipped: MW_SELF_CONTAINED=0 — thin shell without the bundled engine)'
}

Write-Host '[build-shells] 4/5 building desktop shell...'
pnpm -C apps/desktop install
pnpm -C apps/desktop exec tauri build --no-bundle

Write-Host '[build-shells] 5/5 checking bundle-size budgets (SPEC 16)...'
node scripts/check-bundle-size.mjs

# Android needs Android SDK+NDK+JDK; this machine has SDK+NDK but no JDK on PATH
# (plan risk #1) — the APK build is the e8 CI gate:
#   pnpm -C apps/mobile install
#   pnpm -C apps/mobile exec tauri android init --ci
#   pnpm -C apps/mobile exec tauri android build --apk
Write-Host '[build-shells] done (Android APK is the CI gate — see docs/deploy).'
