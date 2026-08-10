# mw-search p95 latency gate (SPEC §23). Windows counterpart of search-bench.sh:
# builds the synthetic 100k-document index and asserts p95 query latency < 50 ms.
#
# The gate itself lives in the Rust test `crates/mw-search/tests/bench.rs`
# (`p95_under_50ms_over_100k`, `#[ignore]`), which `assert!`s p95 < 50 ms — so a
# regression fails the test and therefore this script. This wrapper runs it in
# release, echoes the measured p95 for the log/trend, and propagates the exit code.
#
# Usage: pwsh scripts/search-bench.ps1
$ErrorActionPreference = 'Stop'
Set-Location (Join-Path $PSScriptRoot '..')

Write-Host '[search-bench] building + running the 100k p95 gate (release)...'
# Do not let a non-zero cargo exit terminate the script before the p95 line is echoed.
$ErrorActionPreference = 'Continue'
$output = cargo test -p mw-search --release --test bench -- --ignored --nocapture 2>&1
$status = $LASTEXITCODE
$output | ForEach-Object { Write-Host $_ }
$ErrorActionPreference = 'Stop'

# Surface the timing line so the run's p95 is visible at a glance / trendable.
$p95 = $output | Select-String -Pattern '^\s*p95\s*:' | Select-Object -First 1
if ($p95) {
    # "  p95 : 2.825 ms" -> "measured : 2.825 ms"
    Write-Host "[search-bench] measured$(($p95.Line -split 'p95', 2)[1])"
}

if ($status -ne 0) {
    Write-Error '[search-bench] FAIL: p95 gate (<50 ms over 100k) not met - see above'
    exit $status
}
Write-Host '[search-bench] OK: p95 < 50 ms over 100k (SPEC §23)'
