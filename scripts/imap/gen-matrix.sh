#!/usr/bin/env bash
# Regenerate docs/protocols/imap-extensions.md from the compliance table in
# crates/mw-imap/src/extensions.rs.
#
# The test crates/mw-imap/tests/extensions_doc.rs compares the document with
# the table; with MW_WRITE_IMAP_MATRIX=1 it writes the document instead.
#
# Usage: scripts/imap/gen-matrix.sh
set -euo pipefail

root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$root"
MW_WRITE_IMAP_MATRIX=1 cargo test --locked -p mw-imap --test extensions_doc -- --quiet
echo "wrote docs/protocols/imap-extensions.md"
