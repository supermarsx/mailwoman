#!/usr/bin/env bash
# HTTP/2 + HTTP/3 leg of the reverse-proxy conformance cells (t39-e12, L9).
#
# Mailwoman does not speak QUIC itself. HTTP/3 is terminated at the proxy, which
# talks HTTP/1.1 to the app and forwards the client's address, scheme and host
# in X-Forwarded-* exactly as it does for HTTP/2. This script checks that claim
# against a booted cell of docker-compose.proxy.yml:
#
#   1. Over TCP, the proxy negotiates HTTP/2 and advertises `Alt-Svc: h3=...`.
#   2. Over UDP, an `--http3-only` request is answered with HTTP/3.
#   3. For BOTH protocols, the app behind the proxy:
#        - sees the Host the client used: a login whose Origin names that host
#          passes the app's same-site check, and one naming another host is
#          refused (hardening::origin_ok compares Origin with Host);
#        - sees the scheme as https: the session cookie carries Secure and the
#          response carries Strict-Transport-Security, both of which the app
#          derives from a trusted X-Forwarded-Proto;
#        - attributes the request to the real client address: an API key
#          allowlisted to that address answers 200, one allowlisted to a decoy
#          answers 403 (the same read-out the Rust suite uses).
#
# The client is a curl build with HTTP/3 (quiche), run as a container on the
# cell's compose network with a fixed address outside MW_TRUSTED_PROXIES, so the
# expected client address is known and is not a proxy address.
#
# Optional `--published PORT` adds one request from the HOST network to the
# cell's published UDP port (127.0.0.1:PORT), which is what a real client does
# and what a CI runner may not allow. It checks only that h3 is negotiated.
#
# Usage:
#   docker compose -f docker-compose.proxy.yml --profile caddy up -d ...
#   bash scripts/proxy-h3-check.sh caddy [--published 8643]
#
# Environment (all optional):
#   MW_H3_CURL_IMAGE   curl image with HTTP/3 (default: pinned ymuski/curl-http3)
#   MW_H3_NETWORK      compose network   (default: mailwoman-proxy_proxynet)
#   MW_H3_CLIENT_IP    client address    (default: 172.28.30.5)
#   MW_H3_CA           CA/leaf PEM       (default: docs/deploy/proxy/tls/certs/server.crt)
#   MW_T20_USER / MW_T20_PASS            (default: testuser@example.org / testpass)
set -Eeuo pipefail

ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
IMAGE=${MW_H3_CURL_IMAGE:-ymuski/curl-http3@sha256:9eadfcaa6e541ef61880795c3044f7a603dd62fe5be5a3b45b5e154025c783de}
NETWORK=${MW_H3_NETWORK:-mailwoman-proxy_proxynet}
CLIENT_IP=${MW_H3_CLIENT_IP:-172.28.30.5}
CA=${MW_H3_CA:-$ROOT/docs/deploy/proxy/tls/certs/server.crt}
USER_NAME=${MW_T20_USER:-testuser@example.org}
USER_PASS=${MW_T20_PASS:-testpass}

