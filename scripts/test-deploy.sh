#!/usr/bin/env bash
# Real Docker lifecycle test. Never touches the normal mailwoman deployment.
# Usage: bash scripts/test-deploy.sh mailwoman:deploy-verify
# Build the supplied image with: docker build --target runtime -t mailwoman:deploy-verify .
# When Playwright can start (`pnpm exec playwright` in apps/web, or else
# apps/web/node_modules/.bin/playwright; needs Node, apps/web/node_modules and
# Playwright's chromium) it also drives the admin panel in a browser (project
# `deploy`). Otherwise that step is skipped; DEPLOY_TEST_REQUIRE=all turns the skip
# into a failure, reported before anything is deployed.
# DEPLOY_TEST_PREFIX (default mw-deploy-test) starts the random project names.
# DEPLOY_TEST_PORT (default 18089) is the localhost port of the first installation;
# the port above it must be free too. DEPLOY_TEST_SCRIPT runs another copy of
# deploy.sh, e.g. one with a seeded defect, to show a case fails.
# One step holds localhost:8080 for a few seconds when it is free (see below).
# shellcheck disable=SC2016 # Dollar signs below deliberately test literal env values.
set -Eeuo pipefail
ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
IMAGE=${1:?Pass a built Mailwoman runtime image}
DEPLOY=${DEPLOY_TEST_SCRIPT:-$ROOT/deploy.sh}
WEB="$ROOT/apps/web"
fail() { printf 'FAIL: %s\n' "$*" >&2; exit 1; }

# Decide first whether the browser step can run, so DEPLOY_TEST_REQUIRE=all fails
# before any container, volume or directory exists.
PLAYWRIGHT=()
if (cd "$WEB" && pnpm exec playwright --version) >/dev/null 2>&1; then
    # Run from apps/web: corepack picks the pinned pnpm from the working directory.
    PLAYWRIGHT=(pnpm exec playwright)
elif [[ -x $WEB/node_modules/.bin/playwright ]] && "$WEB/node_modules/.bin/playwright" --version >/dev/null 2>&1; then
    PLAYWRIGHT=("$WEB/node_modules/.bin/playwright")
elif [[ ${DEPLOY_TEST_REQUIRE:-} == all ]]; then
    fail 'Playwright cannot start (neither `pnpm exec playwright` in apps/web nor apps/web/node_modules/.bin/playwright runs) and DEPLOY_TEST_REQUIRE=all: the admin panel browser test cannot run'
fi

PROJECT="${DEPLOY_TEST_PREFIX:-mw-deploy-test}-$$-$RANDOM"
STATE=$(mktemp -d "$ROOT/.deploy-test-XXXXXX.local")
PORT=${DEPLOY_TEST_PORT:-18089}
NEXT_PORT=$((PORT + 1))
NET_PROJECT="$PROJECT-net"
NET_STATE=$(mktemp -d "$ROOT/.deploy-test-XXXXXX.local")
PICK_STATE=$(mktemp -d "$ROOT/.deploy-test-XXXXXX.local")
ACME_STATE=$(mktemp -d "$ROOT/.deploy-test-XXXXXX.local")
NETWORK="$PROJECT-proxy"
SERVER_KEY=
PASSED=false

