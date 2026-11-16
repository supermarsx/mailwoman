#!/usr/bin/env bash
# Exercise the actual migration helper and failure cleanup without live data.
set -Eeuo pipefail
ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
fixture=$(mktemp -d)
trap 'rm -rf -- "$fixture"' EXIT
cat > "$fixture/case.sh" <<'CASE'
set -Eeuo pipefail
# Variables shared with the deployment helpers/cleanup sourced below.
# shellcheck disable=SC2034
ROOT=$1 CASE=$2 STATE=$3 TRACE=$3/trace ACTIVE=$3/active WORK=$3/.work.fixture
EXPOSURE=$4
PROJECT=fixture DATA_VOLUME=original_sqlite IMAGE_ID=fixture-image POSTGRES_IMAGE=fixture-postgres TIMEOUT=30 ACTION=deploy PROGRESS_STARTED=0
RESUME_CONTAINER='' HELPER_CONTAINER='' EXTRACT='' LOG_ERROR='' LOCKED=false
LAST_BACKUP=$STATE/backup
mkdir -p "$ACTIVE" "$WORK" "$LAST_BACKUP"
printf 'original configuration\n' > "$ACTIVE/compose.yaml"
: > "$LAST_BACKUP/data.tar"
log() { printf '%s\n' "$*" >> "$TRACE"; }
colored_log() { :; }
docker() {
    printf 'docker %s\n' "$*" >> "$TRACE"
    case "$1" in
        create) printf 'migration-helper\n' ;;
        volume|cp|rm|start|run) ;;
        *) exit 92 ;;
    esac
}
compose() {
    local directory=$1; shift
    printf 'compose %s %s\n' "$directory" "$*" >> "$TRACE"
    case "$1" in
        ps) printf 'original-app\n' ;;
        up) [[ $directory == "$WORK" && $* == 'up -d --wait --wait-timeout 30 postgres valkey' ]] || exit 97 ;;
        run)
            [[ $directory == "$WORK" && $* == *'--complete --upgrade-source'* ]] || exit 93
            if [[ $CASE == failure ]]; then return 7; fi ;;
        *) exit 94 ;;
    esac
}
render() {
    [[ $DATA_VOLUME != original_sqlite && $1 == "$WORK" ]] || exit 95
    [[ $EXPOSURE == local || $EXPOSURE == external || $EXPOSURE == network ]] || exit 98
    printf 'render exposure=%s\n' "$EXPOSURE" >> "$TRACE"
    printf 'new volume config\n' > "$WORK/compose.yaml"
}
source "$ROOT/scripts/deploy-stack.sh"
# shellcheck source=/dev/null
source <(awk '/^cleanup\(\)/ { copy=1 } /^trap cleanup EXIT/ { copy=0 } copy { print }' "$ROOT/deploy.sh")
trap cleanup EXIT
stack_migrate_sqlite
[[ $RESUME_CONTAINER == original-app ]] || exit 96
# This is cleared by deploy.sh only after migration is verified, before cutover.
RESUME_CONTAINER=''
CASE
for exposure in local external network; do
for scenario in success failure; do
    directory=$fixture/$exposure-$scenario
    mkdir "$directory"
    set +e
    bash "$fixture/case.sh" "$ROOT" "$scenario" "$directory" "$exposure"
    status=$?
    set -e
    if [[ $scenario == success ]]; then
        [[ $status == 0 ]] || { printf 'FAIL: migration success flow\n' >&2; exit 1; }
        ! grep -q '^docker start ' "$directory/trace" || exit 1
    else
        [[ $status == 7 ]] || { printf 'FAIL: migration error did not propagate\n' >&2; exit 1; }
        grep -qx 'docker start original-app' "$directory/trace"
    fi
    grep -qx 'original configuration' "$directory/active/compose.yaml"
    grep -qx "render exposure=$exposure" "$directory/trace"
    if grep -q 'src=original_sqlite\|caddy' "$directory/trace"; then exit 1; fi
done
done
printf 'PASS: local and both BYO proxy migrations start only database/cache sidecars, preserve active configuration and source data, and resume the original app on failure\n'
