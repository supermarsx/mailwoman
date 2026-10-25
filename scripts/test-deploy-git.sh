#!/usr/bin/env bash
# Real local Git fixtures; stop after the update boundary, before Docker/state.
set -Eeuo pipefail
ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
FIXTURE=$(mktemp -d)
trap 'rm -rf -- "$FIXTURE"' EXIT
export GIT_CONFIG_NOSYSTEM=1
export GIT_CONFIG_GLOBAL=/dev/null
export GIT_AUTHOR_NAME=DeployTest GIT_AUTHOR_EMAIL=deploy@example.test
export GIT_COMMITTER_NAME=$GIT_AUTHOR_NAME GIT_COMMITTER_EMAIL=$GIT_AUTHOR_EMAIL
export GIT_TERMINAL_PROMPT=0
unset MW_DEPLOY_GIT_SYNCED
fail() { printf 'FAIL: %s\n' "$*" >&2; exit 1; }
git init -q --bare "$FIXTURE/remote.git"
git init -q -b main "$FIXTURE/author"
AUTHOR="$FIXTURE/author"
awk '{ print } $0 == "update_checkout" { print "printf '\''%s\\n'\'' \"$STATE\" > \"$ROOT/reached\"; exit 0" }' "$ROOT/deploy.sh" > "$AUTHOR/deploy.sh"
printf 'old\n' > "$AUTHOR/version"
printf 'reached\n*.local/\n' > "$AUTHOR/.gitignore"
git -C "$AUTHOR" add .
git -C "$AUTHOR" commit -qm initial
git -C "$AUTHOR" remote add origin "$FIXTURE/remote.git"
git -C "$AUTHOR" push -qu origin main
git --git-dir="$FIXTURE/remote.git" symbolic-ref HEAD refs/heads/main
clone() { git clone -q "$FIXTURE/remote.git" "$FIXTURE/$1"; }
for name in clean dirty staged diverged detached untracked collision dry offline; do clone "$name"; done
printf 'new\n' > "$AUTHOR/version"
printf 'incoming\n' > "$AUTHOR/conflict"
# A changed script must be reloaded, not continue executing old parsed code.
sed -i 's|> "$ROOT/reached"|> "$ROOT/reached-new"|' "$AUTHOR/deploy.sh"
printf 'reached-new\n' >> "$AUTHOR/.gitignore"
git -C "$AUTHOR" add .
git -C "$AUTHOR" commit -qm update
git -C "$AUTHOR" push -q
LATEST=$(git -C "$AUTHOR" rev-parse HEAD)
run() { bash "$FIXTURE/$1/deploy.sh" "${@:2}" > "$FIXTURE/$1.log" 2>&1; }
reject() {
    local checkout=$1; shift
    rm -f -- "$FIXTURE/$checkout/reached" "$FIXTURE/$checkout/reached-new"
    if run "$checkout" "$@"; then fail "$checkout unexpectedly succeeded"; fi
    [[ ! -e $FIXTURE/$checkout/reached && ! -e $FIXTURE/$checkout/reached-new ]] || fail "$checkout crossed update boundary"
    [[ ! -d $FIXTURE/$checkout/.deploy.local ]] || fail "$checkout created deployment state"
}
run clean --state-dir "$FIXTURE/state with spaces"
[[ $(git -C "$FIXTURE/clean" rev-parse HEAD) == "$LATEST" ]] || fail 'did not pull'
[[ $(cat "$FIXTURE/clean/reached-new") == "$FIXTURE/state with spaces" ]] || fail 'did not re-exec with original arguments'
printf 'local\n' >> "$FIXTURE/dirty/version"
reject dirty
grep -q local "$FIXTURE/dirty/version" || fail 'dirty work lost'
printf 'staged\n' >> "$FIXTURE/staged/version"
git -C "$FIXTURE/staged" add version
reject staged
printf 'local\n' > "$FIXTURE/diverged/local"
git -C "$FIXTURE/diverged" add local
git -C "$FIXTURE/diverged" commit -qm divergence
BEFORE=$(git -C "$FIXTURE/diverged" rev-parse HEAD)
reject diverged
[[ $(git -C "$FIXTURE/diverged" rev-parse HEAD) == "$BEFORE" ]] || fail 'divergence changed HEAD'
git -C "$FIXTURE/detached" checkout -q --detach
reject detached
printf 'keep\n' > "$FIXTURE/untracked/notes"
run untracked
[[ $(cat "$FIXTURE/untracked/notes") == keep ]] || fail 'untracked work lost'
printf 'keep\n' > "$FIXTURE/collision/conflict"
reject collision
[[ $(cat "$FIXTURE/collision/conflict") == keep ]] || fail 'untracked collision overwritten'
BEFORE=$(git -C "$FIXTURE/dry" rev-parse HEAD)
run dry --dry-run
[[ $(git -C "$FIXTURE/dry" rev-parse HEAD) == "$BEFORE" ]] || fail 'dry-run pulled'
run dirty --no-pull
git -C "$FIXTURE/offline" remote set-url origin "$FIXTURE/missing.git"
reject offline
run offline --no-pull
git -C "$FIXTURE/offline" branch --unset-upstream
reject offline
run detached --no-pull
run staged status
printf 'PASS: fast-forward, refreshed script/arguments, dirty and staged work, divergence, detached HEAD, untracked preservation/collision, dry-run, failed fetch, missing upstream, opt-out and status\n'
