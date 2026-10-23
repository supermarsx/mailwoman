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
#   2 npm      deploy.sh --proxy-network installs the app behind a real Nginx Proxy
#              Manager (pinned image) on that network, set up through its API: a proxy
#              host for the test domain forwarding to http://<project>:8080 with
#              WebSocket support, served with an uploaded throwaway certificate. A
#              client container on a second network then checks, through the proxy:
#              that the certificate verifies against the throwaway CA and not without
#              it, /healthz, the SPA, admin login and a write under strict CSRF, a
#              WebSocket upgrade on /jmap/ws with a mailbox session (the JMAP mock is
#              the upstream; 401 without one), and that the address the app logs for a
#              failed admin login is the client's, not the proxy's.
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
# Case 2's proxy. Its curl and openssl also serve as case 2's client.
NPM_IMAGE=jc21/nginx-proxy-manager:2.15.1@sha256:52b2c59994f3d36acfcf70a1626f29734df0ed8c71bacc0269f78b6f939858bb
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
    npm) docker image inspect "$MOCK_IMAGE" >/dev/null 2>&1 \
        || skip "the JMAP mock image $MOCK_IMAGE is not present; the WebSocket check needs a mailbox session (pass it as the second argument)" ;;
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

case_npm() {
    local domain=mail.npm-test.mailwoman.test network="$PROJECT-proxy" clients="$PROJECT-clients"
    local mock="$PROJECT-mock" npm_user=npm-test@example.test npm_password="npm-$RANDOM-$RANDOM-$RANDOM"
    local out cert_id npm_proxy_ip client_ip code probe rhost answer
    NPM="$PROJECT-npm" CLIENT="$PROJECT-client"

    # The proxy network exists before deploy.sh runs, as when the proxy's own stack
    # made it; the JMAP mock on it is the mailbox upstream for the WebSocket check.
    NETWORKS+=("$network" "$clients")
    docker network create --label "mailwoman.deploy.project=$PROJECT" "$network" >/dev/null
    docker network create --label "mailwoman.deploy.project=$PROJECT" "$clients" >/dev/null
    CONTAINERS+=("$mock" "$NPM" "$CLIENT")
    docker run --detach --name "$mock" --network "$network" --env MW_MOCK_PORT=8181 "$MOCK_IMAGE" >/dev/null
    wait_for 60 'healthy JMAP mock' cx_in "$mock" /usr/local/bin/mailwoman healthcheck --url http://127.0.0.1:8181/healthz

    run init --project "$PROJECT" --public-url "https://$domain" --proxy-network "$network" \
        || fail "deploy.sh init failed: $(tail -n 5 "$STATE/deploy.log")"
    printf '\nMW_JMAP_UPSTREAMS=http://%s:8181\n' "$mock" >> "$STATE/app.env"
    run deploy --image "$IMAGE" --timeout 120 || fail "deploy.sh deploy failed: $(tail -n 5 "$STATE/deploy.log")"
    APP=$(dc ps -q mailwoman)
    [[ -n $APP ]] || fail 'no app container after deploy'
    read_admin_credentials

    # Nginx Proxy Manager on the proxy network, and on a second network shared only
    # with the client, so the client's address is outside the app's trusted subnet.
    MSYS_NO_PATHCONV=1 docker run --detach --name "$NPM" --network "$network" --volume /data --volume /etc/letsencrypt \
        --env "INITIAL_ADMIN_EMAIL=$npm_user" --env "INITIAL_ADMIN_PASSWORD=$npm_password" "$NPM_IMAGE" >/dev/null
    docker network connect "$clients" "$NPM"
    MSYS_NO_PATHCONV=1 docker run --detach --name "$CLIENT" --network "$clients" --entrypoint sleep "$NPM_IMAGE" infinity >/dev/null
    NPM_IP=$(network_ip "$NPM" "$clients")
    npm_proxy_ip=$(network_ip "$NPM" "$network")
    client_ip=$(network_ip "$CLIENT" "$clients")
    [[ -n $NPM_IP && -n $npm_proxy_ip && -n $client_ip && $client_ip != "$npm_proxy_ip" ]] \
        || fail "unexpected addresses: proxy $NPM_IP/$npm_proxy_ip, client $client_ip"

    # A throwaway CA and a certificate for the test domain, made in the client.
    # shellcheck disable=SC2016 # $1 expands in the client shell.
    cx sh -c 'set -e; mkdir /w; cd /w
        openssl req -x509 -newkey rsa:2048 -nodes -days 2 -subj "/CN=Mailwoman proxy test CA" -keyout ca.key -out ca.pem
        openssl req -newkey rsa:2048 -nodes -subj "/CN=$1" -keyout leaf.key -out leaf.csr
        printf "subjectAltName=DNS:%s\nbasicConstraints=CA:FALSE\nextendedKeyUsage=serverAuth\n" "$1" > leaf.ext
        openssl x509 -req -in leaf.csr -CA ca.pem -CAkey ca.key -CAcreateserial -days 2 -extfile leaf.ext -out leaf.pem' \
        sh "$domain" > "$STATE/openssl.log" 2>&1 || fail "could not make the throwaway certificate: $(tail -n 3 "$STATE/openssl.log")"

    # Proxy Manager's API: sign in, upload the certificate, create the proxy host.
    wait_for 180 'Nginx Proxy Manager API' npm_ready
    TOKEN=$(cx curl --silent --show-error --fail-with-body --header 'content-type: application/json' \
        --data "{\"identity\":\"$npm_user\",\"secret\":\"$npm_password\"}" "http://$NPM:81/api/tokens" \
        | sed -n 's/.*"token":"\([^"]*\)".*/\1/p')
    [[ -n $TOKEN ]] || fail 'Nginx Proxy Manager issued no API token'
    out=$(npm_api --header 'content-type: application/json' \
        --data '{"provider":"other","nice_name":"Mailwoman proxy test"}' "http://$NPM:81/api/nginx/certificates")
    cert_id=$(grep -o '"id":[0-9]*' <<< "$out" | head -n 1 | cut -d: -f2)
    [[ -n $cert_id ]] || fail "no certificate id in: $out"
    npm_api --form certificate=@/w/leaf.pem --form certificate_key=@/w/leaf.key \
        "http://$NPM:81/api/nginx/certificates/$cert_id/upload" > "$STATE/npm-upload.json"
    out=$(npm_api --header 'content-type: application/json' --data "{\"domain_names\":[\"$domain\"],
        \"forward_scheme\":\"http\",\"forward_host\":\"$PROJECT\",\"forward_port\":8080,
        \"allow_websocket_upgrade\":true,\"certificate_id\":$cert_id,\"ssl_forced\":true,\"http2_support\":false,
        \"block_exploits\":false,\"caching_enabled\":false,\"access_list_id\":0,\"advanced_config\":\"\",
        \"locations\":[],\"meta\":{}}" "http://$NPM:81/api/nginx/proxy-hosts")
    [[ $out == *'"allow_websocket_upgrade":true'* && $out == *"\"certificate_id\":$cert_id"* ]] \
        || fail "unexpected proxy host: $out"

    CURL=(env MSYS_NO_PATHCONV=1 docker exec "$CLIENT" curl --resolve "$domain:443:$NPM_IP" --cacert /w/ca.pem)
    JAR=/w/cookies JAR_CAT=(cx cat /w/cookies) BASE="https://$domain" BAN_IP=203.0.113.252
    wait_for 60 '/healthz through the proxy' proxied_health

    # The proxy serves the uploaded certificate: it verifies against the throwaway
    # CA (above) and fails verification without it (curl exit 60).
    code=0
    cx curl --silent --output /dev/null --resolve "$domain:443:$NPM_IP" "$BASE/healthz" || code=$?
    ((code == 60)) || fail "without the throwaway CA, curl exited $code, expected 60 (certificate not verified)"

    req "$BASE/"
    if [[ $CODE != 200 ]] || ! grep -qi '<!doctype html' <<< "$BODY"; then fail "the SPA did not load through the proxy ($CODE)"; fi

    admin_login
    admin_ban
    req --request DELETE --header "x-csrf-token: $CSRF" "$BASE/admin/bans/$BAN_IP"
    [[ $CODE == 2?? ]] || fail "removing the test ban answered $CODE: $BODY"

    # The forwarded client address: the app trusts the proxy network's peers
    # (deploy.sh) and must log the client's address, which it never sees directly.
    probe="proxy-probe-$RANDOM$RANDOM"
    req_json --data "{\"username\":\"$probe\",\"password\":\"not-the-password\"}" "$BASE/admin/login"
    [[ $CODE == 401 ]] || fail "a wrong admin login answered $CODE, expected 401"
    wait_for 10 'log line for the failed admin login' logged_rhost "$probe"
    rhost=$(logged_rhost "$probe")
    [[ $rhost == "$client_ip" ]] \
        || fail "the app logged rhost=$rhost for a client at $client_ip behind the proxy at $npm_proxy_ip"

    # WebSocket upgrade on the push endpoint: refused without a session, upgraded
    # with one. The accept value for this key is the one RFC 6455 section 1.3 gives.
    answer=$(ws_head)
    [[ $answer == 'HTTP/1.1 401'* ]] || fail "a WebSocket request without a session answered: ${answer%%$'\n'*}"
    req_json --data "{\"jmapUrl\":\"http://$mock:8181/.well-known/jmap\",\"username\":\"testuser@example.org\",\"password\":\"testpass\"}" \
        "$BASE/api/login"
    [[ $CODE == 200 ]] || fail "mailbox login through the proxy answered $CODE: $BODY"
    answer=$(ws_head --cookie /w/cookies)
    [[ $answer == 'HTTP/1.1 101'* ]] || fail "the WebSocket upgrade through the proxy answered: ${answer%%$'\n'*}"
    grep -qix 'sec-websocket-accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=' <<< "$answer" || fail "wrong Sec-WebSocket-Accept: $answer"
    grep -qix 'sec-websocket-protocol: jmap' <<< "$answer" || fail "the jmap subprotocol was not echoed: $answer"
    PASSED=true
    printf 'PASS: %s: certificate uploaded through the API verifies against the throwaway CA only, /healthz, SPA, admin login and a write under strict CSRF, WebSocket 101 on /jmap/ws (401 without a session), app logged the client address %s rather than the proxy'"'"'s %s\n' \
        "$TITLE" "$client_ip" "$npm_proxy_ip"
}
cx() { MSYS_NO_PATHCONV=1 docker exec "$CLIENT" "$@"; }
cx_in() { MSYS_NO_PATHCONV=1 docker exec "$@"; }
network_ip() { docker inspect --format "{{(index .NetworkSettings.Networks \"$2\").IPAddress}}" "$1"; }
npm_ready() { [[ $(cx curl --silent --fail --max-time 3 "http://$NPM:81/api/") == *'"setup":true'* ]]; }
npm_api() { cx curl --silent --show-error --fail-with-body --header "authorization: Bearer $TOKEN" "$@"; }
proxied_health() { req "$BASE/healthz"; [[ $CODE == 200 ]]; }
# Print the address of the last fail2ban line the app logged for this user name.
logged_rhost() {
    docker logs "$APP" 2>&1 | tr -d '\r' | grep -o "logname=$1 rhost=[0-9a-fA-F.:]*" | tail -n 1 | sed 's/.*rhost=//' | grep .
}
# The response head of a WebSocket handshake on /jmap/ws through the proxy.
ws_head() {
    "${CURL[@]}" --silent --http1.1 --include --no-buffer --max-time 5 --header 'Connection: Upgrade' \
        --header 'Upgrade: websocket' --header 'Sec-WebSocket-Version: 13' \
        --header 'Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==' --header 'Sec-WebSocket-Protocol: jmap' \
        "$@" "$BASE/jmap/ws" | tr -d '\r' | sed '/^$/q' || true
}

"case_$CASE"
