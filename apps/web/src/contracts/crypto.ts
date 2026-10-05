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
  /** `Name <email>` or a bare address. */
  userId: string;
  passphrase: string;
  /** S/MIME only: RSA modulus size. Absent → the worker's default (3072). */
  rsaBits?: 2048 | 3072 | 4096;
}
/**
 * `generateKey` result; the private key is wrapped by `passphrase`.
 *
 * - `kind: 'pgp'` → a v6 Ed25519/X25519 key: `publicKeyArmored` is set, the
 *   certificate fields are absent.
 * - `kind: 'smime'` → an RSA key and an X.509 certificate signed by that key
 *   itself: `certPem`, `addresses`, `algorithm` and `expiresAt` are set (the last
 *   three read back from the certificate), `publicKeyArmored` is absent.
 *
 * A field that does not apply is absent, never a placeholder.
 */
export interface GenerateKeyResult {
  publicKeyArmored?: string;
  fingerprint: string;
  keyId: string;
  encryptedPrivateBundle: string;
  certPem?: string;
  addresses?: string[];
  /** `rsa-<modulus bits>`. */
  algorithm?: string;
  /** The certificate's `notAfter`, RFC 3339. */
  expiresAt?: string;
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
  /**
   * The sender's armored public key, supplied so the worker VERIFIES an inline
   * signature and returns a real `signature` verdict. Additive + optional: when
   * omitted the decrypt runs exactly as before (verdict status `"none"` — the
   * signature is not verified). An absent/mismatched key never yields a false
   * `"verified"` (the worker returns `"none"`/`"invalid"`). The wasm `decrypt`
   * DTO already accepts this key (`signerPublicKey`); mirrored native in
   * `crates/mw-crypto/src/pgp.rs::decrypt`.
   */
  signerPublicKey?: string;
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
  /** Read from the certificate: its addresses, key algorithm and `notAfter`. */
  addresses: string[];
  algorithm: string;
  expiresAt: string;
}

/** An own S/MIME key as the worker needs it: certificate, wrapped key, passphrase. */
export interface SmimeKeyRequest {
  certPem: string;
  encryptedPrivateBundle: string;
  passphrase: string;
}
/**
 * `exportPkcs12` result: the certificate and key as a PKCS#12 (`.p12`) file,
 * base64. The key bag is encrypted with the key's passphrase and the file carries
 * a password-integrity MAC (HMAC-SHA-256) under the same passphrase.
 */
export interface ExportPkcs12Result {
  p12Base64: string;
}
/** `certificateRequest` result: a PKCS#10 request (PEM) for a certificate authority. */
export interface CertificateRequestResult {
  csrPem: string;
}
/** `attachIssuedCert` argument: one certificate (PEM text or DER) for a held key. */
export interface AttachIssuedCertRequest {
  certBytes: Uint8Array;
  encryptedPrivateBundle: string;
  passphrase: string;
}
/** `attachIssuedCert` result, read from the certificate. Rejects if it is for another key. */
export interface AttachIssuedCertResult {
  certPem: string;
  fingerprint: string;
  addresses: string[];
  algorithm: string;
  expiresAt: string;
  /** Issuer and subject are the same name (no certificate authority behind it). */
  selfIssued: boolean;
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

/** `exportBackup` argument/result (Autocrypt Setup Message; OpenPGP keys only). */
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
  /** An own S/MIME key as a PKCS#12 file for another mail program. */
  exportPkcs12(req: SmimeKeyRequest): Promise<ExportPkcs12Result>;
  /** A certification request for an own S/MIME key. */
  certificateRequest(req: SmimeKeyRequest): Promise<CertificateRequestResult>;
  /** Check a certificate issued for an own S/MIME key before it replaces the held one. */
  attachIssuedCert(req: AttachIssuedCertRequest): Promise<AttachIssuedCertResult>;
  importArmored(req: ImportArmoredRequest): Promise<ImportArmoredResult>;
  exportPublic(req: { keyRef: KeyRef }): Promise<string>;
  exportBackup(req: ExportBackupRequest): Promise<ExportBackupResult>;
  /** Decrypt into the worker session cache; returns a key ref. */
  unlockKey(req: UnlockKeyRequest): Promise<KeyRef>;
  /** `zeroize` + drop the cached private key for `keyRef` (also on timeout). */
  lockKey(req: { keyRef: KeyRef }): Promise<void>;
}
