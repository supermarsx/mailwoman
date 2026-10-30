# Root deployment script

[`deploy.sh`](../../deploy.sh) manages a single-host Docker deployment of the
Mailwoman web app: packaged SPA, Rust server, render worker, first-party plugins,
SQLite database and filesystem upload storage. It does not install a mail server.
Users connect their existing JMAP accounts, or IMAP/POP3 + SMTP in engine mode.

## First deployment

On the deployment host, install **Git**, **Bash 4+**, **Docker with Linux containers and
Compose 2.30+**, `curl`, `tar`, and coreutils (`sha256sum`, `od`, `cmp`, etc.).
Run as a user who can access the Docker daemon. Host Rust, Node and pnpm are not
required: the repository's Dockerfile builds the web app and binaries. Use the
same local Docker daemon for every operation; remote Docker contexts are not
supported by the localhost exposure and verification steps.

### Build reuse

Deploy normally with `sh deploy.sh deploy` to pull and build the updated source.
Docker builds the web app separately from Rust and packages its files in the
image at `MW_WEB_DIR`. With the same builder and its cache retained, a UI-only
change reuses the completed Rust build layer. Rust source or dependency changes
rebuild Rust, reusing Cargo's registry and target caches where possible.
The first build still compiles the dependencies; changing the toolchain or
pruning the builder cache can require another expensive build.

### Tabs open across a deploy

A browser tab opened before a deploy keeps running the previous web build and
loads that build's hashed files (`/assets/<name>-<hash>.js`) as it opens lazy
pages. The new image does not contain them. At each start the server copies the
hashed files of the build it serves into `/data/retained-assets` (beside the
database, in the backed-up data volume) and keeps the files of the last 5 builds,
each for 14 days after a newer build replaced it, within 512 MiB. Those requests
are then answered with the same immutable files. When a tab still cannot load a
file, the app checks whether the server has a newer build, shows "A new version is
available, reloading…" and reloads once (no automatic reload while an editor has
unsaved changes). `index.html` is always sent with `Cache-Control: no-cache`, so a
new tab gets the new build. Add `MW_RETAINED_ASSETS_DIR=off` to `app.env` to disable retention; deploy.sh
needs no other setting for it.

Docker Rust builds enable incremental compilation for workspace/path crates in
its persistent target cache. Optimization remains release level3 with16 codegen
units, overflow checks and stripping unchanged. The first build after enabling
this has to create incremental state; benefits apply to subsequent Rust changes,
not a cold build. Cache storage grows. Keep the same builder and avoid pruning
its cache between deploys. Use `--no-incremental` to disable this for a build;
switching modes can itself require recompilation.

`sh deploy.sh deploy --build-jobs 4` limits Cargo's concurrent jobs if compilation
exhausts memory or causes swapping. Without it Cargo chooses concurrency from
available CPUs. This is separate from `CPU_LIMIT`/`MEMORY_LIMIT`, which limit the
running app. The corresponding direct Docker build arguments are
`MW_BUILD_JOBS=4` and `MW_CARGO_INCREMENTAL=0` (default1).
No build-time reduction is promised without measuring on the deployment host.
For the shortest deployment, build a trusted image elsewhere and use the existing
`--image`/`--pull` path so the deployment host does not compile Rust.

`--no-build` reuses an existing application image. It applies deployment
configuration but does not put newly pulled application code into that image.

Clone the repository and check out the branch you intend to deploy, then run:

```sh
# Automatic HTTPS: first point this hostname's A/AAAA records at this host.
# TCP ports 80 and 443 must be free and reachable from the Internet.
# UDP 443 is also needed for HTTP/3 (QUIC).
bash deploy.sh --domain mail.example.org --mode engine
```

