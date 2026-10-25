#!/bin/sh
# t32 Dovecot container command wrapper. Starts the server in the background,
# waits for the auth service by retrying doveadm (the minimal image ships no
# `sleep`; doveadm's own startup cost paces the loop), creates folders with
# non-ASCII names, delivers the seed messages, then keeps the server in the
# foreground.
#
# Folders (all subscribed, so a NOTIFY `subscribed` watch covers them):
#   Entwürfe        modified UTF-7 "Entw&APw-rfe"
#   Корзина         modified UTF-7 "&BBoEPgRABDcEOAQ9BDA-"
#   Archive/2026    a nested folder for subtree filters
set -u

SEED_DIR="${SEED_DIR:-/seed}"
SEED_USER="${SEED_USER:-testuser}"
SEED_TRIES="${SEED_TRIES:-150}"
DOVEADM=/dovecot/bin/doveadm
DOVECOT=/dovecot/sbin/dovecot

"$DOVECOT" -F &
DPID=$!
trap 'kill -TERM "$DPID" 2>/dev/null' TERM INT

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

for box in "Entwürfe" "Корзина" "Archive" "Archive/2026"; do
  if retry "$DOVEADM" mailbox create -s -u "$SEED_USER" "$box"; then
    echo "[t32-seed] created and subscribed $box"
  else
    echo "[t32-seed] WARN could not create $box (continuing)" >&2
  fi
done

count=0
for f in "$SEED_DIR"/*.eml; do
  [ -e "$f" ] || continue
  if retry "$DOVEADM" save -u "$SEED_USER" -m INBOX < "$f"; then
    count=$((count + 1))
  else
    echo "[t32-seed] WARN gave up on $f (continuing)" >&2
  fi
done
# One message in a non-ASCII folder, for STATUS and NOTIFY on it.
retry "$DOVEADM" save -u "$SEED_USER" -m "Entwürfe" < "$SEED_DIR/03-utf8.eml" || true
echo "[t32-seed] seeded $count message(s) into $SEED_USER INBOX"

wait "$DPID"
