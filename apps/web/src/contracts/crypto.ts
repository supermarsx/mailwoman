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

import {
  CAP_CRYPTO,
  CAP_SECURITY,
  type CryptoKey,
  type KeyKind,
} from "../api/crypto-types.ts";
import type { SignatureVerdict } from "../api/security-types.ts";

/** The crypto/security capability URNs, in `JmapRequest.using` order (§2.2). */
export const CRYPTO_CAPABILITIES = [CAP_CRYPTO, CAP_SECURITY] as const;

/** Keyring method names (§2.2). */
export type CryptoKeyMethod =
  | "CryptoKey/get"
  | "CryptoKey/set"
  | "CryptoKey/query"
  | "CryptoKey/changes"
  | "CryptoKey/lookup"
  | "CryptoKey/setTrust";

/** Verdict / sender-control / mail-rule / DLP method names (§2.2). */
export type SecurityMethod =
  | "SecurityVerdict/get"
  | "SenderControl/set"
  | "MailRule/get"
  | "MailRule/set"
  | "MailRule/changes"
  | "Dlp/getRules"
  | "Dlp/scan";

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
  keyType?:
    "rsa2048" | "rsa3072" | "rsa4096" | "ecdsa-p256" | "ecdsa-p384" | "ed25519";
  /**
   * PGP only: `'v6'` (default; Ed25519 + X25519, RFC 9580) or `'compatible'` (a v4
   * EdDSA + Curve25519 key that GnuPG 2.2–2.4 read).
   */
  profile?: KeyProfile;
  /** PGP only: seconds after creation when the key expires; absent = never. */
  expiresInSecs?: number;
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
  /** S/MIME capabilities, in recipient order; unknown recipients use CBC. */
  recipientGcm?: (boolean | null)[];
  cipher?: "auto" | "cbc" | "gcm";
  signWithCertPem?: string;
}
export interface EncryptResult {
  armoredCiphertext: string;
  encryptedSubjectApplied: boolean;
  /**
   * PGP: 2 (SEIPDv2, AES-256/OCB) when every recipient is a v6 key or advertises
   * it, else 1 (SEIPDv1, readable by v4-only programs such as GnuPG 2.4).
   */
  seipdVersion?: 1 | 2;
  smimeType?: "enveloped-data" | "authEnveloped-data";
}

/** `decrypt` argument. */
export interface ReaderSanitizePolicy {
  blockTrackers: boolean;
  cleanLinks: boolean;
  /** Explicit consent/policy: show the allowlisted remainder, never raw HTML. */
  allowSanitizedHtml?: boolean;
}
export interface DecryptRequest {
  /** Reader-only policy, consumed before the WASM crypto call. */
  readerPolicy?: ReaderSanitizePolicy;
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
  /** PGP: every candidate signer key (all of the sender's keys in the keyring). */
  signerPublicKeys?: string[];
}
/**
 * `decrypt` result — the plaintext is sanitized IN-WORKER via the mw-sanitize
 * wasm build before it is returned (plan §1.3). Exactly one of `plaintextHtml` /
 * `plaintextText` is present.
 */
export interface DecryptResult {
  plaintextHtml?: string;
  /** Already allowlist-sanitized alternative for explicit reader consent. Never raw HTML. */
  sanitizedHtmlAlternative?: string;
  plaintextText?: string;
  subject?: string;
  signature: SignatureVerdict;
  /** The full signature state (`signature.status` is derived from it). */
  verification?: Verification;
  /** PGP: the message used SEIPDv2. */
  aead?: boolean;
}