The managed Caddy listener offers HTTP/1.1, HTTP/2 and HTTP/3. Its Compose
manifest publishes both TCP and UDP 443. QUIC requires UDP forwarding through
the host firewall and any upstream load balancer; TCP clients retain HTTP/2
or HTTP/1.1. For a TCP-only installation, initialize with
`--http-protocols 'h1 h2'`. On an existing installation, set
`CADDY_PROTOCOLS=h1 h2` in `.deploy.local/deploy.env` and run
`sh deploy.sh deploy --no-build`. `h1` is also supported. These are listener
settings for managed Caddy, not a claim of native application HTTP/3 or a
change to an externally managed proxy. The application connection behind
Caddy remains on the private deployment network.

For a local installation:

```sh
bash deploy.sh                       # http://localhost:8080; JMAP proxy mode
# Or, for IMAP/POP3 + SMTP:
bash deploy.sh --mode engine --port 8081
```

Without `--port`, a first installation uses 8080, or the next free port up to
8099 when 8080 is taken; the script prints the port and keeps it in `deploy.env`.
Later deploys stop before building if a published port is held by another
service, and name the setting to change.

These examples describe separate first installations. The first `deploy` sets
up and starts the app; every later `deploy` updates the same installation. Setup
flags may be repeated unchanged or left out on later runs; a different value is
refused, and `deploy.env` is the place to change it. All commands accept `--state-dir`
and can be invoked from any working directory by passing the path to `deploy.sh`.

For an existing TLS reverse proxy:

```sh
bash deploy.sh init --public-url https://mail.example.org
# Configure your proxy to forward to 127.0.0.1:8080, including WebSocket and SSE.
# Edit .deploy.local/app.env for upstreams, SMTP settings, proxy trust, etc.
bash deploy.sh doctor
bash deploy.sh deploy
```

For a proxy that runs as a container on the same host, such as Nginx Proxy
Manager, share a Docker network with it instead of publishing a host port:

```sh
bash deploy.sh --public-url https://mail.example.org --proxy-network proxy
```

The app joins the network `proxy` under its project name (default `mailwoman`)
and publishes nothing on the host. The script creates the network when it does
not exist and never removes it; name your proxy's existing network to reuse it.
Attach the proxy to that network (`docker network connect proxy <container>`, or
an `external: true` network in its own Compose file), then add a proxy host for
`mail.example.org` forwarding to `http://mailwoman:8080` with WebSocket support
and a certificate. Until that proxy host exists, `deploy` reports the app as
healthy on the network and notes that the public URL does not answer yet.

`init` generates configuration without Docker access. `doctor` checks prerequisites,
application configuration and the generated Compose model without starting services.
`--dry-run` describes any operation without writing files or contacting Docker.

Deployments show six numbered phases on stderr: source update, configuration,
runtime image, data protection, service health, and backup history. Each phase
has a distinct colored heading and reports elapsed time. UTC start and finish
timestamps accompany the total duration in seconds and hours/minutes/seconds.
Git updates show the branch, previous and new short commit IDs, incoming count,
and every incoming commit in chronological order, without shortening subjects.
Subjects are rendered
as printable ASCII so repository metadata cannot issue terminal controls.
The updated script keeps the original arguments and phase count when it restarts
itself after pulling. `--no-pull` reports the current revision, or identifies a
source archive when Git metadata is unavailable.

Phase headings use cyan, blue, magenta, yellow, and green; completed steps are
green, warnings yellow, and errors red. Color is used only on an interactive
stderr terminal. Set
`NO_COLOR=1` to disable it; redirected logs and `TERM=dumb` remain plain. Command
data stays on stdout. Snapshot resume and restart use Compose's
`up -d --no-recreate --wait --wait-timeout` to wait for the existing containers;
Compose 2.30 does not support `start --wait`.

## Configuration and exposure

The default state directory is `.deploy.local` beside the script. Its `*.local`
name is already excluded by both `.gitignore` and `.dockerignore`. Custom state
directories inside the repository must also sit beneath a top-level `*.local`
directory; directories outside the repository are supported. Files use mode `0600`
and the state directory uses `0700` on filesystems supporting Unix permissions.

