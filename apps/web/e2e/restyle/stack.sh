#!/usr/bin/env bash
# Engine-mode stack for the t32 restyle harness (see README.md in this folder).
#
#   stack.sh up   <id> <portBase> <distDir>   start Greenmail + mailwoman-engine serving <distDir>, seed mail
#   stack.sh swap <id> <portBase> <distDir>   down (volumes too), then up with another dist and the same seed
#   stack.sh down <id>                         remove the project's containers, networks and volumes
#   stack.sh env  <id>                         print the env file the Playwright config reads
#
# The compose override (docker-compose.dev.yml + image, ports, web dir, admin) is
# written to a scratch directory, never to the repository:
#   ${T32_STACK_DIR:-${TMPDIR:-/tmp}/mw-t32-stack}/<project>/
# Ports inside <portBase>: SMTP +25, IMAP +43, POP3 +10, HTTP +90.
# The project is "t32-<id>" (an id that already starts with "t32-" is used as is).
# Images: MW_IMAGE (default mailwoman:t31-w0) and Greenmail from docker-compose.dev.yml,
# always with --no-build. Set MW_BASE_PATH to serve under a sub-path (for example /mail).
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo="$(cd "$here/../../../.." && pwd)"
image="${MW_IMAGE:-mailwoman:t31-w0}"

usage() {
  sed -n '2,7p' "$0" >&2
  exit 2
}

[ $# -ge 2 ] || usage
cmd="$1"
id="$2"
case "$id" in
  t32-*) project="$id" ;;
  *) project="t32-$id" ;;
esac
dir="${T32_STACK_DIR:-${TMPDIR:-/tmp}/mw-t32-stack}/$project"
override="$dir/compose.override.yml"
envfile="$dir/stack.env"

# Git Bash rewrites path-like values passed to native programs; docker gets Windows paths as written.
export MSYS_NO_PATHCONV=1

compose() {
  docker compose -p "$project" -f "$(native_path "$repo/docker-compose.dev.yml")" -f "$(native_path "$override")" "$@"
}

native_path() {
  # A dist directory as Docker Desktop on Windows expects it (F:/x), unchanged elsewhere.
  if command -v cygpath >/dev/null 2>&1; then cygpath -m "$1"; else printf '%s\n' "$1"; fi
}

write_override() {
  local base="$1" dist="$2" password="$3"
  mkdir -p "$dir"
  local subpath=""
  if [ -n "${MW_BASE_PATH:-}" ]; then
    subpath="      MW_BASE_PATH: \"${MW_BASE_PATH}\""
  fi
  cat >"$override" <<EOF
services:
  greenmail:
    ports: !override
      - "$((base + 25)):3025"
      - "$((base + 43)):3143"
      - "$((base + 10)):3110"
  mailwoman-engine:
    image: $image
    environment:
      MW_WEB_DIR: /web
      MW_ADMIN_ENABLED: "true"
      MW_ADMIN_USER: t32admin
      MW_ADMIN_PASSWORD: "$password"
$subpath
    volumes:
      - "$dist:/web:ro"
    ports: !override
      - "$((base + 90)):8080"
EOF
  cat >"$envfile" <<EOF
T32_STACK_PROJECT=$project
PLAYWRIGHT_ENGINE_BASE_URL=http://localhost:$((base + 90))${MW_BASE_PATH:-}
MW_E2E_SMTP_HOST=127.0.0.1
MW_E2E_SMTP_PORT=$((base + 25))
MW_ADMIN_USER=t32admin
MW_ADMIN_PASSWORD=$password
T32_DIST=$dist
T32_IMAGE=$image
EOF
}

wait_healthy() {
  local url="$1" i
  for i in $(seq 1 90); do
    if curl -fsS "$url/healthz" >/dev/null 2>&1; then
      return 0
    fi
    sleep 2
  done
  echo "stack.sh: $url did not answer within 180 s" >&2
  compose ps >&2 || true
  compose logs --tail 40 mailwoman-engine >&2 || true
  return 1
}

up() {
  [ $# -ge 3 ] || usage
  local base="$3" dist
  dist="$(native_path "$(cd "$4" && pwd)")"
  [ -f "$4/index.html" ] || { echo "stack.sh: $4 has no index.html" >&2; exit 1; }
  docker image inspect "$image" >/dev/null 2>&1 || { echo "stack.sh: image $image is missing (no docker build here)" >&2; exit 1; }
  local password
  password="$(od -An -N16 -tx1 /dev/urandom | tr -d ' \n')"
  write_override "$base" "$dist" "$password"
  compose up -d --no-build --wait greenmail mailwoman-engine
  wait_healthy "http://localhost:$((base + 90))"
  # The seed is part of the measured state: same subjects, senders and order on every stack.
  MW_E2E_SMTP_HOST=127.0.0.1 MW_E2E_SMTP_PORT=$((base + 25)) node "$(native_path "$here/seed.mjs")"
  echo "stack.sh: $project up on http://localhost:$((base + 90))${MW_BASE_PATH:-} serving $dist"
  echo "stack.sh: env file $(native_path "$envfile") (T32_STACK_ENV)"
}

down() {
  if [ -f "$override" ]; then
    compose down -v --remove-orphans
  else
    # No override left: remove whatever carries the project label.
    docker ps -aq --filter "label=com.docker.compose.project=$project" | xargs -r docker rm -f
    docker volume ls -q --filter "label=com.docker.compose.project=$project" | xargs -r docker volume rm
    docker network ls -q --filter "label=com.docker.compose.project=$project" | xargs -r docker network rm
  fi
  rm -rf "$dir"
  echo "stack.sh: $project removed"
}

case "$cmd" in
  up) up "$@" ;;
  swap)
    [ $# -ge 4 ] || usage
    down
    up "$@"
    ;;
  down) down ;;
  env) cat "$envfile" ;;
  *) usage ;;
esac
