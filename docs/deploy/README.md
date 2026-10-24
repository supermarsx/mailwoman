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
- [`egress-proxy.md`](./egress-proxy.md) — routing outbound fetches through your
  own HTTP `CONNECT` or SOCKS5 proxy: configuring a route, the **write-only**
  credential (leave the field blank to keep the stored password), the live route
  test and what its outcome/stage mean, and why tunnel failure is **fail-closed**
  rather than falling back to a direct connection.
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
| `MW_UPSTREAM_DEFAULT` | *(unset)* | 26.20. The mail server every sign-in uses. Proxy mode: the JMAP server, e.g. `https://jmap.example.org` (the session resource is fetched from `/.well-known/jmap` unless the URL names it). Engine mode: `imaps://imap.example.org:993`, `imap://host:143` (STARTTLS), `pop3s://host:995` or `pop3://host:110`. **Set it at install:** the sign-in form asks only for the email address and the password, so with neither this nor a one-origin `MW_JMAP_UPSTREAMS` every sign-in answers "This address is not set up on this server". Its origin is treated as operator-named, like an `MW_JMAP_UPSTREAMS` entry. See [Upgrading: the server chooses the mail server](#upgrading-the-server-chooses-the-mail-server). |
| `MW_JMAP_UPSTREAMS` | *(unset)* | 26.20. Proxy mode only. Comma-separated JMAP upstream **origins** the server may connect to, e.g. `https://jmap.example.org,http://stalwart:8080`. **Unset:** any upstream on a public address; loopback, private (RFC 1918), link-local and cloud-metadata, CGNAT and unique-local targets are refused. **Set:** only the listed origins (scheme, host and port must all match), at whatever address they resolve to. This is how you use an internal JMAP server. An entry that is not an `http(s)` origin never matches. With `MW_UPSTREAM_DEFAULT` unset and **exactly one** usable origin listed, that origin is the upstream every sign-in uses. See [Upgrading: the proxy upstream is now checked](#upgrading-the-proxy-upstream-is-now-checked). |
| `MW_ENGINE_TLS` | *(from URL)* | Engine mode only. Force the IMAP/POP3 transport (`implicit`/`starttls`/`plaintext`) regardless of the scheme in `MW_UPSTREAM_DEFAULT` — used to point at a plaintext test server (Greenmail) without changing the URL. |
| `MW_SMTP_HOST` | *(IMAP host)* | Engine mode only. SMTP submission host for `EmailSubmission/set`. |
| `MW_SMTP_PORT` | `587`/`465`/`25` | Engine mode only. SMTP port (default keys off `MW_SMTP_SECURITY`). |
| `MW_SMTP_SECURITY` | `starttls` | Engine mode only. `starttls` / `implicit` / `plaintext`. |
| `RUST_LOG` | `info` | Tracing filter. |
| `MW_REDIS_URL` | *(unset)* | V6. Redis/Valkey URL for the layered cache. Unset → memory + store only. See [`cache.md`](./cache.md). |
| `MW_ADMIN_ENABLED` | `true` | V6. `false`/`0` unmounts the `/admin` panel (returns `401`). See [`../security/admin-panel.md`](../security/admin-panel.md). |
| `MW_ADMIN_USER` / `MW_ADMIN_PASSWORD` | *(unset)* | V6. Admin operator credential (separate session domain). Unset → admin login fails. |
| `MW_ADMIN_MIN_TLS` | *(unset)* | 26.20. `1.2` or `1.3`. Pins the security policy's minimum TLS version (HTTPS listener and outbound mail connections); the panel cannot change it while set. See [`../security/admin-panel.md`](../security/admin-panel.md#security-policy). |
| `MW_ADMIN_REQUIRE_2FA` | *(unset)* | 26.20. `true` or `false`. Pins the global require-two-factor rule. |
| `MW_ADMIN_SESSION_COOKIE` | `mw_admin_session` | 26.20. Name of the admin session cookie. |
| `MW_ADMIN_PATH` | `/admin` | 26.20. URL path of the admin console, its sign-in and its API, under `MW_BASE_PATH` when that is set, e.g. `/ops-7f3k`. `/` followed by one to four segments of `A-Z a-z 0-9 . _ ~ -`, no trailing `/`; a path under `/api`, `/jmap`, `/oauth`, `/.well-known`, `/dav`, `/mcp`, `/assets`, `/fonts` or another path the server already uses is refused, and so is one that starts with the same segment as `MW_BASE_PATH`. When set, `/admin` answers exactly like any unknown path. A custom path cuts down scanning and drive-by sign-in attempts; it does not replace the admin password, two-factor sign-in, rate limits or network restrictions. See [Moving the admin console](#moving-the-admin-console). |
| `MW_ADMIN_PORT`, `MW_ADMIN_LOG_LEVEL`, `MW_ADMIN_OTLP_DSN`, `MW_ADMIN_METRICS`, `MW_ADMIN_THEME`, `MW_ADMIN_BRAND`, `MW_ADMIN_ACCENT` | — | 26.20: **setting any of these stops the start**, with an error saying what to use instead (before 26.20 they logged a warning). Nothing reads them. An invalid value of the four variables above also stops the start. |
| `MW_OTLP_ENDPOINT` | *(unset)* | V6. OTLP collector (e.g. `http://otel:4317`); rustls transport. Unset → OTLP export off. See [`../security/observability.md`](../security/observability.md). |
| `MW_METRICS_TOKEN` | *(unset)* | V6. Bearer token guarding `GET /metrics`. Unset → `/metrics` is unreachable (never open). |
| `MW_LOG` | `info` | V6. Per-subsystem tracing directives; hot-reloaded on `SIGHUP`. |
| `MW_REALTIME_RECHECK_SECS` | `30` | 26.20. How often, in whole seconds, an open `/jmap/ws` or `/jmap/eventsource` stream re-checks its session (minimum `1`; a value that is not a whole number means `30`). The stream ends at the first re-check after the session is signed out, revoked or expired, the bearer token is signed out, the account is disabled or held for a password change, or the database cannot be read. Each stream only ever carries its own account's changes. Lower values end streams sooner but cost a few database reads per open stream each period. Read once at start. |
| `MW_TRUSTED_PROXIES` | *(empty)* | 26.19. CIDRs/addresses whose peers may assert a forwarded header. Empty ⇒ no forwarded header is ever trusted. See [`reverse-proxy.md`](./reverse-proxy.md). |
| `MW_FORWARDED_MODE` | `off` | 26.19. Which forwarded header to read: `off` / `xff` / `forwarded`. Inert without `MW_TRUSTED_PROXIES`, and vice versa. An unrecognised value is `off`. |
| `MW_PROXY_PROTOCOL` | `off` | 26.19. PROXY protocol on the HTTP/HTTPS listener: `off` / `accept` / `require`. Use `require` behind an L4 balancer. |
| `MW_PUBLIC_URL` | *(unset)* | 26.19. Canonical external base (`https://mail.example.com`). Highest-precedence source of public scheme + host. **Setting an https base changes the WebAuthn origin and can invalidate existing passkeys** — see [`reverse-proxy.md`](./reverse-proxy.md) §3. |
| `MW_BASE_PATH` | *(unset)* | 26.19. Serve under a path prefix (`/mail`). The app **also** stays mounted at the origin root by design; the prefix is routing, not isolation. |
| `MW_HEADER_AUTH_TRUSTED_IPS` | *(empty)* | Peers allowed to assert `X-Remote-User` when `MW_HEADER_AUTH=1`. Fails closed — empty authenticates nobody. Not independent of `MW_TRUSTED_PROXIES` once `MW_PROXY_PROTOCOL != off`. |
| `MW_ASSIST_RATE_LIMIT_PER_MIN` | *(unset)* | 26.19. Per-account AI assistant budget in **outbound endpoint requests**/minute. `0` = hard stop. A cold-cache semantic search costs up to 33; a chat turn costs 1. |

### Upgrading: the server chooses the mail server

Applies to both modes, from 26.20.

The sign-in form asks for the email address and the password, and nothing
else. It has no server URL field, no "enter server details manually" link, and
it no longer shows a server it found for the address. The server decides which
mail server an address signs in to, from its own configuration:

1. `MW_UPSTREAM_DEFAULT` (see the table above);
2. in proxy mode, if that is unset, the one origin in `MW_JMAP_UPSTREAMS` when
   it lists exactly one;
3. otherwise nothing: the sign-in answers `404` with code
   `addressNotConfigured`, and the form says "This address is not set up on
   this server. Contact your administrator."

Per-domain settings in the admin panel are planned; until then every address
signs in to the same server.

**What to do on upgrade.** Set `MW_UPSTREAM_DEFAULT` to the server your users
used to type, e.g. `MW_UPSTREAM_DEFAULT=https://jmap.example.org` (proxy) or
`MW_UPSTREAM_DEFAULT=imaps://imap.example.org:993` (engine). A proxy install
that already lists exactly one origin in `MW_JMAP_UPSTREAMS` keeps working
without it.

**Also changed:**

* `POST /api/login` refuses a body that names a server (`jmapUrl`, `host`,
  `port`, `server`, `serverUrl`, `imapUrl`, `pop3Url`, `sessionUrl`,
  `upstream` or `url`, even empty) with `400` and code
  `clientServerNotAllowed`. A script that signs in must send only `username`
  and `password` (plus `clientType` for a native client).
* `POST /api/discover` answers `410 Gone`. It used to look the address's domain
  up and return the servers it found to the browser.
* With header authentication (`MW_HEADER_AUTH`) in proxy mode, the upstream is
  `MW_HEADER_AUTH_JMAP_URL`, else the order above; a `jmapUrl` in the login
  body no longer overrides it.
* Desktop and mobile apps built before this release still send the server they
  found to `/api/login` and are refused; rebuild them from this release.

### Moving the admin console

From 26.20. Nothing changes on upgrade: unless `MW_ADMIN_PATH` is set, the
console stays at `/admin` (under `MW_BASE_PATH` when that is set).

To move it, set the path and restart:

```
MW_ADMIN_PATH=/ops-7f3k
```

The console page, the admin sign-in and every admin API route are then served at
`{MW_BASE_PATH}/ops-7f3k` instead of `{MW_BASE_PATH}/admin`, and `/admin` gets
the same `404` as any path the server does not know: no redirect, no hint. The
path is not written into the web app, the service worker, the mail client's page
or any public answer, so nothing a visitor outside the console can fetch reveals
it. An invalid value stops the start with an error naming the variable.

**What it does and does not do.** A path that is not `/admin` cuts down
automated scanning and drive-by sign-in attempts against the console. It is an
extra layer, not a secret that protects the console on its own: the path appears
in your reverse proxy's logs and in the browser history of anyone who uses it.
Keep a strong admin password, two-factor sign-in for admin accounts, the sign-in
rate limit and, where you can, a network restriction on the path (an allowlist
in the reverse proxy, a VPN).

**Also update:** bookmarks, any reverse-proxy rule that names `/admin`, and
monitoring that probes it. The admin session cookie is unchanged (`Path=/`), so
an admin who was signed in stays signed in at the new path. A browser end-to-end
run against a moved console takes `MW_E2E_ADMIN_PATH` (default `/admin`).

### Upgrading: the proxy upstream is now checked

Applies to proxy mode (`MW_MODE=proxy`, the default), from 26.20.

Before 26.20 the login form's server URL came from whoever submitted it, and the
server fetched it unchecked. (From 26.20 the server chooses the upstream itself;
see the section above.) It then relayed `/jmap/api`, `/jmap/download` and
`/jmap/upload` to whatever URLs that server's session document named. Anyone
running a JMAP server of their own could log in against it and read back
responses from addresses only your server can reach, such as a cloud metadata
endpoint or an internal admin API.

**What stops working on upgrade.** A deployment whose JMAP server has a
**private or loopback address**: another container on a Docker network
(`http://stalwart:8080`), a Kubernetes service, `localhost`, or a LAN address.
Logins against it fail with the usual "invalid credentials", and the server logs
`proxy login refused: refused by the egress policy (Blocked)`. **Fix:** list the
upstream in `MW_JMAP_UPSTREAMS`, e.g. `MW_JMAP_UPSTREAMS=http://stalwart:8080`,
or set it as `MW_UPSTREAM_DEFAULT`, with the same scheme, host and port. A
deployment whose JMAP server is on a **public address**
(`https://jmap.example.org`) needs no change to this list.

**Also enforced, whether or not the variable is set:**

* The session document's `apiUrl`, `downloadUrl` and `uploadUrl` must be on the
  same origin as the upstream URL. An upstream that serves its API from a different
  host or port is refused at login, and the server logs `the upstream session's
  URLs: not on the upstream's origin`.
* The initial session request follows a redirect only to the same origin. For a
  provider whose `/.well-known/jmap` redirects to another host, set
  `MW_UPSTREAM_DEFAULT` to the redirect's target URL.
* Redirects on the API, download and upload requests are not followed.
* Every request is pinned to the address checked for it, with no ambient
  `HTTP_PROXY`. Each request is limited to 300 seconds; before 26.20 there was no
  limit.
* Downloads are served as `Content-Disposition: attachment` with a type derived
  from the upstream's but never an HTML, XML or script type. An upload response is
  labelled `application/json`.

**What you gain.** With the variable unset, an anonymous caller can no longer use
your server to reach internal addresses. Setting it narrows logins to the JMAP
servers you name, which is the right setting for any single-provider deployment,
public or not.

## Docker

For a complete single-host deployment with configuration, persistent data,
optional automatic HTTPS, health checks, backups and recovery, use the root
[`deploy.sh`](../../deploy.sh). See the [script guide](script.md).

```sh
bash deploy.sh --domain mail.example.org --mode engine
```

```sh
docker build --target runtime -t mailwoman:local .
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

> ⚠️ **Changed in 26.19 — read [`reverse-proxy.md`](./reverse-proxy.md) before
> upgrading.** Mailwoman no longer trusts `X-Forwarded-For` from whoever
> connects. Until you set **both** `MW_TRUSTED_PROXIES` and
> `MW_FORWARDED_MODE`, every client address the app records — audit log, per-key
> IP allowlists, rate-limit buckets, ban list — is your proxy's address, not the
> client's. Nothing fails loudly.

[`reverse-proxy.md`](./reverse-proxy.md) is the operator-facing page: the full
environment reference, the migration note for `MW_PUBLIC_URL` (it can invalidate
existing passkeys), sub-path hosting with `MW_BASE_PATH`, the interaction
between header auth and the proxy trust list, and **which proxies have actually
been tested versus merely shipped**. Per-proxy configuration trees are under
[`proxy/`](./proxy/).

V7 (release 26.8.0) — bridges, directory, AI assistant, plugins — adds:

- [`ldap.md`](./ldap.md) — the read-only **LDAP/GAL directory** (`mw-directory`):
  endpoint list + priority, attribute mapping, StartTLS/LDAPS, S/MIME cert + photo
  lookup, and LDAP-bind login.
- [`../assist.md`](../assist.md) — **AI assistant**: BYO endpoint adapters, capability
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

- **proxy mode** — the server proxies to the JMAP server set in
  `MW_UPSTREAM_DEFAULT`; users sign in with their email address and password. For local development and E2E, `docker-compose.dev.yml`
  provides the in-repo `mw-mock-jmap` (default, deterministic) and an optional,
  profile-gated Stalwart service (`--profile stalwart`, experimental — see the
  compose file and `scripts/stalwart-seed.sh`).
- **engine mode** — `MW_UPSTREAM_DEFAULT` holds an `imap(s)://` /
  `pop3(s)://` URL; the engine drives the account each user signs in to there
  (IMAP/POP3 sync + SMTP submission) and answers JMAP locally. See
  [`imap-pop3.md`](./imap-pop3.md). Testing backends: **Greenmail** (the
  deterministic conformance gate) and **Dovecot** (the production-fidelity
  target), both in `docker-compose.dev.yml`.
