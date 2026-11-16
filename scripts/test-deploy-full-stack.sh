#!/usr/bin/env bash
# Live sidecar and backup/restore checks; never starts or changes a real install.
# These globals are read by functions extracted from deploy.sh.
# shellcheck disable=SC2034
set -Eeuo pipefail
ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
work=$(mktemp -d)
project="mw-stack-test-$$"
trap 'docker compose -p "$project" -f "$work/active/compose.yaml" down --remove-orphans >/dev/null 2>&1 || true; docker volume rm "${project}_postgres" "${project}_restored_postgres" "${project}_app" >/dev/null 2>&1 || true; rm -rf -- "$work"' EXIT
log() { printf '%s\n' "$*"; }
die() { printf 'FAIL: %s\n' "$*" >&2; exit 1; }
# Source just the renderer and formatting helper; the deployment's main entry
# point is exercised separately by the init/doctor checks.
source "$ROOT/scripts/deploy-stack.sh"
# shellcheck source=/dev/null
source <(awk '/^write_config\(\)/ { copy=1 } /^validate_app_env\(\)/ { copy=0 } copy { print }' "$ROOT/deploy.sh")
# shellcheck source=/dev/null
source <(awk '/^indent\(\)/ { copy=1 } /^verify\(\)/ { copy=0 } copy { print }' "$ROOT/deploy.sh")
compose() { local directory=$1; shift; docker compose -p "$project" --env-file /dev/null --project-directory "$directory" -f "$directory/compose.yaml" "$@"; }
PROJECT=$project STACK=full MODE=proxy EXPOSURE=local PUBLIC_URL=http://localhost:18182 PORT=18182
POSTGRES_IMAGE=postgres:16 VALKEY_IMAGE=valkey/valkey:8 POSTGRES_VOLUME=${project}_postgres DATA_VOLUME=${project}_app
GEOIP_FILE=dbip-country-lite.mmdb
CPU_LIMIT='' MEMORY_LIMIT='' PROXY_NETWORK='' PROXY_SUBNETS='' UPSTREAM_DEFAULT='' IMAGE_DIGEST='' CADDY_ID=''
PROXY_PREFIX=172.30.70 CADDY_IMAGE=caddy:2-alpine CADDY_PROTOCOLS='h1 h2 h3' DOMAIN='' ACME_CA='' ACME_CA_ROOT=''
SIGNATURE_IDENTITY='' SIGNATURE_ISSUER='' IMAGE_REPOSITORY='' BACKUP_KEEP='' BACKUP_KEEP_DAYS='' BACKUP_RECIPIENT=''
BACKUP_HOOK='' BACKUP_TARGETS_FILE='' ALERT_HOOK='' STATE=$work ACTIVE=$work/active TIMEOUT=120 HELPER_CONTAINER=''
mkdir -p "$ACTIVE"
printf 'MW_SERVER_KEY=%064d\n' 0 > "$ACTIVE/app.env"
stack_secrets "$ACTIVE/app.env"
cp "$ACTIVE/app.env" "$work/original.env"
stack_secrets "$ACTIVE/app.env"
cmp "$ACTIVE/app.env" "$work/original.env" || die 'credentials changed on repeat'
stack_prepare
IMAGE_ID=$(docker image inspect --format '{{.Id}}' "$POSTGRES_IMAGE")
docker volume create "$DATA_VOLUME" >/dev/null
render "$ACTIVE"
compose "$ACTIVE" config --format json > "$work/config.json"
python3 - "$work/config.json" <<'PY'
import json,sys
c=json.load(open(sys.argv[1])); s=c['services']; app=s['mailwoman']
assert set(s)=={'mailwoman','postgres','valkey'}
assert set(c['volumes'])=={'app_data','postgres_data'}, 'Caddy resources must be absent without managed HTTPS'
assert not s['postgres'].get('ports') and not s['valkey'].get('ports')
assert c['networks']['store']['internal']
assert set(app['networks'])=={'default','store'}
assert app['environment']['MW_DB_PATH'].startswith('postgres://')
assert app['environment']['MW_REDIS_URL'].startswith('redis://:')
assert app['depends_on']['postgres']['condition']=='service_healthy'
PY
# Check both BYO proxy shapes and a managed-Caddy positive control using the
# real renderer and Compose merger. No proxy/app container is started here.
for exposure in external network caddy; do
    EXPOSURE=$exposure PUBLIC_URL=https://mail.example.test DOMAIN=mail.example.test
    PROXY_NETWORK=${project}_proxy CADDY_ID=$IMAGE_ID
    render "$ACTIVE"
    compose "$ACTIVE" config --format json > "$work/$exposure.json"
    python3 - "$work/$exposure.json" "$exposure" "$project" <<'PY'
