# OpenPGP and S/MIME

Mailwoman can sign, encrypt, decrypt and verify OpenPGP mail in the browser, and it manages OpenPGP and S/MIME keys under **Keys & certificates**. This page explains where private keys live, what the server stores and can do, how contact keys are found and trusted, and the current limits.

<!-- screenshot: id=security/keys-overview; screen=Keys & certificates; state=one own OpenPGP key and two contact keys; element=whole module with Your keys and Contact keys lists -->

## One library, two builds

All cryptography is in the Rust crate `mw-crypto`. OpenPGP uses rPGP (the `pgp` crate); S/MIME uses the RustCrypto crates (`cms`, `x509-cert`, `rsa`, `p256`, `p384`, `ed25519-dalek`). The crate is built twice:

- **In the browser**, as WebAssembly inside a dedicated crypto Web Worker. Every operation that needs a private key runs here: key generation, decryption, signing, passphrase changes, PKCS#12 import and export, backups.
- **On the server**, for operations that need no secret: verifying signatures on received mail, looking up keys by Web Key Directory (WKD), and computing the message's security verdict. The server build does not contain the S/MIME RSA decryption or PKCS#12 private-key code.

## Private keys

<!-- screenshot: id=security/keys-generate; screen=Keys & certificates > Generate key; state=OpenPGP selected; element=Generate a key dialog with Key passphrase field -->

- **Generate key** creates the key in the crypto worker. An OpenPGP key is, by default, a version 6 key (Ed25519 primary key for certifying and signing, X25519 encryption subkey, RFC 9580); a compatible profile makes the Ed25519/Curve25519 key type that GnuPG 2.2 to 2.4 use. An S/MIME key is RSA 3072-bit with a self-signed certificate valid for two years.
- Before the private key leaves the worker it is locked with the **Key passphrase**: OpenPGP keys with OpenPGP passphrase protection (S2K) on the primary key and every subkey; S/MIME keys as a PKCS#8 `EncryptedPrivateKeyInfo` (PBES2 with PBKDF2-HMAC-SHA256 and AES-256-CBC).
- The locked key is uploaded to the server and stored in the key record's `encryptedPrivateBackup` field, so the key can be used from another browser. This happens for every generated or imported private key. The passphrase is never sent.
- Each private-key operation passes the locked key and the passphrase to the worker, which unlocks the key for that operation. In the page itself the locked key is held only in memory; nothing is written to browser storage.

What this means for the server and its operator: the server holds every user's private keys in passphrase-locked form. It cannot use them without the passphrase, but anyone with a copy of the database (or the running server) can try to guess passphrases offline. A long, unique passphrase is the only protection here. A forgotten passphrase cannot be reset by the user or the operator.

The key panel's subtitle says private keys "never reach the server"; read that as "never reach the server unencrypted".

## Reading encrypted mail

When a message is OpenPGP-encrypted, the server returns the encrypted MIME as it is and marks it as encrypted. The browser decrypts it in the crypto worker. If the decrypted body is HTML, the worker sanitizes it with the WebAssembly build of the same HTML sanitizer the server uses, before it is shown in the sandboxed message frame. Decrypted content is not sent back to the server for sanitizing or rendering.

## Contact keys and trust

<!-- screenshot: id=security/keys-lookup; screen=Keys & certificates; state=lookup form filled in; element=Look up a key form with Consent to external lookup checkbox -->

Contact keys come from:

- **Import key**: an armored OpenPGP key (**Armored (PGP)**) or a PKCS#12 file (**PKCS#12 (S/MIME)**);
- **Look up a key**, which asks only after you tick **Consent to external lookup**. The server queries WKD; the key-server (VKS) option is offered but the server does not query key servers;
- S/MIME certificates from a configured LDAP directory;
- automatic WKD lookups while composing and while decrypting (see below).

Each key has a **Trust level**: **unverified**, **tofu**, **verified** or **revoked**. Mailwoman does not change the trust level by itself. You set it in the key details after comparing the fingerprint with its owner (the details view shows the fingerprint and a word form of it). Compose prefers verified keys, then tofu keys, then the rest.

