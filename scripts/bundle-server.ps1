# Bundle the release `mw-server` binary into the desktop shell resources for
# SELF-CONTAINED mode (§4.1 / plan §3 e3). Windows counterpart of
# bundle-server.sh. The engine is spawned as a SIBLING PROCESS, never linked in.
$ErrorActionPreference = 'Stop'
Set-Location (Join-Path $PSScriptRoot '..')

Write-Host '[bundle-server] building release mw-server...'
cargo build --release -p mw-server

$dest = 'apps/desktop/src-tauri/resources'
New-Item -ItemType Directory -Force -Path $dest | Out-Null
# e3: copy target/release/mw-server.exe into $dest, declare it under
# tauri.conf.json bundle.resources, and spawn it on serverless launch.
Write-Host "[bundle-server] TODO(e3): copy target/release/mw-server.exe into $dest and wire the resource + spawn."
