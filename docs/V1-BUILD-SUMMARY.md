# Mailwoman V1 — Build Summary

**Release:** `26.2` (rolling `YY.N` — see [../VERSIONING.md](../VERSIONING.md)) · **Date:** 2026-07-12 · **Repo:** https://github.com/supermarsx/mailwoman

V1 turns Mailwoman from "a JMAP proxy that drives Stalwart" into "an engine that
drives the mail servers the world actually runs" — Dovecot, Gmail-over-IMAP, a
POP3 host — **behind the identical JMAP surface the web UI already speaks.** The
frontend was not modified.

## What's new

- **Real backends** behind a frozen `AccountBackend` seam, interchangeable:
  - `mw-imap` — IMAP4rev2 (RFC 9051) + rev1 fallback on `tokio`/`rustls`/`imap-proto`
    (not async-imap): CAPABILITY/ENABLE/ID, SASL PLAIN/LOGIN/XOAUTH2, LIST SPECIAL-USE +
    LIST-STATUS, SELECT QRESYNC, UID FETCH CHANGEDSINCE, UID SEARCH/STORE/MOVE (+COPY+EXPUNGE
    fallback), APPEND (UIDPLUS), IDLE.
  - `mw-pop3` — RFC 1939 + CAPA + STLS/995 + SASL; UIDL-diff pull; leave-on-server policies.
  - `mw-smtp` — submission MVP: 465/587, SASL PLAIN/LOGIN/XOAUTH2, SIZE/8BITMIME.
  - `mw-mime` — `mail-parser`/`mail-builder` ↔ JMAP `Email`; runs in the render jail; fuzzed.
- **`mw-engine`** — the orchestrator: per-mailbox sync with the cursor ladder
  (QRESYNC → CONDSTORE → UID-window; POP3 UIDL diff), UID↔stable-id mapping,
  engine-side **JWZ threading**, and a **JMAP surface** (`Mailbox/get`,
  `Email/query`, `Email/get`, `Email/set`, `EmailSubmission/set` with RFC 8620
  §3.7 result references) byte-shape-compatible with what the UI sends.
- **Engine mode vs proxy mode** — `MW_MODE=proxy|engine` config switch. Proxy
  (V0) is unchanged and remains the default for JMAP-native upstreams; engine
  mode handles IMAP/POP3 accounts locally. **The web app cannot tell the difference.**
- **`mw-autoconfig`** — discovery ladder (SRV → Thunderbird autoconfig →
  Autodiscover v2 → provider DB → manual) behind `POST /api/discover`.
- **Encrypted message cache** — `mw-store` schema additions (accounts, mailboxes,
  messages, bodies, threads, pop3_uidl, sync_state); bodies/envelopes sealed at
  rest; a `Redacted<T>` logging primitive so wire tracing can't leak PII.

## Frozen seam (why parallel build worked)

One `AccountBackend` trait (`crates/mw-engine/src/backend.rs`) authored by the
scaffolder before any backend existed — IMAP, POP3, and future bridges implement
it; the engine consumes only the trait. Backends never see stable ids; the engine
never leaks raw UIDs upward. `SyncCursor` is persisted as opaque JSON so `mw-store`
never depends on `mw-engine` (no cycle); backend construction lives in `mw-server`.

## Verification (final coordinator gate, all green)

- **162 Rust tests** across 12 crates + **13 web tests** · clippy `-D warnings`
  clean · fmt clean.
- **Fuzz targets** on `mw-imap`/`mw-pop3`/`mw-mime` (SPEC §4.3); `mw-mime` ran
  32,887 execs / 0 crashes.
- **`cargo-deny`** advisories/bans/licenses/sources clean — no GPL/LGPL/AGPL,
  no async-std (`imap-proto` + `mail-parser`/`mail-builder` confirmed permissive).
- **CI conformance**: Greenmail (deterministic required gate) + Dovecot
  (fidelity, continue-on-error) — live IMAP/POP3/SMTP + engine tests run against
  real servers.
- **Playwright E2E** (`--project=engine`, run live against Greenmail): login with
  `imap://…` → real IMAP folders → compose → SMTP send → message arrives →
  open MIME-parsed sanitized body in the sandboxed reader. V0 mock specs still
  green as a regression gate.

## Known V1 limitations (tracked follow-ups)

- **Cross-folder move changes `Email.id`** (delete+reinsert; the store's identity
  match is per-mailbox). Invisible to the poll-based UI; needs a
  `store.relocate_message` helper if id-stable moves are wanted later.
- **SRV autoconfig is a no-op** in V1 (no bundled DNS resolver, to stay off the
  license floor); the Thunderbird/Autodiscover/provider rungs carry discovery.
- **CI actions pin to `@vN` tags**, not SHAs — recommend a workflow-wide SHA-pin
  pass rather than piecemeal.
- **ACME/TLS listener** still deferred (dev/E2E run cleartext inside the compose
  network) — carried from the V0 gap.

## Next (V2, per SPEC §27 + gap analysis)

Search (Tantivy), offline (SW/OPFS) + WebSocket push + multi-window, the modern
mail UX (tags/pins/colors/snooze/sweep/undo-send/outbox/unified+focused inbox,
signatures/identities), Sieve rules, EML/mbox/PDF/MD export, the **design-token
theming system + Grove themes + font puller**, and **embedded attachment viewers**
(images/PDF/video, themed, jail-rendered). An optional Outlook-style **ribbon
preset** is a candidate V2 UX addition (pending a scope decision).
