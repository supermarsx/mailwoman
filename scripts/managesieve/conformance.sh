#!/usr/bin/env bash
# ManageSieve (RFC 5804) conformance smoke against a live Dovecot-Pigeonhole
# server (plan §3 e11, risk #9). It drives the EXACT command set the
# `mw-sieve` client implements (`crates/mw-sieve/src/managesieve.rs`) over the
# wire — greeting/CAPABILITY, AUTHENTICATE PLAIN, PUTSCRIPT (non-sync literal),
# LISTSCRIPTS, SETACTIVE, GETSCRIPT, DELETESCRIPT, LOGOUT — asserting the server
# speaks the protocol our client targets, and that Pigeonhole COMPILES the
# generated Sieve on PUTSCRIPT.
#
# This is the CI `continue-on-error` fidelity leg: Greenmail (which has no
# ManageSieve) stays the always-green gate; engine-side rule execution is the
# always-green path (e9). The `mw-sieve` transcript unit tests run in the `rust`
# job; this exercises the LIVE path a container makes possible.
#
# Uses bash /dev/tcp (the image ships no nc/curl) with one long-lived fd 3, the
# same technique as scripts/greenmail/wait-for-greenmail.sh.
#
# Usage: scripts/managesieve/conformance.sh [host] [port] [timeout_s]
# Env: SIEVE_USER / SIEVE_PASS  (default testuser / testpass)
set -euo pipefail

HOST="${1:-127.0.0.1}"
PORT="${2:-4190}"
TIMEOUT="${3:-60}"
USER="${SIEVE_USER:-testuser}"
PASS="${SIEVE_PASS:-testpass}"
SCRIPT_NAME="mailwoman-ci"

# A minimal, valid Sieve — Pigeonhole COMPILES it on PUTSCRIPT, so an OK here
# proves generated-Sieve fidelity, not just transport. Shape mirrors what
# mw-sieve's codegen emits for a "subject contains -> fileinto" rule.
SIEVE_BODY=$'require ["fileinto"];\r\nif header :contains "subject" "ci-test" {\r\n    fileinto "INBOX";\r\n}\r\n'

# SASL PLAIN initial-response: base64( authzid \0 authcid \0 passwd ), empty authzid.
IR="$(printf '\0%s\0%s' "$USER" "$PASS" | base64 | tr -d '\n')"

DEADLINE=$(( $(date +%s) + TIMEOUT ))
echo "[managesieve] connecting to $HOST:$PORT (user=$USER, timeout ${TIMEOUT}s)"

# 1) Wait for the ManageSieve listener to accept a TCP connection.
until bash -c "exec 3<>/dev/tcp/$HOST/$PORT" >/dev/null 2>&1; do
  if [ "$(date +%s)" -ge "$DEADLINE" ]; then
    echo "[managesieve] TIMEOUT: no listener on $HOST:$PORT after ${TIMEOUT}s" >&2
    exit 1
  fi
  sleep 2
done

# 2) Run the whole request/response dialogue with one persistent fd 3. Each
#    command is followed by reading data lines until an OK/NO/BYE completion.
exec 3<>"/dev/tcp/$HOST/$PORT"

# Read one response: echo data lines, return 0 on OK, 1 on NO/BYE. Literals in
# data lines are read as ordinary lines (our payloads are newline-delimited and
# the completion sits on its own line), which is sufficient for a smoke.
read_response() {
  local label="$1" line first
  while IFS= read -r line <&3; do
    line="${line%$'\r'}"
    first="$(printf '%s' "$line" | tr '[:lower:]' '[:upper:]' | cut -d' ' -f1)"
    case "$first" in
      OK)  echo "[managesieve] $label -> OK"; return 0 ;;
      NO|BYE) echo "[managesieve] $label -> $line" >&2; return 1 ;;
      *) : ;;  # data / capability / literal line
    esac
  done
  echo "[managesieve] $label -> connection closed before completion" >&2
  return 1
}

send() { printf '%s\r\n' "$1" >&3; }

# Greeting: a capability listing terminated by OK.
read_response "greeting"

send "AUTHENTICATE \"PLAIN\" \"$IR\""
read_response "AUTHENTICATE PLAIN"

# PUTSCRIPT with a non-synchronizing literal ({n+}): stream the body immediately.
BODY_LEN=$(printf '%s' "$SIEVE_BODY" | wc -c | tr -d ' ')
printf 'PUTSCRIPT "%s" {%s+}\r\n' "$SCRIPT_NAME" "$BODY_LEN" >&3
printf '%s' "$SIEVE_BODY" >&3
printf '\r\n' >&3
read_response "PUTSCRIPT (compiles generated Sieve)"

send "LISTSCRIPTS"
read_response "LISTSCRIPTS"

send "SETACTIVE \"$SCRIPT_NAME\""
read_response "SETACTIVE"

send "GETSCRIPT \"$SCRIPT_NAME\""
read_response "GETSCRIPT"

send "SETACTIVE \"\""
read_response "SETACTIVE (deactivate)"

send "DELETESCRIPT \"$SCRIPT_NAME\""
read_response "DELETESCRIPT"

send "LOGOUT"
read_response "LOGOUT" || true   # OK or BYE both terminate cleanly.

exec 3<&- 3>&-
echo "[managesieve] conformance OK — Pigeonhole speaks the mw-sieve client's RFC 5804 command set"
