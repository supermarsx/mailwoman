#!/usr/bin/env bash
# Restart and reverse-proxy cases for deploy.sh, run against real containers.
# Never touches the normal mailwoman deployment.
# Usage: bash scripts/test-deploy-proxy.sh IMAGE [MOCK_IMAGE]
#   IMAGE       a Mailwoman runtime image: docker build --target runtime -t mailwoman:proxy-test .
#   MOCK_IMAGE  the in-repo JMAP mock (--target mock); default: IMAGE with
#               mailwoman: replaced by mailwoman-mock:
#
# Each case prints PASS, FAIL or SKIP: <reason>.
#   1 restart  deploy.sh deploys the app on a localhost port. The server process is
#              killed with SIGKILL from the host PID namespace, not with `docker kill`,
#              which Docker records as a manual stop. Docker must restart it (die event
#              with exit code 137, RestartCount up by one), health must return, and an
#              admin ban written before the kill must still be listed.
#   2 npm      not written yet.
#   3 caddy    Caddy with a local ACME server (Pebble). Not written yet: it needs the
#              ACME_CA and ACME_CA_ROOT keys in deploy.sh.
#
# Not proven by this script: issuance by the public Let's Encrypt endpoint for a
# real domain; DNS and inbound reachability on ports 80 and 443; certificate
# renewal over time; Nginx Proxy Manager's own Let's Encrypt flow and any setting
# made by hand in its UI; behaviour on Docker engines other than the one used.
#
# Environment:
#   DEPLOY_TEST_REQUIRE=all  a SKIP makes the script exit non-zero
#   DEPLOY_PROXY_CASES       which cases to run, from "restart npm caddy" (default: all)
#   DEPLOY_PROXY_TEST_PORT   localhost port of case 1 (default 18090)
#   DEPLOY_TEST_PREFIX       prefix of every project, container and network it
#                            creates (default mw-proxy-test)
# Needs Bash 4+, curl, and Docker with Linux containers that allows a --privileged
# container (case 1). A failed case keeps its files under .deploy-test-*.local.
set -Eeuo pipefail
ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
SELF="$ROOT/scripts/test-deploy-proxy.sh"
# Sends the kill from the host PID namespace in case 1.
TOOL_IMAGE=busybox:1.37-glibc@sha256:42a9ff19af33811ee466b24c66384461d37c32dcc28d1d27d49b8aca816bd5ee
PREFIX=${DEPLOY_TEST_PREFIX:-mw-proxy-test}
[[ $PREFIX =~ ^[a-z0-9][a-z0-9-]{0,30}$ ]] || { printf 'Invalid DEPLOY_TEST_PREFIX\n' >&2; exit 2; }
# Exit codes of one case run.
SKIP_CODE=3

if [[ ${1:-} != --case ]]; then
    IMAGE=${1:?Pass a built Mailwoman runtime image}
    MOCK_IMAGE=${2:-${IMAGE/mailwoman:/mailwoman-mock:}}
    failed=0 skipped=0
    # Each case runs in its own process so that its cleanup trap and `set -e` apply
    # to it alone and a failure in one case does not stop the next.
    for name in ${DEPLOY_PROXY_CASES:-restart npm caddy}; do
        case $name in restart|npm|caddy) ;; *) printf 'Unknown case: %s\n' "$name" >&2; exit 2 ;; esac
        code=0
        bash "$SELF" --case "$name" "$IMAGE" "$MOCK_IMAGE" || code=$?
        if ((code == SKIP_CODE)); then skipped=$((skipped + 1)); elif ((code != 0)); then failed=$((failed + 1)); fi
    done
    if ((failed)); then printf 'FAIL: %d case(s) failed\n' "$failed" >&2; exit 1; fi
    if ((skipped)) && [[ ${DEPLOY_TEST_REQUIRE:-} == all ]]; then
        printf 'FAIL: %d case(s) skipped and DEPLOY_TEST_REQUIRE=all\n' "$skipped" >&2; exit 1
    fi
    exit 0
fi

CASE=$2 IMAGE=$3 MOCK_IMAGE=$4
case $CASE in
    restart) TITLE='case 1 (crash restart)' ;;
    npm) TITLE='case 2 (Nginx Proxy Manager)' ;;
    caddy) TITLE='case 3 (Caddy with a local ACME server)' ;;
