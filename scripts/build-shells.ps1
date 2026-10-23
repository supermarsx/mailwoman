# Build the Mailwoman thin shells (plan §3 e7 — mount/wire + build glue). Windows
# counterpart of build-shells.sh. See that file for the full rationale: ONE UI
# bundle, built once (apps/web), consumed by both the server and the shells. The
# 1→2→4 order matters: the shell compiles the emitted bundle-hash.json into the
# binary AND embeds the same dist, then verifies them at launch (the §7.4 gate).
$ErrorActionPreference = 'Stop'
Set-Location (Join-Path $PSScriptRoot '..')

# `$ErrorActionPreference = 'Stop'` covers cmdlet errors, NOT the exit code of a
# native command: `pnpm`, `node` and `tauri` could fail and this script would run
# on and exit 0. t24-e11 found the Windows desktop-shell CI leg green that way
# while check-bundle-size.mjs was printing budget failures. Hence a $LASTEXITCODE
# check after every native call below. (A wrapper function is not used: arguments
# like `-e` bind to the wrapper's own common parameters instead of being passed on.)
function Assert-LastExit([string]$Label) {
  if ($LASTEXITCODE -ne 0) {
    throw "[build-shells] $Label failed with exit code $LASTEXITCODE"
  }
}

# Per deployment (t32, docs/deploy/shells.md; the same variables as build-shells.sh):
#   MW_SHELL_SERVER_URL       required: the server this build talks to, compiled in
#                             (https://host[:port][/path]; http only to localhost,
#                             127.x or [::1]). A desktop provisioning file
#                             (server.json) takes precedence at run time.
#   MW_SHELL_NO_SERVER_URL=1  build without one; the app then needs a provisioning file.
#   MW_SHELL_PRODUCT_NAME     the app name (window title, bundle name); default Mailwoman.
#   MW_SHELL_IDENTIFIER       the desktop bundle identifier; default com.mailwoman.desktop.
# A first check before the long SPA build; the shell crates check the server
# address again at compile time and refuse anything else.
if ($env:MW_SHELL_SERVER_URL) {
  $url = $env:MW_SHELL_SERVER_URL
  $ok = ($url -notmatch '[\s@?#\\]') -and (
    ($url -cmatch '^https://[^/:]') -or
    ($url -cmatch '^http://(localhost|127\.[0-9.]+|\[::1\])([:/]|$)'))
  if (-not $ok) {
    throw "[build-shells] MW_SHELL_SERVER_URL must be https://host[:port][/path] (http only to localhost, 127.x or [::1]), with no spaces, @, ?, # or backslash: $url"
  }
  Write-Host "[build-shells] server address: $url"
} elseif ($env:MW_SHELL_NO_SERVER_URL -eq '1') {
  Remove-Item Env:MW_SHELL_SERVER_URL -ErrorAction SilentlyContinue
  Write-Host '[build-shells] no server address (MW_SHELL_NO_SERVER_URL=1): the app needs a provisioning file'
} else {
  throw '[build-shells] set MW_SHELL_SERVER_URL=https://mail.example.org (the server this build talks to), or MW_SHELL_NO_SERVER_URL=1 for a build provisioned only by server.json (docs/deploy/shells.md)'
}

# Branding: a tauri config overlay, and the environment for the shell crates
# (window title).
$tauriConfigArgs = @()
$tauriConfigFile = $null
if ($env:MW_SHELL_PRODUCT_NAME -or $env:MW_SHELL_IDENTIFIER) {
  $overlay = [ordered]@{}
  if ($env:MW_SHELL_PRODUCT_NAME) {
    $name = $env:MW_SHELL_PRODUCT_NAME
    if ($name.Length -gt 64 -or $name -match '[\p{Cc}/\\]') {
      throw '[build-shells] MW_SHELL_PRODUCT_NAME must be at most 64 characters with no control characters, / or backslash'
    }
    $overlay.productName = $name
  }
  if ($env:MW_SHELL_IDENTIFIER) {
    if ($env:MW_SHELL_IDENTIFIER -cnotmatch '^[A-Za-z][A-Za-z0-9-]*(\.[A-Za-z][A-Za-z0-9-]*)+$') {
      throw "[build-shells] MW_SHELL_IDENTIFIER must be reverse DNS, such as org.example.mail: $($env:MW_SHELL_IDENTIFIER)"
    }
    $overlay.identifier = $env:MW_SHELL_IDENTIFIER
  }
  $tauriConfigFile = [System.IO.Path]::GetTempFileName()
  [System.IO.File]::WriteAllText($tauriConfigFile, ($overlay | ConvertTo-Json -Compress))
  $tauriConfigArgs = @('--config', $tauriConfigFile)
  Write-Host "[build-shells] branding: $(Get-Content -Raw $tauriConfigFile)"
}

# Include the self-contained mw-server unless MW_SELF_CONTAINED=0.
$selfContained = if ($env:MW_SELF_CONTAINED) { $env:MW_SELF_CONTAINED } else { '1' }

Write-Host '[build-shells] 1/5 building shared SPA (apps/web)...'
pnpm -C apps/web install --frozen-lockfile
Assert-LastExit 'pnpm -C apps/web install'
pnpm -C apps/web build
Assert-LastExit 'pnpm -C apps/web build'

Write-Host '[build-shells] 2/5 emitting UI-bundle integrity hash (SPEC 7.4)...'
node scripts/emit-bundle-hash.mjs
Assert-LastExit 'scripts/emit-bundle-hash.mjs'

Write-Host '[build-shells] 3/5 bundling mw-server for self-contained mode...'
if ($selfContained -eq '1') {
  & "$PSScriptRoot/bundle-server.ps1"
} else {
  Write-Host '  (skipped: MW_SELF_CONTAINED=0 — thin shell without the bundled engine)'
}

Write-Host '[build-shells] 4/5 building desktop shell...'
pnpm -C apps/desktop install
Assert-LastExit 'pnpm -C apps/desktop install'
try {
  pnpm -C apps/desktop exec tauri build --no-bundle @tauriConfigArgs
  Assert-LastExit 'tauri build'
} finally {
  if ($tauriConfigFile) { Remove-Item $tauriConfigFile -ErrorAction SilentlyContinue }
}

Write-Host '[build-shells] 5/5 checking bundle-size budgets (SPEC 16)...'
node scripts/check-bundle-size.mjs
Assert-LastExit 'scripts/check-bundle-size.mjs'

# Android needs Android SDK+NDK+JDK; this machine has SDK+NDK but no JDK on PATH
# (plan risk #1) — the APK build is the e8 CI gate (MW_SHELL_SERVER_URL and
# MW_SHELL_PRODUCT_NAME apply the same way; see docs/deploy/shells.md):
#   pnpm -C apps/mobile install
#   pnpm -C apps/mobile exec tauri android init --ci
#   pnpm -C apps/mobile exec tauri android build --apk
Write-Host '[build-shells] done (Android APK is the CI gate — see docs/deploy).'
