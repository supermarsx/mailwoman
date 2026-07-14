#!/usr/bin/env bash
# Host-side readiness poll for the t9-e6 live SSO Keycloak (docker-compose.ci.yml).
#
# Keycloak 26's base image ships no shell/curl, so a container-level HTTP
# healthcheck is impractical; this script asserts readiness from the host by
# polling the seeded realm's OIDC discovery document — which only returns 200
# once the realm import has completed AND the server is serving. That is the
# exact precondition the mw-server live harness + the Playwright SSO specs need.
#
# Usage: scripts/keycloak/wait-for-keycloak.sh [BASE_URL] [REALM] [TIMEOUT_SECS]
set -euo pipefail

BASE_URL="${1:-http://localhost:8080}"
REALM="${2:-mailwoman}"
TIMEOUT="${3:-180}"
DISCOVERY="${BASE_URL}/realms/${REALM}/.well-known/openid-configuration"

echo "waiting for Keycloak realm '${REALM}' at ${DISCOVERY} (timeout ${TIMEOUT}s)…"
deadline=$(( $(date +%s) + TIMEOUT ))
while true; do
  code=$(curl -s -o /dev/null -w '%{http_code}' "${DISCOVERY}" || echo 000)
  if [ "${code}" = "200" ]; then
    echo "Keycloak realm '${REALM}' is ready (discovery 200)."
    exit 0
  fi
  if [ "$(date +%s)" -ge "${deadline}" ]; then
    echo "ERROR: Keycloak realm '${REALM}' not ready within ${TIMEOUT}s (last code ${code})." >&2
    exit 1
  fi
  sleep 3
done
