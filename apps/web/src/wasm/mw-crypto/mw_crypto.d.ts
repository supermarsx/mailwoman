/* tslint:disable */
/* eslint-disable */

/**
 * Install the panic→console hook once at module init (wasm-bindgen `start`).
 */
export function __init(): void;

/**
 * `decrypt({kind, ciphertext, encryptedPrivateBundle, passphrase})` →
 * `{ plaintextText, subject?, signature }`. (In-worker mw-sanitize wasm sanitizes
 * HTML before it reaches the iframe — plan §1.3, wired by e8.)
 */
export function decrypt(options: any): any;

/**
 * `encrypt({kind, plaintext, recipientPublicKeys, signWithKeyRef?,
 * protectedSubject?})` → `{ armoredCiphertext, encryptedSubjectApplied }`.
 */
export function encrypt(options: any): any;

/**
 * `exportBackup({encryptedPrivateBundle, kind})` → `{ autocryptSetupMessage }`.
 */
export function exportBackup(options: any): any;

/**
 * `exportPublic({keyRef})` → armored public key string. `keyRef` may be a session
 * ref or an armored key/bundle directly.
 */
export function exportPublic(options: any): any;

/**
 * `generateKey({kind:"pgp", userId, passphrase})` → `{ publicKeyArmored,
 * fingerprint, keyId, encryptedPrivateBundle }` — v6 Ed25519/X25519.
 */
export function generateKey(options: any): any;

/**
 * `importArmored({armored})` → `CryptoKey` (+ `encryptedPrivateBundle` when the
 * armor carried a private key).
 */
export function importArmored(options: any): any;

/**
 * `importPkcs12({p12Bytes, password})` → `{ certPem, fingerprint,
 * encryptedPrivateBundle }` — S/MIME private-key material, client-side only.
 */
export function importPkcs12(options: any): any;

/**
 * `lockKey({keyRef})` — `zeroize` + drop the cached private material for this ref.
 */
export function lockKey(options: any): any;

/**
 * `sign({kind, data, encryptedPrivateBundle, passphrase, detached})` →
 * `{ signatureArmored }`. S/MIME additionally needs the signer `certPem`.
 */
export function sign(options: any): any;

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
    readonly zaDeriveKek: (a: any) => [number, number, number];
    readonly zaDeriveRootKey: (a: any) => [number, number, number];
    readonly zaDeriveSubkey: (a: any) => [number, number, number];
    readonly zaGenerateDataKey: () => [number, number, number];
    readonly zaLock: (a: any) => [number, number, number];
    readonly zaLockAll: () => [number, number, number];
    readonly zaOpenRow: (a: any) => [number, number, number];
    readonly zaPairComplete: (a: any) => [number, number, number];
    readonly zaPairGenerate: () => [number, number, number];
    readonly zaPairSeal: (a: any) => [number, number, number];
    readonly zaRecoveryPhrase: (a: any) => [number, number, number];
    readonly zaRestoreFromPhrase: (a: any) => [number, number, number];
    readonly zaSealRow: (a: any) => [number, number, number];
    readonly zaUnwrapKey: (a: any) => [number, number, number];
    readonly zaWrapKey: (a: any) => [number, number, number];
    readonly __init: () => void;
    readonly decrypt: (a: any) => [number, number, number];
    readonly encrypt: (a: any) => [number, number, number];
    readonly exportBackup: (a: any) => [number, number, number];
    readonly exportPublic: (a: any) => [number, number, number];
    readonly generateKey: (a: any) => [number, number, number];
    readonly importArmored: (a: any) => [number, number, number];
    readonly importPkcs12: (a: any) => [number, number, number];
    readonly lockKey: (a: any) => [number, number, number];
    readonly sign: (a: any) => [number, number, number];
    readonly unlockKey: (a: any) => [number, number, number];
    readonly verify: (a: any) => [number, number, number];
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __externref_table_dealloc: (a: number) => void;
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
