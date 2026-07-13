# Build the browser WASM crypto bundle on Windows (plan §2.5 / §3 e8; §1.13 Win+
# Linux both build). Windows-dev twin of `build-wasm.sh` — see it for the full
# rationale. Compiles `mw-crypto` (+ `mw-sanitize`) to wasm32 via wasm-pack into
# `apps/web/src/wasm/`. e0 scaffold; e1 fills the crypto, e8 wires the worker.
$ErrorActionPreference = 'Stop'

$root = Resolve-Path (Join-Path $PSScriptRoot '..')
$outDir = Join-Path $root 'apps/web/src/wasm'

# getrandom's JS backend on wasm32 is selected by this cfg (getrandom 0.3+/0.4),
# NOT a cargo feature — required once e1 pulls in real RNG (rPGP keygen).
$env:RUSTFLAGS = "$($env:RUSTFLAGS) --cfg getrandom_backend=`"wasm_js`""

if (-not (Get-Command wasm-pack -ErrorAction SilentlyContinue)) {
  Write-Error 'wasm-pack not found. Install: cargo install wasm-pack'
  exit 1
}
rustup target add wasm32-unknown-unknown 2>$null | Out-Null

Write-Host "building mw-crypto -> $outDir/mw-crypto"
wasm-pack build (Join-Path $root 'crates/mw-crypto') `
  --target web --out-dir (Join-Path $outDir 'mw-crypto') --out-name mw_crypto `
  -- --features wasm

Write-Host "building mw-sanitize -> $outDir/mw-sanitize"
wasm-pack build (Join-Path $root 'crates/mw-sanitize') `
  --target web --out-dir (Join-Path $outDir 'mw-sanitize') --out-name mw_sanitize

Write-Host "wasm bundle built into $outDir"
