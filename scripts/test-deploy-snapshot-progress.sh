#!/usr/bin/env bash
# Real progress helper with deterministic commands; no running Docker required.
set -Eeuo pipefail
ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
fixture=$(mktemp -d)
trap 'rm -rf -- "$fixture"' EXIT
log() { printf '[deploy] %s\n' "$*"; } # Even stdout loggers must not pollute data.
source "$ROOT/scripts/deploy-stack.sh"

snapshot_progress 'Copying fixture' "$fixture/data" bash -c 'printf beginning; for i in {1..100}; do grep -q "MiB written" "$1" && break; sleep .2; done; printf end' _ "$fixture/progress" > "$fixture/data" 2> "$fixture/progress"
[[ $(cat "$fixture/data") == beginningend ]]
grep -q 'Copying fixture: .* MiB written; .*s elapsed' "$fixture/progress"
grep -q 'Copying fixture: complete in ' "$fixture/progress"

snapshot_progress 'Checking fixture' '' bash -c 'for i in {1..100}; do grep -q "still working" "$1" && exit 0; sleep .2; done; exit 1' _ "$fixture/check" 2> "$fixture/check"
grep -q 'Checking fixture: .*s elapsed; still working' "$fixture/check"
grep -q 'Checking fixture: complete in ' "$fixture/check"

set +e
snapshot_progress 'Failing fixture' '' bash -c 'exit 7' 2> "$fixture/failure"
result=$?
set -e
[[ $result == 7 ]]
grep -q 'Failing fixture: failed after .*s (exit 7)' "$fixture/failure"
! grep -q 'complete in' "$fixture/failure"

# A stage's EXIT trap must reap its watcher, not leave delayed output behind.
before=$(wc -c < "$fixture/failure")
sleep 6
[[ $(wc -c < "$fixture/failure") == "$before" ]]

ACTIVE="$fixture/active"
mkdir -p "$ACTIVE" "$fixture/pg"
printf 'STACK=full\n' > "$ACTIVE/deploy.env"
tar -cf "$fixture/pg/data.tar" --files-from /dev/null
compose() { printf 'database fixture'; }
stack_dump "$fixture/pg" 2> "$fixture/pg-progress"
[[ $(tar -xOf "$fixture/pg/data.tar" .deploy-postgres.dump) == 'database fixture' ]]
grep -q 'Backing up PostgreSQL: complete in ' "$fixture/pg-progress"
grep -q 'Adding PostgreSQL backup to snapshot: complete in ' "$fixture/pg-progress"
printf 'PASS: snapshot byte progress, elapsed heartbeat, clean stdout, failure status and watcher cleanup\n'
