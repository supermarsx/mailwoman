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
- **Double-submit CSRF** (`mw_csrf` cookie ↔ `X-CSRF-Token` header) — **opt-in**
  via `--csrf-strict` / `MW_CSRF_STRICT`. Login and rotate/`/api/me` issue the
  token (cookie + `csrfToken` in the JSON). `/api/login` and `/api/discover` are
  exempt (no prior token; covered by Origin + `SameSite`).

  > Enable `--csrf-strict` only once the web app echoes `X-CSRF-Token` on writes.
  > Until then, Origin + `SameSite=Strict` are the CSRF defense.

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
| `--csrf-strict` / `MW_CSRF_STRICT` | off | on (once the web sends `X-CSRF-Token`) |
| `--session-idle-secs` | `1800` | tune to policy |
| `--session-absolute-secs` | `43200` | tune to policy |
