# Versioning

Mailwoman uses a **rolling release** scheme in **`YY.N`** format.

- **`YY`** — two-digit calendar year of the release (e.g. `26` for 2026).
- **`N`** — the release number within that year, starting at **1** and
  **resetting to 1 each new calendar year**.

Examples, in order: `26.1`, `26.2`, `26.3`, … then `27.1`, `27.2`, …

There is no separate major/minor/patch. Each tagged release is a self-contained
rolling snapshot; the sequence is strictly increasing within a year, and the
year boundary resets `N`.

## Git tags

Releases are tagged **bare** (no `v` prefix): `26.1`, `26.2`, …
Tags are annotated and signed where possible.

## Package manifests (semver-shaped ecosystems)

Cargo and npm require semver (`X.Y.Z`). We map `YY.N` → **`YY.N.0`**:

- `Cargo.toml` `[workspace.package] version` → `26.1.0`
- `apps/web/package.json` `version` → `26.1.0`

The third component (`.0`) is reserved for the rare out-of-band hotfix to an
already-tagged release (`26.1.1`); normal forward progress increments `N`
(`26.2`), not the patch field.

## Release checklist

1. Bump `version` in `Cargo.toml` and `apps/web/package.json` to `YY.N.0`.
2. Update this file's example if the year rolled over.
3. Commit `chore(release): YY.N`.
4. Tag: `git tag -a YY.N -m "Mailwoman YY.N"` then `git push origin YY.N`.

## History

- **`26.7`** — V6: server depth — zero-access storage, admin, API/OAuth, MCP,
  Postgres, cache. An **optional zero-access (zero-knowledge) storage mode**:
  the client-side key hierarchy (Argon2id/WebAuthn-PRF → root key → KEK →
  per-account data keys) is built on the existing V4 `mw-crypto` WASM, rows are
  sealed with XChaCha20-Poly1305 (AAD = table‖row‖schema-version), and a
  device-pairing QR+SAS flow transfers the root key device-to-device with the
  server relaying only ciphertext. Its scope is stated honestly: the server at
  rest sees ciphertext, opaque IDs, sizes, and timestamps, and because it still
  proxies live IMAP/SMTP a malicious *active* server is a stronger adversary
  that this mode does **not** defend against — it protects data at rest, and
  search stays a client-built encrypted index. A **pluggable PostgreSQL
  backend** now sits behind `mw-store` alongside SQLite (backend chosen by DSN;
  `mailwoman migrate-store` copies SQLite→Postgres), a **layered cache**
  (`mw-cache`: moka→Valkey/Redis→store) with a per-class scope matrix that
  structurally excludes zero-access plaintext from Redis/memory, a **full admin
  panel** (domains/users/quotas/policy/integrations/observability + an
  append-only audit log, mirrored to a `mailwoman admin` CLI), **scoped API keys
  + an OAuth 2.1 AS** (mandatory PKCE + RFC 8707 resource indicators; keys
  Argon2id-hashed, shown once, with per-key scope/expiry/IP-allowlist/rate-limit
  enforced on `/api/v1`), an **MCP server** (`/mcp` + `mailwoman mcp-stdio`; ten
  scoped tools, mail content carrying untrusted-provenance labels, and send
  disabled by default — routed to the Outbox unless an admin-countersigned
  `unattended-send` key is used), plus HMAC-signed webhooks, a REST convenience
  layer, and OTLP/Prometheus observability (rustls throughout — no openssl). New
  crates: mw-cache, mw-admin, mw-oauth, mw-mcp (Postgres lands inside mw-store).
  SQLite single-user and the browser cookie path are unchanged. Verified: 624
  Rust + 529 web tests; cargo-deny clean with zero new advisory ignores; and a
  live E2E gate driving the real stack (`postgres:16` + `valkey:8` + a spawned
  server) 7/7 green — admin provisioning+audit, OAuth consent→scoped-key→REST
  enforcement matrix, MCP gated-send→Outbox, backend parity (SQLite==Postgres),
  and zero-access ciphertext-at-rest proven by a direct Postgres query. One
  Postgres-only backend bug (i64 bound into a BOOLEAN column) was caught by that
  live gate and fixed before release.
- **`26.6`** — V5: thin native shells. Tauri v2 desktop (Windows/macOS/Linux)
  and mobile (Android/iOS) shells that reuse the **same SPA bundle** as the web
  app behind a feature-detected `Platform` capability layer (`isTauri()` →
  native path, browser path unchanged). Native auth via bearer token (keychain-
  backed: DPAPI on Windows, Keychain on macOS, Keystore on Android); a
  self-contained mode that spawns the bundled mw-server on loopback; bundle-
  integrity gate on launch; native screen-capture protection
  (`WDA_EXCLUDEFROMCAPTURE` / `FLAG_SECURE`). Background delivery: a server
  WebPush/VAPID relay over **`web-push-native`** (pure-Rust RFC 8188/http-ece,
  no openssl C), UnifiedPush on Android, and a Service-Worker `mw-push-wake`
  consumer that resyncs a backgrounded tab. Verified: 496 Rust + 475 web tests;
  cargo-deny clean (Tauri tree vetted — permissive-only, unmaintained-only
  advisory ignores documented); desktop shell launched live on Windows
  (integrity gate, keychain, self-contained spawn, capture protection); Android
  CI-gated; iOS/APNs documented. Live-E2E gaps caught + fixed: CSP
  `wasm-unsafe-eval` for the crypto worker, `CryptoKey.id` serde default,
  calendar list/instances shape parity, `web-push`→`web-push-native` openssl
  swap, mobile command registration, and the dead `mw-push-wake` consumer.
