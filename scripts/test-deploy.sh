#!/usr/bin/env bash
# Real Docker lifecycle test. Never touches the normal mailwoman deployment.
# Usage: bash scripts/test-deploy.sh mailwoman:deploy-verify
# Build the supplied image with: docker build --target runtime-static -t mailwoman:deploy-verify .
# When Playwright can start (`pnpm exec playwright` in apps/web, or else
# apps/web/node_modules/.bin/playwright; needs Node, apps/web/node_modules and
# Playwright's chromium) it also drives the admin panel in a browser (project
# `deploy`). Otherwise that step is skipped; DEPLOY_TEST_REQUIRE=all turns the skip
# into a failure, reported before anything is deployed.
# DEPLOY_TEST_PREFIX (default mw-deploy-test) starts the random project names.
# DEPLOY_TEST_PORT (default 18089) is the localhost port of the first installation;
# the two ports above it must be free too. DEPLOY_TEST_SCRIPT runs another copy of
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

# Encrypted backups need gpg or age (BACKUP_RECIPIENT); each one present is tested.
if ! command -v gpg >/dev/null 2>&1 && ! command -v age >/dev/null 2>&1 && [[ ${DEPLOY_TEST_REQUIRE:-} == all ]]; then
    fail 'neither gpg nor age is installed and DEPLOY_TEST_REQUIRE=all: the encrypted backup case cannot run'
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
BAD_IMAGE="$PROJECT-unhealthy:test"
SIGNED=
REGISTRY_IMAGE=registry:2.8.3@sha256:a3d8aaa63ed8681a604f1dea0aa03f100d5895b6a58ace528858a7b332415373
# The base scripts/test-systemd-unit.sh boots; here it only runs systemd-analyze.
SYSTEMD_IMAGE=ubuntu:24.04@sha256:68ee22004a9109b44973313b02ff7b1de250f1de0f5ecb628d73f579aaff33ac
TIMERS='timer units'
SERVER_KEY=
GNUPG_FULL=
GNUPG_PUBLIC=
ENCRYPTION='encrypted backups SKIPPED'
PASSED=false

