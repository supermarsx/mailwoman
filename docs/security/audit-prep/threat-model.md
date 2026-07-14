# Consolidated threat model (external-audit prep)

Milestone: 1.0 hardening (baseline release 26.8.0). This document consolidates the
threat model across Mailwoman's security-sensitive surfaces into one place for an
external auditor. Each surface is described as: **assets**, **trust boundaries**,
**adversary model**, **existing mitigations**, and **residual risk**. The per-subsystem
docs under [`docs/security/`](../) remain authoritative for mechanism detail; this is the
cross-cutting view.

> **HUMAN-GATED AUDIT.** The funded external audit **run** and the **resolution of its
> findings** are a hard, human-gated condition of the actual 1.0 tag (SPEC §25 / §27).
> This document *prepares* that engagement; it does not substitute for it. Do not read
> "existing mitigations" as "audited and cleared" — they are self-assessed, pending
> independent review.

## System model in one paragraph

Mailwoman is a Rust workspace (engine + server + client crypto) plus a SolidJS web app,
optionally wrapped in Tauri desktop/mobile shells. The server proxies a JMAP-shaped
surface to upstream mail (IMAP/SMTP/POP3) and PIM (CalDAV/CardDAV) providers, or serves a
native store. The three defining security stances are: (1) **private keys and E2EE
plaintext are handled only client-side in a WASM crypto worker**; (2) **untrusted input
(mail bodies, MIME, plugin code, LLM-facing content) is treated as hostile and confined**;
(3) **the supply-chain floor is permissive-license, pure-Rust, rustls-only, no openssl/C
TLS** (enforced by `cargo deny`, see [`deny.toml`](../../../deny.toml)).

## Global trust boundaries

| # | Boundary | Untrusted side | Trusted side |
|---|---|---|---|
| B1 | Network → server | Internet clients, upstream mail/PIM providers | `mw-server` request handlers |
| B2 | Server ↔ browser | The page/DOM, rendered mail | The engine/JMAP surface |
| B3 | Browser main thread ↔ crypto worker | Main-thread JS, rendered content | The WASM `mw-crypto` / `mw-sanitize` worker (holds keys/plaintext) |
| B4 | Host ↔ WASM plugin guest | Third-party plugin bytes | The `mw-plugin` wasmtime host |
| B5 | Engine ↔ external AI endpoint | The AI provider (and its responses) | The `mw-assist` gateway |
| B6 | Server ↔ at-rest store | A curious operator / breached DB / stolen backup | The client that holds the keys |
| B7 | MCP/API caller → engine | Automation clients, LLM agents | Scoped-key enforcement + engine surface |

A recurring principle: **secret material and E2EE plaintext live on the trusted side of
B3; the server is treated as honest-but-curious for data at rest (B6) but is NOT assumed
to defend against a fully malicious active proxy** — that boundary is stated plainly
below and in [`zero-access.md`](../zero-access.md).

---

## 1. Client-side crypto (OpenPGP / S/MIME / PQC)

Authoritative: [`crypto.md`](../crypto.md).

**Assets.** OpenPGP + S/MIME private keys; passphrases; decrypted mail plaintext; the
S/MIME PKCS#12 import material; contact-key trust state (TOFU records).

**Trust boundaries.** B3 (main thread ↔ crypto worker) and B2 (server ↔ browser). Private
keys and passphrases never cross B3 outward and never cross B2 at all in plaintext.

**Adversary model.**
- A curious/breached **server** wanting to read mail or steal keys.
- A **malicious mail sender** shipping a crafted OpenPGP/S/MIME message (parser attack,
  signature-confusion, key-substitution).
- A **network MITM** on WKD key lookup.
- **Malicious main-thread JS** (e.g. via a rendering escape) attempting to exfiltrate a
  key handle.

**Existing mitigations.**
- Private-key ops (`generateKey`, decrypt, private-key sign, `importPkcs12`, backup) run
  **only in the WASM crypto worker**; the server stores public keys plus an opaque,
  client-encrypted `encryptedPrivateBackup` it cannot decrypt (asserted by an engine
  test — no plaintext-private field exists in the wire schema or store).
