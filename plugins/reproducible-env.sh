#!/usr/bin/env sh
# Shared build-input normalisation for the first-party plugin components (t24-e15).
#
# Source this from a `plugins/<id>/build.sh`, with the path that is correct for
# wherever that script has `cd`'d to — the scripts are not consistent about it:
#
#     . ../reproducible-env.sh          # from plugins/<id>      (most of them)
#     . plugins/reproducible-env.sh     # from the workspace root (gmail, graph)
#
# The workspace root itself is discovered by walking up, so it does NOT depend on
# the caller's cwd. Getting that wrong is silent: the remap prefix simply never
# matches and the paths stay embedded, with the build still succeeding.
#
# ── Why ──────────────────────────────────────────────────────────────────────
# `strip`/`--release` do not remove `core::panic::Location` file strings, so every
# component built before 26.20 embedded the absolute paths of whoever built it.
# The shipped `plugins/dist/bridge-gmail.wasm` and `languagetool.wasm` literally
# carried `C:\Users\<name>\.cargo\registry\src\...`, and languagetool also carried
# `C:\Users\<name>\.rustup\toolchains\...`, out to every user. Two consequences,
# and the second is the one that cost time:
#
#   1. A developer's username ships inside a binary, for no reason.
#   2. The bytes are a function of the builder's home directory, so NO two machines
#      ever produce the same component. That is why the CI gate could only compare
#      capability surfaces, and why `wasm-plugin-build`'s first red looked like five
#      artifacts had drifted when none had — the runner's rebuild of a fixture
#      necessarily differs from the committed one, and nothing could tell that
#      apart from real drift.
#
# With the registry and workspace paths remapped, a build with the pinned rustc is
# deterministic, so the comparison can be the digest itself. `crates/mw-media-wasm`
# got this treatment first; its header carries the full rationale and the evidence.
#
# HONEST LIMIT: reproducible per platform, not across them. `--remap-path-prefix`
# replaces the prefix and the remainder keeps the host's separators (`a\b` vs
# `a/b`), so a Windows build still differs from a Linux one. Linux is canonical —
# it is what the Dockerfile and the CI runner build with. To reproduce the
# committed artifacts off Linux:
#
#     docker run --rm -v "$PWD:/w" -w /w rust:1.98.1-bookworm sh plugins/<id>/build.sh
#
# Sourced, not executed, so it deliberately does not `set -e` — the caller already
# has `set -eu` and sourcing must not change its shell options.

# Native-form absolute paths, because rustc matches the remap prefix against the
# paths IT sees. Under MSYS/git-bash `pwd` gives `/f/...` while rustc is handed
# `F:\...`, so convert where cygpath exists or the remap silently does nothing.
_mw_native() {
  if command -v cygpath >/dev/null 2>&1; then cygpath -w "$1"; else printf '%s' "$1"; fi
}

# Walk up to the workspace root rather than assuming a depth: some build.sh scripts
# cd into `plugins/<id>` and others cd to the root, and a wrong prefix fails silently.
_mw_find_workspace() {
  _d="$(pwd)"
  while [ "$_d" != "/" ] && [ -n "$_d" ]; do
    if [ -f "$_d/Cargo.toml" ] && grep -q '^\[workspace\]' "$_d/Cargo.toml" 2>/dev/null; then
      printf '%s' "$_d"
      return 0
    fi
    _d="$(dirname "$_d")"
  done
  return 1
}

if ! _MW_WORKSPACE_RAW="$(_mw_find_workspace)"; then
  echo "reproducible-env.sh: no workspace Cargo.toml above $(pwd); refusing to build a" >&2
  echo "  non-reproducible artifact that would embed this machine's paths." >&2
  exit 1
fi
_MW_WORKSPACE="$(_mw_native "$_MW_WORKSPACE_RAW")"
_MW_REGISTRY="$(_mw_native "${CARGO_HOME:-$HOME/.cargo}/registry/src")"

# CARGO_ENCODED_RUSTFLAGS rather than RUSTFLAGS: it is 0x1f-separated, so a builder
# whose home directory contains a space does not get the flag split in half. It
# overrides RUSTFLAGS, which is intended — a shipped artifact must not depend on
# the caller's ambient flags either.
CARGO_ENCODED_RUSTFLAGS="$(printf -- '--remap-path-prefix=%s=/cargo\037--remap-path-prefix=%s=/src' \
  "$_MW_REGISTRY" "$_MW_WORKSPACE")"
export CARGO_ENCODED_RUSTFLAGS