import json,sys
c=json.load(open(sys.argv[1])); exposure=sys.argv[2]; project=sys.argv[3]
s=c['services']; app=s['mailwoman']; managed=exposure=='caddy'
assert set(s)==({'mailwoman','postgres','valkey','caddy'} if managed else {'mailwoman','postgres','valkey'})
assert set(c['volumes'])==({'app_data','postgres_data','caddy_data','caddy_config'} if managed else {'app_data','postgres_data'})
assert app['environment']['MW_COOKIE_SECURE']=='true'
assert app['environment']['MW_PUBLIC_URL']=='https://mail.example.test'
assert not s['postgres'].get('ports') and not s['valkey'].get('ports')
assert set(s['postgres']['networks'])=={'store'} and set(s['valkey']['networks'])=={'store'}
assert c['networks']['store']['internal']
if exposure=='network':
    assert not app.get('ports')
    assert set(app['networks'])=={'proxy','store'}
    assert c['networks']['proxy']['external']
    assert c['networks']['proxy']['name']==project+'_proxy'
    assert project in app['networks']['proxy']['aliases']
else:
    assert set(app['networks'])=={'default','store'}
    if exposure=='external':
        assert len(app['ports'])==1 and app['ports'][0]['host_ip']=='127.0.0.1'
        assert app['ports'][0]['target']==8080
PY
done
EXPOSURE=local PUBLIC_URL=http://localhost:18182 DOMAIN='' CADDY_ID=''
# render() adds managed trust to its effective app.env; keep the original
# operator configuration for subsequent backup/restore checks.
cp "$work/original.env" "$ACTIVE/app.env"
render "$ACTIVE"
compose "$ACTIVE" up -d --wait --wait-timeout "$TIMEOUT" postgres valkey
compose "$ACTIVE" exec -T postgres psql -U mailwoman -d mailwoman -v ON_ERROR_STOP=1 -c "CREATE TABLE fixture (content text NOT NULL); INSERT INTO fixture VALUES ('backup retained');" >/dev/null
mkdir "$work/backup"
tar -cf "$work/backup/data.tar" --files-from /dev/null
stack_dump "$work/backup"
tar -tf "$work/backup/data.tar" | grep -qx '.deploy-postgres.dump' || die 'backup omitted PostgreSQL'
# Copy the archive into the new app volume as restore_snapshot does.
HELPER_CONTAINER=$(docker create --network none --mount "type=volume,src=$DATA_VOLUME,dst=/data" "$IMAGE_ID")
docker cp -a - "$HELPER_CONTAINER:/data" < "$work/backup/data.tar"
docker rm "$HELPER_CONTAINER" >/dev/null
HELPER_CONTAINER=''
compose "$ACTIVE" stop postgres >/dev/null
POSTGRES_VOLUME=${project}_restored_postgres
stack_prepare
render "$ACTIVE"
stack_restore_database "$ACTIVE"
value=$(compose "$ACTIVE" exec -T postgres psql -U mailwoman -d mailwoman -Atc 'SELECT content FROM fixture')
[[ $value == 'backup retained' ]] || die 'PostgreSQL restore lost content'
# Reverse-proxy deployments retain that connection alongside the isolated store.
EXPOSURE=network PROXY_NETWORK=${project}_proxy
render "$ACTIVE"
compose "$ACTIVE" config --format json > "$work/network.json"
python3 - "$work/network.json" <<'PY'
import json,sys
c=json.load(open(sys.argv[1])); assert set(c['services']['mailwoman']['networks'])=={'proxy','store'}
PY
printf 'PASS: stable secrets, private sidecars, health readiness, complete PostgreSQL backup/restore, host and Docker-network BYO proxy topology without Caddy, managed HTTPS control\n'
