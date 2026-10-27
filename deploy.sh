#!/usr/bin/env bash
# Mailwoman single-host deployment and recovery. Run `bash deploy.sh --help`.
# Requires Bash 4+, Docker with Linux containers, Compose 2.30+, curl and tar.
# Configuration files are parsed as data, never sourced as shell scripts.
# Accept `sh deploy.sh` on hosts where sh is dash or another POSIX shell.
# Keep this bootstrap POSIX-compatible and before any Bash-only syntax.
if [ -z "${BASH_VERSION:-}" ]; then
    exec bash "$0" "$@"
fi
set -Eeuo pipefail
umask 077

ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
ORIGINAL_ARGS=("$@")
GIT_PULL=true
STATE="$ROOT/.deploy.local"
ACTION=deploy
ACTION_SET=false
PROJECT=mailwoman
EXPOSURE=local
DOMAIN=
PUBLIC_URL=
PORT=8080
PORT_SET=false
MODE=proxy
PROXY_PREFIX=172.30.70
PROXY_NETWORK=
PROXY_SUBNETS=
IMAGE_ID=
DATA_VOLUME=
# New installations name Caddy by exact version and multi-platform index digest;
# a deploy.env written earlier keeps the value it has.
CADDY_IMAGE=caddy:2.11.7-alpine@sha256:eb4d35b7452d1ab6171fbdbe8d9275b2301dad6dad92115d91e19fb9b24d510f
CADDY_ID=
CADDY_PROTOCOLS='h1 h2 h3'
ACME_CA=
ACME_CA_ROOT=
MEMORY_LIMIT=
CPU_LIMIT=
SIGNATURE_IDENTITY=
SIGNATURE_ISSUER=
UPSTREAM_DEFAULT=
IMAGE_REPOSITORY=
IMAGE_DIGEST=
IMAGE_TAG=
# The identity that signs the published image: .github/workflows/supply-chain.yml
# on a push to master, through GitHub's OIDC issuer. deploy.env may name another
# (SIGNATURE_IDENTITY / SIGNATURE_ISSUER) for a fork's images.
DEFAULT_SIGNATURE_IDENTITY=https://github.com/supermarsx/mailwoman/.github/workflows/supply-chain.yml@refs/heads/master
DEFAULT_SIGNATURE_ISSUER=https://token.actions.githubusercontent.com
IMAGE=
PULL=false
NO_BUILD=false
BUILD_JOBS=
BUILD_INCREMENTAL=1
AUTO_ROLLBACK=true
REQUIRE_SIGNATURE=false
DRY_RUN=false
TIMEOUT=180
SNAPSHOT=
INIT_OPTIONS=false
LOCKED=false
RESUME_CONTAINER=
HELPER_CONTAINER=
WORK=
LAST_BACKUP=
PRUNE_REPLACED=false
ASSUME_YES=false
BACKUP_KEEP=
BACKUP_KEEP_DAYS=
BACKUP_RECIPIENT=
BACKUP_HOOK=
BACKUP_TARGETS_FILE=
ALERT_HOOK=
IDENTITY=
EXTRACT=

usage() {
    cat <<'HELP'
Usage: bash deploy.sh [command] [options]

Commands (default: deploy):
  init              Generate private, editable configuration; do not start Docker
  deploy            First run: set up and start. Later runs: update in place
                    (build/pull, back up the existing install, start, verify)
  doctor            Check prerequisites, configuration, and Compose syntax
  status            Show containers, the configured public URL and whether
                    metrics are enabled
  check             Exit 0 when the app is healthy and answers at its URL;
                    otherwise run ALERT_HOOK and exit non-zero
  logs              Follow the last 100 lines of service logs (Ctrl-C to leave)
  stop              Stop services; retain all containers, volumes, and backups
  start             Start the last applied configuration and verify health
  restart           Restart the last applied configuration and verify health
  backup            Stop the app briefly; snapshot data, uploads, config and key
                    (one encrypted archive when BACKUP_RECIPIENT is set), then
                    publish to BACKUP_TARGETS_FILE destinations and run BACKUP_HOOK
  restore PATH      Restore a snapshot directory or encrypted archive into a NEW
                    volume, then verify health
  rollback          Restore the snapshot taken immediately before the last deploy
  prune             Remove old snapshots (BACKUP_KEEP / BACKUP_KEEP_DAYS in
                    deploy.env); list replaced data volumes and old built images
  timers            Write systemd service/timer units for backup and check into
                    the state directory and print them with equivalent cron lines

Options:
  --state-dir PATH   Deployment files/backups (default: <repo>/.deploy.local)
  --project NAME     Isolated Compose project name (init/first deploy only)
  --domain HOST     Publish through Caddy with automatic HTTPS (init only)
  --public-url URL  Existing HTTPS reverse proxy; bind app to localhost (init only)
  --proxy-network NAME
                    With --public-url: publish no host port; join Docker network
                    NAME (created if missing) so a proxy container on it, such as
                    Nginx Proxy Manager, reaches http://<project>:8080 (init only)
  --port NUMBER     Localhost HTTP port (init only; default: the first free
                    port from 8080 to 8099)
  --mode MODE       proxy for JMAP; engine for IMAP/POP3 + SMTP (init only)
  --http-protocols LIST  Caddy protocols: 'h1', 'h1 h2', or 'h1 h2 h3' (default)
  --proxy-prefix IP First 3 octets of a private /24 for Caddy, e.g. 172.30.71
                    (init only; default 172.30.70; choose an unused subnet)
  --image REF       Use a ready-built Mailwoman image instead of building source
  --pull            Refresh --image from its registry before deploying
  --image-tag TAG   Deploy IMAGE_REPOSITORY:TAG from deploy.env, fetched from its
                    registry (as --image IMAGE_REPOSITORY:TAG --pull)
  --require-signature
                    Refuse an --image whose signature cannot be checked (no
                    cosign, or used from the local image store without --pull)
  --no-build        Reuse the last applied app image (configuration-only deploy)
  --build-jobs N    Limit concurrent Cargo jobs (default: Cargo auto-detects CPUs)
  --no-incremental Disable cached incremental Rust compilation for this build
  --no-pull         Deploy the current checkout without updating Git
  --no-auto-rollback
                    Leave a failed update in place instead of restoring the
                    previous release and its data automatically
  --timeout SECS    Health/startup timeout, 10..3600 (default 180)
  --replaced        With prune: also delete the replaced data volumes and old
                    built images it lists (needs --yes)
  --yes             Confirm the deletion asked for with --replaced
  --identity FILE   With restore of a .tar.age archive: the age identity (private
                    key) to decrypt it with; .tar.gpg uses your gpg keyring
  --dry-run         Describe the operation without writing or contacting Docker
  -h, --help        Show this help

Examples:
  bash deploy.sh --domain mail.example.org --mode engine
  bash deploy.sh --public-url https://mail.example.org --proxy-network proxy
  bash deploy.sh init --public-url https://mail.example.org
  # Edit .deploy.local/app.env and deploy.env, then:
  bash deploy.sh deploy
  bash deploy.sh backup
  bash deploy.sh restore .deploy.local/backups/20261006T120000Z-1234

Running the same command again updates the installation; setup options may be
repeated unchanged or left out. No domain means a local HTTP installation at http://localhost:8080, or the
next free port; the chosen URL is printed and kept in deploy.env.
For public HTTPS, point DNS to this host and allow TCP ports 80 and 443.
HTTP/3 (QUIC) also needs UDP port 443; HTTP/2 and HTTP/1.1 remain available.
The script deploys the webmail app; mailbox/SMTP servers remain external.
Restore/rollback reverts data to the snapshot time (including configuration).
The replaced volume is retained. Backups contain secrets: store them privately.
See docs/deploy/script.md for operations, recovery, and optional integrations.
HELP
}

LOG_ACCENT='' LOG_RESET='' LOG_BOLD='' LOG_SUCCESS='' LOG_ERROR='' LOG_WARNING='' LOG_DETAIL=''
PHASE_COLORS=("" "" "" "" "" "")
if [[ -t 2 && ${TERM:-dumb} != dumb && ! ${NO_COLOR+x} ]]; then
    LOG_ACCENT=$'\033[36m'; LOG_RESET=$'\033[0m'
    LOG_BOLD=$'\033[1m'; LOG_SUCCESS=$'\033[32m'; LOG_ERROR=$'\033[31m'
    LOG_WARNING=$'\033[33m'; LOG_DETAIL=$'\033[36m'
    PHASE_COLORS=($'\033[36m' $'\033[34m' $'\033[35m' $'\033[33m' $'\033[32m' $'\033[96m')
fi
colored_log() { local color=$1; shift; printf '%s[deploy] %s%s\n' "$color" "$*" "$LOG_RESET" >&2; }
log() { printf '%s[deploy]%s %s\n' "$LOG_ACCENT" "$LOG_RESET" "$*" >&2; }
# Git subjects/ref names are data, not terminal commands. Bound displayed labels
# and remove control/non-ASCII bytes (including escape sequences and bidi marks).
git_subject() { LC_ALL=C tr -cd ' -~'; }
git_label() { LC_ALL=C tr -cd ' -~' | cut -c 1-180; }
git_diagnostic() { LC_ALL=C tr -cd '\12\40-\176' | awk 'NR <= 12 { print substr($0, 1, 180) }' >&2; }
PROGRESS_STEP=0 PROGRESS_STARTED=0 PHASE_STARTED=0
if [[ -n ${MW_DEPLOY_GIT_SYNCED:-} && ${MW_DEPLOY_PROGRESS_STEP:-} =~ ^[1-6]$ &&
      ${MW_DEPLOY_PROGRESS_STARTED:-} =~ ^[1-9][0-9]{0,10}$ && ${MW_DEPLOY_PHASE_STARTED:-} =~ ^[1-9][0-9]{0,10}$ ]]; then
    PROGRESS_STEP=$MW_DEPLOY_PROGRESS_STEP
    PROGRESS_STARTED=$MW_DEPLOY_PROGRESS_STARTED
    PHASE_STARTED=$MW_DEPLOY_PHASE_STARTED
