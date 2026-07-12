# Mailwoman V0 — Build Summary

**Tag:** `v0.0.0` · **Date:** 2026-07-12 · **Repo:** https://github.com/supermarsx/mailwoman

The first tagged release: a genuinely wired, daily-drivable walking skeleton per
SPEC §27 (V0). Not a mockup — the real data path is browser → `mw-server`
(session auth + JMAP proxy + sanitize worker + embedded SPA) → JMAP upstream,
proven end-to-end by Playwright against a live Docker stack.

## What works

- **Login** against a JMAP server (validated upstream), opaque cookie session,
  upstream credentials sealed at rest (XChaCha20-Poly1305).
- **Mailbox list → message list → read** with the HTML body sanitized in a
  **separate render process** (SPEC §7.5 boundary) and rendered in a sandboxed
  iframe with no `allow-scripts`/`allow-same-origin`.
- **Compose + send** via `Email/set` + `EmailSubmission/set` (JMAP result
  references), appearing in Sent.
- **Deploy**: single `mailwoman` binary serving an embedded SPA; multi-stage
  non-root distroless image (~58 MB); `docker compose` stack (mock + optional
  Stalwart profile).

## Crates

| Crate | Role |
|---|---|
| `mw-jmap` | JMAP (RFC 8620/8621) types + async client; `forbid(unsafe_code)` |
| `mw-sanitize` | ammonia-based email HTML sanitizer (SPEC §7.2) |
| `mw-render` | disposable child-process sanitize worker (stdio JSON frames) |
| `mw-store` | SQLite (sqlx, runtime queries) — sessions + settings, sealed creds |
| `mw-mock-jmap` | in-repo JMAP server for deterministic tests/E2E (resolves result refs) |
| `mw-server` | axum: `/api/login|logout|me`, `/jmap/session|api` proxy, `/api/sanitize`, embedded SPA, `serve`/`healthcheck` CLI |

Web: SolidJS + strict TypeScript (no `any`), ~9.6 KB gzip entry (budget < 250 KB).

## Verification (final coordinator gate, all green)

- **41 Rust tests** (jmap serde round-trips, sanitizer torture corpus, render
  round-trip, store crypto/CRUD, mock send flow + result-reference resolution,
  server integration vs mock) · clippy `-D warnings` clean · fmt clean.
- **13 web unit tests** (JMAP request builders + Login component).
- **2 Playwright E2E specs** vs the live compose stack, 6/6 stable under
  `--repeat-each=3`:
  - *happy path* — login → sidebar → seeded messages → open → compose → send → appears in Sent.
  - *sanitizer wiring* — hostile message: `window.__mw_pwned` never set, iframe
    sandbox lacks script/same-origin, script/tracker-pixel/`javascript:` absent
    from rendered DOM, legitimate text preserved.
- **`cargo-deny`**: licenses / advisories / bans clean — MIT/Apache/BSD/ISC/
  Zlib/MPL-2.0 only, zero GPL/LGPL.

## CI (`.github/workflows/ci.yml`)

Jobs: `rust` (fmt/clippy/test/build) · `deny` · `web` (typecheck/lint/test/build)
· `js-licenses` · `e2e` (compose mock stack → wait-healthy → Playwright chromium).

## Notable decisions

- **JMAP as the internal bus** even in V0; IMAP/POP3 arrive in V1.
- **Process-isolated sanitizer from day one** rather than retrofitted later.
- **Mock JMAP server as the default E2E backend** (deterministic); Stalwart
  behind a compose profile for manual/nightly runs.
- **sqlx runtime queries + bundled SQLite** so builds need no database on
  Windows or CI.

## Known follow-ups (V1+)

- Real Stalwart bootstrap in CI (currently continue-on-error/manual).
- `FROM scratch` + musl runtime image (V0 uses distroless to de-risk).
- IMAP/POP3/MIME crates, Postgres backend, search, offline/SW, PGP/S/MIME, PIM
  modules — per SPEC §27 roadmap.
- One local-machine nit surfaced during the build: a broken global pnpm config
  at `%LOCALAPPDATA%\pnpm\config\config.yaml` (`allowBuilds.esbuild` placeholder)
  — worked around in-project; worth clearing to avoid biting other pnpm projects.
