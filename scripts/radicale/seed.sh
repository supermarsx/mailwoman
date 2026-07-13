#!/usr/bin/env sh
# Seed (and optionally reset) a live Radicale with the collections the V3
# CalDAV/CardDAV live tests discover. It waits for the server, then creates —
# idempotently, over the DAV protocol itself (no reliance on Radicale's on-disk
# collection format, which is version-specific) — for principal `testuser`:
#
#   /testuser/calendar/   a VEVENT calendar   (mw-dav discovery + mw-engine
#                                               event round-trip land here)
#   /testuser/tasks/      a VTODO task list   (mw-engine VTODO round-trip)
#   /testuser/contacts/   a CardDAV addressbook (mw-carddav discovery)
#
# Discovery (RFC 6764 .well-known → current-user-principal → home-set → PROPFIND)
# needs these collections to already exist, so the create/sync/conflict tests
# (which are the transcripts) have somewhere to write. The tests create/delete
# their own resources; this script only guarantees the collections.
#
# Usage: scripts/radicale/seed.sh [host] [port] [timeout_s]
# Env:
#   RADICALE_USER / RADICALE_PASS   login   (default testuser / testpass)
#   RADICALE_RESET=1                DELETE the collections first (clean slate)
set -eu

HOST="${1:-127.0.0.1}"
PORT="${2:-5232}"
TIMEOUT="${3:-60}"
USER="${RADICALE_USER:-testuser}"
PASS="${RADICALE_PASS:-testpass}"
BASE="http://${HOST}:${PORT}"

DEADLINE=$(( $(date +%s) + TIMEOUT ))

printf '[radicale] waiting for %s (timeout %ss)\n' "$BASE" "$TIMEOUT"
while :; do
  # Radicale serves its web UI at / unauthenticated once it is up.
  if curl -fsS -o /dev/null "${BASE}/" 2>/dev/null; then
    break
  fi
  if [ "$(date +%s)" -ge "$DEADLINE" ]; then
    printf '[radicale] TIMEOUT: server not up after %ss\n' "$TIMEOUT" >&2
    exit 1
  fi
  sleep 2
done
printf '[radicale] up; seeding collections for principal %s\n' "$USER"

# Issue a DAV request; echo the HTTP status. Args: METHOD PATH [BODY]
dav() {
  _method="$1"; _path="$2"; _body="${3:-}"
  if [ -n "$_body" ]; then
    curl -sS -o /dev/null -w '%{http_code}' \
      -u "${USER}:${PASS}" -X "$_method" \
      -H 'Content-Type: application/xml; charset=utf-8' \
      --data "$_body" "${BASE}${_path}"
  else
    curl -sS -o /dev/null -w '%{http_code}' \
      -u "${USER}:${PASS}" -X "$_method" "${BASE}${_path}"
  fi
}

# Create a collection idempotently. 201 = created; 405/409 = already exists
# (Radicale rejects re-creating an existing collection) — both are success here.
ensure() {
  _label="$1"; _method="$2"; _path="$3"; _body="$4"
  if [ "${RADICALE_RESET:-0}" = "1" ]; then
    dav DELETE "$_path" >/dev/null 2>&1 || true
  fi
  _code="$(dav "$_method" "$_path" "$_body" || true)"
  case "$_code" in
    201) printf '[radicale] %-9s created (%s)\n' "$_label" "$_path" ;;
    405|409) printf '[radicale] %-9s exists  (%s)\n' "$_label" "$_path" ;;
    *) printf '[radicale] %-9s FAILED (%s -> HTTP %s)\n' "$_label" "$_path" "$_code" >&2
       exit 1 ;;
  esac
}

MKCAL_VEVENT='<?xml version="1.0" encoding="utf-8"?>
<C:mkcalendar xmlns:D="DAV:" xmlns:C="urn:ietf:params:xml:ns:caldav">
  <D:set><D:prop>
    <D:displayname>Calendar</D:displayname>
    <C:supported-calendar-component-set><C:comp name="VEVENT"/></C:supported-calendar-component-set>
  </D:prop></D:set>
</C:mkcalendar>'

MKCAL_VTODO='<?xml version="1.0" encoding="utf-8"?>
<C:mkcalendar xmlns:D="DAV:" xmlns:C="urn:ietf:params:xml:ns:caldav">
  <D:set><D:prop>
    <D:displayname>Tasks</D:displayname>
    <C:supported-calendar-component-set><C:comp name="VTODO"/></C:supported-calendar-component-set>
  </D:prop></D:set>
</C:mkcalendar>'

MKCOL_ADDRESSBOOK='<?xml version="1.0" encoding="utf-8"?>
<D:mkcol xmlns:D="DAV:" xmlns:CR="urn:ietf:params:xml:ns:carddav">
  <D:set><D:prop>
    <D:resourcetype><D:collection/><CR:addressbook/></D:resourcetype>
    <D:displayname>Contacts</D:displayname>
  </D:prop></D:set>
</D:mkcol>'

ensure "calendar"  MKCALENDAR /testuser/calendar/ "$MKCAL_VEVENT"
ensure "tasks"     MKCALENDAR /testuser/tasks/    "$MKCAL_VTODO"
ensure "contacts"  MKCOL      /testuser/contacts/ "$MKCOL_ADDRESSBOOK"

printf '[radicale] seed OK — VEVENT + VTODO calendars + addressbook for %s\n' "$USER"
