#!/bin/sh
# Dovecot-Pigeonhole container command (plan §3 e11): ensure the test user's
# home exists (Pigeonhole stores uploaded Sieve scripts there), then run the
# server in the foreground for the container lifetime.
#
# Unlike the IMAP/POP3 fidelity leg (scripts/dovecot/seed-and-serve.sh) there is
# NO message seeding — the ManageSieve conformance smoke only needs auth + the
# managesieve service; PUTSCRIPT/GETSCRIPT create their own script storage.
#
# The official image is minimal (sh + nc + the dovecot binaries). This is the
# `continue-on-error` leg, so any prep failure only logs — Greenmail is the gate.
set -u

# passwd-file maps testuser -> home /srv/vmail/testuser, uid/gid 1000. Create it
# writable so Pigeonhole can persist scripts under ~/sieve.
mkdir -p /srv/vmail/testuser 2>/dev/null || true
chown -R 1000:1000 /srv/vmail 2>/dev/null || true

trap 'kill -TERM "$DPID" 2>/dev/null' TERM INT
/dovecot/sbin/dovecot -F &
DPID=$!
wait "$DPID"
