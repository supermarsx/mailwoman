#!/usr/bin/env bash
# Internal helpers for deploy.sh. Configuration and credentials remain in the
# deployment's private files; PostgreSQL and Valkey publish no host ports.

stack_secrets() {
    local file=$1 key value
    for key in MW_STACK_DB_PASSWORD MW_STACK_CACHE_PASSWORD; do
        value=$(sed -n "s/^$key=//p" "$file")
        if [[ -z $value ]]; then
            value=$(od -An -N32 -tx1 /dev/urandom | tr -d ' \n')
            printf '\n%s=%s\n' "$key" "$value" >> "$file"
        fi
        [[ $value =~ ^[0-9a-f]{64}$ ]] || die "Invalid $key in app.env"
    done
}

# Host memory in MiB as Docker sees it (Docker Desktop: its VM), else /proc/meminfo.
stack_host_memory_mib() {
    local bytes kib
    bytes=$(docker info --format '{{.MemTotal}}' 2>/dev/null)
    if [[ $bytes =~ ^[0-9]+$ ]] && ((bytes > 0)); then echo $((bytes / 1048576)); return; fi
    kib=$(awk '/^MemTotal:/ {print $2}' /proc/meminfo 2>/dev/null)
    [[ $kib =~ ^[0-9]+$ ]] && echo $((kib / 1024)) || echo 1024
}

# How one host's memory (MiB, $1) is divided, for stack $2 (full|single) with
# Caddy run by deploy.sh or not ($3 = 1|0). Prints "app postgres valkey reserve
# caddy" in MiB. A reserve for the OS and its page cache (10%, at least 256),
# Caddy 64, Valkey's maxmemory (1/16 of the host, 32..256) plus 32 overhead;
# PostgreSQL gets 35% of the rest on the full stack, the app the remainder.
# scripts/perf/postgres-profile.mjs memoryShares holds the same arithmetic.
stack_memory_shares() {
    local host=$1 stack=$2 caddy=0 reserve valkey=0 overhead=0 rest postgres=0
    ((${3:-0})) && caddy=64
    reserve=$((host / 10)); ((reserve >= 256)) || reserve=256
    if [[ $stack == full ]]; then
        valkey=$((host / 16)); ((valkey >= 32)) || valkey=32; ((valkey <= 256)) || valkey=256
        overhead=32
    fi
    rest=$((host - reserve - caddy - valkey - overhead))
    [[ $stack == full ]] && postgres=$((rest * 35 / 100))
    echo "$((rest - postgres)) $postgres $valkey $reserve $caddy"
}

# The app's memory limit for this host: MEMORY_LIMIT ($1, e.g. 4g or 768m),
# lowered to the app's share of the host (stack_memory_shares) when the share is
# smaller, so the app and the services beside it do not each size themselves
# from the whole host. Empty in, empty out (no limit). Prints the limit.
stack_app_memory_limit() {
    local limit=$1 host=$2 stack=$3 caddy=$4 share wanted
    [[ -n $limit ]] || return 0
    read -r share _ < <(stack_memory_shares "$host" "$stack" "$caddy")
    case $limit in *g) wanted=$((${limit%g} * 1024)) ;; *m) wanted=${limit%m} ;; *) echo "$limit"; return ;; esac
    if ((share < wanted)); then echo "${share}m"; else echo "$limit"; fi
}

