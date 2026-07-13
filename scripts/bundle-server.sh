#!/usr/bin/env bash
# Bundle the release `mw-server` binary into the desktop shell's Tauri resources
# for SELF-CONTAINED mode (§4.1 / plan §3 e3). The shell spawns this binary as a
# SIBLING PROCESS (loopback / Unix socket) — the engine is NEVER linked into the
# shell. Size budget: self-contained desktop < 40 MB (§16).
#
# e0 ships this stub; e3 fills the copy + the tauri.conf.json `resources` entry +
# the spawn/health-probe/lifecycle in apps/desktop/src-tauri/src/selfcontained.rs.
set -euo pipefail
cd "$(dirname "$0")/.."

echo "[bundle-server] building release mw-server…"
cargo build --release -p mw-server

BIN="target/release/mw-server"
[ "${OS:-}" = "Windows_NT" ] && BIN="target/release/mw-server.exe"

DEST="apps/desktop/src-tauri/resources"
mkdir -p "$DEST"
# e3: copy the binary + declare it under tauri.conf.json `bundle.resources` and
# spawn it on serverless launch.
#   cp "$BIN" "$DEST/"
echo "[bundle-server] TODO(e3): copy $BIN into $DEST and wire the resource + spawn."