- `deploy.env`: project, mode, URL, port, exposure, proxy network, Caddy
  image/network/ACME settings, the app's memory and CPU limits, the mail server
  (`UPSTREAM_DEFAULT`), the image repository `--image-tag` uses
  (`IMAGE_REPOSITORY`), and the backup and alert settings (`BACKUP_KEEP`,
  `BACKUP_KEEP_DAYS`, `BACKUP_RECIPIENT`, `BACKUP_HOOK`, `ALERT_HOOK`; see
  [backups.md](backups.md) and [monitoring.md](monitoring.md)).
- `app.env`: persistent random `MW_SERVER_KEY`, random admin password, and optional
  application environment variables. Values are **literal, unquoted** text; dollar
  signs and spaces are preserved. Do not add `export`, quotes or shell commands.
- `active/`: the last applied Compose/configuration and exact local image IDs.
  Edit the parent files, then run `deploy`; `start`/`restart` use the applied files.
- `backups/`: data and configuration snapshots. `rollback.path` identifies the
  snapshot taken before the most recent deployment attempt.

New installations receive a readable random username, `admin-` followed by 12 hex
characters. Setup prints the username and saves it as `MW_ADMIN_USER`; existing
installations keep their configured name across deploys and upgrades. Retrieve its generated password privately
from `app.env`. `init` also writes a random `MW_METRICS_TOKEN`, the bearer token
for `/metrics`; [monitoring.md](monitoring.md) covers metrics, `check` and
`ALERT_HOOK`. Mailbox users authenticate against their mail provider. Secrets are
never printed by the script. Preserve `MW_SERVER_KEY`: it seals credentials and
uploads. A deploy refuses to change that key under an existing data volume.

The script manages database/upload paths, runtime binaries, listen address, mode,
public URL and cookie security. SQLite and uploads always live together under
`/data` in a persistent named volume, owned by UID/GID 65532. The app runs non-root
with a read-only root filesystem, dropped capabilities, no-new-privileges, a PID
limit, log rotation, and writable temporary space. No dev/test services start.

`MEMORY_LIMIT` (a size such as `1g` or `768m`) and `CPU_LIMIT` (a number of CPUs
such as `2` or `1.5`) in `deploy.env` become the app container's `mem_limit` and
`cpus`. A new installation gets `MEMORY_LIMIT=1g` and `CPU_LIMIT=2`, the limits of
the Helm chart. An empty or missing key means no limit; a `deploy.env` written
before these keys existed has neither, and every `deploy` says so in one line
until they are added. A `CPU_LIMIT` above the number of CPUs Docker can use is
lowered to that number, with a message, because Docker refuses it otherwise. When
the app reaches its memory limit the kernel ends it and Docker restarts it; Caddy
has no limits.

Local and external-proxy modes bind HTTP to `127.0.0.1` only. Local mode uses HTTP
cookies; both HTTPS modes enable Secure cookies. Caddy mode publishes only ports
80/443, renews certificates automatically, and forwards WebSocket/SSE. Its private
network defaults to `172.30.70.0/24`; use `--proxy-prefix 172.30.71` at initialization
if that subnet overlaps another network. Only Caddy's fixed `.2` address is trusted
for forwarded headers. Caddy certificates persist in separate named volumes.

In Caddy mode, `ACME_CA` in `deploy.env` names another ACME directory by its
`https://` URL, for example Let's Encrypt's staging endpoint
`https://acme-staging-v02.api.letsencrypt.org/directory` or a private CA; empty
keeps Caddy's default issuers. `ACME_CA_ROOT` is the absolute path of a PEM file
with the root certificate that signs that directory's own HTTPS certificate, for a
CA whose root is not in the Caddy image's trust store. The script makes them Caddy's
`acme_ca` / `acme_ca_root` global options. The Caddyfile and the certificates from
the PEM file are environment values of the Caddy service in the generated Compose
file, so snapshots keep them; its entrypoint writes both to the container's
temporary filesystem and starts Caddy (Compose does not mount inline files into a
service whose root filesystem is read-only). The PEM file must exist whenever
`doctor`, `deploy`, `restore` or `rollback` renders the configuration. Both keys
are refused for the other exposures. Browsers do not trust certificates from a staging endpoint or a
private CA unless their root is installed on the client.

