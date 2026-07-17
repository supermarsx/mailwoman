#!/bin/sh
# t13 (26.13) Dovecot container command wrapper. Mirrors dovecot-sasl's approach:
# start the server in the background, wait for the auth service by retrying the
# seed (doveadm's own startup cost paces the loop — the minimal image ships no
# `sleep`), create the shared mailbox the ACL leg targets, deliver the threaded
# + GeoIP seed corpus into testuser's INBOX, then keep the server foregrounded.
set -u

SEED_DIR="${SEED_DIR:-/seed}"
SEED_USER="${SEED_USER:-testuser}"
SEED_TRIES="${SEED_TRIES:-150}"
DOVEADM=/dovecot/bin/doveadm
DOVECOT=/dovecot/sbin/dovecot

"$DOVECOT" -F &
DPID=$!
trap 'kill -TERM "$DPID" 2>/dev/null' TERM INT

# Retry a doveadm command until the auth/userdb service is up (or budget spent).
retry() {
  n=0
  while [ "$n" -lt "$SEED_TRIES" ]; do
    if "$@" 2>/dev/null; then
      return 0
    fi
    n=$((n + 1))
  done
  return 1
}

# Create the mailbox the ACL round-trip targets (owner testuser holds all rights).
if retry "$DOVEADM" mailbox create -u "$SEED_USER" Shared; then
  echo "[t13-seed] created mailbox Shared for $SEED_USER"
else
  echo "[t13-seed] WARN could not create Shared mailbox (continuing)" >&2
fi

# Deliver the seed messages in filename order (= UID/ingest order), so the reply
# (01) lands before its origin (03) — exercising JWZ convergence on ingest.
count=0
for f in "$SEED_DIR"/*.eml; do
  [ -e "$f" ] || continue
  if retry "$DOVEADM" save -u "$SEED_USER" -m INBOX < "$f"; then
    echo "[t13-seed] delivered $f"
    count=$((count + 1))
  else
    echo "[t13-seed] WARN gave up on $f after $SEED_TRIES tries (continuing)" >&2
  fi
done
echo "[t13-seed] seeded $count message(s) into $SEED_USER INBOX"

wait "$DPID"