# PostgreSQL settings for the full stack, where PostgreSQL shares the host with
# Mailwoman: the "colocated" profile of docs/admin/postgresql-tuning.md, sized
# from PostgreSQL's OWN share of the host ($1, MiB, from stack_memory_shares),
# the host's memory ($2, MiB) and CPUs ($3), and the CPUs the app may use ($4),
# which decide how many connections the app opens (pool 4 per CPU, 4..64, plus
# about 8 dedicated). scripts/perf/postgres-profile.mjs colocatedProfile holds
# the same formulas and its test compares the two. Nothing here changes
# durability: fsync, full_page_writes and synchronous_commit keep their
# defaults (on). max_wal_size is disk space PostgreSQL may use for WAL.
stack_postgres_settings() {
    local pg=$1 host=$2 cpus=$3 app_cpus=${4:-$3} pool active connections shared work maintenance autovac cache wal minwal workers parallel pmaint avworkers
    pool=$((4 * app_cpus)); ((pool >= 4)) || pool=4; ((pool <= 64)) || pool=64
    active=$((pool + 8))
    connections=$((pool + 25)); ((connections >= 30)) || connections=30
    shared=$((pg / 4)); ((shared >= 32)) || shared=32; ((shared <= 32768)) || shared=32768
    work=$(( (pg - shared - active * 8) / (2 * active) )); ((work >= 4)) || work=4; ((work <= 64)) || work=64
    cache=$((pg * 3 / 4)); ((cache >= 128)) || cache=128
    maintenance=$((pg / 8)); ((maintenance >= 32)) || maintenance=32; ((maintenance <= 2048)) || maintenance=2048
    autovac=$((pg / 32)); ((autovac >= 16)) || autovac=16; ((autovac <= 1024)) || autovac=1024
    if ((host < 8192)); then wal=1GB; minwal=256MB; else wal=4GB; minwal=512MB; fi
    workers=$cpus; ((workers >= 8)) || workers=8
    parallel=$((cpus / 2)); ((parallel >= 2)) || parallel=2
    pmaint=$((cpus / 4)); ((pmaint >= 2)) || pmaint=2; ((pmaint <= 4)) || pmaint=4
    avworkers=$((cpus / 4)); ((avworkers >= 3)) || avworkers=3; ((avworkers <= 8)) || avworkers=8
    printf '%s\n' "shared_buffers=${shared}MB" "effective_cache_size=${cache}MB" "work_mem=${work}MB" \
        "maintenance_work_mem=${maintenance}MB" "autovacuum_work_mem=${autovac}MB" "max_connections=$connections" \
        wal_compression=lz4 "max_wal_size=$wal" "min_wal_size=$minwal" checkpoint_timeout=15min \
        effective_io_concurrency=200 maintenance_io_concurrency=100 jit=off default_toast_compression=lz4 \
        "max_worker_processes=$workers" "max_parallel_workers=$parallel" max_parallel_workers_per_gather=2 \
        "max_parallel_maintenance_workers=$pmaint" "autovacuum_max_workers=$avworkers" \
        autovacuum_vacuum_cost_limit=2000 autovacuum_vacuum_scale_factor=0.05 \
        autovacuum_vacuum_insert_scale_factor=0.05 autovacuum_analyze_scale_factor=0.02 \
        idle_in_transaction_session_timeout=10min log_lock_waits=on log_checkpoints=on \
        log_autovacuum_min_duration=10s log_temp_files=16MB
}

