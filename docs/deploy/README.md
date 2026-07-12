# Deploying Mailwoman

Mailwoman ships as a single static-ish binary (`mailwoman`) plus its render
worker (`mw-render`). It serves the SPA (embedded), a session-authed JMAP proxy,
and the `/api/sanitize` boundary. This directory has three deployment aids:

- `Dockerfile` (repo root) — multi-stage build, distroless non-root runtime.
- `mailwoman.service` — a hardened systemd unit (SPEC §7.5).
- `nginx.conf` — a TLS-terminating reverse-proxy snippet.

## Configuration (environment)

| Env | Default | Meaning |
|-----|---------|---------|
| `MW_BIND` | `0.0.0.0:8080` | Listen address for the HTTP server. |
| `MW_DB_PATH` | `mailwoman.db` | SQLite file (sessions + settings + sealed creds). |
| `MW_SERVER_KEY` | *(ephemeral)* | Hex-encoded 32-byte key sealing upstream creds. **Set it** in production so sessions survive restarts; keep it secret. Generate: `openssl rand -hex 32`. |
| `MW_RENDER_BIN` | *(auto-detect)* | Path to `mw-render`. The image sets `/usr/local/bin/mw-render`. |
| `MW_WEB_DIR` | *(embedded)* | Serve the SPA from disk instead of the embedded copy (dev override). |
| `MW_COOKIE_SECURE` | `false` | Mark the session cookie `Secure`. **Set `true` behind TLS** (i.e. always in production). |
| `RUST_LOG` | `info` | Tracing filter. |

## Docker

```sh
docker build -t mailwoman:local .
docker run --rm -p 8080:8080 \
  -e MW_SERVER_KEY="$(openssl rand -hex 32)" \
  -e MW_COOKIE_SECURE=true \
  -v mailwoman-data:/data \
  mailwoman:local
```

The container runs as the non-root `nonroot` user (uid 65532), writes only to
the `/data` volume, and exposes a `HEALTHCHECK` via `mailwoman healthcheck`.

## systemd (bare metal)

Install the two binaries to `/usr/local/bin`, create a dedicated user and data
dir, then install the unit:

```sh
sudo useradd --system --no-create-home --shell /usr/sbin/nologin mailwoman
sudo install -d -o mailwoman -g mailwoman /var/lib/mailwoman
sudo install -m0755 target/release/mailwoman  /usr/local/bin/mailwoman
sudo install -m0755 target/release/mw-render   /usr/local/bin/mw-render
sudo install -m0644 docs/deploy/mailwoman.service /etc/systemd/system/mailwoman.service
sudo systemctl edit mailwoman   # drop in [Service] Environment=MW_SERVER_KEY=...
sudo systemctl enable --now mailwoman
```

Keep `MW_SERVER_KEY` out of the unit file itself — use a drop-in or
`EnvironmentFile=` with `0600` permissions.

## Reverse proxy (TLS)

Terminate TLS at nginx (or Caddy/Traefik) and proxy to `127.0.0.1:8080`. See
`nginx.conf`. Set `MW_COOKIE_SECURE=true` so the session cookie is only sent
over HTTPS. The proxy must forward the `Cookie`/`Set-Cookie` headers verbatim.

## Backends

Mailwoman is backend-agnostic: the JMAP server URL is entered at login and the
server proxies to it. For local development and E2E, `docker-compose.dev.yml`
provides the in-repo `mw-mock-jmap` (default, deterministic) and an optional,
profile-gated Stalwart service (`--profile stalwart`, experimental — see the
compose file and `scripts/stalwart-seed.sh`).
