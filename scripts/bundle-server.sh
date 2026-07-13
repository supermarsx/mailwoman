#!/usr/bin/env bash
# Bundle the release `mw-server` binary into the desktop shell's Tauri resources
# for SELF-CONTAINED mode (§4.1 / plan §3 e3). The shell spawns this binary as a
# SIBLING PROCESS (loopback / Unix socket) — the engine is NEVER linked into the
# shell (SPEC §16). Size budget: self-contained desktop < 40 MB (§16).
#
# `tauri.conf.json` ships `resources/mw-server*`; the shell resolves it at runtime
# (apps/desktop/src-tauri/src/selfcontained.rs `resolve_bundled_server`). Run this
# BEFORE `tauri build` (scripts/build-shells.sh sequences it).
set -euo pipefail
cd "$(dirname "$0")/.."

echo "[bundle-server] building release mw-server…"
cargo build --release -p mw-server

# The `mw-server` crate's binary is named `mailwoman` ([[bin]] name); rename it to
# the stable resource name `mw-server[.exe]` the shell resolves at runtime.
BIN="target/release/mailwoman"
OUT="mw-server"
if [ "${OS:-}" = "Windows_NT" ]; then BIN="target/release/mailwoman.exe"; OUT="mw-server.exe"; fi
if [ ! -f "$BIN" ]; then
  echo "[bundle-server] ERROR: $BIN not found after build" >&2
  exit 1
fi

DEST="apps/desktop/src-tauri/resources"
mkdir -p "$DEST"
cp -f "$BIN" "$DEST/$OUT"

SIZE_MB=$(( $(wc -c < "$BIN") / 1024 / 1024 ))
echo "[bundle-server] copied $(basename "$BIN") -> $DEST/$OUT (${SIZE_MB} MB)"
if [ "$SIZE_MB" -gt 40 ]; then
  echo "[bundle-server] WARNING: bundled mw-server is ${SIZE_MB} MB (> 40 MB self-contained budget, §16)" >&2
fi