esac
fail() { printf 'FAIL: %s: %s\n' "$TITLE" "$*" >&2; exit 1; }
skip() { printf 'SKIP: %s: %s\n' "$TITLE" "$*"; exit "$SKIP_CODE"; }

case $CASE in
    npm) skip 'not written yet' ;;
    caddy) skip 'not written yet: it needs ACME_CA and ACME_CA_ROOT in deploy.sh' ;;
esac

PROJECT="$PREFIX-$CASE-$$-$RANDOM"
STATE=$(mktemp -d "$ROOT/.deploy-test-XXXXXX.local")
PASSED=false
CONTAINERS=()
NETWORKS=()

run() { bash "$ROOT/deploy.sh" "$@" --state-dir "$STATE" >> "$STATE/deploy.log" 2>&1; }
dc() { docker compose --project-name "$PROJECT" --env-file /dev/null -f "$STATE/active/compose.yaml" "$@"; }
inspect() { docker inspect --format "$1" "$APP"; }
cleanup() {
    local result=$? name volume
    trap - EXIT ERR
    if [[ -f $STATE/active/compose.yaml ]]; then
        $PASSED || dc logs --no-color > "$STATE/app.log" 2>&1 || true
        dc down --timeout 10 >/dev/null 2>&1 || true
    fi
    for name in "${CONTAINERS[@]}"; do
        $PASSED || docker logs "$name" > "$STATE/$name.log" 2>&1 || true
        docker rm --force --volumes "$name" >/dev/null 2>&1 || true
    done
    # Exact random test project and explicit label checks protect unrelated data.
    while IFS= read -r volume; do
        [[ -n $volume && $volume == "${PROJECT}"[_-]* ]] || continue
        docker volume rm "$volume" >/dev/null || true
    done < <(docker volume ls -q --filter "label=mailwoman.deploy.project=$PROJECT")
    for name in "${NETWORKS[@]}"; do docker network rm "$name" >/dev/null 2>&1 || true; done
    if $PASSED && [[ $STATE == "$ROOT"/.deploy-test-*.local ]]; then
        rm -rf -- "$STATE"
    else
        printf 'Diagnostics of %s retained at %s\n' "$TITLE" "$STATE" >&2
    fi
    exit "$result"
}
trap cleanup EXIT
trap 'fail "command failed at line $LINENO: $BASH_COMMAND"' ERR

# wait_for SECONDS WHAT COMMAND...: run COMMAND once a second until it succeeds.
wait_for() {
    local seconds=$1 what=$2 deadline=$((SECONDS + $1))
    shift 2
    until "$@" >/dev/null 2>&1; do
        ((SECONDS < deadline)) || fail "no $what within ${seconds}s"
        sleep 1
    done
}

