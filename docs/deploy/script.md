# Deploy with deploy.sh

[`deploy.sh`](https://github.com/supermarsx/mailwoman/blob/master/deploy.sh) installs and runs Mailwoman on one host with Docker. It deploys the webmail app (web client, Rust server, render worker and first-party plugins) with a SQLite database and upload storage in one Docker volume, or, with `--full-stack`, with PostgreSQL and Valkey beside it. It also takes backups, restores them, rolls back failed updates and writes scheduling units. It does not install a mail server: users sign in to your existing JMAP server (proxy mode), or to an IMAP or POP3 server with SMTP for sending (engine mode).

## Requirements

- Git, Bash 4 or newer, `curl`, `tar` and coreutils (`sha256sum`, `od`, `cmp`).
- Docker running Linux containers, with the Compose plugin 2.30 or newer, on the same host. Remote Docker contexts are refused: the script checks the app through `localhost`.
- A user that can reach the Docker daemon.

You do not need Rust, Node or pnpm on the host: the repository's Dockerfile builds everything. `sh deploy.sh` also works; the script re-runs itself under Bash.

## First deployment

Clone the repository, check out the branch you want to deploy, and pick one of the following. Every first installation prints the admin username and the admin console address when it finishes.

### Local, plain HTTP

```sh
bash deploy.sh                              # proxy mode (JMAP)
bash deploy.sh --mode engine --port 8081    # engine mode (IMAP/POP3 + SMTP)
```

The app listens on `127.0.0.1` only. Without `--port` the script uses 8080, or the first free port from 8081 to 8099 when 8080 is taken; it prints the port and saves it in `deploy.env`.

### Automatic HTTPS with Caddy

1. Point the hostname's A/AAAA records at this host.
2. Make TCP ports 80 and 443 free and reachable from the Internet. For HTTP/3, also allow UDP 443.
3. Run:

   ```sh
   bash deploy.sh --domain mail.example.org --mode engine
   ```

The script adds a Caddy container that gets and renews certificates and forwards WebSocket and server-sent events to the app. Caddy offers HTTP/1.1, HTTP/2 and HTTP/3 by default. For a TCP-only listener, add `--http-protocols 'h1 h2'` (or `'h1'`) at the first deploy; on an existing installation set `CADDY_PROTOCOLS=h1 h2` in `deploy.env` and run `bash deploy.sh deploy --no-build`. This setting only affects Caddy; the connection from Caddy to the app stays on the private Docker network.

### Behind your own HTTPS reverse proxy

```sh
bash deploy.sh init --public-url https://mail.example.org
# Edit .deploy.local/app.env (mail server, SMTP, proxy trust), then:
bash deploy.sh doctor
bash deploy.sh deploy
```

Configure your proxy to forward to `127.0.0.1:8080`, including WebSocket and server-sent events. Set `MW_TRUSTED_PROXIES` and `MW_FORWARDED_MODE` in `app.env` to match the proxy's address as the container sees it; see [Reverse proxy](reverse-proxy.md).

### Behind a reverse proxy container

For a proxy that runs as a container on the same host, such as Nginx Proxy Manager or Traefik, put the app on a shared Docker network instead of a host port:

```sh
bash deploy.sh --public-url https://mail.example.org --proxy-network proxy
```

1. The script creates the network `proxy` if it does not exist (and never removes it). Name your proxy's existing network to reuse it.
2. Attach the proxy to that network (`docker network connect proxy <container>`, or an `external: true` network in the proxy's own Compose file).
3. In the proxy, forward `mail.example.org` to `http://mailwoman:8080` (the project name, `mailwoman` by default) with WebSocket support and a certificate.

The app publishes no host port. Until the proxy host exists, `deploy` reports the app as healthy on the network and notes that the public URL does not answer yet.

The script sets `MW_TRUSTED_PROXIES` to the network's subnets and `MW_FORWARDED_MODE=xff` unless `app.env` sets either one. Every container on that network can then state a client address, so keep the network to the proxy and the apps behind it, or set the proxy's own address in `app.env`.

### Later runs

The first `deploy` sets up and starts the app; every later `deploy` updates the same installation. You may repeat the setup options unchanged or leave them out. A different value is refused with a message naming the `deploy.env` key to edit instead. All commands accept `--state-dir`, and you can run the script from any directory by giving its path.

`init` writes the configuration without touching Docker. `doctor` checks prerequisites, the configuration and the generated Compose file without starting anything. `--dry-run` describes any command without writing files or contacting Docker.

## Updates and daily operations

```sh
bash deploy.sh --help
bash deploy.sh status       # containers, URL, data volume, whether /metrics is on
bash deploy.sh check        # exit 0 when healthy; otherwise run ALERT_HOOK and exit 1
bash deploy.sh logs         # follow the last 100 lines; Ctrl-C to leave
bash deploy.sh stop         # keeps containers, volumes and backups
bash deploy.sh start
bash deploy.sh restart

bash deploy.sh deploy                # update from Git, build, back up, start, verify
bash deploy.sh deploy --no-pull      # deploy the current checkout as it is
bash deploy.sh deploy --no-build     # apply configuration changes; reuse the app image
bash deploy.sh deploy --pull-only    # update the Git checkout only, then exit
```

`check`, `ALERT_HOOK` and metrics are described in [Monitoring](monitoring.md).

### How deploy updates the source

`deploy` runs `git pull --ff-only` on the current branch before it reads any configuration or touches a container, then restarts itself from the updated checkout with the same arguments. Local tracked, staged and untracked edits are kept when Git can fast-forward without overwriting them. Git refuses overlapping edits and incoming path collisions; a detached HEAD, a branch without an upstream, diverged history or a failed fetch also stop the deploy before anything is built. The script never stashes, resets, cleans or creates a merge commit.

- `--no-pull` deploys the checkout as it is: use it for a local revision, a source archive or an offline host. A source archive without Git needs `MW_BUILD_REVISION` set to the full 40-character commit.
- `--pull-only` fast-forwards and exits without building, backing up or restarting anything. With `--dry-run` it only says what it would do.
- `--pull` is unrelated to Git: it refreshes a container image given with `--image`.
- Other commands do not pull.

### Progress output

A deploy prints six numbered steps on stderr: prepare source checkout, validate configuration and prerequisites, prepare runtime image, protect installation data, start services and verify health, and finalize backup history. Each step reports its duration, and the run ends with UTC start and finish times and the total duration. A Git update lists the branch, the old and new commit, and every incoming commit subject (control characters removed).

Colour is used only when stderr is a terminal. `NO_COLOR=1` or `TERM=dumb` turns it off, and redirected output is plain.

Snapshots report each stage (stopping the app, copying `/data`, dumping PostgreSQL on a full stack, validating the archive, calculating checksums, resuming the app) with its duration. Stages longer than five seconds print an update every five seconds; copies and dumps show the MiB written. These are activity updates, not a percentage or an estimate.

### Health checks and automatic rollback

Before replacing a running app, `deploy` builds or pulls the image (a failure there causes no outage), then snapshots the installation, which stops the app briefly. It then starts the new release, waits for the container's health check and requests `/healthz` and `/` at the public URL, with valid TLS where the URL is HTTPS. `--timeout SECS` (10 to 3600, default 180) sets how long each wait may take. These checks show that the app answers over HTTP; they do not test mailbox sign-in, SMTP delivery or reachability from outside the host.

When the health wait or the public check fails, the script rolls back:

1. It copies the failed release's files and its last 300 log lines into `failed/<time>-<pid>/` in the state directory, and stops it.
2. It starts the previous release (its image, configuration and exposure) on a copy of the snapshot taken just before the update, in a new volume.
3. It exits non-zero and names what it restored and where.

The failed release's data volume, including any database migrations it ran, is kept. `deploy.env` and `app.env` keep the settings that failed: correct them, or the image, and deploy again.

- An update whose app is healthy but whose public URL does not answer (DNS, firewall or proxy not ready) is rolled back too. With a proxy network, an unanswered public URL only prints a note.
- `--no-auto-rollback` leaves a failed update in place for diagnosis; `bash deploy.sh rollback` then restores the same snapshot.
- `--skip-backup` skips the snapshot and therefore turns automatic rollback off.
- A first installation has nothing to roll back to. On failure it keeps its volume and containers; after fixing DNS, firewall or proxy, run `bash deploy.sh start` to check it again.

After an interrupted SQLite write or maintenance operation, startup can take longer than the default timeout; see [SQLite startup and recovery](../admin/database-startup.md).

### Concurrent runs

Commands that change the installation hold a lock directory, `.lock`, in the state directory; `status`, `check`, `logs` and `doctor` do not. After a `SIGKILL` or a host crash, remove a stale `.lock` only after making sure no operation is running.

## Choosing the image

By default `deploy` builds the image from the checkout. You can use a published or ready-built image instead.

### Building from source

The build uses the Dockerfile target `runtime-static`, the same target CI scans, signs and publishes: a `scratch` image with statically linked x86_64 binaries. Docker builds the web client in a separate stage, so a change to the web client alone reuses the cached Rust build when the same builder and its cache are kept. The first build compiles every dependency; pruning the builder cache brings that back.

| Option | What it does | Default |
|---|---|---|
| `--build-jobs N` | Concurrent Cargo jobs (1 to 999). Lower it if the build runs out of memory. This is separate from `CPU_LIMIT` and `MEMORY_LIMIT`, which limit the running app. | `8` |
| `--build-profile release\|fast` | `fast` compiles the `mw-server-core` and `mw-server` crates at optimization level 2; dependencies stay at release level. It makes no promise about build time or runtime speed. | `release` |
| `--incremental` / `--no-incremental` | Turns cached incremental Rust compilation on or off. | off |

For a direct Docker build, the matching build arguments are `MW_BUILD_JOBS`, `MW_BUILD_PROFILE` and `MW_CARGO_INCREMENTAL` (`0` or `1`), plus the required `MW_BUILD_REVISION` (the full commit):

```sh
docker build --target runtime-static --build-arg MW_BUILD_REVISION="$(git rev-parse HEAD)" -t mailwoman:local .
```

### Using a published image

```sh
bash deploy.sh deploy --prebuilt
```

`--prebuilt` pulls `ghcr.io/supermarsx/mailwoman:master` (or `<IMAGE_REPOSITORY>:master` when `IMAGE_REPOSITORY` is set in `deploy.env`) and skips local compilation. The published tag is `master`, not `latest`, and it can lag behind your checkout. The script still updates the Git checkout unless you add `--no-pull`. `--prebuilt` works only with `deploy` and cannot be combined with `--image`, `--image-tag`, `--no-build`, `--build-profile fast` or `--pull-only`.

For an exact tag or digest, use `--image`, or `--image-tag` with `IMAGE_REPOSITORY` set:

```sh
bash deploy.sh deploy --image registry.example.org/mailwoman:TAG --pull
bash deploy.sh deploy --image ghcr.io/supermarsx/mailwoman:master --pull --require-signature
# With IMAGE_REPOSITORY=ghcr.io/supermarsx/mailwoman in deploy.env:
bash deploy.sh deploy --image-tag master --require-signature
```

`--image` uses a copy already in the local image store unless `--pull` is given; a missing image is pulled. `--image-tag TAG` always pulls `IMAGE_REPOSITORY:TAG`, so an unattended update job passes only a tag. None of the commands asks questions, so all of them can run unattended.

`--no-build` reuses the last applied image and applies configuration changes only; it does not put newly pulled code into the image.

### Signature checks

When `deploy` fetches an `--image`, `--image-tag` or `--prebuilt` image from its registry and `cosign` is on `PATH`, it runs `cosign verify` and then deploys exactly the digest that was verified. The certificate must name `SIGNATURE_IDENTITY` and `SIGNATURE_ISSUER` from `deploy.env`; when they are empty, the script expects this repository's publishing workflow, `https://github.com/supermarsx/mailwoman/.github/workflows/supply-chain.yml@refs/heads/master`, and GitHub's issuer, `https://token.actions.githubusercontent.com`. A failed verification stops the deploy before any snapshot or container change.

Without `cosign`, or for an image used from the local store without `--pull`, the script prints that the signature was not checked and continues; `--require-signature` refuses the image instead. Images that `deploy.sh` builds from source are not signature-checked. A fork that publishes its own images sets `SIGNATURE_IDENTITY` (and `SIGNATURE_ISSUER` for another CI) in `deploy.env`.

The publishing workflow pushes an image before it signs it, so for a few minutes after a push to `master` the `master` tag can name an unsigned image; a verified deploy of it fails until the signature exists.

### Image digest in the app environment

Each deploy sets `MW_IMAGE_DIGEST` in the app's environment: `<repository>@sha256:…` when an `--image` has a registry digest for its repository, otherwise the local image ID `sha256:…` (always the case for an image built from source). The script manages it; `app.env` may not set it.

## Full stack and one-step SQLite migration

```sh
bash deploy.sh deploy --full-stack                                       # existing installation
bash deploy.sh deploy --full-stack --domain mail.example.org --mode engine   # first installation with Caddy
```

The same command starts a first full-stack installation or migrates an existing SQLite installation managed by this script. It runs Mailwoman with PostgreSQL 16 and Valkey 8, plus Caddy when `--domain` is used. Mail servers stay external.

- PostgreSQL holds the data. Valkey is a cache with no persistence and a 256 MB memory limit; each cache class's own policy decides what may be stored in it.
- PostgreSQL and Valkey publish no host ports and share an internal network with the app; the app also keeps its normal network for upstream mail and the proxy. The app starts after both pass their health checks.
- `STACK=full`, `POSTGRES_IMAGE` and `VALKEY_IMAGE` (both pinned by digest) are saved in `deploy.env`.
- The database and cache passwords are generated once into `app.env` as `MW_STACK_DB_PASSWORD` and `MW_STACK_CACHE_PASSWORD`. Do not change them on an existing PostgreSQL volume.
- Remove any `MW_REDIS_URL` from `app.env` first; the full stack manages it.
- Disk encryption on the host is your responsibility; selecting PostgreSQL does not encrypt its volume.

### Full stack with your own reverse proxy

`--full-stack` never adds Caddy by itself. An existing installation keeps its saved exposure, public URL, port, proxy network and trust settings during the migration:

```sh
bash deploy.sh deploy --full-stack --state-dir .deploy.local
bash deploy.sh deploy --full-stack --state-dir .deploy.local --dry-run   # inspect the plan first
```

For a first full-stack installation behind a proxy:

```sh
bash deploy.sh --full-stack --public-url https://mail.example.org --port 8080             # host proxy: 127.0.0.1:8080
bash deploy.sh --full-stack --public-url https://mail.example.org --proxy-network proxy    # proxy container: http://mailwoman:8080
```

Both enable Secure cookies and leave TLS to your proxy. Do not pass `--domain`, which selects Caddy. To move an existing local SQLite installation behind your proxy at the same time, first edit `deploy.env` (`EXPOSURE=external` or `network`, `PUBLIC_URL=https://…`, and `PORT` or `PROXY_NETWORK`), configure the proxy and trust settings, then run the migration command.

### What the migration does

On an existing SQLite installation managed by this script, `deploy --full-stack`:

1. Prepares the images and checks that the app image supports complete migration (`migrate-store --complete`); otherwise it stops before touching the app.
2. Stops the app and takes a checksummed snapshot of SQLite, uploads and configuration.
3. Copies `/data` into a fresh volume, upgrades that offline copy's schema, and copies every application table into a fresh PostgreSQL volume, including admin identities, domains, configuration, OAuth registrations and plugin grants.
4. Refuses a non-empty destination or a schema mismatch, and rolls the destination transaction back on any count or content mismatch.
5. Removes the SQLite files from the new volume, starts the app on PostgreSQL and verifies health. The original SQLite volume is kept.

The app is unavailable while the copy and migration run. Make sure there is room for the snapshot, the original volume, the working copy and the PostgreSQL data. A migration error resumes the previous app; a failed start uses the normal automatic rollback unless you passed `--no-auto-rollback`. `--skip-backup` is refused for a migration.

Full-stack snapshots include a PostgreSQL custom-format dump inside `data.tar`, covered by the same checksums and optional encryption. Restore and rollback create fresh app and PostgreSQL volumes, restore the dump and start the app; Valkey starts empty. Returning from `STACK=full` to `single` is refused; restoring a snapshot taken before the migration is the way back.

## Configuration and exposure

The state directory is `.deploy.local` beside the script, or the path given with `--state-dir`. Inside the repository it must sit under a top-level `*.local` directory, which `.gitignore` and `.dockerignore` already exclude; directories outside the repository are allowed. The script creates files with mode `0600` and the directory with `0700`.

| Path | What it holds |
|---|---|
| `deploy.env` | Deployment settings (table below). Plain `KEY=value`, no quotes. |
| `app.env` | `MW_SERVER_KEY`, the admin credentials, the metrics token and other app environment variables. Values are literal and unquoted: dollar signs and spaces are kept. No `export`, quotes or shell commands. |
| `active/` | The last applied Compose file, configuration and exact image IDs. `start` and `restart` use these; edit the parent files and run `deploy` to change them. |
| `backups/` | Snapshots; see [Backups](backups.md). |
| `rollback.path` | The snapshot taken before the most recent deploy, which `rollback` restores. |
| `failed/` | Files and logs of updates that were rolled back. |
| `geoip/` | The country database downloaded by `geoip`. |
| `systemd/` | Units written by `timers`. |

### deploy.env

| Key | What it does | Default |
|---|---|---|
| `PROJECT` | Compose project name; also the app's name on a proxy network. Fixed after the first deploy. | `mailwoman` |
| `EXPOSURE` | `local`, `caddy`, `external` (host proxy) or `network` (proxy container). Set by the setup options. | `local` |
| `DOMAIN` | Hostname for Caddy. | empty |
| `PUBLIC_URL` | The URL users open; an HTTPS origin without a path for `caddy`, `external` and `network`. | `http://localhost:<PORT>` |
| `PORT` | Host port on `127.0.0.1` for `local` and `external`. | first free port from 8080 to 8099 |
| `MODE` | `proxy` (JMAP) or `engine` (IMAP/POP3 + SMTP). | `proxy` |
| `UPSTREAM_DEFAULT` | The mail server users sign in to: an `http(s)://` JMAP URL in proxy mode, an `imaps://`, `imap://`, `pop3s://` or `pop3://` URL in engine mode. Passed to the app as `MW_UPSTREAM_DEFAULT`. | empty |
| `STACK` | `single` (SQLite) or `full` (PostgreSQL and Valkey). | `single` |
| `POSTGRES_IMAGE`, `VALKEY_IMAGE` | Full-stack images. | `postgres:16` and `valkey/valkey:8`, pinned by digest |
| `PROXY_PREFIX` | First three octets of Caddy's private `/24` network. Fixed after the first deploy. | `172.30.70` |
| `PROXY_NETWORK` | Docker network for `network` exposure. | empty |
| `CADDY_IMAGE` | Caddy image. | `caddy:2.11.7-alpine`, pinned by digest |
| `CADDY_PROTOCOLS` | `h1`, `h1 h2` or `h1 h2 h3`. | `h1 h2 h3` |
| `ACME_CA`, `ACME_CA_ROOT` | Another ACME directory and its root certificate, for Caddy only (below). | empty |
| `MEMORY_LIMIT` | App container memory limit, such as `1g` or `768m`. Empty means no limit. | `4g` |
| `CPU_LIMIT` | App container CPU limit, such as `2` or `1.5`. Empty means no limit. | `4` |
| `IMAGE_REPOSITORY` | Repository (no tag or digest) for `--image-tag` and `--prebuilt`. | empty |
| `SIGNATURE_IDENTITY`, `SIGNATURE_ISSUER` | Expected signer of pulled images. | empty (this repository's workflow) |
| `BACKUP_KEEP`, `BACKUP_KEEP_DAYS`, `BACKUP_RECIPIENT`, `BACKUP_HOOK`, `BACKUP_TARGETS_FILE` | Backup retention, encryption and off-host copies; see [Backups](backups.md). | empty |
| `ALERT_HOOK` | Program run when `check` fails; see [Monitoring](monitoring.md). | empty |

The backup and alert keys take effect as soon as `deploy.env` is saved. The other keys take effect at the next `deploy`.

`MEMORY_LIMIT` and `CPU_LIMIT` become the container's `mem_limit` and `cpus`. They are ceilings, not reservations. A `CPU_LIMIT` above the number of CPUs Docker can use is lowered to that number, with a message. When either key is empty, each deploy prints a line saying the app has no such limit. When the app reaches its memory limit, the kernel stops it and Docker restarts it. Caddy has no limits.

### app.env

The first `init` or `deploy` writes:

- `MW_SERVER_KEY`: 64 random hex digits. It seals stored credentials and uploads. Keep it with your backups; a deploy refuses to run when it differs from the applied key.
- `MW_ADMIN_USER`: `admin-` followed by 12 hex characters, and `MW_ADMIN_PASSWORD`: 48 random hex characters. The script prints the username, never the password; read the password from `app.env`.
- `MW_ADMIN_PATH`: a random 20-character path. The admin console answers only at this path under the public URL; `/admin` answers 404. The deploy output prints the full console address. While the variable is set, the console cannot change the path. The random path reduces scanning; it does not replace the admin password, two-factor sign-in or a network restriction.
- `MW_CSRF_STRICT=true`, `RUST_LOG=info`.
- `MW_METRICS_TOKEN`: the bearer token for `/metrics`; see [Monitoring](monitoring.md).

The sign-in form asks only for an email address and password, so set the mail server before users sign in: `UPSTREAM_DEFAULT` in `deploy.env`, or `MW_UPSTREAM_DEFAULT` in `app.env` (setting both is refused). Other settings you can add to `app.env` include `MW_JMAP_UPSTREAMS` (allowed JMAP origins for proxy-mode sign-in), and `MW_SMTP_HOST`, `MW_SMTP_PORT` and `MW_SMTP_SECURITY` when submission differs from the mailbox server in engine mode.

The script manages these variables, and `app.env` may not set them: `MW_BIND`, `MW_DB_PATH`, `MW_UPLOAD_DIR`, `MW_MODE`, `MW_PUBLIC_URL`, `MW_COOKIE_SECURE`, `MW_RENDER_BIN`, `MW_PLUGIN_DIR`, `MW_WEB_DIR`, `MW_BASE_PATH`, `MW_ACME*`, `MW_TLS_*`, `MW_PROXY_PROTOCOL` and `MW_IMAGE_DIGEST`. With Caddy, `MW_TRUSTED_PROXIES` and `MW_FORWARDED_MODE` are managed too; with the full stack, `MW_REDIS_URL`. As a result, in-app TLS, sub-path hosting and custom volume mounts are not available through this script.

### Container settings

The app container runs as UID/GID 65532 with a read-only root filesystem, all capabilities dropped, `no-new-privileges`, a limit of 512 processes, a 64 MB `/tmp` and log rotation (three 10 MB files). SQLite and uploads live together under `/data` in a named volume. Local mode uses plain HTTP cookies; every HTTPS exposure sets Secure cookies.

### Caddy settings

Caddy publishes ports 80 and 443 (and UDP 443 with HTTP/3) and keeps certificates in the named volumes `<project>_caddy_data` and `<project>_caddy_config`. Its private network is `<PROXY_PREFIX>.0/24`; choose another private prefix with `--proxy-prefix 172.30.71` at the first deploy if that subnet overlaps one you use. Only Caddy's address, `.2`, is trusted for forwarded headers.

`ACME_CA` names another ACME directory by its `https://` URL, for example Let's Encrypt's staging endpoint `https://acme-staging-v02.api.letsencrypt.org/directory` or a private CA; empty keeps Caddy's default issuers. `ACME_CA_ROOT` is the absolute path of a PEM file with the root certificate that signs that directory's HTTPS certificate, for a CA whose root is not in the Caddy image. The file must exist whenever `doctor`, `deploy`, `restore` or `rollback` runs. Both keys are refused for other exposures. Browsers do not trust certificates from a staging endpoint or a private CA unless that root is installed on the client.

To change Caddy, set `CADDY_IMAGE` to another version and digest (`docker buildx imagetools inspect caddy:<version>-alpine` prints the digest) and deploy.

### Changing the exposure

You can move an installation between `local`, `external` and `network`: edit `EXPOSURE`, `PUBLIC_URL`, `PORT` and `PROXY_NETWORK` in `deploy.env`, then run `deploy`. The project name and the Caddy subnet cannot change after the first deploy, and an installation cannot move to or from Caddy; create a separate deployment for that. Passkeys are bound to the origin, so changing the public URL can make passkeys registered under the old one unusable.

To put an existing installation on a proxy container's network:

1. In `.deploy.local/deploy.env` set `EXPOSURE=network`, `PROXY_NETWORK=<name>` and `PUBLIC_URL` to the `https://` address the proxy serves.
2. Run `bash deploy.sh deploy --no-build`.
3. Point the proxy at `http://<project>:8080` with WebSocket support. The host port is closed.

Several installations on one host need distinct project names, state directories, ports and Caddy subnets; only one can own ports 80 and 443.

### Country database (optional)

Country sign-in policies need a local country database. `bash deploy.sh init --geoip`, `deploy --geoip`, or `bash deploy.sh geoip` at any time downloads DB-IP "IP to Country Lite" on the host:

- over HTTPS only, this month's file or, failing that, last month's;
- at most 128 MB compressed and 256 MB unpacked, checked for the MaxMind DB marker;
- saved as `<state>/geoip/dbip-country-lite.mmdb`.

The next `deploy` mounts `<state>/geoip` read-only at `/geoip-seed` and sets `MW_GEOIP_SEED`. The server installs the file only when its data directory `/data/geoip` holds no database yet; after that, the admin console page **Country database** keeps it current. The data is licensed CC BY 4.0: show "IP Geolocation by DB-IP" with a link to https://db-ip.com where country results are used. See [Country database](../admin/country-database.md).

## Database maintenance

```sh
bash deploy.sh maintenance --orphan-bodies --compact            # preview only
bash deploy.sh maintenance --apply --orphan-bodies --compact    # stop, snapshot, clean, resume
bash deploy.sh maintenance --online                             # convert SQLite while the app runs
```

| Option | What it does |
|---|---|
| `--apply` | Apply the selected cleanup. Without it, `maintenance` only previews. |
| `--orphan-bodies` | Reclaim cached message bodies that no message references. |
| `--compact` | Rebuild SQLite to reclaim free pages. |
| `--purge-search-index` | Remove the default search index; rebuild it from the admin console afterwards. |
| `--logs-older-than-days N` | Purge operational diagnostics older than N days (`0` = all). |
| `--enable-incremental-vacuum` | Turn on background page reclamation (one offline rebuild if needed). |
| `--skip-backup` | Skip the snapshot before `--apply`. |
| `--online`, `--online-status`, `--online-cancel` | Start, inspect or cancel an online SQLite conversion through the running app; no stop and no snapshot. |

Offline cleanup applies to SQLite installations only. See [Database maintenance](../admin/database-maintenance.md) and [Online SQLite conversion](online-sqlite-maintenance.md).

## Backups and recovery

```sh
bash deploy.sh backup
bash deploy.sh rollback
bash deploy.sh restore .deploy.local/backups/TIMESTAMP-PID-RANDOM
```

`backup` stops the app and copies all of `/data` (SQLite with its WAL files, uploads), keeping numeric ownership, plus the applied configuration (`app.env` with `MW_SERVER_KEY` included) and the exact image IDs. A checksum file and a final `COMPLETE` marker separate usable snapshots from interrupted ones. The app is resumed even if the copy fails or the command is interrupted.

`restore` checks the snapshot, fills a new volume from it, snapshots the current installation (which then stays stopped) and starts the saved release and configuration on the restored data. Everything written after the snapshot is lost. The backup keys in `deploy.env` (`BACKUP_*`, `ALERT_HOOK`) are not taken from the snapshot. The replaced volume and the pre-restore snapshot stay available. `rollback` restores the snapshot taken before the last deploy, old image and data included.

After every snapshot (`backup`, the one `deploy` takes, the one `restore` takes, the one `maintenance --apply` takes) the script writes `/data/last-snapshot.json` into the app's data volume. It holds no secret and is not needed for a restore; the server does not read it.

```json
{"schema":"mailwoman-deploy-snapshot/1","kind":"backup","takenAt":"2026-10-25T03:30:00Z","path":"/srv/mailwoman/.deploy.local/backups/20261025T033000Z-4242-17","encrypted":false}
```

`kind` is `backup`, `pre-deploy`, `pre-restore` or `pre-maintenance`; `path` is the snapshot directory or encrypted archive on the host.

::: warning
Snapshots contain `MW_SERVER_KEY`, the admin password and your users' cached mail. Keep them on private, encrypted storage. Checksums detect accidental corruption, not deliberate replacement: restore only snapshots you trust.
:::

Retention, encryption, off-host copies and scheduling are covered in [Backups](backups.md).

### Recovering on another host

Docker images are not inside snapshots, and neither are Caddy's certificate volumes. Before pruning images, keep every image a snapshot needs with `docker image save` (Caddy's too, for HTTPS installations). On the replacement host, load those images, then:

```sh
# Use the original project name; the restore replaces this generated configuration.
bash deploy.sh init --project mailwoman
bash deploy.sh restore /secure/path/to/snapshot
```

Caddy requests new certificates unless you copy its volumes across. Images built by `deploy.sh` contain x86_64 binaries, so restore on an x86_64 host. Test restores periodically.

## Old browser tabs after a deploy

A browser tab opened before a deploy keeps running the previous web build and loads that build's content-hashed files (`/assets/<name>-<hash>.js`) when it opens a page for the first time. To keep those tabs working, the server copies the hashed files of the build it serves into `retained-assets` beside the SQLite database (`/data/retained-assets`, inside the backed-up volume) at each start. It keeps the files of the last 5 builds, each for 14 days after a newer build replaced it, within 512 MiB.

When a tab still cannot load a file and the server serves a different build, the app shows **A new version is available, reloading…** and reloads once. It does not reload automatically while an editor has unsaved changes. Files without a content hash, such as `index.html`, are sent with `Cache-Control: no-cache`, so a new tab always gets the current build.

On a full-stack installation the database is PostgreSQL, so nothing is retained unless you set `MW_RETAINED_ASSETS_DIR` in `app.env` to a directory under `/data`. `MW_RETAINED_ASSETS_DIR=off` turns retention off.

## Testing the script

The repository includes a Docker lifecycle test that never touches your normal deployment:

```sh
docker build --target runtime-static --build-arg MW_BUILD_REVISION="$(git rev-parse HEAD)" -t mailwoman:deploy-verify .
bash scripts/test-deploy.sh mailwoman:deploy-verify
```

It uses random project names and localhost port 18089 and the two ports above it (`DEPLOY_TEST_PORT` changes the first). It holds localhost:8080 for a few seconds when that port is free, to test the port choice. It covers, among other things, `--dry-run`, exposure moves, the refusal to move to Caddy, automatic rollback, `rollback`, image signature checks with a stand-in `cosign`, resource limits, metrics, `check` and `ALERT_HOOK`, `prune`, encrypted backups with each installed tool, `BACKUP_HOOK` and the `timers` units. Nothing reaches the Internet or uses ports 80 and 443. It removes its containers, volumes and network afterwards; failed runs keep local diagnostics.

When Playwright can start (`pnpm exec playwright` in `apps/web`, or `apps/web/node_modules/.bin/playwright`), the test also drives the admin console in a browser; otherwise that step is skipped and the result says so.

| Variable | What it does |
|---|---|
| `DEPLOY_TEST_PORT` | First localhost port (default `18089`). |
| `DEPLOY_TEST_PREFIX` | Start of the random project names (default `mw-deploy-test`). |
| `DEPLOY_TEST_REQUIRE=all` | Fail, before deploying anything, when Playwright cannot start or neither gpg nor age is installed. |
| `DEPLOY_TEST_SCRIPT` | Path of another copy of `deploy.sh` to test, for example one with a seeded defect. |

## Troubleshooting

| Symptom | Cause | Fix |
|---|---|---|
| `Port N is already in use on this host` | Another service holds the app's port (or 80/443 for Caddy). | Stop that service, or set `PORT` (and `PUBLIC_URL` for local mode) in `deploy.env` to a free port. |
| `<KEY> is '…' in …/deploy.env, not '…'` | A setup option differs from the saved configuration. | Edit `deploy.env` instead of passing a different option. |
| `Git update failed; nothing was built or restarted` | Diverged history, overlapping local edits or a failed fetch. | Reconcile the checkout, or deploy with `--no-pull`. |
| `MW_SERVER_KEY changed` | `app.env` holds a different key from the applied one. | Restore the original key; stored credentials and uploads need it. |
| `<VAR> is managed by deploy.sh` | `app.env` sets a variable the script controls. | Remove the line. |
| `App is healthy internally, but <URL> is not reachable` | DNS, firewall or proxy does not route to the app. | Fix the route; the update was rolled back unless `--no-auto-rollback` was given. |
| `Another deployment operation holds …/.lock` | Another command is running, or one was killed. | Wait, or remove the empty `.lock` after confirming nothing runs. |
| `Snapshot image is missing` during `restore` | The image the snapshot needs is not in the local store. | Load the saved image with `docker image load`. |

## Related

- [Backups](backups.md)
- [Monitoring](monitoring.md)
- [Reverse proxy](reverse-proxy.md)
- [PostgreSQL](postgres.md)
- [Deployment overview](README.md)
