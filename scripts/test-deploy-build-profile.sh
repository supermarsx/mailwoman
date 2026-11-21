#!/usr/bin/env bash
# No Docker build. The optional tiny Cargo probe must use the orchestration slot.
set -Eeuo pipefail
ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
work=$(mktemp -d)
trap 'rm -rf -- "$work"' EXIT
fail() { printf 'FAIL: %s\n' "$*" >&2; exit 1; }

if [[ ${1:-} == --cargo-probe ]]; then
    mkdir -p "$work/probe/server/src" "$work/probe/dep/src"
    cat > "$work/probe/Cargo.toml" <<'TOML'
[workspace]
members = ["server", "dep"]
resolver = "3"
[profile.release]
overflow-checks = true
strip = true
panic = "unwind"
TOML
    cat > "$work/probe/dep/Cargo.toml" <<'TOML'
[package]
name = "fixture-dep"
version = "0.1.0"
edition = "2024"
TOML
    printf 'pub fn value() -> usize { std::hint::black_box(42) }\n' > "$work/probe/dep/src/lib.rs"
    cat > "$work/probe/server/Cargo.toml" <<'TOML'
[package]
name = "mw-server"
version = "0.1.0"
edition = "2024"
[dependencies]
fixture-dep = { path = "../dep" }
TOML
    cat > "$work/probe/server/src/main.rs" <<'RS'
fn main() {
    assert_eq!(fixture_dep::value(), 42);
    assert!(std::panic::catch_unwind(|| {
        let n = std::hint::black_box(u8::MAX);
        std::hint::black_box(n + 1)
    }).is_err());
    assert!(std::panic::catch_unwind(|| panic!("unwind remains enabled")).is_err());
}
RS
    (cd "$work/probe" && CARGO_TARGET_DIR="$work/probe/target" cargo build --offline --release -vv --config 'profile.release.package.mw-server.opt-level=2') > "$work/probe.log" 2>&1
    grep -- '--crate-name fixture_dep' "$work/probe.log" | grep -q -- '-C opt-level=3' || fail 'dependency was not optimized at level 3'
    grep -- '--crate-name mw_server' "$work/probe.log" | grep -q -- '-C opt-level=2' || fail 'server override did not use opt-level 2'
    grep -- '--crate-name mw_server' "$work/probe.log" | grep -q -- '-C overflow-checks=on' || fail 'release overflow checking was lost'
    "$work/probe/target/release/mw-server" > "$work/runtime.log" 2>&1 || fail 'overflow/unwind runtime invariants failed'
    printf 'PASS: actual Cargo server opt-level 2, dependency opt-level 3, overflow checks and unwind\n'
    exit 0
