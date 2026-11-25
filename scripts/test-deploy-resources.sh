#!/usr/bin/env bash
# Exercise actual resource config parsing/rendering with a mocked Docker CPU report.
# Starts no container and does not need a Docker daemon.
set -Eeuo pipefail
TEST_ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
DEPLOY=$TEST_ROOT/deploy.sh
# shellcheck source=/dev/null
source <(awk '/^usage\(\)/ {exit} {print}' "$DEPLOY")
ROOT=$TEST_ROOT
fail() { printf 'FAIL: %s\n' "$*" >&2; exit 1; }
die() { fail "$*"; }
log() { printf '%s\n' "$*" >&2; }
# shellcheck source=/dev/null
source <(awk '/^read_config\(\)/ {copy=1} /^validate_config\(\)/ {copy=0} copy {print}' "$DEPLOY")
# shellcheck source=/dev/null
source <(awk '/^write_config\(\)/ {copy=1} /^validate_app_env\(\)/ {copy=0} copy {print}' "$DEPLOY")
# shellcheck source=/dev/null
source <(awk '/^indent\(\)/ {copy=1} /^verify\(\)/ {copy=0} copy {print}' "$DEPLOY")
# shellcheck source=scripts/deploy-stack.sh
source "$TEST_ROOT/scripts/deploy-stack.sh"
work=$(mktemp -d)
trap 'rm -rf -- "$work"' EXIT
TEST_HOST_CPUS=8 TEST_HOST_MEMORY=$((16 * 1024 * 1024 * 1024))
docker() {
    case $* in
        "info --format {{.NCPU}}") printf '%s\n' "$TEST_HOST_CPUS" ;;
        "info --format {{.MemTotal}}") printf '%s\n' "$TEST_HOST_MEMORY" ;;
        *) fail "unexpected Docker operation: $*" ;;
    esac
}
IMAGE_ID=sha256:$(printf '0%.0s' {1..64}) DATA_VOLUME=fixture-resources PROJECT=fixture-resources
GEOIP_FILE=dbip-country-lite.mmdb
mkdir "$work/active"
printf 'MW_SERVER_KEY=fixture-only\n' > "$work/active/app.env"
MEMORY_LIMIT=4g CPU_LIMIT=4
render "$work/active"
grep -qx '    mem_limit: 4g' "$work/active/compose.yaml" || fail 'memory default did not reach the app container'
grep -qx '    cpus: 4' "$work/active/compose.yaml" || fail 'CPU default did not reach the app container'
# Reading existing explicit limits must not substitute new defaults.
printf 'MEMORY_LIMIT=768m\nCPU_LIMIT=1.5\n' > "$work/deploy.env"
read_config "$work/deploy.env"
render "$work/active"
grep -qx '    mem_limit: 768m' "$work/active/compose.yaml" || fail 'saved memory override was discarded'
grep -qx '    cpus: 1.5' "$work/active/compose.yaml" || fail 'saved CPU override was discarded'
# Historical files without limits retain their unlimited behavior.
printf 'PROJECT=fixture-resources\n' > "$work/deploy.env"
read_config "$work/deploy.env"
render "$work/active"
! grep -qE '^    (mem_limit|cpus):' "$work/active/compose.yaml" || fail 'missing historical limits acquired defaults'
# CPU ceiling is reduced on smaller Docker hosts; keep configured intent intact.
MEMORY_LIMIT=4g CPU_LIMIT=4 TEST_HOST_CPUS=2
render "$work/active" 2> "$work/clamp.log"
grep -qx '    cpus: 2' "$work/active/compose.yaml" || fail 'CPU ceiling was not clamped to Docker capacity'
grep -qx 'CPU_LIMIT=4' "$work/active/deploy.env" || fail 'rendering rewrote the operator CPU setting'
grep -q 'more than the 2 CPUs' "$work/clamp.log" || fail 'CPU adjustment was not explained'
# A host smaller than the limit: the app gets its share (2 GiB host, full stack
# with Caddy: 2048 - 256 reserve - 64 Caddy - 128 Valkey - 32 = 1568, 65% = 1020).
[[ $(stack_app_memory_limit 4g 2048 full 1) == 1020m ]] || fail 'app share on a 2 GiB full-stack host'
[[ $(stack_app_memory_limit 768m 2048 full 1) == 768m ]] || fail 'a limit below the share must stay'
[[ $(stack_app_memory_limit 4g 1024 single 0) == 768m ]] || fail 'app share on a 1 GiB single-container host'
[[ $(stack_app_memory_limit 4g 16384 single 0) == 4g ]] || fail 'a large host keeps the configured limit'
STACK=single EXPOSURE=local MEMORY_LIMIT=4g CPU_LIMIT=4 TEST_HOST_CPUS=2 TEST_HOST_MEMORY=$((1024 * 1024 * 1024))
render "$work/active" 2> "$work/memory.log"
grep -qx '    mem_limit: 768m' "$work/active/compose.yaml" || fail 'the 4 GiB default was not lowered to the share of a 1 GiB host'
grep -qx 'MEMORY_LIMIT=4g' "$work/active/deploy.env" || fail 'rendering rewrote the operator memory setting'
grep -q "more than the app's share" "$work/memory.log" || fail 'memory adjustment was not explained'
printf 'PASS: 4 GiB/four CPU rendering, saved overrides, historical unlimited configuration, CPU capacity clamp, memory share on small hosts\n'
