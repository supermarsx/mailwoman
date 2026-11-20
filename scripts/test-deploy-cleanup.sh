#!/usr/bin/env bash
# Real shell branches and cancellation traps; mocked Docker, no application data.
set -Eeuo pipefail
ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
fixture=$(mktemp -d)
trap 'rm -rf -- "$fixture"' EXIT
trap 'printf "FAIL: case=%s line=%s\n" "${name:-setup}" "$LINENO" >&2; if [[ -f ${trace:-/nonexistent} ]];then cat "$trace" >&2;fi' ERR
cat > "$fixture/case.sh" <<'CASE'
set -Eeuo pipefail
ROOT=$1 STATE=$2 CASE=$3 TRACE=$2/trace ACTIVE=$2/active PROJECT=fixture
mkdir -p "$ACTIVE"
printf 'fixture\n' > "$ACTIVE/release.env"
RESUME_CONTAINER='' HELPER_CONTAINER='' EXTRACT='' WORK='' LOCKED=false LAST_BACKUP=''
MAINTENANCE_CONTAINER='' MAINTENANCE_ID='' MAINTENANCE_TOKEN='' MAINTENANCE_LOG_PID=''
LOG_ERROR='' TIMEOUT=2 STACK=single ACTION=maintenance PROGRESS_STARTED=0 SKIP_BACKUP=true AUTO_ROLLBACK=false previous_rollback=old
MAINTENANCE_APPLY=true MAINTENANCE_OPTIONS=(--orphan-bodies --compact) MAINTENANCE_RPC_SECONDS=1
CASE_PID=$BASHPID
worker_id=$(printf 'a%.0s' {1..64})
log(){ printf '%s\n' "$*" >> "$TRACE"; }
colored_log(){ :; }
die(){ log "ERROR $*"; exit 1; }
load_active(){ :; }
snapshot(){ log 'UNEXPECTED SNAPSHOT'; exit 98; }
record_snapshot(){ :; }
snapshot_progress(){ shift 2; "$@"; }
docker(){
 printf 'docker %s\n' "$*" >> "$TRACE"
 case "$1" in
 inspect)
  if [[ ${*: -1} == fixture-app ]];then printf true;return;fi
  [[ $CASE != launch-unknown && $CASE != unknown-cancel ]] || return 1
  local status=exited code=0 running=false token=$MAINTENANCE_TOKEN
  [[ ! -f $STATE/running ]] || { status=running;running=true; }
  [[ $CASE != failure ]] || code=7
  [[ $CASE != foreign ]] || token=foreign-owner
  [[ $CASE != unknown-exit ]] || code=invalid
  local pid=0
  [[ $running == false ]] || pid=12
  [[ $CASE != stale-pid ]] || pid=12
  printf '%s|%s|%s|%s|no|%s|%s\n' "$worker_id" "$token" "$status" "$code" "$running" "$pid" ;;
 compose)
  [[ $* == *' run -d '* && $* == *'mailwoman.deploy.maintenance='* ]] || return 96
  [[ $CASE != launch-unknown ]] || return 9
  if [[ $CASE == cancel || $CASE == cancel-int || $CASE == stuck-cancel ]];then touch "$STATE/running";fi
  printf '%s\n' "$worker_id" ;;
 logs)
  if [[ $CASE == cancel || $CASE == stuck-cancel ]];then kill -TERM "$CASE_PID";fi
  if [[ $CASE == cancel-int ]];then kill -INT "$CASE_PID";fi ;;
 kill)
  [[ $* == "kill --signal TERM $worker_id" ]] || return 95
  if [[ $CASE != stuck-cancel ]];then rm -f -- "$STATE/running";fi ;;
 rm) [[ $CASE != rm-unknown ]];;
 stop|start) : ;;
 *) return 97 ;;
 esac
}
compose(){
 shift;printf 'compose %s\n' "$*" >> "$TRACE"
 case "$1" in ps) printf fixture-app;; up|run) :;; *) return 96;; esac
}
source <(awk '/^maintenance_rpc\(\)/ {copy=1} /^trap cleanup EXIT/ {copy=0} copy{print}' "$ROOT/deploy.sh")
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
if [[ $CASE == skip-deploy ]];then
 printf 'old snapshot\n' > "$STATE/rollback.path"
 source <(awk '/^        phase "Protect installation data"/ {copy=1;next} /^        phase "Start services and verify health"/ {copy=0} copy{print}' "$ROOT/deploy.sh")
 [[ ! -f $STATE/rollback.path && -z $LAST_BACKUP && -z $previous_rollback ]]
