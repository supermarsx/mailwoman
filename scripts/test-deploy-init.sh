#!/usr/bin/env bash
# Fast bootstrap identity checks; starts no server and requires no Docker daemon.
set -euo pipefail
ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
work=$(mktemp -d "${TMPDIR:-/tmp}/mw-deploy-init.XXXXXX")
trap 'rm -rf -- "$work"' EXIT
fail() { printf '%s\n' "$*" >&2; exit 1; }
init() { bash "$ROOT/deploy.sh" init --no-pull --state-dir "$1" --port 18180; }
value() { awk -F= -v key="$2" '$1 == key {print $2}' "$1/app.env"; }

init "$work/first" > "$work/first.log" 2>&1
username=$(value "$work/first" MW_ADMIN_USER)
password=$(value "$work/first" MW_ADMIN_PASSWORD)
[[ $username =~ ^admin-[0-9a-f]{12}$ ]] || fail 'username lacks the readable secure suffix'
[[ $password =~ ^[0-9a-f]{48}$ ]] || fail 'password generation changed'
[[ $(stat -c %a "$work/first/app.env") == 600 ]] || fail 'credentials file is not private'
grep -F "Admin username: $username" "$work/first.log" >/dev/null || fail 'operator was not told the username'
if grep -F "$password" "$work/first.log" >/dev/null; then fail 'password leaked into setup output'; fi
cp "$work/first/app.env" "$work/original.env"
init "$work/first" > "$work/repeat.log" 2>&1
cmp -s "$work/original.env" "$work/first/app.env" || fail 'repeated init changed existing credentials'

init "$work/second" > "$work/second.log" 2>&1
[[ $(value "$work/second" MW_ADMIN_USER) != "$username" ]] || fail 'new installs share a username'
sed -i 's/^MW_ADMIN_USER=.*/MW_ADMIN_USER=existing-admin/' "$work/first/app.env"
cp "$work/first/app.env" "$work/explicit.env"
init "$work/first" > "$work/existing.log" 2>&1
cmp -s "$work/explicit.env" "$work/first/app.env" || fail 'explicit existing username was renamed'
printf 'PASS: new random identity, private credentials, safe guidance, stable repeat init, independent installs, existing identity preserved\n'
