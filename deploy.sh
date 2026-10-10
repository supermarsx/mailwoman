#!/usr/bin/env bash
# Mailwoman single-host deployment and recovery. Run `bash deploy.sh --help`.
# Requires Bash 4+, Docker with Linux containers, Compose 2.30+, curl and tar.
# Configuration files are parsed as data, never sourced as shell scripts.
set -Eeuo pipefail
umask 077

ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
STATE="$ROOT/.deploy.local"
ACTION=deploy
ACTION_SET=false
PROJECT=mailwoman
EXPOSURE=local
DOMAIN=
PUBLIC_URL=
PORT=8080
MODE=proxy
PROXY_PREFIX=172.30.70
IMAGE_ID=
DATA_VOLUME=
CADDY_IMAGE=caddy:2-alpine
CADDY_ID=
IMAGE=
PULL=false
NO_BUILD=false
DRY_RUN=false
TIMEOUT=180
SNAPSHOT=
INIT_OPTIONS=false
LOCKED=false
RESUME_CONTAINER=
HELPER_CONTAINER=
WORK=
LAST_BACKUP=

usage() {
    cat <<'HELP'
Usage: bash deploy.sh [command] [options]

Commands (default: deploy):
  init              Generate private, editable configuration; do not start Docker
  deploy            Build/pull, back up an existing install, start, verify health
  doctor            Check prerequisites, configuration, and Compose syntax
  status            Show containers and the configured public URL
  logs              Follow the last 100 lines of service logs (Ctrl-C to leave)
  stop              Stop services; retain all containers, volumes, and backups
  start             Start the last applied configuration and verify health
  restart           Restart the last applied configuration and verify health
  backup            Stop the app briefly; snapshot data, uploads, config and key
  restore PATH      Restore a snapshot into a NEW volume, then verify health
  rollback          Restore the snapshot taken immediately before the last deploy

Options:
  --state-dir PATH   Deployment files/backups (default: <repo>/.deploy.local)
  --project NAME     Isolated Compose project name (init/first deploy only)
  --domain HOST     Publish through Caddy with automatic HTTPS (init only)
  --public-url URL  Existing HTTPS reverse proxy; bind app to localhost (init only)
  --port NUMBER     Localhost HTTP port, default 8080 (init only)
  --mode MODE       proxy for JMAP; engine for IMAP/POP3 + SMTP (init only)
  --proxy-prefix IP First 3 octets of a private /24 for Caddy, e.g. 172.30.71
                    (init only; default 172.30.70; choose an unused subnet)
  --image REF       Use a ready-built Mailwoman image instead of building source
  --pull            Refresh --image from its registry before deploying
  --no-build        Reuse the last applied app image (configuration-only deploy)
  --timeout SECS    Health/startup timeout, 10..3600 (default 180)
  --dry-run         Describe the operation without writing or contacting Docker
  -h, --help        Show this help

Examples:
  bash deploy.sh --domain mail.example.org --mode engine
  bash deploy.sh init --public-url https://mail.example.org
  # Edit .deploy.local/app.env and deploy.env, then:
  bash deploy.sh deploy
  bash deploy.sh backup
  bash deploy.sh restore .deploy.local/backups/20261006T120000Z-1234

No domain means a local HTTP installation at http://localhost:8080.
For public HTTPS, point DNS to this host and allow TCP ports 80 and 443.
The script deploys the webmail app; mailbox/SMTP servers remain external.
Restore/rollback reverts data to the snapshot time (including configuration).
The replaced volume is retained. Backups contain secrets: store them privately.
See docs/deploy/script.md for operations, recovery, and optional integrations.
HELP
}

