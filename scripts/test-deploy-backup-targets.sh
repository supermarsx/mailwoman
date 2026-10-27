#!/usr/bin/env bash
# Exercise the real backup branch with deterministic snapshot/transport doubles.
set -Eeuo pipefail
ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
fixture=$(mktemp -d)
trap 'rm -rf -- "$fixture"' EXIT
trace="$fixture/trace"
branch=$(awk '/^    backup\)/ {copying=1; next} copying && /^        ;;$/ {exit} copying {print}' "$ROOT/deploy.sh")
[[ -n $branch ]]
# The values are read by the deploy.sh backup branch that is eval'd below.
# shellcheck disable=SC2034
BACKUP_TARGETS_FILE=/etc/mailwoman/targets.json BACKUP_RECIPIENT='fixture-public-key' BACKUP_HOOK=''
# shellcheck disable=SC2034
BACKUP_KEEP=7 BACKUP_KEEP_DAYS=''
LAST_BACKUP="$fixture/snapshot.tar.age"
load_active() { :; }
snapshot() { printf 'snapshot\n' >> "$trace"; printf 'retained\n' > "$LAST_BACKUP"; }
encrypt_snapshot() { printf 'encrypt\n' >> "$trace"; }
record_snapshot() { printf 'record\n' >> "$trace"; }
prune_snapshots() { printf 'prune\n' >> "$trace"; }
die() { printf '%s\n' "$*" >&2; exit 1; }
python3() {
    printf '%s\n' "$2" >> "$trace"
    [[ $2 != "${FAIL_AT:-}" ]]
}
eval "$branch"
expected=$'check\nsnapshot\nencrypt\nrecord\npublish\nprune'
[[ $(cat "$trace") == "$expected" ]]

# Failed copy preserves the completed local snapshot and never runs retention.
: > "$trace"
if (FAIL_AT=publish; eval "$branch"); then
    printf 'FAIL: failed destination was reported as success\n' >&2; exit 1
fi
[[ -f $LAST_BACKUP && $(cat "$trace") == $'check\nsnapshot\nencrypt\nrecord\npublish' ]]

# Invalid destination configuration is rejected before stopping/snapshotting.
: > "$trace"
set +e
(set -e; FAIL_AT=check; eval "$branch")
result=$?
set -e
[[ $result -ne 0 && $(cat "$trace") == check ]]
printf 'PASS: backup destination preflight, encrypted publication, and failure retention ordering\n'
