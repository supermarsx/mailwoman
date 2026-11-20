#!/usr/bin/env bash
# Local parser/Git/Docker-stub fixtures. No registry, build, container or data access.
set -Eeuo pipefail
ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
work=$(mktemp -d)
trap 'rm -rf -- "$work"' EXIT
fail(){ printf 'FAIL: %s\n' "$*" >&2; exit 1; }
mkdir -p "$work/parser/scripts"
cp "$ROOT/scripts/deploy-stack.sh" "$work/parser/scripts/"
awk '{print} $0=="update_checkout" {print "printf '\''prebuilt=%s pull=%s\\n'\'' \"$PREBUILT\" \"$PULL\"; exit 0"}' "$ROOT/deploy.sh" > "$work/parser/deploy.sh"
reject(){ if bash "$work/parser/deploy.sh" "$@" > "$work/reject.log" 2>&1; then fail "conflicting prebuilt options accepted: $*"; fi; }
reject deploy --no-pull --prebuilt --image example:test
reject deploy --no-pull --prebuilt --image-tag test
reject deploy --no-pull --prebuilt --no-build
reject deploy --no-pull --prebuilt --build-profile fast
reject deploy --prebuilt --pull-only
reject maintenance --no-pull --prebuilt
reject init --no-pull --prebuilt
bash "$work/parser/deploy.sh" deploy --no-pull --prebuilt --pull --require-signature > "$work/parse.log" 2>&1
grep -qx 'prebuilt=true pull=true' "$work/parse.log" || fail 'prebuilt pull/signature parsing failed'
bash "$work/parser/deploy.sh" deploy --no-pull > "$work/default.log" 2>&1
grep -qx 'prebuilt=false pull=false' "$work/default.log" || fail 'local build default changed'
mkdir -p "$work/dry/scripts"
cp "$ROOT/deploy.sh" "$work/dry/deploy.sh"
cp "$ROOT/scripts/deploy-stack.sh" "$work/dry/scripts/"
bash "$work/dry/deploy.sh" deploy --prebuilt --dry-run --no-pull > "$work/dry.log" 2>&1
grep -q 'ghcr.io/supermarsx/mailwoman:master' "$work/dry.log" || fail 'dry run omitted published image reference'
grep -q 'No local compilation' "$work/dry.log" || fail 'dry run omitted prebuilt behavior'
[[ ! -d $work/dry/.deploy.local ]] || fail 'dry run created deployment state'

# Source exact late resolution and existing signature/pull/image-ID branches.
awk '/^if \$PREBUILT; then$/ {n++;if(n==2)copy=1} copy{print} copy && /^fi$/ {exit}' "$ROOT/deploy.sh" > "$work/select.sh"
awk '/^image_repository\(\)/ {copy=1} /^check_signature\(\)/ {copy=0} copy{print}' "$ROOT/deploy.sh" > "$work/repository.sh"
awk '/^image_digest\(\)/ {copy=1} /^load_active\(\)/ {copy=0} copy{print}' "$ROOT/deploy.sh" > "$work/digest.sh"
awk '/^        built=false$/ {copy=1} copy && /^        if \[\[ \$EXPOSURE == caddy/ {exit} copy{print}' "$ROOT/deploy.sh" > "$work/image.sh"
log(){ printf '%s\n' "$*" >> "$work/trace"; }
source "$work/repository.sh"
source "$work/digest.sh"
image_id=sha256:$(printf '1%.0s' {1..64})
digest=sha256:$(printf '2%.0s' {1..64})
docker(){
    printf 'docker' >> "$work/trace"; printf ' <%s>' "$@" >> "$work/trace"; printf '\n' >> "$work/trace"
    case "$1 $2" in
        'pull '*) [[ ${REGISTRY_FAIL:-false} == false ]] || return 9 ;;
        'image inspect')
            if [[ $* == *RepoDigests* ]]; then
                [[ ${*: -1} == "$image_id" ]] || fail 'digest lookup used mutable tag instead of selected ID'
                printf '%s@%s\n' "$repository" "$digest"
            else printf '%s\n' "$image_id"; fi ;;
        *) fail "unexpected Docker operation $*" ;;
    esac
}
check_signature(){
    [[ $1 == true ]] || fail 'prebuilt did not select registry verification path'
    log "signature $IMAGE"
    if [[ ${SIGNED:-true} == true ]]; then IMAGE="$repository@$digest"; fi
}
build_local_image(){ fail 'prebuilt attempted source build'; }
NO_BUILD=false PREBUILT=true PULL=true
for repository in ghcr.io/supermarsx/mailwoman registry.example.test:5000/custom/mailwoman; do
    if [[ $repository == ghcr.io/supermarsx/mailwoman ]]; then IMAGE_REPOSITORY=; else IMAGE_REPOSITORY=$repository; fi
    IMAGE= IMAGE_ID= IMAGE_DIGEST= SIGNED=true
    source "$work/select.sh"
    [[ $IMAGE == "$repository:master" ]] || fail 'prebuilt reference/default/override differs'
    source "$work/image.sh"
    [[ $IMAGE == "$repository@$digest" && $IMAGE_ID == "$image_id" && $IMAGE_DIGEST == "$repository@$digest" && $built == false ]] || fail 'verified pulled image was not pinned'
    grep -qF "docker <pull> <$repository@$digest>" "$work/trace" || fail 'verified immutable reference not pulled'
done
# Registry failures must stop before image selection, with no fallback build.
export -f docker log check_signature build_local_image image_digest image_repository fail
export work image_id digest
if REGISTRY_FAIL=true NO_BUILD=false PREBUILT=true PULL=true SIGNED=true bash -e -c '
    IMAGE= IMAGE_ID= IMAGE_DIGEST= IMAGE_REPOSITORY= repository=ghcr.io/supermarsx/mailwoman
    source "$work/select.sh"
    source "$work/image.sh"
    printf "UNEXPECTED CONTINUATION\n"
' > "$work/unavailable.log" 2>&1; then fail 'registry failure was ignored'; fi
! grep -q 'UNEXPECTED CONTINUATION' "$work/unavailable.log" || fail 'registry failure continued'

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
printf '\n# changed script\n' >> "$work/author/deploy.sh"
git -C "$work/author" commit -qam updated
git -C "$work/author" push -q
bash "$work/client/deploy.sh" deploy --prebuilt > "$work/reexec.log" 2>&1
grep -qx 'prebuilt=true pull=true' "$work/reexec.log" || fail 'Git re-exec discarded prebuilt mode'
[[ $(git -C "$work/client" rev-parse HEAD) == "$(git -C "$work/author" rev-parse HEAD)" ]] || fail 'script update did not occur'
printf 'PASS: prebuilt CLI/re-exec, canonical/configured master tags, verified digest pull, immutable ID metadata, registry failure and no local build\n'