stack_render() {
    local directory=$1 db_password cache_password public_network=default extra_network='  default: {}' postgres_command setting host cpus app_cpus caddy=0 shares postgres_mib valkey_mib
    if [[ $EXPOSURE == network ]]; then public_network=proxy; extra_network=; fi
    [[ $STACK == full ]] || return 0
    db_password=$(sed -n 's/^MW_STACK_DB_PASSWORD=//p' "$directory/app.env")
    cache_password=$(sed -n 's/^MW_STACK_CACHE_PASSWORD=//p' "$directory/app.env")
    [[ $db_password =~ ^[0-9a-f]{64}$ && $cache_password =~ ^[0-9a-f]{64}$ ]] || die "Missing full-stack credentials; initialize them before rendering"
    [[ $POSTGRES_VOLUME =~ ^[a-zA-Z0-9][a-zA-Z0-9_.-]+$ ]] || die "Invalid PostgreSQL volume"
    # PostgreSQL and Valkey are sized from their own shares of this host.
    host=$(stack_host_memory_mib)
    cpus=$(docker info --format '{{.NCPU}}' 2>/dev/null); [[ $cpus =~ ^[1-9][0-9]*$ ]] || cpus=$(nproc 2>/dev/null || echo 1)
    app_cpus=${CPU_LIMIT%%.*}; [[ $app_cpus =~ ^[1-9][0-9]*$ ]] || app_cpus=$cpus; ((app_cpus <= cpus)) || app_cpus=$cpus
    [[ $EXPOSURE == caddy ]] && caddy=1
    shares=$(stack_memory_shares "$host" full "$caddy")
    read -r _ postgres_mib valkey_mib _ <<< "$shares"
    postgres_command="[postgres"
    while IFS= read -r setting; do postgres_command+=", -c, '$setting'"; done < <(stack_postgres_settings "$postgres_mib" "$host" "$cpus" "$app_cpus")
    postgres_command+="]"
    cat > "$directory/stack.yaml" <<EOF
services:
  mailwoman:
    environment:
      MW_DB_PATH: 'postgres://mailwoman:$db_password@postgres:5432/mailwoman'
      MW_REDIS_URL: 'redis://:$cache_password@valkey:6379'
    networks:
      store:
      $public_network:
    depends_on:
      postgres: {condition: service_healthy}
      valkey: {condition: service_healthy}
  postgres:
    image: $POSTGRES_IMAGE
    restart: unless-stopped
    command: $postgres_command
    environment:
      POSTGRES_USER: mailwoman
      POSTGRES_DB: mailwoman
      POSTGRES_PASSWORD: '$db_password'
      POSTGRES_INITDB_ARGS: '--data-checksums --auth-host=scram-sha-256'
    volumes: ['postgres_data:/var/lib/postgresql/data']
    networks: [store]
    security_opt: [no-new-privileges:true]
    shm_size: 128m
    healthcheck:
      test: [CMD-SHELL, 'pg_isready -U mailwoman -d mailwoman']
      interval: 5s
      timeout: 5s
      start_period: 15s
      retries: 24
    logging:
      driver: json-file
      options: {max-size: 10m, max-file: '3', compress: 'true'}
  valkey:
    image: $VALKEY_IMAGE
    restart: unless-stopped
    user: '999:999'
    read_only: true
    cap_drop: [ALL]
    security_opt: [no-new-privileges:true]
    tmpfs: ['/tmp:rw,noexec,nosuid,nodev,size=16m,mode=1777']
    environment:
      VALKEY_PASSWORD: '$cache_password'
    entrypoint: [/bin/sh, -ec]
    command:
      - 'umask 077; printf "requirepass %s\nsave \"\"\nappendonly no\nmaxmemory ${valkey_mib}mb\nmaxmemory-policy allkeys-lru\n" "\$\$VALKEY_PASSWORD" > /tmp/valkey.conf; exec valkey-server /tmp/valkey.conf'
    networks: [store]
    healthcheck:
      test: [CMD-SHELL, 'test "\$\$(valkey-cli --no-auth-warning -a "\$\$VALKEY_PASSWORD" ping)" = PONG']
      interval: 5s
      timeout: 3s
      retries: 12
    logging:
      driver: json-file
      options: {max-size: 10m, max-file: '3', compress: 'true'}
networks:
  store:
    internal: true
$extra_network
volumes:
  postgres_data:
    external: true
    name: $POSTGRES_VOLUME
EOF
    # Merge once, so snapshots and existing Compose operations continue to use
    # one self-contained compose.yaml. Never print this (it contains passwords).
    COMPOSE_PROFILES='' docker compose --project-name "$PROJECT" --env-file /dev/null --project-directory "$directory" -f "$directory/compose.yaml" -f "$directory/stack.yaml" config > "$directory/merged.yaml"
    mv "$directory/merged.yaml" "$directory/compose.yaml"
    rm "$directory/stack.yaml"
}

stack_prepare() {
    [[ $STACK == full ]] || return 0
    if ! docker image inspect "$POSTGRES_IMAGE" >/dev/null 2>&1; then docker pull "$POSTGRES_IMAGE"; fi
    if ! docker image inspect "$VALKEY_IMAGE" >/dev/null 2>&1; then docker pull "$VALKEY_IMAGE"; fi
    docker volume create --label "mailwoman.deploy.project=$PROJECT" "$POSTGRES_VOLUME" >/dev/null
}

# Run a snapshot stage without touching its stdout (which may be an archive).
# A local EXIT trap reaps the heartbeat on success, command failure or interruption;
# the caller's normal recovery trap still resumes the app after a failed backup.
snapshot_progress() (
    local label=$1 output=$2 started=$SECONDS watcher result=0
    shift 2
    log "$label" >&2
    (
        local delay='' bytes elapsed
        trap '[[ -z $delay ]] || kill "$delay" 2>/dev/null || true; exit 0' TERM INT
        while :; do
            sleep 5 & delay=$!
            wait "$delay" || exit 0
            elapsed=$((SECONDS - started))
            if [[ -n $output && -f $output ]]; then
                bytes=$(wc -c < "$output")
                log "$label: $((bytes / 1048576)).$((bytes % 1048576 * 10 / 1048576)) MiB written; ${elapsed}s elapsed" >&2
            else
                log "$label: ${elapsed}s elapsed; still working" >&2
            fi
        done
    ) & watcher=$!
    trap 'kill "$watcher" 2>/dev/null || true; wait "$watcher" 2>/dev/null || true' EXIT
    "$@" || result=$?
    if ((result != 0)); then
        log "$label: failed after $((SECONDS - started))s (exit $result)" >&2
        exit "$result"
    fi
    log "$label: complete in $((SECONDS - started))s" >&2
)