run() { bash "$DEPLOY" "$@" --state-dir "$STATE"; }
dc() { docker compose --project-name "$PROJECT" --env-file /dev/null -f "$STATE/active/compose.yaml" "$@"; }
reject() { if run "$@" > "$STATE/rejected.log" 2>&1; then fail "unexpected success: $*"; fi; }
port_in_use() { (exec 3<>"/dev/tcp/127.0.0.1/$1") 2>/dev/null; }
# Something else listening on a localhost port: a throwaway app container that
# publishes it. Waits until the app answers there: Docker Desktop's port proxy
# accepts connections for a container that is about to exit, and with --rm the
# port is then free again. /tmp needs mode 1777, as deploy.sh sets it: a tmpfs over
# the static image's /tmp is not writable for the app user otherwise.
hold() {
    local port=$1 i
    MSYS_NO_PATHCONV=1 docker run -d --rm --name "$PROJECT-hold-$port" --label "mailwoman.deploy.test=$PROJECT" \
        --publish "127.0.0.1:$port:8080" --tmpfs /tmp:mode=1777 --env MW_DB_PATH=/tmp/hold.db \
        --env "MW_SERVER_KEY=$(od -An -N32 -tx1 /dev/urandom | tr -d ' \n')" "$IMAGE" >/dev/null
    for i in {1..30}; do
        if curl --fail --silent --max-time 2 "http://127.0.0.1:$port/healthz" -o /dev/null; then return 0; fi
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
    local result=$? volume container reference
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
    if [[ -f $ACME_STATE/caddy-check.yaml ]]; then
        docker compose --project-name "$PROJECT-acme" --env-file /dev/null -f "$ACME_STATE/caddy-check.yaml" down --volumes >/dev/null 2>&1 || true
    fi
    # A compose run that failed can leave its created container behind.
    while IFS= read -r container; do
        [[ -n $container ]] && docker rm --force "$container" >/dev/null 2>&1 || true
    done < <(docker ps -aq --filter "label=com.docker.compose.project=$PROJECT-acme")
    docker volume rm "$PROJECT-acme_caddy_data" "$PROJECT-acme_caddy_config" >/dev/null 2>&1 || true
    docker network rm "$PROJECT-acme_default" >/dev/null 2>&1 || true
    docker network rm "$NETWORK" >/dev/null 2>&1 || true
    # A move from local to network exposure leaves the project's default network.
    docker network rm "${PROJECT}_default" "${NET_PROJECT}_default" >/dev/null 2>&1 || true
    docker image rm "$BAD_IMAGE" "mailwoman-deploy:$PROJECT-active" "mailwoman-deploy:$PROJECT-old" >/dev/null 2>&1 || true
    if [[ -n $SIGNED ]]; then
        docker image rm "$SIGNED" >/dev/null 2>&1 || true
        while IFS= read -r reference; do
            [[ -n $reference ]] && docker image rm "$reference" >/dev/null 2>&1 || true
        done < <(docker image inspect --format '{{range .RepoDigests}}{{println .}}{{end}}' "$IMAGE" 2>/dev/null | grep -F "${SIGNED%:signed}@")
    fi
    local home
    for home in "$GNUPG_FULL" "$GNUPG_PUBLIC"; do
        [[ -n $home && -d $home ]] || continue
        GNUPGHOME=$home gpgconf --kill gpg-agent >/dev/null 2>&1 || true
        rm -rf -- "$home"
    done
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
DEPLOY_TEST_BUILD_ARGS="$ACME_STATE/build.args"
export REAL_DOCKER DEPLOY_TEST_RENDERED DEPLOY_TEST_BUILD_ARGS
mkdir "$ACME_STATE/bin"
cat > "$ACME_STATE/bin/docker" <<'WRAPPER'
#!/usr/bin/env bash
if [[ ${1:-} == build ]]; then
    printf '%s\n' "$@" > "$DEPLOY_TEST_BUILD_ARGS"
    exit 3
fi
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
# A new Caddy installation names Caddy by exact version and digest.
grep -E '^CADDY_IMAGE=caddy:[0-9]+\.[0-9]+\.[0-9]+-alpine@sha256:[0-9a-f]{64}$' "$ACME_STATE/deploy.env" >/dev/null || fail "a new Caddy installation does not pin Caddy: $(grep CADDY_IMAGE "$ACME_STATE/deploy.env")"
if grep -E '^ +acme_ca(_root)? ' "$DEPLOY_TEST_RENDERED" >/dev/null; then fail 'a Caddy installation without ACME_CA names an ACME directory'; fi
# A real certificate stands in for the ACME root, since Caddy parses it below: the
# first one in the test image's CA bundle.
CERT_CID=$(docker create "$IMAGE")
docker cp "$CERT_CID:/etc/ssl/certs/ca-certificates.crt" - | tar -xO > "$ACME_STATE/bundle.pem"
docker rm "$CERT_CID" >/dev/null
awk '/^-----BEGIN CERTIFICATE-----$/ {copy = 1} copy {print} /^-----END CERTIFICATE-----$/ {exit}' "$ACME_STATE/bundle.pem" > "$ACME_STATE/root.pem"
ROOT_LINE=$(sed -n 2p "$ACME_STATE/root.pem")
[[ $ROOT_LINE =~ ^[A-Za-z0-9+/=]+$ ]] || fail 'no certificate in the test image CA bundle'
set_in "$ACME_STATE/deploy.env" ACME_CA=https://pebble.test:14000/dir "ACME_CA_ROOT=$ACME_STATE/root.pem"
acme doctor 2> "$ACME_STATE/doctor.log" || fail "doctor refused valid ACME settings: $(cat "$ACME_STATE/doctor.log")"
# The global options block comes first in the Caddyfile, before the site block.
ACME_LINES=$({ grep -E '^        (\{|mail\.example\.test \{|    acme_ca https://pebble\.test:14000/dir|    acme_ca_root /tmp/acme-ca-root\.pem)$' "$DEPLOY_TEST_RENDERED" || true; } | tr -s ' ' | tr '\n' '|')
[[ $ACME_LINES == ' {| acme_ca https://pebble.test:14000/dir| acme_ca_root /tmp/acme-ca-root.pem| mail.example.test {|' ]] || fail "the Caddyfile does not open with the ACME global options: $ACME_LINES"
grep -Fx '      MW_ACME_CA_ROOT: |' "$DEPLOY_TEST_RENDERED" >/dev/null || fail 'the ACME root is not given to the Caddy container'
grep -Fx "        $ROOT_LINE" "$DEPLOY_TEST_RENDERED" >/dev/null || fail 'the ACME root certificate is not in the Compose file'
printf 'acme: %s\n' "$ACME_LINES"
# The Caddy service as rendered (read-only root filesystem, tmpfs, capabilities,
# entrypoint) starts, and Caddy accepts the Caddyfile and the root the entrypoint
# writes: a copy without the app service and the fixed-address network runs
# `caddy validate` instead of `caddy run`. No port is published by compose run.
CADDY_REF=$(awk -F= '$1 == "CADDY_IMAGE" {print $2}' "$ACME_STATE/deploy.env")
docker image inspect "$CADDY_REF" >/dev/null 2>&1 || docker pull --quiet "$CADDY_REF" >/dev/null
awk -v image="$CADDY_REF" '
    /^[^ ]/ { section = $1; entry = "" }
    /^  [a-z_]/ { entry = $1 }
    section == "services:" && entry == "mailwoman:" { next }
    section == "volumes:" && entry == "app_data:" { next }
    section == "networks:" { next }
    skip && /^      / { next }
    { skip = 0 }
    /^    (networks|depends_on):$/ { skip = 1; next }
    entry == "caddy:" && /^    image: / { print "    image: " image; next }
    { sub(/exec caddy run /, "exec caddy validate "); print }
' "$DEPLOY_TEST_RENDERED" > "$ACME_STATE/caddy-check.yaml"
grep -F 'exec caddy validate ' "$ACME_STATE/caddy-check.yaml" >/dev/null || fail 'the rendered Caddy service does not start Caddy through its entrypoint'
docker compose --project-name "$PROJECT-acme" --env-file /dev/null -f "$ACME_STATE/caddy-check.yaml" run --rm --no-deps caddy > "$ACME_STATE/caddy-check.log" 2>&1 || fail "the rendered Caddy service did not start or Caddy refused its configuration: $(tail -n 5 "$ACME_STATE/caddy-check.log")"
grep -Fx 'Valid configuration' "$ACME_STATE/caddy-check.log" >/dev/null || fail "Caddy did not validate its configuration: $(tail -n 5 "$ACME_STATE/caddy-check.log")"
docker compose --project-name "$PROJECT-acme" --env-file /dev/null -f "$ACME_STATE/caddy-check.yaml" down --volumes >/dev/null 2>&1
printf 'caddy: %s (%s)\n' "$(grep -Fx 'Valid configuration' "$ACME_STATE/caddy-check.log")" "$CADDY_REF"
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
set_in "$PICK_STATE/deploy.env" ACME_CA=

# A new installation's deploy.env carries the Helm chart's limits; values that are
# not a size or a CPU count are refused.
[[ $(grep -E '^(MEMORY|CPU)_LIMIT=' "$PICK_STATE/deploy.env" | tr '\n' ' ') == 'MEMORY_LIMIT=1g CPU_LIMIT=2 ' ]] || fail "init wrote no default limits: $(grep LIMIT "$PICK_STATE/deploy.env")"
pick_reject() {
    local expected=$1
    shift
    set_in "$PICK_STATE/deploy.env" "$@"
    if bash "$DEPLOY" doctor --state-dir "$PICK_STATE" > "$ACME_STATE/rejected.log" 2>&1; then fail "doctor accepted $*"; fi
    grep -F "$expected" "$ACME_STATE/rejected.log" >/dev/null || fail "doctor refused $* for another reason: $(cat "$ACME_STATE/rejected.log")"
}
pick_reject 'MEMORY_LIMIT must be a size' MEMORY_LIMIT=lots
pick_reject 'CPU_LIMIT must be a number of CPUs' MEMORY_LIMIT=1g CPU_LIMIT=0
set_in "$PICK_STATE/deploy.env" CPU_LIMIT=2

# A deploy without --image builds the Dockerfile's runtime-static target, the one
# CI scans, signs and publishes. The docker wrapper records the build and fails it,
# so nothing is built here and the deploy stops before creating anything.
if PATH="$ACME_STATE/bin:$PATH" bash "$DEPLOY" deploy --state-dir "$PICK_STATE" > "$ACME_STATE/build.log" 2>&1; then fail 'a deploy whose image build failed succeeded'; fi
[[ $(tr '\n' ' ' < "$DEPLOY_TEST_BUILD_ARGS") == "build --target runtime-static "* ]] || fail "deploy built: $(tr '\n' ' ' < "$DEPLOY_TEST_BUILD_ARGS")"
[[ -z $(docker volume ls -q --filter "label=mailwoman.deploy.project=$PROJECT-pick") ]] || fail 'a deploy whose build failed created a volume'
printf 'build: %s\n' "$(tr '\n' ' ' < "$DEPLOY_TEST_BUILD_ARGS" | cut -d' ' -f1-3)"

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

run deploy --image "$IMAGE" 2> "$STATE/first-deploy.log" || { cat "$STATE/first-deploy.log" >&2; fail 'first deploy failed'; }
# An image used from the local image store is deployed unchecked, and the deploy
# says so.
grep -F "Signature NOT checked: $IMAGE is used from the local image store" "$STATE/first-deploy.log" >/dev/null || fail "the first deploy did not say the signature was not checked: $(cat "$STATE/first-deploy.log")"
CID=$(dc ps -q mailwoman)
# limits CONTAINER MEMORY_BYTES CPUS WHEN (Compose stores CPUs as a float32, so
# 26 CPUs read back as 25999998976 nanoCPUs; compared to two decimals)
limits() {
    local got
    got=$(docker inspect --format '{{.HostConfig.Memory}} {{.HostConfig.NanoCpus}}' "$1" | awk '{printf "%d %.2f", $1, $2 / 1e9}')
    [[ $got == "$2 $3" ]] || fail "$4: memory and CPU limits are '$got', expected '$2 $3'"
}
limits "$CID" 1073741824 2.00 'a new installation'
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

# A config deploy takes another backup and preserves the key and data. It also
# applies edited limits; a CPU limit above what Docker can use is lowered to that.
HOST_CPUS=$(docker info --format '{{.NCPU}}')
set_config MEMORY_LIMIT=768m CPU_LIMIT=999
run deploy --no-build 2> "$STATE/limits.log" || { cat "$STATE/limits.log" >&2; fail 'config deploy failed'; }
[[ $(awk -F= '$1 == "MW_SERVER_KEY" {print $2}' "$STATE/app.env") == "$SERVER_KEY" ]] || fail 'deploy rotated key'
CID=$(dc ps -q mailwoman)
limits "$CID" 805306368 "$HOST_CPUS.00" 'after editing MEMORY_LIMIT=768m CPU_LIMIT=999'
grep -F "CPU_LIMIT=999 is more than the $HOST_CPUS CPUs Docker can use" "$STATE/limits.log" >/dev/null || fail "the lowered CPU limit was not reported: $(cat "$STATE/limits.log")"
printf 'limits: %s\n' "$(grep -F 'CPU_LIMIT=999' "$STATE/limits.log")"
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
limits "$CID" 1073741824 2.00 'after rolling back the limit change'

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

# prune: BACKUP_KEEP=1 keeps the newest complete snapshot and the one rollback.path
# names, removes the other complete one, removes an incomplete snapshot older than
# seven days but not a recent one, and leaves files it did not write. It lists the
# volume the rollback replaced and an image deploy.sh would have built, deleting
# them only with --replaced --yes, never the active volume or the active image.
ROLLBACK_SNAPSHOT=$(cat "$STATE/rollback.path")
mapfile -t COMPLETE_SNAPSHOTS < <(find "$STATE/backups" -mindepth 2 -maxdepth 2 -name COMPLETE -printf '%h
' | LC_ALL=C sort -r)
((${#COMPLETE_SNAPSHOTS[@]} == 3)) || fail "expected three complete snapshots before prune, found: ${COMPLETE_SNAPSHOTS[*]}"
NEWEST_SNAPSHOT=${COMPLETE_SNAPSHOTS[0]}
[[ $NEWEST_SNAPSHOT != "$ROLLBACK_SNAPSHOT" ]] || fail 'the newest snapshot is the rollback snapshot; the prune case needs another'
for OTHER_SNAPSHOT in "${COMPLETE_SNAPSHOTS[@]}"; do
    [[ $OTHER_SNAPSHOT == "$NEWEST_SNAPSHOT" || $OTHER_SNAPSHOT == "$ROLLBACK_SNAPSHOT" ]] || break
done
mkdir "$STATE/backups/20200101T000000Z-1-1" "$STATE/backups/$(date -u +%Y%m%dT%H%M%SZ)-2-2"
RECENT_INCOMPLETE=$(find "$STATE/backups" -mindepth 1 -maxdepth 1 -name '*-2-2')
printf 'kept by the operator\n' > "$STATE/backups/notes.txt"
docker tag "$IMAGE" "mailwoman-deploy:$PROJECT-active"
OLD_BUILT_CID=$(docker create --label "mailwoman.deploy.test=$PROJECT" "$IMAGE")
docker commit --change "LABEL mailwoman.deploy.test=$PROJECT" "$OLD_BUILT_CID" "mailwoman-deploy:$PROJECT-old" >/dev/null
docker rm "$OLD_BUILT_CID" >/dev/null
ACTIVE_VOLUME=$(awk -F= '$1 == "DATA_VOLUME" {print $2}' "$STATE/active/release.env")
set_config BACKUP_KEEP=1
run prune 2> "$STATE/prune.log" || { cat "$STATE/prune.log" >&2; fail 'prune failed'; }
[[ -d $NEWEST_SNAPSHOT ]] || fail 'prune removed the newest snapshot'
[[ -d $ROLLBACK_SNAPSHOT ]] || fail 'prune removed the snapshot rollback.path names'
[[ ! -e $OTHER_SNAPSHOT ]] || fail "prune kept $OTHER_SNAPSHOT beyond BACKUP_KEEP=1"
[[ ! -e $STATE/backups/20200101T000000Z-1-1 ]] || fail 'prune kept an incomplete snapshot older than seven days'
[[ -d $RECENT_INCOMPLETE ]] || fail 'prune removed a recent incomplete snapshot'
[[ -f $STATE/backups/notes.txt ]] || fail 'prune removed a file it did not write'
grep -F "Replaced data volume: $OLD_VOLUME (" "$STATE/prune.log" >/dev/null || fail "prune did not list the replaced volume $OLD_VOLUME: $(cat "$STATE/prune.log")"
grep -F "Old built image: mailwoman-deploy:$PROJECT-old (" "$STATE/prune.log" >/dev/null || fail "prune did not list the old built image: $(cat "$STATE/prune.log")"
if grep -F -e "$ACTIVE_VOLUME (" -e "mailwoman-deploy:$PROJECT-active" "$STATE/prune.log" >/dev/null; then fail "prune listed the active volume or image: $(cat "$STATE/prune.log")"; fi
docker volume inspect "$OLD_VOLUME" >/dev/null || fail 'prune without --replaced --yes deleted a volume'
run prune --replaced 2> "$STATE/prune.log" || { cat "$STATE/prune.log" >&2; fail 'prune --replaced failed'; }
docker volume inspect "$OLD_VOLUME" >/dev/null || fail 'prune --replaced without --yes deleted a volume'
docker image inspect "mailwoman-deploy:$PROJECT-old" >/dev/null || fail 'prune --replaced without --yes deleted an image'
reject prune --yes
run prune --replaced --yes 2> "$STATE/prune.log" || { cat "$STATE/prune.log" >&2; fail 'prune --replaced --yes failed'; }
if docker volume inspect "$OLD_VOLUME" >/dev/null 2>&1; then fail 'prune --replaced --yes kept the replaced volume'; fi
if docker image inspect "mailwoman-deploy:$PROJECT-old" >/dev/null 2>&1; then fail 'prune --replaced --yes kept the old built image'; fi
docker volume inspect "$ACTIVE_VOLUME" >/dev/null || fail 'prune --replaced --yes deleted the active volume'
docker image inspect "mailwoman-deploy:$PROJECT-active" >/dev/null || fail 'prune --replaced --yes deleted an image the active release uses'
[[ $(docker inspect --format '{{.State.Health.Status}}' "$(dc ps -q mailwoman)") == healthy ]] || fail 'the app is not healthy after prune'
printf 'prune: %s\n' "$(grep -F -e 'Deleted' -e 'kept,' "$STATE/prune.log" | tr '\n' ' ')"
docker image rm "mailwoman-deploy:$PROJECT-active" >/dev/null
rm -r -- "$RECENT_INCOMPLETE" "$STATE/backups/notes.txt"
set_config BACKUP_KEEP=

# Encrypted backups. With BACKUP_RECIPIENT set, backup leaves one archive that
# holds no clear text, and no snapshot directory; the archive restores with the
# private key and is refused without it, leaving the app as it was. gpg: the
# deploy runs with a keyring that holds only the public key (the host needs no
# private key), and the restore with one that holds the private key. age: the
# identity file is given with --identity. A later deploy removes the clear-text
# snapshot the previous deploy took for rollback.
# encrypted_case TOOL RECIPIENT, with BACKUP_ENV (environment of doctor and backup),
# REFUSED_ENV/REFUSED_ARGS (a restore that must be refused) and RESTORE_ENV/
# RESTORE_ARGS (the restore that must succeed).
encrypted_case() {
    local tool=$1 recipient=$2 archive name
    set_config "BACKUP_RECIPIENT=$recipient"
    printf 'encrypted-%s\n' "$tool" > "$STATE/uploads/deploy-test-object"
    docker cp "$STATE/uploads" "$(dc ps -q mailwoman):/data/"
    env "${BACKUP_ENV[@]}" bash "$DEPLOY" doctor --state-dir "$STATE" 2> "$STATE/doctor.log" || fail "doctor refused BACKUP_RECIPIENT for $tool: $(cat "$STATE/doctor.log")"
    grep -F "Backups are encrypted with $tool to $recipient" "$STATE/doctor.log" >/dev/null || fail "doctor did not report $tool: $(cat "$STATE/doctor.log")"
    env "${BACKUP_ENV[@]}" bash "$DEPLOY" backup --state-dir "$STATE" 2> "$STATE/backup.log" || { cat "$STATE/backup.log" >&2; fail "an encrypted backup with $tool failed"; }
    archive=$(grep -o "Encrypted backup ready: [^ ]*" "$STATE/backup.log" | cut -d' ' -f4)
    name=${archive##*/}
    name=${name%.tar."$tool"}
    [[ -s $archive && $archive == "$STATE/backups/"*".tar.$tool" ]] || fail "no $tool archive: $(cat "$STATE/backup.log")"
    [[ ! -e $STATE/backups/$name ]] || fail "the clear-text snapshot $name is still there beside the $tool archive"
    if grep -aF -e "$SERVER_KEY" -e "encrypted-$tool" "$archive" >/dev/null; then fail "the $tool archive contains clear text"; fi
    printf 'after-encrypted-backup\n' > "$STATE/uploads/deploy-test-object"
    CID=$(dc ps -q mailwoman)
    docker cp "$STATE/uploads" "$CID:/data/"
    if env "${REFUSED_ENV[@]}" bash "$DEPLOY" restore "$archive" "${REFUSED_ARGS[@]}" --state-dir "$STATE" > "$STATE/rejected.log" 2>&1; then fail "the $tool archive restored without its private key"; fi
    grep -F 'nothing was changed' "$STATE/rejected.log" >/dev/null || fail "the $tool restore without the key was refused for another reason: $(cat "$STATE/rejected.log")"
    [[ $(dc ps -q mailwoman) == "$CID" && $(docker inspect --format '{{.State.Running}}' "$CID") == true ]] || fail "a refused $tool restore replaced or stopped the app"
    env "${RESTORE_ENV[@]}" bash "$DEPLOY" restore "$archive" "${RESTORE_ARGS[@]}" --state-dir "$STATE" 2> "$STATE/restore.log" || { cat "$STATE/restore.log" >&2; fail "the $tool archive did not restore with its private key"; }
    CID=$(dc ps -q mailwoman)
    docker cp "$CID:/data/uploads/deploy-test-object" "$STATE/restored-object"
    [[ $(cat "$STATE/restored-object") == "encrypted-$tool" ]] || fail "the $tool archive restored other data: $(cat "$STATE/restored-object")"
    # The restored configuration is the snapshot's, but backup settings stay as set.
    grep -Fx "BACKUP_RECIPIENT=$recipient" "$STATE/deploy.env" >/dev/null || fail "the $tool restore reset BACKUP_RECIPIENT in deploy.env"
    [[ -z $(find "$STATE" -maxdepth 1 -name '.work.*') ]] || fail "the decrypted snapshot was left in the state directory"
    ENCRYPTION+=" $tool"
}
ENCRYPTION='encrypted backups with'
if command -v gpg >/dev/null 2>&1; then
    # Short paths: gpg-agent's socket path has a length limit.
    GNUPG_FULL=$(mktemp -d)
    GNUPG_PUBLIC=$(mktemp -d)
    chmod 700 "$GNUPG_FULL" "$GNUPG_PUBLIC"
    GNUPGHOME=$GNUPG_FULL gpg --batch --quiet --pinentry-mode loopback --passphrase '' --quick-generate-key 'mailwoman deploy test <deploy-test@example.invalid>' default default never 2>/dev/null
    FINGERPRINT=$(GNUPGHOME=$GNUPG_FULL gpg --batch --with-colons --list-keys 2>/dev/null | awk -F: '$1 == "fpr" {print $10; exit}')
    [[ $FINGERPRINT =~ ^[0-9A-F]{40}$ ]] || fail "could not create a test gpg key: '$FINGERPRINT'"
    GNUPGHOME=$GNUPG_FULL gpg --batch --export "$FINGERPRINT" | GNUPGHOME=$GNUPG_PUBLIC gpg --batch --quiet --import 2>/dev/null
    BACKUP_ENV=("GNUPGHOME=$GNUPG_PUBLIC"); REFUSED_ENV=("GNUPGHOME=$GNUPG_PUBLIC"); REFUSED_ARGS=()
    RESTORE_ENV=("GNUPGHOME=$GNUPG_FULL"); RESTORE_ARGS=()
    encrypted_case gpg "$FINGERPRINT"
fi
if command -v age >/dev/null 2>&1 && command -v age-keygen >/dev/null 2>&1; then
    age-keygen -o "$STATE/age-identity.txt" 2>/dev/null
    age-keygen -o "$STATE/age-other.txt" 2>/dev/null
    AGE_RECIPIENT=$(age-keygen -y "$STATE/age-identity.txt")
    BACKUP_ENV=(); REFUSED_ENV=(); RESTORE_ENV=(); RESTORE_ARGS=(--identity "$STATE/age-identity.txt")
    # Refused with an identity that does not match, and without --identity.
    REFUSED_ARGS=(--identity "$STATE/age-other.txt")
    encrypted_case age "$AGE_RECIPIENT"
    ARCHIVE=$(find "$STATE/backups" -maxdepth 1 -name '*.tar.age' | head -n 1)
    reject restore "$ARCHIVE"
    grep -F 'give the identity file that matches BACKUP_RECIPIENT with --identity FILE' "$STATE/rejected.log" >/dev/null || fail "an age restore without --identity was refused for another reason: $(cat "$STATE/rejected.log")"
else
    printf 'SKIP: age is not installed; only the gpg encrypted backup case ran.\n' >&2
fi
if [[ $ENCRYPTION == 'encrypted backups with' ]]; then
    ENCRYPTION='encrypted backups SKIPPED (neither gpg nor age)'
else
    PREVIOUS_ROLLBACK=$(cat "$STATE/rollback.path")
    [[ -d $PREVIOUS_ROLLBACK ]] || fail 'the rollback snapshot is missing before the encrypted deploy case'
    run deploy --no-build 2> "$STATE/encrypted-deploy.log" || { cat "$STATE/encrypted-deploy.log" >&2; fail 'a deploy with BACKUP_RECIPIENT set failed'; }
    [[ ! -e $PREVIOUS_ROLLBACK ]] || fail 'with BACKUP_RECIPIENT set, a deploy kept the clear-text snapshot the previous deploy took'
    [[ -d $(cat "$STATE/rollback.path") ]] || fail 'the deploy took no clear-text snapshot for rollback'
    set_config BACKUP_RECIPIENT=
fi
printf '%s\n' "$ENCRYPTION"

# BACKUP_HOOK gets the path of each finished backup. A hook that fails makes
# backup fail, keeps that backup, and stops the retention that would otherwise
# follow; once it succeeds, BACKUP_KEEP=1 removes the older snapshots except the
# rollback one.
mkdir "$STATE/hookbin"
cat > "$STATE/hookbin/hook" <<'HOOK'
#!/usr/bin/env bash
printf '%s\n' "$1" >> "${0%/hookbin/hook}/hook.args"
exit "$(cat "${0%/hookbin/hook}/hook.exit")"
HOOK
chmod +x "$STATE/hookbin/hook"
printf '3\n' > "$STATE/hook.exit"
set_config "BACKUP_HOOK=$STATE/hookbin/hook" BACKUP_KEEP=1
run doctor 2> "$STATE/doctor.log" || fail "doctor refused BACKUP_HOOK: $(cat "$STATE/doctor.log")"
SNAPSHOTS_BEFORE=$(find "$STATE/backups" -mindepth 1 -maxdepth 1 -type d | wc -l)
if run backup > "$STATE/hook.log" 2>&1; then fail 'a backup whose BACKUP_HOOK exited 3 succeeded'; fi
grep -F "BACKUP_HOOK $STATE/hookbin/hook exited 3; the backup is kept at" "$STATE/hook.log" >/dev/null || fail "the failing hook was not reported: $(cat "$STATE/hook.log")"
HOOKED=$(tail -n 1 "$STATE/hook.args")
[[ $HOOKED == "$STATE/backups/"* && -f $HOOKED/COMPLETE ]] || fail "the hook was not given the finished snapshot: '$HOOKED'"
[[ $(find "$STATE/backups" -mindepth 1 -maxdepth 1 -type d | wc -l) == $((SNAPSHOTS_BEFORE + 1)) ]] || fail 'after a failing hook, retention removed snapshots'
printf '0\n' > "$STATE/hook.exit"
run backup 2> "$STATE/hook.log" || { cat "$STATE/hook.log" >&2; fail 'a backup whose BACKUP_HOOK exited 0 failed'; }
HOOKED=$(tail -n 1 "$STATE/hook.args")
[[ -f $HOOKED/COMPLETE ]] || fail "the hook was not given the finished snapshot: '$HOOKED'"
[[ $(find "$STATE/backups" -mindepth 2 -maxdepth 2 -name COMPLETE | wc -l) == 2 && -d $(cat "$STATE/rollback.path") ]] || fail "after a successful hook, BACKUP_KEEP=1 did not leave the new and the rollback snapshot: $(ls "$STATE/backups")"
printf 'backup hook: exit 3 failed the backup and kept it, exit 0 passed and retention ran (%s calls)\n' "$(wc -l < "$STATE/hook.args" | tr -d ' ')"
set_config BACKUP_HOOK= BACKUP_KEEP=

# timers writes a service and a timer for backup and for check, with this
# installation's paths, into the state directory, and prints them with the
# equivalent cron lines; systemd-analyze verify accepts the units (in a container
# with systemd installed; a seeded unit with a bad calendar or a missing
# executable is refused there, so the step is not vacuous).
run timers > "$STATE/timers.out" 2> "$STATE/timers.log" || { cat "$STATE/timers.log" >&2; fail 'timers failed'; }
BASH_PATH=$(command -v bash)
for UNIT in backup.service backup.timer check.service check.timer; do
    [[ -f $STATE/systemd/mailwoman-$PROJECT-$UNIT ]] || fail "timers did not write mailwoman-$PROJECT-$UNIT into the state directory"
done
for WHAT in backup check; do
    grep -Fx "ExecStart=$BASH_PATH $DEPLOY $WHAT --state-dir $STATE" "$STATE/systemd/mailwoman-$PROJECT-$WHAT.service" >/dev/null || fail "the $WHAT unit does not run this installation's $WHAT: $(grep ExecStart "$STATE/systemd/mailwoman-$PROJECT-$WHAT.service")"
done
grep -Fx "30 3 * * * $BASH_PATH $DEPLOY backup --state-dir $STATE" "$STATE/timers.out" >/dev/null || fail "timers printed no cron line for backup: $(tail -n 3 "$STATE/timers.out")"
grep -Fx "*/5 * * * * $BASH_PATH $DEPLOY check --state-dir $STATE" "$STATE/timers.out" >/dev/null || fail "timers printed no cron line for check: $(tail -n 3 "$STATE/timers.out")"
mkdir "$STATE/units-seeded"
cp "$STATE/systemd/"* "$STATE/units-seeded/"
sed -i 's/^OnCalendar=.*/OnCalendar=not-a-date/' "$STATE/units-seeded/mailwoman-$PROJECT-backup.timer"
# verify_units DIR: 0 when systemd-analyze accepts DIR's units, 1 when it refuses
# them, 90 when systemd could not be installed in the container.
verify_units() {
    tar -C "$1" -cf - . | MSYS_NO_PATHCONV=1 timeout 420 docker run -i --rm --label "mailwoman.deploy.test=$PROJECT" "$SYSTEMD_IMAGE" bash -c '
        mkdir -p /etc/systemd/system && tar -xf - -C /etc/systemd/system || exit 90
        export DEBIAN_FRONTEND=noninteractive
        { timeout 150 apt-get update -qq && timeout 200 apt-get install -y -qq --no-install-recommends systemd; } >/dev/null 2>&1 || exit 90
        cd /etc/systemd/system && systemd-analyze verify ./mailwoman-*.service ./mailwoman-*.timer || exit 1' > "$STATE/verify.log" 2>&1
}
VERIFY=0
verify_units "$STATE/systemd" || VERIFY=$?
if ((VERIFY == 90)); then
    [[ ${DEPLOY_TEST_REQUIRE:-} != all ]] || fail 'systemd could not be installed in a container (no network?) and DEPLOY_TEST_REQUIRE=all: the timer units were not verified'
    printf 'SKIP: systemd could not be installed in a container; the timer units were not verified.\n' >&2
    TIMERS='timer units (systemd-analyze SKIPPED)'
else
    ((VERIFY == 0)) || fail "systemd-analyze verify refused the timer units: $(cat "$STATE/verify.log")"
    SEEDED=0
    verify_units "$STATE/units-seeded" || SEEDED=$?
    ((SEEDED == 1)) || fail "systemd-analyze verify on a timer with a bad calendar ended with $SEEDED, expected 1 (refused): $(cat "$STATE/verify.log")"
    grep -F 'Failed to parse calendar specification' "$STATE/verify.log" >/dev/null || fail "the seeded timer was refused for another reason: $(cat "$STATE/verify.log")"
    TIMERS='timer units (systemd-analyze verify)'
fi
printf 'timers: %s; %s\n' "$TIMERS" "$(grep -c '^[*0-9]' "$STATE/timers.out") cron lines"

# --image signature check. A registry container on localhost port DEPLOY_TEST_PORT + 2
# holds a copy of the test image (a fixed port: Docker Desktop's daemon cannot reach
# a randomly assigned one). A stand-in cosign on PATH answers as a passing or failing
# keyless verification and records its arguments: this proves the script's control
# flow and arguments, not sigstore itself.
REGISTRY_PORT=$((PORT + 2))
if port_in_use "$REGISTRY_PORT"; then fail "port $REGISTRY_PORT (DEPLOY_TEST_PORT + 2) must be free for this test"; fi
MSYS_NO_PATHCONV=1 docker run -d --rm --name "$PROJECT-registry" --label "mailwoman.deploy.test=$PROJECT" --publish "127.0.0.1:$REGISTRY_PORT:5000" "$REGISTRY_IMAGE" >/dev/null
SIGNED="127.0.0.1:$REGISTRY_PORT/$PROJECT/mailwoman:signed"
docker tag "$IMAGE" "$SIGNED"
for attempt in {1..10}; do
    if docker push --quiet "$SIGNED" >/dev/null 2>&1; then break; fi
    ((attempt < 10)) || fail "could not push the test image to the local registry"
    sleep 1
done
SIGNED_DIGEST=$(docker image inspect --format '{{range .RepoDigests}}{{println .}}{{end}}' "$SIGNED" | awk -F@ -v repository="${SIGNED%:signed}" '$1 == repository {print $2}')
[[ $SIGNED_DIGEST =~ ^sha256:[0-9a-f]{64}$ ]] || fail "no registry digest for $SIGNED"
mkdir "$STATE/cosignbin"
cat > "$STATE/cosignbin/cosign" <<'STANDIN'
#!/usr/bin/env bash
printf '%s\n' "$@" > "$DEPLOY_TEST_COSIGN_ARGS"
if [[ $DEPLOY_TEST_COSIGN == pass ]]; then
    printf '[{"critical":{"identity":{"docker-reference":"stand-in"},"image":{"docker-manifest-digest":"%s"},"type":"cosign container image signature"},"optional":null}]\n' "$DEPLOY_TEST_COSIGN_DIGEST"
    exit 0
fi
printf 'Error: no matching signatures: none of the expected identities matched what was in the certificate\n' >&2
exit 12
STANDIN
chmod +x "$STATE/cosignbin/cosign"
DEPLOY_TEST_COSIGN_ARGS="$STATE/cosign.args"
DEPLOY_TEST_COSIGN_DIGEST=$SIGNED_DIGEST
export DEPLOY_TEST_COSIGN_ARGS DEPLOY_TEST_COSIGN_DIGEST
BACKUPS=$(find "$STATE/backups" -mindepth 1 -maxdepth 1 -type d | wc -l)
# A failed verification stops the deploy before any snapshot or container change.
DEPLOY_TEST_COSIGN=fail PATH="$STATE/cosignbin:$PATH" reject deploy --image "$SIGNED" --pull
grep -F "The signature of $SIGNED did not verify; nothing was changed" "$STATE/rejected.log" >/dev/null || fail "a failed verification was not refused for that reason: $(cat "$STATE/rejected.log")"
[[ $(dc ps -q mailwoman) == "$CID" && $(docker inspect --format '{{.State.Running}}' "$CID") == true ]] || fail 'a refused signature replaced or stopped the app'
[[ $(find "$STATE/backups" -mindepth 1 -maxdepth 1 -type d | wc -l) == "$BACKUPS" ]] || fail 'a refused signature took a snapshot'
printf 'signature refused: %s\n' "$(grep -F 'did not verify' "$STATE/rejected.log" | cut -c1-140)"
# A passing verification deploys the digest that was verified, and cosign was asked
# for this repository's signing identity and issuer.
DEPLOY_TEST_COSIGN=pass PATH="$STATE/cosignbin:$PATH" run deploy --image "$SIGNED" --pull 2> "$STATE/signed.log" || { cat "$STATE/signed.log" >&2; fail 'a deploy with a verified signature failed'; }
grep -F "Signature verified: ${SIGNED%:signed}@$SIGNED_DIGEST" "$STATE/signed.log" >/dev/null || fail "the verified digest was not deployed: $(cat "$STATE/signed.log")"
[[ $(tr '\n' ' ' < "$DEPLOY_TEST_COSIGN_ARGS") == "verify --certificate-identity https://github.com/supermarsx/mailwoman/.github/workflows/supply-chain.yml@refs/heads/master --certificate-oidc-issuer https://token.actions.githubusercontent.com $SIGNED " ]] || fail "cosign was called with: $(tr '\n' ' ' < "$DEPLOY_TEST_COSIGN_ARGS")"
CID=$(dc ps -q mailwoman)
[[ $(docker inspect --format '{{.Image}}' "$CID") == "$(docker image inspect --format '{{.Id}}' "$IMAGE")" ]] || fail 'the verified image is not the one running'
printf 'signature verified: %s\n' "$(grep -F 'Signature verified' "$STATE/signed.log")"
# Without cosign the signature cannot be checked: --require-signature refuses.
NO_COSIGN_PATH=
IFS=: read -r -a PATH_DIRS <<< "$PATH"
for dir in "${PATH_DIRS[@]}"; do
    [[ -x $dir/cosign ]] || NO_COSIGN_PATH+="${NO_COSIGN_PATH:+:}$dir"
done
PATH="$NO_COSIGN_PATH" reject deploy --image "$SIGNED" --pull --require-signature
grep -F "The signature of $SIGNED cannot be checked (cosign is not installed) and --require-signature was given" "$STATE/rejected.log" >/dev/null || fail "--require-signature without cosign was not refused for that reason: $(cat "$STATE/rejected.log")"
# Nor is an image used from the local image store, cosign or not.
DEPLOY_TEST_COSIGN=pass PATH="$STATE/cosignbin:$PATH" reject deploy --image "$IMAGE" --require-signature
grep -F "$IMAGE is used from the local image store; --pull fetches it from its registry and checks it there) and --require-signature was given" "$STATE/rejected.log" >/dev/null || fail "--require-signature with a local-only image was not refused for that reason: $(cat "$STATE/rejected.log")"
[[ $(dc ps -q mailwoman) == "$CID" ]] || fail 'a refused --require-signature deploy replaced the app'
docker rm --force "$PROJECT-registry" >/dev/null

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

# An update that never becomes healthy is rolled back: the previous image, the
# previous exposure (the proxy network, its trust lines, no host port) and a copy
# of the data saved just before the update come back on a new volume; the failed
# release's files and volume are kept and the lock is released. The update also
# moves the installation to local, so a rollback that kept the new Compose file
# would publish a port. The broken image is the test image with an unknown command.
BAD_CID=$(docker create --label "mailwoman.deploy.test=$PROJECT" "$IMAGE")
docker commit --change 'CMD ["no-such-command"]' "$BAD_CID" "$BAD_IMAGE" >/dev/null
docker rm "$BAD_CID" >/dev/null
BAD_ID=$(docker image inspect --format '{{.Id}}' "$BAD_IMAGE")
GOOD_ID=$(awk -F= '$1 == "IMAGE_ID" {print $2}' "$STATE/active/release.env")
printf 'before-failed-update\n' > "$STATE/uploads/deploy-test-object"
docker cp "$STATE/uploads" "$CID:/data/"
# rolled_back WHEN: the network release with the seeded object runs again.
rolled_back() {
    [[ ! -d $STATE/.lock ]] || fail "$1: the lock was left behind"
    [[ $(awk -F= '$1 == "IMAGE_ID" {print $2}' "$STATE/active/release.env") == "$GOOD_ID" ]] || fail "$1: the previous image is not the active one"
    grep -Fx 'EXPOSURE=network' "$STATE/active/deploy.env" >/dev/null || fail "$1: the previous exposure is not back: $(grep EXPOSURE "$STATE/active/deploy.env")"
    CID=$(dc ps -q mailwoman)
    [[ -n $CID && $(docker inspect --format '{{.Image}} {{.State.Health.Status}}' "$CID") == "$GOOD_ID healthy" ]] || fail "$1: the previous image is not running healthy"
    [[ -z $(docker port "$CID") ]] || fail "$1: the app publishes $(docker port "$CID")"
    MSYS_NO_PATHCONV=1 docker run --rm --network "$NETWORK" "$IMAGE" healthcheck --url "http://$PROJECT:8080/" || fail "$1: http://$PROJECT:8080/ does not answer on the proxy network"
    app_env "$CID" | grep -Fx 'MW_FORWARDED_MODE=xff' >/dev/null || fail "$1: the app does not trust its proxy network"
    docker cp "$CID:/data/uploads/deploy-test-object" "$STATE/restored-object"
    [[ $(cat "$STATE/restored-object") == before-failed-update ]] || fail "$1: the data saved before the update is not there"
    ROLLED_VOLUME=$(awk -F= '$1 == "DATA_VOLUME" {print $2}' "$STATE/active/release.env")
    [[ $ROLLED_VOLUME == "${PROJECT}_restore_"* && $ROLLED_VOLUME != "$PRE_VOLUME" ]] || fail "$1: the data was not restored into a new volume (now $ROLLED_VOLUME, the failed update used $PRE_VOLUME)"
    docker volume inspect "$PRE_VOLUME" >/dev/null || fail "$1: the volume the failed update used, $PRE_VOLUME, was removed"
}
set_config EXPOSURE=local "PUBLIC_URL=http://localhost:$PORT" PROXY_NETWORK=
PRE_VOLUME=$MOVE_VOLUME
if run deploy --image "$BAD_IMAGE" --timeout 60 > "$STATE/failed-update.log" 2>&1; then fail 'an update to an image that never becomes healthy succeeded'; fi
grep -F 'The update was rolled back' "$STATE/failed-update.log" >/dev/null || { cat "$STATE/failed-update.log" >&2; fail 'a failed update was not rolled back'; }
rolled_back 'after the automatic rollback'
FAILED_DIR=$(find "$STATE/failed" -mindepth 1 -maxdepth 1 -type d | head -n 1)
grep -F "127.0.0.1:$PORT:8080" "$FAILED_DIR/compose.yaml" >/dev/null || fail 'the failed release files were not kept'
grep -Fx "IMAGE_ID=$BAD_ID" "$FAILED_DIR/release.env" >/dev/null || fail 'the kept release files are not those of the failed release'
grep -Fx 'EXPOSURE=local' "$STATE/deploy.env" >/dev/null || fail 'the automatic rollback rewrote deploy.env'
printf 'auto-rollback: %s\n' "$(grep -o 'The update was rolled back\. The previous release ([^)]*)' "$STATE/failed-update.log")"
# With --no-auto-rollback the failed update stays in place; rollback then restores
# the snapshot taken before it, exposure included, although the failed app cannot
# become healthy for the snapshot that rollback takes of it first.
PRE_VOLUME=$ROLLED_VOLUME
if run deploy --image "$BAD_IMAGE" --timeout 30 --no-auto-rollback > "$STATE/failed-update.log" 2>&1; then fail 'an update to an image that never becomes healthy succeeded'; fi
if grep -F 'rolled back' "$STATE/failed-update.log" >/dev/null; then fail 'a deploy with --no-auto-rollback rolled back'; fi
[[ $(awk -F= '$1 == "IMAGE_ID" {print $2}' "$STATE/active/release.env") == "$BAD_ID" ]] || fail 'with --no-auto-rollback the failed release is not the active one'
run rollback > "$STATE/rollback.log" 2>&1 || { cat "$STATE/rollback.log" >&2; fail 'rollback after a failed update failed'; }
rolled_back 'after rollback of an update made with --no-auto-rollback'
MOVE_VOLUME=$(awk -F= '$1 == "DATA_VOLUME" {print $2}' "$STATE/active/release.env")
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
# A deploy.env written before the limit keys existed renders no limit, and deploy
# says so in one line.
grep -v -E '^(MEMORY|CPU)_LIMIT=' "$NET_STATE/deploy.env" > "$NET_STATE/deploy.env.old"
cat "$NET_STATE/deploy.env.old" > "$NET_STATE/deploy.env"
net deploy --no-build 2> "$NET_STATE/limits.log" || { cat "$NET_STATE/limits.log" >&2; fail 'update of the proxy-network installation failed'; }
NET_CID=$(docker ps -q --filter "label=com.docker.compose.project=$NET_PROJECT")
limits "$NET_CID" 0 0.00 'a deploy.env without MEMORY_LIMIT and CPU_LIMIT'
grep -F 'No memory or CPU limit for the app' "$NET_STATE/limits.log" >/dev/null || fail "deploy did not say the app runs without limits: $(cat "$NET_STATE/limits.log")"
[[ $(awk -F= '$1 == "DATA_VOLUME" {print $2}' "$NET_STATE/active/release.env") == "$NET_VOLUME" ]] || fail 'update replaced the data volume'
docker network inspect "$NETWORK" >/dev/null || fail 'update removed the proxy network'
PASSED=true
printf 'PASS: dry-run writes nothing (empty and applied state), port auto-pick when 8080 is taken, deployment, literal secrets, health, persistent data, admin login and writes under strict CSRF (curl), %s, refusal of a held port before any change, backup, rollback, corruption refusal, copy-failure recovery, repeated setup options, exposure moves local -> external -> network -> local with the data volume kept, refusal of a move to caddy, a proxy-network installation, the ACME directory settings for Caddy (rendered, accepted by Caddy in the rendered read-only service, and refused when invalid), Caddy pinned by version and digest for a new installation, memory/CPU limits (defaults for a new installation, edited, lowered to the CPUs Docker has, rolled back, none for an older deploy.env, invalid values refused), and the automatic rollback of an update that never becomes healthy (previous image, exposure network with its trust lines, data saved before the update; failed release kept), rollback after --no-auto-rollback, the Dockerfile target a deploy without --image builds (runtime-static), and the --image signature check with a stand-in cosign (verified digest deployed, failed verification refused before any change, --require-signature without cosign or for an image from the local store refused, unchecked local image reported)\n' "$BROWSER"
