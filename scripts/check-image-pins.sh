#!/usr/bin/env bash
# Fail when a base image is named by tag alone.
#
# A tag such as `node:22-alpine` can point at different bytes from one day to the next; a digest
# cannot. This script checks:
#   - every FROM in a Dockerfile names its image as `name:tag@sha256:<64 hex>`. `FROM scratch`
#     and `FROM <an earlier stage>` need no digest. An image given by a build argument
#     (`FROM ${BASE}`) is refused, because it cannot be checked here;
#   - a `FROM rust:<version>-...` tag equals the channel in rust-toolchain.toml, so the image and
#     the pinned toolchain are bumped together;
#   - the `CADDY_IMAGE=` default in deploy.sh carries a digest.
#
# The digests are refreshed by Dependabot (.github/dependabot.yml, `docker` ecosystem).
#
# Usage: scripts/check-image-pins.sh [FILE...]
#   With no FILE it checks Dockerfile and deploy.sh at the repository root. A FILE whose name ends
#   in `.sh` is checked for its CADDY_IMAGE default; any other FILE is read as a Dockerfile.
# Exit status: 0 all pinned, 1 at least one unpinned reference (each printed as file:line),
# 2 usage error or unreadable file.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
digest_re='@sha256:[0-9a-f]{64}$'
failures=0

fail() {
  printf '%s\n' "$*" >&2
  failures=$((failures + 1))
}

toolchain_channel() {
  local file="$root/rust-toolchain.toml" line
  [[ -f "$file" ]] || return 0
  while IFS= read -r line || [[ -n "$line" ]]; do
    line="${line%$'\r'}"
    if [[ "$line" =~ ^[[:space:]]*channel[[:space:]]*=[[:space:]]*\"([^\"]+)\" ]]; then
      printf '%s\n' "${BASH_REMATCH[1]}"
      return 0
    fi
  done <"$file"
}

check_dockerfile() {
  local file="$1" channel n=0 line rest image stage word lower
  local -a words
  local -A stages=()
  channel="$(toolchain_channel)"
  while IFS= read -r line || [[ -n "$line" ]]; do
    n=$((n + 1))
    line="${line%$'\r'}"
    [[ "$line" =~ ^[[:space:]]*[Ff][Rr][Oo][Mm][[:space:]]+(.*)$ ]] || continue
    rest="${BASH_REMATCH[1]}"
    read -r -a words <<<"$rest"
    image=""
    stage=""
    while ((${#words[@]})); do
      word="${words[0]}"
      words=("${words[@]:1}")
      if [[ -z "$image" && "$word" == --* ]]; then
        continue
      fi
      if [[ -z "$image" ]]; then
        image="$word"
      elif [[ "${word,,}" == "as" && ${#words[@]} -gt 0 ]]; then
        stage="${words[0],,}"
        break
      fi
    done
    lower="${image,,}"
    if [[ -z "$image" ]]; then
      fail "$file:$n: FROM without an image"
    elif [[ "$lower" == "scratch" || -n "${stages[$lower]:-}" ]]; then
      :
    elif [[ "$image" == *'$'* ]]; then
      fail "$file:$n: FROM $image: an image from a build argument cannot be checked; name it with a digest"
    elif ! [[ "$image" =~ $digest_re ]]; then
      fail "$file:$n: FROM $image has no @sha256: digest"
    fi
    if [[ -n "$channel" && "$image" =~ ^(docker\.io/(library/)?)?rust:([0-9]+\.[0-9]+(\.[0-9]+)?)- ]]; then
      if [[ "${BASH_REMATCH[3]}" != "$channel" ]]; then
        fail "$file:$n: FROM rust:${BASH_REMATCH[3]} differs from rust-toolchain.toml channel $channel"
      fi
    fi
    if [[ -n "$stage" ]]; then stages[$stage]=1; fi
  done <"$file"
}

check_script() {
  local file="$1" n=0 line found=0
  while IFS= read -r line || [[ -n "$line" ]]; do
    n=$((n + 1))
    line="${line%$'\r'}"
    [[ "$line" =~ ^[[:space:]]*CADDY_IMAGE=(.*)$ ]] || continue
    # `CADDY_IMAGE=$CADDY_IMAGE` (writing the state file) copies the value; it sets no default.
    [[ "${BASH_REMATCH[1]}" =~ ^\"?\$\{?[A-Za-z_][A-Za-z0-9_]*\}?\"?$ ]] && continue
    found=1
    if ! [[ "$line" =~ @sha256:[0-9a-f]{64} ]]; then
      fail "$file:$n: ${line#"${line%%[![:space:]]*}"} has no @sha256: digest"
    fi
  done <"$file"
  if ((found == 0)); then
    fail "$file: no CADDY_IMAGE= default found (renamed? update scripts/check-image-pins.sh)"
  fi
}

if (($# == 0)); then
  set -- "$root/Dockerfile" "$root/deploy.sh"
fi

for file in "$@"; do
  if [[ ! -r "$file" ]]; then
    printf 'check-image-pins: cannot read %s\n' "$file" >&2
    exit 2
  fi
  case "$file" in
    *.sh) check_script "$file" ;;
    *) check_dockerfile "$file" ;;
  esac
done

if ((failures > 0)); then
  printf 'check-image-pins: %d problem(s) with image pins\n' "$failures" >&2
  exit 1
fi
printf 'check-image-pins: all image references carry a digest (%s)\n' "$*"