With an external proxy, configure `MW_TRUSTED_PROXIES` and `MW_FORWARDED_MODE` in
`app.env` using the proxy's actual source address as seen by the container. See
[reverse-proxy.md](reverse-proxy.md). With `--proxy-network`, the script sets
`MW_TRUSTED_PROXIES` to that network's subnets and `MW_FORWARDED_MODE=xff` unless
`app.env` sets either one. Every container on that network can then state a
client address, so keep the network to the proxy and the apps behind it, or pin
the proxy's own address in `app.env`. `PROXY_NETWORK` in `deploy.env` may be
changed later; the next `deploy` moves the app to the named network.

An applied installation may move among local, external-proxy and proxy-network
exposure: edit `EXPOSURE` (`local`, `external` or `network`), `PUBLIC_URL`, `PORT`
and `PROXY_NETWORK` in `deploy.env`, then run `deploy`. The project name and the
Caddy subnet are fixed after the first deployment, and an installation cannot
move to or from Caddy mode; create another deployment for those changes.
Changing the public URL can invalidate passkeys bound to the previous origin.

To put an existing installation on a reverse proxy's Docker network:

1. In `.deploy.local/deploy.env` set `EXPOSURE=network`, `PROXY_NETWORK=<name>`
   and `PUBLIC_URL` to the `https://` address the proxy serves.
2. Run `bash deploy.sh deploy --no-build`.
3. Point the proxy at `http://<project>:8080` (web UI and API; WebSocket support
   is needed). The app is no longer published on the host port.

Set `UPSTREAM_DEFAULT` in `deploy.env` to the mail server users sign in to: an
`http(s)://` JMAP URL in proxy mode, an `imaps://`, `imap://`, `pop3s://` or
`pop3://` server URL in engine mode (`doctor` refuses a URL for the other mode).
The script passes it to the app as `MW_UPSTREAM_DEFAULT`. The sign-in form has no
server field, so without it nobody can sign in. An installation that already sets
`MW_UPSTREAM_DEFAULT` in `app.env` keeps working; setting both is refused. Set
`MW_JMAP_UPSTREAMS` for private/internal JMAP origins (or to restrict allowed
public providers). In engine mode, use `MW_SMTP_HOST`, `MW_SMTP_PORT`, and
`MW_SMTP_SECURITY` when submission differs from the mailbox server. Optional
external cache, SSO, observability and integration settings go in `app.env`; see
the [deployment reference](README.md). PostgreSQL, custom volume mounts, in-app
TLS and sub-path hosting need a separately managed Compose deployment; this
script deliberately keeps its backup/restore contract to SQLite plus `/data`.

### Country database (optional)

Country sign-in policies need a local country database. `bash deploy.sh init
--geoip` (or `bash deploy.sh geoip` at any later time) downloads DB-IP "IP to
Country Lite" on the host: HTTPS only (`curl --proto =https`), this month's file
or last month's, at most 128 MB, unpacked to at most 256 MB and checked for the
MaxMind DB marker. It is saved as `<state>/geoip/dbip-country-lite.mmdb`. The next
`deploy` mounts `<state>/geoip` read-only at `/geoip-seed` and sets
`MW_GEOIP_SEED`; the server installs the file (after validating it) only when its
data directory `/data/geoip`, on the `data` volume, holds no database yet. From
then on the admin console (Security → Country database) downloads updates on its
schedule. The data is licensed CC BY 4.0: show "IP Geolocation by DB-IP" with a
link to https://db-ip.com where country results are used. See
[Country database](../admin/country-database.md) for the other providers, the
data directory and `MW_GEOIP_DB`.

## Updates and daily operations

