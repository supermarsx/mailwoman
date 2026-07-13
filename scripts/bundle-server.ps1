# Bundle the release `mw-server` binary into the desktop shell resources for
# SELF-CONTAINED mode (§4.1 / plan §3 e3). Windows counterpart of
# bundle-server.sh. The engine is spawned as a SIBLING PROCESS, never linked in
# (SPEC §16). `tauri.conf.json` ships `resources/mw-server*`; the shell resolves it
# at runtime (selfcontained.rs `resolve_bundled_server`). Run before `tauri build`.
$ErrorActionPreference = 'Stop'
Set-Location (Join-Path $PSScriptRoot '..')

Write-Host '[bundle-server] building release mw-server...'
cargo build --release -p mw-server

# The `mw-server` crate's binary is named `mailwoman` ([[bin]] name); rename it to
# the stable resource name `mw-server.exe` the shell resolves at runtime.
$bin = 'target/release/mailwoman.exe'
if (-not (Test-Path $bin)) { throw "[bundle-server] $bin not found after build" }

$dest = 'apps/desktop/src-tauri/resources'
New-Item -ItemType Directory -Force -Path $dest | Out-Null
Copy-Item -Force $bin (Join-Path $dest 'mw-server.exe')

$sizeMb = [int]((Get-Item $bin).Length / 1MB)
Write-Host "[bundle-server] copied mailwoman.exe -> $dest/mw-server.exe ($sizeMb MB)"
if ($sizeMb -gt 40) {
    Write-Warning "[bundle-server] bundled mw-server is $sizeMb MB (> 40 MB self-contained budget, §16)"
}
