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
grep -qx 'MEMORY_LIMIT=4g' "$work/first/deploy.env" || fail 'new installation lacks the 4 GiB memory default'
grep -qx 'CPU_LIMIT=4' "$work/first/deploy.env" || fail 'new installation lacks the four CPU default'
username=$(value "$work/first" MW_ADMIN_USER)
password=$(value "$work/first" MW_ADMIN_PASSWORD)
[[ $username =~ ^admin-[0-9a-f]{12}$ ]] || fail 'username lacks the readable secure suffix'
[[ $password =~ ^[0-9a-f]{48}$ ]] || fail 'password generation changed'
[[ $(stat -c %a "$work/first/app.env") == 600 ]] || fail 'credentials file is not private'
grep -F "Admin username: $username" "$work/first.log" >/dev/null || fail 'operator was not told the username'
if grep -F "$password" "$work/first.log" >/dev/null; then fail 'password leaked into setup output'; fi
cp "$work/first/app.env" "$work/original.env"
# Saved overrides remain authoritative, even below the new-install defaults.
sed -i 's/^MEMORY_LIMIT=.*/MEMORY_LIMIT=768m/;s/^CPU_LIMIT=.*/CPU_LIMIT=1.5/' "$work/first/deploy.env"
cp "$work/first/deploy.env" "$work/resources-original.env"
init "$work/first" > "$work/repeat.log" 2>&1
cmp -s "$work/original.env" "$work/first/app.env" || fail 'repeated init changed existing credentials'
cmp -s "$work/resources-original.env" "$work/first/deploy.env" || fail 'repeated init changed saved resource overrides'

init "$work/second" > "$work/second.log" 2>&1
[[ $(value "$work/second" MW_ADMIN_USER) != "$username" ]] || fail 'new installs share a username'
sed -i 's/^MW_ADMIN_USER=.*/MW_ADMIN_USER=existing-admin/' "$work/first/app.env"
cp "$work/first/app.env" "$work/explicit.env"
init "$work/first" > "$work/existing.log" 2>&1
cmp -s "$work/explicit.env" "$work/first/app.env" || fail 'explicit existing username was renamed'
printf 'PASS: new random identity, private credentials, safe guidance, stable repeat init, independent installs, existing identity preserved\n'

# Full-stack bootstrap keeps database/cache credentials stable and private too.
bash "$ROOT/deploy.sh" init --full-stack --no-pull --state-dir "$work/full" --port 18183 > "$work/full.log" 2>&1
grep -qx 'MEMORY_LIMIT=4g' "$work/full/deploy.env" || fail 'full-stack app lacks the 4 GiB memory default'
grep -qx 'CPU_LIMIT=4' "$work/full/deploy.env" || fail 'full-stack app lacks the four CPU default'
grep -qx 'STACK=full' "$work/full/deploy.env" || fail 'full stack was not saved'
for key in MW_STACK_DB_PASSWORD MW_STACK_CACHE_PASSWORD; do
    [[ $(value "$work/full" "$key") =~ ^[0-9a-f]{64}$ ]] || fail "invalid $key"
done
cp "$work/full/app.env" "$work/full-original.env"
bash "$ROOT/deploy.sh" init --full-stack --no-pull --state-dir "$work/full" --port 18183 > "$work/full-repeat.log" 2>&1
cmp "$work/full/app.env" "$work/full-original.env" || fail 'full-stack credentials changed on repeat init'
# A migration dry-run must neither change the source mode nor generate secrets.
cp "$work/first/deploy.env" "$work/single-original.env"
bash "$ROOT/deploy.sh" deploy --full-stack --dry-run --no-pull --state-dir "$work/first" > "$work/migration-dry.log" 2>&1
cmp "$work/first/deploy.env" "$work/single-original.env" || fail 'dry-run changed the SQLite configuration'
cmp "$work/first/app.env" "$work/explicit.env" || fail 'dry-run changed credentials'
printf 'PASS: full-stack bootstrap, stable private credentials, and migration dry-run without writes\n'

# Existing SQLite deployments can migrate behind either BYO proxy shape. No
# --domain, Caddy installation, public listener or Docker daemon is required.
for exposure in external network; do
    state=$work/byo-$exposure
    proxy_options=(--public-url https://mail.example.test)
    if [[ $exposure == network ]]; then proxy_options+=(--proxy-network existing-proxy); fi
    bash "$ROOT/deploy.sh" init --no-pull --state-dir "$state" --port 18184 "${proxy_options[@]}" > "$work/$exposure-init.log" 2>&1
    cp "$state/deploy.env" "$work/$exposure-deploy.env"
    cp "$state/app.env" "$work/$exposure-app.env"
    bash "$ROOT/deploy.sh" deploy --full-stack --dry-run --no-pull --state-dir "$state" "${proxy_options[@]}" > "$work/$exposure-migrate.log" 2>&1
    grep -F "stack=full; exposure=$exposure" "$work/$exposure-migrate.log" >/dev/null || fail "$exposure migration lost the proxy mode"
    grep -F 'URL: https://mail.example.test' "$work/$exposure-migrate.log" >/dev/null || fail "$exposure migration lost the public URL"
    grep -F 'Bring-your-own HTTPS proxy: no Caddy' "$work/$exposure-migrate.log" >/dev/null || fail "$exposure migration omitted the BYO proxy instructions"
    cmp "$state/deploy.env" "$work/$exposure-deploy.env" || fail "$exposure dry-run changed saved configuration"
    cmp "$state/app.env" "$work/$exposure-app.env" || fail "$exposure dry-run changed source credentials"
    grep -qx 'DOMAIN=' "$state/deploy.env" || fail "$exposure unexpectedly configured managed Caddy"
done
printf 'PASS: SQLite migration dry-runs retain host and Docker-network BYO proxy configuration without Caddy or source writes\n'
