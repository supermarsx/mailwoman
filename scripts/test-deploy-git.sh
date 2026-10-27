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
printf 'local defaults\n' > "$AUTHOR/local-settings"
printf 'reached\n*.local/\n' > "$AUTHOR/.gitignore"
git -C "$AUTHOR" add .
git -C "$AUTHOR" commit -qm initial
git -C "$AUTHOR" remote add origin "$FIXTURE/remote.git"
git -C "$AUTHOR" push -qu origin main
git --git-dir="$FIXTURE/remote.git" symbolic-ref HEAD refs/heads/main
clone() { git clone -q "$FIXTURE/remote.git" "$FIXTURE/$1"; }
for name in clean dirty staged dirty-unrelated staged-unrelated diverged detached untracked collision dry offline; do clone "$name"; done
printf 'new\n' > "$AUTHOR/version"
printf 'incoming\n' > "$AUTHOR/conflict"
# A changed script must be reloaded, not continue executing old parsed code.
# shellcheck disable=SC2016 # the sed pattern matches a literal $ROOT in the fixture script
sed -i 's|> "$ROOT/reached"|> "$ROOT/reached-new"|' "$AUTHOR/deploy.sh"
printf 'reached-new\n' >> "$AUTHOR/.gitignore"
git -C "$AUTHOR" add .
git -C "$AUTHOR" commit -qm update
git -C "$AUTHOR" push -q
LATEST=$(git -C "$AUTHOR" rev-parse HEAD)
clone current-dirty
run() { bash "$FIXTURE/$1/deploy.sh" "${@:2}" > "$FIXTURE/$1.log" 2>&1; }
reject() {
    local checkout=$1; shift
    rm -f -- "$FIXTURE/$checkout/reached" "$FIXTURE/$checkout/reached-new"
    if run "$checkout" "$@"; then fail "$checkout unexpectedly succeeded"; fi
    [[ ! -e $FIXTURE/$checkout/reached && ! -e $FIXTURE/$checkout/reached-new ]] || fail "$checkout crossed update boundary"
    [[ ! -d $FIXTURE/$checkout/.deploy.local ]] || fail "$checkout created deployment state"
}
run clean --state-dir "$FIXTURE/state with spaces" --build-jobs 4 --no-incremental
[[ $(git -C "$FIXTURE/clean" rev-parse HEAD) == "$LATEST" ]] || fail 'did not pull'
[[ $(cat "$FIXTURE/clean/reached-new") == "$FIXTURE/state with spaces" ]] || fail 'did not re-exec with original arguments'
[[ $(grep -c 'Mailwoman deployment' "$FIXTURE/clean.log") == 1 ]] || fail 're-exec duplicated deployment banner'
[[ $(grep -c '\[1/6\]' "$FIXTURE/clean.log") == 1 ]] || fail 're-exec duplicated source step'
grep -Eq 'main: [0-9a-f]{12} -> [0-9a-f]{12} \(1 incoming commit\(s\)\)' "$FIXTURE/clean.log" || fail 'missing compact revision summary'
grep -Eq '  [0-9a-f]+ update$' "$FIXTURE/clean.log" || fail 'missing incoming subject'
if grep -q $'\033' "$FIXTURE/clean.log"; then fail 'redirected output contains terminal escapes'; fi
printf 'local change\n' > "$FIXTURE/dirty-unrelated/local-settings"
git -C "$FIXTURE/dirty-unrelated" config pull.rebase true
run dirty-unrelated
[[ $(git -C "$FIXTURE/dirty-unrelated" rev-parse HEAD) == "$LATEST" ]] || fail 'unrelated dirty checkout did not pull'
[[ $(cat "$FIXTURE/dirty-unrelated/local-settings") == 'local change' ]] || fail 'unrelated dirty work lost'
printf 'staged change\n' > "$FIXTURE/staged-unrelated/local-settings"
git -C "$FIXTURE/staged-unrelated" add local-settings
run staged-unrelated
[[ $(git -C "$FIXTURE/staged-unrelated" rev-parse HEAD) == "$LATEST" ]] || fail 'unrelated staged checkout did not pull'
[[ $(git -C "$FIXTURE/staged-unrelated" show :local-settings) == 'staged change' ]] || fail 'staged work lost'
printf 'local\n' >> "$FIXTURE/current-dirty/version"
run current-dirty
grep -q local "$FIXTURE/current-dirty/version" || fail 'up-to-date dirty work lost'
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
# A complete batch of hostile/long subjects remains data, never terminal control
# or a printf format. Use a fresh clone so precisely seven commits are incoming.
clone subjects
for n in {1..7}; do
    printf '%s\n' "$n" >> "$AUTHOR/version"
    git -C "$AUTHOR" add version
    git -C "$AUTHOR" commit -qm "$(printf 'subject-%s %%s %%n \033[31m%s' "$n" "$(printf '%0250d' 0)")"
