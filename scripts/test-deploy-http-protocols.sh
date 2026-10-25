#!/usr/bin/env bash
# Render actual deployment manifests, validate with Compose and the installed Caddy.
set -Eeuo pipefail
TEST_ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
DEPLOY="$TEST_ROOT/deploy.sh"
source <(awk '/^usage\(\)/ { exit } { print }' "$DEPLOY")
ROOT=$TEST_ROOT
fail() { printf 'FAIL: %s\n' "$*" >&2; exit 1; }
die() { fail "$*"; }
log() { printf '%s\n' "$*" >&2; }
source <(awk '/^write_config\(\)/ { copy=1 } /^validate_app_env\(\)/ { copy=0 } copy { print }' "$DEPLOY")
source <(awk '/^indent\(\)/ { copy=1 } /^verify\(\)/ { copy=0 } copy { print }' "$DEPLOY")
mkdir -p "$ROOT/.orchestration/reports"
FIXTURE=$(mktemp -d "$ROOT/.orchestration/reports/http-protocols.XXXXXX")
PROJECT=codex-http-protocols EXPOSURE=caddy DOMAIN=mail.example.test PUBLIC_URL=https://mail.example.test
IMAGE_ID=$(docker image inspect --format '{{.Id}}' "${DEPLOY_TEST_CADDY_IMAGE:-caddy:2-alpine}")
CADDY_ID=$IMAGE_ID DATA_VOLUME=codex-http-protocols-data CPU_LIMIT= MEMORY_LIMIT=
for protocols in 'h1 h2 h3' 'h1 h2' 'h1'; do
    CADDY_PROTOCOLS=$protocols
    directory="$FIXTURE/${protocols// /-}"
    mkdir -p "$directory"
    : > "$directory/app.env"
    render "$directory"
    docker compose -f "$directory/compose.yaml" config --format json > "$directory/compose.json"
    python - "$directory" "$protocols" <<'PY'
import json, pathlib, sys
p = pathlib.Path(sys.argv[1]); protocols = sys.argv[2]
config = json.loads((p / 'compose.json').read_text())
caddy = config['services']['caddy']
udp = [port for port in caddy['ports'] if port.get('protocol') == 'udp']
assert bool(udp) == ('h3' in protocols), udp
assert not udp or int(udp[0]['target']) == 443
text = caddy['environment']['MW_CADDYFILE']
assert 'protocols ' + protocols in text
(p / 'Caddyfile').write_text(text)
assert 'CADDY_PROTOCOLS=' + protocols in (p / 'deploy.env').read_text()
PY
    # Feed the actual rendered file over stdin: no Windows mount-path conversion.
    MSYS_NO_PATHCONV=1 docker run --rm -i "$CADDY_ID" caddy validate --config /dev/stdin --adapter caddyfile < "$directory/Caddyfile"
    printf 'PASS: %s Compose ports, persisted options and Caddy configuration\n' "$protocols"
done
printf 'Artifacts: %s\n' "$FIXTURE"