fi
phase() {
    [[ $ACTION == deploy ]] || return 0
    local now elapsed
    now=$(date +%s)
    if ((PHASE_STARTED > 0)); then
        elapsed=$((now - PHASE_STARTED)); ((elapsed >= 0)) || elapsed=0
        colored_log "$LOG_SUCCESS" "  Step $PROGRESS_STEP finished in ${elapsed}s"
    fi
    PROGRESS_STEP=$((PROGRESS_STEP + 1)); PHASE_STARTED=$now
    printf '\n' >&2
    colored_log "$LOG_BOLD${PHASE_COLORS[$((PROGRESS_STEP - 1))]}" "[$PROGRESS_STEP/6] $*"
}
progress_start() {
    if ((PROGRESS_STEP == 0)); then
        PROGRESS_STARTED=$(date +%s)
        colored_log "$LOG_BOLD$LOG_ACCENT" "Mailwoman deployment"
        log "Started: $(date -u '+%Y-%m-%d %H:%M:%S UTC')"
        phase "Prepare source checkout"
    fi
}
progress_finish() {
    local now elapsed
    now=$(date +%s); elapsed=$((now - PROGRESS_STARTED)); ((elapsed >= 0)) || elapsed=0
    colored_log "$LOG_SUCCESS" "  Step $PROGRESS_STEP finished in $((now - PHASE_STARTED))s"
    log "Finished: $(date -u '+%Y-%m-%d %H:%M:%S UTC')"
    colored_log "$LOG_BOLD$LOG_SUCCESS" "Deployment complete in ${elapsed}s ($((elapsed / 3600))h $((elapsed % 3600 / 60))m $((elapsed % 60))s)"
}
die() { colored_log "$LOG_BOLD$LOG_ERROR" "ERROR: $*"; exit 1; }
need_value() { [[ $# -ge 2 && -n $2 && $2 != --* ]] || die "$1 needs a value"; }
need() { command -v "$1" >/dev/null 2>&1 || die "Required command missing: $1"; }
# True when something on this host accepts connections on the given TCP port.
port_in_use() { (exec 3<>"/dev/tcp/127.0.0.1/$1") 2>/dev/null; }

[[ ${BASH_VERSINFO[0]} -ge 4 ]] || die "Bash 4 or newer is required"
# Setup options as given on this command line, by deploy.env key.
declare -A REQUESTED=()
while (($#)); do
    case "$1" in
        init|deploy|doctor|status|check|logs|stop|start|restart|backup|rollback|restore|prune|timers)
            $ACTION_SET && die "Specify only one command"
            ACTION=$1; ACTION_SET=true
            if [[ $ACTION == restore ]]; then need_value "$@"; SNAPSHOT=$2; shift; fi ;;
        --state-dir) need_value "$@"; STATE=$2; shift ;;
        --project) need_value "$@"; PROJECT=$2; REQUESTED[PROJECT]=$2; INIT_OPTIONS=true; shift ;;
        --domain) need_value "$@"; DOMAIN=$2; REQUESTED[DOMAIN]=$2; INIT_OPTIONS=true; shift ;;
        --public-url) need_value "$@"; PUBLIC_URL=$2; REQUESTED[PUBLIC_URL]=$2; INIT_OPTIONS=true; shift ;;
        --port) need_value "$@"; PORT=$2; REQUESTED[PORT]=$2; PORT_SET=true; INIT_OPTIONS=true; shift ;;
        --proxy-network) need_value "$@"; PROXY_NETWORK=$2; REQUESTED[PROXY_NETWORK]=$2; INIT_OPTIONS=true; shift ;;
        --mode) need_value "$@"; MODE=$2; REQUESTED[MODE]=$2; INIT_OPTIONS=true; shift ;;
        --http-protocols) need_value "$@"; CADDY_PROTOCOLS=$2; REQUESTED[CADDY_PROTOCOLS]=$2; INIT_OPTIONS=true; shift ;;
        --proxy-prefix) need_value "$@"; PROXY_PREFIX=$2; REQUESTED[PROXY_PREFIX]=$2; INIT_OPTIONS=true; shift ;;
        --image) need_value "$@"; IMAGE=$2; shift ;;
        --image-tag) need_value "$@"; IMAGE_TAG=$2; shift ;;
        --timeout) need_value "$@"; TIMEOUT=$2; shift ;;
        --pull) PULL=true ;;
        --require-signature) REQUIRE_SIGNATURE=true ;;
        --no-build) NO_BUILD=true ;;
        --build-jobs) need_value "$@"; BUILD_JOBS=$2; shift ;;
        --no-incremental) BUILD_INCREMENTAL=0 ;;
        --no-pull) GIT_PULL=false ;;
        --no-auto-rollback) AUTO_ROLLBACK=false ;;
        --replaced) PRUNE_REPLACED=true ;;
        --yes) ASSUME_YES=true ;;
        --identity) need_value "$@"; IDENTITY=$2; shift ;;
        --dry-run) DRY_RUN=true ;;
        -h|--help) usage; exit 0 ;;
        *) die "Unknown argument: $1 (see --help)" ;;
    esac
    shift
done

if [[ -n $BUILD_JOBS ]]; then
    [[ $BUILD_JOBS =~ ^[1-9][0-9]{0,2}$ ]] || die "--build-jobs must be an integer from 1 to 999"
