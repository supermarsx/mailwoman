#!/usr/bin/env sh
# Wait until Greenmail's IMAP/SMTP/POP3 listeners accept TCP connections, then
# assert that the seeded test user can actually LOG IN over IMAP. This is the
# deterministic gate's readiness probe: CI runs it before the live conformance
# tests so a slow container start fails loudly here rather than as a flaky test.
#
# Usage: scripts/greenmail/wait-for-greenmail.sh [host] [imap_port] [timeout_s]
# Env (defaults match docker-compose.dev.yml / the GREENMAIL_* live tests):
#   GM_HOST       host                       (default 127.0.0.1)
#   GM_IMAP_PORT  plaintext IMAP port        (default 3143)
#   GM_SMTP_PORT  plaintext SMTP port        (default 3025)
#   GM_POP3_PORT  plaintext POP3 port        (default 3110)
#   GREENMAIL_USER / GREENMAIL_PASS  login   (default testuser / testpass)
set -eu

HOST="${1:-${GM_HOST:-127.0.0.1}}"
IMAP_PORT="${2:-${GM_IMAP_PORT:-3143}}"
TIMEOUT="${3:-90}"
SMTP_PORT="${GM_SMTP_PORT:-3025}"
POP3_PORT="${GM_POP3_PORT:-3110}"
USER="${GREENMAIL_USER:-testuser}"
PASS="${GREENMAIL_PASS:-testpass}"

DEADLINE=$(( $(date +%s) + TIMEOUT ))

# Open a TCP connection to $1:$2 using bash's /dev/tcp (no nc/curl needed).
tcp_up() {
  bash -c "exec 3<>/dev/tcp/$1/$2" >/dev/null 2>&1
}

printf '[greenmail] waiting for %s IMAP:%s SMTP:%s POP3:%s (timeout %ss)\n' \
  "$HOST" "$IMAP_PORT" "$SMTP_PORT" "$POP3_PORT" "$TIMEOUT"

# 1) Wait for all three plaintext listeners to accept connections.
while :; do
  if tcp_up "$HOST" "$IMAP_PORT" && tcp_up "$HOST" "$SMTP_PORT" && tcp_up "$HOST" "$POP3_PORT"; then
    break
  fi
  if [ "$(date +%s)" -ge "$DEADLINE" ]; then
    printf '[greenmail] TIMEOUT: listeners not up after %ss\n' "$TIMEOUT" >&2
    exit 1
  fi
  sleep 2
done
printf '[greenmail] listeners are up; verifying IMAP login for %s\n' "$USER"

# 2) Assert the seeded user can LOGIN over IMAP — proves seeding, not just a
#    bound socket. A bare LOGIN + LOGOUT dialogue over bash /dev/tcp.
while :; do
  RESP=$(bash -c '
    exec 3<>/dev/tcp/'"$HOST"'/'"$IMAP_PORT"' || exit 1
    IFS= read -r _greeting <&3
    printf "a1 LOGIN %s %s\r\n" "'"$USER"'" "'"$PASS"'" >&3
    IFS= read -r line <&3
    printf "a2 LOGOUT\r\n" >&3
    printf "%s" "$line"
  ' 2>/dev/null || true)
  case "$RESP" in
    a1\ OK*)
      printf '[greenmail] IMAP login OK: %s\n' "$RESP"
      exit 0
      ;;
  esac
  if [ "$(date +%s)" -ge "$DEADLINE" ]; then
    printf '[greenmail] TIMEOUT: IMAP login for %s failed; last response: %s\n' "$USER" "${RESP:-<none>}" >&2
    exit 1
  fi
  sleep 2
done