/** `sign` argument. */
export interface SignRequest {
  kind: KeyKind;
  data: string;
  encryptedPrivateBundle: string;
  passphrase: string;
  detached: boolean;
  certPem?: string;
  chain?: string[];
  pss?: boolean;
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
  /** PGP: further candidate signer keys. */
  signerPublicKeys?: string[];
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

// ── OpenPGP core (t32-e37; Rust: crates/mw-crypto/src/pgp*, mime.rs, verdict.rs) ──

/** PGP key generation profile (see `GenerateKeyRequest.profile`). */
export type KeyProfile = "v6" | "compatible";

/**
 * Signature states shared by OpenPGP and S/MIME (`crates/mw-crypto/src/verdict.rs`).
 * OpenPGP has no chain: the worker reports `valid` for a good signature by a usable
 * key and the caller lowers it to "valid, unknown or untrusted key" from keyring
 * trust — no green state without a trusted key.
 */
export type VerificationState =
  | "valid"
  | "untrusted"
  | "unknown-key"
  | "key-expired"
  | "not-yet-valid"
  | "revoked"
  | "revocation-unchecked"
  | "address-mismatch"
  | "bad-signature"
  | "not-signed"
  | "partial";

/** A signature check with its full state. */
export interface Verification {
  state: VerificationState;
  /**
   * Why, when the state alone does not say: for `bad-signature` one of
   * `tampered` | `weak-hash` | `not-a-signing-key` | `unbound-key` | `malformed`.
   */
  detail: string | null;
  /** The frozen four-state verdict, `status` derived from `state`. */
  signature: SignatureVerdict;
  /** Primary key fingerprint (PGP) or certificate SHA-256 (S/MIME), upper hex. */
  signerFingerprint: string | null;
  signerAddresses: string[];
  /** RFC 3339 signing time from the signature. */
  signedAt: string | null;
}

/** State of a key or one of its parts, judged from verified self-signatures. */
export type PgpKeyState =
  "valid" | "expired" | "revoked" | "invalid" | "not-yet-valid";

export type PgpKeyFlag =
  | "certify"
  | "sign"
  | "encrypt-communications"
  | "encrypt-storage"
  | "authenticate";

export type RevocationReason =
  "compromised" | "superseded" | "retired" | "unspecified";

export interface PgpRevocation {
  at: string;
  reason: RevocationReason | "user-id-invalid";
  /** A hard revocation voids every signature the key made; a soft one only later ones. */
  hard: boolean;
  message: string;
}

export interface PgpUserId {
  value: string;
  address: string | null;
  primary: boolean;
  state: "valid" | "revoked" | "invalid";
}

export interface PgpSubkeyInfo {
  fingerprint: string;
  keyId: string;
  algorithm: string;
  createdAt: string;
  expiresAt: string | null;
  state: PgpKeyState;
  flags: PgpKeyFlag[];
  revocation: PgpRevocation | null;
}

/** `inspectKey` result. */
export interface PgpKeyInfo {
  fingerprint: string;
  keyId: string;
  version: number;
  /** `ed25519`, `x25519`, `eddsa-legacy`, `rsa-3072`, `ecdsa-p256`, ... */
  algorithm: string;
  createdAt: string;
  expiresAt: string | null;
  state: PgpKeyState;
  flags: PgpKeyFlag[];
  revocation: PgpRevocation | null;
  /** The key is v6 or advertises SEIPDv2 (AEAD). */
  seipdV2: boolean;
  userIds: PgpUserId[];
  subkeys: PgpSubkeyInfo[];
  canEncrypt: boolean;
  canSign: boolean;
  hasSecret: boolean;
}

/** `verifyPgp`: detached when `data` is given, else `signature` is cleartext-signed. */
export interface VerifyPgpRequest {
  data?: string | Uint8Array;
  signature: string;
  signerPublicKeys: string[];
}
export interface VerifyPgpResult {
  verification: Verification;
  /** Cleartext signatures: the signed text, un-escaped. */
  signedText?: string;
}

/** A locked own key plus its passphrase, for key management. */
export interface PgpKeyOpRequest {
  encryptedPrivateBundle: string;
  passphrase: string;
}
export interface RevokeKeyRequest extends PgpKeyOpRequest {
  reason?: RevocationReason;
  message?: string;
}
/** The key after a management operation (publish the public half again). */
export interface ManagedKey {
  publicKeyArmored: string;
  encryptedPrivateBundle: string;
}

/** The header fields of a message as `[name, value]` pairs. */
export type HeaderPairs = [string, string][];

/** `pgpMimeEncrypt` / `pgpMimeSign` argument. */
export interface PgpMimeBuildRequest {
  /** The body entity (headers + body) as composed, attachments included. */
  payload: string | Uint8Array;
  /** The message header fields; the protected ones go inside (RFC 9788). */
  headers: HeaderPairs;
  /** Encrypt only: recipients' armored keys (include the sender's own key). */
  recipientPublicKeys?: string[];
  signWithKeyRef?: KeyRef;
  encryptedPrivateBundle?: string;
  passphrase?: string;
  /** Encrypt only: `[address, armoredKey]` pairs written as `Autocrypt-Gossip`. */
  gossip?: HeaderPairs;
}
/** A built PGP/MIME entity the server places as the message body. */
export interface BuiltEntity {
  /** `Content-Type` header, blank line, body. */
  entity: string;
  contentType: string;
  /** What the outer `Subject:` says: `[...]` when the real one is encrypted. */
  outerSubject: string | null;
  seipdVersion: 1 | 2 | null;
}

/** RFC 9788 header protection read back from a payload. */
export interface ProtectedHeaders {
  headers: HeaderPairs;
  scheme: "cipher" | "clear" | "v1";
  /** `HP-Outer`: what the outer header said for each obscured field. */
  outer: HeaderPairs;
  /** The payload with the legacy display element removed. */
  entity: string;
}

export interface AutocryptHeader {
  addr: string;
  preferEncrypt: boolean;
  key: CryptoKey;
}

/**
 * `pgpMimeOpen` / `pgpMimeVerify` result. `entity` is untrusted MIME: every
 * `text/html` part in it goes through the in-worker sanitizer before display.
 */
export interface OpenedMessage {
  entity: string;
  protected: ProtectedHeaders | null;
  verification: Verification;
  encrypted: boolean;
  aead: boolean;
  /** Only part of the message is signed; `verification` covers that part. */
  partial: boolean;
  gossip: AutocryptHeader[];
}
export interface PgpMimeOpenRequest {
  raw: string | Uint8Array;
  encryptedPrivateBundle?: string;
  passphrase?: string;
  signerPublicKeys?: string[];
}

export type InlineSegment =
  | { kind: "text"; text: string }
  | { kind: "message"; armored: string }
  | { kind: "signed"; armored: string }
  | { kind: "publicKey"; armored: string };

/** `analyzeMessage` result: where the signed or encrypted part is. */
export interface AnalyzeMessageResult {
  kind:
    | "pgp-encrypted"
    | "pgp-signed"
    | "smime-signed"
    | "smime-opaque"
    | "inline-pgp"
    | "none";
  partial: boolean;
  /** Child indexes from the root to the part. */
  path: number[];
  smimeType?: string;
  micalg?: string;
  segments?: InlineSegment[];
}

/** What Autocrypt knows about one correspondent (Unix seconds). */
export interface AutocryptPeerState {
  lastSeen: number;
  autocryptTimestamp: number | null;
  hasAutocryptKey: boolean;
  hasGossipKey: boolean;
  preferEncrypt: boolean;
}
export type AutocryptRecommendation =
  "disable" | "discourage" | "available" | "encrypt";

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
  sanitizeReader(req: ReaderSanitizePolicy & { html: string }): Promise<string>;
  verifySmime(req: VerifySmimeRequest): Promise<SmimeVerification>;
  smimeRootSetInfo(req: Record<string, never>): Promise<SmimeRootSetInfo>;
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
  attachIssuedCert(
    req: AttachIssuedCertRequest,
  ): Promise<AttachIssuedCertResult>;
  importArmored(req: ImportArmoredRequest): Promise<ImportArmoredResult>;
  exportPublic(req: { keyRef: KeyRef }): Promise<string>;
  exportBackup(req: ExportBackupRequest): Promise<ExportBackupResult>;
  /** Decrypt into the worker session cache; returns a key ref. */
  unlockKey(req: UnlockKeyRequest): Promise<KeyRef>;
  /** `zeroize` + drop the cached private key for `keyRef` (also on timeout). */
  lockKey(req: { keyRef: KeyRef }): Promise<void>;