```sh
bash deploy.sh --help
bash deploy.sh status               # also says whether /metrics is enabled
bash deploy.sh check                 # exit 0 when healthy, else ALERT_HOOK + exit 1
bash deploy.sh logs                  # Ctrl-C exits log following
bash deploy.sh stop                  # keeps containers and all data
bash deploy.sh start
bash deploy.sh restart

# Update the current branch from its configured Git upstream:
bash deploy.sh deploy                # fast-forward, then build before stopping the old app
bash deploy.sh deploy --no-pull      # deploy exactly the current checkout
bash deploy.sh deploy --no-build     # apply configuration, reuse the app image

# Or supply an image containing the same runtime binaries/plugins:
bash deploy.sh deploy --image registry.example.org/mailwoman:VERSION --pull
# The published image, refused unless its signature verifies:
bash deploy.sh deploy --image ghcr.io/supermarsx/mailwoman:master --pull --require-signature
# With IMAGE_REPOSITORY=ghcr.io/supermarsx/mailwoman in deploy.env, the same as
# --image ghcr.io/supermarsx/mailwoman:master --pull:
bash deploy.sh deploy --image-tag master --require-signature
```

`deploy` runs `git pull --ff-only` before reading deployment configuration or
touching containers, then restarts the script from the updated checkout with the
same arguments. Local tracked, staged, and untracked edits are retained when Git
can fast-forward without overwriting them. An already-current dirty checkout can
deploy too. Git refuses overlapping edits or incoming path collisions. Detached
HEAD, a missing upstream, divergence, or a failed fetch also stop deployment.
The script never stashes, resets, cleans, or creates a merge commit; an existing
`pull.rebase` setting does not change this behavior.
Use `--no-pull` for an intentional local revision, a source archive, or an offline
deployment. `--dry-run` only describes the Git update. Other commands do not pull.
The existing `--pull` option fetches a **container image**, independently of Git.

`--image-tag TAG` takes the repository from `IMAGE_REPOSITORY` in `deploy.env`
(a repository without tag or digest), so an update script or a host update agent
passes only the tag and never a registry address. It always fetches from the
registry, so the signature check below applies. None of the commands asks
questions; all of them can run unattended.

Each deployment sets `MW_IMAGE_DIGEST` in the app's environment to the image it
runs: `<repository>@sha256:...` when the `--image` (or
`--image-tag`) has a registry digest for its repository, otherwise the local
image ID `sha256:...` (always so for an image built from source). It is set by
the script; `app.env` may not set it. After every snapshot (`backup`, the one a
`deploy` takes, the one `restore` takes of the installation it replaces) the
script writes `/data/last-snapshot.json` into the app's data volume, so the app
can tell when the last snapshot was taken:

```json
{"schema":"mailwoman-deploy-snapshot/1","kind":"backup","takenAt":"2026-10-25T03:30:00Z","path":"/srv/mailwoman/.deploy.local/backups/20261025T033000Z-4242-17","encrypted":false}
```

`kind` is `backup`, `pre-deploy` or `pre-restore`; `path` is the snapshot
directory or encrypted archive on the host. The file holds no secret and is not
needed for a restore.

The build selects the Dockerfile target `runtime-static`, not the Dockerfile's
final stage: a `scratch` image with statically linked binaries, the target CI
scans, signs and publishes, so an installation built from source has the shape
of the published image. `--image` uses a locally present image unless `--pull`
is supplied; a missing image is pulled. Each deployment records immutable image
IDs.

With `--image`, the script checks the image's keyless signature when `cosign` is
on `PATH` and the deploy fetches the image from its registry (with `--pull`, or
because it is not in the local image store). `cosign verify` must find a
signature whose certificate names `SIGNATURE_IDENTITY` and `SIGNATURE_ISSUER` from
`deploy.env`; empty values mean this repository's publishing workflow,
`https://github.com/supermarsx/mailwoman/.github/workflows/supply-chain.yml@refs/heads/master`,
and GitHub's issuer `https://token.actions.githubusercontent.com`. The deploy then
uses the exact digest that was verified. A failed verification stops the deploy
before any snapshot or container change. Without `cosign`, or for an image used
from the local image store without `--pull` (Docker cannot tell whether a local
copy came from a registry), the script prints that the signature was not checked
and continues; `--require-signature` refuses such an image instead.
Images that `deploy.sh` builds from source are not signature-checked. A fork that
publishes its own images sets `SIGNATURE_IDENTITY` (and `SIGNATURE_ISSUER` for
another CI) in `deploy.env`. The workflow pushes an image before it signs it, so
for a few minutes after a push to `master` the `master` tag can name an image
that is not signed yet; a deploy of it is refused until the signature exists.

