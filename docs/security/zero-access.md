# Zero-Access Storage Mode

Optional, per-deployment (admin) and per-account (user). When enabled, the hosting
server stores mail and PIM data **only as ciphertext it cannot read**. The keys are
derived and held on the client; the server never receives a plaintext key.

This document states exactly what that does and does not protect. It is deliberately
narrow: the guarantee is about **data at rest**, not about a server that has turned
actively malicious.

## What it protects

Zero-access defends the **contents of your stored mail, notes, and PIM data against a
curious operator or a breach of the storage host**. An attacker who reads the database,
the on-disk files, or a stolen backup sees XChaCha20-Poly1305 ciphertext and cannot
recover message bodies, subjects, attachment contents, note text, or the search index.

## What the server still sees

Even with zero-access enabled, the server observes, by necessity:

- **Ciphertext blobs** — the encrypted rows themselves.
- **Opaque row IDs** — the identifiers used to store and fetch rows.
- **Sizes** — the length of each ciphertext (approximate message/attachment size).
- **Timestamps** — when rows are written and updated.
- **Envelope routing metadata** needed to proxy IMAP/SMTP — the server still connects
  to your upstream mail provider on your behalf, so the connection metadata and the
  routing envelope required to send and receive mail pass through it. Where the upstream
  does not offer OAuth, the upstream credentials are sealed to the client session.

Zero-access does **not** hide this metadata, and it makes no attempt to.

## The boundary we do not cross (the honest caveat)

Zero-access protects **data at rest** against a curious or breached host. **A fully
malicious _active_ server that proxies your live IMAP/SMTP traffic is a stronger
adversary, and zero-access does not defend against it.** Such a server sits on the live
connection to your mail provider and could observe or tamper with mail as it flows
through, regardless of how the stored copy is encrypted. The user interface states this
same difference plainly at the point where the mode is enabled.

Choose zero-access when your threat model is *"I do not want the people running (or
breaching) the storage host to be able to read my stored mail."* It is not a defense
against an operator who actively subverts the live mail path.

## No searchable-encryption claim

There is **no server-side searchable encryption** here, and we make no such claim. Search
runs entirely on the client: the browser builds a Tantivy index slice over content it has
decrypted locally, stored in OPFS and encrypted at rest under the search-index key. The
server never holds a searchable form of your plaintext.

## Key hierarchy (SPEC §9.1)

```text
passphrase  |  WebAuthn-PRF secret (passwordless)
        │  Argon2id (client-side, in the WASM crypto worker)
        ▼
    Root Key   (never leaves the client)
        ├─► Key-Encryption Key (KEK) ──wraps──► per-account Data Keys
        │                                         ├─► message-cache key
        │                                         ├─► search-index key
        │                                         ├─► notes key
        │                                         └─► attachment-cache key
        └─► Recovery phrase (printable, optional — an explicit offline backup)
```

- The **root key** is derived on the client with Argon2id from a passphrase, or from a
  passkey's WebAuthn-PRF output (passwordless). The Argon2id parameters and salt are
  stored so any of the user's devices can re-derive the same root key.
- The **KEK** wraps each per-account **data key**; per-class keys (message cache, search,
  notes, attachments) are derived from the data key by a domain-separated SHA-256 KDF.
- Only wrapped keys and ciphertext are persisted server-side. No plaintext key is
  serialized out of the WASM worker; keys are addressed there by opaque session refs and
  zeroized on lock/logout.

## Encryption at rest (SPEC §9.3)

- Rows for zero-access accounts hold only **XChaCha20-Poly1305 ciphertext**, framed as
  `nonce(24) ‖ ciphertext+tag` — the same construction the server already uses to seal
  credentials at rest.
- Each row is authenticated with associated data (AAD) binding it to its location:

  ```text
  AAD = table ‖ 0x1F ‖ row_id ‖ 0x1F ‖ schema_version
  ```

  where `‖` is concatenation and `0x1F` is the ASCII unit separator. Decryption fails if
  a row is moved to a different table, re-labelled with a different id, or read under a
  different schema version — so ciphertext cannot be silently relocated or replayed.
- **Caching:** Redis/memory caching of plaintext-derived data is disabled for zero-access
  accounts (SPEC §15.6) — the cache never holds a decrypted form.

## Multi-device pairing (SPEC §9.1)

A new device obtains the root key from an existing one through a SAS-verified,
client-to-client exchange. **The server only relays an opaque sealed envelope; it never
sees a plaintext key.**

1. The **new device** generates an ephemeral P-256 key pair and shows its public point in
   a QR code. The secret stays on the new device.
2. The **existing device** (which holds the root key) scans the QR, performs P-256 ECDH
   with its own ephemeral key, and seals the root key into an envelope
   (`ephemeral_public ‖ nonce ‖ ciphertext`). Both devices derive the same short
   authentication string (SAS) — six words — from the full transcript.
3. The envelope is relayed **through the server as ciphertext** to the new device, which
   performs the matching ECDH and recovers the root key.
4. The user **compares the six SAS words on both screens**. A match authenticates the
   channel and defeats a machine-in-the-middle relay; a mismatch means the pairing is
   aborted.

## Recovery phrase

The recovery phrase is an **explicit, user-initiated export** of the root key as a
printable word list, for offline backup. It is the only path by which key material is
meant to leave a device, and only when the user asks for it. Anyone who holds the recovery
phrase can derive every account key, so it must be stored offline and treated as
equivalent to the account's master secret. Losing it, with no other paired device, means
the encrypted data cannot be recovered — this tradeoff is stated at the point of enabling
the mode.

## Implementation

The client hierarchy and pairing live in `crates/mw-crypto/src/zeroaccess.rs`, compiled to
WebAssembly for the browser crypto worker. It composes existing, reviewed primitives —
Argon2id, XChaCha20-Poly1305 (the same at-rest cipher as `mw-store::seal`), P-256 ECDH,
and SHA-256 — and introduces no new cipher of its own.