fi
if [[ ! $TIMEOUT =~ ^[0-9]{2,4}$ ]] || ((10#$TIMEOUT < 10 || 10#$TIMEOUT > 3600)); then die "Invalid timeout"; fi
TIMEOUT=$((10#$TIMEOUT))
[[ -z $IMAGE || $IMAGE != -* && $IMAGE != *[[:space:]]* ]] || die "Invalid image reference"
[[ -z $IMAGE_TAG || $IMAGE_TAG =~ ^[a-zA-Z0-9_][a-zA-Z0-9_.-]{0,127}$ ]] || die "Invalid image tag: $IMAGE_TAG"
if [[ -n $IMAGE && -n $IMAGE_TAG ]]; then die "Give --image or --image-tag, not both"; fi
if $NO_BUILD && { [[ -n $IMAGE$IMAGE_TAG ]] || $PULL; }; then die "--no-build cannot be combined with --image/--image-tag/--pull"; fi
if $PULL && [[ -z $IMAGE$IMAGE_TAG ]]; then die "--pull requires --image"; fi
if $REQUIRE_SIGNATURE && [[ -z $IMAGE$IMAGE_TAG ]]; then die "--require-signature requires --image or --image-tag"; fi
if [[ $ACTION != deploy ]] && { [[ -n $IMAGE$IMAGE_TAG ]] || $NO_BUILD || $PULL || ! $AUTO_ROLLBACK; }; then
    die "Image and rollback options apply only to deploy"
fi
if $INIT_OPTIONS && [[ $ACTION != init && $ACTION != deploy ]]; then die "Setup options apply only to init/first deploy"; fi
if { $PRUNE_REPLACED || $ASSUME_YES; } && [[ $ACTION != prune ]]; then die "--replaced and --yes apply only to prune"; fi
if $ASSUME_YES && ! $PRUNE_REPLACED; then die "--yes confirms --replaced; give both"; fi
if [[ -n $IDENTITY && $ACTION != restore ]]; then die "--identity applies only to restore"; fi

# Update before reading configuration, creating state or touching containers.
# Keep this in a function: Bash has parsed its whole body before pull can replace
# this script. Re-exec the updated script with exactly the original arguments.
update_checkout() {
    [[ $ACTION == deploy ]] || return 0
    progress_start
    if ! $GIT_PULL; then
        log "--no-pull: deploying the current checkout"
        if command -v git >/dev/null 2>&1 && git -C "$ROOT" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
            log "Source: $(git -C "$ROOT" rev-parse --short=12 HEAD | git_label) ($(git -C "$ROOT" symbolic-ref -q --short HEAD | git_label || printf 'detached HEAD'))"
        else
            log "Source archive: Git revision information is unavailable"
        fi
        return 0
    fi
    if $DRY_RUN; then
        log "Would update the current branch from its Git upstream (git pull --ff-only); no Git changes made"
        return 0
    fi
    need git
    git -C "$ROOT" rev-parse --is-inside-work-tree >/dev/null 2>&1 ||
        die "Deployment needs a Git checkout; use --no-pull for a source archive"
    local before upstream status branch after incoming subject pull_output
    before=$(git -C "$ROOT" rev-parse HEAD) || die "Cannot read Git HEAD"
    if [[ ${MW_DEPLOY_GIT_SYNCED:-} == "$before" ]]; then
        unset MW_DEPLOY_GIT_SYNCED
        unset MW_DEPLOY_PROGRESS_STEP MW_DEPLOY_PROGRESS_STARTED MW_DEPLOY_PHASE_STARTED
        return 0
    fi
    git -C "$ROOT" symbolic-ref -q HEAD >/dev/null ||
        die "Detached Git HEAD; check out a tracking branch or use --no-pull"
    upstream=$(git -C "$ROOT" rev-parse --abbrev-ref --symbolic-full-name '@{upstream}') ||
        die "This branch has no upstream; configure it or use --no-pull"
    branch=$(git -C "$ROOT" symbolic-ref --short HEAD)
    # Compare content, not cached stat metadata; ignore unrelated untracked files.
    # Git itself refuses an untracked file that an incoming commit would overwrite.
    status=0
    git -c core.safecrlf=false -C "$ROOT" diff --quiet || status=$?
    [[ $status == 0 || $status == 1 ]] || die "Cannot read the Git working tree"
    [[ $status == 0 ]] || colored_log "$LOG_WARNING" "Local tracked edits detected; Git will preserve them or refuse an overlapping update"
    status=0
    git -c core.safecrlf=false -C "$ROOT" diff --cached --quiet || status=$?
    [[ $status == 0 || $status == 1 ]] || die "Cannot read the Git index"
    [[ $status == 0 ]] || colored_log "$LOG_WARNING" "Staged edits detected; Git will preserve them or refuse an overlapping update"
    log "Updating $(printf '%s' "$branch" | git_label) from $(printf '%s' "$upstream" | git_label) (${before:0:12})"
    status=0
    pull_output=$(git -C "$ROOT" -c color.ui=false -c rebase.autoStash=false -c merge.autoStash=false pull --quiet --ff-only --no-rebase 2>&1) || status=$?
    [[ -z $pull_output ]] || printf '%s\n' "$pull_output" | git_diagnostic
    [[ $status == 0 ]] ||
        die "Git update failed; nothing was built or restarted. Reconcile the checkout or use --no-pull"
    export MW_DEPLOY_GIT_SYNCED
    MW_DEPLOY_GIT_SYNCED=$(git -C "$ROOT" rev-parse HEAD) || die "Cannot read updated Git HEAD"
    after=$MW_DEPLOY_GIT_SYNCED
    incoming=$(git -C "$ROOT" rev-list --count "$before..$after" --)
    log "$(printf '%s' "$branch" | git_label): ${before:0:12} -> ${after:0:12} ($incoming incoming commit(s))"
    while IFS= read -r subject; do
        [[ -z $subject ]] || colored_log "$LOG_DETAIL" "  $(printf '%s' "$subject" | git_subject)"
    done < <(git -C "$ROOT" -c log.showSignature=false log --no-decorate --format='%h %s' --reverse "$before..$after" --)
    export MW_DEPLOY_PROGRESS_STEP=$PROGRESS_STEP MW_DEPLOY_PROGRESS_STARTED=$PROGRESS_STARTED MW_DEPLOY_PHASE_STARTED=$PHASE_STARTED
    exec bash "$ROOT/deploy.sh" "${ORIGINAL_ARGS[@]}"
}
update_checkout
phase "Validate configuration and prerequisites"

read_config() {
    local file=$1 line key value seen='|'
    [[ -f $file && ! -L $file ]] || die "Missing or symlinked configuration: $file"
    # Settings a deploy.env may leave out (older files, or ones edited by hand) are
    # empty unless this file sets them, never left over from a file read earlier.
    if [[ ${file##*/} == deploy.env ]]; then
        PROXY_NETWORK=; ACME_CA=; ACME_CA_ROOT=; MEMORY_LIMIT=; CPU_LIMIT=; SIGNATURE_IDENTITY=; SIGNATURE_ISSUER=; UPSTREAM_DEFAULT=; IMAGE_REPOSITORY=
        BACKUP_KEEP=; BACKUP_KEEP_DAYS=; BACKUP_RECIPIENT=; BACKUP_HOOK=; BACKUP_TARGETS_FILE=; ALERT_HOOK=
    elif [[ ${file##*/} == release.env ]]; then
        # Written before the image digest was recorded: render falls back to IMAGE_ID.
        IMAGE_DIGEST=
    fi
    while IFS= read -r line || [[ -n $line ]]; do
        [[ -z $line || $line == \#* ]] && continue
        [[ $line != *$'\r'* && $line == *=* ]] || die "Invalid line in $file; use unquoted KEY=value with LF endings"
        key=${line%%=*}; value=${line#*=}
        [[ $seen != *"|$key|"* ]] || die "Duplicate $key in $file"
        seen+="$key|"
        case "$key" in
            PROJECT|EXPOSURE|DOMAIN|PUBLIC_URL|PORT|MODE|PROXY_PREFIX|PROXY_NETWORK|CADDY_IMAGE|CADDY_PROTOCOLS|ACME_CA|ACME_CA_ROOT|MEMORY_LIMIT|CPU_LIMIT|SIGNATURE_IDENTITY|SIGNATURE_ISSUER|UPSTREAM_DEFAULT|IMAGE_REPOSITORY|BACKUP_KEEP|BACKUP_KEEP_DAYS|BACKUP_RECIPIENT|BACKUP_HOOK|BACKUP_TARGETS_FILE|ALERT_HOOK|IMAGE_ID|IMAGE_DIGEST|DATA_VOLUME|CADDY_ID)
                printf -v "$key" '%s' "$value" ;;
            *) die "Unknown setting $key in $file" ;;
        esac
    done < "$file"
}

validate_config() {
    local a b c
    [[ $PROJECT =~ ^[a-z][a-z0-9_-]{0,39}$ ]] || die "Invalid project name"
    [[ $MODE == proxy || $MODE == engine ]] || die "Mode must be proxy or engine"
    if [[ ! $PORT =~ ^[0-9]{1,5}$ ]] || ((10#$PORT < 1 || 10#$PORT > 65535)); then die "Invalid port"; fi
    PORT=$((10#$PORT))
    [[ $PROXY_PREFIX =~ ^([0-9]{1,3})\.([0-9]{1,3})\.([0-9]{1,3})$ ]] || die "Invalid private network prefix"
    a=$((10#${BASH_REMATCH[1]})); b=$((10#${BASH_REMATCH[2]})); c=$((10#${BASH_REMATCH[3]}))
    ((b <= 255 && c <= 255 && (a == 10 || (a == 172 && b >= 16 && b <= 31) || (a == 192 && b == 168)))) || die "Proxy subnet must be a private /24"
    PROXY_PREFIX=$a.$b.$c
    [[ $CADDY_IMAGE =~ ^[a-zA-Z0-9][a-zA-Z0-9./_:@-]+$ ]] || die "Invalid Caddy image reference"
    case "$CADDY_PROTOCOLS" in 'h1'|'h1 h2'|'h1 h2 h3') ;; *) die "CADDY_PROTOCOLS must be h1, h1 h2, or h1 h2 h3" ;; esac
    case "$EXPOSURE" in
        local) [[ -z $DOMAIN && $PUBLIC_URL == "http://localhost:$PORT" ]] || die "Local URL must be http://localhost:$PORT" ;;
        caddy)
            [[ ${#DOMAIN} -le 253 && $DOMAIN == *.* ]] || die "Use a fully qualified domain name"
            local label
            local -a labels
            IFS=. read -r -a labels <<< "$DOMAIN"
            [[ $DOMAIN != *. ]] || die "Domain must not end in a dot"
            for label in "${labels[@]}"; do
                [[ $label =~ ^[a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?$ ]] || die "Invalid domain name"
            done
            [[ $PUBLIC_URL == "https://$DOMAIN" ]] || die "Caddy URL must match the domain" ;;
        external|network)
            [[ -z $DOMAIN && $PUBLIC_URL =~ ^https://[a-zA-Z0-9][a-zA-Z0-9.-]*(:[0-9]{1,5})?$ ]] || die "Public URL must be an HTTPS origin without a trailing slash/path"
            [[ $EXPOSURE == external || $PROXY_NETWORK =~ ^[a-zA-Z0-9][a-zA-Z0-9_.-]+$ ]] || die "Invalid proxy network name" ;;
        *) die "EXPOSURE must be local, caddy, external or network" ;;
    esac
    [[ $EXPOSURE == network || -z $PROXY_NETWORK ]] || die "PROXY_NETWORK applies only to EXPOSURE=network"
    [[ $EXPOSURE == caddy || -z $ACME_CA$ACME_CA_ROOT ]] || die "ACME_CA and ACME_CA_ROOT apply only to EXPOSURE=caddy"
    [[ -z $ACME_CA || $ACME_CA =~ ^https://[a-zA-Z0-9][a-zA-Z0-9.-]*(:[0-9]{1,5})?(/[a-zA-Z0-9._~/-]*)?$ ]] || die "ACME_CA must be the https:// URL of an ACME directory"
    [[ -z $ACME_CA_ROOT || $ACME_CA_ROOT =~ ^/[a-zA-Z0-9._/+@-]+$ ]] || die "ACME_CA_ROOT must be an absolute path to a PEM file (letters, digits and ._/+@- only)"
    [[ -z $MEMORY_LIMIT || $MEMORY_LIMIT =~ ^[1-9][0-9]{0,5}[mg]$ ]] || die "MEMORY_LIMIT must be a size such as 1g or 768m, or empty for no limit"
    [[ -z $CPU_LIMIT || ($CPU_LIMIT =~ ^[0-9]{1,3}(\.[0-9]{1,2})?$ && $CPU_LIMIT =~ [1-9]) ]] || die "CPU_LIMIT must be a number of CPUs such as 2 or 1.5, or empty for no limit"
    [[ -z $SIGNATURE_IDENTITY || $SIGNATURE_IDENTITY =~ ^[a-zA-Z0-9][a-zA-Z0-9._~:/@%+=-]*$ ]] || die "SIGNATURE_IDENTITY must be a certificate identity such as $DEFAULT_SIGNATURE_IDENTITY"
    [[ -z $SIGNATURE_ISSUER || $SIGNATURE_ISSUER =~ ^https://[a-zA-Z0-9][a-zA-Z0-9._~:/%+-]*$ ]] || die "SIGNATURE_ISSUER must be the https:// URL of an OIDC issuer"
    if [[ -n $UPSTREAM_DEFAULT ]]; then
        if [[ $MODE == proxy ]]; then
            [[ $UPSTREAM_DEFAULT =~ ^https?://[a-zA-Z0-9][a-zA-Z0-9.-]*(:[0-9]{1,5})?(/[a-zA-Z0-9._~/%-]*)?$ ]] || die "UPSTREAM_DEFAULT must be the http(s):// URL of a JMAP server in proxy mode"
        else
            [[ $UPSTREAM_DEFAULT =~ ^(imaps?|pop3s?)://[a-zA-Z0-9][a-zA-Z0-9.-]*(:[0-9]{1,5})?/?$ ]] || die "UPSTREAM_DEFAULT must be an imaps://, imap://, pop3s:// or pop3:// server URL in engine mode"
        fi
    fi
    if [[ -n $IMAGE_REPOSITORY ]]; then
        [[ $IMAGE_REPOSITORY =~ ^[a-zA-Z0-9][a-zA-Z0-9._/:-]*$ && ${IMAGE_REPOSITORY##*/} != *:* && $IMAGE_REPOSITORY != */ ]] || die "IMAGE_REPOSITORY must be an image repository without a tag or digest, such as ghcr.io/supermarsx/mailwoman"
    fi
    [[ -z $BACKUP_KEEP || $BACKUP_KEEP =~ ^[1-9][0-9]{0,3}$ ]] || die "BACKUP_KEEP must be a number of snapshots from 1 to 9999, or empty"
    [[ -z $BACKUP_KEEP_DAYS || $BACKUP_KEEP_DAYS =~ ^[1-9][0-9]{0,4}$ ]] || die "BACKUP_KEEP_DAYS must be a number of days from 1 to 99999, or empty"
    # age: an age1 recipient or an SSH public key. gpg: a full key fingerprint, so
    # that exactly one key in the keyring can match.
    [[ -z $BACKUP_HOOK || $BACKUP_HOOK =~ ^/[a-zA-Z0-9._/+@-]+$ ]] || die "BACKUP_HOOK must be the absolute path of an executable (letters, digits and ._/+@- only), or empty"
    [[ -z $BACKUP_TARGETS_FILE || $BACKUP_TARGETS_FILE =~ ^/[a-zA-Z0-9._/+@-]+$ ]] || die "BACKUP_TARGETS_FILE must be an absolute JSON configuration path"
    [[ -z $ALERT_HOOK || $ALERT_HOOK =~ ^/[a-zA-Z0-9._/+@-]+$ ]] || die "ALERT_HOOK must be the absolute path of an executable (letters, digits and ._/+@- only), or empty"
    [[ -z $BACKUP_RECIPIENT || $BACKUP_RECIPIENT =~ ^(age1[02-9ac-hj-np-z]{58}|ssh-(ed25519|rsa)\ [A-Za-z0-9+/]+={0,2}|[0-9A-Fa-f]{40}|[0-9A-Fa-f]{64})$ ]] || die "BACKUP_RECIPIENT must be an age recipient (age1...), an SSH public key (ssh-ed25519 AAAA...) or a full gpg key fingerprint (40 hex digits), or empty"
}

write_config() {
    cat <<EOF
# Mailwoman deployment settings. Plain KEY=value, no quotes or shell expansion.
PROJECT=$PROJECT
EXPOSURE=$EXPOSURE
DOMAIN=$DOMAIN
PUBLIC_URL=$PUBLIC_URL
PORT=$PORT
MODE=$MODE
PROXY_PREFIX=$PROXY_PREFIX
PROXY_NETWORK=$PROXY_NETWORK
CADDY_IMAGE=$CADDY_IMAGE
CADDY_PROTOCOLS=$CADDY_PROTOCOLS
ACME_CA=$ACME_CA
ACME_CA_ROOT=$ACME_CA_ROOT
MEMORY_LIMIT=$MEMORY_LIMIT
CPU_LIMIT=$CPU_LIMIT
SIGNATURE_IDENTITY=$SIGNATURE_IDENTITY
SIGNATURE_ISSUER=$SIGNATURE_ISSUER
UPSTREAM_DEFAULT=$UPSTREAM_DEFAULT
IMAGE_REPOSITORY=$IMAGE_REPOSITORY
BACKUP_KEEP=$BACKUP_KEEP
BACKUP_KEEP_DAYS=$BACKUP_KEEP_DAYS
BACKUP_RECIPIENT=$BACKUP_RECIPIENT
BACKUP_HOOK=$BACKUP_HOOK
BACKUP_TARGETS_FILE=$BACKUP_TARGETS_FILE
ALERT_HOOK=$ALERT_HOOK
EOF
}

validate_app_env() {
    local line key value found=false seen='|'
    [[ -f $1 && ! -L $1 ]] || die "Missing or symlinked application configuration: $1"
    while IFS= read -r line || [[ -n $line ]]; do
        [[ -z $line || $line == \#* ]] && continue
        [[ $line != *$'\r'* && $line =~ ^[A-Z][A-Z0-9_]*= ]] || die "app.env requires one unquoted KEY=value per line (LF endings)"
        key=${line%%=*}; value=${line#*=}
        [[ $seen != *"|$key|"* ]] || die "Duplicate $key in app.env"
        seen+="$key|"
        case "$key" in
            MW_SERVER_KEY) [[ $value =~ ^[0-9a-fA-F]{64}$ ]] || die "MW_SERVER_KEY must contain 64 hex digits"; found=true ;;
            MW_BIND|MW_DB_PATH|MW_UPLOAD_DIR|MW_MODE|MW_PUBLIC_URL|MW_COOKIE_SECURE|MW_RENDER_BIN|MW_PLUGIN_DIR|MW_WEB_DIR|MW_BASE_PATH|MW_ACME*|MW_TLS_*|MW_PROXY_PROTOCOL|MW_IMAGE_DIGEST)
                die "$key is managed by deploy.sh; see docs/deploy/script.md" ;;
            MW_TRUSTED_PROXIES|MW_FORWARDED_MODE)
                [[ $EXPOSURE != caddy ]] || die "$key is managed for Caddy; remove it from app.env" ;;
            MW_UPSTREAM_DEFAULT)
                [[ -z $UPSTREAM_DEFAULT ]] || die "The mail server is set twice: UPSTREAM_DEFAULT in deploy.env and MW_UPSTREAM_DEFAULT in app.env; keep one" ;;
        esac
    done < "$1"
    $found || die "MW_SERVER_KEY is missing from app.env"
}

cleanup() {
    local result=$?
    trap - EXIT
    if [[ -n $RESUME_CONTAINER ]]; then
        log "Resuming app after interrupted/failed snapshot"
        docker start "$RESUME_CONTAINER" >/dev/null || log "Recovery needed: docker start $RESUME_CONTAINER"
    fi
    if [[ -n $HELPER_CONTAINER ]]; then docker rm "$HELPER_CONTAINER" >/dev/null 2>&1 || true; fi
    # EXTRACT holds a decrypted snapshot while it is restored.
    if [[ -n $EXTRACT && -d $EXTRACT && $EXTRACT == "$STATE"/.work.* ]]; then rm -rf -- "$EXTRACT"; fi
    if [[ -n $WORK && -d $WORK && $WORK == "$STATE"/.work.* ]]; then
        # WORK is always a private mktemp child of STATE; never remove arbitrary input.
        rm -rf -- "$WORK"
    fi
    if $LOCKED; then rmdir -- "$STATE/.lock" 2>/dev/null || true; fi
    if ((result != 0)) && [[ $ACTION != check ]]; then
        colored_log "$LOG_ERROR" "Command failed. Data volumes have been retained. See 'status' and 'logs'."
        if ((PROGRESS_STARTED > 0)); then
            log "Stopped: $(date -u '+%Y-%m-%d %H:%M:%S UTC'); elapsed $(($(date +%s) - PROGRESS_STARTED))s"
        fi
    fi
    exit "$result"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

# Dry-run must work even on a machine without Docker and must not mkdir/chmod.
if [[ -f $STATE/deploy.env ]]; then
    read_config "$STATE/deploy.env"
    # Repeating the original setup command is an update. A different value is
    # never applied from the command line; deploy.env is the one place to change it.
    for key in "${!REQUESTED[@]}"; do
        [[ ${REQUESTED[$key]} == "${!key}" ]] || die "$key is '${!key}' in $STATE/deploy.env, not '${REQUESTED[$key]}'; edit that file to change it"
    done
else
    [[ -z $DOMAIN || -z $PUBLIC_URL ]] || die "Choose --domain OR --public-url"
    [[ -z $PROXY_NETWORK || -n $PUBLIC_URL ]] || die "--proxy-network needs --public-url (the HTTPS address your proxy serves)"
    if [[ -n $DOMAIN ]]; then EXPOSURE=caddy; PUBLIC_URL=https://$DOMAIN
    elif [[ -n $PROXY_NETWORK ]]; then EXPOSURE=network
    else
        # A new install without --port takes the first free port; deploy.env then keeps it.
        if ! $PORT_SET && port_in_use "$PORT"; then
            for candidate in {8081..8099}; do
                if ! port_in_use "$candidate"; then
                    log "Port $PORT is in use; using $candidate instead"
                    PORT=$candidate
                    break
                fi
            done
        fi
        if [[ -n $PUBLIC_URL ]]; then EXPOSURE=external; else PUBLIC_URL=http://localhost:$PORT; fi
    fi
fi
validate_config
if [[ -n $IMAGE_TAG ]]; then
    [[ -n $IMAGE_REPOSITORY ]] || die "--image-tag needs IMAGE_REPOSITORY in deploy.env, such as ghcr.io/supermarsx/mailwoman"
    IMAGE="$IMAGE_REPOSITORY:$IMAGE_TAG"
    PULL=true
fi
if $DRY_RUN; then
    log "Dry run: $ACTION; project=$PROJECT; mode=$MODE; exposure=$EXPOSURE"
    log "State: $STATE; URL: $PUBLIC_URL"
    case "$ACTION" in
        deploy) log "Prepare runtime image (${IMAGE:-$($NO_BUILD && printf 'last applied image' || printf 'build Dockerfile --target runtime-static')}${IMAGE:+; signature checked when it is fetched from its registry and cosign is installed}); snapshot existing data; apply Compose; verify app and public HTTP(S)$($AUTO_ROLLBACK && printf '; if an update fails that check, restore the previous release and its data')." ;;
        restore|rollback) log "Validate snapshot checksums; save current installation; restore snapshot to a new volume; apply and verify. Post-snapshot writes will not be in the restored app." ;;
        *) log "Would run $ACTION. No files, containers, networks or volumes changed." ;;
    esac
    exit 0
fi

if [[ ! -d $STATE ]]; then
    [[ $ACTION == init || $ACTION == deploy ]] || die "Run init or deploy first"
    mkdir -p -- "$STATE"
fi
[[ ! -L $STATE ]] || die "State directory must not be a symlink"
STATE="$(cd -- "$STATE" && pwd -P)"
case "$STATE/" in
    "$ROOT/"*)
        relative=${STATE#"$ROOT/"}
        [[ $STATE != "$ROOT" && ${relative%%/*} == *.local ]] || die "Inside the repository, state must be in a top-level *.local directory (excluded from Git and Docker)" ;;
esac
ACTIVE="$STATE/active"
if [[ $ACTION != status && $ACTION != check && $ACTION != logs && $ACTION != doctor ]]; then
    chmod 700 "$STATE"
    mkdir "$STATE/.lock" 2>/dev/null || die "Another deployment operation holds $STATE/.lock; remove this empty lock only after verifying no operation is running"
    LOCKED=true
fi

initialize() {
    [[ ! -e $STATE/deploy.env && ! -e $STATE/app.env ]] || die "Partial configuration found; retain the existing key and complete both deploy.env and app.env"
    need od
    local key password metrics
    key=$(od -An -N32 -tx1 /dev/urandom | tr -d ' \n')
    password=$(od -An -N24 -tx1 /dev/urandom | tr -d ' \n')
    metrics=$(od -An -N32 -tx1 /dev/urandom | tr -d ' \n')
    [[ ${#key} == 64 && ${#password} == 48 && ${#metrics} == 64 ]] || die "Failed to generate secrets"
    # New installations get the Helm chart's limits; deploy.env files written
    # before these keys existed keep running without one.
    MEMORY_LIMIT=1g
    CPU_LIMIT=2
    write_config > "$STATE/deploy.env"
    cat > "$STATE/app.env" <<EOF
# Raw Docker environment: values are literal; do not quote or escape dollars.
# Preserve MW_SERVER_KEY across upgrades. Back up this file with the data.
MW_SERVER_KEY=$key
MW_ADMIN_USER=admin
MW_ADMIN_PASSWORD=$password
# Optional: serve the admin console at a path of your own instead of /admin,
# e.g. /ops-7f3k (then /admin answers 404). Fewer scans and drive-by sign-in
# attempts; not a substitute for the admin password, two-factor sign-in or a
# network restriction. See docs/deploy/README.md, "Moving the admin console".
# MW_ADMIN_PATH=/admin
MW_CSRF_STRICT=true
RUST_LOG=info
# Bearer token for GET /metrics (Prometheus text format); without it /metrics is
# refused. Remove the line to turn metrics off. See docs/deploy/monitoring.md.
MW_METRICS_TOKEN=$metrics
# Required for sign-in: the mail server every user signs in to, set as
# UPSTREAM_DEFAULT in deploy.env (or as MW_UPSTREAM_DEFAULT here, not both). The
# sign-in form asks only for the email address and the password.
# Optional: comma-separated origins allowed for proxy-mode JMAP logins.
# MW_JMAP_UPSTREAMS=https://jmap.example.org
# Engine-mode SMTP overrides (otherwise inferred from the mailbox host):
# MW_SMTP_HOST=smtp.example.org
# MW_SMTP_PORT=587
# MW_SMTP_SECURITY=starttls
# Optional external cache (not required):
# MW_REDIS_URL=redis://cache.example.org:6379
EOF
    chmod 600 "$STATE/deploy.env" "$STATE/app.env"
    log "Created $STATE/deploy.env and app.env (admin credentials and the metrics token are in app.env)"
}
if [[ ! -f $STATE/deploy.env ]]; then
    [[ $ACTION == init || $ACTION == deploy ]] || die "Run init first"
    initialize
fi
if [[ $ACTION == init || $ACTION == deploy || $ACTION == doctor ]]; then validate_app_env "$STATE/app.env"; fi
if [[ $ACTION == init ]]; then log "Configuration ready. Edit it as needed, then run deploy."; exit 0; fi

preflight() {
    need docker; need curl; need tar; need sha256sum
    local version major minor endpoint
    endpoint=${DOCKER_HOST:-$(docker context inspect --format '{{.Endpoints.docker.Host}}')}
    [[ $endpoint == unix://* || $endpoint == npipe://* ]] || die "Use a local Docker daemon; remote contexts cannot be verified through localhost"
    version=$(docker compose version --short) || die "Docker Compose plugin is required"
    version=${version#v}
    [[ $version =~ ^([0-9]+)\.([0-9]+) ]] || die "Cannot parse Compose version: $version"
    major=${BASH_REMATCH[1]}; minor=${BASH_REMATCH[2]}
    ((major > 2 || (major == 2 && minor >= 30))) || die "Docker Compose 2.30+ is required (raw env files)"
    [[ $(docker info --format '{{.OSType}}') == linux ]] || die "Docker must be running Linux containers"
}

compose() {
    local directory=$1; shift
    # Explicit project, file and empty interpolation environment isolate this stack
    # from the caller's .env / COMPOSE_FILE / COMPOSE_PROFILES settings.
    COMPOSE_PROFILES='' docker compose --project-name "$PROJECT" --env-file /dev/null \
        --project-directory "$directory" -f "$directory/compose.yaml" "$@"
}

# Fail before any build, snapshot or container change when a host port this
# deployment publishes is held by something other than its own running service.
require_free_ports() {
    local service=mailwoman port cid own=
    local -a ports=("$PORT")
    [[ $EXPOSURE != network ]] || return 0
    if [[ $EXPOSURE == caddy ]]; then service=caddy; ports=(80 443); fi
    if [[ -f $ACTIVE/compose.yaml ]]; then
        cid=$(compose "$ACTIVE" ps -q "$service")
        if [[ -n $cid ]]; then own=$(docker port "$cid"); fi
    fi
    for port in "${ports[@]}"; do
        if grep -q ":$port\$" <<< "$own" || ! port_in_use "$port"; then continue; fi
        if [[ $EXPOSURE == caddy ]]; then die "Port $port is already in use on this host; Caddy needs ports 80 and 443. Stop the service holding it, then deploy again"; fi
        die "Port $port is already in use on this host. Stop the service holding it, or set PORT$([[ $EXPOSURE != local ]] || printf ' and PUBLIC_URL') in $STATE/deploy.env to a free port, then deploy again"
    done
}

# The proxy network outlives this deployment and may belong to the proxy's own
# stack, so it is created when missing and never removed.
ensure_proxy_network() {
    [[ $EXPOSURE == network ]] || return 0
    if ! docker network inspect "$PROXY_NETWORK" >/dev/null 2>&1; then
        log "Creating Docker network $PROXY_NETWORK for the reverse proxy"
        docker network create --label "mailwoman.deploy.project=$PROJECT" "$PROXY_NETWORK" >/dev/null
    fi
    PROXY_SUBNETS=$(docker network inspect --format '{{range .IPAM.Config}}{{.Subnet}},{{end}}' "$PROXY_NETWORK")
    PROXY_SUBNETS=${PROXY_SUBNETS%,}
    [[ $PROXY_SUBNETS =~ ^[0-9a-fA-F:./,]*$ ]] || die "Cannot read the subnets of network $PROXY_NETWORK"
}

# image_repository REF: REF without its tag or digest.
image_repository() {
    local ref=${1%%@*}
    if [[ ${ref##*/} == *:* ]]; then ref=${ref%:*}; fi
    printf '%s' "$ref"
}

# check_signature FROM_REGISTRY: verify the keyless signature of an --image that
# this deploy fetches from its registry, with cosign when it is installed. On
# success IMAGE becomes the verified digest, so the pull fetches exactly what was
# verified. A failed verification stops the deploy before anything changes; an
# image that cannot be checked is reported, and refused under --require-signature.
# A copy already in the local image store is not checked: whether it came from a
# registry cannot be told (Docker's containerd store gives locally built images a
# repository digest too).
check_signature() {
    local from_registry=$1 ref=$IMAGE repository reason='' output digest identity issuer
    repository=$(image_repository "$IMAGE")
    $from_registry || reason="$IMAGE is used from the local image store; --pull fetches it from its registry and checks it there"
    if [[ -z $reason ]] && ! command -v cosign >/dev/null 2>&1; then reason="cosign is not installed"; fi
    if [[ -n $reason ]]; then
        $REQUIRE_SIGNATURE && die "The signature of $IMAGE cannot be checked ($reason) and --require-signature was given; nothing was changed"
        log "Signature NOT checked: $reason. Use --require-signature to refuse such an image"
        return 0
    fi
    identity=${SIGNATURE_IDENTITY:-$DEFAULT_SIGNATURE_IDENTITY}
    issuer=${SIGNATURE_ISSUER:-$DEFAULT_SIGNATURE_ISSUER}
    log "Checking the signature of $ref: signed by $identity ($issuer)"
    if ! output=$(cosign verify --certificate-identity "$identity" --certificate-oidc-issuer "$issuer" "$ref" 2> "$WORK/cosign.log"); then
        die "The signature of $ref did not verify; nothing was changed. cosign: $(tail -n 3 "$WORK/cosign.log" | tr '\n' ' ')"
    fi
    digest=$(grep -o '"docker-manifest-digest":"sha256:[0-9a-f]\{64\}"' <<< "$output" | awk -F'"' 'NR == 1 {print $4}')
    [[ -n $digest ]] || die "cosign accepted $ref but printed no image digest; nothing was changed"
    IMAGE="$repository@$digest"
    log "Signature verified: $IMAGE"
}

# Keys that steer backup, prune and check rather than the running app. They take
# effect as soon as deploy.env is saved, so they are not taken from the applied copy.
OPERATION_KEYS=(BACKUP_KEEP BACKUP_KEEP_DAYS BACKUP_RECIPIENT BACKUP_HOOK BACKUP_TARGETS_FILE ALERT_HOOK)

# image_digest: set IMAGE_DIGEST to repository@sha256:... when the local image
# IMAGE has a registry digest for the repository it is named by.
image_digest() {
    local repository
    repository=$(image_repository "$IMAGE")
    IMAGE_DIGEST=$(docker image inspect --format '{{range .RepoDigests}}{{println .}}{{end}}' "$IMAGE" | awk -F@ -v repository="$repository" '$1 == repository && $2 ~ /^sha256:[0-9a-f]+$/ {print; exit}')
}

load_active() {
    local key
    local -A operation=()
    [[ -f $ACTIVE/release.env ]] || die "No applied deployment. Run deploy first"
    for key in "${OPERATION_KEYS[@]}"; do operation[$key]=${!key}; done
    read_config "$ACTIVE/deploy.env"
    read_config "$ACTIVE/release.env"
    for key in "${OPERATION_KEYS[@]}"; do printf -v "$key" '%s' "${operation[$key]}"; done
    validate_config
}

# indent PREFIX TEXT: TEXT with PREFIX before each line.
indent() { printf '%s%s\n' "$1" "${2//$'\n'/$'\n'$1}"; }

# Prints the certificates of a PEM file, refusing anything else.
pem_certificates() {
    local pem
    [[ -f $1 && -r $1 ]] || die "ACME_CA_ROOT names no readable file: $1"
    pem=$(tr -d '\r' < "$1")
    if ! grep -qx -- '-----BEGIN CERTIFICATE-----' <<< "$pem" || grep -qvxE -- '-----(BEGIN|END) CERTIFICATE-----|[A-Za-z0-9+/=]*' <<< "$pem"; then
        die "ACME_CA_ROOT must hold PEM certificates only: $1"
    fi
    printf '%s\n' "$pem"
}

render() {
    local directory=$1 secure=true trust='' upstream='' caddyfile caddy_env root_pem limits='' cpus=$CPU_LIMIT host_cpus
    [[ $EXPOSURE != local ]] || secure=false
    # Client addresses come from the proxy network's peers unless app.env says otherwise.
    if [[ $EXPOSURE == network && -n $PROXY_SUBNETS ]] && ! grep -Eq '^MW_(TRUSTED_PROXIES|FORWARDED_MODE)=' "$directory/app.env"; then
        trust=$'\n'"      MW_TRUSTED_PROXIES: '$PROXY_SUBNETS'"$'\n'"      MW_FORWARDED_MODE: xff"
    fi
    if [[ -n $UPSTREAM_DEFAULT ]]; then upstream=$'\n'"      MW_UPSTREAM_DEFAULT: '$UPSTREAM_DEFAULT'"; fi
    if [[ -n $MEMORY_LIMIT ]]; then limits+=$'\n'"    mem_limit: $MEMORY_LIMIT"; fi
    if [[ -n $cpus ]]; then
        # Docker refuses a CPU limit above the CPUs it can use.
        host_cpus=$(docker info --format '{{.NCPU}}')
        [[ $host_cpus =~ ^[1-9][0-9]*$ ]] || die "Cannot read the number of CPUs Docker can use"
        if awk -v want="$cpus" -v have="$host_cpus" 'BEGIN { exit !(want > have) }'; then
            log "CPU_LIMIT=$cpus is more than the $host_cpus CPUs Docker can use; the app is limited to $host_cpus"
            cpus=$host_cpus
        fi
        limits+=$'\n'"    cpus: $cpus"
    fi
    [[ $IMAGE_ID =~ ^sha256:[0-9a-f]{64}$ ]] || die "Invalid image ID"
    [[ -z $IMAGE_DIGEST || $IMAGE_DIGEST =~ ^([a-zA-Z0-9][a-zA-Z0-9._/:-]*@)?sha256:[0-9a-f]{64}$ ]] || die "Invalid image digest"
    [[ $DATA_VOLUME =~ ^[a-zA-Z0-9][a-zA-Z0-9_.-]+$ ]] || die "Invalid volume name"
    write_config > "$directory/deploy.env"
    printf 'IMAGE_ID=%s\nIMAGE_DIGEST=%s\nDATA_VOLUME=%s\nCADDY_ID=%s\n' "$IMAGE_ID" "$IMAGE_DIGEST" "$DATA_VOLUME" "$CADDY_ID" > "$directory/release.env"
    cat > "$directory/compose.yaml" <<EOF
# Generated by deploy.sh. Edit the parent deploy.env / app.env, then deploy.
services:
  mailwoman:
    image: $IMAGE_ID
    pull_policy: never
    restart: unless-stopped
    working_dir: /data
    user: '65532:65532'
    read_only: true
    cap_drop: [ALL]
    security_opt: [no-new-privileges:true]
    pids_limit: 512$limits
    # The server's graceful shutdown handler listens for Ctrl-C / SIGINT.
    stop_signal: SIGINT
    stop_grace_period: 60s
    tmpfs: ['/tmp:rw,noexec,nosuid,nodev,size=64m,mode=1777']
    env_file:
      - path: ./app.env
        format: raw
    environment:
      MW_BIND: '0.0.0.0:8080'
      MW_DB_PATH: /data/mailwoman.db
      MW_UPLOAD_DIR: /data/uploads
      MW_RENDER_BIN: /usr/local/bin/mw-render
      MW_PLUGIN_DIR: /usr/lib/mailwoman/plugins
      MW_MODE: '$MODE'
      MW_PUBLIC_URL: '$PUBLIC_URL'
      MW_COOKIE_SECURE: '$secure'$trust$upstream
      MW_IMAGE_DIGEST: '${IMAGE_DIGEST:-$IMAGE_ID}'
    volumes: ['app_data:/data']
    healthcheck:
      test: [CMD, /usr/local/bin/mailwoman, healthcheck, --url, 'http://127.0.0.1:8080/healthz']
      interval: 5s
      timeout: 5s
      start_period: 20s
      retries: 12
    logging:
      driver: json-file
      options: {max-size: 10m, max-file: '3'}
EOF
    if [[ $EXPOSURE == caddy ]]; then
        [[ $CADDY_ID =~ ^sha256:[0-9a-f]{64}$ ]] || die "Invalid Caddy image ID"
        # Another ACME directory (a staging endpoint or a private CA) and the root
        # that signs its TLS certificate become Caddy global options. The Caddyfile
        # and that root travel in the Compose file as environment values, so a
        # snapshot keeps them, and the entrypoint writes them to the tmpfs: Compose
        # refuses inline configs for a service with a read-only root filesystem.
        caddyfile="{"$'\n'"    servers {"$'\n'"        protocols $CADDY_PROTOCOLS"$'\n'"    }"$'\n'
        [[ -z $ACME_CA ]] || caddyfile+="    acme_ca $ACME_CA"$'\n'
        [[ -z $ACME_CA_ROOT ]] || caddyfile+="    acme_ca_root /tmp/acme-ca-root.pem"$'\n'
        caddyfile+="}"$'\n'
        caddyfile+="$DOMAIN {"$'\n'"    reverse_proxy mailwoman:8080 {"$'\n'"        flush_interval -1"$'\n'"    }"$'\n'"}"
        caddy_env="      MW_CADDYFILE: |"$'\n'"$(indent '        ' "$caddyfile")"
        if [[ -n $ACME_CA_ROOT ]]; then
            root_pem=$(pem_certificates "$ACME_CA_ROOT")
            caddy_env+=$'\n'"      MW_ACME_CA_ROOT: |"$'\n'"$(indent '        ' "$root_pem")"
        fi
        cat >> "$directory/compose.yaml" <<EOF
    networks:
      default:
        ipv4_address: $PROXY_PREFIX.3
  caddy:
    image: $CADDY_ID
    pull_policy: never
    restart: unless-stopped
    read_only: true
    security_opt: [no-new-privileges:true]
    cap_drop: [ALL]
    cap_add: [NET_BIND_SERVICE]
    pids_limit: 128
    tmpfs: ['/tmp:rw,noexec,nosuid,nodev,size=16m']
    ports: ['80:80/tcp', '443:443/tcp'$(if [[ $CADDY_PROTOCOLS == *h3* ]]; then printf ", '443:443/udp'"; fi)]
    volumes: ['caddy_data:/data', 'caddy_config:/config']
    environment:
$caddy_env
    entrypoint: [/bin/sh, -c, 'printf "%s\n" "\$\$MW_CADDYFILE" > /tmp/Caddyfile && printf "%s\n" "\$\${MW_ACME_CA_ROOT:-}" > /tmp/acme-ca-root.pem && exec caddy run --config /tmp/Caddyfile --adapter caddyfile']
    depends_on:
      mailwoman:
        condition: service_healthy
    networks:
      default:
        ipv4_address: $PROXY_PREFIX.2
    logging:
      driver: json-file
      options: {max-size: 10m, max-file: '3'}
networks:
  default:
    ipam:
      config:
        - subnet: $PROXY_PREFIX.0/24
EOF
        printf '\nMW_TRUSTED_PROXIES=%s.2\nMW_FORWARDED_MODE=xff\n' "$PROXY_PREFIX" >> "$directory/app.env"
    elif [[ $EXPOSURE == network ]]; then
        cat >> "$directory/compose.yaml" <<EOF
    networks:
      proxy:
        aliases: ['$PROJECT']
networks:
  proxy:
    external: true
    name: $PROXY_NETWORK
EOF
    else
        printf "    ports: ['127.0.0.1:%s:8080']\n" "$PORT" >> "$directory/compose.yaml"
    fi
    cat >> "$directory/compose.yaml" <<EOF
volumes:
  app_data:
    external: true
    name: $DATA_VOLUME
  caddy_data:
    name: ${PROJECT}_caddy_data
  caddy_config:
    name: ${PROJECT}_caddy_config
EOF
}

verify() {
    local cid wait=$TIMEOUT deadline url=$PUBLIC_URL
    # Behind a separately managed proxy the public URL may not be routed yet.
    if [[ $EXPOSURE == network ]] && ((wait > 15)); then wait=15; fi
    deadline=$((SECONDS + wait))
    cid=$(compose "$ACTIVE" ps -q mailwoman)
    [[ -n $cid ]] || die "App container is missing"
    MSYS_NO_PATHCONV=1 docker exec "$cid" /usr/local/bin/mailwoman healthcheck --url http://127.0.0.1:8080/healthz || die "App health check failed"
    log "Verifying $url/healthz and the web UI (up to ${wait}s)"
    while ((SECONDS < deadline)); do
        if curl --fail --silent --show-error --connect-timeout 3 --max-time 5 "$url/healthz" -o /dev/null 2>/dev/null &&
           curl --fail --silent --show-error --connect-timeout 3 --max-time 5 "$url/" -o /dev/null 2>/dev/null; then
            log "Healthy: $url (admin credentials: $STATE/app.env)"
            return 0
        fi
        sleep 2
    done
    if [[ $EXPOSURE == network ]]; then
        log "App is healthy on Docker network $PROXY_NETWORK, but $url does not answer yet."
        log "Attach your proxy to that network and forward ${url#https://} to http://$PROJECT:8080 with WebSocket support."
        return 0
    fi
    die "App is healthy internally, but $url is not reachable with valid HTTP(S). Check DNS, firewall, proxy and service logs; containers remain available."
}

# The files of a snapshot directory, which is also the order in an encrypted archive.
SNAPSHOT_FILES=(COMPLETE SHA256SUMS data.tar compose.yaml app.env deploy.env release.env)

# record_snapshot KIND PATH: write last-snapshot.json into the app's data volume,
# for the admin console: which snapshot was taken last, when, and whether it is
# encrypted. KIND is backup, pre-deploy or pre-restore. A failure to write it is
# reported and does not fail the command.
record_snapshot() {
    local kind=$1 path=$2 cid name stamp encrypted=false escaped
    cid=$(compose "$ACTIVE" ps -a -q mailwoman 2>/dev/null) || cid=
    [[ -n $cid ]] || { log "No app container; last-snapshot.json not written"; return 0; }
    name=${path##*/}
    if [[ $name == *.tar.age || $name == *.tar.gpg ]]; then encrypted=true; name=${name%.tar.*}; fi
    stamp=${name:0:16}
    [[ -n $WORK && -d $WORK ]] || WORK=$(mktemp -d "$STATE/.work.XXXXXX")
    escaped=${path//\\/\\\\}
    escaped=${escaped//\"/\\\"}
    printf '{"schema":"mailwoman-deploy-snapshot/1","kind":"%s","takenAt":"%s-%s-%sT%s:%s:%sZ","path":"%s","encrypted":%s}\n' \
        "$kind" "${stamp:0:4}" "${stamp:4:2}" "${stamp:6:2}" "${stamp:9:2}" "${stamp:11:2}" "${stamp:13:2}" "$escaped" "$encrypted" > "$WORK/last-snapshot.json"
    # Readable by the app user; the record holds no secret.
    chmod 644 "$WORK/last-snapshot.json"
    docker cp "$WORK/last-snapshot.json" "$cid:/data/last-snapshot.json" >/dev/null 2>&1 || log "Could not write last-snapshot.json into the app's data volume"
}

# encryption_tool: age for an age or SSH recipient, gpg for a key fingerprint.
encryption_tool() { case "$BACKUP_RECIPIENT" in age1*|ssh-*) printf age ;; *) printf gpg ;; esac; }

# encrypt_snapshot DIR: write DIR as one archive encrypted to BACKUP_RECIPIENT, then
# remove DIR. Only the recipient's public key is used; nothing here can decrypt it.
# On failure DIR stays as it is and no archive is left.
encrypt_snapshot() {
    local from=$1 name tool archive
    name=${from##*/}
    tool=$(encryption_tool)
    need "$tool"
    archive="$STATE/backups/$name.tar.$tool"
    log "Encrypting the snapshot with $tool to $BACKUP_RECIPIENT"
    if [[ $tool == age ]]; then
        tar -C "$from" -cf - "${SNAPSHOT_FILES[@]}" | age --encrypt --recipient "$BACKUP_RECIPIENT" --output "$archive.partial" || { rm -f -- "$archive.partial"; die "age could not encrypt the snapshot; the clear-text copy is still at $from"; }
    else
        tar -C "$from" -cf - "${SNAPSHOT_FILES[@]}" | gpg --batch --yes --quiet --trust-model always --encrypt --recipient "$BACKUP_RECIPIENT" --output "$archive.partial" || { rm -f -- "$archive.partial"; die "gpg could not encrypt the snapshot to $BACKUP_RECIPIENT (is its public key in this user's keyring?); the clear-text copy is still at $from"; }
    fi
    [[ -s $archive.partial ]] || { rm -f -- "$archive.partial"; die "$tool wrote an empty archive; the clear-text copy is still at $from"; }
    mv -- "$archive.partial" "$archive"
    # The image IDs stay readable beside the archive: prune keeps the images a
    # snapshot needs, and restore needs the image before it can start.
    cp -- "$from/release.env" "$STATE/backups/$name.release.env"
    [[ $from == "$STATE"/backups/* ]] && rm -rf -- "$from"
    LAST_BACKUP=$archive
    log "Encrypted backup ready: $archive (clear-text copy removed)"
}

# run_backup_hook: hand the finished backup (archive or snapshot directory) to
# BACKUP_HOOK, for example to copy it off the host. A failing hook fails the backup;
# the local copy stays either way.
run_backup_hook() {
    local status=0
    [[ -f $BACKUP_HOOK && -x $BACKUP_HOOK ]] || die "BACKUP_HOOK $BACKUP_HOOK is not an executable file; the backup is at $LAST_BACKUP"
    log "Running BACKUP_HOOK $BACKUP_HOOK"
    "$BACKUP_HOOK" "$LAST_BACKUP" < /dev/null || status=$?
    ((status == 0)) || die "BACKUP_HOOK $BACKUP_HOOK exited $status; the backup is kept at $LAST_BACKUP"
    log "BACKUP_HOOK finished"
}

# decrypt_snapshot ARCHIVE: decrypt an archive written by encrypt_snapshot into a
# private directory under the state directory, removed when the command ends, and
# set SNAPSHOT to it.
decrypt_snapshot() {
    local archive=$1 name tool
    [[ -f $archive && ! -L $archive ]] || die "Not an encrypted snapshot: $archive"
    name=${archive##*/}
    tool=${name##*.}
    name=${name%.tar.*}
    [[ $name =~ $SNAPSHOT_NAME ]] || die "Not an encrypted snapshot name: $archive"
    need "$tool"
    EXTRACT=$(mktemp -d "$STATE/.work.XXXXXX")
    mkdir "$EXTRACT/$name"
    if [[ $tool == age ]]; then
        [[ -n $IDENTITY ]] || die "$archive is encrypted with age: give the identity file that matches BACKUP_RECIPIENT with --identity FILE"
        [[ -f $IDENTITY && -r $IDENTITY ]] || die "--identity names no readable file: $IDENTITY"
        age --decrypt --identity "$IDENTITY" "$archive" | tar -xf - -C "$EXTRACT/$name" "${SNAPSHOT_FILES[@]}" || die "Could not decrypt $archive with $IDENTITY; nothing was changed"
    else
        [[ -z $IDENTITY ]] || die "--identity is for .tar.age archives; gpg decrypts with the private key in your keyring"
        gpg --batch --quiet --decrypt "$archive" | tar -xf - -C "$EXTRACT/$name" "${SNAPSHOT_FILES[@]}" || die "Could not decrypt $archive: gpg needs the private key of its recipient in your keyring; nothing was changed"
    fi
    SNAPSHOT="$EXTRACT/$name"
}

# snapshot [keep-stopped]: with keep-stopped a running app stays stopped once the
# copy is complete (restore replaces it next, healthy or not); after a failed copy
# it is resumed either way.
snapshot() {
    local cid running destination keep_stopped=${1:-}
    cid=$(compose "$ACTIVE" ps -a -q mailwoman)
    [[ -n $cid ]] || die "No app container to back up"
    destination="$STATE/backups/$(date -u +%Y%m%dT%H%M%SZ)-$$-${RANDOM}"
    mkdir -p "$destination"
    cp "$ACTIVE/"{compose.yaml,app.env,deploy.env,release.env} "$destination/"
    # Stopping the only app writer makes SQLite + WAL + uploads a consistent set.
    running=$(docker inspect --format '{{.State.Running}}' "$cid")
    if [[ $running == true ]]; then
        RESUME_CONTAINER=$cid
        docker stop -t 60 "$cid" >/dev/null
    fi
    log "Snapshotting /data to $destination"
    docker cp -a "$cid:/data/." - > "$destination/data.tar"
    tar -tf "$destination/data.tar" >/dev/null
    (cd "$destination" && sha256sum data.tar compose.yaml app.env deploy.env release.env > SHA256SUMS)
    # A snapshot is usable only once COMPLETE exists; partial backups remain for diagnosis.
    printf 'mailwoman-deploy-snapshot-v1\n' > "$destination/COMPLETE"
    if [[ -n $RESUME_CONTAINER && -n $keep_stopped ]]; then
        RESUME_CONTAINER=
    elif [[ -n $RESUME_CONTAINER ]]; then
        docker start "$RESUME_CONTAINER" >/dev/null
        RESUME_CONTAINER=
        # Compose 2.30 supports waiting on `up`, but not on `start`. Keep the
        # existing container and its exact configuration while waiting for health.
        compose "$ACTIVE" up -d --no-recreate --wait --wait-timeout "$TIMEOUT" mailwoman
    fi
    LAST_BACKUP=$destination
    log "Backup ready: $LAST_BACKUP"
}

activate() {
    local source=$1
    mkdir -p "$ACTIVE"
    cp "$source/"{compose.yaml,app.env,deploy.env,release.env} "$ACTIVE/"
    compose "$ACTIVE" up -d --remove-orphans --wait --wait-timeout "$TIMEOUT"
    verify
}

# restore_snapshot DIR save|replace-failed: start the release saved in DIR on a
# copy of its data in a new volume. save first snapshots the current installation
# and makes DIR's configuration the state directory's; replace-failed is the
# automatic rollback, which leaves deploy.env and app.env as the user wrote them.
restore_snapshot() {
    local from=$1 how=$2 file target_project key
    local -A operation=()
    [[ -d $from && -f $from/COMPLETE ]] || die "Not a complete snapshot: $from"
    from=$(cd "$from" && pwd -P)
    [[ $(cat "$from/COMPLETE") == mailwoman-deploy-snapshot-v1 ]] || die "Unknown snapshot format"
    for file in data.tar compose.yaml app.env deploy.env release.env SHA256SUMS; do
        [[ -f $from/$file && ! -L $from/$file ]] || die "Missing or symlinked snapshot file: $file"
    done
    # Check the fixed expected set, ignoring any extra manifest paths.
    cmp -s <(cd "$from" && sha256sum data.tar compose.yaml app.env deploy.env release.env) "$from/SHA256SUMS" || die "Snapshot checksum mismatch"
    target_project=$PROJECT
    # The snapshot brings back the app's configuration, not how backups are taken.
    for key in "${OPERATION_KEYS[@]}"; do operation[$key]=${!key}; done
    read_config "$from/deploy.env"
    read_config "$from/release.env"
    for key in "${OPERATION_KEYS[@]}"; do printf -v "$key" '%s' "${operation[$key]}"; done
    validate_config
    [[ $PROJECT == "$target_project" ]] || die "Snapshot belongs to project $PROJECT; initialize recovery state with that project name"
    if [[ ! -f $ACTIVE/release.env && -n $(docker ps -aq --filter "label=com.docker.compose.project=$PROJECT") ]]; then
        die "Project $PROJECT is already running without this state directory; recover its original state before restoring"
    fi
    docker image inspect "$IMAGE_ID" >/dev/null || die "Snapshot image is missing; load the matching saved Docker image before restoring"
    if [[ $EXPOSURE == caddy ]]; then docker image inspect "$CADDY_ID" >/dev/null; fi
    WORK=$(mktemp -d "$STATE/.work.XXXXXX")
    cp "$from/app.env" "$WORK/app.env"
    # Managed trust lines in the effective backup are regenerated by render.
    if [[ $EXPOSURE == caddy ]]; then
        awk '!/^MW_TRUSTED_PROXIES=/ && !/^MW_FORWARDED_MODE=/' "$WORK/app.env" > "$WORK/app.clean"
        mv "$WORK/app.clean" "$WORK/app.env"
    fi
    validate_app_env "$WORK/app.env"
    DATA_VOLUME="${PROJECT}_restore_$(date -u +%Y%m%dT%H%M%SZ)_$$_${RANDOM}"
    ensure_proxy_network
    render "$WORK"
    compose "$WORK" config --quiet
    # Extract only into a fresh Docker volume. No host tar extraction or volume deletion.
    docker volume create --label "mailwoman.deploy.project=$PROJECT" "$DATA_VOLUME" >/dev/null
    HELPER_CONTAINER=$(MSYS_NO_PATHCONV=1 docker create --network none --mount "type=volume,src=$DATA_VOLUME,dst=/data" "$IMAGE_ID")
    docker cp -a - "$HELPER_CONTAINER:/data" < "$from/data.tar"
    docker rm "$HELPER_CONTAINER" >/dev/null
    HELPER_CONTAINER=
    if [[ $how == save ]]; then
        if [[ -f $ACTIVE/release.env ]]; then
            snapshot keep-stopped
            log "Saved the replaced installation at $LAST_BACKUP"
            compose "$ACTIVE" stop --timeout 60
        fi
        cp "$WORK/deploy.env" "$STATE/deploy.env"
        cp "$WORK/app.env" "$STATE/app.env"
        if [[ $EXPOSURE == caddy ]]; then
            awk '!/^MW_TRUSTED_PROXIES=/ && !/^MW_FORWARDED_MODE=/' "$STATE/app.env" > "$WORK/app.clean"
            mv "$WORK/app.clean" "$STATE/app.env"
        fi
    fi
    activate "$WORK"
}

# An update that did not become healthy: keep its files and logs for diagnosis,
# stop it, and start the release saved just before it (image, configuration,
# exposure and data as they were) on a copy of that data. Whatever the failed
# release wrote, migrations included, stays in its own volume, which is kept.
auto_rollback() {
    local failed failed_volume=$DATA_VOLUME saved=$LAST_BACKUP
    failed="$STATE/failed/$(date -u +%Y%m%dT%H%M%SZ)-$$"
    mkdir -p "$failed"
    cp "$WORK/"{compose.yaml,app.env,deploy.env,release.env} "$failed/"
    compose "$ACTIVE" logs --no-color --tail 300 > "$failed/logs.txt" 2>&1 || true
    log "The update did not pass its health check; restoring the release saved in $saved"
    log "Should this rollback fail too, recover with: bash deploy.sh restore $saved --state-dir $STATE"
    compose "$ACTIVE" stop --timeout 60
    rm -rf -- "$WORK"
    WORK=
    restore_snapshot "$saved" replace-failed
    record_snapshot pre-deploy "$saved"
    die "The update was rolled back. The previous release (exposure $EXPOSURE, image $IMAGE_ID) runs on a copy of the data saved in $saved, in volume $DATA_VOLUME. The failed release's files and logs are in $failed and its data volume $failed_volume is kept; deploy.env and app.env still hold the settings that failed"
}

# A snapshot directory name: UTC time, process ID, random number.
SNAPSHOT_NAME='^[0-9]{8}T[0-9]{6}Z-[0-9]+-[0-9]+$'

# days_ago_stamp DAYS: the snapshot-name timestamp of DAYS days ago (GNU or BSD date).
days_ago_stamp() {
    local epoch=$(($(date -u +%s) - $1 * 86400))
    date -u -d "@$epoch" +%Y%m%dT%H%M%SZ 2>/dev/null || date -u -r "$epoch" +%Y%m%dT%H%M%SZ
}

# human_size BYTES
human_size() { awk -v b="$1" 'BEGIN { split("B KB MB GB TB", u); i = 1; while (b >= 1000 && i < 5) { b /= 1000; i++ } printf (i == 1 ? "%d %s" : "%.1f %s"), b, u[i] }'; }

# prune_snapshots: apply BACKUP_KEEP (default 7) and BACKUP_KEEP_DAYS to the
# snapshots in $STATE/backups, newest first. A complete snapshot is removed only
# when it is outside the newest BACKUP_KEEP and, if BACKUP_KEEP_DAYS is set, older
# than that. The snapshot named in rollback.path is always kept; an incomplete one
# is removed only after seven days; anything else in the directory is left alone.
prune_snapshots() {
    local keep=${BACKUP_KEEP:-7} protected='' entry name archive stamp position=0 kept=0 removed=0 old_stamp='' stale_stamp
    [[ -d $STATE/backups && ! -L $STATE/backups ]] || { log "No snapshots in $STATE/backups"; return 0; }
    if [[ -f $STATE/rollback.path ]]; then IFS= read -r protected < "$STATE/rollback.path"; fi
    stale_stamp=$(days_ago_stamp 7)
    [[ -z $BACKUP_KEEP_DAYS ]] || old_stamp=$(days_ago_stamp "$BACKUP_KEEP_DAYS")
    while IFS= read -r entry; do
        name=${entry##*/}
        archive=false
        if [[ $name == *.tar.age || $name == *.tar.gpg ]]; then name=${name%.tar.*}; archive=true; fi
        [[ $name =~ $SNAPSHOT_NAME && ! -L $entry ]] || continue
        if $archive; then [[ -f $entry ]] || continue; else [[ -d $entry ]] || continue; fi
        stamp=${name:0:16}
        if ! $archive && [[ ! -f $entry/COMPLETE ]]; then
            if [[ $stamp < $stale_stamp ]]; then
                log "Removing incomplete snapshot $name (older than seven days)"
                rm -rf -- "$entry"
                removed=$((removed + 1))
            fi
            continue
        fi
        position=$((position + 1))
        if ((position <= keep)) || [[ -n $old_stamp && ! $stamp < $old_stamp ]]; then
            kept=$((kept + 1))
        elif [[ $entry == "$protected" ]]; then
            log "Keeping $name: it is the snapshot rollback restores"
            kept=$((kept + 1))
        else
            log "Removing snapshot ${entry##*/}"
            rm -rf -- "$entry"
            if $archive; then rm -f -- "$STATE/backups/$name.release.env"; fi
            removed=$((removed + 1))
        fi
    done < <(find "$STATE/backups" -mindepth 1 -maxdepth 1 -print | LC_ALL=C sort -r)
    log "Snapshots: $kept kept, $removed removed (BACKUP_KEEP=$keep${BACKUP_KEEP_DAYS:+, BACKUP_KEEP_DAYS=$BACKUP_KEEP_DAYS})"
}

# prune_replaced: list the data volumes of this project other than the active one,
# and the images deploy.sh built for it that neither the active release nor a kept
# snapshot uses, with their sizes. With --replaced --yes, delete them, checking
# each again first; the active volume and a volume a container uses are never
# deleted.
prune_replaced() {
    local volume ref id size sizes='' protected="|$IMAGE_ID|" file line
    local -a volumes=() images=()
    while IFS= read -r volume; do
        [[ -n $volume && $volume != "$DATA_VOLUME" ]] || continue
        if [[ -n $(docker ps -aq --filter "volume=$volume") ]]; then
            log "Data volume $volume is used by a container; not listed"
            continue
        fi
        volumes+=("$volume")
    done < <(docker volume ls -q --filter "label=mailwoman.deploy.project=$PROJECT")
    # A kept snapshot can be restored only with its image: those images stay.
    for file in "$STATE"/backups/*/release.env "$STATE"/backups/*.release.env; do
        [[ -f $file ]] || continue
        while IFS= read -r line; do
            [[ $line == IMAGE_ID=* ]] && protected+="${line#IMAGE_ID=}|"
        done < "$file"
    done
    while IFS= read -r ref; do
        [[ -n $ref ]] || continue
        id=$(docker image inspect --format '{{.Id}}' "$ref")
        if [[ $protected == *"|$id|"* ]]; then continue; fi
        images+=("$ref")
    done < <(docker image ls --format '{{.Repository}}:{{.Tag}}' --filter "reference=mailwoman-deploy:${PROJECT}-*")
    if ((${#volumes[@]} + ${#images[@]} == 0)); then
        log "No replaced data volumes or old built images"
        return 0
    fi
    if ((${#volumes[@]})); then
        sizes=$(docker system df -v --format '{{range .Volumes}}{{.Name}} {{.Size}}
{{end}}')
    fi
    for volume in "${volumes[@]}"; do
        size=$(awk -v name="$volume" '$1 == name {print $2; exit}' <<< "$sizes")
        log "Replaced data volume: $volume (${size:-size unknown})"
    done
    for ref in "${images[@]}"; do
        log "Old built image: $ref ($(human_size "$(docker image inspect --format '{{.Size}}' "$ref")"))"
    done
    if ! $PRUNE_REPLACED; then
        log "Nothing above was deleted. 'prune --replaced --yes' deletes them; save an image first (docker image save) if an old snapshot may need it"
        return 0
    fi
    if ! $ASSUME_YES; then
        log "Nothing was deleted: add --yes to delete the volumes and images listed above"
        return 0
    fi
    for volume in "${volumes[@]}"; do
        if [[ $volume == "$DATA_VOLUME" || -n $(docker ps -aq --filter "volume=$volume") ]] ||
           [[ $(docker volume inspect --format '{{index .Labels "mailwoman.deploy.project"}}' "$volume" 2>/dev/null) != "$PROJECT" ]]; then
            log "Not deleting $volume: it changed since it was listed"
            continue
        fi
        docker volume rm "$volume" >/dev/null
        log "Deleted data volume $volume"
    done
    for ref in "${images[@]}"; do
        id=$(docker image inspect --format '{{.Id}}' "$ref" 2>/dev/null) || continue
        if [[ $protected == *"|$id|"* ]]; then continue; fi
        docker image rm "$ref" >/dev/null
        log "Deleted image $ref"
    done
}

# write_timers: systemd service and timer units that run backup (daily) and check
# (every five minutes) for this installation, written to $STATE/systemd and
# printed with the equivalent cron lines. Nothing outside $STATE is written; the
# operator installs them.
write_timers() {
    local self bash_path user directory what name description calendar delay persistent file path
    self="$ROOT/${BASH_SOURCE[0]##*/}"
    bash_path=$(command -v bash)
    for path in "$self" "$STATE" "$bash_path"; do
        # No quoting: systemd and cron would need different escaping.
        [[ $path =~ ^/[a-zA-Z0-9._/+@-]+$ ]] || die "timers needs paths of letters, digits and ._/+@- only; this one is not: $path"
    done
    user=$(id -un)
    directory="$STATE/systemd"
    mkdir -p "$directory"
    for what in backup check; do
        name="mailwoman-$PROJECT-$what"
        if [[ $what == backup ]]; then
            description="Back up the Mailwoman installation $PROJECT"; calendar='*-*-* 03:30:00'; delay=30min; persistent=true
        else
            description="Check the health of the Mailwoman installation $PROJECT"; calendar='*:0/5'; delay=30s; persistent=false
        fi
        {
            printf '# Generated by deploy.sh timers for %s.\n[Unit]\nDescription=%s\n' "$STATE" "$description"
            printf 'Wants=network-online.target\nAfter=network-online.target docker.service\n\n[Service]\nType=oneshot\n'
            [[ $user == root ]] || printf 'User=%s\n' "$user"
            printf 'ExecStart=%s %s %s --state-dir %s\n' "$bash_path" "$self" "$what" "$STATE"
        } > "$directory/$name.service"
        printf '# Generated by deploy.sh timers for %s.\n[Unit]\nDescription=%s (schedule)\n\n[Timer]\nOnCalendar=%s\nRandomizedDelaySec=%s\nPersistent=%s\n\n[Install]\nWantedBy=timers.target\n' \
            "$STATE" "$description" "$calendar" "$delay" "$persistent" > "$directory/$name.timer"
    done
    for file in "$directory/mailwoman-$PROJECT-"{backup,check}.{service,timer}; do
        printf '# ---- %s\n' "$file"
        cat -- "$file"
        printf '\n'
    done
    cat <<EOF
# ---- Install as root:
#   cp $directory/mailwoman-$PROJECT-*.service $directory/mailwoman-$PROJECT-*.timer /etc/systemd/system/
#   systemctl daemon-reload
#   systemctl enable --now mailwoman-$PROJECT-backup.timer mailwoman-$PROJECT-check.timer
# ---- Or with cron instead (crontab -e as $user):
30 3 * * * $bash_path $self backup --state-dir $STATE
*/5 * * * * $bash_path $self check --state-dir $STATE
EOF
    log "Wrote the units to $directory; nothing was installed"
}

# check_failed NAME MESSAGE: report a failed check, run ALERT_HOOK NAME MESSAGE
# when it is set, and exit 1.
check_failed() {
    local status=0
    log "CHECK FAILED ($1): $2"
    if [[ -n $ALERT_HOOK ]]; then
        if [[ -f $ALERT_HOOK && -x $ALERT_HOOK ]]; then
            "$ALERT_HOOK" "$1" "$2" < /dev/null || status=$?
            ((status == 0)) || log "ALERT_HOOK $ALERT_HOOK exited $status"
        else
            log "ALERT_HOOK $ALERT_HOOK is not an executable file; no alert was sent"
        fi
    fi
    exit 1
}

# run_check: the app container is running and healthy, and /healthz answers at the
# public URL, or by name on the proxy network for network exposure (whose public
# URL is served by a proxy this script does not manage).
run_check() {
    local cid health
    cid=$(compose "$ACTIVE" ps -q mailwoman 2>/dev/null) || cid=
    [[ -n $cid ]] || check_failed container "the app container of $PROJECT is not running"
    health=$(docker inspect --format '{{if .State.Health}}{{.State.Health.Status}}{{else}}without a health check{{end}}' "$cid" 2>/dev/null) || health=unknown
    [[ $health == healthy ]] || check_failed container "the app container of $PROJECT is $health, not healthy"
    if [[ $EXPOSURE == network ]]; then
        MSYS_NO_PATHCONV=1 docker run --rm --network "$PROXY_NETWORK" "$IMAGE_ID" healthcheck --url "http://$PROJECT:8080/healthz" >/dev/null 2>&1 ||
            check_failed proxy-network "http://$PROJECT:8080/healthz does not answer on the Docker network $PROXY_NETWORK"
        log "OK: $PROJECT is healthy and answers on the Docker network $PROXY_NETWORK"
    else
        curl --fail --silent --connect-timeout 5 --max-time 10 "$PUBLIC_URL/healthz" -o /dev/null 2>/dev/null ||
            check_failed public-url "$PUBLIC_URL/healthz does not answer"
        log "OK: $PROJECT is healthy and $PUBLIC_URL/healthz answers"
    fi
}

# metrics_state: whether the applied app.env sets MW_METRICS_TOKEN, and whether
# app.env says otherwise since.
metrics_state() {
    local applied edited
    applied=$(awk -F= '$1 == "MW_METRICS_TOKEN" && $2 != "" {print "on"}' "$ACTIVE/app.env")
    edited=$(awk -F= '$1 == "MW_METRICS_TOKEN" && $2 != "" {print "on"}' "$STATE/app.env")
    if [[ -n $applied ]]; then
        printf 'enabled at %s/metrics (bearer token: MW_METRICS_TOKEN in %s/app.env)' "$PUBLIC_URL" "$STATE"
    else
        printf 'off (no MW_METRICS_TOKEN in the applied app.env)'
    fi
    if [[ $applied != "$edited" ]]; then printf '; app.env has been edited since, run deploy --no-build to apply it'; fi
}

preflight
case "$ACTION" in
    doctor)
        if [[ -f $ACTIVE/release.env ]]; then read_config "$ACTIVE/release.env"; fi
        # Compose config validates references without needing to build/pull images.
        IMAGE_ID=${IMAGE_ID:-sha256:0000000000000000000000000000000000000000000000000000000000000000}
        CADDY_ID=${CADDY_ID:-sha256:0000000000000000000000000000000000000000000000000000000000000000}
        DATA_VOLUME=${DATA_VOLUME:-${PROJECT}_data}
        WORK=$(mktemp -d "$STATE/.work.XXXXXX")
        cp "$STATE/app.env" "$WORK/app.env"
        render "$WORK"
        compose "$WORK" config --quiet
        if [[ -n $BACKUP_RECIPIENT ]]; then
            tool=$(encryption_tool)
            command -v "$tool" >/dev/null 2>&1 || die "BACKUP_RECIPIENT needs $tool, which is not installed"
            if [[ $tool == gpg ]]; then
                gpg --batch --list-keys "$BACKUP_RECIPIENT" >/dev/null 2>&1 || die "BACKUP_RECIPIENT $BACKUP_RECIPIENT is not in this user's gpg keyring; import its public key"
                if gpg --batch --list-secret-keys "$BACKUP_RECIPIENT" >/dev/null 2>&1; then
                    log "Note: this host holds the private key for $BACKUP_RECIPIENT, so anyone with this account can decrypt the backups; keep only the public key here"
                fi
            fi
            log "Backups are encrypted with $tool to $BACKUP_RECIPIENT"
        else
            log "Backups are not encrypted (BACKUP_RECIPIENT is empty; age $(command -v age >/dev/null 2>&1 && printf 'is' || printf 'is not') installed, gpg $(command -v gpg >/dev/null 2>&1 && printf 'is' || printf 'is not'))"
        fi
        if [[ -n $ALERT_HOOK ]]; then
            [[ -f $ALERT_HOOK && -x $ALERT_HOOK ]] || die "ALERT_HOOK $ALERT_HOOK is not an executable file"
            log "A failed check runs ALERT_HOOK: $ALERT_HOOK <check> <message>"
        fi
        if [[ -n $BACKUP_HOOK ]]; then
            [[ -f $BACKUP_HOOK && -x $BACKUP_HOOK ]] || die "BACKUP_HOOK $BACKUP_HOOK is not an executable file"
            log "After each backup, BACKUP_HOOK runs: $BACKUP_HOOK <backup path>"
        fi
        if [[ -n $BACKUP_TARGETS_FILE ]]; then
            [[ -n $BACKUP_RECIPIENT ]] || die "BACKUP_TARGETS_FILE requires BACKUP_RECIPIENT encryption"
            command -v python3 >/dev/null || die "python3 is required for configured backup destinations"
            python3 "$ROOT/scripts/backup-destinations.py" check --config "$BACKUP_TARGETS_FILE"
            log "Backup destination configuration checked; connectivity is verified when copying"
        fi
        log "Prerequisites and configuration OK. App mode: $MODE; URL: $PUBLIC_URL" ;;
    status)
        load_active
        compose "$ACTIVE" ps -a
        log "URL: $PUBLIC_URL; data volume: $DATA_VOLUME${PROXY_NETWORK:+; proxy network: $PROXY_NETWORK (http://$PROJECT:8080)}"
        log "Metrics: $(metrics_state)"
        ;;
    check) load_active; run_check ;;
    logs) load_active; compose "$ACTIVE" logs --tail 100 --follow ;;
    stop) load_active; compose "$ACTIVE" stop --timeout 60; log "Stopped; data retained" ;;
    start) load_active; require_free_ports; ensure_proxy_network; compose "$ACTIVE" up -d --wait --wait-timeout "$TIMEOUT"; verify ;;
    restart) load_active; require_free_ports; ensure_proxy_network; compose "$ACTIVE" restart; compose "$ACTIVE" up -d --no-recreate --wait --wait-timeout "$TIMEOUT"; verify ;;
    backup)
        load_active
        if [[ -n $BACKUP_TARGETS_FILE ]]; then
            [[ -n $BACKUP_RECIPIENT ]] || die "BACKUP_TARGETS_FILE requires BACKUP_RECIPIENT encryption"
            command -v python3 >/dev/null || die "python3 is required for configured backup destinations"
            python3 "$ROOT/scripts/backup-destinations.py" check --config "$BACKUP_TARGETS_FILE"
        fi
        snapshot
        if [[ -n $BACKUP_RECIPIENT ]]; then encrypt_snapshot "$LAST_BACKUP"; fi
        record_snapshot backup "$LAST_BACKUP"
        if [[ -n $BACKUP_TARGETS_FILE ]]; then
            python3 "$ROOT/scripts/backup-destinations.py" publish --config "$BACKUP_TARGETS_FILE" --source "$LAST_BACKUP" || die "Destination copy failed; local backup retained and retention skipped"
        fi
        if [[ -n $BACKUP_HOOK ]]; then run_backup_hook; fi
        # Retention runs after a backup only when deploy.env asks for it.
        if [[ -n $BACKUP_KEEP$BACKUP_KEEP_DAYS ]]; then prune_snapshots; fi
        ;;
    timers) write_timers ;;
    prune)
        prune_snapshots
        if [[ -f $ACTIVE/release.env ]]; then load_active; prune_replaced; fi
        ;;
    deploy)
        WORK=$(mktemp -d "$STATE/.work.XXXXXX")
        if [[ -f $ACTIVE/release.env ]]; then
            read_config "$ACTIVE/release.env"
            # Project and Caddy subnet cannot change in place; create a new state
            # directory for another install. Otherwise Compose could orphan its data.
            # local, external and network render the same single service and differ
            # only in its ports, networks and environment, so those three may change.
            configured_project=$PROJECT; configured_exposure=$EXPOSURE; configured_prefix=$PROXY_PREFIX
            read_config "$ACTIVE/deploy.env"
            applied_exposure=$EXPOSURE
            [[ $PROJECT == "$configured_project" && $PROXY_PREFIX == "$configured_prefix" ]] || die "Project and proxy subnet are fixed after the first deployment; use a new state directory"
            if [[ $applied_exposure != "$configured_exposure" && ($applied_exposure == caddy || $configured_exposure == caddy) ]]; then
                die "EXPOSURE cannot change from $applied_exposure to $configured_exposure: an installation cannot move to or from caddy. local, external and network may be exchanged; use a new state directory for caddy"
            fi
            read_config "$STATE/deploy.env"
            validate_config
            [[ $EXPOSURE == "$applied_exposure" ]] || log "Changing exposure from $applied_exposure to $EXPOSURE"
            old_key=$(awk -F= '$1 == "MW_SERVER_KEY" {print $2}' "$ACTIVE/app.env")
            new_key=$(awk -F= '$1 == "MW_SERVER_KEY" {print $2}' "$STATE/app.env")
            [[ $old_key == "$new_key" ]] || die "MW_SERVER_KEY changed; retain the original key to keep stored credentials and uploads readable"
        else
            $NO_BUILD && die "--no-build requires an existing deployment"
            DATA_VOLUME=${PROJECT}_data
            if docker volume inspect "$DATA_VOLUME" >/dev/null 2>&1; then
                die "Volume $DATA_VOLUME already exists without this deployment's state; restore its snapshot or choose another project"
            fi
            if [[ -n $(docker ps -aq --filter "label=com.docker.compose.project=$PROJECT") ]]; then
                die "Compose project $PROJECT already exists; use its original state directory or choose another project"
            fi
        fi
        unlimited=
        [[ -n $MEMORY_LIMIT ]] || unlimited=memory
        [[ -n $CPU_LIMIT ]] || unlimited+="${unlimited:+ or }CPU"
        [[ -z $unlimited ]] || log "No $unlimited limit for the app: MEMORY_LIMIT and CPU_LIMIT in $STATE/deploy.env set them (new installations get 1g and 2)"
        require_free_ports
        phase "Prepare runtime image"
        built=false
        if ! $NO_BUILD; then
            if [[ -n $IMAGE ]]; then
                from_registry=false
                if $PULL || ! docker image inspect "$IMAGE" >/dev/null 2>&1; then from_registry=true; fi
                check_signature "$from_registry"
                if $from_registry; then docker pull "$IMAGE"; fi
            else
                IMAGE="mailwoman-deploy:${PROJECT}-$(date -u +%Y%m%dT%H%M%SZ)-$$"
                log "Building web UI, server, render worker and bundled plugins"
                # runtime-static is the target CI scans, signs and publishes
                # (.github/workflows/supply-chain.yml), so a built installation runs
                # the same shape as the published image.
                log "Rust build: optimized release, incremental=$BUILD_INCREMENTAL, jobs=${BUILD_JOBS:-automatic}; cache retained between builds"
                docker build --target runtime-static \
                    --build-arg "MW_CARGO_INCREMENTAL=$BUILD_INCREMENTAL" \
                    --build-arg "MW_BUILD_JOBS=$BUILD_JOBS" --tag "$IMAGE" "$ROOT"
                built=true
            fi
            IMAGE_ID=$(docker image inspect --format '{{.Id}}' "$IMAGE")
            # MW_IMAGE_DIGEST: the registry digest an --image has for its own
            # repository, else the local image ID (a built image has no registry).
            IMAGE_DIGEST=
            if ! $built; then image_digest; fi
            IMAGE_DIGEST=${IMAGE_DIGEST:-$IMAGE_ID}
        fi
        if [[ $EXPOSURE == caddy ]]; then
            if ! docker image inspect "$CADDY_IMAGE" >/dev/null 2>&1; then docker pull "$CADDY_IMAGE"; fi
            CADDY_ID=$(docker image inspect --format '{{.Id}}' "$CADDY_IMAGE")
        fi
        ensure_proxy_network
        cp "$STATE/app.env" "$WORK/app.env"
        render "$WORK"
        compose "$WORK" config --quiet
        previous_rollback=
        if [[ -f $STATE/rollback.path ]]; then IFS= read -r previous_rollback < "$STATE/rollback.path"; fi
        phase "Protect installation data"
        if [[ -f $ACTIVE/release.env ]]; then
            snapshot
            printf '%s\n' "$LAST_BACKUP" > "$STATE/rollback.path"
        else
            log "First installation: creating the data volume; no previous snapshot needed"
            docker volume create --label "mailwoman.deploy.project=$PROJECT" "$DATA_VOLUME" >/dev/null
            AUTO_ROLLBACK=false
        fi
        phase "Start services and verify health"
        if $AUTO_ROLLBACK; then
            # activate runs in a subshell so that a failed start or check returns
            # here instead of ending the script.
            set +e
            (set -e; activate "$WORK")
            activated=$?
            set -e
            ((activated == 0)) || auto_rollback
        else
            activate "$WORK"
        fi
        phase "Finalize backup history"
        if [[ -n $LAST_BACKUP ]]; then record_snapshot pre-deploy "$LAST_BACKUP"; fi
        # With encrypted backups, only one clear-text snapshot stays on the host: the
        # one this deploy took for rollback. The one the previous deploy took goes.
        if [[ -n $BACKUP_RECIPIENT && -n $previous_rollback && $previous_rollback != "$LAST_BACKUP" &&
              $previous_rollback == "$STATE"/backups/* && -d $previous_rollback && ! -L $previous_rollback &&
              ${previous_rollback##*/} =~ $SNAPSHOT_NAME ]]; then
            rm -rf -- "$previous_rollback"
            log "Removed the previous pre-deploy snapshot $previous_rollback (clear text; BACKUP_RECIPIENT is set)"
        fi
        progress_finish
        ;;
    restore|rollback)
        if [[ $ACTION == rollback ]]; then
            [[ -f $STATE/rollback.path ]] || die "No pre-deploy snapshot recorded"
            IFS= read -r SNAPSHOT < "$STATE/rollback.path"
        fi
        restored=$SNAPSHOT
        if [[ $ACTION == restore && ($SNAPSHOT == *.tar.age || $SNAPSHOT == *.tar.gpg) ]]; then decrypt_snapshot "$SNAPSHOT"; fi
        restore_snapshot "$SNAPSHOT" save
        if [[ -n $LAST_BACKUP ]]; then record_snapshot pre-restore "$LAST_BACKUP"; fi
        log "Restored snapshot $restored; replaced volumes and backups remain available"
        ;;
esac
