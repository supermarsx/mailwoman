# OpenPGP & S/MIME end-to-end encryption

Mailwoman's crypto lives in one crate, **`mw-crypto`**, built two ways from the
same source (plan §1.1):

- a **native** build linked into `mw-engine`/`mw-server` for public operations
  (signature *verification*, cert harvesting, WKD HTTP lookup, PQC store-key
  wrapping), and
- a **`wasm32`** build (via `wasm-pack`, loaded by the browser crypto Web Worker)
  for every operation that touches private key material.

OpenPGP is implemented over **rPGP** (`pgp` crate, MIT/Apache — deliberately *not*
`sequoia-openpgp`, which is LGPL). S/MIME is implemented over the RustCrypto
stack (`cms`/`x509-cert`/`rsa`/`p256`). Both are pure Rust and compile to WASM.

## The client-side private-key model (the important part)

**Private keys are generated and held in the browser and are never sent to the
server in plaintext.** Concretely:

- `generateKey` runs in the crypto worker (WASM). It returns a public key plus an
  `encryptedPrivateBundle` — the private key wrapped by a passphrase-derived key
  (rPGP's S2K for PGP; PBES2 for S/MIME). The passphrase never leaves the browser.
- The client vault keeps the opaque bundle; the worker caches an unlocked handle
  in memory only, and `zeroize`s it on lock or timeout.
- `CryptoKey/set` uploads the **public** key and, optionally, the opaque
  `encryptedPrivateBundle` as `encryptedPrivateBackup` (for restoring the key on
  another device). The server stores those bytes verbatim and **cannot decrypt
  them** — there is no plaintext-private field anywhere in the wire schema or the
  store, and an engine test asserts this.
- When `Email/get` returns an encrypted body, the engine marks it encrypted and
  returns the ciphertext MIME opaquely. **Decryption happens in the browser.** The
  decrypted plaintext is sanitized in-worker (WASM `mw-sanitize`) and rendered in
  the existing no-scripts / no-same-origin sandboxed iframe.

If you forget the key passphrase, the backup blob is unrecoverable by design.
Mailwoman (and its operator) cannot reset it.

## Trust (TOFU)

Contact keys are trusted **trust-on-first-use**. The first key seen for an address
is recorded (`tofu`); a later *different* fingerprint for the same address raises a
`keyChanged` alert rather than silently replacing it. A user can promote a key to
`verified` (fingerprint safe-words / QR in the key-management UI) or mark it
`revoked`. Per-contact associations populate the V3 `ContactCard.pgpKey` /
`smimeCert` fields.

## Key discovery: WKD

Mailwoman looks up recipient public keys by **Web Key Directory** (WKD), with user
consent, in addition to keys harvested from received signed mail and imported keys.
Both the advanced (`openpgpkey.<domain>`) and direct methods are derived per RFC.

To **publish** your own users' keys over WKD, point `MW_WKD_DIR` at a directory of
keys and Mailwoman serves the `/.well-known/openpgpkey/...` endpoints. See
[`../deploy/crypto-security.md`](../deploy/crypto-security.md) for the directory
layout and the exact routes.

## S/MIME

S/MIME sign/verify/encrypt/decrypt uses RSA-2048+ / ECDSA-P256 with AES content
encryption. A user imports their certificate + private key as a **PKCS#12 (`.p12`)
bundle** — this is private-key material, so, like PGP keygen, `importPkcs12` runs
**client-side in the worker**; only the resulting cert (public) and an opaque
wrapped bundle are stored. Certificates are also **harvested** from received signed
mail and validated best-effort against a bundled common-CA trust store plus pinned
/ harvested certs. Live OCSP/CRL fetching and LDAP/GAL directory lookup are later
milestones (V6); V4 shows revocation status only when it is present in the cert.

## The 3-state verdict

Both PGP and S/MIME present the same frozen **three-state signature verdict** in
the Security panel: `verified` (good signature from a known key),
`unverified-key` (good signature but the signer key is not yet trusted/stored), or
`invalid` (signature check failed). `none` means the message carried no signature.
This is the frozen UI contract — see the `SecurityVerdict.signature.status` field.

## Post-quantum posture (PQC)

The committed PQC deliverable is a **hybrid X25519 + ML-KEM-768 key-wrap** applied
to the `mw-store` seal key at rest (crypto-agility groundwork; the suite is tagged
on the key material). This is **not** a user-facing E2EE claim, and `ml-kem` is
noted as unaudited. OpenPGP-PQC is behind an off-by-default `pqc` cargo feature.
TLS hybrid (X25519MLKEM768) is not enabled — the tree ships the `ring` provider;
enabling it needs an `aws-lc-rs` license decision and is a follow-up, not a V4 gate.
