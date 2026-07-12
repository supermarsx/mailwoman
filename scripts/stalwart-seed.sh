#!/usr/bin/env sh
# Seed a Stalwart instance with the E2E test domain + account.
#
# EXPERIMENTAL / best-effort. Stalwart's admin API surface changed across
# versions (the REST management API is being folded into JMAP as of v0.16),
# so this script tries the documented REST shape and treats every step as
# non-fatal: a failure here must never break the stack, because the in-repo
# mw-mock-jmap is the authoritative deterministic E2E backend. Verify against
# your pinned Stalwart tag before relying on it for a real-backend run.
#
# Env (see docker-compose.dev.yml):
#   STALWART_URL    base URL, e.g. http://stalwart:8080
#   STALWART_ADMIN  admin creds "user:pass" for Basic auth
#   SEED_DOMAIN     e.g. example.org
#   SEED_USER       e.g. testuser@example.org
#   SEED_PASS       e.g. testpass
set -u

URL="${STALWART_URL:-http://stalwart:8080}"
ADMIN="${STALWART_ADMIN:-admin:adminpass}"
DOMAIN="${SEED_DOMAIN:-example.org}"
USER="${SEED_USER:-testuser@example.org}"
PASS="${SEED_PASS:-testpass}"

log() { printf '[stalwart-seed] %s\n' "$*"; }

# Wait for the admin API to answer at all (bounded).
i=0
while [ "$i" -lt 60 ]; do
  if curl -fsS -o /dev/null "$URL/healthz" 2>/dev/null; then break; fi
  i=$((i + 1))
  sleep 2
done

# Create the domain (idempotent-ish: ignore "already exists").
log "creating domain $DOMAIN"
curl -fsS -u "$ADMIN" -X POST "$URL/api/principal" \
  -H 'content-type: application/json' \
  -d "{\"type\":\"domain\",\"name\":\"$DOMAIN\"}" \
  && log "domain ok" || log "domain create failed (may already exist); continuing"

# Create the individual account with a plaintext secret (Stalwart hashes it).
log "creating account $USER"
curl -fsS -u "$ADMIN" -X POST "$URL/api/principal" \
  -H 'content-type: application/json' \
  -d "{\"type\":\"individual\",\"name\":\"$USER\",\"secrets\":[\"$PASS\"],\"emails\":[\"$USER\"]}" \
  && log "account ok" || log "account create failed (may already exist); continuing"

log "seed complete (best-effort). If this failed, use the mock backend."
exit 0
