#!/usr/bin/env bash
# Run a current OpenSSL (Alpine 3.22, OpenSSL 3.5) in Docker, as if it were on PATH.
#
# Why: Ed25519 in CMS (RFC 8419) needs OpenSSL 3.2 or later, and ubuntu-latest ships
# 3.0.13, which refuses even its own Ed25519 CMS signatures. smime_openssl_live.rs
# judges the Ed25519 signatures with the command in MW_SMIME_OPENSSL_ED25519 when the
# openssl on PATH is older; CI points that variable at this script.
#
# The temp directory is mounted at the same path, so the file arguments the test passes
# (under std::env::temp_dir()) resolve inside the container unchanged.
#
# Usage: scripts/smime/openssl-docker.sh <openssl args...>
#   e.g. scripts/smime/openssl-docker.sh version
# The first run builds the image `mw-openssl-docker:3.22` from the pinned base below.
set -euo pipefail

base='alpine:3.22@sha256:2e0b81d60597f449baa56dc25d355367869d2dca4cd57e866f1e2b52af3eb55f'
image='mw-openssl-docker:3.22'
tmp="${TMPDIR:-/tmp}"
tmp="${tmp%/}"

if ! docker image inspect "$image" >/dev/null 2>&1; then
  printf 'FROM %s\nRUN apk add --no-cache openssl\nENTRYPOINT ["openssl"]\n' "$base" |
    docker build -q -t "$image" - >/dev/null
fi

exec docker run --rm -i --network none -v "$tmp:$tmp" -w "$tmp" "$image" "$@"
