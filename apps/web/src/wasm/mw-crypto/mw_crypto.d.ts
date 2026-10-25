/* tslint:disable */
/* eslint-disable */

/**
 * Install the panic→console hook once at module init (wasm-bindgen `start`).
 */
export function __init(): void;

/**
 * `addSubkey({encryptedPrivateBundle, passphrase, purpose, expiresAt?})` →
 * `{publicKeyArmored, encryptedPrivateBundle}`. `purpose`: `encrypt` | `sign`.
 */
export function addSubkey(options: any): any;

/**
 * `analyzeMessage({raw})` → `{kind, partial, path, smimeType?, micalg?,
 * segments?}`: where the message's signed or encrypted part is.
 */
export function analyzeMessage(options: any): any;

/**
 * `applyRevocationCertificate({keyArmored, certificate})` → `{publicKeyArmored}`.
 */
export function applyRevocationCertificate(options: any): any;

/**
 * `attachIssuedCert({certBytes, encryptedPrivateBundle, passphrase})` →
 * `{ certPem, fingerprint, addresses, algorithm, expiresAt, selfIssued }`. Throws
 * when the certificate's public key is not the key in the bundle.
 */
export function attachIssuedCert(options: any): any;

/**
 * `autocryptHeader({publicKeyArmored, addr, preferEncrypt, gossip?})` → the
 * header value (unfolded) for `Autocrypt:`, or `Autocrypt-Gossip:` with `gossip`.
 */
export function autocryptHeader(options: any): any;

/**
 * `autocryptRecommend({peers, ownPreferEncrypt})` → `{perRecipient, message}`,
 * each `disable` | `discourage` | `available` | `encrypt` (Autocrypt §2.4).
 */
export function autocryptRecommend(options: any): any;

/**
 * `certificateRequest({certPem, encryptedPrivateBundle, passphrase})` →
 * `{ csrPem }` — a PKCS#10 request a certificate authority can issue from.
 */
export function certificateRequest(options: any): any;

/**
 * `changePassphrase({encryptedPrivateBundle, oldPassphrase, newPassphrase})` →
 * `{encryptedPrivateBundle}`.
 */
export function changePassphrase(options: any): any;

/**
 * `decrypt({kind, ciphertext, encryptedPrivateBundle, passphrase})` →
 * `{ plaintextText, subject?, signature }`. (In-worker mw-sanitize wasm sanitizes
 * HTML before it reaches the iframe — plan §1.3, wired by e8.)
 */
export function decrypt(options: any): any;

/**
 * `detectAttachedKeys({contentType, filename?, bytes})` → `CryptoKey[]` or
 * `null` when the attachment is not a key.
 */
export function detectAttachedKeys(options: any): any;

/**
 * `encrypt({kind, plaintext, recipientPublicKeys, signWithKeyRef?,
 * protectedSubject?})` → `{ armoredCiphertext, encryptedSubjectApplied }`.
 */
export function encrypt(options: any): any;

/**
 * `exportBackup({encryptedPrivateBundle, kind:"pgp"})` → `{ autocryptSetupMessage }`.
 * An Autocrypt Setup Message is an OpenPGP format, so any other kind is refused
 * (an S/MIME key leaves through `exportPkcs12`).
 */
export function exportBackup(options: any): any;

/**
 * `exportBackupFull({encryptedPrivateBundle, from})` → `{setupCode, payload,
 * mime}`: the still-passphrase-locked key, symmetrically encrypted under a
 * numeric setup code (Autocrypt Level 1 §4.2).
 */
export function exportBackupFull(options: any): any;

/**
 * `exportPkcs12({certPem, encryptedPrivateBundle, passphrase})` →
 * `{ p12Base64 }` — the certificate and key as a PKCS#12 file protected by the
 * same passphrase (see [`smime::export_pkcs12`] for what the file does and does
 * not contain). Base64, so the result is a plain string on its way out of the
 * worker.
 */
export function exportPkcs12(options: any): any;

/**
 * `exportPublic({keyRef})` → armored public key string. `keyRef` may be a session
 * ref or an armored key/bundle directly.
 */
