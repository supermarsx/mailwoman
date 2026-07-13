# Crypto interop fixtures (mw-crypto, V4)

Recorded (never live) interop vectors for the `mw-crypto` acceptance tests
(plan §3 e1 / §6#6). No test hits a live keyserver, WKD host, or DNS.

## `pgp/` — GnuPG interop

Generated with GnuPG 2.4.x (Ed25519 primary + cv25519 encryption subkey, passphrase
`interop-pass`) in a throwaway `GNUPGHOME`:

```sh
gpg --batch --gen-key <spec>                      # eddsa/ed25519 + ecdh/cv25519
gpg --armor --export bob@example.com          > gnupg-public.asc
gpg --armor --export-secret-keys bob@…        > gnupg-secret.asc   # passphrase-locked
gpg --armor --sign --encrypt -r bob@… plain   > gnupg-message.asc  # signed + encrypted
```

`tests/pgp.rs::gnupg_interop_decrypt_verify` decrypts + verifies `gnupg-message.asc`
with rPGP — proving rPGP consumes real GnuPG output.

Thunderbird output is standard RFC 9580/Autocrypt OpenPGP (same rPGP path); a
recorded Thunderbird vector can be dropped in here for CI (e9) without code change.

## `smime/` — openssl (Outlook-style) interop

Generated with OpenSSL 3.x — an RSA-2048 self-signed S/MIME cert (`alice`), its
PKCS#12 bundle (password `test`), and CMS messages:

```sh
openssl req -x509 -newkey rsa:2048 -nodes -keyout alice.key.pem -out alice.crt.pem …
openssl pkcs12 -export -inkey alice.key.pem -in alice.crt.pem -out alice.p12 -passout pass:test
openssl cms -encrypt -aes-256-cbc -binary -outform DER -in plain.txt -out enveloped.der alice.crt.pem
openssl cms -sign -signer alice.crt.pem -inkey alice.key.pem -in plain.txt -outform DER -out signed.der -nodetach
```

`tests/smime.rs` imports `alice.p12`, decrypts `enveloped.der` (RSA key transport +
AES-256-CBC — the Outlook-style profile), and verifies `signed.der`.