usage() { printf 'usage: %s <caddy|nginx|traefik> [--published PORT]\n' "${0##*/}" >&2; exit 2; }
[[ $# -ge 1 ]] || usage
PROFILE=$1; shift
PUBLISHED=
while [[ $# -gt 0 ]]; do
    case $1 in
        --published) [[ $# -ge 2 ]] || usage; PUBLISHED=$2; shift 2 ;;
        *) usage ;;
    esac
done
case $PROFILE in caddy|nginx|traefik) ;; *) usage ;; esac
[[ -z $PUBLISHED || $PUBLISHED =~ ^[0-9]+$ ]] || usage
[[ -r $CA ]] || { printf 'FAIL: %s is not readable; run docs/deploy/proxy/tls/gen-certs.sh\n' "$CA" >&2; exit 1; }

# The certificate travels in an environment variable, not a bind mount, so the
# same command works from Git Bash on Windows without path conversion.
CA_PEM=$(cat -- "$CA")

# The in-container part. POSIX sh: the image has sh, grep, sed and python3.
# shellcheck disable=SC2016 # expanded by the container's shell, not this one
INNER='
set -eu
umask 077
printf "%s\n" "$MW_CA_PEM" > /tmp/ca.pem
BASE="https://$MW_HOST:8443"
CURL="curl -sS --cacert /tmp/ca.pem --max-time 20"
# This curl (quiche backend) does not apply --cacert to QUIC connections, so the
# HTTP/3 requests skip verification. The TCP requests verify the same leaf, and
# certificate checking is not what this script tests.
H3="--http3-only --insecure"
fail() { echo "FAIL [$MW_PROFILE $proto]: $*"; exit 1; }
json() { python3 -c "import json,sys; print(json.load(sys.stdin)$1)"; }
csrf() { awk "\$6 == \"mw_csrf\" { v = \$7 } END { print v }" "$1"; }

proto=tcp
echo "== $MW_PROFILE: HTTP/2 over TCP, $BASE/healthz"
$CURL --http2 -D /tmp/h2.hdr -o /dev/null -w "negotiated=%{http_version} status=%{http_code}\n" "$BASE/healthz" | tee /tmp/h2.out
cat /tmp/h2.hdr
grep -q "^negotiated=2 status=200" /tmp/h2.out || fail "HTTP/2 was not negotiated over TCP"
grep -iq "^alt-svc:.*h3=" /tmp/h2.hdr || fail "no Alt-Svc h3 advertisement on the TCP response"

proto=h3
echo "== $MW_PROFILE: HTTP/3 over UDP, $BASE/healthz"
$CURL $H3 -D /tmp/h3.hdr -o /dev/null -w "negotiated=%{http_version} status=%{http_code}\n" "$BASE/healthz" | tee /tmp/h3.out
cat /tmp/h3.hdr
grep -q "^negotiated=3 status=200" /tmp/h3.out || fail "HTTP/3 was not negotiated over UDP"

for proto in h2 h3; do
    if [ "$proto" = h2 ]; then P=--http2; want=2; else P=$H3; want=3; fi
    jar=/tmp/jar.$proto
    rm -f "$jar"
    body="{\"username\":\"$MW_USER\",\"password\":\"$MW_PASS\"}"
    echo "== $MW_PROFILE $proto: forwarded host, scheme and client address"

    # Host: the app compares Origin with the Host it received.
    code=$($CURL $P -o /dev/null -w "%{http_code}" -H "content-type: application/json" \
        -H "origin: https://elsewhere.example" --data "$body" "$BASE/api/login")
    [ "$code" = 403 ] || fail "a login with a foreign Origin answered $code, expected 403"
    $CURL $P -c "$jar" -D /tmp/login.$proto -o /tmp/login.body \
        -w "negotiated=%{http_version} status=%{http_code}\n" \
        -H "content-type: application/json" -H "origin: $BASE" \
        --data "$body" "$BASE/api/login" | tee /tmp/login.out
    grep -q "^negotiated=$want status=200" /tmp/login.out \
        || fail "login with Origin $BASE did not answer 200 over HTTP/$want (the app saw another Host)"
    echo "host: ok (Origin $BASE accepted, foreign Origin refused)"

    # Scheme: Secure session cookie + HSTS come from a trusted X-Forwarded-Proto.
    grep -iq "^set-cookie: mw_session=.*; *secure" /tmp/login.$proto \
        || fail "mw_session is not Secure: the app did not see X-Forwarded-Proto: https"
    grep -i "^strict-transport-security:" /tmp/login.$proto \
        || fail "no Strict-Transport-Security: the app did not see X-Forwarded-Proto: https"
    echo "scheme: ok (Secure cookie + HSTS)"

    # Client address: scoped-key allowlist read-out.
    account=$(json "[\"accountId\"]" < /tmp/login.body)
    for pair in "$MW_CLIENT_IP/32:200" "203.0.113.9/32:403"; do
        net=${pair%:*}; expect=${pair##*:}
        token=$(csrf "$jar")
        [ -n "$token" ] || fail "no mw_csrf cookie after login"
        code=$($CURL $P -b "$jar" -c "$jar" -o /dev/null -w "%{http_code}" \
            -H "content-type: application/json" -H "origin: $BASE" -H "x-csrf-token: $token" \
            --data "{\"password\":\"$MW_PASS\"}" "$BASE/api/security/step-up")
        case $code in 200|204) ;; *) fail "step-up answered $code" ;; esac
        token=$(csrf "$jar")
        key=$($CURL $P -b "$jar" -c "$jar" -H "content-type: application/json" \
            -H "origin: $BASE" -H "x-csrf-token: $token" \
            --data "{\"label\":\"t39-e12\",\"accountId\":\"$account\",\"scope\":{\"read\":true,\"send\":false,\"delete\":false,\"accounts\":{\"subset\":[\"$account\"]},\"folders\":\"all\",\"mail\":true,\"pim\":false,\"ip_allowlist\":[\"$net\"],\"expires_at\":null,\"rate_limit\":null,\"mcp_tools\":[],\"unattended_send\":false}}" \
            "$BASE/api/keys" | json "[\"displayToken\"]")
        code=$($CURL $P -o /dev/null -w "%{http_code}" -H "x-api-key: $key" "$BASE/api/v1/messages")
        [ "$code" = "$expect" ] || fail "key allowlisted to $net answered $code, expected $expect"
        echo "client address: key for $net -> $code (expected $expect)"
    done
done
echo "PASS: $MW_PROFILE negotiates h2 and h3; host, scheme and client $MW_CLIENT_IP reach the app over both"
'

host=$PROFILE
printf 'proxy-h3-check: %s via %s on %s as %s (%s)\n' "$PROFILE" "$host:8443" "$NETWORK" "$CLIENT_IP" "$IMAGE"
docker run --rm -i --network "$NETWORK" --ip "$CLIENT_IP" \
    -e MW_CA_PEM="$CA_PEM" -e MW_HOST="$host" -e MW_PROFILE="$PROFILE" \
    -e MW_CLIENT_IP="$CLIENT_IP" -e MW_USER="$USER_NAME" -e MW_PASS="$USER_PASS" \
    --entrypoint sh "$IMAGE" -c "$INNER"

if [[ -n $PUBLISHED ]]; then
    # From the host's network to the published UDP port. A runner or a desktop
    # Docker VM that does not forward UDP fails here while the leg above passes.
    printf '== %s: HTTP/3 to the published port 127.0.0.1:%s/udp\n' "$PROFILE" "$PUBLISHED"
    out=$(docker run --rm --network host --entrypoint curl "$IMAGE" -sS -k --max-time 20 \
        --http3-only -o /dev/null -w 'negotiated=%{http_version} status=%{http_code}' \
        "https://127.0.0.1:$PUBLISHED/healthz") || true
    printf '%s\n' "$out"
    [[ $out == 'negotiated=3 status=200' ]] || { printf 'FAIL [%s published]: HTTP/3 not negotiated on 127.0.0.1:%s/udp\n' "$PROFILE" "$PUBLISHED" >&2; exit 1; }
    printf 'PASS: %s published UDP port %s answers HTTP/3\n' "$PROFILE" "$PUBLISHED"
fi