run() { bash "$DEPLOY" "$@" --state-dir "$STATE"; }
dc() { docker compose --project-name "$PROJECT" --env-file /dev/null -f "$STATE/active/compose.yaml" "$@"; }
reject() { if run "$@" > "$STATE/rejected.log" 2>&1; then fail "unexpected success: $*"; fi; }
port_in_use() { (exec 3<>"/dev/tcp/127.0.0.1/$1") 2>/dev/null; }
# Something else listening on a localhost port: a throwaway app container that
# publishes it. Waits until the port accepts connections.
hold() {
    local port=$1 i
    MSYS_NO_PATHCONV=1 docker run -d --rm --name "$PROJECT-hold-$port" --label "mailwoman.deploy.test=$PROJECT" \
        --publish "127.0.0.1:$port:8080" --tmpfs /tmp --env MW_DB_PATH=/tmp/hold.db \
        --env "MW_SERVER_KEY=$(od -An -N32 -tx1 /dev/urandom | tr -d ' \n')" "$IMAGE" >/dev/null
    for i in {1..30}; do
        if port_in_use "$port"; then return 0; fi
        sleep 1
    done
    fail "could not hold port $port (attempts: $i)"
}
release() { docker rm --force "$PROJECT-hold-$1" >/dev/null 2>&1 || true; }
# Rewrites KEY=value lines of a deploy.env, as a user would.
set_in() {
    local pair file=$1
    shift
    for pair in "$@"; do
        awk -v key="${pair%%=*}" -v line="$pair" -F= '$1 == key {print line; next} {print}' "$file" > "$file.new"
        cat "$file.new" > "$file"
        rm -- "$file.new"
    done
}
set_config() { set_in "$STATE/deploy.env" "$@"; }
app_env() { docker inspect --format '{{range .Config.Env}}{{println .}}{{end}}' "$1"; }
state_listing() { (cd "$STATE" && find . -print | sort && find . -type f -exec sha256sum {} + | sort); }
cleanup() {
    local result=$? volume container
    trap - EXIT
    # Exact random test project and explicit label checks protect unrelated data.
    while IFS= read -r container; do
        [[ -n $container ]] && docker rm --force "$container" >/dev/null 2>&1 || true
    done < <(docker ps -aq --filter "label=mailwoman.deploy.test=$PROJECT")
    if [[ -f $STATE/active/compose.yaml ]]; then dc down --timeout 10 >/dev/null 2>&1 || true; fi
    if [[ -f $NET_STATE/active/compose.yaml ]]; then
        docker compose --project-name "$NET_PROJECT" --env-file /dev/null -f "$NET_STATE/active/compose.yaml" down --timeout 10 >/dev/null 2>&1 || true
    fi
    while IFS= read -r volume; do
        [[ -n $volume && $volume == "${PROJECT}"[_-]* ]] || continue
        docker volume rm "$volume" >/dev/null || true
    done < <(docker volume ls -q --filter "label=mailwoman.deploy.project=$PROJECT"; docker volume ls -q --filter "label=mailwoman.deploy.project=$NET_PROJECT")
    docker network rm "$NETWORK" >/dev/null 2>&1 || true
    if $PASSED && [[ $STATE == "$ROOT"/.deploy-test-*.local && $NET_STATE == "$ROOT"/.deploy-test-*.local && $PICK_STATE == "$ROOT"/.deploy-test-*.local && $ACME_STATE == "$ROOT"/.deploy-test-*.local ]]; then
        rm -rf -- "$STATE" "$NET_STATE" "$PICK_STATE" "$ACME_STATE"
    else
        printf 'Test diagnostics retained at %s, %s, %s and %s\n' "$STATE" "$NET_STATE" "$PICK_STATE" "$ACME_STATE" >&2
    fi
    exit "$result"
}
trap cleanup EXIT

# Input failures, and dry-run's zero-write contract.
run --dry-run --domain mail.example.org
[[ -z $(ls -A "$STATE") ]] || fail "dry-run wrote into an empty state directory: $(ls -A "$STATE")"
reject --not-an-option
reject --port 70000
reject --domain 'mail.example.org;touch bad'

# A new local installation without --port takes 8080, or the first free port from
# 8081 to 8099 when 8080 is taken. When 8080 is free a listener takes it for the
# length of one init, which starts nothing.
HELD_8080=false
if ! port_in_use 8080; then hold 8080; HELD_8080=true; fi
bash "$DEPLOY" init --project "$PROJECT-pick" --state-dir "$PICK_STATE" 2> "$STATE/pick.log" || { $HELD_8080 && release 8080; fail "init without --port failed: $(cat "$STATE/pick.log")"; }
if $HELD_8080; then release 8080; fi
PICKED=$(awk -F= '$1 == "PORT" {print $2}' "$PICK_STATE/deploy.env")
if [[ ! $PICKED =~ ^[0-9]+$ ]] || ((PICKED < 8081 || PICKED > 8099)); then
    fail "with 8080 taken a new installation chose port '$PICKED', expected one from 8081 to 8099"