A new installation's `CADDY_IMAGE` names an exact Caddy version and its
multi-platform index digest (`caddy:2.11.7-alpine@sha256:…`), so every deploy uses
the same Caddy bytes until that line changes. A `deploy.env` written before 26.20
keeps the value it has (`caddy:2-alpine`, a moving tag). To update Caddy, set
`CADDY_IMAGE` to a new version and digest (`docker buildx imagetools inspect
caddy:<version>-alpine` prints the digest), then deploy. Multiple installations need distinct project
names, state directories, ports, and Caddy subnets; only one can own ports 80/443.

Before replacing an existing app, the script snapshots it. Building/pulling fails
before any outage. Snapshotting briefly stops and resumes the app. Compose then
waits for the new container's health check, and the script checks `/healthz` and
`/` at the configured public URL, with valid TLS when applicable. `--timeout 300`
extends each startup/public-check deadline. These checks prove app/HTTP readiness,
not mailbox login, SMTP delivery, external integrations or off-host reachability.

When an update fails Compose's health wait or the public check, the script rolls
it back: it keeps the failed release's files and its last 300 log lines in
`failed/<time>-<pid>/`, stops it, and starts the previous release — its image,
configuration and exposure (Compose file, port or proxy network, trust settings)
— on a copy of the snapshot taken just before the update, in a new volume.
Whatever the failed release wrote, database migrations included, stays in the
volume it used, which is kept. The command still exits nonzero and names what it
restored and where. `deploy.env` and `app.env` keep the settings that failed:
correct them, or the image, and deploy again. Because the public check is part of
this, an update whose app is healthy but whose public URL does not answer (DNS,
firewall or proxy not ready) is rolled back too; with `network` exposure an
unanswered public URL only produces a note. `--no-auto-rollback` leaves a failed
update in place for diagnosis; `bash deploy.sh rollback` then restores the same
snapshot. A first installation has nothing to roll back to: on failure it keeps
its data volume and containers, and after fixing DNS, firewall or proxy, `start`
checks it again.
The script never runs `git pull` or installs host packages. It deletes data
volumes and images only when asked with `prune --replaced --yes`
([backups.md](backups.md)). A directory lock prevents concurrent mutations; after SIGKILL or a
host crash, remove a stale `.lock` only after verifying no operation is running.

## Backups and recovery

```sh
bash deploy.sh backup
bash deploy.sh rollback
bash deploy.sh restore .deploy.local/backups/TIMESTAMP-PID-RANDOM
```

A backup stops the app writer and copies **all of `/data`**, including SQLite WAL
files and attachments, with numeric ownership. It also captures the applied
configuration, encryption key and exact image IDs. A checksum manifest and a final
`COMPLETE` marker distinguish usable snapshots from interrupted copies. The app is
resumed even if copying fails or the operation receives SIGINT/SIGTERM.

Restore validates the snapshot, populates a **new volume**, snapshots the current
installation (which then stays stopped, so a release that cannot become healthy
can still be replaced), and starts the saved version/configuration against the
recovered data. It reverts all writes since the snapshot time. The backup
settings in `deploy.env` (`BACKUP_*`, `ALERT_HOOK`) are not taken from the
snapshot: they stay as they are. The old volume and the
pre-restore snapshot remain available. `rollback` restores the pre-deployment
snapshot, including the old image and data; it is not just an image downgrade.
Snapshots and old volumes are kept until `prune` removes them: the retention
keys `BACKUP_KEEP` / `BACKUP_KEEP_DAYS` in `deploy.env`, and
`prune --replaced --yes` for replaced volumes and old images, are described in
[backups.md](backups.md).

