# Web hardening flags (V2)

V2 adds hardening deltas to `mw-server` (SPEC §7.4). All are **additive** and
default to safe values that keep V1 clients working; the notes below say which
to flip once the whole stack is on V2.

## Headers (always on)

- **CSP** — base policy unchanged; V2 adds `worker-src 'self' blob:` for the
  self-hosted pdfjs worker. `/api/sanitize` also returns a locked-down
  **per-message CSP** (`{html, csp}`) the web applies to the message iframe;
  existing consumers that read only `html` are unaffected.
- **COEP** `require-corp` — on by default; toggle with `--coep` / `MW_COEP`.
- **CORP** `same-origin` and a restrictive **Permissions-Policy**
  (camera/mic/geo/usb/… denied) — always on.

## Origin / CSRF

- **Origin/Referer same-site check** on all state-changing methods — **always
  on**, needs no client change. Browsers send `Origin` on cross-site writes; a
  cross-origin write is rejected with `403`. Native clients that send neither are
  allowed.
- **Double-submit CSRF** (`mw_csrf` cookie ↔ `X-CSRF-Token` header) — on with
  `--csrf-strict` / `MW_CSRF_STRICT`. The server's own default is off;
  `deploy.sh` writes `MW_CSRF_STRICT=true` into every new installation. When it
  is on, a POST, PUT, PATCH or DELETE that a cookie authorises is answered
  `403 {"error":"missing or invalid CSRF token"}` unless it repeats the value of
  the `mw_csrf` cookie in `X-CSRF-Token`.
  - **Where the token comes from.** Every route that completes a sign-in sets a
    new `mw_csrf` cookie: `/api/login`, the second-factor step `/api/login/2fa`,
    the SSO callback and `/admin/login`. So does `/api/session/rotate`.
    `GET /api/me` and `GET /admin/session` set one for a signed-in client that
    has none and leave an existing one alone. The JSON answers carry the value
    as `csrfToken` too.
  - **Who sends it.** The web client, on every write, from 26.20 on. Any other
    client that signs in with a cookie (a script with a cookie jar, for example)
    has to read the cookie and send the header itself.
  - **Routes not asked for it.** The sign-in routes (`/api/login`,
    `/api/login/2fa…`, `/api/discover`, `/admin/login`), which run before any
    token exists, and the routes no cookie authorises (`/oauth/token`,
    `/oauth/introspect`, `/oauth/revoke`, `/oauth/register…`,
    `/api/webhooks/inbound`, `/errors`, `/mcp`). All of them are still
    Origin-checked. The SSO callback and ACS landings skip both checks.
  - **Bearer requests.** A request with an `Authorization: Bearer` header and no
    session cookie (`mw_session`, `mw_admin_session`) skips the Origin check and
    the token check: the bearer credential is its only authority. That is how the
    native shells, API keys and OAuth clients call the server. With a session
    cookie beside the bearer header both checks apply (from 26.20 on; before, any
    bearer header switched them off).
  - **What it does not cover.** The token is a random value compared with a
    cookie; it is not tied to the session, so it adds nothing against someone who
    can already set cookies for the site's domain.

## Sessions

- **Idle timeout** — `--session-idle-secs` (default `1800`).
- **Absolute timeout** — `--session-absolute-secs` (default `43200`).
- **Rotation** — a fresh session id is issued on every login, plus
  `POST /api/session/rotate` (new id + csrf, old id invalidated, absolute clock
  preserved). Expired sessions are deleted and answered `401`.

  > Timing is in-process in V2. Sessions survive for the process lifetime; after
  > a restart a session is re-seeded leniently on next use.

## TLS

See [`acme.md`](./acme.md) for `--acme` / external-cert `--tls-cert`/`--tls-key`
with `SIGHUP` hot-reload. Set `MW_COOKIE_SECURE=true` whenever the browser
reaches Mailwoman over HTTPS.

## Recommended production posture

| Flag | Default | Production |
|------|---------|-----------|
| `MW_COOKIE_SECURE` | `false` | `true` (behind TLS) |
| `--coep` / `MW_COEP` | on | on |
| `--csrf-strict` / `MW_CSRF_STRICT` | off | on (`deploy.sh` sets it for a new installation; the web client sends `X-CSRF-Token`) |
| `--session-idle-secs` | `1800` | tune to policy |
| `--session-absolute-secs` | `43200` | tune to policy |
