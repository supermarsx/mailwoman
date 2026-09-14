#!/usr/bin/env sh
# Poll a URL until it returns a 2xx, or fail after a bounded timeout.
# Usage: scripts/wait-for-health.sh <url> [timeout_seconds]
# Example: scripts/wait-for-health.sh http://localhost:8080/healthz 120
set -eu

URL="${1:?usage: wait-for-health.sh <url> [timeout_seconds]}"
TIMEOUT="${2:-120}"
DEADLINE=$(( $(date +%s) + TIMEOUT ))

printf 'waiting for %s (timeout %ss)...\n' "$URL" "$TIMEOUT"
while :; do
  if curl -fsS -o /dev/null "$URL" 2>/dev/null; then
    printf 'healthy: %s\n' "$URL"
    exit 0
  fi
  if [ "$(date +%s)" -ge "$DEADLINE" ]; then
    printf 'TIMEOUT waiting for %s after %ss\n' "$URL" "$TIMEOUT" >&2
    exit 1
  fi
  sleep 2
done
