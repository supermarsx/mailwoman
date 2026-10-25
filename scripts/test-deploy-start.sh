#!/usr/bin/env bash
# Mock only Docker/Compose; execute deploy.sh's actual snapshot and restart code.
# The mock models Compose2.30: `start --wait` is unsupported, `up --wait` works.
set -Eeuo pipefail
ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
DEPLOY=${DEPLOY_TEST_SCRIPT:-$ROOT/deploy.sh}
FIXTURE=$(mktemp -d)
trap 'rm -rf -- "$FIXTURE"' EXIT
fail() { printf 'FAIL: %s\n' "$*" >&2; exit 1; }
STATE="$FIXTURE/state"; ACTIVE="$STATE/active"; TIMEOUT=180
RESUME_CONTAINER= LAST_BACKUP=
mkdir -p "$ACTIVE" "$FIXTURE/data"
printf 'retained data\n' > "$FIXTURE/data/message"
for name in compose.yaml app.env deploy.env release.env; do printf 'fixture\n' > "$ACTIVE/$name"; done
TRACE="$FIXTURE/trace"
log() { printf '[deploy] %s\n' "$*" >&2; }
die() { fail "$*"; }
docker() {
    printf 'docker %s\n' "$*" >> "$TRACE"
    case "$1" in
        inspect) printf 'true\n' ;;
        stop|start) ;;
        cp) tar -C "$FIXTURE/data" -cf - . ;;
        *) fail "unexpected Docker call: $*" ;;
    esac
}
compose() {
    shift
    printf 'compose %s\n' "$*" >> "$TRACE"
    case "$1" in
        ps) printf 'fixture-container\n' ;;
        start) fail 'Compose2.30: unknown flag: --wait' ;;
        restart) ;;
        up)
            [[ " $* " == *' -d '* && " $* " == *' --no-recreate '* && " $* " == *' --wait '* && " $* " == *' --wait-timeout 180 '* ]] || fail "unsafe resume: $*"
            ;;
        *) fail "unexpected Compose call: $*" ;;
    esac
}
load_active() { :; }
require_free_ports() { :; }
ensure_proxy_network() { :; }
verify() { printf 'verified\n' >> "$TRACE"; }
# These boundaries are actual function/case markers, not copies of their bodies.
source <(awk '/^snapshot\(\)/ { copying=1 } /^activate\(\)/ { copying=0 } copying { print }' "$DEPLOY")
declare -F snapshot >/dev/null || fail 'snapshot function not found'
snapshot
[[ -f $LAST_BACKUP/COMPLETE && -z $RESUME_CONTAINER ]] || fail 'snapshot incomplete or resume still pending'
(cd "$LAST_BACKUP" && sha256sum -c SHA256SUMS >/dev/null)
grep -q '^docker start fixture-container$' "$TRACE" || fail 'snapshot did not resume existing container'
grep -q '^compose up -d --no-recreate --wait --wait-timeout 180 mailwoman$' "$TRACE" || fail 'snapshot did not wait without recreation'
restart=$(sed -n 's/^    restart) \(.*\) ;;$/\1/p' "$DEPLOY")
[[ -n $restart ]] || fail 'restart branch not found'
eval "$restart"
grep -q '^compose restart$' "$TRACE" || fail 'restart not requested'
grep -q '^compose up -d --no-recreate --wait --wait-timeout 180$' "$TRACE" || fail 'restart did not use compatible health wait'
grep -q '^verified$' "$TRACE" || fail 'restart skipped verification'
printf 'PASS: Compose2.30 snapshot resume and restart preserve containers, wait for health, and verify\n'
