#!/bin/sh
# Generate the self-signed TLS material the reverse-proxy conformance cells use.
#
#   sh docs/deploy/proxy/tls/gen-certs.sh
#
# Writes into docs/deploy/proxy/tls/certs/ (gitignored — never committed):
#
#   server.key   PEM private key            (nginx, apache, caddy, traefik,
#                                             envoy, and the app's own TLS
#                                             listener via MW_TLS_KEY)
#   server.crt   PEM certificate            (same consumers, via MW_TLS_CERT)
#   server.pem   crt + key concatenated     (HAProxy `bind ... ssl crt`)
#
# This exists only so the HTTPS half of each cell can boot. It is TEST material:
# a 1-day-old self-signed leaf with no CA, and every client that talks to it has
# to skip verification or pin server.crt explicitly. Nothing here is a pattern to
# copy into a deployment — real deployments use the built-in ACME client
# (MW_ACME) or an operator-supplied cert (MW_TLS_CERT/MW_TLS_KEY).
#
# Mirrors the existing scripts/dovecot-t13/gen-certs.sh convention: host openssl,
# run before `docker compose up`, output mounted read-only.
#
# Idempotent: re-running regenerates. Pass --force to regenerate when the current
# cert is still valid (the default is to leave a usable cert alone, so repeat CI
# steps do not churn the file every time).

set -eu

DIR="$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)"
OUT="$DIR/certs"
DAYS=825
FORCE=0

[ "${1:-}" = "--force" ] && FORCE=1

if ! command -v openssl >/dev/null 2>&1; then
    echo "gen-certs: openssl not found on PATH" >&2
    exit 1
fi

mkdir -p "$OUT"

if [ "$FORCE" -eq 0 ] && [ -s "$OUT/server.crt" ] && [ -s "$OUT/server.key" ] &&
    openssl x509 -in "$OUT/server.crt" -noout -checkend 86400 >/dev/null 2>&1; then
    echo "gen-certs: $OUT/server.crt is present and valid for >24h; nothing to do (--force to regenerate)"
    exit 0
fi

# SANs cover every name a cell is reached by: the published host ports resolve as
# localhost/127.0.0.1/::1, and inside the compose network the proxies dial the
# app by service name.
cat >"$OUT/openssl.cnf" <<'EOF'
[req]
distinguished_name = dn
x509_extensions    = v3
prompt             = no

[dn]
CN = localhost
O  = Mailwoman proxy conformance (TEST ONLY)

[v3]
basicConstraints       = critical, CA:FALSE
keyUsage               = critical, digitalSignature, keyEncipherment
extendedKeyUsage       = serverAuth
subjectAltName         = @san

[san]
DNS.1 = localhost
DNS.2 = app
DNS.3 = app-tls
DNS.4 = nginx
DNS.5 = apache
DNS.6 = caddy
DNS.7 = haproxy-l7
DNS.8 = haproxy-l4
DNS.9 = traefik
DNS.10 = envoy
DNS.11 = mail.example.org
IP.1  = 127.0.0.1
IP.2  = ::1
EOF

openssl req -x509 -newkey rsa:2048 -nodes \
    -keyout "$OUT/server.key" \
    -out "$OUT/server.crt" \
    -days "$DAYS" \
    -sha256 \
    -config "$OUT/openssl.cnf" >/dev/null 2>&1

# HAProxy wants one file holding both. Order matters: cert first, then key.
cat "$OUT/server.crt" "$OUT/server.key" >"$OUT/server.pem"

# The app runs as uid 65532 (distroless nonroot) and every proxy image runs as a
# non-root user too, so the mounted key must be world-readable. This is test
# material with a throwaway key; do not carry this mode into a deployment.
chmod 0644 "$OUT/server.key" "$OUT/server.crt" "$OUT/server.pem" 2>/dev/null || true

rm -f "$OUT/openssl.cnf"

echo "gen-certs: wrote $OUT/{server.key,server.crt,server.pem}"
openssl x509 -in "$OUT/server.crt" -noout -subject -dates
