#!/bin/sh
# t13 (26.13) SCRAM-SHA-256-PLUS channel-binding live gate — cert generation.
#
# Generates the self-signed server certs the CB-capable Dovecot presents. These
# are TEST-ONLY, localhost, ephemeral — regenerated on every CI run and NEVER
# committed (the whole certs/ dir is gitignored). Two certs with DIFFERENT
# signature hashes so the live leg exercises RFC 5929 tls-server-end-point digest
# selection against a REAL server, not just the SHA-256 path:
#
#   t13-rsa-sha256.{crt,key}  RSA-2048, sha256WithRSAEncryption  -> 32-byte binding
#   t13-rsa-sha512.{crt,key}  RSA-2048, sha512WithRSAEncryption  -> 64-byte binding
#
# Both use an RSA key so the TLS handshake CertificateVerify uses an RSA scheme the
# rustls ring provider verifies; only the cert's SIGNATURE hash differs, which is
# exactly what RFC 5929 tls-server-end-point keys off — so the SHA-512 leaf proves
# the SHA-384/512 digest-selection path interoperates with a real server.
#
# Run BEFORE `docker compose -f docker-compose.ci.yml up dovecot-t13 dovecot-t13-sha512`
# (the compose services mount scripts/dovecot-t13/certs/ read-only). Requires a
# host openssl (the minimal dovecot image ships none).
set -eu

DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)/certs"
mkdir -p "$DIR"

# RSA-2048 leaf, self-signed with SHA-256 (the RFC-common channel-binding case).
openssl req -x509 -newkey rsa:2048 -sha256 -nodes \
  -keyout "$DIR/t13-rsa-sha256.key" -out "$DIR/t13-rsa-sha256.crt" \
  -days 3650 -subj "/CN=localhost" \
  -addext "subjectAltName=DNS:localhost,IP:127.0.0.1" >/dev/null 2>&1

# RSA-2048 leaf, self-signed with SHA-512 (proves the SHA-512 digest-selection
# interoperates with a real server, per RFC 5929 sig-hash floor SHA-256).
openssl req -x509 -newkey rsa:2048 -sha512 -nodes \
  -keyout "$DIR/t13-rsa-sha512.key" -out "$DIR/t13-rsa-sha512.crt" \
  -days 3650 -subj "/CN=localhost" \
  -addext "subjectAltName=DNS:localhost,IP:127.0.0.1" >/dev/null 2>&1

echo "[t13-certs] wrote RSA/SHA-256 + RSA/SHA-512 leaves to $DIR"