fi
grep -Fx "PUBLIC_URL=http://localhost:$PICKED" "$PICK_STATE/deploy.env" >/dev/null || fail "the picked port $PICKED is not in PUBLIC_URL"
grep -F "Port 8080 is in use; using $PICKED instead" "$STATE/pick.log" >/dev/null || fail "init did not say which port it chose: $(cat "$STATE/pick.log")"
printf 'auto-pick: 8080 %s; %s\n' "$($HELD_8080 && echo 'held by this test' || echo 'already in use')" "$(grep -F 'Port 8080 is in use' "$STATE/pick.log")"
rm -- "$STATE/pick.log"

# A Caddy installation may name another ACME directory and the root that signs its
# TLS certificate. doctor renders the Compose model without starting anything; a
# docker wrapper keeps a copy of the file it validates. Caddy itself is not started
# here (it needs ports 80 and 443): scripts/test-deploy-proxy.sh runs it against Pebble.
REAL_DOCKER=$(command -v docker)
DEPLOY_TEST_RENDERED="$ACME_STATE/rendered.yaml"
export REAL_DOCKER DEPLOY_TEST_RENDERED
mkdir "$ACME_STATE/bin"
cat > "$ACME_STATE/bin/docker" <<'WRAPPER'
#!/usr/bin/env bash
if [[ ${1:-} == compose && " $* " == *" config --quiet "* ]]; then
    previous=
    for arg in "$@"; do
        if [[ $previous == -f ]]; then cp -- "$arg" "$DEPLOY_TEST_RENDERED"; fi
        previous=$arg
    done
fi
exec "$REAL_DOCKER" "$@"
WRAPPER
chmod +x "$ACME_STATE/bin/docker"
acme() { PATH="$ACME_STATE/bin:$PATH" bash "$DEPLOY" "$@" --state-dir "$ACME_STATE"; }
acme_reject() {
    local expected=$1
    shift
    set_in "$ACME_STATE/deploy.env" "$@"
    if acme doctor > "$ACME_STATE/rejected.log" 2>&1; then fail "doctor accepted $*"; fi
    grep -F "$expected" "$ACME_STATE/rejected.log" >/dev/null || fail "doctor refused $* for another reason: $(cat "$ACME_STATE/rejected.log")"
}
acme init --project "$PROJECT-acme" --domain mail.example.test 2> "$ACME_STATE/doctor.log" || fail "init of a Caddy installation failed: $(cat "$ACME_STATE/doctor.log")"
acme doctor 2> "$ACME_STATE/doctor.log" || fail "doctor refused a Caddy installation: $(cat "$ACME_STATE/doctor.log")"
if grep -E '^ +acme_ca(_root)? ' "$DEPLOY_TEST_RENDERED" >/dev/null; then fail 'a Caddy installation without ACME_CA names an ACME directory'; fi
printf -- '-----BEGIN CERTIFICATE-----\nTWFpbHdvbWFuIGRlcGxveSB0ZXN0IHJvb3Q=\n-----END CERTIFICATE-----\n' > "$ACME_STATE/root.pem"
set_in "$ACME_STATE/deploy.env" ACME_CA=https://pebble.test:14000/dir "ACME_CA_ROOT=$ACME_STATE/root.pem"
acme doctor 2> "$ACME_STATE/doctor.log" || fail "doctor refused valid ACME settings: $(cat "$ACME_STATE/doctor.log")"
# The global options block comes first in the Caddyfile, before the site block.
ACME_LINES=$(grep -E '^      (\{|mail\.example\.test \{|    acme_ca https://pebble\.test:14000/dir|    acme_ca_root /etc/caddy/acme-ca-root\.pem)$' "$DEPLOY_TEST_RENDERED" | tr -s ' ' | tr '\n' '|')
[[ $ACME_LINES == ' {| acme_ca https://pebble.test:14000/dir| acme_ca_root /etc/caddy/acme-ca-root.pem| mail.example.test {|' ]] || fail "the Caddyfile does not open with the ACME global options: $ACME_LINES"
grep -Fx '        target: /etc/caddy/acme-ca-root.pem' "$DEPLOY_TEST_RENDERED" >/dev/null || fail 'the ACME root is not given to the Caddy container'
grep -Fx '      TWFpbHdvbWFuIGRlcGxveSB0ZXN0IHJvb3Q=' "$DEPLOY_TEST_RENDERED" >/dev/null || fail 'the ACME root certificate is not in the Compose file'
printf 'acme: %s\n' "$ACME_LINES"
cp "$ACME_STATE/deploy.env" "$ACME_STATE/deploy.env.good"
acme_reject 'ACME_CA must be the https:// URL of an ACME directory' ACME_CA=http://pebble.test:14000/dir
cat "$ACME_STATE/deploy.env.good" > "$ACME_STATE/deploy.env"
acme_reject 'ACME_CA_ROOT must be an absolute path' ACME_CA_ROOT=root.pem
acme_reject 'ACME_CA_ROOT names no readable file' "ACME_CA_ROOT=$ACME_STATE/absent.pem"
printf 'not a certificate\n' > "$ACME_STATE/other.txt"
acme_reject 'ACME_CA_ROOT must hold PEM certificates only' "ACME_CA_ROOT=$ACME_STATE/other.txt"
set_in "$PICK_STATE/deploy.env" ACME_CA=https://pebble.test:14000/dir
if bash "$DEPLOY" doctor --state-dir "$PICK_STATE" > "$ACME_STATE/rejected.log" 2>&1; then fail 'a local installation accepted ACME_CA'; fi
grep -F 'ACME_CA and ACME_CA_ROOT apply only to EXPOSURE=caddy' "$ACME_STATE/rejected.log" >/dev/null || fail "ACME_CA on a local installation was refused for another reason: $(cat "$ACME_STATE/rejected.log")"

