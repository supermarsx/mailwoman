# Mailwoman — Technical Specification

**Version:** 0.3.0-draft · **Date:** 2026-07-12 · **License:** MIT

> A high-performance, security-hardened webmail client and standalone mail app.
> Spiritual successor to [SnappyMail](https://github.com/the-djmaze/snappymail)'s
> "drop it anywhere, it just works" ethos — rebuilt on Rust + TypeScript, with a
> JMAP-first architecture, verifiable end-to-end encryption, full PIM modules
> (calendar, tasks, notes, contacts), a scoped AI subsystem, and thin desktop
> and mobile clients around a **web-first** core. Goal: close the gap with
> state-of-the-art clients (Outlook, Gmail, Fastmail, Proton) while remaining
> fully open source and self-hostable.

---

## Table of Contents

1. [Vision & Goals](#1-vision--goals)
2. [Non-Goals](#2-non-goals)
3. [Licensing & Governance](#3-licensing--governance)
4. [Architecture Overview](#4-architecture-overview)
5. [Technology Stack](#5-technology-stack)
6. [Protocol Support](#6-protocol-support)
7. [Security Architecture](#7-security-architecture)
8. [End-to-End Encryption](#8-end-to-end-encryption)
9. [Zero-Access Storage Mode](#9-zero-access-storage-mode)
10. [Mail Features](#10-mail-features)
11. [Calendar](#11-calendar)
12. [Tasks & Notes](#12-tasks--notes)
13. [Contacts & Directory](#13-contacts--directory)
14. [AI Subsystem — Assist](#14-ai-subsystem--assist)
15. [Realtime, Sync, Offline & Caching](#15-realtime-sync-offline--caching)
16. [Clients: Web-First, Thin Desktop & Mobile](#16-clients-web-first-thin-desktop--mobile)
17. [Theming, Fonts & Personalization](#17-theming-fonts--personalization)
18. [Server, Hosting & Ecosystem Integration](#18-server-hosting--ecosystem-integration)
19. [Admin Panel](#19-admin-panel)
20. [API, Webhooks & MCP](#20-api-webhooks--mcp)
21. [Observability & Logging](#21-observability--logging)
22. [Plugin System](#22-plugin-system)
23. [Performance Targets](#23-performance-targets)
24. [Accessibility & Internationalization](#24-accessibility--internationalization)
25. [Testing & Quality](#25-testing--quality)
26. [Repository Layout](#26-repository-layout)
27. [Roadmap](#27-roadmap)
28. [Open Questions](#28-open-questions)

---

## 1. Vision & Goals

Mailwoman is a **mail client and personal-information manager**, not a mail
server. Like SnappyMail, it connects to the servers people already run
(Dovecot, Postfix, Stalwart, Exchange, Gmail, Microsoft 365) — but unlike
SnappyMail it is:

- **Memory-safe end to end.** All hostile-input parsing (MIME, HTML, IMAP/JMAP
  wire data, crypto) happens in Rust, in sandboxed workers (§7.5).
- **Web-first, with thin native shells.** The web client is the product; the
  desktop and mobile apps are thin clients onto the same server, sharing 100%
  of the UI and behavior (§16).
- **JMAP-native internally.** The UI speaks JMAP to the engine regardless of
  what the upstream server speaks — IMAP, POP3, EWS, Graph, or JMAP itself.
- **Encrypted by default, verifiable by design.** OpenPGP and S/MIME as
  first-class citizens, hybrid post-quantum key exchange where standards allow,
  and an optional zero-access mode where the hosting server never sees plaintext.
- **Feature-competitive with Outlook.** Not just mail: full calendar (with
  sharing, conflict resolution, every view), tasks, encrypted notes, contacts
  with GAL/LDAP, focused inbox, sweep, snooze, follow-ups, message recall,
  voting buttons, reactions, pinning — the whole Outlook feature surface,
  self-hosted.
- **AI-capable without AI-dependence.** A deeply integrated but strictly
  opt-in, permission-scoped Assist subsystem (§14) against endpoints the user
  or admin configures. Zero AI without explicit configuration.

### Target audiences (in priority order)

1. **Self-hosters & homelabs** — single binary next to Dovecot/Stalwart/mailcow,
   1–50 users. Drives: trivial deployment, UnifiedPush, F-Droid-friendly builds,
   recipe docs for the popular mail stacks.
2. **Enterprises** — drives: OIDC/SAML SSO, early S/MIME, full on-prem
   Exchange (EWS) and cloud (Graph) support at 1.0 (§6.5), LDAP/GAL, DLP,
   MDM-deployable apps, audit log exports, retention-aware UI.
3. **Privacy-focused individuals** — drives: polished thin-client apps,
   zero-access mode, PGP UX that normal humans survive, tracker blocking,
   app-store presence.

**Explicitly not a 1.0 audience:** ISP-scale hosting providers
(millions of mailboxes, billing hooks, deep panel integration). The
architecture shouldn't preclude it, but nothing is optimized or tested for
that scale before 1.0 (see §2).

### Design principles

1. **Hostile input is the norm.** Every byte from the network or from an email
   is treated as an attack until parsed, validated, and sandboxed.
2. **The server is not trusted** (in zero-access mode) or is **minimally
   trusted** (in standard mode).
3. **No telemetry, ever.** Not opt-out, not anonymized. None. (Admin-configured
   self-hosted error reporting, §21, is the operator watching their own
   deployment — not us watching them.)
4. **Boring deployment.** One static binary (or one container). PostgreSQL for
   real deployments, embedded SQLite for tiny ones, optional Redis — all in
   one TOML file or env vars.
5. **Standards over proprietary.** JMAP, IMAP4rev2, POP3, CalDAV/CardDAV,
   OpenPGP, Sieve, WebAuthn. Proprietary bridges (Graph, EWS, Gmail API) are
   supported at 1.0 but live in first-party *plugins* (§6.5) so core stays
   standards-pure.
6. **Everything configurable, nothing mandatory.** Features from AI to caching
   to drag-and-drop are individually scopeable per deployment, per user, or
   both. The admin's policy always wins over the user's preference.

---

## 2. Non-Goals

- **Not an MTA/MDA.** Mailwoman never accepts port-25 traffic or stores the
  canonical mailbox. (A future optional "Mailwoman Server" suite is out of scope
  for 1.0; see §28.)
- **Not a groupware server.** Calendar/tasks/notes/contacts are *client* views
  over CalDAV/CardDAV/JMAP/bridges — Mailwoman syncs them, it does not host
  the canonical copy (except Notes, which may be Mailwoman-native, §12.2).
- **No AI in core, no default-on AI anywhere.** Smart features ship only in
  the opt-in Assist subsystem (§14) against a user/admin-configured endpoint;
  nothing in core phones home.
- **Not ISP-scale multi-tenancy (at 1.0).** No billing hooks, no
  million-mailbox horizontal-scaling work, no deep hosting-panel automation
  beyond documented recipes. Revisit post-1.0.
- **No Electron.** Binary size, memory, and attack surface budgets forbid it.
- **No dishonest security theater.** Features that cannot actually be
  guaranteed on a platform (e.g., print-screen blocking in a browser, message
  recall over plain SMTP) are specced with their real limits stated in the UI,
  not just the docs (§7.6, §10.8).

---

## 3. Licensing & Governance

- **License:** MIT for all first-party code. This is a hard constraint on
  dependencies:
  - Allowed: MIT, Apache-2.0, BSD-2/3, ISC, Zlib, MPL-2.0 (file-level copyleft
    is acceptable for unmodified use).
  - **Disallowed in the dependency tree:** GPL, LGPL (this rules out
    `sequoia-openpgp`; we use `rPGP` instead — see §5), AGPL, SSPL, BUSL.
  - CI enforces this with `cargo-deny` (`licenses` check) and a JS equivalent
    (`license-checker-rseidelsohn`) on every PR.
- **CLA:** none. DCO (`Signed-off-by`) only.
- **Development in the open:** public repository from the first commit. Every
  pre-release is tagged and signed; there is no private "polish" phase — a
  security product earns trust through visible history.
- **Sustainability:** donations and sponsorship only (GitHub Sponsors /
  OpenCollective). No SaaS arm, no open-core split, no dual licensing, no paid
  tiers — everyone gets the same MIT code. Sponsorship funds, in priority
  order: third-party security audits, the Apple Developer account, and (if it
  ever stretches that far) Google CASA assessment for a shared Gmail OAuth
  client (§28.6). Budget items the project can't fund simply have a
  bring-your-own path instead.
- **Security policy:** `SECURITY.md` with a 90-day coordinated disclosure
  window, a security contact, and signed release artifacts (Sigstore/cosign +
  SLSA provenance).

---

## 4. Architecture Overview

### 4.1 Web-first, server-centric

The Mailwoman **server** is the product's brain everywhere. Every client —
browser, desktop shell, mobile shell — is the same TypeScript UI speaking
**JMAP over HTTPS + WebSocket** to it. The engine either:

- **proxies** to an upstream JMAP server (thin, fast path), or
- **translates** JMAP ⇄ IMAP4rev2 / POP3 / SMTP / Sieve for legacy servers, or
- **bridges** to Graph / EWS / Gmail API via first-party plugins (§6.5).

```
                       ┌──────────────────────────────────────────┐
  Browser (PWA) ───┐   │  mailwoman server (Rust)                 │
                   │   │  ┌────────────────────────────────────┐  │
  Desktop shell ───┼──►│  │ engine: JMAP surface, sync, rules, │  │──► IMAP/JMAP/
  (Tauri, thin)    │   │  │ search, crypto envelope, Assist    │  │    POP3/SMTP/
                   │   │  │ gateway, DAV client, bridges       │  │    Sieve/DAV/
  Mobile shell ────┘   │  └────────────────────────────────────┘  │    Graph/EWS
  (Tauri, thin)        │   PostgreSQL │ SQLite   +  Redis (opt.)  │
                       └──────────────────────────────────────────┘
```

- **Thin clients:** desktop/mobile shells ship the UI bundle (verified by
  hash, §7.4) and add OS integration — notifications, keychain, mailto:,
  FLAG_SECURE, share targets — but hold **no protocol logic**. They connect to
  the user's Mailwoman server exactly like the browser does. One server, N
  devices, one consistent state.
- **Offline is a client capability, not a deployment mode:** all clients
  (including the plain browser) get the same Service-Worker + OPFS encrypted
  offline cache (§15.4).
- **Self-contained mode:** for a laptop-only user with no server, the desktop
  shell *can* spawn a bundled local `mailwoman` server process (localhost,
  Unix-socket/named-pipe, auto-managed). Same binary, same architecture — the
  thin client just happens to point at 127.0.0.1. This is a convenience mode,
  not a second architecture.
- **E2EE stays end-to-end:** private-key operations always run client-side in
  a WASM build of `mw-crypto` (§8), regardless of where the server runs.

### 4.2 Data layer

| Store | Role | Notes |
|---|---|---|
| **PostgreSQL** (≥14) | **Primary backend** for real deployments: accounts, settings, sync state, message cache, contacts DB, calendar/tasks/notes cache, tags, rules, audit log | Via `sqlx` (compile-time-checked queries); migrations embedded; per-user rows encrypted where zero-access applies (§9.3) |
| **SQLite** (bundled) | Zero-config backend for single-user/eval/self-contained mode | Same schema via `sqlx`; a `mailwoman migrate-store` command moves SQLite → Postgres |
| **Redis / Valkey** (optional) | Accelerator only — never the source of truth: sessions, hot header windows, search hot-set, presence/push fan-out, rate-limit counters | Fully scope-configurable (§15.6); losing Redis loses performance, never data |
| **Tantivy** | Full-text + attachment index | Per-user index dirs, encrypted at rest |
| **Blob store** | Raw messages/attachments cache | Filesystem (default) or S3-compatible; content-addressed, encrypted |

### 4.3 Crate topology

```
mailwoman/
├─ crates/
│  ├─ mw-engine        # orchestration: accounts, sync, rules, JMAP surface
│  ├─ mw-jmap          # JMAP client + server types (RFC 8620/8621/8887)
│  ├─ mw-imap          # hardened IMAP4rev2 client (RFC 9051 + extensions)
│  ├─ mw-pop3          # POP3 client (RFC 1939, UIDL, STLS/995)
│  ├─ mw-smtp          # submission client (RFC 6409, DSN, REQUIRETLS)
│  ├─ mw-sieve         # ManageSieve client + Sieve script model/AST
│  ├─ mw-mime          # MIME parse/build (wraps stalwart mail-parser/builder)
│  ├─ mw-crypto        # OpenPGP (rPGP), S/MIME (RustCrypto cms), PQC hybrid
│  │                   #   (also compiled to WASM for client-side E2EE ops)
│  ├─ mw-store         # sqlx data layer (Postgres/SQLite) + blob store + crypto-at-rest
│  ├─ mw-cache         # layered cache: memory → Redis (optional) → store
│  ├─ mw-search        # Tantivy full-text index (encrypted at rest)
│  ├─ mw-dav           # CalDAV/CardDAV client (calendar, tasks, contacts)
│  ├─ mw-ics           # iCalendar/vCard/ICS/.hol parse+emit, RRULE engine
│  ├─ mw-sanitize      # HTML email sanitizer (ammonia-based, CSS rewriter)
│  ├─ mw-export        # EML/mbox/MSG/OFT/PDF-print/TXT/MD/DOCX converters
│  ├─ mw-autoconfig    # RFC 6186 SRV, Thunderbird autoconfig, MS autodiscover v2
│  ├─ mw-assist        # AI gateway: endpoint adapters, permission scoping (§14)
│  ├─ mw-directory     # LDAP client, GAL, distribution groups (§13)
│  ├─ mw-sandbox       # process isolation: worker spawn, seccomp/Landlock
│  │                   # policies, wasmtime jail (§7.5)
│  └─ mw-server        # axum HTTP server, sessions, ACME, admin API, MCP,
│                      # webhooks, asset embedding
├─ apps/
│  ├─ web/             # TypeScript UI (SolidJS + Vite) — THE client
│  ├─ desktop/         # Tauri v2 thin shell (Win/macOS/Linux)
│  └─ mobile/          # Tauri v2 thin shell (iOS/Android)
└─ plugins/            # first-party plugins (§22)
```

Every crate that touches network bytes (`mw-imap`, `mw-pop3`, `mw-mime`,
`mw-jmap`, `mw-ics`, `mw-sanitize`, `mw-crypto`, `mw-export`) has
`#![forbid(unsafe_code)]` and a fuzz target.

---

## 5. Technology Stack

### 5.1 Backend / engine — Rust (stable, edition 2024)

| Concern | Choice | Why |
|---|---|---|
| Async runtime | `tokio` | Ecosystem standard; io_uring optional later |
| HTTP server | `axum` + `tower` | Middleware model fits security layering |
| TLS | `rustls` (+ `rustls-platform-verifier`, `rustls-acme` for Let's Encrypt) | Memory-safe, PQC-hybrid capable (X25519MLKEM768) |
| Database | `sqlx` → **PostgreSQL** primary, SQLite embedded fallback | Compile-time-checked SQL; one schema, two backends |
| Cache | `mw-cache` over `fred` (Redis/Valkey client) + in-process `moka` | Optional, scope-configurable (§15.6) |
| MIME | `mail-parser`, `mail-builder` (Stalwart) | MIT/Apache dual, battle-tested, fuzzed |
| Auth verdicts | `mail-auth` | DKIM/SPF/DMARC/ARC verification for display + metadata analysis |
| OpenPGP | `rpgp` (rPGP) | **MIT/Apache** (Sequoia is LGPL — excluded); powers Delta Chat |
| S/MIME | RustCrypto `cms`, `x509-cert`, `rsa`, `p256` | Pure Rust, permissive |
| PQC | RustCrypto `ml-kem` (FIPS 203) | Hybrid KEM for store keys & TLS |
| Password hashing | `argon2` (Argon2id) | OWASP-recommended parameters |
| Full-text search | `tantivy` | Lucene-class performance, pure Rust |
| HTML sanitizing | `ammonia` + custom CSS filter | Allowlist-based, no regex "sanitizing" |
| LDAP | `ldap3` | Auth bind, GAL, groups, cert lookup |
| DOCX export | `docx-rs` | MIT |
| MSG/OFT (CFB) | `cfb` crate + own MS-OXMSG layer in `mw-export` | Outlook interop |
| Serialization | `serde`, `serde_json` | JMAP is JSON-native |
| Logging/tracing | `tracing` + `tracing-subscriber` (+ optional OTLP) | §21 |
| Error reporting | `sentry` SDK (DSN-compatible: Sentry, GlitchTip, Bugsink) | Off by default, self-hosted targets first-class (§21.2) |
| Config | `figment` (TOML + env) | One file, env overrides for containers |
| i18n (server) | `fluent` | Matches frontend Fluent usage |

### 5.2 Frontend — TypeScript (strict)

| Concern | Choice | Why |
|---|---|---|
| Framework | **SolidJS** | Fine-grained reactivity; renders 100k-row virtual lists without VDOM diff cost; ~7 KB runtime honors the SnappyMail lightness ethos |
| Build | Vite + Rolldown | Fast, code-splitting per route/module |
| State/sync | Custom JMAP client store (typed from `mw-jmap` via codegen) in a **SharedWorker** | One live session shared by all windows/tabs (§15.5) |
| Heavy work | Dedicated Web Workers: search, crypto (WASM), indexing, export rendering | Main thread stays at 60 fps; Service Worker handles cache/offline/push |
| Styling | Vanilla-extract (zero-runtime CSS) + design tokens | Themeable (§17) without runtime CSS-in-JS cost |
| Editor | ProseMirror | Sane HTML output, plain-text mode, tables |
| Offline | Service Worker + OPFS (encrypted) + IndexedDB for queue | Full offline PWA (§15.4) |
| Tests | Vitest + Playwright | Unit + E2E |

TypeScript config: `strict`, `noUncheckedIndexedAccess`, `exactOptionalPropertyTypes`.
No `any` in `apps/web/src` (lint-enforced). JMAP types codegen'd from Rust —
drift is a build failure.

---

## 6. Protocol Support

### 6.1 Mail

| Protocol | RFCs | Level |
|---|---|---|
| **JMAP Core/Mail** | 8620, 8621 | Full client; WebSocket push (8887); Blob mgmt (9404) |
| **JMAP Sieve / quotas / contacts / calendars / tasks** | 9661, 9425, 9610, drafts | As servers ship them |
| **IMAP4rev2** | 9051 | Full, plus graceful IMAP4rev1 (3501) fallback |
| IMAP extensions | IDLE, CONDSTORE/QRESYNC (7162), OBJECTID (8474), MOVE, ESEARCH, SORT/THREAD, LIST-STATUS, SPECIAL-USE (6154), COMPRESS, LITERAL+, UTF8=ACCEPT, NOTIFY (5465), PREVIEW (8970), SAVEDATE, STATUS=SIZE, BINARY, ID, **ACL (4314)**, METADATA (5464) | Feature-detect; degrade gracefully |
| **POP3** — *guaranteed* | 1939, 2449 (CAPA), 1734/5034 (SASL), STLS (2595) + implicit TLS :995 | UIDL-based pull sync into the engine store; leave-on-server policies (keep / delete after N days / delete on retrieval); POP3 accounts get the full feature surface (search, tags, rules) because the engine owns the local copy |
| **SMTP Submission** | 6409 | Ports 465 (implicit TLS, preferred) & 587 (STARTTLS); PIPELINING, SIZE, 8BITMIME, SMTPUTF8, DSN (3461), REQUIRETLS (8689), CHUNKING |
| **Sieve/ManageSieve** | 5228, 5804 | Full script editor + GUI rule builder that round-trips to readable Sieve |
| Auth mechanisms | LOGIN, PLAIN, CRAM-MD5 (legacy), **OAUTHBEARER/XOAUTH2** (Gmail, M365, generic OIDC), SCRAM-SHA-256(-PLUS) | Channel binding where available |

### 6.2 Calendar, Tasks & Contacts

- **CardDAV** (6352) + vCard 3/4; **CalDAV** (4791) + iCalendar (5545);
  **VTODO** over CalDAV for tasks; scheduling via iTIP/iMIP (5546/6047);
  free/busy (CalDAV `free-busy-query` + iMIP VFREEBUSY).
- **Calendar sharing:** CalDAV sharing extensions (draft-pot-caldav-sharing /
  WebDAV ACL 3744) for cross-server; native fast-path sharing between users on
  the same Mailwoman server (§11.5).
- **Foreign calendars:** ICS/webcal:// subscriptions (poll with ETag), Google
  Calendar & Outlook calendars via the bridges (§6.5), read-only overlay
  calendars.
- **ICS files:** full import/export — single events, whole calendars,
  reply/counter objects; drag an .ics in, get an event; export any
  event/calendar as .ics. **.hol files** (Outlook holiday format) import and
  export, plus bundled holiday calendars per locale.
- JMAP Contacts/Calendars/Tasks used natively when the server supports them.

### 6.3 Discovery & account setup

Setup flow tries, in order: **(1)** JMAP session autodiscovery
(`/.well-known/jmap`), **(2)** SRV records (RFC 6186 / 8314), **(3)**
Thunderbird-style autoconfig XML, **(4)** Microsoft Autodiscover v2, **(5)** a
curated offline provider database, **(6)** manual entry (incl. POP3-only).
Target: any provider configured with just address + password/OAuth in < 10 s.

### 6.4 Transport security

- TLS 1.2 minimum, TLS 1.3 preferred, via rustls. **Cleartext connections are
  refused** — no "continue anyway" button; an admin-level override
  (`allow_insecure_localhost`) exists solely for same-host sockets.
- **Let's Encrypt built in:** `rustls-acme` (ALPN-01 and HTTP-01) — a bare
  `mailwoman serve --acme mail.example.com` gets and renews certs with zero
  external tooling. Also accepts external cert paths (with hot-reload on
  SIGHUP) for reverse-proxy/HSM setups.
- MTA-STS (8461) and DANE awareness for the submission path: the composer
  shows a transport-security indicator per recipient domain.
- Certificate pinning optional per-account (TOFU with alert-on-change).

### 6.5 Proprietary bridges (first-party plugins, shipped at 1.0)

Each bridge is a WASM engine plugin (§22) that implements the same internal
account-backend trait as the IMAP adapter — the engine and UI cannot tell a
bridge from a standards server. All three ship with 1.0. **Full Exchange
support — on-prem and cloud — is a commitment, not a stretch goal.**

| Bridge | Covers | Notes |
|---|---|---|
| **Microsoft Graph** | M365 / Outlook.com / Exchange Online (incl. tenants with IMAP disabled) | OAuth device-code + auth-code flows; mail, calendar (incl. shared calendars & rooms), contacts/GAL, tasks (To Do), categories, **Focused Inbox sync**, **native message recall**, **reactions**, **voting buttons**; admin app-registration guide + BYO app ID |
| **EWS** | On-prem Exchange 2013/2016/2019/SE | SOAP subset: sync, send, calendar + free/busy + rooms, GAL, Out-of-Office, recall, voting; NTLM/Kerberos + Basic auth |
| **Gmail API** | Google Workspace accounts where IMAP is org-disabled; label fidelity | True label semantics, history-ID delta sync, per-user OAuth client option |

Bridge policy: quirks live in the bridge, never in core; each bridge has its
own recorded-fixture test suite (no live-service dependency in CI) plus a
nightly live-interop job against real test tenants.

---

## 7. Security Architecture

### 7.1 Threat model

| Adversary | Capabilities | Mitigations |
|---|---|---|
| **Malicious email sender** | Hostile MIME/HTML/CSS/attachments, tracking, phishing, MIME smuggling, decompression bombs | Rust parsers (fuzzed), sanitizer (§7.2), no remote content by default, attachment sandbox, parser resource limits (depth/size/time) |
| **Network attacker** | MitM, downgrade, traffic analysis | rustls + no-cleartext policy, HSTS preload, pinning option, REQUIRETLS |
| **Compromised/curious host server** | Reads disk, memory, logs | Zero-access mode (§9); in standard mode: encrypted at rest, no plaintext logging of bodies/subjects ever |
| **Malicious other tenant / XSS** | Script injection via email content | CSP `default-src 'none'`-rooted policy (no inline scripts, `style-src 'self'`), sandboxed iframe rendering, no `innerHTML` of unsanitized data (lint-banned); Trusted Types enforced (`require-trusted-types-for 'script'` in the shipped CSP, §7.4) |
| **Stolen device** | Local data access | OS keychain-wrapped keys, optional app lock (biometric/PIN), auto-lock timer, remote cache wipe on next connect |
| **Credential stuffing / account takeover** | Password attacks on the webmail login | Argon2id, rate limiting + exponential backoff, WebAuthn/passkeys, TOTP, IP allowlists (admin), new-device notification |
| **Supply chain** | Malicious dependency | `cargo-deny` + `cargo-vet`/`cargo-audit`, lockfiles, pinned CI actions, reproducible builds, signed releases |
| **Exploited parser / compromised worker** | RCE in a parsing path (e.g., `unsafe` in a transitive dep, image codec bug) | Privilege-separated disposable render workers: no network, no filesystem, no keys, seccomp/Landlock/namespace-jailed, WASM second layer (§7.5) — exploit lands in an empty room |
| **Data exfiltration by insiders/users** | Sensitive content leaving via mail | DLP pipeline (§7.6), audit trail, admin policies |
| **Over-privileged automation** | API keys / MCP / AI agents doing more than intended | Scoped keys, per-capability grants, human-approval gates for send (§14, §20) |

### 7.2 HTML email rendering (the #1 webmail attack surface)

Rendering pipeline, all engine-side in Rust before anything reaches the DOM:

1. **Parse** with a real HTML5 parser (`html5ever` via ammonia). Never regex.
2. **Sanitize** against a strict allowlist: no `<script>`, `<object>`,
   `<embed>`, `<form>` (forms optionally re-enabled read-only), no event
   handlers, no `javascript:`/`data:text/html` URLs.
3. **CSS rewrite:** parse stylesheets, drop `position:fixed/sticky`,
   `@import`, external `url()`; namespace all selectors under the message
   container; enforce max z-index.
4. **Remote content proxy:** images load **off by default**; when enabled they
   go through an engine-side anonymizing proxy (strips cookies/referrer,
   normalizes UA, caches, optionally re-encodes images in the WASM jail to
   strip exploit payloads and metadata). The proxy fetch is SSRF-filtered
   deny-by-default: DNS-pinned against rebinding, private/loopback/link-local/
   ULA and the cloud-metadata address refused — including the IPv4 those ranges
   embed in NAT64 (`64:ff9b::/96`) and 6to4 (`2002::/16`) v6 forms — and every
   redirect hop re-validated. **Partial image loading:** load a
   single image, load all from this message, always-load per sender, or
   always-load per domain — four distinct grants, each revocable, with the
   remote-content bar showing exactly how many images were blocked and from
   which hosts.
5. **Tracker stripping:** known tracking-pixel patterns (1×1, known ESP hosts)
   removed and *reported* ("This message contained 3 trackers").
6. **Render inside a sandboxed `<iframe sandbox>`** with a per-message CSP,
   `credentialless`, no same-origin, height negotiated via postMessage.
7. **Link protection:** on click, show real destination; flag homograph/
   punycode lookalikes, mismatched text-vs-href, known-bad patterns.
8. **Maximum-security opening mode** (per-message action and per-sender/global
   policy): render plain-text-only, or sanitized-HTML-without-any-media, or
   full sanitized HTML — a three-position switch in the message toolbar;
   admin can pin the floor (e.g., "unknown senders always open in plain text").
   Attachments in this mode open only via the re-encoding preview jail, never
   the original bytes.

### 7.3 Message security analysis (metadata + signatures)

Every message gets a **Security panel** (collapsed chip by default, one click
to expand):

- **Authentication verdicts:** DKIM/SPF/DMARC/ARC parsed and explained in
  plain language ("Really from paypal.com ✅"), with the raw alignment details
  for experts.
- **Metadata analysis:** full `Received` chain visualization (hops, delays,
  countries/ASNs of relays via offline GeoIP db), submission client hints,
  reply-to vs from mismatches, envelope vs header divergence, message-ID
  domain anomalies, date skew.
- **Signature analysis:** for PGP/S/MIME — algorithm strength, key/cert age
  and expiry, chain of trust, revocation status (CRL/OCSP for S/MIME, key
  server/WKD recheck for PGP), verdict history with this correspondent
  ("first time this sender's key changed" alerts).
- **Attachment risk:** type mismatches (extension vs magic bytes), macro
  documents, encrypted archives, executables — flagged before opening.
- **Sender controls right in the panel:** Block sender (reject to Junk +
  Sieve rule), **Silence sender** (deliver normally, never notify), Ignore
  conversation (auto-archive the thread's future messages), Report phishing /
  Report junk (feeds the spam trainer §10.10 and an admin-configurable abuse
  reporting address, with ARF format where supported).

### 7.4 Web application hardening

- CSP: `default-src 'none'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self'; img-src 'self' blob: data:; font-src 'self'; connect-src 'self' blob:; frame-src 'self'; worker-src 'self' blob:; base-uri 'none'; form-action 'none'; require-trusted-types-for 'script'` (message iframes get their own stricter policy). No inline scripts anywhere, and `style-src 'self'` carries no `'unsafe-inline'`. Trusted Types (`require-trusted-types-for 'script'`) is **enforced**: the SPA entrypoint registers a **default** Trusted Types policy before first render, so the shell's template engine (`innerHTML` on a `<template>`) and every other HTML sink pass through a policy while dynamic script/script-URL sinks stay fail-closed. Fonts are self-hosted (§17.2) so `font-src 'self'` — never a third-party origin.
- `Cross-Origin-Opener-Policy: same-origin`, `COEP: require-corp`, `CORP`,
  `X-Frame-Options: DENY`, `Referrer-Policy: no-referrer`,
  `Permissions-Policy` denying everything unused.
- Session: httpOnly, Secure, SameSite=Strict cookies; opaque server-side
  session tokens (no JWT for sessions); rotation on privilege change; absolute
  + idle timeouts; concurrent-session listing and revocation in settings.
- CSRF: double-submit + Origin checks. All state-changing endpoints require
  the JMAP session state token.
- Login: constant-time compares, uniform error messages/timing, Argon2id
  (m=64 MiB, t=3, p=4 baseline; admin-tunable upward).
- 2FA: WebAuthn/passkeys and TOTP (RFC 6238) as a **second factor**, with
  recovery codes as the break-glass path. A TOTP code cannot be replayed within
  its validity window — the last step a login consumed is recorded and any step
  at or below it is refused (a compare-and-swap that also resolves two logins
  racing the same code to a single winner). Passkey verification uses attestation
  `"none"` — the server verifies the assertion signature and stores the COSE
  public key; it does not validate attestation certificate chains. A factor,
  once enrolled, is required at login (no silent downgrade to password-only);
  admins may require a second factor org-wide or per domain. A passkey as a
  **passwordless primary** login credential (replacing the password) is not yet
  offered — the passwordless mechanism today is the zero-access WebAuthn-PRF
  root-key derivation (§9.1), a separate feature.
- Secrets in memory: `zeroize` on drop; `mlock` best-effort for key material.
- Web-asset integrity manifest: thin shells verify the UI bundle hash served
  by the server against the shell's pinned release manifest before executing it.

### 7.5 Runtime hardening & memory segregation (defense in depth)

Rust removes the memory-corruption bug class from *our* code, but it does not
segregate memory: a logic bug, an exploited `unsafe` block in a dependency, or
a compromised transitive crate still owns the whole process. §7.5 assumes the
parser *will* one day be exploited and limits the blast radius.

#### Process architecture (privilege separation)

```
mailwoman (supervisor, no network)
├─ mw-net        # TLS termination + HTTP; no filesystem write, no upstream creds
├─ mw-session    # sessions, auth, JMAP routing; holds session keys only
├─ mw-sync       # upstream IMAP/JMAP/POP3/SMTP connections; no listening sockets
└─ mw-render[N]  # DISPOSABLE parser workers: MIME parse, HTML sanitize,
                 # image re-encode, PDF thumbnail, crypto envelope parse,
                 # export rendering. No network. No filesystem (fd-passing
                 # only). Short-lived, recycled after N jobs.
```

- **`mw-render` workers are the hostile-input jail.** CPU/memory/time-limited
  (rlimits + cgroup); a successful exploit lands in a process that can reach
  nothing — no keys, no network, no disk — and dies in seconds.
- **Linux (kernel jail):** per-worker seccomp-BPF allowlists, Landlock
  filesystem scoping, unprivileged user + PID/net namespaces —
  containerization-grade isolation *without requiring* Docker, so bare-metal
  self-hosters get it too. These kernel features are Linux-only
  (`#[cfg(target_os = "linux")]`) and **fail closed**: where a jail was expected
  but its setup fails, the render child is refused rather than run unsandboxed.
  Landlock is applied best-effort by default (it is unavailable on kernels
  < 5.13, so a missing/partial ruleset does not by itself refuse the child);
  `MW_RENDER_JAIL=strict` makes a Landlock setup that is not fully enforced
  fatal under a required jail, for operators who want fail-closed filesystem
  confinement.
- **Non-Linux (degraded mode):** on Windows and macOS the kernel jail is
  unavailable; isolation is the disposable process split plus the WASM layer
  below, with no seccomp/Landlock/namespace enforcement. `mailwoman doctor`
  reports this reduced posture plainly rather than letting operators assume it.
- **WASM second layer:** the riskiest transformers run compiled to WASM in
  wasmtime even in first-party builds. In this layer today: MSG/OFT CFB parsing
  and remote-image re-encode; PDF thumbnailing is tracked but not yet run here.
  On platforms without good process-sandbox primitives the WASM layer is the
  primary jail.
- A dedicated `mw-sandbox` crate owns spawn/seccomp/Landlock/wasmtime policy.

#### Toolchain & allocator hardening

- Release builds: `overflow-checks = true`, full RELRO + PIE + stack
  protectors, Windows Control Flow Guard, `panic = abort` in workers.
- Hardened allocator configuration in workers (guard pages, canaries); parser
  arena allocations bounded and torn down whole.
- Key material lives only in `mw-session`/crypto paths — never mapped in
  render workers.

#### Hardened container images (the spec, not the aspiration)

- Base: `FROM scratch` (fully static musl binary) — no shell, no package
  manager, no libc. Health checks via `mailwoman healthcheck` subcommand.
- Fixed non-root UID/GID (10001), `readOnlyRootFilesystem`, writable state
  only on an explicit volume, scratch on `tmpfs`.
- Shipped compose/Helm defaults encode: `cap_drop: [ALL]`,
  `no-new-privileges:true`, shipped custom seccomp + AppArmor profiles,
  `runAsNonRoot`, `seccompProfile: RuntimeDefault`, resource limits. Secure is
  the default someone copy-pastes.
- Supply chain: base pinned by digest, images signed (cosign) with SBOM +
  provenance attestations, CI vulnerability scan (grype/trivy) gating release,
  scheduled rebuilds.
- Runtimes that forbid nested namespaces: detected, fall back to WASM-only
  jailing, reduced posture logged at startup.

#### Systemd (bare-metal) hardening

Shipped unit file (CI-tested): `ProtectSystem=strict`, `ProtectHome=yes`,
`PrivateTmp`, `PrivateDevices`, `NoNewPrivileges`, `CapabilityBoundingSet=`,
`RestrictAddressFamilies=AF_INET AF_INET6 AF_UNIX`,
`SystemCallFilter=@system-service`, `ProtectKernel*`, `LockPersonality`,
`UMask=0077`, socket activation. `MemoryDenyWriteExecute` everywhere except
render workers when wasmtime JIT needs W^X pages; a `pure-interpreter` build
flag (wasmtime Pulley) exists for MDWE-everywhere at a throughput cost.

`mailwoman doctor` prints the live sandbox posture so operators verify
hardening instead of assuming it.

#### Phasing (matches §27)

Process boundaries exist from V0 — privilege separation never gets retrofitted
into a monolith. Policy depth phases in: V0 ships the split + hardened
container/systemd files + WASM jail; V1 adds seccomp/Landlock/namespace
allowlists and worker recycling; the external audit at V6 validates the posture.

### 7.6 Data protection features

- **Data Loss Prevention (DLP):** engine-side outbound pipeline hooks —
  admin-defined rules (regex/dictionary/detector packs for card numbers, IBANs,
  national IDs; attachment type and size policies; recipient-domain policies)
  with actions **warn / block / require-encryption / notify-admin**; per-rule
  audit trail; evaluated on send *and* on autosaved drafts leaving zero-access
  scope. Deeper classifiers pluggable via WASM (§22).
- **Screen-capture protection** (honest edition): thin shells set
  `SetWindowDisplayAffinity(WDA_EXCLUDEFROMCAPTURE)` on Windows,
  `FLAG_SECURE` on Android, screen-capture detection + content hiding on iOS,
  and screenshot-obscuring on macOS where the API allows. **The browser cannot
  prevent screenshots** — web deployments can enable a visible per-user
  watermark overlay (name/time tiled faintly) as a deterrent, and the docs and
  admin UI say exactly this instead of pretending.
- **Retention awareness:** the UI surfaces server-side retention/litigation
  hold status where bridges expose it (Graph/EWS); Mailwoman never silently
  deletes anything under hold.

---

## 8. End-to-End Encryption

### 8.1 OpenPGP (primary interop standard)

- **Library:** rPGP (MIT/Apache). RFC 9580 profile: v6 keys, Ed25519/X25519
  default; AEAD (OCB) packets; SHA-256 minimum.
- **Key management UX** (where PGP clients die; we invest here):
  - Generate per-account keys on first use (opt-in, one click, plain-language).
  - **Autocrypt Level 1**: opportunistic encryption "just happens".
  - **WKD** lookup + publishing guide; keys.openpgp.org (VKS) lookup with consent.
  - Key backup: encrypted export + Autocrypt Setup Message.
  - Trust model: TOFU with explicit verification (QR/fingerprint words).
    Verified badge per correspondent.
- **Private-key operations always run client-side** in the WASM build of
  `mw-crypto` — in every deployment shape, since the server is always remote
  from the UI now (§4.1). Private keys never reach the server unencrypted.
- Encrypted subject (protected headers), drafts always stored encrypted,
  encrypted search via the client-side index slice (§9.3).
- Signature verdict UI: three-state (verified ✅ / unverified key ⚠️ /
  invalid ❌) with plain-language explanations; deep detail in the Security
  panel (§7.3).

### 8.2 S/MIME (enterprise interop)

- RustCrypto `cms` stack; RSA-2048+ / ECDSA P-256; AES-GCM content encryption.
- Certificate sources: PKCS#12 import, OS keychain (via shells), **LDAP
  directory lookup** (§13), certificate harvesting from received signed mail.
- Same verdict UI as PGP. Outlook/Exchange interop test matrix is a release
  gate (§25). Full-message encryption (body + attachments + protected subject)
  for both standards.

### 8.3 Post-quantum readiness

- **TLS:** hybrid X25519MLKEM768 is **not enabled**. The shipped rustls `ring`
  provider does not offer the group, and the only provider that does
  (`aws-lc-rs`) is a C/`-sys` dependency the project's pure-Rust, no-`-sys`
  dependency posture excludes — so it cannot ship today. Tracked for when a
  pure-Rust rustls provider offers the group. (See `docs/security/crypto.md`.)
- **At rest:** store master keys wrapped with hybrid X25519 + ML-KEM-768.
- **OpenPGP PQC:** track `draft-ietf-openpgp-pqc`; behind a feature flag as
  rPGP lands support; interop-test with GnuPG 2.5+ and Thunderbird.
- Crypto agility: every stored object records its algorithm suite.

---

## 9. Zero-Access Storage Mode

Optional per-deployment (admin) and per-account (user): **the hosting server
can never read mail content.**

### 9.1 Key hierarchy

```
User passphrase / passkey (PRF extension)
        │  Argon2id (client-side)
        ▼
Root Key (never leaves client)
        ├─► Key-Encryption Key ──wraps──► Account Data Keys (per account)
        │                                   ├─► Message cache key
        │                                   ├─► Search index key
        │                                   ├─► Notes key (§12.2)
        │                                   └─► Attachment cache key
        └─► Recovery Key (printable phrase, optional — tradeoff explained)
```

- WebAuthn PRF lets a passkey derive the root key — passwordless zero-access.
- Multi-device: root key transferred via device-pairing QR flow (SAS-verified),
  never through the server in plaintext.

### 9.2 What the server sees in zero-access mode

Ciphertext blobs, opaque IDs, sizes, timestamps. It still must proxy
IMAP/SMTP (upstream credentials are sealed to the client session where OAuth
is unavailable). Documented honestly: zero-access protects **data at rest**
against a curious or breached host — a fully malicious *active* server
proxying live traffic is a stronger adversary, and the UI states this
difference plainly. Redis caching of plaintext-derived data is automatically
disabled for zero-access accounts (§15.6).

### 9.3 Encrypted store & search

- Postgres/SQLite rows for zero-access accounts hold only
  XChaCha20-Poly1305 ciphertext (AAD = table‖row‖schema-version).
- **Search:** the client builds a Tantivy index slice over decrypted content
  in OPFS, encrypted at rest with the search key. No server-side
  searchable-encryption snake oil.

---

## 10. Mail Features

### 10.1 Mailbox, folders & organization

- Unified inbox across accounts; per-account and per-folder views; SPECIAL-USE
  auto-mapping; **account reordering** (drag accounts into any order,
  persisted per user).
- **Full folder management:** create/rename/move/delete, nested folders,
  per-folder sync policy, IMAP ACL (RFC 4314) visualization and editing for
  shared folders, subscription management.
- **Favorites, colors, tags, pins:** favorite folders (pinned section at top),
  per-folder colors, tags (labels) on folders *and* messages with color +
  icon, **pin messages** (stay at top of folder/thread), pin folders, pin tags
  — all sync via IMAP keywords/METADATA or JMAP where possible, engine-side
  otherwise (sync convention documented publicly).
- **Search folders:** saved searches materialized as virtual folders (like
  Outlook's) — live-updating, nestable under a "Search Folders" tree, usable
  as notification sources.
- Virtualized list: smooth at 100k+ messages; sender avatars/BIMI (DMARC-gated);
  snippet previews; density options (compact/cozy/relaxed).
- **Conversation threading:** JMAP threads natively; engine-side JWZ threading
  for IMAP/POP3. Per-folder thread on/off.
- **Focused Inbox:** two-tab inbox (Focused/Other) driven by a local
  classifier (sender history, interaction frequency, rules) — optionally
  Assist-enhanced (§14); syncs bidirectionally with Outlook's Focused state
  via the Graph bridge; per-account toggle, trainable via "Move to
  Focused/Other" with "always do this for sender".
- Swipe actions (mobile, fully configurable per direction/length), hover
  quick-actions, drag-and-drop everywhere (§10.7), multi-select with
  Outlook muscle-memory semantics.
- **Sweep** (Outlook-style): from any sender — delete all, delete all + block,
  keep latest only, auto-delete older than N days — implemented as engine
  rules with preview-before-apply and an undo window.
- **Pinning, snooze, follow-up:** snooze (hide + resurface, cross-device);
  **follow-up flags** with due dates that surface in Tasks/My Day (§12.1) and
  as reminder notifications; "has the recipient not replied in N days?"
  follow-up nudges (local heuristic, opt-in).
- **Message classification:** user-defined classification labels (e.g.,
  Public/Internal/Confidential) attachable to messages and enforced by DLP
  rules on reply/forward (§7.6); Assist can suggest classifications (§14).
- **Ignore conversation / Block sender / Silence sender** (§7.3).
- Batch operations stream progress and are cancelable; optimistic UI with
  rollback; **undo everything** — archive, delete, move, spam, sweep, rules —
  10 s undo toast backed by real inverse operations.

### 10.2 Reading

- Reader pane (right/bottom/off), standalone view, and **sub-tabs**: any
  message, draft, event, or contact opens as a tab inside the app (restorable
  session), with tear-off into a real browser window (§15.5).
- Security panel with metadata/signature analysis (§7.3); remote-content bar
  with partial image loading (§7.2); maximum-security opening mode (§7.2).
- **Reactions:** react to messages with emoji — native via Graph bridge
  (Outlook reactions), Mailwoman-to-Mailwoman via a documented header
  convention; aggregated display on the thread. Degrades to nothing (never
  broken text) for other clients.
- **Voting buttons:** render and answer Outlook voting buttons (Graph/EWS
  native); compose votes on standards accounts as one-click reply buttons
  (`X-Mailwoman-Vote` header + plain-text fallback so any client can answer);
  tally view for the sender.
- Attachments: inline preview (images, PDF via sandboxed PDF.js, text,
  audio/video, office docs via re-encoded preview), thumbnail strip, save-all,
  save-to-Nextcloud (§18.4), drag-out to OS (shells).
- **Attachment intelligence:** dedicated **Attachments module** — a global
  view of every attachment across accounts (grid/list, filter by type, sender,
  size, date, account), full attachment search (`filename:report type:pdf
  larger:5M from:anna`), storage statistics.
- ICS invites: accept/tentative/decline inline with conflict awareness (§11.4).
- `.eml` open/save natively; **MSG and OFT** open/import (CFB parsing in the
  WASM jail); print with dedicated print CSS and **print-to-PDF** (§10.6).
- Quick entity actions (addresses, tracking numbers, flight codes) — all local
  heuristics, no cloud extraction.

### 10.3 Composing

- ProseMirror rich text with sane HTML output (tested against Outlook/Gmail
  quirks); markdown-shortcut input; plain-text mode with format=flowed;
  per-identity default. Font family/size defaults per user (§17.2).
- **Identities & profiles, fully configurable:**
  - Multiple **profiles** (e.g., Work/Personal) grouping accounts, identities,
    signatures, and theme — switchable in two clicks.
  - Multiple **from addresses** per account: manually added aliases *and*
    **server-provided allowed-froms pulled automatically** (JMAP `Identity/get`,
    Sieve capabilities, Dovecot METADATA, Graph `proxyAddresses`, admin-
    provisioned per-domain alias lists) — user picks which pulled identities
    to show; reply-identity auto-selection by recipient/folder.
  - Per-identity: signature, reply-to, sent-folder mapping, S/MIME cert,
    PGP key, default encryption posture.
- **Signature facilities, extensive:** rich-text/plain/image signatures with
  template variables ({{name}}, {{title}}, {{date}}, per-locale), multiple
  signatures per identity with rules (new vs reply/forward, internal vs
  external recipients), signature preview in composer, admin-managed
  org-wide signature templates with locked regions, vCard/business-card
  attachment option, per-profile defaults.
- **Undo send:** true delayed submission (engine holds N seconds, 0–120,
  survives tab close). **Send later:** engine queue, survives restarts, uses
  JMAP `sendAt` when available. **Outbox:** a real, visible outbox — queued,
  scheduled, failed-retrying, and offline-queued messages with per-item
  cancel/edit/send-now; failures surface as actionable toasts, never silence.
- **Drafts, everywhere, for everything:** autosave (encrypted always);
  server-synced mail drafts; and a universal **Drafts drawer** that also holds
  unfinished events, meetings, contacts, tasks, and notes (§11.3, §12) — any
  half-created item is resumable on any device.
- **Message recall (honest matrix):** native recall via Graph/EWS on Exchange
  targets; Mailwoman→same-server-Mailwoman recall deletes-if-unread (admin
  policy); plain SMTP to foreign servers — **impossible**, and the UI says so,
  offering a "send correction" flow instead of pretending.
- Attachments: chunked/background uploads, pause/resume, size warnings with
  server-limit awareness, inline images, "forgot the attachment" nudge; large
  attachments via **Nextcloud share links** (§18.4).
- Recipient chips show encryption capability; live banner: "this message will
  be: E2EE / TLS / mixed". DLP evaluation pre-send with inline explanations (§7.6).
- **Read-receipt / open-tracking (sender side, disclosed):** two mechanisms,
  both **off by default**, both admin-lockable:
  1. Standards: MDN read receipts (RFC 8098) — request + respond, default "ask me".
  2. **Pixel tracking (self-hosted):** embeds a pixel served by *your*
     Mailwoman server — never a third party; open events (time, open count,
     coarse client hint — no IP retention by default, admin-configurable)
     show on the sent message and in a per-message open timeline.
     **Activation modes** (per identity and per account): off · per-message
     opt-in (compose toolbar toggle) · **default-on with per-message
     opt-out** (toggle inverts). **Disclosure modes:** footer disclosure
     line (localized, links to a served notice page) · **silent** (no
     recipient-visible marker). Both axes are user-configurable where the
     admin allows; admin policy can pin either axis tenant-wide (e.g.,
     force-disclosure or force-off) and defaults ship as off + disclosure.
     The spec acknowledges the tension frankly: Mailwoman *blocks* others'
     trackers by default while offering its own — and recipient-side,
     Mailwoman's own remote-content proxy defeats pixels like these anyway,
     so open data is best-effort by nature (cached loads, proxies, and
     preview fetchers can all mask or fake opens; the UI labels results as
     approximate).
- Templates ("quick parts"), canned responses; **OFT template import/export**;
  keyboard-driven emoji/mention pickers; spellcheck via OS/browser; grammar
  and text review via Assist (§14) — local LanguageTool integration remains
  the no-AI path.
- **Dictation:** browser SpeechRecognition / OS dictation where available, or
  Assist speech-to-text against a configured endpoint (local Whisper server
  first-class) — push-to-talk in the composer, with AI cleanup pass optional (§14).

### 10.4 Search

- Local/server hybrid: engine-side Tantivy index — **< 50 ms** p95 over 100k
  messages; prefix, phrase, fuzzy, field queries (`from:`, `to:`, `subject:`,
  `has:attachment`, `filename:`, `before:/after:`, `in:`, `is:unread`,
  `larger:`, `tag:`, `pinned:`), boolean operators; attachment content
  indexing (text extracted in the render jail).
- Query builder UI round-trips with text syntax; saved searches become search
  folders (§10.1).
- **AI-powered search (opt-in, §14):** the Assist endpoint can compute query
  embeddings for natural-language queries ("that invoice from the contractor
  in spring") through the user's configured endpoint. Re-ranking the search
  index by those embeddings — and persisting them encrypted alongside the
  index — is **not yet wired**: today the capability produces embeddings but
  the Tantivy index does the ranking. Tracked follow-up. Off = classic search
  only, full-featured.
- Falls back to server search (IMAP SEARCH / JMAP `Email/query`) for
  not-yet-synced ranges, transparently merged and labeled.

### 10.5 Rules & automation

- **Mail rules like Outlook:** condition/action builder (sender, recipients,
  subject/body matches, has-attachment, size, importance, tags, classification)
  with actions (move, copy, tag, forward, reply-with-template, mark, play
  sound, notify, run-webhook §20) — executed server-side via **Sieve
  round-trip** when the server supports it, engine-side otherwise, with a
  clear indicator of where each rule runs.
- **Folder rules:** per-folder auto-tagging, retention (auto-archive/delete
  after N days), notification policies.
- Raw Sieve editor (syntax highlighting, linting) for power users; client
  rules for POP3/no-Sieve servers; "filter messages like this" one-click;
  rules testable against existing mail (dry-run preview).
- **AI auto-organization (opt-in, §14):** Assist suggests tags, folders, and
  focused/other placement; runs in **suggest mode** (badge + one-click apply)
  or **auto mode** per rule, with per-rule scopes and full audit of what AI
  moved where — reversible in bulk.

### 10.6 Export, import & printing

- **Export formats:** EML (native), mbox, **MSG**, **PDF**, TXT, **Markdown**,
  **DOCX** — single message or bulk; conversations exportable as one document.
  MSG/OFT written by `mw-export` (MS-OXMSG via `cfb`), rendered exports go
  through the sanitizer + print pipeline.
- **Print & print-to-PDF:** dedicated print stylesheets (message, thread,
  calendar views, contact cards); client-side print dialog everywhere;
  engine-side batch HTML→PDF rendering in the jail for bulk export.
- Import: mbox/EML/Maildir/**MSG**; server-to-server migration wizard
  (IMAP→IMAP with progress/resume); Thunderbird profile settings import.
  **No lock-in, ever.**

### 10.7 Drag & drop (configurable)

Drag in from anywhere (OS files → attachments or folder import; ICS → event;
vCard → contact), drag out (attachments, messages as .eml, events as .ics),
drag between (messages → folders/tags, attachments → composer). Every DnD
surface individually toggleable in settings (and by admin policy — DLP can
disable drag-out of classified content).

### 10.8–10.10 (Junk, honesty matrices)

- **Junk / spam — train the server:** Junk/Not-junk buttons set `$Junk`/
  `$NotJunk` keywords + SPECIAL-USE moves; first-party **Rspamd** and
  **SpamAssassin trainer** plugins; verdict display parses `X-Spam-*`/Rspamd
  headers into plain language; block/allow lists sync to Sieve. **Report
  phishing** and **Report junk** buttons appear contextually (§7.3), feeding
  trainers + admin abuse address (ARF). No client-side Bayes in core.
- Message recall honesty matrix lives in §10.3; screen-capture honesty in §7.6.

---

## 11. Calendar

### 11.1 Views

Day · **3-day** · work week · week · month · **tri-month (quarter)** ·
**schedule view** (Outlook-style horizontal timeline for comparing calendars)
· **list/agenda** · year heat-map. All views: keyboard navigable, printable
(with print-to-PDF), time-zone aware (secondary TZ column optional), week
numbers optional, configurable work hours/days, mini-calendar navigator.

### 11.2 Calendars & sync

- Multiple calendars per account, color-coded (user-overridable colors);
  CalDAV, JMAP Calendars, Graph/EWS (incl. shared & room calendars), ICS/
  webcal subscriptions (read-only overlays), Nextcloud calendars (CalDAV).
- Offline-capable: full event cache in the client store; changes queue and
  replay (§15.4).
- Holidays: bundled per-locale holiday calendars + **.hol import/export**.

### 11.3 Events (full Outlook parity)

- Create/edit with: title, location(s), online-meeting URL field, all-day,
  multi-day, RRULE recurrence (editor covering Outlook's cases + raw RRULE),
  reminders (multiple, per-event), categories/tags, color, busy status
  (free/tentative/busy/OOO), private flag, attachments, rich-text body.
- **Quick create:** click-drag on any view or natural-language quick-add
  ("lunch with Ana tomorrow 1pm") parsed locally; Assist can enhance parsing
  (§14) but the local parser is the default.
- **Attendees, full support:** required/optional/resource; availability
  lookup (free/busy) with suggested times; invite send/receive (iTIP/iMIP);
  attendee responses incl. **counter-proposals** (accept/decline/tentative/
  propose-new-time); forward-invite handling; **send updates to participants**
  on any change, with "only added/changed attendees" option; organizer view
  of response status.
- **Event drafts:** any half-filled event saves to the Drafts drawer (§10.3).
- ICS import/export at event and calendar granularity (§6.2).

### 11.4 Conflict management

- Live conflict detection while composing an event (your calendars + attendee
  free/busy); conflict chips on the event card.
- **Resolve on the spot:** side-by-side conflict resolver — move either event
  (with update-sends), shorten, mark tentative, or double-book deliberately;
  bulk conflict scanner ("show all conflicts this month") with the same
  resolution actions inline.

### 11.5 Sharing

- **Share calendars with other users:** on-server sharing (instant, permission
  levels: availability-only / read / read+write / delegate) and cross-server
  via CalDAV sharing/WebDAV ACL where supported; share by email invitation.
- **Receive & visualize shared calendars:** accept into your list, overlay
  with distinct styling, per-shared-calendar notification settings, delegate
  mode (act-on-behalf where the backend supports it — Graph/EWS).
- All sharing state visible and revocable in one settings page ("who can see
  what, and what have I accepted").

### 11.6 Meeting intelligence (Assist-gated, §14)

- **Meeting recaps:** post-meeting, generate a recap from the invite thread,
  attached agenda/notes, and (if the user pastes/uploads one) a transcript —
  action items extracted into Tasks (§12.1). Strictly on-demand, endpoint-BYO.
- Scheduling assistance: "find a slot for these 4 people next week" — works
  rules-based from free/busy without AI; Assist adds natural-language polish.

---

## 12. Tasks & Notes

### 12.1 Tasks

- Full tasks module: lists, due/start dates, reminders, recurrence, priority,
  progress, subtasks (checklist), tags, notes field, attachments (by
  reference).
- Sync: **VTODO over CalDAV** (Nextcloud/Radicale-compatible), JMAP Tasks
  where available, Microsoft To Do via Graph bridge.
- **My Day / "today" view:** a daily working set — tasks due today, follow-up
  flagged mail (§10.1), today's events; drag anything in; resets daily with
  carry-over prompts (Outlook's My Day + To Do model).
- Convert anywhere: mail → task ("follow up"), event → task, note → task.
- Task drafts in the universal Drafts drawer.

### 12.2 Notes (encrypted at rest, always)

- Outlook-style notes module: quick notes with rich text, tags, colors, search,
  pinning; linkable to messages/events/contacts.
- **Encryption at rest is not optional for notes:** stored via the notes key
  (§9.1) even for accounts not otherwise in zero-access mode — server sees
  ciphertext only; search via the client-side index slice. Sealing covers the
  note's metadata as well as its body: title, tags, color, and the pinned flag
  are sealed columns at rest, with the pinned-first list ordering applied in
  Rust after decrypt so no plaintext sort key survives on disk.
- Sync: Mailwoman-native (encrypted blobs through the server) as primary;
  optional export/interop via IMAP Notes folder convention and CalDAV
  VJOURNAL for interoperability (both lose the zero-access property when
  enabled — the toggle explains this).

---

## 13. Contacts & Directory

- **Contacts database** backed by the primary store (PostgreSQL): CardDAV /
  JMAP Contacts / Graph/EWS sync; vCard 3/4 with photos, multiple emails/
  phones/addresses, custom fields, birthdays (feed the calendar).
- **Organization:** address books (multiple, per-account + local), contact
  **lists**, **groups**, **favorites** (quick-access row in composer +
  dedicated view), tags and colors like mail.
- **Distribution groups:** personal distribution lists (expand on send or
  send-as-list), plus **server-side distribution groups** read from LDAP/GAL/
  Graph — visible, expandable-before-send ("who is actually in this?").
- **Global Address List:** LDAP-backed GAL (§5 `mw-directory`) and Exchange
  GAL via bridges — searchable in every recipient field, offline GAL cache
  with configurable refresh, photos where the directory provides them.
- **Business cards:** vCard-based business-card render (Outlook-style card
  view), attach-my-card in composer (per identity, §10.3), receive/import
  cards from attachments in one click.
- Merge-duplicates assistant; import/export vCard + CSV with mapping UI;
  per-contact security tab (PGP keys, S/MIME certs, verified status, key
  history); per-contact policies (always load images, always plain-text, …).
- **LDAP, full support:** authentication bind (§18.5), GAL, distribution
  groups, S/MIME certificate lookup, photo attributes, paged search, StartTLS/
  LDAPS, multiple directories with priority order. Read-only at 1.0
  (directory writes are an admin-provisioning concern, not a client's).

---

## 14. AI Subsystem — Assist

Assist is a **first-party but strictly opt-in subsystem** — deeply integrated,
never load-bearing. Every feature in this section is invisible until an
endpoint is configured, and every capability is individually permission-gated.

### 14.1 Endpoint model (BYO, always)

- Adapters: **OpenAI-compatible** (covers Ollama, llama.cpp, LM Studio, vLLM,
  most gateways), **Anthropic API**, and a local-process adapter (spawn a
  configured binary — fully offline). Embeddings + chat + speech-to-text
  (Whisper-compatible) endpoint slots, each independently configurable.
- Configured per deployment (admin) and/or per user (if admin allows). No
  Mailwoman-hosted default, no trial, no baked-in key. Unconfigured = zero AI
  UI.
- All Assist traffic goes through the engine's **Assist gateway** (`mw-assist`),
  which enforces scopes, redaction, rate limits, and audit — the UI never
  talks to an AI endpoint directly.

### 14.2 Permission & scoping model

- Capabilities are granted individually, like OS permissions:
  `assist.summarize`, `assist.draft`, `assist.grammar`, `assist.dictation`,
  `assist.search-semantic`, `assist.auto-tag`, `assist.recap`,
  `assist.assistant` (chat) — each per-user-grantable, admin-lockable
  tenant-wide, and scoped by **data class**: which accounts, which folders,
  whether E2EE-decrypted content may ever be included (default: never),
  whether attachments may be included (default: no).
- **Send is always human-gated:** Assist can draft, never transmit. There is
  no capability that sends mail, accepts invites, or deletes anything.
- Every Assist call is audit-logged (capability, data scope summary, endpoint
  host — never content) and the per-message UI shows a "what left the device"
  disclosure on demand.
- Background/batch AI (auto-tagging, §10.5) runs only for capabilities
  explicitly set to auto mode, and each action is attributed + bulk-reversible.

### 14.3 Capabilities (all gated per §14.2)

- **Text:** grammar and text review (inline suggestions in composer),
  rewrite/tone/translate, thread summarization, reply drafting.
- **Dictation:** speech-to-text in composer/notes/tasks (Whisper-endpoint or
  OS/browser speech), optional cleanup pass.
- **Organization:** auto-tag/auto-file suggestions, focused-inbox scoring
  boost, classification suggestions (§10.1).
- **Search:** natural-language query embeddings for semantic re-ranking
  (§10.4) — the embedding capability ships; wiring the re-rank into the
  search index is a tracked follow-up.
- **Calendar:** meeting recaps + action-item extraction (§11.6), NL quick-add
  enhancement.
- **Assistant:** a chat panel that can *read* (scoped) mailbox/calendar
  context and *propose* actions rendered as one-click confirmations — powered
  by the same tool surface as MCP (§20.3), inheriting its scoping. The
  assistant is a client of the API like any other; it has no privileged path.

---

## 15. Realtime, Sync, Offline & Caching

### 15.1 Realtime

- **WebSocket JMAP push** (RFC 8887) from server to every client; EventSource
  fallback; upstream change ingestion via JMAP push / IMAP IDLE+NOTIFY / POP3
  polling / Graph change notifications — normalized into one push stream.
- Live everywhere: new-mail, flag changes, calendar updates, task changes,
  presence of other sessions — UI updates in place without refresh.

### 15.2 Sync engine

- JMAP upstream: delta sync (`/changes` + query state). IMAP: QRESYNC →
  CONDSTORE → UID-window polling. POP3: UIDL diff pull. Bridges: native delta
  APIs (Graph delta queries, EWS sync folders).
- Per-account sync policy: headers window (30/90/365 days/all), bodies
  on-demand or prefetch, attachment policy separate, per-folder overrides.
- Conflict rules: server wins on flags, client wins on drafts, moves
  idempotent by stable ID.

### 15.3 Connection status UX

- A persistent, unobtrusive connection indicator (per upstream account and to
  the Mailwoman server itself) with **nice toasts**: offline ("working
  offline — 3 messages queued"), reconnected ("back online — syncing"),
  degraded ("Gmail throttling, retrying in 30 s"), auth-expired (actionable
  re-auth button). Toasts are actionable, deduplicated, and respect
  reduced-motion + quiet hours.

### 15.4 Offline (web included)

- **Full offline for the web client:** Service Worker precaches the app shell;
  OPFS holds the encrypted message/PIM cache + search index slice; IndexedDB
  holds the outbound queue (sends, edits, RSVP, task changes) — everything
  replayable. Read, search, compose, file, flag, manage calendar/tasks/notes
  offline; the Outbox (§10.3) shows queued state honestly.
- Background Sync API where available; the thin shells get the same behavior
  via the same code plus OS background-fetch privileges.
- Storage budgets configurable (per device: "keep 90 days + pinned + flagged
  offline"), with an explicit eviction policy screen.

### 15.5 Multi-window & sub-tabs

- **One session, many windows:** a SharedWorker owns the JMAP client store per
  browser profile; all windows/tabs subscribe — state (reads, selections,
  drafts, toasts) is consistent across windows in real time; BroadcastChannel
  fallback where SharedWorker is unavailable.
- **Sub-tabs:** an in-app tab strip (messages, composers, events, contacts,
  notes, settings pages) with pinnable tabs, restorable sessions, and
  keyboard cycling; any sub-tab tears off into a real OS window (`window.open`
  → same SharedWorker session; in shells, a real second window).
- Compose windows survive: crash/close recovery from the autosaved encrypted
  draft, always.

### 15.6 Caching (layered, fully scope-configurable)

`mw-cache` implements: **in-process memory (moka) → Redis/Valkey (optional) →
store (Postgres/SQLite)**. Redis is *never* required and never authoritative.

Admin configuration is an explicit scope matrix — each cacheable class is
individually assignable to layers with TTLs:

| Class | Default | Redis-eligible | Notes |
|---|---|---|---|
| Sessions | memory + store | ✅ | Enables zero-sticky multi-replica later |
| Header windows (hot folders) | memory | ✅ | Biggest win for large mailboxes |
| Message bodies | store only | opt-in | Encrypted values only |
| Blobs/attachments | disk/S3 | ❌ | Content-addressed |
| Search hot-set | memory | ✅ | |
| Push fan-out / presence | memory | ✅ | |
| Rate-limit counters | memory | ✅ | |
| GAL/directory cache | memory + store | ✅ | |

Zero-access accounts: any class containing plaintext-derived data is
automatically excluded from Redis and memory layers beyond per-request scope —
enforced in `mw-cache`, not by operator diligence. `mailwoman doctor` prints
the effective cache posture.

---

## 16. Clients: Web-First, Thin Desktop & Mobile

- **The web client is the product.** Every feature ships web-first; shells add
  OS integration only. PWA installable (manifest, share target, file handlers
  for .eml/.ics/.vcf/.msg).
- **Thin shells (Tauri v2):** Windows (msi/winget), macOS (universal,
  notarized), Linux (AppImage/deb/rpm/Flatpak), iOS, Android (Play +
  F-Droid-friendly). Shells contain: pinned UI bundle + integrity verification
  (§7.4), server URL management (multiple servers, e.g., work + personal),
  OS keychain, native notifications with actions, default-mailto/share/file
  handlers, badge counts, biometric app-lock, FLAG_SECURE/capture protection
  (§7.6), drag-out, print services.
- **Self-contained desktop mode:** bundled local server for serverless users
  (§4.1) — the one place the "thin" client carries the engine, as a spawned
  sibling process, not linked in.
- Push: Web Push (VAPID) on web/desktop; **UnifiedPush** on Android;
  APNs on iOS (opaque wake signal only — content never transits push);
  self-hostable push relay (§28.7).
- Size budgets: shells **< 10 MB** (no engine), self-contained desktop
  < 40 MB. Auto-update signed + staged; self-hosters can pin/disable.

---

## 17. Theming, Fonts & Personalization

### 17.1 Theming

- **Design-token architecture** (vanilla-extract): every color, radius,
  spacing, elevation, and texture is a token; themes are token packs — **no
  arbitrary CSS injection** (security, §7.4).
- Built-in themes: light, dark, high-contrast (both), AMOLED black, and the
  flagship pair — **"Grove Light" and "Grove Dark": warm woody palettes with
  subtle wood-grain and paper textures** (SVG/AVIF texture assets, tiled,
  < 30 KB total, disabled automatically under `prefers-reduced-transparency`/
  data-saver, and in high-contrast).
- Theme packs: tokens + texture assets + preview, distributed as signed
  packages (same registry as plugins); per-user theme choice, per-profile
  themes (§10.3), auto light/dark by OS schedule; admin-brandable login page
  and default theme; density and accent-color user overrides on top of any theme.

### 17.2 Fonts

- **Self-hosted Google Fonts with a puller — zero tracking:**
  `mailwoman fonts pull "Inter" "Lora"` (CLI + admin UI) downloads families
  from Google Fonts at *setup time*, subsets them (unicode-range), and serves
  them from the Mailwoman origin. **No runtime requests to Google ever** —
  CSP `font-src 'self'` makes this structural, not a promise. Bundled
  defaults: Inter (UI), a serif and a mono, packaged in-repo.
- **Font configurability:** per-user UI font, reading font (applies to
  plain-text and can force-override HTML mail typography in "reader mode"),
  compose default font family/size (sent as inline styles for interop), mono
  font for raw/source views; size scaling (85–150%) independent of browser zoom;
  admin can pin an org font set.

### 17.3 Personalization

Configurable: layout (three-pane/two-pane/vertical), density, reading-pane
position, swipe gestures, keyboard preset (Gmail/Outlook/custom/vim), start
module (mail/calendar/tasks), toast verbosity, sounds, per-folder + per-sender
notification rules, quiet hours, snooze presets, sweep defaults — all synced
server-side per user, all exportable as a settings JSON.

---

## 18. Server, Hosting & Ecosystem Integration

### 18.1 Deployment shapes

| Shape | How |
|---|---|
| **Single binary** | `mailwoman serve` — embedded assets, embedded ACME/Let's Encrypt (§6.4), TCP or Unix socket |
| Behind **nginx / Apache / Caddy / Traefik / HAProxy** | `X-Forwarded-*`/PROXY protocol, WebSocket pass-through, subpath hosting (`/mail`), tested config snippets in `docs/deploy/` |
| **FastCGI** | `mailwoman fcgi` for shared-hosting environments (closest analog to SnappyMail's PHP deployability) |
| **Container** | `FROM scratch` hardened image (§7.5); compose + Helm chart with secure defaults |
| **Systemd** | Socket activation + hardened unit (§7.5) |
| Hosting panels | Recipes for cPanel, Plesk, CloudPanel, ISPConfig; Cloudron/YunoHost/runtipi packages (community-maintained, CI-smoke-tested) |

### 18.2 Mail-server pairings (tested first-class)

Stalwart (JMAP flagship), Dovecot + Postfix (incl. master-user), Cyrus, Maddy,
mailcow / Mailu / docker-mailserver recipes, Gmail/Workspace (OAuth + quirks
layer), Microsoft 365 + on-prem Exchange (bridges §6.5), iCloud/Yahoo/Fastmail.
Conformance matrix CI-tested against Dovecot, Stalwart, Cyrus, Greenmail
containers on every merge.

### 18.3 Authentication & password management

- Login backends: local (Argon2id), **upstream-IMAP passthrough** (SnappyMail
  model), **OIDC/OAuth2 SSO** (Keycloak, Authentik, Authelia, Entra ID),
  **LDAP bind**, header auth behind trusted proxies (explicitly enabled +
  IP-restricted).
- **Password change, first-class:** in-app password change with pluggable
  backends — local store, **LDAP password modify (RFC 3062)**, Dovecot HTTP
  admin API, poppassd, **custom webhook** (HMAC-signed, for any panel/PAM
  glue) — plus forced-change-on-next-login (admin), password policy display,
  and coordinated re-encryption of sealed credentials after a change.
  Zero-access accounts additionally re-wrap their key hierarchy (§9.1)
  client-side; the recovery-key path is offered proactively before any change.

### 18.4 Nextcloud integration (first-party plugin)

- Link account via OAuth/app-password; **attach from Nextcloud Files**
  (WebDAV picker), **save attachments to Nextcloud**, send large attachments
  as **Nextcloud share links** (with optional password/expiry set in the
  composer); calendars/contacts/tasks via its CalDAV/CardDAV endpoints work
  in core already — the plugin just auto-configures them from the linked account.

### 18.5 LDAP (directory) — see §13; LDAP auth — §18.3.

---

## 19. Admin Panel

Separate route (`/admin`), separate session domain, optional separate
port/Unix socket; a required second factor (passkey/TOTP) can be enforced for
admin access (§7.4). **Full management surface:**

- **Domains:** per-domain upstream settings, autoconfig test button, login
  domain allow/blocklists, per-domain identity/alias provisioning (feeds
  §10.3 allowed-froms).
- **Users:** provisioning (local mode), quotas, session listing + revocation,
  per-user feature flags (zero-access, Assist, tracking pixel, DnD scopes),
  password reset/force-change, remote cache wipe.
- **Security policy:** min TLS, 2FA required, session lifetimes, Argon2
  params, remote-content proxy policy, max-security floors (§7.2), DLP rules
  (§7.6), watermarking, screen-capture policy for shells.
- **Assist governance:** endpoint allowlist, capability locks, data-class
  ceilings, tenant-wide kill switch (§14.2).
- **Integrations:** LDAP directories, Nextcloud, spam trainers, password-change
  backends, webhooks, MCP/API key oversight (§20).
- **Observability:** log level/target control, error-reporting DSN, audit log
  viewer + export, login monitor with ban list (fail2ban-compatible log
  format).
- **Appearance:** branding, default theme, font packs (§17), org signatures.
- Everything in the panel is also in TOML/env/`mailwoman admin` CLI —
  GitOps-friendly; panel optional (`admin.enabled = false`).

---

## 20. API, Webhooks & MCP

### 20.1 API

- **JMAP is the API** — the same surface the UI uses is the public API,
  documented and versioned. A thin REST convenience layer
  (`/api/v1/messages…`) generated over it for curl-friendliness.
- **Scoped API keys:** created per user (and per admin for admin APIs) with
  explicit scopes — read-only, per-account, per-folder subset, mail-only vs
  PIM, **no-send**, no-delete, time-boxed expiry, IP allowlist per key.
  Keys are hashed at rest, shown once, individually revocable, per-key rate
  limits and full per-key audit trail (§21).
- OAuth 2.1 authorization-code flow for third-party apps (admin-approved
  client registry), token exchange for the shells.

### 20.2 Webhooks

Outbound webhooks (HMAC-signed, retried with backoff): message-received
(filtered by rule §10.5), flag/tag events, calendar events, task events, DLP
verdicts, admin events. Inbound webhook actions available to rules.

### 20.3 MCP (Model Context Protocol) — full support, maximum security

- `mailwoman` exposes an **MCP server** (Streamable HTTP at `/mcp`, plus
  stdio mode via `mailwoman mcp-stdio` against a configured server) so any
  MCP client (Claude, IDEs, agents) can operate on the mailbox.
- **Tools** (each individually grantable per key): search/read mail,
  list/read folders, create drafts, **send — disabled by default and, when
  enabled, gated by human-in-the-loop approval** (the send lands in Outbox
  §10.3 pending in-app confirmation unless the key was explicitly created
  with `unattended-send`, which requires admin countersign), calendar
  read/propose, tasks read/write, contacts read.
- Auth per MCP spec: OAuth 2.1 with resource indicators (RFC 8707); MCP keys
  are API keys (§20.1) and inherit the same scoping, expiry, rate limiting, and
  audit. Audience enforcement is **on by default** for any deployment that has
  configured its public origin (`MW_WEBAUTHN_ORIGIN`) or set `MW_MCP_RESOURCE`
  explicitly: a bearer token bound to a different resource is rejected as
  wrong-audience before it reaches a tool. With neither configured, no canonical
  resource can be derived and enforcement stays off — so a deployment exposing
  MCP publicly must set one of the two.
  Zero-access accounts: MCP sees only what a logged-in client session could
  decrypt — i.e., nothing, unless the user runs a client-side MCP bridge.
- Prompt-injection posture: tool results carry provenance labels (message
  content is marked untrusted); tool descriptions instruct agents that mail
  bodies are untrusted input; no tool composes raw protocol commands.

---

## 21. Observability & Logging

### 21.1 Logging (extensive, privacy-preserving)

- `tracing`-based structured logs (JSON or logfmt), per-subsystem levels
  (`mw_imap=debug,mw_sync=info`) hot-reloadable via admin panel/SIGHUP;
  targets: stdout, rotating files, syslog, journald.
- **Privacy floor is structural:** message bodies, subjects, and attachment
  names never enter logs at any level — enforced by typed logging wrappers
  (the types that could leak don't implement `Display`), not discipline.
- Protocol wire tracing (redacted literals) available per-account for
  debugging, time-boxed with auto-off.
- **Audit log** (separate, append-only, exportable): logins, session events,
  settings changes, admin actions, API/MCP key usage, Assist calls, DLP
  verdicts, recalls, rule executions.
- Optional **OpenTelemetry** (OTLP) traces + metrics + Prometheus `/metrics`
  endpoint (auth-gated) — self-hosted observability first-class.

### 21.2 Error reporting (Sentry & open-source equivalents)

- Built-in `sentry`-SDK integration: point a DSN at **self-hosted Sentry,
  GlitchTip, or Bugsink** (all DSN-compatible) — or any future compatible
  sink. **Off by default**; enabling is an admin action with an in-UI
  disclosure to users of that deployment.
- Event scrubbing before send: no mail content, no addresses, no
  identifiers beyond an install-random ID; breadcrumbs allowlisted.
  Client-side (browser) errors are tunneled through the Mailwoman server
  (`/errors` endpoint) so CSP stays `connect-src 'self'` and the scrubber
  runs server-side too. This is operator-owned monitoring of their own
  instance — it never reports to the Mailwoman project (§1, no telemetry).

---

## 22. Plugin System

Two tiers, both capability-gated (unchanged in design from v0.2):

1. **Engine plugins — WASM (wasmtime), WASI p2.** Manifest-declared,
   admin-approved capabilities; hooks: auth flow, message in/out pipeline,
   address book sources, autoconfig sources, DLP detectors, spam actions.
2. **UI plugins — TypeScript.** Declarative extension points only (composer
   actions, message-view panels, settings pages, command-palette entries) in
   sandboxed frames with a postMessage API; **no DOM access to the host app**.

Signed plugin registry; unsigned requires admin `allow_unsigned` + permanent
banner.

**First-party plugins at 1.0:** Graph bridge, EWS bridge, Gmail API bridge
(§6.5) · Rspamd + SpamAssassin trainers (§10.8) · Nextcloud (§18.4) ·
LanguageTool (grammar without AI) · LDAP address book extras (§13) ·
theme/texture packs (§17).

---

## 23. Performance Targets (release gates, measured in CI)

| Metric | Target |
|---|---|
| Initial JS (gzipped, login→inbox critical path) | < 250 KB (calendar/tasks/notes lazy-loaded per module) |
| Cold load to interactive inbox (warm server, 4× CPU throttle) | < 1.5 s |
| Warm navigation between folders/modules | < 100 ms |
| Local search over 100k messages | p95 < 50 ms |
| Message list scroll | 60 fps at 100k messages |
| Open a 5 MB HTML monster email (sanitized) | < 300 ms |
| Calendar month view with 500 events | < 150 ms render |
| Server RSS (idle, 1 account) / (100 active sessions) | < 40 MB / < 512 MB |
| WebSocket push latency (server → UI) | < 100 ms intra-region |
| IMAP full initial sync 50k-message mailbox (headers) | < 60 s on LAN |
| Server binary size / container image | < 91 MB / < 205 MB [^size-budget] |

Budget regressions fail CI (Lighthouse CI + bench harness + `cargo bench`
trend tracking).

[^size-budget]: **Revised in 26.9 (measured, full-feature).** The original
    `< 45 MB` binary / `< 30 MB` image targets assumed a *core-only* build. The
    shipped 1.0/V7 server statically links the full feature set (wasmtime plugin
    JIT + every mail/PIM protocol + crypto + the embedded SPA), which measures
    **~79 MB stripped** on Linux *after* the first-party `.wasm` bridge/plugin
    components were externalized out of the binary and digest-pinned (26.9,
    t9-e5), and a **~178 MB** distroless runtime image. The budgets above are set
    at `ceil(measured × 1.15)` — ~15% regression headroom that still trips on a
    new heavy dependency (they are measured ceilings, not round-number padding).
    A future feature-gated **core SKU** (dropping the bundled bridges/plugins and
    wasmtime) plus a `FROM scratch`/musl-static image can re-approach the smaller
    original numbers. Full rationale + measurement:
    `docs/perf/size-budget-revision.md`.

---

## 24. Accessibility & Internationalization

- **WCAG 2.2 AA** as a release gate: full keyboard operability, visible focus,
  screen-reader tested flows, reduced-motion (textures/toasts respect it),
  high-contrast themes, touch target minimums. Calendar views get dedicated
  SR interaction patterns (grid navigation, event announcements).
- i18n via **Fluent**; RTL first-class (mirrored layouts incl. calendar,
  bidi-isolation for mixed-direction subjects — a spoofing vector);
  locale-aware dates/collation/week-starts; translation via Weblate; ship
  en/de/fr/es/pt-BR/nl/it/pl/ru/uk/zh/ja at 1.0.

---

## 25. Testing & Quality

- **Fuzzing:** cargo-fuzz targets for MIME, IMAP/POP3 wire, HTML sanitizer,
  vCard/iCal/.hol, MSG/OFT (CFB), PGP/CMS; corpus from real-world weird mail;
  OSS-Fuzz application once public.
- **Protocol conformance:** CI matrix vs Dovecot, Stalwart, Cyrus, Greenmail
  containers; recorded-quirk fixtures (Gmail `\All`, UIDPLUS absence, …).
- **Interop gates:** S/MIME ↔ Outlook; PGP ↔ Thunderbird/GnuPG/Proton Bridge;
  ICS ↔ Outlook/Google/Apple Calendar round-trips; MSG/OFT ↔ Outlook;
  HTML rendering screenshot suite over the top-50 email torture corpus.
- **E2E:** Playwright vs docker-composed full stack (Postgres + Redis +
  Dovecot/Stalwart), including offline/multi-window scenarios; Tauri E2E via
  WebDriver on all desktop OSes.
- **Security:** cargo-audit/deny per PR; ZAP baseline in CI; annual
  third-party audit funded before 1.0 (crypto + web app), published.
- Coverage floor 80% on protocol/crypto crates; mutation testing on
  `mw-crypto`, `mw-sanitize`, `mw-export`.

---

## 26. Repository Layout

```
mailwoman/
├─ Cargo.toml            # workspace
├─ crates/               # §4.3
├─ apps/web|desktop|mobile
├─ plugins/              # first-party WASM + UI plugins
├─ fonts/                # bundled font packs + puller manifests (§17.2)
├─ themes/               # built-in token packs incl. Grove textures (§17.1)
├─ docs/
│  ├─ deploy/            # nginx, apache, caddy, systemd, docker, k8s, panels
│  ├─ spec/              # this document, split per subsystem as it grows
│  └─ security/          # threat model, disclosure policy, audit reports
├─ fixtures/             # email torture corpus, protocol recordings, ICS/MSG suites
├─ xtask/                # cargo xtask: codegen (TS types), release, bench
├─ license.md            # MIT
├─ SECURITY.md
└─ CONTRIBUTING.md       # DCO, style, fuzzing guide
```

---

## 27. Roadmap

**Capacity model:** one person driving, heavy AI-agent-assisted
implementation. Strictly sequential vertical slices; every milestone ends in
a tagged, releasable, daily-drivable artifact; no calendar estimates; JMAP
before IMAP (the walking skeleton ships on the easy protocol). Fixture-based
suites, recorded protocol sessions, fuzz targets, and codegen'd types exist
partly so implementation can be delegated to agents and verified mechanically.

| Milestone | Scope | Exit criteria (each = a public tagged release) |
|---|---|---|
| **V0 — Walking skeleton** | Workspace + CI, `mw-jmap`, `mw-server` (Postgres+SQLite via sqlx), `mw-sanitize`, sandbox process split, minimal UI: login, list, read, compose, send — JMAP upstream (Stalwart) only; Let's Encrypt; hardened container/systemd files | Daily-drivable vs Stalwart; sanitizer passes torture corpus; perf budgets in CI from day one |
| **V1 — IMAP + POP3 adapters** | `mw-imap`, `mw-pop3`, `mw-mime`, `mw-store` full schema, sync engine, threading, autoconfig, seccomp/Landlock depth | Daily-drivable vs Dovecot, Gmail (IMAP+XOAUTH2), and a POP3-only host; fallback chains proven against fixtures |
| **V2 — Modern mail layer** | Search (Tantivy), offline (SW+OPFS), WebSocket push, connection toasts, outbox, undo send, send later, snooze, sweep, follow-ups, pins/tags/colors/search folders, Sieve GUI + rules, unified inbox, focused inbox (rules-based), multi-window + sub-tabs, import/export (EML/mbox/PDF-print/TXT/MD), signatures & identities incl. server-pulled froms, Grove themes + font puller | Feature-parity checkpoint vs Gmail-web for daily mail; ZAP baseline green; offline + multi-window E2E green |
| **V3 — PIM** | Calendar (all views, events, attendees, invites, conflicts, sharing, ICS/.hol), Tasks (VTODO/My Day), Notes (encrypted), Contacts (CardDAV, lists/groups/favorites/business cards), `mw-dav`, `mw-ics` | Invite + counter-proposal round-trips vs Google/Fastmail/Stalwart/Nextcloud; conflict resolver E2E; .hol/.ics fixture suites green |
| **V4 — Crypto & security depth** | OpenPGP (client-side WASM), Autocrypt, WKD, S/MIME, verdict UI, Security panel (metadata/signature analysis), DLP, max-security opening, message classification | Thunderbird/GnuPG/Outlook interop suites green; DLP rule engine audited |
| **V5 — Thin shells** | Tauri desktop + mobile thin clients, self-contained desktop mode, UnifiedPush/APNs, capture protection, keychain, mailto/share/file handlers | Signed installers < 10 MB; store-ready builds; shell integrity verification E2E |
| **V6 — Zero-access + Admin + API/MCP + Plugins** | Zero-access mode + device pairing, PQC store wrapping, WASM plugin runtime, full admin panel, scoped API keys, webhooks, **MCP server**, LDAP/GAL, password-change backends, Redis cache layer, observability (OTLP/Sentry-compat) | External security audit (incl. MCP surface) passed; cache scope matrix enforced in tests |
| **V7 — Bridges + Assist + Outlook parity tail** | Graph, EWS, Gmail API bridges; recall/reactions/voting/focused-sync via bridges; MSG/OFT/DOCX export; Nextcloud plugin; **Assist subsystem** (all capabilities, scoping, dictation, recaps, AI search/auto-tag) | Bridge fixture suites green; nightly live-interop vs M365/Workspace tenants; Assist works vs Ollama + OpenAI-compatible + Anthropic endpoints with scope audit E2E |
| **1.0** | Hardening, WCAG audit, docs, translations | All release gates green; audit findings resolved |

Scope-cut ladder if reality bites (cut from the bottom up, never quality):
**Gmail API bridge (IMAP covers most users) → mobile store presence
(sideload/F-Droid floor) → zero-access webmail mode (client-side E2EE stays) →
Assist auto-modes (suggest-only stays).** Exchange support (Graph + EWS) is
a commitment and survives cuts; POP3 is guaranteed and ships in V1.

---

## 28. Open Questions

1. ~~EWS/Graph bridge priority~~ — **Resolved:** Graph + EWS + Gmail API all
   ship at 1.0 (§6.5); Exchange support is cut-proof, Gmail API is the first
   cut candidate (§27).
2. **Optional "Mailwoman Server" companion** (bundled Stalwart config / LMTP
   sidecar for snooze-without-keywords) — post-1.0 evaluation.
3. **iOS background sync limits** — how aggressive can mobile sync be without
   APNs wakes for F-Droid-style purists? Spike in V5.
4. **Masked email / alias services** (SimpleLogin, addy.io, Fastmail masked)
   integration as first-party plugin?
5. ~~Name collision check~~ — **Resolved:** ships as Mailwoman. Claim GitHub
   org, mailwoman.app/.org, and registry names before first public release
   (crates may need `mailwoman-*` prefixes given Pelias's npm package).
6. **OAuth app ownership & verification costs** — BYO app ID is the primary
   documented path (5-minute wizards); shared Mailwoman-owned client IDs
   (legal entity, MS publisher verification, Google CASA) only if sponsorship
   funds them.
7. **Apple/Google store presence & push relay** — Apple account is
   sponsorship-funded; the push relay must be self-hostable by design so no
   Mailwoman-operated infrastructure is load-bearing. Decide before V5;
   F-Droid/sideload is the guaranteed floor.
8. **MSG/OFT write fidelity** — MS-OXMSG is documented but deep; determine at
   V7 how much of rich content (embedded objects, custom named properties)
   export must preserve for the enterprise audience vs "faithful body +
   attachments + headers".
9. **Reactions/voting header conventions** — publish the Mailwoman header
   conventions as an internet-draft so other open clients can interoperate?
10. **Notes interop default** — Mailwoman-native encrypted notes are the
    primary; is IMAP-Notes-folder interop worth its zero-access tradeoff as a
    default-off toggle, or docs-only?
