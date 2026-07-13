# Build the Mailwoman thin shells (plan §3 e0 scaffold → e7 fills). Windows
# counterpart of build-shells.sh. See that file for the full rationale: ONE UI
# bundle, built once (apps/web), consumed by both the server and the shells.
$ErrorActionPreference = 'Stop'
Set-Location (Join-Path $PSScriptRoot '..')

Write-Host '[build-shells] 1/3 building shared SPA (apps/web)...'
pnpm -C apps/web install --frozen-lockfile
pnpm -C apps/web build

Write-Host '[build-shells] 2/3 emitting UI-bundle integrity hash (SPEC 7.4)...'
node scripts/emit-bundle-hash.mjs

Write-Host '[build-shells] 3/3 building desktop shell...'
pnpm -C apps/desktop install
# e3 bundles the release mw-server first: ./scripts/bundle-server.ps1
pnpm -C apps/desktop exec tauri build --no-bundle   # e7/e8: enable installer bundling

Write-Host '[build-shells] done.'