export function exportPublic(options: any): any;

/**
 * `generateKey({kind, userId, passphrase})`.
 *
 * - `kind:"pgp"` → `{ publicKeyArmored, fingerprint, keyId,
 *   encryptedPrivateBundle }` — v6 Ed25519/X25519.
 * - `kind:"smime"` → `{ certPem, fingerprint, keyId, encryptedPrivateBundle,
 *   addresses, algorithm, expiresAt }` — an RSA key and an X.509 certificate for
 *   the address in `userId`, signed by that key itself (see [`smime::generate`]).
 *   `addresses`, `algorithm` and `expiresAt` are read back from the certificate.
 */
export function generateKey(options: any): any;

/**
 * `importArmored({armored})` → `CryptoKey` (+ `encryptedPrivateBundle` when the
 * armor carried a private key).
 */
export function importArmored(options: any): any;

/**
 * `importPkcs12({p12Bytes, password})` → `{ certPem, fingerprint,
 * encryptedPrivateBundle, addresses, algorithm, expiresAt }` — S/MIME private-key
 * material, client-side only. The last three are read from the certificate.
 */
export function importPkcs12(options: any): any;

/**
 * `inspectKey({armored})` → `PgpKeyInfo`: version, algorithm, creation, expiry,
 * state (`valid`/`expired`/`revoked`/`invalid`/`not-yet-valid`), flags, User IDs
 * and subkeys, each judged from verified self-signatures.
 */
export function inspectKey(options: any): any;

/**
 * `lockKey({keyRef})` — `zeroize` + drop the cached private material for this ref.
 */
export function lockKey(options: any): any;

/**
 * `parseAutocrypt({value, gossip?})` → `{addr, preferEncrypt, key}`.
 */
export function parseAutocrypt(options: any): any;

/**
 * `pgpMimeEncrypt({payload, headers, recipientPublicKeys, signWithKeyRef? |
 * encryptedPrivateBundle+passphrase, gossip?})` → `{entity, contentType,
 * outerSubject, seipdVersion}`: an RFC 3156 `multipart/encrypted` entity with RFC
 * 9788 header protection inside (`Subject` outside becomes `[...]`).
 */
export function pgpMimeEncrypt(options: any): any;

/**
 * `pgpMimeOpen({raw, encryptedPrivateBundle, passphrase, signerPublicKeys})` →
 * `{entity, protected, verification, encrypted, aead, partial, gossip}`. The
 * `entity` is MIME; its HTML parts must go through the sanitizer before display.
 */
export function pgpMimeOpen(options: any): any;

/**
 * `pgpMimeSign({payload, headers, signWithKeyRef? |
 * encryptedPrivateBundle+passphrase})` → `{entity, contentType, outerSubject}`:
 * an RFC 3156 `multipart/signed` entity (`hp="clear"` header protection).
 */
export function pgpMimeSign(options: any): any;

/**
 * `pgpMimeVerify({raw, signerPublicKeys})` → the `pgpMimeOpen` shape for a
 * signed-only message (`multipart/signed` or an inline cleartext signature).
 */
export function pgpMimeVerify(options: any): any;

/**
 * `restoreBackup({payload, setupCode})` → `{encryptedPrivateBundle}` (still locked
 * with the key's own passphrase).
 */
export function restoreBackup(options: any): any;

/**
 * `revocationCertificate({encryptedPrivateBundle, passphrase, reason, message})`
 * → `{certificate}` (armored; keep it aside to revoke the key later).
 */
export function revocationCertificate(options: any): any;

/**
 * `revokeKey({encryptedPrivateBundle, passphrase, reason, message})` →
 * `{publicKeyArmored, encryptedPrivateBundle}`. `reason`: `compromised` |
 * `superseded` | `retired` | `unspecified`.
 */
export function revokeKey(options: any): any;

/**
 * `revokeSubkey({encryptedPrivateBundle, passphrase, subkeyFingerprint, reason,
 * message})` → `{publicKeyArmored, encryptedPrivateBundle}`.
 */
export function revokeSubkey(options: any): any;

