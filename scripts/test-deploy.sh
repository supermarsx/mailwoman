#!/usr/bin/env bash
# Real Docker lifecycle test. Never touches the normal mailwoman deployment.
# Usage: bash scripts/test-deploy.sh mailwoman:deploy-verify
# Build the supplied image with: docker build --target runtime -t mailwoman:deploy-verify .
# shellcheck disable=SC2016 # Dollar signs below deliberately test literal env values.
set -Eeuo pipefail
ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
IMAGE=${1:?Pass a built Mailwoman runtime image}
PROJECT="mw-deploy-test-$$-$RANDOM"
STATE=$(mktemp -d "$ROOT/.deploy-test-XXXXXX.local")
PORT=${DEPLOY_TEST_PORT:-18089}
NET_PROJECT="$PROJECT-net"
NET_STATE=$(mktemp -d "$ROOT/.deploy-test-XXXXXX.local")
NETWORK="$PROJECT-proxy"
SERVER_KEY=
PASSED=false

run() { bash "$ROOT/deploy.sh" "$@" --state-dir "$STATE"; }
dc() { docker compose --project-name "$PROJECT" --env-file /dev/null -f "$STATE/active/compose.yaml" "$@"; }
fail() { printf 'FAIL: %s\n' "$*" >&2; exit 1; }
reject() { if run "$@" > "$STATE/rejected.log" 2>&1; then fail "unexpected success: $*"; fi; }
cleanup() {
    local result=$? volume
    trap - EXIT
    # Exact random test project and explicit label checks protect unrelated data.
    if [[ -f $STATE/active/compose.yaml ]]; then dc down --timeout 10 >/dev/null 2>&1 || true; fi
    if [[ -f $NET_STATE/active/compose.yaml ]]; then
        docker compose --project-name "$NET_PROJECT" --env-file /dev/null -f "$NET_STATE/active/compose.yaml" down --timeout 10 >/dev/null 2>&1 || true
    fi
    while IFS= read -r volume; do
        [[ -n $volume && $volume == "${PROJECT}"[_-]* ]] || continue
        docker volume rm "$volume" >/dev/null || true
    done < <(docker volume ls -q --filter "label=mailwoman.deploy.project=$PROJECT"; docker volume ls -q --filter "label=mailwoman.deploy.project=$NET_PROJECT")
    docker network rm "$NETWORK" >/dev/null 2>&1 || true
    if $PASSED && [[ $STATE == "$ROOT"/.deploy-test-*.local && $NET_STATE == "$ROOT"/.deploy-test-*.local ]]; then
        rm -rf -- "$STATE" "$NET_STATE"
    else
        printf 'Test diagnostics retained at %s and %s\n' "$STATE" "$NET_STATE" >&2
    fi
    exit "$result"
}
trap cleanup EXIT

# Input failures, and dry-run's zero-write contract.
run --dry-run --domain mail.example.org
[[ ! -e $STATE/deploy.env ]] || fail 'dry-run created config'
reject --not-an-option
reject --port 70000
reject --domain 'mail.example.org;touch bad'
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
docker inspect --format '{{range .Config.Env}}{{println .}}{{end}}' "$CID" | \
    grep -Fx 'DEPLOY_LITERAL_TEST=dollar$sign with spaces and "quotes"' >/dev/null || fail 'raw environment was expanded'
curl --fail --silent "http://localhost:$PORT/" -o "$STATE/index.html"
grep -qi '<!doctype html' "$STATE/index.html" || fail 'missing embedded SPA'
docker cp "$CID:/data/mailwoman.db" "$STATE/first.db"
[[ -s $STATE/first.db ]] || fail 'database not persisted'

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
REAL_DOCKER=$(command -v docker)
export REAL_DOCKER
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

# A second installation on a dedicated proxy network publishes no host port,
# answers a peer on that network by project name, and trusts that network's
# peers for forwarded headers. Its public URL is unroutable here by design.
net() { bash "$ROOT/deploy.sh" "$@" --project "$NET_PROJECT" --public-url https://mail.example.test --proxy-network "$NETWORK" --timeout 10 --state-dir "$NET_STATE"; }
net deploy --image "$IMAGE"
NET_CID=$(docker ps -q --filter "label=com.docker.compose.project=$NET_PROJECT")
[[ -n $NET_CID && -z $(docker port "$NET_CID") ]] || fail 'proxy-network install published a host port'
MSYS_NO_PATHCONV=1 docker run --rm --network "$NETWORK" "$IMAGE" healthcheck --url "http://$NET_PROJECT:8080/healthz" || fail 'app not reachable by name on the proxy network'
docker inspect --format '{{range .Config.Env}}{{println .}}{{end}}' "$NET_CID" > "$NET_STATE/env"
grep -Fx 'MW_FORWARDED_MODE=xff' "$NET_STATE/env" >/dev/null || fail 'proxy network peers are not trusted'
grep -E '^MW_TRUSTED_PROXIES=[0-9a-f:./,]+$' "$NET_STATE/env" >/dev/null || fail 'proxy network subnet is missing'
# The same command again is an update of the same installation.
NET_VOLUME=$(awk -F= '$1 == "DATA_VOLUME" {print $2}' "$NET_STATE/active/release.env")
net deploy --no-build
[[ $(awk -F= '$1 == "DATA_VOLUME" {print $2}' "$NET_STATE/active/release.env") == "$NET_VOLUME" ]] || fail 'update replaced the data volume'
docker network inspect "$NETWORK" >/dev/null || fail 'update removed the proxy network'
PASSED=true
printf 'PASS: deployment, literal secrets, health, persistent data, backup, rollback, corruption refusal, copy-failure recovery, repeated setup options and a proxy-network installation\n'
