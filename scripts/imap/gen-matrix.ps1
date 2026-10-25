# Regenerate docs/protocols/imap-extensions.md from the compliance table in
# crates/mw-imap/src/extensions.rs. Windows counterpart of gen-matrix.sh.
#
# The test crates/mw-imap/tests/extensions_doc.rs compares the document with
# the table; with MW_WRITE_IMAP_MATRIX=1 it writes the document instead.
#
# Usage: pwsh scripts/imap/gen-matrix.ps1
$ErrorActionPreference = 'Stop'

Set-Location (Join-Path $PSScriptRoot '..\..')
$env:MW_WRITE_IMAP_MATRIX = '1'
try {
    cargo test --locked -p mw-imap --test extensions_doc -- --quiet
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
} finally {
    Remove-Item Env:MW_WRITE_IMAP_MATRIX
}
Write-Host 'wrote docs/protocols/imap-extensions.md'