# req ARGS...: one HTTP request through the case's curl and cookie jar; sets CODE and BODY.
req() {
    local out
    out=$("${CURL[@]}" --silent --cookie "$JAR" --cookie-jar "$JAR" --write-out '\n%{http_code}' "$@") || true
    BODY=${out%$'\n'*} CODE=${out##*$'\n'}
}
req_json() { req --header 'content-type: application/json' "$@"; }

# Sign in to the admin panel at $BASE with the generated password; sets CSRF from mw_csrf.
admin_login() {
    req_json --data "{\"username\":\"$ADMIN_USER\",\"password\":\"$ADMIN_PASSWORD\"}" "$BASE/admin/login"
    [[ $CODE == 200 ]] || fail "admin login answered $CODE: $BODY"
    CSRF=$("${JAR_CAT[@]}" | awk '$6 == "mw_csrf" {print $7}')
    [[ -n $CSRF ]] || fail 'admin login set no mw_csrf cookie'
}

# Ban BAN_IP through the admin API under strict CSRF: refused without X-CSRF-Token,
# accepted with it, then listed.
admin_ban() {
    local ban="{\"ip\":\"$BAN_IP\",\"reason\":\"$TITLE\"}"
    req_json --data "$ban" "$BASE/admin/bans"
    [[ $CODE == 403 ]] || fail "an admin write without X-CSRF-Token answered $CODE, expected 403"
    req_json --header "x-csrf-token: $CSRF" --data "$ban" "$BASE/admin/bans"
    [[ $CODE == 2?? ]] || fail "an admin write with X-CSRF-Token answered $CODE: $BODY"
    ban_listed || fail "the ban on $BAN_IP is not listed after it was written: $BODY"
}
ban_listed() { req "$BASE/admin/bans"; [[ $CODE == 200 && $BODY == *"\"ip\":\"$BAN_IP\""* ]]; }

read_admin_credentials() {
    ADMIN_USER=$(awk -F= '$1 == "MW_ADMIN_USER" {print $2}' "$STATE/app.env")
    ADMIN_PASSWORD=$(awk -F= '$1 == "MW_ADMIN_PASSWORD" {print $2}' "$STATE/app.env")
    [[ -n $ADMIN_USER && -n $ADMIN_PASSWORD ]] || fail 'deploy.sh init wrote no admin credentials'
}

case_restart() {
    local port=${DEPLOY_PROXY_TEST_PORT:-18090} pid new_pid before after since started code
    CURL=(curl) JAR="$STATE/admin.cookies" JAR_CAT=(cat "$STATE/admin.cookies")
    BASE="http://localhost:$port" BAN_IP=203.0.113.251
    run init --project "$PROJECT" --port "$port" || fail "deploy.sh init failed: $(tail -n 5 "$STATE/deploy.log")"
    run deploy --image "$IMAGE" --timeout 120 || fail "deploy.sh deploy failed: $(tail -n 5 "$STATE/deploy.log")"
    APP=$(dc ps -q mailwoman)
    [[ -n $APP ]] || fail 'no app container after deploy'
    read_admin_credentials

    # Preconditions: the restart policy deploy.sh writes, a healthy app, a known
    # restart count, and a ban written through the app before the kill.
    [[ $(inspect '{{.HostConfig.RestartPolicy.Name}}') == unless-stopped ]] || fail 'the app has no unless-stopped restart policy'
    [[ $(inspect '{{.State.Health.Status}}') == healthy ]] || fail 'the app is not healthy after deploy'
    before=$(inspect '{{.RestartCount}}')
    admin_login
    admin_ban
    pid=$(inspect '{{.State.Pid}}')
    [[ $pid =~ ^[1-9][0-9]*$ ]] || fail "unexpected host PID: $pid"

    # The kill comes from outside Docker's API, as a crash or the kernel's OOM
    # killer would, so Docker does not mark the container as stopped by hand.
    since=$(date +%s)
    started=$SECONDS
    MSYS_NO_PATHCONV=1 docker run --rm --privileged --pid=host "$TOOL_IMAGE" kill -9 "$pid" \
        || fail "could not send SIGKILL to host PID $pid from the host PID namespace"
    wait_for 60 'restart by Docker' restarted "$before"
    after=$(inspect '{{.RestartCount}}')
    docker events --since "$since" --until "$(($(date +%s) + 1))" --filter "container=$APP" --filter event=die \
        --format '{{.Actor.Attributes.exitCode}}' > "$STATE/die-events"
    grep -qx 137 "$STATE/die-events" || fail "no die event with exit code 137 after the kill: $(tr '\n' ' ' < "$STATE/die-events")"
    new_pid=$(inspect '{{.State.Pid}}')
    [[ $new_pid != "$pid" ]] || fail 'the server process has the same PID after the restart'
    wait_for 120 'healthy app after the restart' healthy
    code=$(curl --silent --output /dev/null --write-out '%{http_code}' "$BASE/healthz") || true
    [[ $code == 200 ]] || fail "/healthz answered $code after the restart"

    # The ban came through the crash: a new session lists it.
    printf '' > "$JAR"
    admin_login
    ban_listed || fail "the ban written before the kill is gone after the restart: $BODY"
    req --request DELETE --header "x-csrf-token: $CSRF" "$BASE/admin/bans/$BAN_IP"
    [[ $CODE == 2?? ]] || fail "removing the test ban answered $CODE: $BODY"
    PASSED=true
    printf 'PASS: %s: SIGKILL from the host PID namespace, die event with exit code 137, RestartCount %s -> %s, healthy again %ss after the kill, the ban written before the kill still listed\n' \
        "$TITLE" "$before" "$after" "$((SECONDS - started))"
}
restarted() { (($(inspect '{{.RestartCount}}') > $1)) && [[ $(inspect '{{.State.Running}}') == true ]]; }
healthy() { [[ $(inspect '{{.State.Health.Status}}') == healthy ]]; }

"case_$CASE"