  // ── OpenPGP core (t32-e37) ──
  /** A key's version, algorithm, times, state, flags, User IDs and subkeys. */
  inspectKey(req: { armored: string }): Promise<PgpKeyInfo>;
  /** Detached or cleartext PGP verification with the full state. */
  verifyPgp(req: VerifyPgpRequest): Promise<VerifyPgpResult>;
  revokeKey(req: RevokeKeyRequest): Promise<ManagedKey>;
  /** A revocation certificate to keep aside (armored). */
  revocationCertificate(
    req: RevokeKeyRequest,
  ): Promise<{ certificate: string }>;
  applyRevocationCertificate(req: {
    keyArmored: string;
    certificate: string;
  }): Promise<{ publicKeyArmored: string }>;
  /** New expiry (Unix seconds, `null` = never) for the key and its subkeys. */
  setKeyExpiry(
    req: PgpKeyOpRequest & { expiresAt: number | null },
  ): Promise<ManagedKey>;
  addSubkey(
    req: PgpKeyOpRequest & {
      purpose: "encrypt" | "sign";
      expiresAt?: number | null;
    },
  ): Promise<ManagedKey>;
  revokeSubkey(
    req: RevokeKeyRequest & { subkeyFingerprint: string },
  ): Promise<ManagedKey>;
  changePassphrase(req: {
    encryptedPrivateBundle: string;
    oldPassphrase: string;
    newPassphrase: string;
  }): Promise<{ encryptedPrivateBundle: string }>;
  /** Autocrypt Setup Message backup under a fresh numeric setup code. */
  exportBackupFull(req: {
    encryptedPrivateBundle: string;
    from: string;
  }): Promise<{ setupCode: string; payload: string; mime: string }>;
  restoreBackup(req: {
    payload: string;
    setupCode: string;
  }): Promise<{ encryptedPrivateBundle: string }>;
  /** RFC 3156 `multipart/encrypted` with RFC 9788 header protection inside. */
  pgpMimeEncrypt(req: PgpMimeBuildRequest): Promise<BuiltEntity>;
  /** RFC 3156 `multipart/signed` (`hp="clear"`). */
  pgpMimeSign(req: PgpMimeBuildRequest): Promise<BuiltEntity>;
  pgpMimeOpen(req: PgpMimeOpenRequest): Promise<OpenedMessage>;
  pgpMimeVerify(req: PgpMimeOpenRequest): Promise<OpenedMessage>;
  analyzeMessage(req: {
    raw: string | Uint8Array;
  }): Promise<AnalyzeMessageResult>;
  /** An `Autocrypt:` (or, with `gossip`, `Autocrypt-Gossip:`) header value. */
  autocryptHeader(req: {
    publicKeyArmored: string;
    addr: string;
    preferEncrypt?: boolean;
    gossip?: boolean;
  }): Promise<string>;
  parseAutocrypt(req: {
    value: string;
    gossip?: boolean;
  }): Promise<AutocryptHeader>;
  autocryptRecommend(req: {
    peers: (AutocryptPeerState | null)[];
    ownPreferEncrypt: boolean;
  }): Promise<{
    perRecipient: AutocryptRecommendation[];
    message: AutocryptRecommendation;
  }>;
  /** Keys in an attachment (`application/pgp-keys`, `.asc`), or `null` if it is not one. */
  detectAttachedKeys(req: {
    contentType: string;
    filename?: string;
    bytes: string | Uint8Array;
  }): Promise<CryptoKey[] | null>;
  /** `[advanced, direct]` WKD URLs for an address. */
  wkdUrls(req: { email: string }): Promise<[string, string]>;
}

