#!/usr/bin/env bash
# Run the live maintenance case with mocked Compose. No real Docker or database.
set -Eeuo pipefail
ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
fixture=$(mktemp -d)
trap 'rm -rf -- "$fixture"' EXIT
cat > "$fixture/case.sh" <<'CASE'
set -Eeuo pipefail
ROOT=$1 ACTIVE=$2 TRACE=$2/trace STACK=single MAINTENANCE_ONLINE=true MAINTENANCE_ONLINE_ACTION=$3
mkdir -p "$ACTIVE"
load_active(){ :; }
log(){ printf 'log %s\n' "$*" >> "$TRACE"; }
die(){ printf 'ERROR %s\n' "$*" >> "$TRACE"; exit 1; }
snapshot(){ die 'snapshot must not run'; }
record_snapshot(){ die 'snapshot metadata must not change'; }
docker(){ die 'direct Docker operation must not run'; }
compose(){ shift;printf 'compose %s\n' "$*" >> "$TRACE";[[ $1 == exec && $2 == -T && $3 == mailwoman && $4 == /usr/local/bin/mailwoman && $5 == db && $6 == online ]]; }
source <(awk '/^    maintenance\)/ {copy=1;next} copy && /^        ;;$/ {exit} copy {print}' "$ROOT/deploy.sh")
CASE
for action in start status cancel; do
 mkdir "$fixture/$action"
 bash "$fixture/case.sh" "$ROOT" "$fixture/$action" "$action"
 trace=$fixture/$action/trace
 grep -q 'compose exec -T mailwoman /usr/local/bin/mailwoman db online' "$trace"
 ! grep -q 'ERROR' "$trace"
 if [[ $action == status ]]; then grep -q 'db online --status' "$trace";fi
 if [[ $action == cancel ]]; then grep -q 'db online --cancel' "$trace";fi
done
printf 'PASS: online start/status/cancel use the running container without stop, helper container or snapshot\n'
