#!/bin/sh
# Dovecot container command wrapper (plan §3 e7): start the server, wait for its
# auth service, deliver the seed messages into the testuser INBOX with
# `doveadm save`, then keep the server in the foreground.
#
# The official image is minimal (only sh + nc + the dovecot binaries — no
# coreutils, no `sleep`), and `doveadm save -u` needs the running auth service's
# /run/dovecot/auth-userdb socket. Sharing /run across a sidecar breaks its
# permissions, so instead we run the server in the background here and seed from
# the SAME container (correct /run ownership). The wait loop is self-pacing:
# each retry pays doveadm's own startup cost, so no `sleep` primitive is needed.
#
# Dovecot is the `continue-on-error` fidelity target, so a seed failure only
# logs — it never aborts the server (Greenmail is the deterministic gate).
#
# Env:
#   SEED_DIR   directory of *.eml messages to deliver (default /seed)
#   SEED_USER  mailbox owner (default testuser)
#   SEED_TRIES max delivery attempts per message before giving up (default 150)
set -u

SEED_DIR="${SEED_DIR:-/seed}"
SEED_USER="${SEED_USER:-testuser}"
SEED_TRIES="${SEED_TRIES:-150}"
DOVEADM=/dovecot/bin/doveadm
DOVECOT=/dovecot/sbin/dovecot

# Start the server in the background and forward termination to it so
# `docker compose down` stops cleanly.
"$DOVECOT" -F &
DPID=$!
trap 'kill -TERM "$DPID" 2>/dev/null' TERM INT

# Deliver one message, retrying until the auth service is up (or we exhaust the
# budget). doveadm's ~tens-of-ms startup paces the loop without a sleep.
deliver() {
  msg="$1"
  n=0
  while [ "$n" -lt "$SEED_TRIES" ]; do
    if "$DOVEADM" save -u "$SEED_USER" -m INBOX < "$msg" 2>/dev/null; then
      echo "[dovecot-seed] delivered $msg"
      return 0
    fi
    n=$((n + 1))
  done
  echo "[dovecot-seed] WARN gave up on $msg after $SEED_TRIES tries (continuing)" >&2
  return 1
}

count=0
for f in "$SEED_DIR"/*.eml; do
  [ -e "$f" ] || continue
  if deliver "$f"; then count=$((count + 1)); fi
done
echo "[dovecot-seed] seeded $count message(s) into $SEED_USER INBOX"

# Keep the server as the foreground process for the container lifetime.
wait "$DPID"