elif [[ $CASE == rpc-timeout ]];then
 result=0
 maintenance_rpc sleep 20 >/dev/null 2>&1 || result=$?
 [[ $result != 0 && $SECONDS -lt 8 ]]
 log 'bounded RPC returned'
else
 if [[ $CASE == preview ]];then MAINTENANCE_APPLY=false;SKIP_BACKUP=false;fi
 if [[ $CASE == unknown-cancel ]];then
  docker(){
   printf 'docker %s\n' "$*" >> "$TRACE"
   case "$1" in
    inspect)
     if [[ ${*: -1} == fixture-app ]];then printf true;return;fi
     [[ ! -e $STATE/unknown ]] || return 1
     printf '%s|%s|running|0|no|true|12\n' "$worker_id" "$MAINTENANCE_TOKEN" ;;
    compose) printf '%s\n' "$worker_id";;
    logs) touch "$STATE/unknown";kill -TERM "$CASE_PID";;
    stop|start|rm) :;;
    *) return 94;;
   esac
  }
 fi
 source <(awk '/^    maintenance\)/ {copy=1;next} copy && /^        ;;$/ {exit} copy{print}' "$ROOT/deploy.sh")
fi
CASE
for name in skip-deploy preview apply failure cancel cancel-int stuck-cancel unknown-cancel foreign unknown-exit launch-unknown stale-pid rm-unknown rpc-timeout;do
 mkdir "$fixture/$name"
 result=0
 bash "$fixture/case.sh" "$ROOT" "$fixture/$name" "$name" || result=$?
 trace=$fixture/$name/trace
 ! grep -q 'UNEXPECTED SNAPSHOT' "$trace"
 ! grep -q 'docker rm -f\|docker stop .*maintenance\|--signal KILL' "$trace"
 case "$name" in
  failure) [[ $result == 7 ]];grep -q 'docker start fixture-app' "$trace";;
  apply)
   [[ $result == 0 ]];grep -q 'docker stop -t 60 fixture-app' "$trace"
   grep -q 'run -d .*--label mailwoman.deploy.maintenance=' "$trace"
   grep -q 'cleanup --apply --offline --orphan-bodies --compact' "$trace"
   grep -q 'docker start fixture-app' "$trace";;
  cancel|cancel-int)
   if [[ $name == cancel-int ]];then [[ $result == 130 ]];else [[ $result == 143 ]];fi
   grep -q 'docker kill --signal TERM' "$trace"
   grep -q 'docker start fixture-app' "$trace"
   [[ $(grep -n 'docker kill --signal TERM' "$trace"|head -1|cut -d: -f1) -lt $(grep -n 'docker start fixture-app' "$trace"|head -1|cut -d: -f1) ]];;
  stuck-cancel|unknown-cancel|foreign|unknown-exit|launch-unknown|stale-pid|rm-unknown)
   [[ $result != 0 ]];! grep -q 'docker start fixture-app' "$trace"
   grep -q 'app remains stopped' "$trace"
   grep -q 'No forced worker kill was attempted' "$trace";;
  preview) [[ $result == 0 ]];! grep -q 'docker stop\|--apply\|--offline' "$trace";;
  rpc-timeout) [[ $result == 0 ]];grep -q 'bounded RPC returned' "$trace";;
  *) [[ $result == 0 ]];;
 esac
 if [[ $name == foreign ]];then ! grep -q 'docker kill --signal' "$trace";fi
done
printf 'PASS: preview/skip-backup, detached worker, cooperative cancellation, bounded RPC and unknown/live/foreign writer resume fences\n'