Backups contain plaintext configuration secrets and potentially sensitive data;
copy them to private, encrypted off-host storage. With `BACKUP_RECIPIENT` in
`deploy.env`, `backup` writes one archive encrypted to an age or gpg public key
instead, and `restore` takes that archive (`--identity FILE` for age); see
[backups.md](backups.md). Checksums detect accidental
corruption, not malicious replacement: restore only trusted snapshots. Caddy's
certificate volumes are not included; preserve them separately if required, or
allow Caddy to reissue certificates on a replacement host.

Images are **not** inside snapshots. Before pruning Docker images, retain every
image needed for recovery with `docker image save`, including Caddy for HTTPS
deployments. Transfer/load those archives on a replacement host, then:

```sh
# Choose the original project's name; the restore replaces this generated config.
bash deploy.sh init --project mailwoman
bash deploy.sh restore /secure/path/to/snapshot
```

Recover on a compatible container architecture. Test restores periodically and
retain the state directory together with the image archives and data snapshots.

The repository also includes an isolated Docker lifecycle test:

```sh
docker build --target runtime-static -t mailwoman:deploy-verify .
bash scripts/test-deploy.sh mailwoman:deploy-verify
```

It uses random test projects and localhost port 18089 (override with
`DEPLOY_TEST_PORT`; the two ports above it must also be free). It checks that
`--dry-run` writes nothing, the port auto-pick (when localhost:8080 is free, a
throwaway container holds it for the few seconds of one `init`), the admin panel
under strict CSRF with curl and, when Playwright can start, in a browser, the
refusal of a deploy onto a port held by something else, backup/rollback, failure
recovery, the moves local → external → proxy network → local, the refusal of a move
to Caddy, the automatic rollback of an update to an image that never becomes
healthy (it also changes the exposure, and the previous exposure must come back)
and `rollback` after an update made with `--no-auto-rollback`, a new Caddy
installation's `CADDY_IMAGE` pinned by version and digest, a second,
proxy-network installation, the `--image` signature check (a stand-in `cosign`
and a registry container: verified digest deployed, failed verification refused
before any change, `--require-signature` without `cosign` or for an image from the
local store refused), and the `ACME_CA` / `ACME_CA_ROOT`
lines in a Caddy installation's generated Compose file (read with `doctor`;
invalid values refused), and the memory and CPU limits Docker applies (defaults,
edited values, a CPU limit above the host's lowered, none for a `deploy.env`
without the keys), `UPSTREAM_DEFAULT`, `MW_IMAGE_DIGEST` and `--image-tag`,
`last-snapshot.json`, the generated metrics token (`/metrics` 200 with it, 401
without), `check` and `ALERT_HOOK` with the app stopped, `prune` (the rollback
snapshot kept; a replaced volume and an old image deleted only with
`--replaced --yes`), encrypted backups with each of gpg and age that is
installed (restored with the key, refused without it; with neither installed
the case is skipped, a failure under `DEPLOY_TEST_REQUIRE=all`), `BACKUP_HOOK`,
and the `timers` units under `systemd-analyze verify` in a container (which
needs network access to install systemd there). The external step stands in a
curl wrapper for the host's reverse proxy; nothing in the test reaches the
Internet or uses ports 80/443. It removes its test containers, volumes and network
afterward; failed runs retain local diagnostics.

The browser step runs `pnpm exec playwright` from `apps/web`, or
`apps/web/node_modules/.bin/playwright` when pnpm cannot start it, and prints
Playwright's summary. Without either the step is skipped and the `PASS` line says
so; `DEPLOY_TEST_REQUIRE=all` makes that a failure, reported before anything is
deployed. `DEPLOY_TEST_SCRIPT=/path/to/deploy.sh` runs the test against another
copy of the script, for example one with a seeded defect.

Implementation references: [Compose raw environment files](https://docs.docker.com/reference/compose-file/services/#env_file),
[Compose health waiting](https://docs.docker.com/reference/cli/docker/compose/up/),
and [Caddy reverse proxy](https://caddyserver.com/docs/caddyfile/directives/reverse_proxy).
