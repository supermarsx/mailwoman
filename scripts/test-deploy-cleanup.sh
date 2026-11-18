#!/usr/bin/env bash
# Exercise real deployment branches with mocked Docker, never production data.
set -Eeuo pipefail
ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
fixture=$(mktemp -d)
trap 'rm -rf -- "$fixture"' EXIT
cat > "$fixture/case.sh" <<'CASE'
set -Eeuo pipefail
ROOT=$1 STATE=$2 CASE=$3 TRACE=$2/trace ACTIVE=$2/active PROJECT=fixture
mkdir -p "$ACTIVE"
printf 'fixture\n' > "$ACTIVE/release.env"
RESUME_CONTAINER='' HELPER_CONTAINER='' EXTRACT='' WORK='' LOCKED=false LAST_BACKUP=''
LOG_ERROR='' TIMEOUT=180 STACK=single ACTION=maintenance PROGRESS_STARTED=0 SKIP_BACKUP=true AUTO_ROLLBACK=false previous_rollback=old
MAINTENANCE_APPLY=true MAINTENANCE_OPTIONS=(--orphan-bodies --compact)
log(){ printf '%s\n' "$*" >> "$TRACE"; }
colored_log(){ :; }
die(){ log "ERROR $*"; exit 1; }
load_active(){ :; }
snapshot(){ log "UNEXPECTED SNAPSHOT"; exit 98; }
record_snapshot(){ :; }
snapshot_progress(){ shift 2; "$@"; }
docker(){
 printf 'docker %s\n' "$*" >> "$TRACE"
 case "$1" in inspect) printf true;; stop|start|rm) :;; *) exit 97;; esac
}
compose(){
 shift;printf 'compose %s\n' "$*" >> "$TRACE"
 case "$1" in ps) printf fixture-app;; up) :;; run) [[ $CASE != failure ]] || return 7;; *) exit 96;; esac
}
source <(awk '/^cleanup\(\)/ { copy=1 } /^trap cleanup EXIT/ { copy=0 } copy { print }' "$ROOT/deploy.sh")
trap cleanup EXIT
if [[ $CASE == skip-deploy ]]; then
 printf 'old snapshot\n' > "$STATE/rollback.path"
 source <(awk '/^        phase "Protect installation data"/ {copy=1;next} /^        phase "Start services and verify health"/ {copy=0} copy {print}' "$ROOT/deploy.sh")
 [[ ! -f $STATE/rollback.path && -z $LAST_BACKUP && -z $previous_rollback ]]
else
 if [[ $CASE == preview ]]; then MAINTENANCE_APPLY=false;SKIP_BACKUP=false;fi
 source <(awk '/^    maintenance\)/ {copy=1;next} copy && /^        ;;$/ {exit} copy {print}' "$ROOT/deploy.sh")
fi
CASE
for name in skip-deploy preview apply failure; do
 mkdir "$fixture/$name"
 result=0
 bash "$fixture/case.sh" "$ROOT" "$fixture/$name" "$name" || result=$?
 trace=$fixture/$name/trace
 ! grep -q 'UNEXPECTED SNAPSHOT' "$trace"
 if [[ $name == failure ]]; then
  [[ $result == 7 ]]
  grep -q 'docker start fixture-app' "$trace"
 elif [[ $name == apply ]]; then
  [[ $result == 0 ]]
  grep -q 'docker stop -t 60 fixture-app' "$trace"
  grep -q 'cleanup --apply --offline --orphan-bodies --compact' "$trace"
  grep -q 'docker start fixture-app' "$trace"
 elif [[ $name == preview ]]; then
  [[ $result == 0 ]]
  ! grep -q 'docker stop\|--apply\|--offline' "$trace"
 else [[ $result == 0 ]]; fi
done
printf 'PASS: skip-backup, read-only maintenance preview, offline cleanup, failure restart\n'
