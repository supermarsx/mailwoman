# Deploying Mailwoman

Mailwoman ships as a single static-ish binary (`mailwoman`) plus its render
worker (`mw-render`). It serves the SPA (embedded), a session-authed JMAP
surface, and the `/api/sanitize` boundary. That JMAP surface runs in one of two
modes (`MW_MODE`):

- **proxy** (default, V0) — forwards to a JMAP upstream entered at login.
- **engine** (V1) — drives a real **IMAP/POP3 + SMTP** account locally through
  `mw-engine`, presenting the *same* JMAP surface to the SPA. See
  [`imap-pop3.md`](./imap-pop3.md) for pairing notes (Dovecot, Gmail-over-IMAP,
  POP3 hosts) and the testing backends (Greenmail).

This directory has these deployment aids:

- `Dockerfile` (repo root) — multi-stage build, distroless non-root runtime.
- `mailwoman.service` — a hardened systemd unit (SPEC §7.5).
- `nginx.conf` — a TLS-terminating reverse-proxy snippet (incl. WS/SSE).

V2 (realtime, TLS, fonts, hardening) adds:

- [`websocket.md`](./websocket.md) — reverse-proxy pass-through for the JMAP
  WebSocket (`/jmap/ws`) + EventSource fallback (`/jmap/eventsource`).
