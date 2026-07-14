# Security-surface inventory (external-audit prep)

A structured enumeration of Mailwoman's attack surface for an external auditor: network-
reachable endpoints, trust-boundary crossings, every place untrusted input is parsed, the
crypto primitives and their crates, the sandbox boundary, and the `deny.toml`
bounded-ignore rationale. Pair with [`threat-model.md`](./threat-model.md) (the per-surface
adversary analysis) and [`self-baseline.md`](./self-baseline.md) (how to reproduce the
existing baseline).

> **HUMAN-GATED AUDIT.** This inventory *scopes* the funded external audit; it does not
> perform it. The audit run + findings resolution are a hard, human-gated condition of the
> real 1.0 tag (SPEC §25/§27). Baseline: release 26.8.0.

## 1. Network-reachable endpoints (`mw-server`)

All served by `mw-server` behind rustls TLS (no openssl). Authentication column notes the
gate; every scoped path resolves to the typed `Scope` (see
[`api-keys-oauth.md`](../api-keys-oauth.md)).

| Surface | Path(s) | Auth | Untrusted input |
|---|---|---|---|
| Health | `/healthz` | none | none (liveness) |
| JMAP-shaped mailbox API | the SPA's JMAP surface | session cookie | request bodies, upstream mail |
| REST convenience layer | `/api/v1/**` | scoped `mwk_` key / OAuth token | request bodies |
| API-key management | `GET/POST /api/keys`, `POST /api/keys/{prefix}/revoke` | session | request bodies |
| OAuth 2.1 AS | `/oauth/consent`, `/oauth/decision`, `/oauth/token`, `/oauth/introspect`, `/oauth/revoke` | PKCE flow / client creds | auth params, redirect URIs, code/verifier |
| MCP server | `/mcp` (Streamable HTTP) + `mailwoman mcp-stdio` | scoped MCP key / OAuth token | tool args, **mail bodies as untrusted content** |
| Assist gateway | server-proxied to the BYO AI endpoint | session + capability | AI endpoint responses |
| Admin panel | admin routes, `GET /admin/api-keys`, `POST /admin/api-keys/{id}/revoke`, oversight | **separate** admin session | admin inputs |
| DLP config read-back | `GET /api/security/dlp/config` | session | none (read) |
| Security panel | server-computed DKIM/SPF/DMARC/ARC verdicts merge | session | upstream mail headers |
| WKD publishing | `/.well-known/openpgpkey/...` (when `MW_WKD_DIR` set) | none (public keys) | request path |
| Observability | Prometheus `/metrics` (**auth-gated**), OTLP export (outbound) | metrics auth | none inbound |
| Static SPA | `MW_WEB_DIR` assets | none | none |

**Outbound connections the server initiates** (each a trust-boundary crossing): upstream
IMAP/SMTP/POP3 (proxy mode), CalDAV/CardDAV, LDAP/GAL directory, the BYO AI endpoint
(Assist), plugin `http-fetch` (host-mediated, allowlisted), OTLP collector, WKD lookup,
push relay. **All TLS is rustls; openssl is banned** (`deny.toml [bans]`).

## 2. Trust-boundary crossings