log() { printf '[deploy] %s\n' "$*" >&2; }
die() { log "ERROR: $*"; exit 1; }
need_value() { [[ $# -ge 2 && -n $2 && $2 != --* ]] || die "$1 needs a value"; }
need() { command -v "$1" >/dev/null 2>&1 || die "Required command missing: $1"; }

while (($#)); do
    case "$1" in
        init|deploy|doctor|status|logs|stop|start|restart|backup|rollback|restore)
            $ACTION_SET && die "Specify only one command"
            ACTION=$1; ACTION_SET=true
            if [[ $ACTION == restore ]]; then need_value "$@"; SNAPSHOT=$2; shift; fi ;;
        --state-dir) need_value "$@"; STATE=$2; shift ;;
        --project) need_value "$@"; PROJECT=$2; INIT_OPTIONS=true; shift ;;
        --domain) need_value "$@"; DOMAIN=$2; INIT_OPTIONS=true; shift ;;
        --public-url) need_value "$@"; PUBLIC_URL=$2; INIT_OPTIONS=true; shift ;;
        --port) need_value "$@"; PORT=$2; INIT_OPTIONS=true; shift ;;
        --mode) need_value "$@"; MODE=$2; INIT_OPTIONS=true; shift ;;
        --proxy-prefix) need_value "$@"; PROXY_PREFIX=$2; INIT_OPTIONS=true; shift ;;
        --image) need_value "$@"; IMAGE=$2; shift ;;
        --timeout) need_value "$@"; TIMEOUT=$2; shift ;;
        --pull) PULL=true ;;
        --no-build) NO_BUILD=true ;;
        --dry-run) DRY_RUN=true ;;
        -h|--help) usage; exit 0 ;;
        *) die "Unknown argument: $1 (see --help)" ;;
    esac
    shift
done

