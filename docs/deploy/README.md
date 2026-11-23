# Install and configure

This page covers the ways to install the Mailwoman server, the configuration every installation needs, and where to go next for TLS, proxies, databases and operations. If you want the shortest path to a running server, follow [Quick install](../getting-started/install-quick.md) instead.

Mailwoman is the webmail client and its server; it does not receive or deliver mail. Users sign in to a mail server you already run, which you name in the configuration (see [Which mail server users sign in to](#which-mail-server-users-sign-in-to)).

## Choose an installation method

| Method | Use it when | Guide |
| --- | --- | --- |
| `deploy.sh` | One Linux host with Docker. The script writes the configuration, builds or pulls the image, adds Caddy for HTTPS if you want, and handles updates, backups, restores and rollbacks. | [Deploy with deploy.sh](script.md) |
| Docker image | You run containers your own way (Compose, a hosting panel, an orchestrator). | [Docker](#docker) below |
| Helm chart | Kubernetes. | [Packaging](packaging.md) |
| Binaries under systemd | A Linux server without containers. | [systemd](#systemd) below |

Every method runs the same two programs: `mailwoman` (the server, which also serves the web app) and `mw-render` (a worker the server starts to process untrusted message content).

## Docker

The repository's `Dockerfile` builds the web app and the server and produces a minimal image with no shell: the server runs as the unprivileged user 65532, writes only to the `/data` volume, listens on port 8080 and has a health check (`mailwoman healthcheck`, which probes `/healthz`).

1. Build the image from a checkout. The build needs the commit it is built from:

   ```sh
   docker build --build-arg MW_BUILD_REVISION="$(git rev-parse HEAD)" -t mailwoman:local .
   ```

   The default (last) stage is `runtime-static`, the same image `deploy.sh` uses. `--target runtime` builds a variant on a distroless base instead.

2. Create an environment file, `mailwoman.env`, readable only by you (`chmod 600`):

   ```ini
   MW_SERVER_KEY=<output of: openssl rand -hex 32>
   MW_UPSTREAM_DEFAULT=https://jmap.example.org
   MW_ADMIN_USER=<an admin username>
   MW_ADMIN_PASSWORD=<a long random password>
   ```

   For an IMAP or POP3 server, add `MW_MODE=engine` and set `MW_UPSTREAM_DEFAULT` to an `imaps://`, `imap://`, `pop3s://` or `pop3://` URL.

3. Start the container:

   ```sh
   docker run -d --name mailwoman --restart unless-stopped \
     -p 127.0.0.1:8080:8080 \
     --env-file mailwoman.env \
     -v mailwoman-data:/data \
     mailwoman:local
   ```

4. Find the admin console's address. A new installation generates a random console path on its first start and logs it once, in a warning that starts `new installation: the admin console is at`. You can print it at any time:

   ```sh
   docker exec mailwoman mailwoman admin console-path show
   ```

5. Put a TLS-terminating reverse proxy in front of port 8080 (see [Reverse proxy and TLS](#reverse-proxy-and-tls)), then sign in at the console path with `MW_ADMIN_USER` and `MW_ADMIN_PASSWORD`. That first sign-in creates the first admin account, with the Super admin role.

The image sets `MW_BIND=0.0.0.0:8080`, `MW_DB_PATH=/data/mailwoman.db`, `MW_RENDER_BIN`, `MW_WEB_DIR` and `MW_PLUGIN_DIR`; you do not need to set them.

::: warning
Keep `MW_SERVER_KEY` the same for the life of the installation and back it up with the volume. Generating a new key at each start (for example with `$(openssl rand -hex 32)` on the `docker run` line) makes every stored secret unreadable.
:::

## systemd

`docs/deploy/mailwoman.service` is a hardened unit that runs `mailwoman serve` as the user `mailwoman`, listens on `127.0.0.1:8080`, keeps its database in `/var/lib/mailwoman` and sets `MW_COOKIE_SECURE=true` for use behind a TLS proxy.

1. Build the web app first (the server embeds it at compile time), then the two binaries. You need Node.js with pnpm and the Rust toolchain named in `rust-toolchain.toml`:

   ```sh
   pnpm -C apps/web install --frozen-lockfile
   pnpm -C apps/web build
   cargo build --release --locked --bin mailwoman --bin mw-render
   ```

2. Install the binaries, the first-party plugins, a system user and the unit:

   ```sh
   sudo useradd --system --no-create-home --shell /usr/sbin/nologin mailwoman
   sudo install -m0755 target/release/mailwoman target/release/mw-render /usr/local/bin/
   sudo install -d /usr/lib/mailwoman/plugins
   sudo install -m0644 plugins/dist/*.wasm /usr/lib/mailwoman/plugins/
   sudo install -m0644 docs/deploy/mailwoman.service /etc/systemd/system/mailwoman.service
   ```

   The unit's `StateDirectory=mailwoman` creates `/var/lib/mailwoman` owned by that user when the service starts.

3. Put the secrets and the mail server in an environment file readable only by root, for example `/etc/mailwoman/mailwoman.env` with mode `0600` (same variables as the Docker example above), and point the unit at it:

   ```sh
   sudo systemctl edit mailwoman
   # add:
   # [Service]
   # EnvironmentFile=/etc/mailwoman/mailwoman.env
   ```

4. Start it, and read the generated console path from the journal:

   ```sh
   sudo systemctl daemon-reload
   sudo systemctl enable --now mailwoman
   sudo journalctl -u mailwoman | grep 'the admin console is at'
   ```

   `mailwoman admin console-path show` prints it too, when run as the `mailwoman` user with the same `MW_DB_PATH` and `MW_SERVER_KEY` as the service.

Keep `MW_SERVER_KEY` out of the unit file itself.

## Configuration

The server reads its configuration from environment variables (most `serve` options also exist as command-line flags; run `mailwoman serve --help`). Many settings can also be changed in the admin console once the server runs; the [configuration reference](../admin/configuration.md) lists every variable, its default, and whether the console can change it.

The variables most installations set:

| Variable | Default | What it does |
| --- | --- | --- |
| `MW_BIND` | `0.0.0.0:8080` | Listen address. |
| `MW_DB_PATH` | `mailwoman.db` | The database: a SQLite file path, or a `postgres://` URL for [PostgreSQL](postgres.md). |
| `MW_SERVER_KEY` | a key file beside the SQLite database | 64 hex characters (32 bytes) that seal every stored secret. See [The server key](#the-server-key). |
| `MW_SERVER_KEY_FILE` | `<database>.server-key` | A file holding the key, read or created at start. PostgreSQL needs this or `MW_SERVER_KEY`. |
| `MW_MODE` | `proxy` | `proxy` for a JMAP mail server, `engine` for IMAP or POP3 with SMTP. |
| `MW_UPSTREAM_DEFAULT` | unset | The mail server users sign in to, unless a domain configured in the console names its own. |
| `MW_JMAP_UPSTREAMS` | unset | Proxy mode: comma-separated JMAP origins the server may connect to. Needed for a JMAP server on a private address that is not `MW_UPSTREAM_DEFAULT`. |
| `MW_SMTP_HOST`, `MW_SMTP_PORT`, `MW_SMTP_SECURITY` | the IMAP/POP3 host; `587`, `465` with `implicit`, `25` with `plaintext`; `starttls` | Engine mode: where mail is sent. |
| `MW_ADMIN_USER`, `MW_ADMIN_PASSWORD` | unset | The bootstrap admin credential. Its first sign-in creates a Super admin account. |
| `MW_ADMIN_PATH` | the stored path | The admin console's path. See [Moving the admin console](#moving-the-admin-console). |
| `MW_COOKIE_SECURE` | derived from the request's scheme | `true` marks cookies `Secure`. Set it behind a TLS proxy unless the server trusts the proxy's forwarded headers. |
| `MW_PUBLIC_URL` | the request's `Host` | The public address, for example `https://mail.example.org`. See [Reverse proxy](reverse-proxy.md) before setting it on a server that already has passkeys. |
| `MW_TRUSTED_PROXIES`, `MW_FORWARDED_MODE` | empty, `off` | Which proxies may state the client address and scheme, and through which header. See [Reverse proxy](reverse-proxy.md). |
| `RUST_LOG` | `info` | Log filter. See [Telemetry](telemetry.md). |

### The server key

Mailwoman seals stored secrets, including session records, with the server key. Sessions survive restarts as long as the database and the key are kept.

- With SQLite and no key configured, the server creates a key file beside the database (`mailwoman.db.server-key`) on the first start and reuses it. Back up both files together. On Unix the file must have private permissions (`0600`). On Windows the generated file is protected with DPAPI and can only be opened by the same Windows account; use `MW_SERVER_KEY` if you move the data to another account or host.
- PostgreSQL requires `MW_SERVER_KEY` or `MW_SERVER_KEY_FILE`, and every server sharing the database must use the same key.
- `MW_SERVER_KEY` overrides any key file. Generate one with `openssl rand -hex 32`.

::: danger
Do not delete or replace the key to get past a start-up error. A different key cannot open what the old one sealed.
:::

## Which mail server users sign in to

The sign-in form asks only for an address and a password; the server decides where that address signs in, in this order:

1. The domain's own connection settings, if the address's domain is configured and enabled in the admin console (**Domains**). Aliases of a domain use its settings.
2. The deployment default: the `upstream.default` setting, otherwise `MW_UPSTREAM_DEFAULT`.
3. Proxy mode only: the single origin in `MW_JMAP_UPSTREAMS`, when it lists exactly one.

If none applies, the sign-in answers "This address is not set up on this server. Contact your administrator." The server never looks the address's domain up in DNS at sign-in.

In proxy mode the server also limits which JMAP servers it connects to:

- An origin named in `MW_JMAP_UPSTREAMS` or `MW_UPSTREAM_DEFAULT` may be on any address, including a private one such as another container (`http://stalwart:8080`).
- When `MW_JMAP_UPSTREAMS` is set, other origins are refused. When it is unset, other origins are allowed only on public addresses.
- The JMAP session's API, download and upload URLs must be on the same origin as the upstream.

## Moving the admin console

The admin console, its sign-in and its API answer at one path only; every other path, `/admin` included, answers like any unknown path. The path comes from, in order:

1. `MW_ADMIN_PATH`, when set. The console and the CLI cannot change it while it is set.
2. The path stored in the database.
3. On the first start of a new installation, a generated path of 20 random characters, which the server stores and logs once. (`deploy.sh init` generates one and writes it to `app.env` as `MW_ADMIN_PATH` instead.)

To print the path: `mailwoman admin console-path show` (with Docker, `docker exec <container> mailwoman admin console-path show`).

To change it, use the console (**Security centre** > **Console address**; you confirm your identity first), or the CLI:

```sh
mailwoman admin console-path generate          # a new random path
mailwoman admin console-path set /ops-7f3k-console
```

A path you choose needs at least 12 characters after the `/`, and cannot be `/admin`. The new path takes effect at the next restart; that start ends every admin session, so admins sign in again at the new address. Update bookmarks, proxy rules and monitoring that use the old path.

::: warning
A hidden path keeps scanners and casual sign-in attempts away from the console, but it does not isolate it: the console runs on the same origin as the mail app. Keep a strong admin password, two-factor sign-in for admin accounts and, where you can, a network restriction on the path in your reverse proxy.
:::

![The Console address tab of the Security centre: what the console address does, the current address (masked) and buttons to copy, share or email it.](/screenshots/deploy/admin-console-address.webp){.mw-crop}

## Reverse proxy and TLS

Run Mailwoman behind a reverse proxy that terminates TLS, or let the server get certificates itself:

- [Reverse proxy](reverse-proxy.md): what the server needs from a proxy (WebSocket and event-stream pass-through, forwarded headers and `MW_TRUSTED_PROXIES`, sub-path hosting), with [example configurations](proxy/README.md) for nginx, Apache, Caddy, Traefik, HAProxy, Envoy and IIS. `docs/deploy/nginx.conf` is a single-file nginx example.
- [ACME and certificates](acme.md): built-in Let's Encrypt (`--acme`) or your own certificate files.
- [WebSocket and event streams](websocket.md).

Until the proxy's address is in `MW_TRUSTED_PROXIES`, the server ignores its forwarded headers: it does not see that the original request was HTTPS, and it records every client as the proxy's address (audit log, rate limits, bans). `MW_TRUSTED_PROXIES` alone is enough for the scheme and host (`X-Forwarded-Proto`, `X-Forwarded-Host`); the client address also needs `MW_FORWARDED_MODE` (`xff` or `forwarded`), which picks the header it is read from.

## Operating the server

- Databases and caches: [PostgreSQL](postgres.md), [Valkey/Redis cache](cache.md), [online SQLite maintenance](online-sqlite-maintenance.md).
- Backups and recovery: [Backups](backups.md).
- Health, metrics and logs: [Monitoring](monitoring.md), [Telemetry](telemetry.md).
- Connections and outbound traffic: [Connection limits](connection-limits.md), [Outbound proxy](egress-proxy.md).
- Background work: [Scheduler](scheduler-runtime.md), [Sieve](sieve-runtime.md).
- Push notifications: [Push relay](push.md).
- Desktop and mobile apps: [Desktop and mobile shells](shells.md), [Desktop](desktop.md), [Android](mobile-android.md).
- Mail servers: [IMAP and POP3](imap-pop3.md), [CalDAV and CardDAV](caldav-carddav.md).

## Development and test backends

`docker-compose.dev.yml` runs the server against test backends: the in-repository JMAP mock (`mw-mock-jmap`) for proxy mode, Greenmail and Dovecot for engine mode, and an optional Stalwart JMAP server (`--profile stalwart`). They are for development and automated tests, not for production.

## Upgrading

Before you update an existing installation, read the [upgrade notes](upgrade-notes/index.md): they list the changes that need action on your side.

## Related

- [Quick install](../getting-started/install-quick.md)
- [Deploy with deploy.sh](script.md)
- [Configuration reference](../admin/configuration.md)
- [The admin console](../admin/index.md)
- [Security model](../security/README.md)
