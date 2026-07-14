// The typed zero-access crypto boundary (SPEC §9, plan §2.6 / §3 e8). This mirrors
// the 15 `za*` `#[wasm_bindgen]` exports built by e6 (`crates/mw-crypto/src/zeroaccess.rs`)
// EXACTLY — the app never hand-rolls crypto in JS (hard constraint): every operation
// is a call into the wasm module, run off the main thread in a dedicated Web Worker
// (`worker.ts` / `worker.entry.ts`). Derived root/KEK/data keys stay inside the worker
// session, addressed by opaque `keyRef`s; only ciphertext, wrapped-key blobs, public
// pairing material, SAS words, and the (explicitly user-exported) recovery phrase ever
// cross this boundary.
//
// FROZEN row binding (state.md, from e6): a sealed row's AAD is
//   AAD = table ‖ 0x1F ‖ row_id ‖ 0x1F ‖ ascii-decimal(schema_version)
// and the ciphertext layout is `nonce(24) ‖ XChaCha20-Poly1305(ct+tag)`. The AAD is
// computed INSIDE the wasm (`zaSealRow`/`zaOpenRow` take the three components), so JS
// only supplies `table` / `rowId` / `schemaVersion` and never assembles the AAD itself.

/** An opaque handle to a key held in the worker session (never the key bytes). */
export type ZaKeyRef = string;

/** Argon2id cost parameters recorded server-side so any device re-derives the root. */
export interface ZaKdfParams {
  readonly mCost: number;
  readonly tCost: number;
  readonly pCost: number;
}

/** OWASP-ish interactive defaults, matching `ArgonParams::interactive()` (e6). */
export const ZA_KDF_INTERACTIVE: ZaKdfParams = { mCost: 19_456, tCost: 2, pCost: 1 };

/** The frozen per-class data-key labels (`zaDeriveSubkey` `label`). */
export const ZA_SUBKEY_LABELS = ['message-cache', 'search', 'notes', 'attachment'] as const;
export type ZaSubkeyLabel = (typeof ZA_SUBKEY_LABELS)[number];

export interface ZaDeriveRootIn {
  readonly secretB64: string;
  readonly saltB64: string;
  readonly mCost: number;
  readonly tCost: number;
  readonly pCost: number;
}
export interface ZaRefOut {
  readonly keyRef: ZaKeyRef;
}
export interface ZaSubkeyIn {
  readonly keyRef: ZaKeyRef;
  readonly label: string;
}
export interface ZaWrapIn {
  readonly kekRef: ZaKeyRef;
  readonly dataKeyRef: ZaKeyRef;
}
export interface ZaBlobOut {
  readonly blobB64: string;
}
export interface ZaUnwrapIn {
  readonly kekRef: ZaKeyRef;
  readonly blobB64: string;
}
export interface ZaSealRowIn {
  readonly keyRef: ZaKeyRef;
  readonly plaintextB64: string;
  readonly table: string;
  readonly rowId: string;
  readonly schemaVersion: number;
}
export interface ZaCiphertextOut {
  readonly ciphertextB64: string;
}
export interface ZaOpenRowIn {
  readonly keyRef: ZaKeyRef;
  readonly ciphertextB64: string;
  readonly table: string;
  readonly rowId: string;
  readonly schemaVersion: number;
}
export interface ZaPlaintextOut {
  readonly plaintextB64: string;
}
export interface ZaPhraseOut {
  readonly phrase: string;
}
export interface ZaPairGenOut {
  readonly publicB64: string;
  readonly secretRef: ZaKeyRef;
}
export interface ZaPairSealIn {
  readonly rootRef: ZaKeyRef;
  readonly peerPublicB64: string;
}
export interface ZaPairSealOut {
  readonly sasWords: readonly string[];
  readonly envelopeB64: string;
}
export interface ZaPairCompleteIn {
  readonly envelopeB64: string;
  readonly secretRef: ZaKeyRef;
}
export interface ZaPairCompleteOut {
  readonly sasWords: readonly string[];
  readonly keyRef: ZaKeyRef;
}

/**
 * The async facade over the 15 `za*` wasm exports. The real implementation
 * (`spawnZeroAccessWorker`) marshals each call to the worker; tests inject a mock
 * that honours the frozen AAD binding (seal→open round-trips iff table/rowId/
 * schemaVersion match).
 */
export interface ZeroAccessCrypto {
  deriveRootKey(input: ZaDeriveRootIn): Promise<ZaRefOut>;
  deriveKek(input: ZaRefOut): Promise<ZaRefOut>;
  deriveSubkey(input: ZaSubkeyIn): Promise<ZaRefOut>;
  generateDataKey(): Promise<ZaRefOut>;
  wrapKey(input: ZaWrapIn): Promise<ZaBlobOut>;
  unwrapKey(input: ZaUnwrapIn): Promise<ZaRefOut>;
  sealRow(input: ZaSealRowIn): Promise<ZaCiphertextOut>;
  openRow(input: ZaOpenRowIn): Promise<ZaPlaintextOut>;
  recoveryPhrase(input: ZaRefOut): Promise<ZaPhraseOut>;
  restoreFromPhrase(input: { readonly phrase: string }): Promise<ZaRefOut>;
  pairGenerate(): Promise<ZaPairGenOut>;
  pairSeal(input: ZaPairSealIn): Promise<ZaPairSealOut>;
  pairComplete(input: ZaPairCompleteIn): Promise<ZaPairCompleteOut>;
  lock(input: ZaRefOut): Promise<void>;
  lockAll(): Promise<void>;
}

/** The worker RPC method names → the `za*` wasm export names (1:1). */
export const ZA_METHODS = {
  deriveRootKey: 'zaDeriveRootKey',
  deriveKek: 'zaDeriveKek',
  deriveSubkey: 'zaDeriveSubkey',
  generateDataKey: 'zaGenerateDataKey',
  wrapKey: 'zaWrapKey',
  unwrapKey: 'zaUnwrapKey',
  sealRow: 'zaSealRow',
  openRow: 'zaOpenRow',
  recoveryPhrase: 'zaRecoveryPhrase',
  restoreFromPhrase: 'zaRestoreFromPhrase',
  pairGenerate: 'zaPairGenerate',
  pairSeal: 'zaPairSeal',
  pairComplete: 'zaPairComplete',
  lock: 'zaLock',
  lockAll: 'zaLockAll',
} as const satisfies Record<keyof ZeroAccessCrypto, string>;

/** UTF-8 string → base64 (for `plaintextB64` inputs). */
export function utf8ToB64(text: string): string {
  const bytes = new TextEncoder().encode(text);
  let bin = '';
  for (const b of bytes) bin += String.fromCharCode(b);
  return btoa(bin);
}

/** base64 → UTF-8 string (for `plaintextB64` outputs). */
export function b64ToUtf8(b64: string): string {
  const bin = atob(b64);
  const bytes = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i += 1) bytes[i] = bin.charCodeAt(i);
  return new TextDecoder().decode(bytes);
}
