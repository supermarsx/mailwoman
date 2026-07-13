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

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
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