- [`acme.md`](./acme.md) — `--acme` (Let's Encrypt) and external-cert
  hot-reload on `SIGHUP`.
- [`fonts.md`](./fonts.md) — `mailwoman fonts pull` to self-host web fonts under
  `font-src 'self'`.
- [`hardening.md`](./hardening.md) — COEP/CORP/Permissions-Policy, CSRF, Origin
  checks, and session-timeout flags.

V3 (PIM: calendar, tasks, notes, contacts) adds:

- [`caldav-carddav.md`](./caldav-carddav.md) — CalDAV/CardDAV pairing (Radicale
  for testing; Nextcloud/Baïkal/Google notes), calendar/address-book **sharing**
  endpoints, **holiday** feeds, and the encrypted-at-rest notes posture.

V4 (crypto & security: OpenPGP/S/MIME, Security panel, DLP, max-security) adds:

- [`crypto-security.md`](./crypto-security.md) — the operator reference for the V4
  security features: WKD publishing (`MW_WKD_DIR`), DLP config (`MW_DLP_RULES`),
  ARF abuse reports (`MW_ABUSE_ADDRESS`/`MW_ABUSE_SPOOL`), and the screen-capture
  watermark (`MW_WATERMARK*`), plus the new `/.well-known/openpgpkey/...` and
  `/api/security/*` endpoints. The private-key crypto is client-side and needs no
  server config. Background + rationale live in [`../security/`](../security/README.md).

V5 (thin desktop & mobile shells, self-contained mode, real screen-capture
protection, self-hostable push) adds:

- [`desktop.md`](./desktop.md) — the Tauri v2 desktop shell: install, **self-contained
  mode** (the shell spawns a bundled `mw-server` sibling so a laptop user needs no
  server), native auth (bearer token + OS keychain), the §7.4 UI-bundle integrity
  gate, and the §16 bundle-size budgets.
- [`push.md`](./push.md) — the **self-hostable push relay** (Web Push/VAPID +
  UnifiedPush, APNs mocked): the privacy model (**no message content transits push**),
  the endpoints, and the server config (`MW_NATIVE_ORIGINS`, `MW_VAPID_CONTACT`,
  `push.quiet_hours`).
- [`mobile-android.md`](./mobile-android.md) — the Android APK build (CI-gated,
  F-Droid-friendly), and the honestly-documented iOS / APNs / app-store-submission
  gaps as ops/sponsorship follow-ups (§28.7).
- Screen-capture protection is now **real** on Windows/macOS/Android and honest
  everywhere else — the matrix is in
  [`../security/screen-capture.md`](../security/screen-capture.md).

V6 (zero-access storage, admin panel, scoped API keys + OAuth 2.1, MCP server,
pluggable Postgres, layered Valkey/Redis cache, observability) adds:

- [`postgres.md`](./postgres.md) — the **pluggable PostgreSQL backend**: backend
  selection by DSN (`MW_DB_PATH=postgres://…`), rustls TLS (no OpenSSL), and the
  `mailwoman migrate-store` SQLite→Postgres copy. SQLite stays the default.
- [`cache.md`](./cache.md) — the layered **Valkey/Redis cache** (`MW_REDIS_URL`): the
  §15.6 scope matrix, the structural zero-access exclusion, Redis-down degradation, and
  the Valkey-vs-Redis licensing note.
- Security background + the operator surface for the admin panel, API-keys/OAuth, MCP,
  and observability live under [`../security/`](../security/README.md)
  ([admin-panel](../security/admin-panel.md) ·
  [api-keys-oauth](../security/api-keys-oauth.md) · [mcp](../security/mcp.md) ·
  [observability](../security/observability.md) ·
  [zero-access](../security/zero-access.md)).

## Configuration (environment)

| Env | Default | Meaning |
|-----|---------|---------|
| `MW_BIND` | `0.0.0.0:8080` | Listen address for the HTTP server. |
| `MW_DB_PATH` | `mailwoman.db` | Store DSN. A bare path or `sqlite://…` selects the SQLite backend (default); a `postgres://…` DSN selects Postgres (V6, see [`postgres.md`](./postgres.md)). |
| `MW_SERVER_KEY` | *(ephemeral)* | Hex-encoded 32-byte key sealing upstream creds. **Set it** in production so sessions survive restarts; keep it secret. Generate: `openssl rand -hex 32`. |
| `MW_RENDER_BIN` | *(auto-detect)* | Path to `mw-render`. The image sets `/usr/local/bin/mw-render`. |
| `MW_WEB_DIR` | *(embedded)* | Serve the SPA from disk instead of the embedded copy (dev override). |
| `MW_COOKIE_SECURE` | `false` | Mark the session cookie `Secure`. **Set `true` behind TLS** (i.e. always in production). |
| `MW_MODE` | `proxy` | `proxy` (JMAP upstream) or `engine` (local IMAP/POP3 + SMTP). |
| `MW_ENGINE_TLS` | *(from URL)* | Engine mode only. Force the IMAP/POP3 transport (`implicit`/`starttls`/`plaintext`) regardless of the `imap(s)://` URL the browser posts — used to point at a plaintext test server (Greenmail) without changing the URL. |
| `MW_SMTP_HOST` | *(IMAP host)* | Engine mode only. SMTP submission host for `EmailSubmission/set`. |
| `MW_SMTP_PORT` | `587`/`465`/`25` | Engine mode only. SMTP port (default keys off `MW_SMTP_SECURITY`). |
| `MW_SMTP_SECURITY` | `starttls` | Engine mode only. `starttls` / `implicit` / `plaintext`. |
| `RUST_LOG` | `info` | Tracing filter. |
| `MW_REDIS_URL` | *(unset)* | V6. Redis/Valkey URL for the layered cache. Unset → memory + store only. See [`cache.md`](./cache.md). |
| `MW_ADMIN_ENABLED` | `true` | V6. `false`/`0` unmounts the `/admin` panel (returns `401`). See [`../security/admin-panel.md`](../security/admin-panel.md). |
| `MW_ADMIN_USER` / `MW_ADMIN_PASSWORD` | *(unset)* | V6. Admin operator credential (separate session domain). Unset → admin login fails. |
| `MW_OTLP_ENDPOINT` | *(unset)* | V6. OTLP collector (e.g. `http://otel:4317`); rustls transport. Unset → OTLP export off. See [`../security/observability.md`](../security/observability.md). |
| `MW_METRICS_TOKEN` | *(unset)* | V6. Bearer token guarding `GET /metrics`. Unset → `/metrics` is unreachable (never open). |
| `MW_LOG` | `info` | V6. Per-subsystem tracing directives; hot-reloaded on `SIGHUP`. |

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

V7 (release 26.8.0) — bridges, directory, Assist, plugins — adds:

- [`ldap.md`](./ldap.md) — the read-only **LDAP/GAL directory** (`mw-directory`):
  endpoint list + priority, attribute mapping, StartTLS/LDAPS, S/MIME cert + photo
  lookup, and LDAP-bind login.
- [`../assist.md`](../assist.md) — **Assist (AI)**: BYO endpoint adapters, capability
  scoping, the content-free audit, the "what left the device" disclosure, and admin
  governance.
- [`../security/plugins.md`](../security/plugins.md) — the **WASM plugin runtime**:
  authoring, Ed25519 signing, the capability model, and resource limits.
- [`../security/password-change.md`](../security/password-change.md) — in-app
  **password change** backends and the zero-access re-wrap.
- [`../bridges/`](../bridges/) — the **Graph / EWS / Gmail** bridges (admin
  app-registration + BYO app-ID + the honest scope boundaries).
- [`../export/msg-oft-docx.md`](../export/msg-oft-docx.md) — **MSG/OFT/DOCX** export.
- [`../integrations/nextcloud.md`](../integrations/nextcloud.md) — **Nextcloud**
  attach/save/share-link.
- [`../RELEASE-NOTES-26.8.md`](../RELEASE-NOTES-26.8.md) — the V7 summary and the three
  honest scope boundaries (bridge PIM-seam, EWS Kerberos, quick-xml write-only ignore).

## Backends

Mailwoman is backend-agnostic in **both** modes:

- **proxy mode** — the JMAP server URL is entered at login and the server
  proxies to it. For local development and E2E, `docker-compose.dev.yml`
  provides the in-repo `mw-mock-jmap` (default, deterministic) and an optional,
  profile-gated Stalwart service (`--profile stalwart`, experimental — see the
  compose file and `scripts/stalwart-seed.sh`).
- **engine mode** — the same login form's server-URL field takes an
  `imap(s)://` / `pop3(s)://` URL; the engine drives that account (IMAP/POP3
  sync + SMTP submission) and answers JMAP locally. See
  [`imap-pop3.md`](./imap-pop3.md). Testing backends: **Greenmail** (the
  deterministic conformance gate) and **Dovecot** (the production-fidelity
  target), both in `docker-compose.dev.yml`.