- Decrypted plaintext is sanitized **in-worker** by WASM `mw-sanitize` and rendered in a
  no-scripts / no-same-origin sandboxed iframe; it never round-trips to the server
  sanitizer.
- OpenPGP over **rPGP** (`pgp`, MIT/Apache — deliberately not LGPL `sequoia-openpgp`);
  S/MIME over the RustCrypto stack (`cms`/`x509-cert`/`rsa`/`p256`). Pure Rust, compiled
  to WASM.
- Key handles are cached in worker memory only and `zeroize`d on lock/timeout.
- Contact keys are **TOFU** with a `keyChanged` alert on fingerprint change; explicit
  `verified` (safe-words/QR) and `revoked` promotion.
- Signature verdicts are the frozen 3-state contract (`verified` / `unverified-key` /
  `invalid`, plus `none`).

**Residual risk (auditor focus).**
- **rPGP / RustCrypto parser hardening** against malformed packets is exactly the kind of
  memory-safety/logic surface an external audit should fuzz (Rust bounds the memory-safety
  class, not the logic class).
- **RSA Marvin timing side-channel** (RUSTSEC-2023-0071) is an accepted, bounded ignore:
  S/MIME RSA *decrypt* is client-side/local, no network timing oracle. See
  [`surface-inventory.md`](./surface-inventory.md#supply-chain-bounded-ignores). An
  auditor should confirm the boundary holds (no server-side RSA decrypt path).
- **PQC is groundwork, not a user claim.** The committed PQC deliverable is a hybrid
  X25519 + ML-KEM-768 key-wrap of the `mw-store` seal key at rest; **`ml-kem` is
  unaudited** and OpenPGP-PQC is behind an off-by-default `pqc` feature. Do not audit it
  as a shipped E2EE guarantee.
- **WKD MITM** is bounded by TLS + user consent + TOFU, not eliminated.
- A rendering escape on the main thread that reaches a live worker handle is the highest-
  value target on B3.

---

## 2. Zero-access at-rest storage & the key hierarchy

Authoritative: [`zero-access.md`](../zero-access.md).

**Assets.** The root key (Argon2id-derived or WebAuthn-PRF); the KEK and per-class data
keys (message-cache, search-index, notes, attachment-cache); the recovery phrase; stored
ciphertext.

**Trust boundaries.** B6 (server ↔ at-rest store) and B3. Keys are derived and held on the
client; the server never receives a plaintext key.

**Adversary model.** A **curious operator** or a **breach of the storage host** (reads the
DB, on-disk files, or a stolen backup). Explicitly **out of model:** a fully malicious
*active* server that proxies live IMAP/SMTP.

**Existing mitigations.**
- Root key derived client-side (Argon2id / WebAuthn-PRF) inside the WASM worker; KEK wraps
  per-account data keys; per-class keys via a domain-separated SHA-256 KDF. Only wrapped
  keys + ciphertext persist server-side.
- Rows are **XChaCha20-Poly1305** ciphertext framed `nonce(24) ‖ ct+tag`, authenticated
  with **AAD = `table ‖ 0x1F ‖ row_id ‖ 0x1F ‖ schema_version`** — decryption fails if a
  row is relocated, re-labelled, or read under a different schema version (anti-replay /
  anti-relocation).
- Caching of plaintext-derived data is **disabled** for zero-access accounts (SPEC §15.6).
- **Search runs entirely client-side** (browser-built Tantivy slice in OPFS, encrypted at
  rest under the search-index key); **no server-side searchable-encryption claim**.
- Multi-device pairing relays only an **opaque sealed envelope** (P-256 ECDH) through the
  server, authenticated by a 6-word SAS the user compares on both screens (defeats a MITM
  relay).

**Residual risk (auditor focus).**
- **The honest boundary:** zero-access protects **data at rest**, not against a malicious
  active server on the live mail path — that adversary can observe/tamper with mail as it
  flows regardless of at-rest encryption. The UI states this at enable time; an auditor
  should confirm the claim is not overstated anywhere in product copy.
- **Metadata leakage is by necessity:** ciphertext blobs, opaque row IDs, sizes,
  timestamps, and envelope routing metadata remain visible to the server. This is
  documented, not hidden — verify no additional plaintext leaks.
- **Argon2id parameters** and the SAS transcript construction are worth an independent
  review (parameter choice, salt storage, SAS derivation from the full transcript).
- **Recovery-phrase handling** — anyone holding it derives every account key; loss with no
  paired device is unrecoverable by design.

---

## 3. WASM plugin sandbox (`mw-plugin`)

Authoritative: [`plugins.md`](../plugins.md).

**Assets.** The host process integrity; OAuth client secrets / refresh tokens (never given
to guests); other accounts' data; the network allowlist; host CPU/memory.

**Trust boundary.** B4 (host ↔ guest). The **host is the trust boundary**; guest bytes are
untrusted third-party code.

**Adversary model.** A **malicious or compromised plugin** attempting to: escape the
sandbox, reach the network beyond its allowlist, exhaust host resources, read secrets, or
act as an over-privileged backend.

**Existing mitigations.**
- **wasmtime + WASI-p2 component model**; `mw-plugin` is `#![forbid(unsafe_code)]` at its
  own boundary (the host mediates every capability; wasmtime carries its own audited
  `unsafe` internally).
- **Capability model, deny-by-default:** the manifest (`plugin.toml`) declares capabilities
  and a `net_allowlist`; **the host denies everything not declared.** An ungranted hook
  returns typed `CapabilityDenied` and is never invoked. There is **no ambient WASI
  authority** — no default filesystem, clock, RNG, or network.
- **Host-mediated I/O only:** a guest cannot open a socket; `http-fetch` is host-enforced
  against `net_allowlist` (empty ⇒ no outbound network at all). OAuth tokens are injected
  by the host (`oauth-token`); the guest never sees client secrets or refresh tokens.
- **Resource limits:** `memory_mb` ceiling (wasmtime `ResourceLimiter`), `deadline_ms`
  wall-clock via epoch-interruption, optional deterministic `fuel`. Any trip returns typed
  `PluginError::LimitExceeded` — never a panic or host crash. Instances recycled per
  session.
- **Ed25519 signing:** the component bytes carry a detached Ed25519 signature verified
  against a configured trust root; an unsigned load requires explicit `allow_unsigned` and
  raises a permanent unsigned banner + audit record.
- The `jail` CI job proves capability + resource-limit enforcement against the real
  LanguageTool component (see [`self-baseline.md`](./self-baseline.md#plugin-jail)).

**Residual risk (auditor focus).**
- **Sandbox escape via wasmtime** is the crown-jewel target: audit the host imports for
  any confused-deputy path where a guest could influence which account/token the host
  acts on.
- **`net_allowlist` enforcement** is DNS/host-based — audit for rebinding or redirect
  following that could reach an off-allowlist host.
- **`allow_unsigned`** is an operator footgun; confirm the banner + audit always fire.
- **Scope boundary (honest):** only the **engine (WASM) plugin tier** exists; the
  declarative **TypeScript UI-plugin tier (§22.2) is not implemented** (document-only,
  tracked post-1.0). The WIT exports the **account-backend (mail)** interface;
  calendar/tasks/reactions are fixture-tested but **not yet drivable through the plugin
  seam** (post-1.0 WIT extension). Nothing to audit there yet — but do not assume it is
  sandboxed, because it does not exist.

---

## 4. MCP server

Authoritative: [`mcp.md`](../mcp.md).

**Assets.** The mailbox (read + send); PIM data; the send capability specifically.

**Trust boundary.** B7 (MCP caller → engine) and, critically, the **content boundary**:
mail bodies returned to an agent are attacker-controlled text.

**Adversary model.** (a) An **over-privileged or leaked MCP key**; (b) **prompt injection**
— a mail sender embedding instructions aimed at the agent reading the mailbox; (c) an
attempt to **compose raw protocol** (IMAP/SMTP) through a tool.

**Existing mitigations.**
- **Every tool goes through the engine/JMAP surface — never raw IMAP/SMTP.** A malicious
  mail body cannot smuggle a protocol command through a tool.
- **MCP keys are API keys:** same scoping/expiry/IP-allowlist/rate-limit/audit as any
  `mwk_` key; the callable tool set is the key's `mcp_tools` scope (unnamed tool ⇒
  denied).
- **Send is disabled by default, human-in-the-loop when enabled:** no `send` scope ⇒
  denied; `send` without `unattended_send` (the default) ⇒ the message lands in the
  **Outbox** for in-app human confirmation and is **not** transmitted; `unattended_send`
  requires an admin countersignature on the key or it is **403**. *Status (26.7): the
  countersign resolver is not yet wired, so every `mail.send` currently lands in the
  Outbox or is refused — the safe default.*
- **Prompt-injection posture:** every mail-derived result is wrapped in an `untrusted:…`
  provenance envelope; tool schemas declare mail bodies untrusted in-band; least authority
  means an injected instruction can only do what the key was already granted.

**Residual risk (auditor focus).**
- **The honest boundary:** provenance labels + least authority **reduce, not eliminate**,
  prompt injection. A client that ignores the untrusted labels, or a key over-granted with
  `unattended_send`, can still be steered by hostile mail. The defense is the **scope
  granted** plus a client that honours provenance.
- Audit the **scope-superset matrix** (`Scope::allows`) for any grant/deny gap.
- Confirm the **transmit path is genuinely unreachable** without a countersigned key (the
  safety test asserts this).

---

## 5. Assist (AI) gateway

Authoritative: [`assist.md`](../../assist.md).

**Assets.** Mail content that could be sent to an AI endpoint; E2EE-decrypted content;
attachments; the record of what left the device.

**Trust boundary.** B5 (engine ↔ external AI endpoint) and B2 (the browser never contacts
the AI host — the server proxies; CSP keeps `connect-src 'self'`).

**Adversary model.** (a) **Data exfiltration** — sensitive/E2EE content leaking to a
third-party endpoint; (b) a **compromised or hostile AI endpoint**; (c) **Assist being
used to send/act** irreversibly; (d) **audit leaking content**.

**Existing mitigations.**
- **Off until configured** — no endpoint ⇒ `Disabled`, zero Assist UI, nothing sent.
- **The engine is the only client** — the browser never talks to the AI host; the server
  proxies. `LocalProcess` keeps everything on-device.
- **E2EE-decrypted content and attachments are excluded by default** (`include_e2ee=false`,
  `include_attachments=false`); including them is explicit opt-in.
- **Redaction runs before anything leaves the engine**; per-capability grant; data-class
  ceiling; rate-limit.
- **Send is always human-gated — structurally:** the capability enum has **no
  send/delete/accept variant**, so there is no code path for Assist to transmit or act
  irreversibly. The assistant chat reuses the **same tool surface as MCP** and inherits its
  send-gating (drafts go to the Outbox).
- **Content-free audit:** each invocation records capability + scope summary + endpoint
  host, **never** content (asserted in tests).
- **"What left the device" disclosure** per Assist action; admin governance (endpoint
  allowlist, per-capability locks, data-class ceilings, kill switch).

**Residual risk (auditor focus).**
- **Redaction completeness** is a best-effort content filter, not a guarantee — audit the
  redactor against realistic PII/secret patterns.
- A **hostile AI endpoint** sees whatever the data-class ceiling + opt-ins allow; the
  disclosure surface should match actual gateway behaviour (verify no silent broadening).
- Confirm the **no-send structural claim** holds across the shared MCP tool surface (no
  Assist path reaches a transmit).

---

## 6. OAuth 2.1 AS + scoped API keys

Authoritative: [`api-keys-oauth.md`](../api-keys-oauth.md).

**Assets.** Access/refresh tokens; API keys; the scope-grant matrix; the client registry.

**Trust boundary.** B7 and B1.

**Adversary model.** Token/key theft; scope escalation; a rogue OAuth client; replay
across resources; brute-force of a key.

**Existing mitigations.**
- **Typed scope model, no implicit escalation:** verbs (`read`/`send`/`delete`), resource
  (per-account/folder or `*`, mail vs PIM), bounds (IP allowlist, expiry, per-key rate
  limit), `mcp_tools`, and the distinct dangerous `unattended_send`. `Scope::allows`
  requires the key's scope to be a **superset** of what the operation needs.
- **API keys:** `mwk_<prefix>.<secret>`, 256-bit secret, **Argon2id-hashed at rest**, shown
  once, individually revocable, per-use audit (prefix + action + source IP).
- **OAuth 2.1:** authorization-code + **mandatory PKCE (S256)**; **resource indicators
  (RFC 8707)** so a token is bound to its resource; **admin-approved client registry** (no
  open dynamic registration); **opaque, hashed tokens** (no JWT dependency by default);
  `/oauth/introspect` + `/oauth/revoke`.

**Residual risk (auditor focus).**
- **Honest status note (26.7):** scoped-key enforcement for the REST convenience layer
  (`/api/v1`) is wired as `Send`-safe middleware; where a capability is not yet enforced
  end-to-end it is tracked in orchestration state + the live-E2E gate. The doc describes
  the model and intended enforcement and **does not claim guarantees the code does not yet
  make** — an auditor should verify enforcement coverage empirically, not from the model.
- **No OIDC/SAML SSO** — never built; a documented 1.0 gap / deferred decision, not a
  vulnerability, but note its absence.
- Audit token lifetime/rotation and the consent-flow CSRF/redirect handling.

---

## 7. DLP (outbound data-loss prevention)

Authoritative: [`dlp.md`](../dlp.md).

**Assets.** Outbound message content; the audit trail.

**Adversary model.** Accidental/insider exfiltration of PAN/IBAN/SSN/national-ID; and the
audit itself leaking the very content it flags.

**Existing mitigations.** Engine-side outbound rules on `EmailSubmission/set` (+ a
`Dlp/scan` compose dry-run); Luhn/mod-97-validated detectors; actions `warn` / `block` /
`require-encryption` / `notify-admin`; **the matched content is never stored in the audit**
(redaction via `mw-store::redact.rs`, asserted by test).

**Residual risk.** DLP is **advisory/best-effort**, config-sourced, and a determined
insider can evade content detectors; it is not a confidentiality control. Regex-based
custom detectors carry ReDoS risk worth a look.

---

## 8. Supporting surfaces (brief)

- **Observability** ([`observability.md`](../observability.md)) — OTLP traces/metrics,
  auth-gated Prometheus `/metrics`, a `/errors` scrubber, and a **no-mail-content-in-
  telemetry** rule. Residual: verify no content/PII escapes into spans, metric labels, or
  error payloads.
- **Password-change** ([`password-change.md`](../password-change.md)) — `mw-passwd`
  backends (local / LDAP RFC 3062 / Dovecot HTTP / poppassd / HMAC webhook), sealed-
  credential re-seal, and the client-side zero-access re-wrap. Residual: credential
  re-seal correctness and backend transport (rustls/TLS) verification.
- **Admin panel** ([`admin-panel.md`](../admin-panel.md)) — separate session, append-only
  audit log, login monitor + ban list. Residual: privilege-separation between the admin
  session and the mailbox session.
- **Message rendering** ([`max-security.md`](../max-security.md),
  [`screen-capture.md`](../screen-capture.md)) — the 3-position opening mode
  (plain / sanitized-no-media / full-sanitized) and the **honest** screen-capture posture
  (the web watermark's real, limited value). Residual: HTML-sanitizer bypass is a classic
  high-value target (see the inventory).

---

## Cross-cutting residual-risk summary for the auditor

1. **Parsers of untrusted input** (MIME, HTML sanitize, CFB/.msg, LDAP, SOAP/EWS, JMAP,
   OpenPGP/S-MIME packets) — the primary fuzz/hardening targets. Enumerated in
   [`surface-inventory.md`](./surface-inventory.md).
2. **The four sandboxes/boundaries** (crypto worker B3, plugin host B4, Assist gateway B5,
   zero-access at-rest B6) — confirm no leak across each, and that the *honest boundaries*
   (active-server, prompt-injection, unaudited ml-kem, unimplemented UI-plugin tier) are
   not overstated in product copy.
3. **Authorization coverage** — verify scoped-key/OAuth enforcement empirically, given the
   honest 26.7 status note.
4. **Supply chain** — the bounded advisory ignores (RSA-Marvin, quick-xml write-only DoS,
   Tauri unmaintained) are documented in `deny.toml`; confirm each boundary claim.

Again: **the funded external audit run + findings resolution are human-gated and required
before the actual 1.0 tag.** This model is the input to that engagement.