/**
 * `setKeyExpiry({encryptedPrivateBundle, passphrase, expiresAt})` →
 * `{publicKeyArmored, encryptedPrivateBundle}`; applies to the key and its
 * subkeys.
 */
export function setKeyExpiry(options: any): any;

/**
 * `sign({kind, data, encryptedPrivateBundle, passphrase, detached})` →
 * `{ signatureArmored }`. S/MIME additionally needs the signer `certPem`.
 */
export function sign(options: any): any;

export function smimeRootSetInfo(_options: any): any;

/**
 * `unlockKey({kind, encryptedPrivateBundle, passphrase})` → `{ keyRef }`. Caches the
 * bundle + passphrase in the worker session so `signWithKeyRef` needs no re-entry.
 */
export function unlockKey(options: any): any;

/**
 * `verify({kind, data, signature, signerPublicKey})` → `SignatureVerdict`.
 */
export function verify(options: any): any;

/**
 * `verifyPgp({data?, signature, signerPublicKeys})` → `{verification,
 * signedText?}`. With `data` the signature is detached (armored or binary);
 * without, `signature` is a `PGP SIGNED MESSAGE`.
 */
export function verifyPgp(options: any): any;

/**
 * Verify exact CMS bytes and return chain, sender and revocation evidence.
 */
export function verifySmime(options: any): any;

/**
 * `wkdUrls({email})` → `[advancedUrl, directUrl]`.
 */
export function wkdUrls(options: any): any;

/**
 * `zaDeriveKek({keyRef})` → `{ keyRef }` (the KEK, as a new in-worker ref).
 */
export function zaDeriveKek(options: any): any;

/**
 * `deriveRootKey({secretB64, saltB64, mCost, tCost, pCost})` → `{ keyRef }`.
 * The root key stays in-worker; only its ref is returned.
 */
export function zaDeriveRootKey(options: any): any;

/**
 * `zaDeriveSubkey({keyRef, label})` → `{ keyRef }` — per-class keys
 * (`"message-cache"`, `"search"`, `"notes"`, `"attachment"`).
 */
export function zaDeriveSubkey(options: any): any;

/**
 * `zaGenerateDataKey()` → `{ keyRef }` — a fresh random per-account data key.
 */
export function zaGenerateDataKey(): any;

/**
 * `zaLock({keyRef})` — zeroize + drop one cached hierarchy key.
 */
export function zaLock(options: any): any;

/**
 * `zaLockAll()` — clear the entire zero-access session (logout/timeout).
 */
export function zaLockAll(): any;

/**
 * `zaOpenRow({keyRef, ciphertextB64, table, rowId, schemaVersion})` →
 * `{ plaintextB64 }`. Fails on a wrong key or a moved row (AAD mismatch).
 */
export function zaOpenRow(options: any): any;

/**
 * `zaPairComplete({envelopeB64, secretRef})` → `{ sasWords, keyRef }` (new
 * device). Recovers the root key into the session; `sasWords` is shown for
 * the user to compare against the other device before trusting.
 */
export function zaPairComplete(options: any): any;

/**
 * `zaPairGenerate()` → `{ publicB64, secretRef }` (new device). `publicB64`
 * goes in the QR; the secret stays in-worker under `secretRef`.
 */
export function zaPairGenerate(): any;

/**
 * `zaPairSeal({rootRef, peerPublicB64})` → `{ sasWords, envelopeB64 }`
 * (existing device). Seals the root key to the scanned public; the envelope
 * is opaque to the relaying server.
 */
export function zaPairSeal(options: any): any;

/**
 * `zaRecoveryPhrase({keyRef})` → `{ phrase }`. EXPLICIT user export of the
 * root key for offline backup — the sole intentional key-egress path.
 */
export function zaRecoveryPhrase(options: any): any;

/**
 * `zaRestoreFromPhrase({phrase})` → `{ keyRef }` — re-imports the root key
 * into the worker session (checksum-verified).
 */
export function zaRestoreFromPhrase(options: any): any;