Not implemented:

- Keys and certificates are **not** collected from received signed mail. The code to extract them exists but is not called when mail arrives, so a key's source is only ever what you imported or looked up.
- There is no automatic alert when a contact's key changes. The security panel has a "key changed" warning, but nothing currently sets it.

### Automatic WKD lookups

Two actions trigger a WKD lookup without the consent tick that the manual lookup asks for:

- **Composing**: when you add a recipient and no key for that address is stored.
- **Decrypting a message**: when no key for the sender's address is stored, to check the signature inside it.

The server then fetches the key from the address's domain (`openpgpkey.<domain>` first, then the domain itself). That domain sees a request from your server's address for that key. When you decrypt a message from a sender whose key you do not have, this tells the sender's domain that the message was decrypted, and roughly when. A key found this way is stored as a contact key with source `wkd`. The fetch goes through the server's outbound-request filter ([Outbound requests](./egress.md)).

## Publishing your users' keys (WKD)

Set `MW_WKD_DIR` to a directory of public keys and Mailwoman serves them at `/.well-known/openpgpkey/hu/<hash>` and `/.well-known/openpgpkey/<domain>/hu/<hash>`, with the matching `policy` files. Only public keys belong in that directory. The layout is in [Crypto and security settings](/deploy/crypto-security).

## S/MIME

<!-- screenshot: id=security/keys-smime; screen=Keys & certificates; state=own S/MIME key selected; element=S/MIME certificate section with export and request buttons -->

- Supported private key types: RSA (2048 to 4096 bits), ECDSA/ECDH P-256 and P-384, Ed25519.
- You can generate a key with a self-signed certificate, download a certificate request (`.csr`) for a certificate authority, replace the certificate with the one the authority issues (a certificate for a different key is refused), import a PKCS#12 (`.p12`) file and export one. PKCS#12 import and export run in the crypto worker.
- The web app does **not** sign, encrypt or decrypt mail with S/MIME keys. Use an exported key in another mail program for that.
- Signatures on received S/MIME mail are verified against the bundled Mozilla email-protection root set (taken from NSS; on by default and can be turned off per domain), roots an administrator adds per domain or globally, and certificates the user trusts.
- Revocation (OCSP and CRL) is not fetched. The revocation status shown for a certificate is "unknown".

## The signature verdict

The security panel shows one signature status for OpenPGP and S/MIME alike:

| Status | Meaning |
|---|---|
| `verified` | The signature is valid and the signing key is known |
| `unverified-key` | The signature is valid but the signing key is not stored or not trusted |
| `invalid` | The signature check failed |
| `none` | The message is not signed |

DKIM, SPF, DMARC and ARC results, the `Received` chain and attachment risk are computed on the server; they need no secret.

## Post-quantum status

- OpenPGP and S/MIME use classical algorithms only. No post-quantum OpenPGP algorithm is implemented.
- TLS uses rustls with the `ring` provider. The hybrid post-quantum key exchange (X25519MLKEM768) is not enabled.
- At first start the server stores a copy of its server key wrapped with a hybrid X25519 + ML-KEM-768 key, but discards the secret needed to unwrap it, so that copy protects nothing (see [Encryption at rest](./encryption-at-rest.md)).

## What this protects against, and what it does not

| Threat | Result |
|---|---|
| Someone reads the server's database or backups | Encrypted mail stays encrypted. Private keys are passphrase-locked; a weak passphrase can be guessed offline |
| The mail server or a relay reads mail in transit | OpenPGP-encrypted message bodies are unreadable to them. Headers outside the protected part, sender, recipients and timing are visible |
| An operator modifies the web app served to users | Not protected. The browser code that handles passphrases and decrypted text comes from the server; a malicious server can change it |
| A recipient's key is swapped (for example via WKD) | Not detected automatically; compare fingerprints and set **verified** |

## Related

- [Zero-access storage](./zero-access.md)
- [Encryption at rest](./encryption-at-rest.md)
- [Outbound requests](./egress.md)
- [Crypto and security settings](/deploy/crypto-security)
- [Encryption keys (user guide)](/user/encryption-keys)
