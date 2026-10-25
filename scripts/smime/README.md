# S/MIME core fixtures and roots

`gen-fixtures.sh` requires OpenSSL 3.4 or later. It creates the test-only keys,
certificates, signatures, envelopes, PKCS#12 files and revocation responses in
`fixtures/crypto/smime/e41/`. The keys are public test material and must never be
used for real mail. The PKCS#12 password is `test`. Regeneration changes random
keys and signatures; commit the complete fixture set together. The fixture
messages use exact CRLF bytes and are covered by `.gitattributes`' binary rule.

Run the portable fixture tests with:

```sh
cargo test -p mw-crypto --locked --test smime_e41 --test smime_e41_red
MW_SMIME_INTEROP=1 cargo test -p mw-crypto --locked --test smime_openssl_live
```

The live test fails if OpenSSL is missing. It verifies Mailwoman-generated RSA,
P-256, P-384 and Ed25519 detached/opaque signatures, decrypts OAEP and ECDH
envelopes under both CBC and GCM, imports exported PKCS#12 files, and parses an
OCSP request. The ordinary fixture tests check the opposite direction.

`gen-roots.mjs` extracts **email-protection** trust from Mozilla NSS certdata;
TLS trust is deliberately not used. The generated roots and metadata live in
`crates/mw-crypto/data/smime-roots/`. For an intentional refresh:

```sh
node scripts/smime/gen-roots.mjs --source /path/to/certdata.txt --snapshot YYYY-MM-DD
```

Keep the original certdata snapshot when reproducing an existing root set:
`roots.json.sourceSha256` identifies the exact input. The upstream `default`
URL changes over time. Generated root data derives from Mozilla NSS certdata,
available under MPL-2.0 at the source URL recorded in `roots.json`.

The browser enables these roots by default; `verifySmime({mozilla:false})`
disables them for a domain. `userTrust` and `adminTrust` accept PEM strings;
the server supplies the effective domain/global administrator policy.
`smimeRootSetInfo({})` exposes the snapshot date, source digest and root count.
The worker never fetches revocation URLs: `verifySmime` returns public
`revocationQueries`, and accepts the server's authenticated OCSP/CRL results
on a second verification. `hardFail:true` keeps unchecked revocation from
counting as verified. Callers must supply the message's From address as `sender`.

Certificate validation uses the current verification clock. A signed
`signingTime` is reported, but cannot override expiration: it is chosen by the
signer and is not a trusted timestamp. Unsupported critical path constraints
are refused. Encryption checks validity, key usage, email EKU and minimum RSA
size before constructing recipient info; X25519 low-order points are refused.

Rebuild the committed browser guest with the pinned Linux process documented
in `scripts/build-wasm.sh`, then run `apps/web/src/crypto/wasm-smime-e41.test.ts`
alongside the existing crypto tests. The new worker functions are declared in
`apps/web/src/contracts/crypto.ts`; ciphertext remains device-side until the
caller submits the already-encrypted CMS bytes.
