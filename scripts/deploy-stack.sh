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

stack_render() {
    local directory=$1 db_password cache_password public_network=default extra_network='  default: {}'
    if [[ $EXPOSURE == network ]]; then public_network=proxy; extra_network=; fi
    [[ $STACK == full ]] || return 0
    db_password=$(sed -n 's/^MW_STACK_DB_PASSWORD=//p' "$directory/app.env")
    cache_password=$(sed -n 's/^MW_STACK_CACHE_PASSWORD=//p' "$directory/app.env")
    [[ $db_password =~ ^[0-9a-f]{64}$ && $cache_password =~ ^[0-9a-f]{64}$ ]] || die "Missing full-stack credentials; initialize them before rendering"
    [[ $POSTGRES_VOLUME =~ ^[a-zA-Z0-9][a-zA-Z0-9_.-]+$ ]] || die "Invalid PostgreSQL volume"
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
      options: {max-size: 10m, max-file: '3'}
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
      - 'umask 077; printf "requirepass %s\nsave \"\"\nappendonly no\nmaxmemory 256mb\nmaxmemory-policy allkeys-lru\n" "\$\$VALKEY_PASSWORD" > /tmp/valkey.conf; exec valkey-server /tmp/valkey.conf'
    networks: [store]
    healthcheck:
      test: [CMD-SHELL, 'test "\$\$(valkey-cli --no-auth-warning -a "\$\$VALKEY_PASSWORD" ping)" = PONG']
      interval: 5s
      timeout: 3s
      retries: 12
    logging:
      driver: json-file
      options: {max-size: 10m, max-file: '3'}
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
