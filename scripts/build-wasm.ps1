# Build the browser WASM bundles on Windows (plan §2.5 / §3 e8; §1.13 Win+Linux both
# build). Windows-dev twin of `build-wasm.sh` — see it for the full rationale.
# Compiles `mw-crypto` AND `mw-sanitize` to wasm32 via wasm-pack (target `web`) into
# `apps/web/src/wasm/`, then prunes wasm-pack's stray `package.json`/`.gitignore`.
# e0 scaffold; e1 crypto; e8 wires the worker; e8b adds the mw-sanitize wasm surface
# so decrypted E2EE HTML is sanitized IN-WORKER, never on the server (plan §1.3).
$ErrorActionPreference = 'Stop'

$root = Resolve-Path (Join-Path $PSScriptRoot '..')
$outDir = Join-Path $root 'apps/web/src/wasm'

# Pinned, and kept in sync with build-wasm.sh — which carries the full rationale
# and the measured reproducibility results. Short version (t24-e13): wasm-pack
# picks the flags given to wasm-bindgen, so a different wasm-pack changes the glue
# ABI and rewrites both committed guests. The wasm-bindgen CLI needs no separate
# pin — wasm-pack resolves it from Cargo.lock. Rebuilds are byte-identical on the
# SAME machine but NOT across Windows/Linux, so never gate CI on a byte compare.
$WasmPackVersion = '0.15.0'

# See build-wasm.sh: getrandom's JS backend is selected by mw-crypto's wasm crate
# features; the cfg is exported for older getrandom generations' safety.
$env:RUSTFLAGS = "$($env:RUSTFLAGS) --cfg getrandom_backend=`"wasm_js`""

if (-not (Get-Command wasm-pack -ErrorAction SilentlyContinue)) {
  Write-Error "wasm-pack not found. Install: cargo install wasm-pack --version $WasmPackVersion --locked"
  exit 1
}
$have = (wasm-pack --version) -split ' ' | Select-Object -Last 1
if ($have -ne $WasmPackVersion) {
  Write-Host "wasm-pack $have found, but these artefacts are pinned to $WasmPackVersion." -ForegroundColor Yellow
  Write-Host 'A different wasm-pack emits a different glue ABI, so the committed guests' -ForegroundColor Yellow
  Write-Host 'would change wholesale. Install the pin:' -ForegroundColor Yellow
  Write-Host "  cargo install wasm-pack --version $WasmPackVersion --locked --force" -ForegroundColor Yellow
  Write-Host 'Set MW_WASM_PACK_ANY=1 to override deliberately (e.g. when bumping the pin).'
  if ($env:MW_WASM_PACK_ANY -ne '1') { exit 1 }
}
rustup target add wasm32-unknown-unknown 2>$null | Out-Null

Write-Host "building mw-crypto -> $outDir/mw-crypto"
wasm-pack build (Join-Path $root 'crates/mw-crypto') `
  --target web --out-dir (Join-Path $outDir 'mw-crypto') --out-name mw_crypto `
  -- --features wasm

# Prune wasm-pack's publish package.json + .gitignore so the module imports
# cleanly from src/ and the bundle is committed.
Remove-Item -Force -ErrorAction SilentlyContinue `
  (Join-Path $outDir 'mw-crypto/package.json'), (Join-Path $outDir 'mw-crypto/.gitignore')

Write-Host "wasm bundle built into $outDir/mw-crypto"

# mw-sanitize → in-worker sanitize of decrypted E2EE HTML (plan §1.3 / risk #5).
Write-Host "building mw-sanitize -> $outDir/mw-sanitize"
wasm-pack build (Join-Path $root 'crates/mw-sanitize') `
  --target web --out-dir (Join-Path $outDir 'mw-sanitize') --out-name mw_sanitize

Remove-Item -Force -ErrorAction SilentlyContinue `
  (Join-Path $outDir 'mw-sanitize/package.json'), (Join-Path $outDir 'mw-sanitize/.gitignore')

Write-Host "wasm bundle built into $outDir/mw-sanitize"