run init --project "$PROJECT" --port "$PORT"
SERVER_KEY=$(awk -F= '$1 == "MW_SERVER_KEY" {print $2}' "$STATE/app.env")
run init
# Repeating the setup options unchanged is accepted; a different value is refused.
run init --project "$PROJECT" --port "$PORT"
reject init --port $((PORT + 1))
[[ $(awk -F= '$1 == "MW_SERVER_KEY" {print $2}' "$STATE/app.env") == "$SERVER_KEY" ]] || fail 'init rotated key'
printf 'DEPLOY_LITERAL_TEST=dollar$sign with spaces and "quotes"\n' >> "$STATE/app.env"
run doctor
mkdir "$STATE/.lock"
reject deploy --image "$IMAGE"
rmdir "$STATE/.lock"

run deploy --image "$IMAGE"
CID=$(dc ps -q mailwoman)
[[ $(docker inspect --format '{{.State.Health.Status}}' "$CID") == healthy ]] || fail 'not healthy'
[[ $(docker inspect --format '{{.Config.User}}' "$CID") == 65532:65532 ]] || fail 'not nonroot'
[[ $(docker inspect --format '{{.HostConfig.ReadonlyRootfs}}' "$CID") == true ]] || fail 'root writable'
app_env "$CID" | grep -Fx 'DEPLOY_LITERAL_TEST=dollar$sign with spaces and "quotes"' >/dev/null || fail 'raw environment was expanded'
curl --fail --silent "http://localhost:$PORT/" -o "$STATE/index.html"
grep -qi '<!doctype html' "$STATE/index.html" || fail 'missing embedded SPA'
docker cp "$CID:/data/mailwoman.db" "$STATE/first.db"
[[ -s $STATE/first.db ]] || fail 'database not persisted'
# On an applied installation dry-run changes nothing either.
LISTING=$(state_listing)
run deploy --dry-run
[[ $(state_listing) == "$LISTING" ]] || fail 'dry-run changed the state directory of an applied installation'