done
git -C "$AUTHOR" push -q
run subjects
grep -q '(7 incoming commit(s))' "$FIXTURE/subjects.log" || fail 'incoming count missing'
[[ $(grep -c 'subject-' "$FIXTURE/subjects.log") == 7 ]] || fail 'incoming subjects were omitted'
if grep -q 'more commit(s)' "$FIXTURE/subjects.log"; then fail 'commit list truncated'; fi
grep -q '%s %n' "$FIXTURE/subjects.log" || fail 'subject interpreted as printf format'
if grep -q $'\033' "$FIXTURE/subjects.log"; then fail 'Git metadata injected terminal escape'; fi
grep -Eq 'subject-7.*0{250}' "$FIXTURE/subjects.log" || fail 'long subject truncated'
mkdir "$FIXTURE/archive"
cp "$ROOT/deploy.sh" "$FIXTURE/archive/deploy.sh"
bash "$FIXTURE/archive/deploy.sh" --no-pull --dry-run > "$FIXTURE/plain.out" 2> "$FIXTURE/plain.log"
[[ ! -s $FIXTURE/plain.out ]] || fail 'progress changed stdout'
grep -q 'Source archive:' "$FIXTURE/plain.log" || fail 'archive source information missing'
grep -Eq 'Started: [0-9]{4}-[0-9]{2}-[0-9]{2} [0-9:]+ UTC' "$FIXTURE/plain.log" || fail 'start time missing'
grep -q '\[2/6\] Validate configuration' "$FIXTURE/plain.log" || fail 'phase numbering did not advance'
[[ ! -e $FIXTURE/archive/.deploy.local ]] || fail 'dry-run wrote state'
for jobs in 0 -1 abc 1000; do
    if bash "$FIXTURE/archive/deploy.sh" --no-pull --dry-run --build-jobs "$jobs" > "$FIXTURE/jobs.log" 2>&1; then
        fail "invalid build job count accepted: $jobs"
    fi
done
bash "$FIXTURE/archive/deploy.sh" --no-pull --dry-run --build-jobs 4 --no-incremental > "$FIXTURE/jobs.log" 2>&1

if grep -q $'\033' "$FIXTURE/plain.log"; then fail 'redirected progress has color'; fi
# Exercise the real presentation helpers without contacting Docker or sleeping.
awk '/^LOG_ACCENT=/{copy=1} /^need_value\(\)/{copy=0} copy' "$ROOT/deploy.sh" > "$FIXTURE/progress.sh"
(
    unset MW_DEPLOY_GIT_SYNCED
    # ACTION and PROGRESS_STARTED are read by the sourced deploy.sh helpers.
    # shellcheck disable=SC2034
    ACTION=deploy
    # shellcheck source=/dev/null # generated above from deploy.sh
    source "$FIXTURE/progress.sh"
    progress_start
    for label in configuration image backup health finalize; do phase "$label"; done
    # shellcheck disable=SC2034
    PROGRESS_STARTED=$(($(date +%s) - 3661))
    progress_finish
) > "$FIXTURE/timing.out" 2> "$FIXTURE/timing.log"
[[ ! -s $FIXTURE/timing.out ]] || fail 'timing changed stdout'
grep -Eq 'Finished: [0-9]{4}-[0-9]{2}-[0-9]{2} [0-9:]+ UTC' "$FIXTURE/timing.log" || fail 'finish time missing'
grep -Eq 'Deployment complete in 366[12]s \(1h 1m [12]s\)' "$FIXTURE/timing.log" || fail 'elapsed duration missing'
grep -q 'Step 6 finished in' "$FIXTURE/timing.log" || fail 'final phase duration missing'
# util-linux script provides a real pseudo-terminal; Git Bash lacks it, so the
# same suite is also run in a Linux helper for these terminal-only assertions.
if command -v script >/dev/null 2>&1 && script --version >/dev/null 2>&1; then
    env -u NO_COLOR TERM=xterm script -q -e -c "bash '$FIXTURE/archive/deploy.sh' --no-pull --dry-run" "$FIXTURE/tty.log" >/dev/null
    grep -q $'\033\[36m' "$FIXTURE/tty.log" || fail 'interactive progress has no accent'
    grep -q $'\033\[34m' "$FIXTURE/tty.log" || fail 'configuration phase has no distinct color'
    NO_COLOR=1 TERM=xterm script -q -e -c "bash '$FIXTURE/archive/deploy.sh' --no-pull --dry-run" "$FIXTURE/no-color.log" >/dev/null
    if grep -q $'\033' "$FIXTURE/no-color.log"; then fail 'NO_COLOR ignored'; fi
    TERM=dumb script -q -e -c "bash '$FIXTURE/archive/deploy.sh' --no-pull --dry-run" "$FIXTURE/dumb.log" >/dev/null
    if grep -q $'\033' "$FIXTURE/dumb.log"; then fail 'TERM=dumb ignored'; fi
    printf 'PASS: real TTY accent, NO_COLOR, TERM=dumb, redirected stderr, untouched stdout, source archive\n'
fi
printf 'PASS: fast-forward/re-exec, unrelated dirty/staged edits, dirty up-to-date checkout, configured rebase override, overlapping edits protected, divergence, detached HEAD, untracked preservation/collision, dry-run, failed fetch, missing upstream, opt-out and status\n'
