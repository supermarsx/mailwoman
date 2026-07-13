// FROZEN Mailwoman crypto client boundary (plan §2.3) — the typed async interface
// the WASM crypto Web Worker (`apps/web/src/crypto/worker.ts`) exposes to the app,
// plus the crypto/security JMAP method-family contract (§2.2). Mirrors the
// `#[wasm_bindgen]` surface in `crates/mw-crypto/src/wasm.rs` and the engine
// `dispatch_security` arms in `crates/mw-engine/src/security/dispatch.rs` — the
// sets MUST stay in lockstep (drift is a build failure, plan §1.5).
//
// Authored by e0; e2/e4 build against the worker STUB (`crypto/index.ts`), e8
// builds the real wasm-pack bundle + wires the worker. ALL private material stays
// in the worker + the passphrase-wrapped client vault and is NEVER posted to the
// main app state or the server in plaintext (plan §1.2 / risk #4).

import { CAP_CRYPTO, CAP_SECURITY, type CryptoKey, type KeyKind } from '../api/crypto-types.ts';
import type { SignatureVerdict } from '../api/security-types.ts';

/** The crypto/security capability URNs, in `JmapRequest.using` order (§2.2). */
export const CRYPTO_CAPABILITIES = [CAP_CRYPTO, CAP_SECURITY] as const;

/** Keyring method names (§2.2). */
export type CryptoKeyMethod =
  | 'CryptoKey/get'
  | 'CryptoKey/set'
  | 'CryptoKey/query'
  | 'CryptoKey/changes'
  | 'CryptoKey/lookup'
  | 'CryptoKey/setTrust';

/** Verdict / sender-control / mail-rule / DLP method names (§2.2). */
export type SecurityMethod =
  | 'SecurityVerdict/get'
  | 'SenderControl/set'
  | 'MailRule/get'
  | 'MailRule/set'
  | 'MailRule/changes'
  | 'Dlp/getRules'
  | 'Dlp/scan';

/** Every crypto/security JMAP method (§2.2). */
export type CryptoSecurityMethod = CryptoKeyMethod | SecurityMethod;

// ── The WASM crypto worker interface (§2.3) ──────────────────────────────────
//
// An `encryptedPrivateBundle` is an OPAQUE, passphrase-wrapped private-key blob —
// it lives only in the worker + client vault, and only its opaque form is ever
// persisted (as `CryptoKey.encryptedPrivateBackup`) for cross-device restore.

/** `generateKey` argument. */
export interface GenerateKeyRequest {
  kind: KeyKind;
  userId: string;
  passphrase: string;
}
/** `generateKey` result — v6 Ed25519/X25519; private key wrapped by `passphrase`. */
export interface GenerateKeyResult {
  publicKeyArmored: string;
  fingerprint: string;
  keyId: string;
  encryptedPrivateBundle: string;
}

/** `encrypt` argument (protected-subject encryption via `protectedSubject`). */
export interface EncryptRequest {
  kind: KeyKind;
  plaintext: string;
  recipientPublicKeys: string[];
  signWithKeyRef?: string;
  passphrase?: string;
  protectedSubject?: string;
}
export interface EncryptResult {
  armoredCiphertext: string;
  encryptedSubjectApplied: boolean;
}

/** `decrypt` argument. */
export interface DecryptRequest {
  kind: KeyKind;
  ciphertext: string;
  encryptedPrivateBundle: string;
  passphrase: string;
}
/**
 * `decrypt` result — the plaintext is sanitized IN-WORKER via the mw-sanitize
 * wasm build before it is returned (plan §1.3). Exactly one of `plaintextHtml` /
 * `plaintextText` is present.
 */
export interface DecryptResult {
  plaintextHtml?: string;
  plaintextText?: string;
  subject?: string;
  signature: SignatureVerdict;
}

/** `sign` argument. */
export interface SignRequest {
  kind: KeyKind;
  data: string;
  encryptedPrivateBundle: string;
  passphrase: string;
  detached: boolean;
}
export interface SignResult {
  signatureArmored: string;
}

/** `verify` argument → `SignatureVerdict` (mirrors `SecurityVerdict.signature`). */
export interface VerifyRequest {
  kind: KeyKind;
  data: string;
  signature: string;
  signerPublicKey: string;
}

/** `importPkcs12` argument/result (S/MIME private-key material, client-side only). */
export interface ImportPkcs12Request {
  p12Bytes: Uint8Array;
  password: string;
}
export interface ImportPkcs12Result {
  certPem: string;
  fingerprint: string;
  encryptedPrivateBundle: string;
}

/** `importArmored` argument/result (`encryptedPrivateBundle` set iff a private key). */
export interface ImportArmoredRequest {
  armored: string;
  passphrase?: string;
}
export interface ImportArmoredResult {
  key: CryptoKey;
  encryptedPrivateBundle?: string;
}

/** `exportBackup` argument/result (Autocrypt Setup Message). */
export interface ExportBackupRequest {
  encryptedPrivateBundle: string;
  kind: KeyKind;
}
export interface ExportBackupResult {
  autocryptSetupMessage: string;
}

/** `unlockKey` argument → an opaque worker-session key ref. */
export interface UnlockKeyRequest {
  encryptedPrivateBundle: string;
  passphrase: string;
}
/** An opaque handle to a key unlocked in the worker session cache. */
export type KeyRef = string;

/**
 * The typed async interface the crypto Web Worker exposes to the app (never
 * blocks the main thread, §2.3). e2/e4 build against a stub implementation
 * (`crypto/index.ts`); e8 backs it with the real wasm-pack module + worker.
 */
export interface CryptoWorkerApi {
  generateKey(req: GenerateKeyRequest): Promise<GenerateKeyResult>;
  encrypt(req: EncryptRequest): Promise<EncryptResult>;
  decrypt(req: DecryptRequest): Promise<DecryptResult>;
  sign(req: SignRequest): Promise<SignResult>;
  verify(req: VerifyRequest): Promise<SignatureVerdict>;
  importPkcs12(req: ImportPkcs12Request): Promise<ImportPkcs12Result>;
  importArmored(req: ImportArmoredRequest): Promise<ImportArmoredResult>;
  exportPublic(req: { keyRef: KeyRef }): Promise<string>;
  exportBackup(req: ExportBackupRequest): Promise<ExportBackupResult>;
  /** Decrypt into the worker session cache; returns a key ref. */
  unlockKey(req: UnlockKeyRequest): Promise<KeyRef>;
  /** `zeroize` + drop the cached private key for `keyRef` (also on timeout). */
  lockKey(req: { keyRef: KeyRef }): Promise<void>;
}