stack_dump() {
    local destination=$1
    grep -qx 'STACK=full' "$ACTIVE/deploy.env" || return 0
    mkdir "$destination/.postgres"
    snapshot_progress "Backing up PostgreSQL" "$destination/.postgres/.deploy-postgres.dump" \
        compose "$ACTIVE" exec -T postgres pg_dump -U mailwoman -d mailwoman --format=custom > "$destination/.postgres/.deploy-postgres.dump"
    [[ -s $destination/.postgres/.deploy-postgres.dump ]] || die "PostgreSQL backup was empty"
    snapshot_progress "Adding PostgreSQL backup to snapshot" "" tar -rf "$destination/data.tar" -C "$destination/.postgres" .deploy-postgres.dump
    rm "$destination/.postgres/.deploy-postgres.dump"
    rmdir "$destination/.postgres"
}

stack_restore_database() {
    local directory=$1
    [[ $STACK == full ]] || return 0
    compose "$directory" up -d --wait --wait-timeout "$TIMEOUT" postgres valkey
    HELPER_CONTAINER=$(MSYS_NO_PATHCONV=1 docker create --network none --mount "type=volume,src=$DATA_VOLUME,dst=/data" "$IMAGE_ID")
    docker cp "$HELPER_CONTAINER:/data/.deploy-postgres.dump" "$directory/postgres.dump"
    docker rm "$HELPER_CONTAINER" >/dev/null
    HELPER_CONTAINER=
    compose "$directory" exec -T postgres pg_restore -U mailwoman -d mailwoman --exit-on-error --single-transaction < "$directory/postgres.dump"
    rm "$directory/postgres.dump"
    MSYS_NO_PATHCONV=1 docker run --rm --network none --mount "type=volume,src=$DATA_VOLUME,dst=/working" --entrypoint /bin/rm "$POSTGRES_IMAGE" /working/.deploy-postgres.dump
}

stack_migrate_sqlite() {
    local original=$DATA_VOLUME migrated
    migrated="${PROJECT}_migrated_$(date -u +%Y%m%dT%H%M%SZ)_$$_$RANDOM"
    # The app stays stopped between the snapshot and cutover. All source schema
    # upgrades happen on the new volume, never on the original SQLite volume.
    # Read by deploy.sh cleanup if migration fails before cutover.
    # shellcheck disable=SC2034
    RESUME_CONTAINER=$(compose "$ACTIVE" ps -a -q mailwoman)
    DATA_VOLUME=$migrated
    docker volume create --label "mailwoman.deploy.project=$PROJECT" "$DATA_VOLUME" >/dev/null
    HELPER_CONTAINER=$(MSYS_NO_PATHCONV=1 docker create --network none --mount "type=volume,src=$DATA_VOLUME,dst=/data" "$IMAGE_ID")
    docker cp -a - "$HELPER_CONTAINER:/data" < "$LAST_BACKUP/data.tar"
    docker rm "$HELPER_CONTAINER" >/dev/null
    HELPER_CONTAINER=
    render "$WORK"
    compose "$WORK" up -d --wait --wait-timeout "$TIMEOUT" postgres valkey
    # Environment is supplied by Compose: passwords and server key never appear
    # in the migration command, logs, or the process's argument list.
    compose "$WORK" run --rm --no-deps -T mailwoman migrate-store --from /data/mailwoman.db --complete --upgrade-source
    # Keep rollback data only in the original volume and the registered backup;
    # an unused SQLite copy in the live PostgreSQL installation would otherwise
    # retain records after their normal deletion/retention processing.
    [[ $DATA_VOLUME != "$original" ]] || die "Refusing to remove the original SQLite database"
    MSYS_NO_PATHCONV=1 docker run --rm --network none --mount "type=volume,src=$DATA_VOLUME,dst=/working" --entrypoint /bin/rm "$POSTGRES_IMAGE" -f /working/mailwoman.db /working/mailwoman.db-wal /working/mailwoman.db-shm
    log "Complete store migration verified; original SQLite volume retained: $original"
}