export interface SmimeRootSetInfo {
  snapshot: string;
  count: number;
  source: string;
  sourceSha256: string;
}
export interface SmimeRevocationResult {
  certFingerprint: string;
  status: "good" | "revoked" | "unknown";
  source: string;
  thisUpdate: string | null;
  nextUpdate: string | null;
  revokedAt: string | null;
  reason: string | null;
}
export interface VerifySmimeRequest {
  cms: Uint8Array;
  content?: string | Uint8Array;
  sender?: string;
  extraCerts?: string[];
  userTrust?: string[];
  adminTrust?: string[];
  mozilla?: boolean;
  revocation?: SmimeRevocationResult[];
  hardFail?: boolean;
  localCaseInsensitive?: boolean;
}
export interface SmimeVerification {
  verification: Verification;
  chain: {
    fingerprint: string;
    subject: string;
    issuer: string;
    notBefore: string;
    notAfter: string;
    algorithm: string;
    addresses: string[];
    certPem: string;
  }[];
  anchor: "user" | "admin" | "mozilla" | null;
  revocationQueries: {
    certFingerprint: string;
    serial: string;
    issuerDer: string;
    ocspUrls: string[];
    crlUrls: string[];
  }[];
  capabilities: string[];
  content: number[] | null;
  certificates: string[];
}
