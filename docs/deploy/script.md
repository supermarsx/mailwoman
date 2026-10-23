# Root deployment script

[`deploy.sh`](../../deploy.sh) manages a single-host Docker deployment of the
Mailwoman web app: embedded SPA, Rust server, render worker, first-party plugins,
SQLite database and filesystem upload storage. It does not install a mail server.
Users connect their existing JMAP accounts, or IMAP/POP3 + SMTP in engine mode.

## First deployment

On the deployment host, install **Bash 4+**, **Docker with Linux containers and
Compose 2.30+**, `curl`, `tar`, and coreutils (`sha256sum`, `od`, `cmp`, etc.).
Run as a user who can access the Docker daemon. Host Rust, Node and pnpm are not
required: the repository's Dockerfile builds the web app and binaries. Use the
same local Docker daemon for every operation; remote Docker contexts are not
supported by the localhost exposure and verification steps.

Clone/check out the version you intend to deploy, then run from the repository:

```sh
# Automatic HTTPS: first point this hostname's A/AAAA records at this host.
# TCP ports 80 and 443 must be free and reachable from the Internet.
bash deploy.sh --domain mail.example.org --mode engine
```

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

## Configuration and exposure

The default state directory is `.deploy.local` beside the script. Its `*.local`
name is already excluded by both `.gitignore` and `.dockerignore`. Custom state
directories inside the repository must also sit beneath a top-level `*.local`
directory; directories outside the repository are supported. Files use mode `0600`
and the state directory uses `0700` on filesystems supporting Unix permissions.

- `deploy.env`: project, mode, URL, port, exposure, proxy network and Caddy
  image/network settings.
- `app.env`: persistent random `MW_SERVER_KEY`, random admin password, and optional
  application environment variables. Values are **literal, unquoted** text; dollar
  signs and spaces are preserved. Do not add `export`, quotes or shell commands.
- `active/`: the last applied Compose/configuration and exact local image IDs.
  Edit the parent files, then run `deploy`; `start`/`restart` use the applied files.
- `backups/`: data and configuration snapshots. `rollback.path` identifies the
  snapshot taken before the most recent deployment attempt.

The admin username is initially `admin`; retrieve its generated password privately
from `app.env`. Mailbox users authenticate against their mail provider. Secrets are
never printed by the script. Preserve `MW_SERVER_KEY`: it seals credentials and
uploads. A deploy refuses to change that key under an existing data volume.

The script manages database/upload paths, runtime binaries, listen address, mode,
public URL and cookie security. SQLite and uploads always live together under
`/data` in a persistent named volume, owned by UID/GID 65532. The app runs non-root
with a read-only root filesystem, dropped capabilities, no-new-privileges, a PID
limit, log rotation, and writable temporary space. No dev/test services start.

Local and external-proxy modes bind HTTP to `127.0.0.1` only. Local mode uses HTTP
cookies; both HTTPS modes enable Secure cookies. Caddy mode publishes only ports
80/443, renews certificates automatically, and forwards WebSocket/SSE. Its private
network defaults to `172.30.70.0/24`; use `--proxy-prefix 172.30.71` at initialization
if that subnet overlaps another network. Only Caddy's fixed `.2` address is trusted
for forwarded headers. Caddy certificates persist in separate named volumes.

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

Set `MW_JMAP_UPSTREAMS` for private/internal JMAP origins (or to restrict allowed
public providers). In engine mode, use `MW_SMTP_HOST`, `MW_SMTP_PORT`, and
`MW_SMTP_SECURITY` when submission differs from the mailbox server. Optional
external cache, SSO, observability and integration settings go in `app.env`; see
the [deployment reference](README.md). PostgreSQL, custom volume mounts, in-app
TLS and sub-path hosting need a separately managed Compose deployment; this
script deliberately keeps its backup/restore contract to SQLite plus `/data`.

## Updates and daily operations

```sh
bash deploy.sh --help
bash deploy.sh status
bash deploy.sh logs                  # Ctrl-C exits log following
bash deploy.sh stop                  # keeps containers and all data
bash deploy.sh start
bash deploy.sh restart

# After checking out the desired source revision:
bash deploy.sh deploy                # build before stopping the old app
bash deploy.sh deploy --no-build     # apply configuration, reuse the app image

# Or supply an image containing the same runtime binaries/plugins:
bash deploy.sh deploy --image registry.example.org/mailwoman:VERSION --pull
```

The build explicitly selects Dockerfile target `runtime`; it does not depend on
the Dockerfile's final stage. `--image` uses a locally present image unless `--pull`
is supplied; a missing image is pulled. Each deployment records immutable image
IDs. To update Caddy, pull the selected `CADDY_IMAGE` yourself (or edit that setting
to a new tag/digest), then deploy. Multiple installations need distinct project
names, state directories, ports, and Caddy subnets; only one can own ports 80/443.

Before replacing an existing app, the script snapshots it. Building/pulling fails
before any outage. Snapshotting briefly stops and resumes the app. Compose then
waits for the new container's health check, and the script checks `/healthz` and
`/` at the configured public URL, with valid TLS when applicable. `--timeout 300`
extends each startup/public-check deadline. These checks prove app/HTTP readiness,
not mailbox login, SMTP delivery, external integrations or off-host reachability.

On failure the script exits nonzero and retains data, backups and containers for
diagnosis. A public check can fail while the app is healthy if DNS/firewall/proxy
setup is incomplete. Fix that and run `start`. There is no automatic database
downgrade: inspect `logs` and explicitly restore a snapshot if recovery is needed.
The script never runs `git pull`, installs host packages, prunes images or deletes
data volumes. A directory lock prevents concurrent mutations; after SIGKILL or a
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
installation, and starts the saved version/configuration against the recovered
data. It reverts all writes since the snapshot time. The old volume and the
pre-restore snapshot remain available. `rollback` restores the pre-deployment
snapshot, including the old image and data; it is not just an image downgrade.
Snapshots and old volumes are not automatically expired.

Backups contain plaintext configuration secrets and potentially sensitive data;
copy them to private, encrypted off-host storage. Checksums detect accidental
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
docker build --target runtime -t mailwoman:deploy-verify .
bash scripts/test-deploy.sh mailwoman:deploy-verify
```

It uses random test projects and localhost port 18089 (override with
`DEPLOY_TEST_PORT`; the port above it must also be free). It checks that
`--dry-run` writes nothing, the port auto-pick (when localhost:8080 is free, a
throwaway container holds it for the few seconds of one `init`), the admin panel
under strict CSRF with curl and, when Playwright can start, in a browser, the
refusal of a deploy onto a port held by something else, backup/rollback, failure
recovery, the moves local → external → proxy network → local, the refusal of a move
to Caddy, and a second, proxy-network installation. The external step stands in a
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