# The admin panel of a default installation, which runs with MW_CSRF_STRICT=true.
# With curl first: the generated password signs in and the answer sets mw_csrf; a
# write without X-CSRF-Token is refused, which shows strict mode is on in the
# deployed app whatever the web client does; the same write with the header is
# accepted; a wrong password is 401, not the guard's 403.
ADMIN_USER=$(awk -F= '$1 == "MW_ADMIN_USER" {print $2}' "$STATE/app.env")
ADMIN_PASSWORD=$(awk -F= '$1 == "MW_ADMIN_PASSWORD" {print $2}' "$STATE/app.env")
ADMIN_URL="http://localhost:$PORT/admin"
JAR="$STATE/admin.cookies"
BAN='{"ip":"203.0.113.250","reason":"deploy test"}'
admin() { curl --silent --output "$STATE/admin.body" --write-out '%{http_code}' --cookie "$JAR" --cookie-jar "$JAR" "$@"; }
admin_json() { admin --header 'content-type: application/json' "$@"; }
CODE=$(admin_json --data "{\"username\":\"$ADMIN_USER\",\"password\":\"not-$ADMIN_PASSWORD\"}" "$ADMIN_URL/login")
[[ $CODE == 401 ]] || fail "a wrong admin password answered $CODE, expected 401"
CODE=$(admin_json --data "{\"username\":\"$ADMIN_USER\",\"password\":\"$ADMIN_PASSWORD\"}" "$ADMIN_URL/login")
[[ $CODE == 200 ]] || fail "admin login with the generated password answered $CODE: $(cat "$STATE/admin.body")"
CSRF=$(awk '$6 == "mw_csrf" {print $7}' "$JAR")
[[ -n $CSRF ]] || fail 'admin login set no mw_csrf cookie'
CODE=$(admin_json --data "$BAN" "$ADMIN_URL/bans")
[[ $CODE == 403 ]] || fail "an admin write without X-CSRF-Token answered $CODE, expected 403: strict CSRF is not on in the deployed app"
grep -F 'missing or invalid CSRF token' "$STATE/admin.body" >/dev/null || fail "unexpected refusal: $(cat "$STATE/admin.body")"
CODE=$(admin_json --header "x-csrf-token: $CSRF" --data "$BAN" "$ADMIN_URL/bans")
[[ $CODE == 2?? ]] || fail "an admin write with X-CSRF-Token answered $CODE: $(cat "$STATE/admin.body")"
CODE=$(admin --request DELETE --header "x-csrf-token: $CSRF" "$ADMIN_URL/bans/203.0.113.250")
[[ $CODE == 2?? ]] || fail "removing the test ban answered $CODE: $(cat "$STATE/admin.body")"

