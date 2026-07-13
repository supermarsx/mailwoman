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