The seven boundaries B1–B7 are defined in [`threat-model.md`](./threat-model.md#global-trust-boundaries).
The load-bearing ones for an auditor:

- **B3 — main thread ↔ crypto worker.** Private keys, passphrases, and E2EE plaintext live
  only on the worker side; nothing crosses outward in plaintext. Handles are opaque session
  refs, zeroized on lock. *Highest-value confidentiality boundary.*
- **B4 — plugin host ↔ WASM guest.** The wasmtime host mediates 100% of guest authority;
  no ambient WASI. *Highest-value integrity/isolation boundary.*
- **B5 — Assist gateway ↔ AI endpoint.** The only place mail content can leave to a third
  party; default-excludes E2EE + attachments; redaction before egress.
- **B6 — server ↔ at-rest store.** Honest-but-curious for data at rest; **not** trusted to
  defend against a malicious active proxy.
- **B7 — automation caller → engine.** Scoped-key/OAuth enforcement; MCP/Assist tools go
  through the engine surface, never raw protocol.

## 3. Untrusted-input parsers (the fuzz/hardening targets)

Every parser below consumes attacker-influenced bytes. These are the primary targets for
an external audit's fuzzing + manual review. Rust bounds the memory-safety class; the
**logic** class (confusion, smuggling, resource exhaustion) still needs review.

| Input | Where parsed | Crate(s) | Notes / boundary |
|---|---|---|---|
| **MIME** (headers, multipart, encodings) | engine mail pipeline | Stalwart mail-parser stack | classic malformed-MIME + header-injection surface |
| **HTML sanitize** (mail bodies) | `mw-sanitize` — **server-side for cleartext, WASM in-worker for E2EE plaintext** | `mw-sanitize` (built native + wasm) | sanitizer bypass → the rendered iframe (no-scripts/no-same-origin sandbox is the backstop) |
| **OpenPGP packets** | crypto worker (WASM) | rPGP (`pgp`) | malformed-packet / signature-confusion |
| **S/MIME (CMS, X.509, PKCS#12)** | crypto worker (WASM) | `cms`, `x509-cert`, `rsa`, `p256` | cert-chain + ASN.1 parsing; PKCS#12 import |
| **CFB / MS-OXMSG (`.msg`/`.oft`)** | `mw-export` (client-side export) | `cfb` + own OXMSG layer | a **CFB-parse fuzz target** exists (§25); write path primarily |
| **DOCX** | `mw-export` — **write-only** | `docx-rs` (pins `quick-xml` 0.36.2) | reader path unreachable — see bounded ignores below |
| **iCalendar / vCard** | PIM engine | `icalendar`, `vcard4` | untrusted PIM payloads |
| **RRULE** | calendar | `rrule` | recurrence-expansion resource use |
| **LDAP / GAL responses** | `mw-directory` | `ldap3` (**`tls-rustls`, no native-tls**) | directory responses, cert/photo attrs |
| **SOAP / EWS XML** | `bridge-ews` | `quick-xml` 0.41 (fixed) | EWS response parsing |
| **JMAP request bodies** | engine surface | serde/JSON | the primary API input surface |
| **JSON-RPC / MCP tool args** | `/mcp` | hand-rolled over axum | tool arguments + provenance-wrapped results |
| **OCS / WebDAV (Nextcloud)** | `plugins/nextcloud` | hand-rolled JSON + `quick-xml` | third-party server responses |
| **AI endpoint responses** | `mw-assist` | hand-rolled JSON over `reqwest` | responses from a possibly-hostile endpoint |
| **Plugin component bytes** | `mw-plugin` | wasmtime / wasmparser | untrusted third-party WASM (see §5) |
| **Mail auth headers** (DKIM/SPF/DMARC/ARC) | engine (public verdicts) | `mail-auth` | header parsing; verdicts are public (no secret input) |

## 4. Crypto primitives + crates

Authoritative mechanism detail: [`crypto.md`](../crypto.md),
[`zero-access.md`](../zero-access.md). All pure-Rust, MIT/Apache/BSD/ISC, compiled to both
native and `wasm32`.

| Purpose | Primitive | Crate |
|---|---|---|
| OpenPGP | RFC 4880 suite | rPGP (`pgp`) — **not** `sequoia-openpgp` (LGPL, banned) |
| S/MIME | CMS / X.509 | `cms`, `x509-cert` |
| RSA | RSA-2048+ (S/MIME) | `rsa` (**RUSTSEC-2023-0071 bounded ignore**, §6) |
| ECC | P-256 (ECDSA / ECDH) | `p256` |
| At-rest AEAD | XChaCha20-Poly1305 | (RustCrypto AEAD) — `nonce(24) ‖ ct+tag`, AAD-bound |
| KDF (root key) | Argon2id | `argon2` |
| KDF (per-class) | domain-separated SHA-256 | `sha2` |
| Hashing | SHA-2 / SHA-1 (legacy verify) | `sha2`, `sha1` |
| Key/password hash at rest | Argon2id | `argon2` (API keys, admin) |
| Webhook MAC | HMAC-SHA256 | `hmac` + `sha2` |
| Plugin signing | Ed25519 (detached) | `ed25519-dalek` (via `mw-crypto`) |
| PQC (groundwork) | hybrid X25519 + **ML-KEM-768** key-wrap of the store seal key | `ml-kem` — **UNAUDITED**, not a user-facing E2EE claim; OpenPGP-PQC behind off-by-default `pqc` feature |
| Memory hygiene | zeroize-on-lock | `zeroize` |
| TLS | rustls throughout | **openssl banned** (`deny.toml [bans]`) |

**TLS hybrid** (X25519MLKEM768) is **not enabled** — the tree ships the `ring` provider;
enabling `aws-lc-rs` needs a license decision (OpenSSL-derived components) recorded in
`deny.toml` first. Not a shipped surface.

## 5. The sandbox boundary (WASM plugin host)

Authoritative: [`plugins.md`](../plugins.md). The precise boundary an auditor tests:

- **Host:** `mw-plugin`, `#![forbid(unsafe_code)]` at its own boundary; wasmtime + WASI-p2
  component model. The host is the trust boundary.
- **Guest authority is exactly the declared host imports — nothing else:** `http-fetch`
  (allowlisted, host-enforced), `oauth-token` (host injects; guest never sees
  secrets/refresh tokens), `kv-get`/`kv-put` (scoped namespace), `log` (no-content floor),
  `now`, `random`. **No ambient WASI filesystem/clock/RNG/socket.**
- **Deny-by-default capabilities** declared in `plugin.toml`; ungranted hook ⇒ typed
  `CapabilityDenied`, never invoked. `net_allowlist` empty ⇒ zero outbound network.
- **Resource limits:** `memory_mb` (ResourceLimiter), `deadline_ms` (epoch-interruption),
  optional `fuel`; any trip ⇒ typed `PluginError::LimitExceeded`, no panic/host crash;
  instances recycled per session.
- **Ed25519 signature** over component bytes vs a configured trust root; unsigned load
  needs explicit `allow_unsigned` + permanent banner + audit.
- **Proven by the `jail` CI job** against the real LanguageTool component.
- **Scope boundary (honest):** engine (WASM) tier only; the **TypeScript UI-plugin tier is
  unimplemented** (document-only, post-1.0); the WIT exports **account-backend (mail)** —
  calendar/tasks/reactions are fixture-tested but **not seam-wired** (post-1.0).

## 6. Supply-chain posture & the `deny.toml` bounded ignores

Authoritative source: [`deny.toml`](../../../deny.toml). Enforced in CI by the `deny` job
(`cargo deny check licenses advisories bans sources`) — see
[`self-baseline.md`](./self-baseline.md#supply-chain).

**Floor:** permissive-license only (MIT / Apache-2.0 / Apache-2.0-WITH-LLVM-exception /
BSD-2/3 / ISC / Zlib / MPL-2.0 / Unicode-3.0 / CDLA-Permissive-2.0 / CC0-1.0 /
bzip2-1.0.6). **GPL/LGPL/AGPL are denied by omission** (an unlisted license fails the
check). **Bans:** `openssl` (rustls only, SPEC §5.1) and `sequoia-openpgp` (LGPL).
`yanked = "deny"`, `unknown-registry`/`unknown-git = "deny"`.

**The bounded advisory ignores** — each is a documented, bounded acceptance with a stated
boundary an auditor should confirm; **all are `unmaintained`- or reader-DoS-class, not
network-reachable vulnerabilities**:

### RSA Marvin (RUSTSEC-2023-0071) {#supply-chain-bounded-ignores}
A timing side-channel in the pure-Rust `rsa` crate (pulled by `cms` for S/MIME). **No fixed
version exists** upstream. **Boundary:** Mailwoman performs S/MIME RSA *decryption*
**client-side in the browser WASM crypto worker** — local use on the user's own device,
precisely the scenario the advisory blesses ("local use on a non-compromised computer is
fine"). There is **no network-reachable server-side timing oracle**. *Auditor check:
confirm no server-side RSA-decrypt path exists.* Revisit when `rsa` ships constant-time
decrypt.

### quick-xml write-only DoS (RUSTSEC-2026-0194 / -0195)
Memory-exhaustion **DoS in the quick-xml reader** (quadratic duplicate-attribute check;
unbounded namespace-declaration allocation). `docx-rs` 0.4.20 hard-pins `quick-xml` 0.36.2.
**Boundary:** `docx-rs` is used for **DOCX writing only** (`mw-export`) — the vulnerable
**reader path is unreachable**, and it is a client-side export, not a network parser. Every
*direct* quick-xml consumer (`mw-ics`/`mw-dav`/`mw-carddav`/`mw-plugin`/`bridge-ews`) is on
the **fixed 0.41**. No permissive newer `docx-rs` lifts the pin. *Auditor check: confirm no
untrusted `.docx`/XML is parsed through `docx-rs`.* Revisit when `docx-rs` bumps quick-xml
≥ 0.37.

### Tauri v2 unmaintained transitive advisories
All **`unmaintained` maintenance-status advisories (zero `vulnerability`-class)** carried by
every Tauri v2 app: the gtk-rs GTK3 bindings (RUSTSEC-2024-0411…0420 — Linux WebKitGTK
backend only, irrelevant to Windows/macOS/mobile), `proc-macro-error` v1 (build-time only,
RUSTSEC-2024-0370), and `unic-*` v0.9 data tables (RUSTSEC-2025-0075/0080/0081/0098/0100).
**Boundary:** no fixed version exists (the ecosystem hasn't migrated GTK3→GTK4); the only
alternative to the ignore is dropping the desktop/mobile shells. Build-/desktop-only, no
server runtime surface. Revisit when Tauri adopts GTK4.

**Vet-before-enable (declared, not linked):** `aws-lc-rs` (only if TLS-hybrid is enabled —
OpenSSL-derived license decision required first), `jsonwebtoken`, `totp-rs`, `sentry`
(rustls-transport vetting required). None ship today.

**Networked CI services out of scope by design** (mere aggregation, never linked/shipped):
postgres:16, valkey:8, OpenLDAP, Radicale (GPLv3 — a *separate program* over the network,
not a dependency), Dovecot/Greenmail, the mock Assist endpoint.

## 7. Honest scope boundaries (do not audit as shipped)

Recorded here and in [`SECURITY.md`](../../../SECURITY.md) / orchestration state so an
auditor does not mistake groundwork for a guarantee:

- **Zero-access protects data at rest, not a malicious active proxying server.**
- **`ml-kem` is unaudited**; PQC is store-key-wrap groundwork, not a user E2EE claim; TLS
  hybrid is off.
- **Prompt injection is bounded, not solved** (provenance + least authority).
- **MCP unattended-send** is unreachable without an admin-countersigned key (26.7: the
  resolver isn't wired, so sends land in the Outbox — the safe default).
- **Scoped-key enforcement** for `/api/v1` is described as the model + intended enforcement;
  verify coverage empirically (honest 26.7 note).
- **TypeScript UI-plugin tier is unimplemented**; the plugin WIT exports mail only
  (calendar/tasks/reactions fixture-tested, not seam-wired).
- **EWS Kerberos** is a documented BYO gap (reverse-proxy auth); native is post-1.0.
- **Screen-capture watermark** is a deterrent with stated limits, not a DRM control.
- **DLP** is advisory/best-effort, not a confidentiality control.
- **No OIDC/SAML SSO** — never built (documented 1.0 gap / deferred decision).
