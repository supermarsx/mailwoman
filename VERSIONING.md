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

- **`26.9`** — enterprise SSO + the accessibility/i18n/perf/packaging hardening pass.
  **Full OIDC and SAML 2.0 single sign-on** as login backends (new `mw-sso` crate),
  configured per-deployment/domain via the admin panel + a `0009` `sso_config` table
  and surfaced as "Sign in with <IdP>" on the login screen: OIDC over the
  `openidconnect` crate (discovery, auth-code + **PKCE**, JWKS ID-token validation,
  userinfo, RP-logout — RustCrypto/rustls, **no openssl**), and a **hand-rolled
  pure-Rust SAML SP** (SP metadata, AuthnRequest, HTTP-POST ACS, exclusive-C14N +
  XML-DSig RSA/ECDSA-SHA256 validation, audience/replay defenses — no `samael`,
  no openssl/libxml) with a content-free login audit and first-login defaulting to
  allowlist/deny. **Both flows are proven end-to-end live against a real Keycloak
  26.0** (headless + real-browser → authenticated inbox). This milestone also folds
  in the 1.0-readiness hardening: a **WCAG 2.2 AA** audit + fixes across every web
  screen (calendar ARIA grid, ribbon tablist, dialog focus, non-color verdict
  badges) gated by axe in CI; **Fluent i18n** with an `en` baseline, a 12-locale
  structure + Weblate config + RTL/bidi plumbing (human translation pending);
  **§23 performance budgets** measured-and-gated in CI (cold-load, render, bundle,
  binary/image); and **packaging recipes** (Flatpak/F-Droid/winget/deb/rpm/AppImage/
  macOS-notarize). Structural size work: the five first-party plugin `.wasm`
  components are **externalized** from the server binary to a plugins dir, each
  **SHA-256 digest-pinned** (fail-closed integrity), and the §23 binary/image budgets
  are revised to measured-realistic values (binary <91MB, image <205MB = measured
  ×1.15, documented) since the full V7 feature set (wasmtime JIT + all protocols +
  crypto) is inherently larger than the original core-build targets. Security posture
  is best-effort self-hardening + a published external-audit-prep dossier (no funded
  audit — open-source). Verified: 934 Rust + 633 web tests; cargo-deny clean with no
  new advisory ignore and **no openssl anywhere**; live SSO E2E green vs real
  Keycloak. Rolling `YY.N` scheme retained (this is 26.9, not a "1.0" tag).
  Remaining ops follow-ups (not release-gating): store/signing account provisioning +
  submissions, and human translation review via Weblate.
- **`26.8`** — V7: extensibility, directory, AI, and Exchange/Gmail bridges (the
  last feature milestone before 1.0). A **WASM engine-plugin runtime** (`mw-plugin`
  over wasmtime + the WASI-p2 component model): capability-deny-by-default, per-
  plugin resource limits (epoch-deadline + memory ceiling + optional fuel → a
  clean `LimitExceeded`, never a host panic), an Ed25519 signed registry, and a
  host-mediated ABI (no ambient network/fs — outbound HTTP and OAuth tokens are
  host-held) — the jail is the security boundary, proven live with a real loaded
  component (out-of-allowlist host denied, resource trip observed). An **LDAP/GAL
  directory** (`mw-directory`, ldap3 over rustls — no openssl): GAL search in
  recipient fields, distribution-group expand-before-send, S/MIME cert + photo
  lookup, multi-directory priority, StartTLS/LDAPS, read-only. **Password-change
  backends** (`mw-passwd`): local/LDAP-3062/Dovecot/poppassd/HMAC-webhook, with
  client-side zero-access key-hierarchy re-wrap and coordinated credential re-seal.
  An **Assist (AI) subsystem** (`mw-assist`): a BYO-endpoint gateway (OpenAI-
  compatible/Anthropic/local-process, hand-rolled over rustls — no LLM SDK) with
  per-capability scoping, data-class ceilings, **E2EE content never forwarded by
  default**, content-free audit, a "what left the device" disclosure, and — by
  construction — no capability that sends/accepts/deletes (send stays human-gated;
  the assistant reuses the MCP tool surface). **Graph, EWS, and Gmail bridges** as
  first-party `wasm32-wasip2` plugins implementing the frozen `AccountBackend`
  trait — indistinguishable from IMAP to the engine, quirks isolated to the bridge,
  OAuth tokens never in the guest, EWS using **hand-rolled pure-Rust NTLMv2** (zero
  new deps); they boot-load from the registry and are full **read + send** accounts.
  Plus **MSG/OFT/DOCX export** (`mw-export` via cfb/docx-rs), a **Nextcloud** attach/
  share-link plugin, GAL/Assist/Nextcloud wired into the mailbox compose+read UX,
  and both V6 follow-ups closed (proxy-mode headless scoped-key REST reads; the real
  MCP unattended-send countersign resolver). New crates: mw-plugin, mw-directory,
  mw-passwd, mw-assist; new `plugins/` (bridge-graph/ews/gmail, languagetool,
  nextcloud). Verified: 846 Rust + 579 web tests; cargo-deny clean; a live E2E gate
  (12/12) against **real OpenLDAP + a real jailed plugin + a mock Assist endpoint**
  — plugin-backed account serves JMAP identically to IMAP via the boot path, bridge
  send routes to the provider exactly once, Assist redaction proven — which caught
  three real deployment gaps (bridge mail-sync cursor, LDAP-3062 result-code
  handling, and boot-time plugin loading) that were fixed before release.
  **Honest scope boundaries** (not overclaimed): bridges deliver **mail** through
  the jail — bridge calendar/tasks/reactions are implemented and fixture-tested but
  reachable only through a **post-1.0 WIT-export extension**; EWS **Kerberos** is a
  documented BYO-reverse-proxy gap (Basic + NTLMv2 ship); third-party (non-bundled)
  plugin byte-storage is post-1.0; and a bounded `quick-xml`-reader-DoS advisory
  ignore is scoped to write-only DOCX export. **V7 completion is not 1.0** — the
  distinct 1.0 hardening gate (WCAG 2.2 AA, translations, perf budgets, and a funded
  external audit incl. the MCP/plugin/Assist surfaces) is enumerated in
  `docs/ROADMAP-1.0.md`.
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