[[ ${BASH_VERSINFO[0]} -ge 4 ]] || die "Bash 4 or newer is required"
if [[ ! $TIMEOUT =~ ^[0-9]{2,4}$ ]] || ((10#$TIMEOUT < 10 || 10#$TIMEOUT > 3600)); then die "Invalid timeout"; fi
TIMEOUT=$((10#$TIMEOUT))
[[ -z $IMAGE || $IMAGE != -* && $IMAGE != *[[:space:]]* ]] || die "Invalid image reference"
if $NO_BUILD && { [[ -n $IMAGE ]] || $PULL; }; then die "--no-build cannot be combined with --image/--pull"; fi
if $PULL && [[ -z $IMAGE ]]; then die "--pull requires --image"; fi
if [[ $ACTION != deploy ]] && { [[ -n $IMAGE ]] || $NO_BUILD || $PULL; }; then
    die "Image options apply only to deploy"
fi
if $INIT_OPTIONS && [[ $ACTION != init && $ACTION != deploy ]]; then die "Setup options apply only to init/first deploy"; fi

read_config() {
    local file=$1 line key value seen='|'
    [[ -f $file && ! -L $file ]] || die "Missing or symlinked configuration: $file"
    while IFS= read -r line || [[ -n $line ]]; do
        [[ -z $line || $line == \#* ]] && continue
        [[ $line != *$'\r'* && $line == *=* ]] || die "Invalid line in $file; use unquoted KEY=value with LF endings"
        key=${line%%=*}; value=${line#*=}
        [[ $seen != *"|$key|"* ]] || die "Duplicate $key in $file"
        seen+="$key|"
        case "$key" in
            PROJECT|EXPOSURE|DOMAIN|PUBLIC_URL|PORT|MODE|PROXY_PREFIX|CADDY_IMAGE|IMAGE_ID|DATA_VOLUME|CADDY_ID)
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
        external) [[ -z $DOMAIN && $PUBLIC_URL =~ ^https://[a-zA-Z0-9][a-zA-Z0-9.-]*(:[0-9]{1,5})?$ ]] || die "Public URL must be an HTTPS origin without a trailing slash/path" ;;
        *) die "EXPOSURE must be local, caddy or external" ;;
    esac
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
CADDY_IMAGE=$CADDY_IMAGE
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
            MW_BIND|MW_DB_PATH|MW_UPLOAD_DIR|MW_MODE|MW_PUBLIC_URL|MW_COOKIE_SECURE|MW_RENDER_BIN|MW_PLUGIN_DIR|MW_WEB_DIR|MW_BASE_PATH|MW_ACME*|MW_TLS_*|MW_PROXY_PROTOCOL)
                die "$key is managed by deploy.sh; see docs/deploy/script.md" ;;
            MW_TRUSTED_PROXIES|MW_FORWARDED_MODE)
                [[ $EXPOSURE != caddy ]] || die "$key is managed for Caddy; remove it from app.env" ;;
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
    if [[ -n $WORK && -d $WORK && $WORK == "$STATE"/.work.* ]]; then
        # WORK is always a private mktemp child of STATE; never remove arbitrary input.
        rm -rf -- "$WORK"
    fi
    if $LOCKED; then rmdir -- "$STATE/.lock" 2>/dev/null || true; fi
    if ((result != 0)); then log "Command failed. Data volumes have been retained. See 'status' and 'logs'."; fi
    exit "$result"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

# Dry-run must work even on a machine without Docker and must not mkdir/chmod.
if [[ -f $STATE/deploy.env ]]; then
    $INIT_OPTIONS && die "Configuration exists; edit $STATE/deploy.env instead of passing setup options"
    read_config "$STATE/deploy.env"
else
    [[ -z $DOMAIN || -z $PUBLIC_URL ]] || die "Choose --domain OR --public-url"
    if [[ -n $DOMAIN ]]; then EXPOSURE=caddy; PUBLIC_URL=https://$DOMAIN
    elif [[ -n $PUBLIC_URL ]]; then EXPOSURE=external
    else PUBLIC_URL=http://localhost:$PORT; fi
fi
validate_config
if $DRY_RUN; then
    log "Dry run: $ACTION; project=$PROJECT; mode=$MODE; exposure=$EXPOSURE"
    log "State: $STATE; URL: $PUBLIC_URL"
    case "$ACTION" in
        deploy) log "Prepare runtime image (${IMAGE:-$($NO_BUILD && printf 'last applied image' || printf 'build Dockerfile --target runtime')}); snapshot existing data; apply Compose; verify app and public HTTP(S)." ;;
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
if [[ $ACTION != status && $ACTION != logs && $ACTION != doctor ]]; then
    chmod 700 "$STATE"
    mkdir "$STATE/.lock" 2>/dev/null || die "Another deployment operation holds $STATE/.lock; remove this empty lock only after verifying no operation is running"
    LOCKED=true
fi

initialize() {
    [[ ! -e $STATE/deploy.env && ! -e $STATE/app.env ]] || die "Partial configuration found; retain the existing key and complete both deploy.env and app.env"
    need od
    local key password
    key=$(od -An -N32 -tx1 /dev/urandom | tr -d ' \n')
    password=$(od -An -N24 -tx1 /dev/urandom | tr -d ' \n')
    [[ ${#key} == 64 && ${#password} == 48 ]] || die "Failed to generate secrets"
    write_config > "$STATE/deploy.env"
    cat > "$STATE/app.env" <<EOF
# Raw Docker environment: values are literal; do not quote or escape dollars.
# Preserve MW_SERVER_KEY across upgrades. Back up this file with the data.
MW_SERVER_KEY=$key
MW_ADMIN_USER=admin
MW_ADMIN_PASSWORD=$password
MW_CSRF_STRICT=true
RUST_LOG=info
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
    log "Created $STATE/deploy.env and app.env (admin credentials are in app.env)"
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

load_active() {
    [[ -f $ACTIVE/release.env ]] || die "No applied deployment. Run deploy first"
    read_config "$ACTIVE/deploy.env"
    read_config "$ACTIVE/release.env"
    validate_config
}

render() {
    local directory=$1 secure=true
    [[ $EXPOSURE != local ]] || secure=false
    [[ $IMAGE_ID =~ ^sha256:[0-9a-f]{64}$ ]] || die "Invalid image ID"
    [[ $DATA_VOLUME =~ ^[a-zA-Z0-9][a-zA-Z0-9_.-]+$ ]] || die "Invalid volume name"
    write_config > "$directory/deploy.env"
    printf 'IMAGE_ID=%s\nDATA_VOLUME=%s\nCADDY_ID=%s\n' "$IMAGE_ID" "$DATA_VOLUME" "$CADDY_ID" > "$directory/release.env"
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
    pids_limit: 512
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
      MW_COOKIE_SECURE: '$secure'
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
    ports: ['80:80', '443:443']
    volumes: ['caddy_data:/data', 'caddy_config:/config']
    configs:
      - source: caddyfile
        target: /etc/caddy/Caddyfile
    depends_on:
      mailwoman:
        condition: service_healthy
    networks:
      default:
        ipv4_address: $PROXY_PREFIX.2
    logging:
      driver: json-file
      options: {max-size: 10m, max-file: '3'}
configs:
  caddyfile:
    content: |
      $DOMAIN {
          reverse_proxy mailwoman:8080 {
              flush_interval -1
          }
      }
networks:
  default:
    ipam:
      config:
        - subnet: $PROXY_PREFIX.0/24
EOF
        printf '\nMW_TRUSTED_PROXIES=%s.2\nMW_FORWARDED_MODE=xff\n' "$PROXY_PREFIX" >> "$directory/app.env"
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
    local cid deadline=$((SECONDS + TIMEOUT)) url=$PUBLIC_URL
    cid=$(compose "$ACTIVE" ps -q mailwoman)
    [[ -n $cid ]] || die "App container is missing"
    MSYS_NO_PATHCONV=1 docker exec "$cid" /usr/local/bin/mailwoman healthcheck --url http://127.0.0.1:8080/healthz || die "App health check failed"
    log "Verifying $url/healthz and the web UI (up to ${TIMEOUT}s)"
    while ((SECONDS < deadline)); do
        if curl --fail --silent --show-error --connect-timeout 3 --max-time 5 "$url/healthz" -o /dev/null 2>/dev/null &&
           curl --fail --silent --show-error --connect-timeout 3 --max-time 5 "$url/" -o /dev/null 2>/dev/null; then
            log "Healthy: $url (admin credentials: $STATE/app.env)"
            return 0
        fi
        sleep 2
    done
    die "App is healthy internally, but $url is not reachable with valid HTTP(S). Check DNS, firewall, proxy and service logs; containers remain available."
}

snapshot() {
    local cid running destination
    cid=$(compose "$ACTIVE" ps -a -q mailwoman)
    [[ -n $cid ]] || die "No app container to back up"
    destination="$STATE/backups/$(date -u +%Y%m%dT%H%M%SZ)-$$-${RANDOM}"
    mkdir -p "$destination"
    cp "$ACTIVE/"{compose.yaml,app.env,deploy.env,release.env} "$destination/"
    # Stopping the only app writer makes SQLite + WAL + uploads a consistent set.
    running=$(docker inspect --format '{{.State.Running}}' "$cid")
    if [[ $running == true ]]; then
        RESUME_CONTAINER=$cid
        docker stop --time 60 "$cid" >/dev/null
    fi
    log "Snapshotting /data to $destination"
    docker cp -a "$cid:/data/." - > "$destination/data.tar"
    tar -tf "$destination/data.tar" >/dev/null
    (cd "$destination" && sha256sum data.tar compose.yaml app.env deploy.env release.env > SHA256SUMS)
    # A snapshot is usable only once COMPLETE exists; partial backups remain for diagnosis.
    printf 'mailwoman-deploy-snapshot-v1\n' > "$destination/COMPLETE"
    if [[ -n $RESUME_CONTAINER ]]; then
        docker start "$RESUME_CONTAINER" >/dev/null
        RESUME_CONTAINER=
        compose "$ACTIVE" start --wait --wait-timeout "$TIMEOUT" mailwoman
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

preflight
case "$ACTION" in
    doctor)
        validate_app_env "$STATE/app.env"
        if [[ -f $ACTIVE/release.env ]]; then read_config "$ACTIVE/release.env"; fi
        # Compose config validates references without needing to build/pull images.
        IMAGE_ID=${IMAGE_ID:-sha256:0000000000000000000000000000000000000000000000000000000000000000}
        CADDY_ID=${CADDY_ID:-sha256:0000000000000000000000000000000000000000000000000000000000000000}
        DATA_VOLUME=${DATA_VOLUME:-${PROJECT}_data}
        WORK=$(mktemp -d "$STATE/.work.XXXXXX")
        cp "$STATE/app.env" "$WORK/app.env"
        render "$WORK"
        compose "$WORK" config --quiet
        log "Prerequisites and configuration OK. App mode: $MODE; URL: $PUBLIC_URL" ;;
    status) load_active; compose "$ACTIVE" ps -a; log "URL: $PUBLIC_URL; data volume: $DATA_VOLUME" ;;
    logs) load_active; compose "$ACTIVE" logs --tail 100 --follow ;;
    stop) load_active; compose "$ACTIVE" stop --timeout 60; log "Stopped; data retained" ;;
    start) load_active; compose "$ACTIVE" up -d --wait --wait-timeout "$TIMEOUT"; verify ;;
    restart) load_active; compose "$ACTIVE" restart; compose "$ACTIVE" start --wait --wait-timeout "$TIMEOUT"; verify ;;
    backup) load_active; snapshot ;;
    deploy)
        WORK=$(mktemp -d "$STATE/.work.XXXXXX")
        if [[ -f $ACTIVE/release.env ]]; then
            read_config "$ACTIVE/release.env"
            # Project and network identity cannot change in place; create a new state
            # directory for another install. Otherwise Compose could orphan its data.
            configured_project=$PROJECT; configured_exposure=$EXPOSURE; configured_prefix=$PROXY_PREFIX
            read_config "$ACTIVE/deploy.env"
            [[ $PROJECT == "$configured_project" && $EXPOSURE == "$configured_exposure" && $PROXY_PREFIX == "$configured_prefix" ]] || die "Project, exposure and proxy subnet are immutable; use a new state directory"
            read_config "$STATE/deploy.env"
            validate_config
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
        if ! $NO_BUILD; then
            if [[ -n $IMAGE ]]; then
                if $PULL || ! docker image inspect "$IMAGE" >/dev/null 2>&1; then docker pull "$IMAGE"; fi
            else
                IMAGE="mailwoman-deploy:${PROJECT}-$(date -u +%Y%m%dT%H%M%SZ)-$$"
                log "Building web UI, server, render worker and bundled plugins"
                docker build --target runtime --tag "$IMAGE" "$ROOT"
            fi
            IMAGE_ID=$(docker image inspect --format '{{.Id}}' "$IMAGE")
        fi
        if [[ $EXPOSURE == caddy ]]; then
            if ! docker image inspect "$CADDY_IMAGE" >/dev/null 2>&1; then docker pull "$CADDY_IMAGE"; fi
            CADDY_ID=$(docker image inspect --format '{{.Id}}' "$CADDY_IMAGE")
        fi
        cp "$STATE/app.env" "$WORK/app.env"
        render "$WORK"
        compose "$WORK" config --quiet
        if [[ -f $ACTIVE/release.env ]]; then
            snapshot
            printf '%s\n' "$LAST_BACKUP" > "$STATE/rollback.path"
        else
            docker volume create --label "mailwoman.deploy.project=$PROJECT" "$DATA_VOLUME" >/dev/null
        fi
        activate "$WORK"
        ;;
    restore|rollback)
        if [[ $ACTION == rollback ]]; then
            [[ -f $STATE/rollback.path ]] || die "No pre-deploy snapshot recorded"
            IFS= read -r SNAPSHOT < "$STATE/rollback.path"
        fi
        [[ -d $SNAPSHOT && -f $SNAPSHOT/COMPLETE ]] || die "Not a complete snapshot: $SNAPSHOT"
        SNAPSHOT=$(cd "$SNAPSHOT" && pwd -P)
        [[ $(cat "$SNAPSHOT/COMPLETE") == mailwoman-deploy-snapshot-v1 ]] || die "Unknown snapshot format"
        for file in data.tar compose.yaml app.env deploy.env release.env SHA256SUMS; do
            [[ -f $SNAPSHOT/$file && ! -L $SNAPSHOT/$file ]] || die "Missing or symlinked snapshot file: $file"
        done
        # Check the fixed expected set, ignoring any extra manifest paths.
        (cd "$SNAPSHOT" && sha256sum data.tar compose.yaml app.env deploy.env release.env) > "$STATE/.verify-snapshot"
        cmp -s "$STATE/.verify-snapshot" "$SNAPSHOT/SHA256SUMS" || die "Snapshot checksum mismatch"
        rm "$STATE/.verify-snapshot"
        target_project=$PROJECT
        read_config "$SNAPSHOT/deploy.env"
        read_config "$SNAPSHOT/release.env"
        validate_config
        [[ $PROJECT == "$target_project" ]] || die "Snapshot belongs to project $PROJECT; initialize recovery state with that project name"
        if [[ ! -f $ACTIVE/release.env && -n $(docker ps -aq --filter "label=com.docker.compose.project=$PROJECT") ]]; then
            die "Project $PROJECT is already running without this state directory; recover its original state before restoring"
        fi
        docker image inspect "$IMAGE_ID" >/dev/null || die "Snapshot image is missing; load the matching saved Docker image before restoring"
        if [[ $EXPOSURE == caddy ]]; then docker image inspect "$CADDY_ID" >/dev/null; fi
        WORK=$(mktemp -d "$STATE/.work.XXXXXX")
        cp "$SNAPSHOT/app.env" "$WORK/app.env"
        # Managed trust lines in the effective backup are regenerated by render.
        if [[ $EXPOSURE == caddy ]]; then
            awk '!/^MW_TRUSTED_PROXIES=/ && !/^MW_FORWARDED_MODE=/' "$WORK/app.env" > "$WORK/app.clean"
            mv "$WORK/app.clean" "$WORK/app.env"
        fi
        validate_app_env "$WORK/app.env"
        DATA_VOLUME="${PROJECT}_restore_$(date -u +%Y%m%dT%H%M%SZ)_$$_${RANDOM}"
        render "$WORK"
        compose "$WORK" config --quiet
        # Extract only into a fresh Docker volume. No host tar extraction or volume deletion.
        docker volume create --label "mailwoman.deploy.project=$PROJECT" "$DATA_VOLUME" >/dev/null
        HELPER_CONTAINER=$(MSYS_NO_PATHCONV=1 docker create --network none --mount "type=volume,src=$DATA_VOLUME,dst=/data" "$IMAGE_ID")
        docker cp -a - "$HELPER_CONTAINER:/data" < "$SNAPSHOT/data.tar"
        docker rm "$HELPER_CONTAINER" >/dev/null
        HELPER_CONTAINER=
        if [[ -f $ACTIVE/release.env ]]; then
            snapshot
            log "Saved the replaced installation at $LAST_BACKUP"
            compose "$ACTIVE" stop --timeout 60
        fi
        cp "$WORK/deploy.env" "$STATE/deploy.env"
        cp "$WORK/app.env" "$STATE/app.env"
        if [[ $EXPOSURE == caddy ]]; then
            awk '!/^MW_TRUSTED_PROXIES=/ && !/^MW_FORWARDED_MODE=/' "$STATE/app.env" > "$WORK/app.clean"
            mv "$WORK/app.clean" "$STATE/app.env"
        fi
        activate "$WORK"
        log "Restored snapshot $SNAPSHOT; replaced volumes and backups remain available"
        ;;
esac