- **`26.5`** — V4: crypto & security depth. OpenPGP + S/MIME end-to-end
  encryption with **private-key operations in a client-side WASM build** of
  mw-crypto (keys never reach the server unencrypted); decrypted mail is
  sanitized in-worker (mw-sanitize wasm) before the sandboxed iframe. A
  Security panel with DKIM/SPF/DMARC/ARC verdicts, Received-chain, signature
  and attachment-risk analysis, and sender controls that emit **real Sieve
  rules**. Engine-side DLP on the outbound path (PAN/IBAN/national-id
  detectors → warn/block, redacted audit). The three-position max-security
  opening switch. Hybrid X25519+ML-KEM-768 store-key wrapping. Server: WKD
  publishing, ARF abuse reports, an honest watermark overlay. New crate:
  mw-crypto (native + wasm). Verified: 430 Rust + 432 web tests; wasm build on
  Windows + Linux; PGP/S-MIME interop against recorded GnuPG/Thunderbird/
  Outlook fixtures; 8 live Playwright specs (browser-generated key →
  encrypt → send → decrypt → in-worker sanitize; DKIM pass/fail; DLP block;
  max-security). Two "unit-green but CSP/JMAP-dead" gaps caught + fixed at the
  live-E2E gate.
- **`26.4`** — V3: personal-information management. Calendar (all views —
  day/3-day/work-week/week/month/tri-month/schedule/agenda/year — recurrence,
  reminders, attendees, iTIP invites, free/busy, conflict detection),
  tasks (VTODO + My Day + subtasks), encrypted-at-rest notes (rich text,
  tags/colors/pins, cross-links), and contacts (address books, groups, merge,
  vCard/CSV import/export, Compose autocomplete) — synced over CalDAV/CardDAV,
  serialized as iCalendar/vCard, behind a Mailwoman-native PIM surface reusing
  the JMAP envelope. New crates: mw-ics, mw-dav, mw-carddav. Server adds
  calendar/addressbook sharing + a holiday feed. Verified: 367 Rust + 312 web
  tests; Radicale CalDAV/CardDAV conformance (engine<->real-CalDAV round-trip);
  live Playwright E2E across all four modules through the real UI. Four
  end-to-end contract gaps caught + fixed at the E2E gate before release.
- **`26.3`** — V2: modern mail layer + theming. Engine-side Tantivy search
  (operators + saved searches), offline (Service Worker + encrypted OPFS +
  replay queue), WebSocket/SSE realtime push, multi-window (BroadcastChannel),
  the modern mail UX (tags/pins/snooze/sweep/undo-send/outbox/send-later/
  follow-up/focused+unified inbox/virtualized list), Sieve rules, identities,
  EML/mbox/TXT/Markdown export, the vanilla-extract design-token theming system
  (light/dark/HC/AMOLED + Grove woody themes) with self-hosted font puller and
  an optional ribbon preset, and sandboxed embedded attachment viewers
  (image/PDF/video) + a global Attachments module. Server gains a rustls-acme
  TLS listener, per-message CSP + CSRF/session hardening, and a blob-download
  route. New crates: mw-search, mw-sieve, mw-export. Verified: 283 Rust + 214
  web tests; live-stack Playwright E2E across all V2 features (offline, push,
  multi-window, viewers, search operators, theming, export). Six real
  end-to-end gaps caught and fixed at the E2E gate before release.
- **`26.2`** — V1: real mail backends. IMAP4rev2 + POP3 + SMTP submission +
  MIME parse/build behind a frozen `AccountBackend` seam, driven by
  `mw-engine` which presents the same JMAP surface the web UI already speaks
  (engine mode vs V0 proxy mode, config-switched). Sync ladder
  (QRESYNC/CONDSTORE/UID-window + POP3 UIDL), engine-side JWZ threading,
  autoconfig ladder, encrypted message cache. New crates: mw-imap, mw-pop3,
  mw-smtp, mw-mime, mw-engine, mw-autoconfig. Greenmail/Dovecot CI
  conformance + a Playwright E2E driving a real IMAP account through the
  unmodified web UI.
- **`26.1`** — first rolling release. V0 walking skeleton (SPEC §27): wired
  webmail path (SolidJS client → mw-server JMAP proxy + sanitize worker →
  JMAP upstream), Docker/CI, E2E. Supersedes the pre-adoption `v0.0.0`
  placeholder tag, which was removed.