fi
[[ $# == 0 ]] || fail 'unknown fixture option'

# Exercise actual parsed command lines, stopping before deployment effects.
mkdir -p "$work/parser/scripts"
cp "$ROOT/scripts/deploy-stack.sh" "$work/parser/scripts/"
awk '{ print } $0 == "update_checkout" { print "printf '\''profile=%s\\n'\'' \"$BUILD_PROFILE\"; exit 0" }' "$ROOT/deploy.sh" > "$work/parser/deploy.sh"
for invalid in bogus RELEASE 'fast;exit 0' ''; do
    if bash "$work/parser/deploy.sh" deploy --no-pull --build-profile "$invalid" > "$work/invalid.log" 2>&1; then fail 'invalid profile accepted'; fi
done
if bash "$work/parser/deploy.sh" deploy --build-profile > "$work/missing.log" 2>&1; then fail 'missing profile value accepted'; fi
bash "$work/parser/deploy.sh" deploy --no-pull > "$work/default.log" 2>&1
grep -qx 'profile=release' "$work/default.log" || fail 'release default changed'
grep -qx 'BUILD_JOBS=8' "$ROOT/deploy.sh" || fail 'deployment job default changed'
[[ $(grep -c '^ARG MW_BUILD_JOBS=8$' "$ROOT/Dockerfile") == 2 ]] || fail 'GNU/musl job defaults differ'
[[ $(grep -Fc 'export CARGO_BUILD_JOBS="${MW_BUILD_JOBS:-8}"' "$ROOT/Dockerfile") == 2 ]] || fail 'GNU/musl empty job fallbacks differ'
bash "$work/parser/deploy.sh" deploy --no-pull --build-profile fast > "$work/fast.log" 2>&1
grep -qx 'profile=fast' "$work/fast.log" || fail 'fast CLI selection lost'

# Execute the real Docker invocation through a stub, preserving argument edges.
# shellcheck source=/dev/null
source <(awk '/^build_local_image\(\)/{copy=1} /^initialize\(\)/{copy=0} copy{print}' "$ROOT/deploy.sh")
docker() { printf '%s\n' "$@" > "$work/docker.args"; }
log() { printf '%s\n' "$*" >> "$work/build.log"; }
BUILD_INCREMENTAL=0 BUILD_JOBS=8 IMAGE=fixture:build ROOT_CONTEXT=$ROOT
ROOT="$work/context with spaces"
for BUILD_PROFILE in release fast; do
    build_local_image
    grep -qx "MW_BUILD_PROFILE=$BUILD_PROFILE" "$work/docker.args" || fail 'profile Docker argument lost'
    grep -qx 'MW_BUILD_JOBS=8' "$work/docker.args" || fail 'eight-job default lost'
    [[ $(tail -n 1 "$work/docker.args") == "$ROOT" ]] || fail 'Docker context was split'
done
ROOT=$ROOT_CONTEXT
grep -q 'runtime differences require measurement' "$work/build.log" || fail 'fast profile tradeoff not logged'
[[ $(grep -c '^ARG MW_BUILD_PROFILE=release$' "$ROOT/Dockerfile") == 2 ]] || fail 'GNU/musl release defaults differ'
[[ $(grep -c "fast) set -- --config 'profile.release.package.mw-server.opt-level=2'" "$ROOT/Dockerfile") == 2 ]] || fail 'GNU/musl package overrides differ'
[[ $(grep -c '&& mirror_cargo cargo build --locked --release "\$@"' "$ROOT/Dockerfile") == 2 ]] || fail 'Cargo arguments are not forwarded through the paired mirror safely'
[[ $(grep -c '^RUN --mount=type=bind,from=rust-source,source=/src,target=/mw-input,ro ' "$ROOT/Dockerfile") == 2 ]] || fail 'GNU/musl inputs are not immutable source mounts'
[[ $(grep -c 'target=/mw-build-cache,sharing=locked' "$ROOT/Dockerfile") == 2 ]] || fail 'GNU/musl source and target must share one locked cache root'
[[ $(grep -c '^COPY apps/web/dist/.gitkeep /src/apps/web/dist/.gitkeep$' "$ROOT/Dockerfile") == 1 ]] || fail 'placeholder must be part of the incoming snapshot'
! grep -q 'touch -t 200001010000' "$ROOT/Dockerfile" || fail 'constant source timestamp workaround returned'
grep -qx 'RUN mkdir -p /mw-build-cache && ln -s /src /mw-build-cache/source' "$ROOT/scripts/perf/Dockerfile" || fail 'fixture runtime source-root compatibility lost'
grep -q 'fixture_runtime::internal_node_driver_uses_runtime_source_root_after_build_mount_release --exact' "$ROOT/scripts/perf/Dockerfile" || fail 'actual internal driver runtime gate missing'
# Execute each actual Docker stage's shell selection, without invoking Cargo.
awk -v directory="$work" '/&& case "\$MW_BUILD_PROFILE" in/ {part++;copy=1} copy {line=$0;sub(/^[[:space:]]*&& /,"",line);sub(/[[:space:]]*\\$/,"",line);print line > (directory "/stage-" part ".sh")} copy && /esac/ {copy=0}' "$ROOT/Dockerfile"
for stage in 1 2; do
    printf '\nprintf "%%s\\n" "$@"\n' >> "$work/stage-$stage.sh"
    MW_BUILD_PROFILE=fast sh "$work/stage-$stage.sh" > "$work/stage.args"
    [[ $(sed -n '1p' "$work/stage.args") == --config && $(sed -n '2p' "$work/stage.args") == profile.release.package.mw-server.opt-level=2 ]] || fail 'actual Docker fast selection differs'
    MW_BUILD_PROFILE=release sh "$work/stage-$stage.sh" > "$work/stage.args"
    [[ ! -s $work/stage.args || $(cat "$work/stage.args") == '' ]] || fail 'release unexpectedly adds Cargo overrides'
    if MW_BUILD_PROFILE=invalid sh "$work/stage-$stage.sh" > "$work/stage.args" 2>&1; then fail 'actual Docker stage accepted invalid profile'; fi
done

# Actual Git fast-forward/re-exec must retain the caller's selected profile.
export GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null GIT_TERMINAL_PROMPT=0
export GIT_AUTHOR_NAME=DeployTest GIT_AUTHOR_EMAIL=deploy@example.test
export GIT_COMMITTER_NAME=$GIT_AUTHOR_NAME GIT_COMMITTER_EMAIL=$GIT_AUTHOR_EMAIL
unset MW_DEPLOY_GIT_SYNCED
git init -q --bare "$work/remote.git"
git init -q -b main "$work/author"
mkdir -p "$work/author/scripts"
cp "$work/parser/deploy.sh" "$work/author/deploy.sh"
cp "$ROOT/scripts/deploy-stack.sh" "$work/author/scripts/"
git -C "$work/author" add .
git -C "$work/author" commit -qm initial
git -C "$work/author" remote add origin "$work/remote.git"
git -C "$work/author" push -qu origin main
git --git-dir="$work/remote.git" symbolic-ref HEAD refs/heads/main
git clone -q "$work/remote.git" "$work/client"
printf '\n# changed upstream script\n' >> "$work/author/deploy.sh"
git -C "$work/author" commit -qam updated
git -C "$work/author" push -q
bash "$work/client/deploy.sh" deploy --build-profile fast > "$work/reexec.log" 2>&1
grep -qx 'profile=fast' "$work/reexec.log" || fail 'Git re-exec discarded fast profile'
[[ $(git -C "$work/client" rev-parse HEAD) == "$(git -C "$work/author" rev-parse HEAD)" ]] || fail 'Git update did not occur'
printf 'PASS: CLI defaults/rejections, actual Docker argument edges, GNU/musl contracts and Git re-exec\n'