# The same panel in a browser, where the web client has to send the token itself
# (apps/web/e2e/deploy-admin.spec.ts). Playwright's own summary is printed.
BROWSER='admin panel in a browser'
if ((${#PLAYWRIGHT[@]})); then
    (cd "$WEB" && MW_DEPLOY_BASE_URL="http://localhost:$PORT" MW_ADMIN_USER="$ADMIN_USER" MW_ADMIN_PASSWORD="$ADMIN_PASSWORD" \
        "${PLAYWRIGHT[@]}" test --project=deploy --reporter=list --output "$STATE/playwright" > "$STATE/playwright.log" 2>&1) \
        || { cat "$STATE/playwright.log" >&2; fail 'the admin panel browser test failed'; }
    grep -E '^ +1 passed' "$STATE/playwright.log" >/dev/null || { cat "$STATE/playwright.log" >&2; fail 'the admin panel browser test did not run'; }
    grep -E '^ +[0-9]+ (passed|failed|flaky|skipped)|✓|✘' "$STATE/playwright.log" | sed 's/^/browser: /'
else
    BROWSER='admin panel in a browser SKIPPED (Playwright cannot start)'
    printf 'SKIP: Playwright cannot start; the admin panel browser test did not run. Set DEPLOY_TEST_REQUIRE=all to make this a failure.\n' >&2
fi

# A deploy whose localhost port is held by something else stops before it builds,
# snapshots or replaces anything; the running app keeps serving.
if port_in_use "$NEXT_PORT"; then fail "port $NEXT_PORT (DEPLOY_TEST_PORT + 1) must be free for this test"; fi
hold "$NEXT_PORT"
cp "$STATE/deploy.env" "$STATE/deploy.env.saved"
set_config "PORT=$NEXT_PORT" "PUBLIC_URL=http://localhost:$NEXT_PORT"
reject deploy --no-build
grep -F "Port $NEXT_PORT is already in use on this host" "$STATE/rejected.log" >/dev/null || fail "a deploy onto a held port was not refused for that reason: $(cat "$STATE/rejected.log")"
[[ $(dc ps -q mailwoman) == "$CID" && $(docker inspect --format '{{.State.Running}}' "$CID") == true ]] || fail 'a refused deploy replaced or stopped the app'
[[ ! -e $STATE/backups ]] || fail 'a refused deploy took a snapshot'
printf 'held port: %s\n' "$(grep -F 'already in use' "$STATE/rejected.log")"
release "$NEXT_PORT"
cat "$STATE/deploy.env.saved" > "$STATE/deploy.env"
rm -- "$STATE/deploy.env.saved"

# Seed an opaque upload-tree object to verify full-volume backup and ownership.
mkdir "$STATE/uploads"
printf 'before-snapshot\n' > "$STATE/uploads/deploy-test-object"
docker cp "$STATE/uploads" "$CID:/data/"
run backup
SNAPSHOT=$(find "$STATE/backups" -mindepth 1 -maxdepth 1 -type d | head -n 1)
[[ -f $SNAPSHOT/COMPLETE ]] || fail 'missing complete snapshot'
tar --numeric-owner -tvf "$SNAPSHOT/data.tar" > "$STATE/tar-listing"
grep -E '65532/65532.*\./mailwoman.db$' "$STATE/tar-listing" >/dev/null || fail 'backup lost database ownership'

# A config deploy takes another backup and preserves the key and data.
run deploy --no-build
[[ $(awk -F= '$1 == "MW_SERVER_KEY" {print $2}' "$STATE/app.env") == "$SERVER_KEY" ]] || fail 'deploy rotated key'
CID=$(dc ps -q mailwoman)
printf 'after-snapshot\n' > "$STATE/uploads/deploy-test-object"
docker cp "$STATE/uploads" "$CID:/data/"
OLD_VOLUME=$(awk -F= '$1 == "DATA_VOLUME" {print $2}' "$STATE/active/release.env")
run rollback
NEW_VOLUME=$(awk -F= '$1 == "DATA_VOLUME" {print $2}' "$STATE/active/release.env")
[[ $NEW_VOLUME != "$OLD_VOLUME" ]] || fail 'restore reused the old volume'
docker volume inspect "$OLD_VOLUME" >/dev/null || fail 'restore deleted old data'
CID=$(dc ps -q mailwoman)
docker cp "$CID:/data/uploads/deploy-test-object" "$STATE/restored-object"
[[ $(cat "$STATE/restored-object") == before-snapshot ]] || fail 'upload object did not restore'

# Corrupt snapshots are refused before taking down the live application.
printf 'tampered\n' >> "$SNAPSHOT/app.env"
reject restore "$SNAPSHOT"
[[ $(docker inspect --format '{{.State.Running}}' "$CID") == true ]] || fail 'bad restore stopped live app'

# Snapshot-copy failure must resume the stopped writer, leave no COMPLETE marker,
# and release the lock. A small Docker wrapper injects precisely that failure.
mkdir "$STATE/bin"
cat > "$STATE/bin/docker" <<'WRAPPER'
#!/usr/bin/env bash
if [[ ${1:-} == cp && ${2:-} == -a ]]; then exit 42; fi
exec "$REAL_DOCKER" "$@"
WRAPPER
chmod +x "$STATE/bin/docker"
if PATH="$STATE/bin:$PATH" run backup > "$STATE/copy-failure.log" 2>&1; then fail 'copy failure was ignored'; fi
[[ $(docker inspect --format '{{.State.Running}}' "$CID") == true ]] || fail 'copy failure left app stopped'
[[ ! -d $STATE/.lock ]] || fail 'copy failure leaked lock'
run start
run stop
run start
run status

# The applied installation moves among local, external and proxy-network exposure
# when deploy.env is edited and deploy runs again, keeping its data volume; a move
# to Caddy is refused. For the external step a curl wrapper stands in for the host's
# reverse proxy: deploy.sh's public check of https://mail.example.test is answered
# by the app on its localhost port.
MOVE_VOLUME=$(awk -F= '$1 == "DATA_VOLUME" {print $2}' "$STATE/active/release.env")
moved() {
    CID=$(dc ps -q mailwoman)
    [[ -n $CID && $(docker inspect --format '{{.State.Health.Status}}' "$CID") == healthy ]] || fail "after the move to $1 the app is not healthy"
    [[ $(awk -F= '$1 == "DATA_VOLUME" {print $2}' "$STATE/active/release.env") == "$MOVE_VOLUME" ]] || fail "the move to $1 replaced the data volume"
    app_env "$CID" > "$STATE/env"
}
local_port_only() {
    [[ $(docker port "$CID") == "8080/tcp -> 127.0.0.1:$PORT" ]] || fail "$1 does not publish exactly 127.0.0.1:$PORT: $(docker port "$CID")"
    curl --fail --silent "http://localhost:$PORT/" -o "$STATE/index.html" || fail "$1: http://localhost:$PORT/ does not answer"
    grep -qi '<!doctype html' "$STATE/index.html" || fail "$1: http://localhost:$PORT/ is not the web UI"
}
mkdir "$STATE/proxybin"
REAL_CURL=$(command -v curl)
DEPLOY_TEST_PROXY_TO="http://127.0.0.1:$PORT"
export REAL_CURL DEPLOY_TEST_PROXY_TO
cat > "$STATE/proxybin/curl" <<'WRAPPER'
#!/usr/bin/env bash
args=()
for arg in "$@"; do
    if [[ $arg == https://mail.example.test/* ]]; then arg="$DEPLOY_TEST_PROXY_TO/${arg#https://mail.example.test/}"; fi
    args+=("$arg")
done
exec "$REAL_CURL" "${args[@]}"
WRAPPER
chmod +x "$STATE/proxybin/curl"
# local -> external
set_config EXPOSURE=external PUBLIC_URL=https://mail.example.test
PATH="$STATE/proxybin:$PATH" run deploy --no-build || fail 'deploy did not move the installation from local to external'
moved external
local_port_only 'the external installation'
grep -Fx 'MW_PUBLIC_URL=https://mail.example.test' "$STATE/env" >/dev/null || fail 'the external installation kept its old public URL'
grep -Fx 'MW_COOKIE_SECURE=true' "$STATE/env" >/dev/null || fail 'the external installation does not use Secure cookies'
# external -> network
set_config EXPOSURE=network "PROXY_NETWORK=$NETWORK"
run deploy --no-build || fail 'deploy did not move the installation from external to network'
moved network
[[ -z $(docker port "$CID") ]] || fail "the installation moved to a proxy network still publishes $(docker port "$CID")"
if port_in_use "$PORT"; then fail "localhost:$PORT still answers after the move to a proxy network"; fi
MSYS_NO_PATHCONV=1 docker run --rm --network "$NETWORK" "$IMAGE" healthcheck --url "http://$PROJECT:8080/" || fail "http://$PROJECT:8080/ does not answer on the proxy network"
grep -Fx 'MW_FORWARDED_MODE=xff' "$STATE/env" >/dev/null || fail 'the moved installation does not trust its proxy network'
# network -> local
set_config EXPOSURE=local "PUBLIC_URL=http://localhost:$PORT" PROXY_NETWORK=
run deploy --no-build || fail 'deploy did not move the installation from network to local'
moved local
local_port_only 'the installation moved back to local'
[[ $(docker inspect --format '{{range $name, $_ := .NetworkSettings.Networks}}{{$name}} {{end}}' "$CID") != *"$NETWORK "* ]] || fail 'the local installation is still on the proxy network'
grep -Fx 'MW_COOKIE_SECURE=false' "$STATE/env" >/dev/null || fail 'the local installation uses Secure cookies over HTTP'
if grep -E '^MW_(FORWARDED_MODE|TRUSTED_PROXIES)=' "$STATE/env" >/dev/null; then fail 'the local installation still trusts the proxy network'; fi
docker network inspect "$NETWORK" >/dev/null || fail 'leaving the proxy network removed it'
# local -> caddy is refused before anything changes. CADDY_IMAGE names an image that
# cannot exist, so a script that lacked the refusal could not start Caddy on this
# host's ports 80 and 443.
cp "$STATE/deploy.env" "$STATE/deploy.env.saved"
set_config EXPOSURE=caddy DOMAIN=mail.example.test PUBLIC_URL=https://mail.example.test CADDY_IMAGE=mailwoman-deploy-test.invalid/absent:none
reject deploy --no-build
grep -F 'EXPOSURE cannot change from local to caddy: an installation cannot move to or from caddy' "$STATE/rejected.log" >/dev/null || fail "a move to caddy was not refused for that reason: $(cat "$STATE/rejected.log")"
[[ $(dc ps -q mailwoman) == "$CID" && $(docker inspect --format '{{.State.Running}}' "$CID") == true ]] || fail 'a refused move to caddy replaced or stopped the app'
printf 'caddy move: %s\n' "$(grep -F 'cannot move to or from caddy' "$STATE/rejected.log")"
cat "$STATE/deploy.env.saved" > "$STATE/deploy.env"
rm -- "$STATE/deploy.env.saved"

# A second installation on a dedicated proxy network publishes no host port,
# answers a peer on that network by project name, and trusts that network's
# peers for forwarded headers. Its public URL is unroutable here by design.
net() { bash "$DEPLOY" "$@" --project "$NET_PROJECT" --public-url https://mail.example.test --proxy-network "$NETWORK" --timeout 10 --state-dir "$NET_STATE"; }
net deploy --image "$IMAGE"
NET_CID=$(docker ps -q --filter "label=com.docker.compose.project=$NET_PROJECT")
[[ -n $NET_CID && -z $(docker port "$NET_CID") ]] || fail 'proxy-network install published a host port'
MSYS_NO_PATHCONV=1 docker run --rm --network "$NETWORK" "$IMAGE" healthcheck --url "http://$NET_PROJECT:8080/healthz" || fail 'app not reachable by name on the proxy network'
app_env "$NET_CID" > "$NET_STATE/env"
grep -Fx 'MW_FORWARDED_MODE=xff' "$NET_STATE/env" >/dev/null || fail 'proxy network peers are not trusted'
grep -E '^MW_TRUSTED_PROXIES=[0-9a-f:./,]+$' "$NET_STATE/env" >/dev/null || fail 'proxy network subnet is missing'
# The same command again is an update of the same installation.
NET_VOLUME=$(awk -F= '$1 == "DATA_VOLUME" {print $2}' "$NET_STATE/active/release.env")
net deploy --no-build
[[ $(awk -F= '$1 == "DATA_VOLUME" {print $2}' "$NET_STATE/active/release.env") == "$NET_VOLUME" ]] || fail 'update replaced the data volume'
docker network inspect "$NETWORK" >/dev/null || fail 'update removed the proxy network'
PASSED=true
printf 'PASS: dry-run writes nothing (empty and applied state), port auto-pick when 8080 is taken, deployment, literal secrets, health, persistent data, admin login and writes under strict CSRF (curl), %s, refusal of a held port before any change, backup, rollback, corruption refusal, copy-failure recovery, repeated setup options, exposure moves local -> external -> network -> local with the data volume kept, refusal of a move to caddy, a proxy-network installation, and the ACME directory settings for Caddy (rendered, and refused when invalid)\n' "$BROWSER"