/**
 * `zaSealRow({keyRef, plaintextB64, table, rowId, schemaVersion})` →
 * `{ ciphertextB64 }`. AAD is bound per [`row_aad`] (§9.3).
 */
export function zaSealRow(options: any): any;

/**
 * `zaUnwrapKey({kekRef, blobB64})` → `{ keyRef }` (the data key, in-worker).
 */
export function zaUnwrapKey(options: any): any;

/**
 * `zaWrapKey({kekRef, dataKeyRef})` → `{ blobB64 }` (the wrapped data key,
 * safe to persist server-side). Raw keys never cross the boundary.
 */
export function zaWrapKey(options: any): any;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __init: () => void;
    readonly addSubkey: (a: number, b: number) => void;
    readonly analyzeMessage: (a: number, b: number) => void;
    readonly applyRevocationCertificate: (a: number, b: number) => void;
    readonly attachIssuedCert: (a: number, b: number) => void;
    readonly autocryptHeader: (a: number, b: number) => void;
    readonly autocryptRecommend: (a: number, b: number) => void;
    readonly certificateRequest: (a: number, b: number) => void;
    readonly changePassphrase: (a: number, b: number) => void;
    readonly decrypt: (a: number, b: number) => void;
    readonly detectAttachedKeys: (a: number, b: number) => void;
    readonly encrypt: (a: number, b: number) => void;
    readonly exportBackup: (a: number, b: number) => void;
    readonly exportBackupFull: (a: number, b: number) => void;
    readonly exportPkcs12: (a: number, b: number) => void;
    readonly exportPublic: (a: number, b: number) => void;
    readonly generateKey: (a: number, b: number) => void;
    readonly importArmored: (a: number, b: number) => void;
    readonly importPkcs12: (a: number, b: number) => void;
    readonly inspectKey: (a: number, b: number) => void;
    readonly lockKey: (a: number, b: number) => void;
    readonly parseAutocrypt: (a: number, b: number) => void;
    readonly pgpMimeEncrypt: (a: number, b: number) => void;
    readonly pgpMimeOpen: (a: number, b: number) => void;
    readonly pgpMimeSign: (a: number, b: number) => void;
    readonly pgpMimeVerify: (a: number, b: number) => void;
    readonly restoreBackup: (a: number, b: number) => void;
    readonly revocationCertificate: (a: number, b: number) => void;
    readonly revokeKey: (a: number, b: number) => void;
    readonly revokeSubkey: (a: number, b: number) => void;
    readonly setKeyExpiry: (a: number, b: number) => void;
    readonly sign: (a: number, b: number) => void;
    readonly smimeRootSetInfo: (a: number, b: number) => void;
    readonly unlockKey: (a: number, b: number) => void;
    readonly verify: (a: number, b: number) => void;
    readonly verifyPgp: (a: number, b: number) => void;
    readonly verifySmime: (a: number, b: number) => void;
    readonly wkdUrls: (a: number, b: number) => void;
    readonly zaDeriveKek: (a: number, b: number) => void;
    readonly zaDeriveRootKey: (a: number, b: number) => void;
    readonly zaDeriveSubkey: (a: number, b: number) => void;
    readonly zaGenerateDataKey: (a: number) => void;
    readonly zaLock: (a: number, b: number) => void;
    readonly zaLockAll: (a: number) => void;
    readonly zaOpenRow: (a: number, b: number) => void;
    readonly zaPairComplete: (a: number, b: number) => void;
    readonly zaPairGenerate: (a: number) => void;
    readonly zaPairSeal: (a: number, b: number) => void;
    readonly zaRecoveryPhrase: (a: number, b: number) => void;
    readonly zaRestoreFromPhrase: (a: number, b: number) => void;
    readonly zaSealRow: (a: number, b: number) => void;
    readonly zaUnwrapKey: (a: number, b: number) => void;
    readonly zaWrapKey: (a: number, b: number) => void;
    readonly __wbindgen_export: (a: number, b: number) => number;
    readonly __wbindgen_export2: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_export3: (a: number) => void;
    readonly __wbindgen_export4: (a: number, b: number, c: number) => void;
    readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
