# Mailwoman security & crypto (V4)

V4 (release 26.5.0) makes Mailwoman a **verifiably end-to-end-encrypted,
security-transparent** client. Everything here is additive behind the existing
JMAP-shaped surface; nothing about the V1/V2/V3 mail + PIM behaviour changes.

The load-bearing architectural facts an operator or auditor should know:

- **Private keys never reach the server unencrypted.** OpenPGP and S/MIME
  private-key operations (key generation, decryption, private-key signing,
  PKCS#12 import, backup) run **client-side in a WebAssembly build of
  `mw-crypto`** inside a dedicated crypto Web Worker. The server only ever stores
  an opaque, client-encrypted `encryptedPrivateBackup` blob (for cross-device
  restore) and **public** keys/certs. See [`crypto.md`](./crypto.md).
- **Decrypted mail is sanitized in the browser.** End-to-end-encrypted plaintext
  is sanitized by a WASM build of `mw-sanitize` **in the crypto worker** before it
  reaches the sandboxed render iframe — it never round-trips to the server
  sanitizer (that would defeat the encryption). Cleartext mail keeps the
  server-side sanitize path unchanged.
- **Public verdicts are computed server-side.** DKIM/SPF/DMARC/ARC verdicts,
  signature *verification*, cert harvesting, the Received chain, and attachment
  risk are public operations with no secret input, so the engine computes them
  (`mail-auth`, Stalwart, Apache-2.0 OR MIT). The Security panel merges these with
  the client-side decrypt/verify results.

Documents in this directory:

- [`crypto.md`](./crypto.md) — OpenPGP + S/MIME key management, the client-side
  private-key model, WKD lookup/publishing, S/MIME PKCS#12 import, PQC posture.
- [`dlp.md`](./dlp.md) — the outbound data-loss-prevention pipeline and the
  `MW_DLP_RULES` configuration format.
- [`max-security.md`](./max-security.md) — the three-position message opening mode
  (plain-text / sanitized-no-media / full-sanitized) and its policy precedence.
- [`screen-capture.md`](./screen-capture.md) — the **honest** screen-capture
  posture: what the web watermark does and, more importantly, what it cannot do.

Operator-facing environment variables and HTTP endpoints for all of the above are
collected in [`../deploy/crypto-security.md`](../deploy/crypto-security.md).

## V6 (release 26.7.0) — zero-access, admin, API/MCP surface, observability

V6 adds a deployable, administrable, integration-ready server. Its security-relevant
surfaces are documented here:

- [`zero-access.md`](./zero-access.md) — the optional **zero-access storage mode**:
  the client-side key hierarchy, what the server can and cannot see, and the honest
  boundary (it protects data at rest, not a malicious active server).
- [`admin-panel.md`](./admin-panel.md) — the separate-session **admin panel**: enable/
  disable, the CLI + config mirror, the append-only audit log, login monitor + ban list.
- [`api-keys-oauth.md`](./api-keys-oauth.md) — **scoped API keys** (`mwk_` opaque,
  Argon2id-hashed, shown once) and the **OAuth 2.1** authorization server (PKCE +
  resource indicators + admin-approved client registry); the typed scope model.
- [`mcp.md`](./mcp.md) — the **MCP server** security model: per-tool scopes, the
  default-off human-in-the-loop **send gating**, and the **prompt-injection posture**
  (provenance labels, no raw protocol composition, least authority) with its honest
  boundary.
- [`observability.md`](./observability.md) — OTLP traces/metrics, the auth-gated
  Prometheus `/metrics`, the `/errors` scrubber, and the no-mail-content-in-telemetry
  rule.

Operator deployment guides for the V6 data layer are in
[`../deploy/postgres.md`](../deploy/postgres.md) (pluggable Postgres backend +
`migrate-store`) and [`../deploy/cache.md`](../deploy/cache.md) (Valkey/Redis cache
posture, the §15.6 scope matrix, and the zero-access exclusion).

## V7 (release 26.8.0) — plugin runtime, directory, password-change, Assist

V7 adds the security-sensitive surfaces documented here:

- [`plugins.md`](./plugins.md) — the **WASM engine-plugin runtime** (`mw-plugin`,
  wasmtime + WASI-p2): the plugin ABI, authoring, **Ed25519 signing**, the
  **capability model (deny by default)**, and the **resource limits**
  (memory/deadline/fuel). The host is the trust boundary.
- [`password-change.md`](./password-change.md) — in-app **password change**
  (`mw-passwd`): the backends (local / LDAP RFC 3062 / Dovecot HTTP / poppassd / HMAC
  webhook), sealed-credential re-seal, and the client-side zero-access re-wrap.
- [`../assist.md`](../assist.md) — **Assist (AI)** privacy and governance: BYO
  endpoint, the default exclusion of E2EE content + attachments, redaction, the
  **content-free audit**, send-always-human-gated, and the "what left the device"
  disclosure.

The read-only **LDAP/GAL directory** operator guide (S/MIME cert lookup feeds the
existing crypto path) is in [`../deploy/ldap.md`](../deploy/ldap.md). The V7 scope
boundaries — bridge PIM-seam, EWS Kerberos, and the quick-xml write-only advisory
ignore — are collected in [`../RELEASE-NOTES-26.8.md`](../RELEASE-NOTES-26.8.md).
