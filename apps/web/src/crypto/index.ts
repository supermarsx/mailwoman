// The crypto module public entry (plan §2.5 / §2.3). Exposes the async
// `CryptoWorkerApi` the app calls for all client-side crypto, plus the private-key
// vault. e0 ships a STUB api (`createStubCryptoWorker`) so e2 (key-mgmt) + e4
// (compose-crypto) can build + component-test the flows ("generate calls the
// worker", "encrypt toggle calls the worker") before the wasm bundle exists; e8
// swaps `getCryptoWorker()` to the real wasm-pack worker in `worker.ts`.
//
// The stub returns clearly-synthetic placeholder values (marked `STUB`) and never
// touches real key material — it exists only to satisfy the interface until e8.

import type {
  AnalyzeMessageResult,
  AttachIssuedCertResult,
  AutocryptHeader,
  AutocryptRecommendation,
  BuiltEntity,
  CertificateRequestResult,
  CryptoWorkerApi,
  DecryptResult,
  EncryptResult,
  ExportBackupResult,
  ExportPkcs12Result,
  GenerateKeyRequest,
  GenerateKeyResult,
  ImportArmoredResult,
  ImportPkcs12Result,
  KeyRef,
  ManagedKey,
  OpenedMessage,
  PgpKeyInfo,
  SignResult,
  Verification,
  VerifyPgpResult,
} from '../contracts/crypto.ts';
import type { CryptoKey } from '../api/crypto-types.ts';
import type { SignatureVerdict } from '../api/security-types.ts';
import { spawnCryptoWorker } from './worker.ts';

export type { CryptoWorkerApi } from '../contracts/crypto.ts';
export { createInMemoryVault, type KeyVault, type VaultEntry } from './vault.ts';

/** A synthetic "none" signature verdict the stub returns (no real verify). */
const STUB_SIGNATURE: SignatureVerdict = {
  kind: 'pgp',
  status: 'none',
  signerKeyId: null,
  algorithm: null,
  keyCreatedAt: null,
  keyExpiresAt: null,
  chainStatus: null,
  revocationStatus: null,
  keyChanged: false,
};

/**
 * A STUB [`CryptoWorkerApi`] (e0). Every method resolves with an obvious
 * placeholder — it performs NO cryptography and holds NO key material. e8
 * replaces this with the wasm-pack-backed worker; the shapes here are the frozen
 * §2.3 contract the replacement must honor.
 */
export function createStubCryptoWorker(): CryptoWorkerApi {
  let refSeq = 0;
  return {
    async sanitizeReader() { throw new Error('Reader sanitization requires the real worker'); },
    async verifySmime() {
      throw new Error('S/MIME verification requires the real crypto worker');
    },
    async smimeRootSetInfo() {
      throw new Error('S/MIME roots require the real crypto worker');
    },
    async generateKey(req: GenerateKeyRequest): Promise<GenerateKeyResult> {
      // Shaped like the real worker's answer for each kind, so a caller that
      // files one kind's fields under the other is caught against the stub too.
      if (req.kind === 'smime') {
        return {
          certPem: '-----BEGIN CERTIFICATE-----\nSTUB\n-----END CERTIFICATE-----',
          fingerprint: 'STUBCERTFINGERPRINT000000000000000000000000000000000000000000000',
          keyId: 'STUBCERTFINGERPR',
          encryptedPrivateBundle: 'STUB_ENCRYPTED_PRIVATE_BUNDLE',
          addresses: ['stub@example.invalid'],
          algorithm: 'rsa-3072',
          expiresAt: new Date(0).toISOString(),
        };
      }
      return {
        publicKeyArmored: '-----BEGIN PGP PUBLIC KEY BLOCK-----\nSTUB\n-----END PGP PUBLIC KEY BLOCK-----',
        fingerprint: 'STUBFINGERPRINT0000000000000000000000000',
        keyId: 'STUBKEYID0000000',
        encryptedPrivateBundle: 'STUB_ENCRYPTED_PRIVATE_BUNDLE',
      };
    },
    async encrypt(): Promise<EncryptResult> {
      return {
        armoredCiphertext: '-----BEGIN PGP MESSAGE-----\nSTUB\n-----END PGP MESSAGE-----',
        encryptedSubjectApplied: false,
      };
    },
    async decrypt(): Promise<DecryptResult> {
      return { plaintextText: 'STUB decrypted body', signature: STUB_SIGNATURE };
    },
    async sign(): Promise<SignResult> {
      return { signatureArmored: '-----BEGIN PGP SIGNATURE-----\nSTUB\n-----END PGP SIGNATURE-----' };
    },
    async verify(): Promise<SignatureVerdict> {
      return STUB_SIGNATURE;
    },
    async importPkcs12(): Promise<ImportPkcs12Result> {
      return {
        certPem: '-----BEGIN CERTIFICATE-----\nSTUB\n-----END CERTIFICATE-----',
        fingerprint: 'STUBFINGERPRINT0000000000000000000000000',
        encryptedPrivateBundle: 'STUB_ENCRYPTED_PRIVATE_BUNDLE',
        addresses: ['stub@example.invalid'],
        algorithm: 'rsa-2048',
        expiresAt: new Date(0).toISOString(),
      };
    },
    async exportPkcs12(): Promise<ExportPkcs12Result> {
      return { p12Base64: btoa('STUB PKCS#12') };
    },
    async certificateRequest(): Promise<CertificateRequestResult> {
      return { csrPem: '-----BEGIN CERTIFICATE REQUEST-----\nSTUB\n-----END CERTIFICATE REQUEST-----' };
    },
    async attachIssuedCert(): Promise<AttachIssuedCertResult> {
      return {
        certPem: '-----BEGIN CERTIFICATE-----\nSTUB ISSUED\n-----END CERTIFICATE-----',
        fingerprint: 'STUBISSUEDFINGERPRINT0000000000000000000000000000000000000000000',
        addresses: ['stub@example.invalid'],
        algorithm: 'rsa-3072',
        expiresAt: new Date(0).toISOString(),
        selfIssued: false,
      };
    },
    async importArmored(): Promise<ImportArmoredResult> {
      return {
        key: {
          id: 'STUB',
          kind: 'pgp',
          isOwn: false,
          addresses: [],
          fingerprint: 'STUBFINGERPRINT0000000000000000000000000',
          keyId: 'STUBKEYID0000000',
          algorithm: 'ed25519',
          createdAt: new Date(0).toISOString(),
          expiresAt: null,
          publicKeyArmored: '-----BEGIN PGP PUBLIC KEY BLOCK-----\nSTUB\n-----END PGP PUBLIC KEY BLOCK-----',
          certPem: null,
          trust: 'unverified',
          autocrypt: false,
          source: 'imported',
          hasPrivate: false,
          encryptedPrivateBackup: null,
          verifiedAt: null,
          keyHistory: [],
        },
      };
    },
    async exportPublic(): Promise<string> {
      return '-----BEGIN PGP PUBLIC KEY BLOCK-----\nSTUB\n-----END PGP PUBLIC KEY BLOCK-----';
    },
    async exportBackup(): Promise<ExportBackupResult> {
      return { autocryptSetupMessage: 'STUB Autocrypt Setup Message' };
    },
    async unlockKey(): Promise<KeyRef> {
      refSeq += 1;
      return `stub-keyref-${refSeq}`;
    },
    async lockKey(): Promise<void> {
      // no-op (nothing cached in the stub)
    },
    async inspectKey(): Promise<PgpKeyInfo> {
      return {
        fingerprint: 'STUBFINGERPRINT0000000000000000000000000',
        keyId: 'STUBKEYID0000000',
        version: 6,
        algorithm: 'ed25519',
        createdAt: new Date(0).toISOString(),
        expiresAt: null,
        state: 'valid',
        flags: ['certify', 'sign'],
        revocation: null,
        seipdV2: true,
        userIds: [],
        subkeys: [],
        canEncrypt: true,
        canSign: true,
        hasSecret: false,
      };
    },
    async verifyPgp(): Promise<VerifyPgpResult> {
      return { verification: STUB_VERIFICATION };
    },
    async revokeKey(): Promise<ManagedKey> {
      return STUB_MANAGED;
    },
    async revocationCertificate(): Promise<{ certificate: string }> {
      return { certificate: '-----BEGIN PGP PUBLIC KEY BLOCK-----\nSTUB\n-----END PGP PUBLIC KEY BLOCK-----' };
    },
    async applyRevocationCertificate(): Promise<{ publicKeyArmored: string }> {
      return { publicKeyArmored: STUB_MANAGED.publicKeyArmored };
    },
    async setKeyExpiry(): Promise<ManagedKey> {
      return STUB_MANAGED;
    },
    async addSubkey(): Promise<ManagedKey> {
      return STUB_MANAGED;
    },
    async revokeSubkey(): Promise<ManagedKey> {
      return STUB_MANAGED;
    },
    async changePassphrase(): Promise<{ encryptedPrivateBundle: string }> {
      return { encryptedPrivateBundle: 'STUB_ENCRYPTED_PRIVATE_BUNDLE' };
    },
    async exportBackupFull(): Promise<{ setupCode: string; payload: string; mime: string }> {
      return { setupCode: '0000-0000-0000-0000-0000-0000-0000-0000-0000', payload: 'STUB', mime: 'STUB' };
    },
    async restoreBackup(): Promise<{ encryptedPrivateBundle: string }> {
      return { encryptedPrivateBundle: 'STUB_ENCRYPTED_PRIVATE_BUNDLE' };
    },
    async pgpMimeEncrypt(): Promise<BuiltEntity> {
      return { entity: 'Content-Type: text/plain\r\n\r\nSTUB', contentType: 'text/plain', outerSubject: '[...]', seipdVersion: 2 };
    },
    async pgpMimeSign(): Promise<BuiltEntity> {
      return { entity: 'Content-Type: text/plain\r\n\r\nSTUB', contentType: 'text/plain', outerSubject: null, seipdVersion: null };
    },
    async pgpMimeOpen(): Promise<OpenedMessage> {
      return STUB_OPENED;
    },
    async pgpMimeVerify(): Promise<OpenedMessage> {
      return STUB_OPENED;
    },
    async analyzeMessage(): Promise<AnalyzeMessageResult> {
      return { kind: 'none', partial: false, path: [] };
    },
    async autocryptHeader(): Promise<string> {
      return 'addr=stub@example.invalid; keydata=STUB';
    },
    async parseAutocrypt(): Promise<AutocryptHeader> {
      throw new Error('STUB: parseAutocrypt has no stub key');
    },
    async autocryptRecommend(req): Promise<{
      perRecipient: AutocryptRecommendation[];
      message: AutocryptRecommendation;
    }> {
      return { perRecipient: req.peers.map(() => 'disable'), message: 'disable' };
    },
    async detectAttachedKeys(): Promise<CryptoKey[] | null> {
      return null;
    },
    async wkdUrls(): Promise<[string, string]> {
      return ['https://openpgpkey.example.invalid/', 'https://example.invalid/'];
    },
  };
}

const STUB_VERIFICATION: Verification = {
  state: 'not-signed',
  detail: null,
  signature: STUB_SIGNATURE,
  signerFingerprint: null,
  signerAddresses: [],
  signedAt: null,
};

const STUB_MANAGED: ManagedKey = {
  publicKeyArmored: '-----BEGIN PGP PUBLIC KEY BLOCK-----\nSTUB\n-----END PGP PUBLIC KEY BLOCK-----',
  encryptedPrivateBundle: 'STUB_ENCRYPTED_PRIVATE_BUNDLE',
};

const STUB_OPENED: OpenedMessage = {
  entity: 'Content-Type: text/plain\r\n\r\nSTUB decrypted body',
  protected: null,
  verification: STUB_VERIFICATION,
  encrypted: true,
  aead: true,
  partial: false,
  gossip: [],
};

/**
 * Whether to use the real wasm-pack worker. The browser build has `Worker`; the
 * unit-test env (vitest/jsdom, `MODE === 'test'`) has neither a `Worker` nor the
 * ability to load a `.wasm`, so it keeps the deterministic stub — the frozen §2.3
 * shapes are identical either way, so the components/slices are unchanged (e2/e4
 * built against the stub; they consume the real worker transparently at runtime).
 */
function useRealWorker(): boolean {
  return typeof Worker !== 'undefined' && import.meta.env.MODE !== 'test';
}

/**
 * A lazy [`CryptoWorkerApi`] that defers spawning the real Worker (and thus loading
 * the ~4.5 MB wasm) until the FIRST crypto call (plan risk #12). Constructed at
 * app startup by the keys slice, it must not touch the wasm on the login→inbox
 * critical path — so the Worker is spawned on demand, then memoized.
 */
function createLazyCryptoWorker(): CryptoWorkerApi {
  let real: CryptoWorkerApi | null = null;
  const get = (): CryptoWorkerApi => (real ??= spawnCryptoWorker());
  return {
    sanitizeReader: (r) => get().sanitizeReader(r),
    verifySmime: (r) => get().verifySmime(r),
    smimeRootSetInfo: (r) => get().smimeRootSetInfo(r),
    generateKey: (r) => get().generateKey(r),
    encrypt: (r) => get().encrypt(r),
    decrypt: (r) => get().decrypt(r),
    sign: (r) => get().sign(r),
    verify: (r) => get().verify(r),
    importPkcs12: (r) => get().importPkcs12(r),
    exportPkcs12: (r) => get().exportPkcs12(r),
    certificateRequest: (r) => get().certificateRequest(r),
    attachIssuedCert: (r) => get().attachIssuedCert(r),
    importArmored: (r) => get().importArmored(r),
    exportPublic: (r) => get().exportPublic(r),
    exportBackup: (r) => get().exportBackup(r),
    unlockKey: (r) => get().unlockKey(r),
    lockKey: (r) => get().lockKey(r),
    inspectKey: (r) => get().inspectKey(r),
    verifyPgp: (r) => get().verifyPgp(r),
    revokeKey: (r) => get().revokeKey(r),
    revocationCertificate: (r) => get().revocationCertificate(r),
    applyRevocationCertificate: (r) => get().applyRevocationCertificate(r),
    setKeyExpiry: (r) => get().setKeyExpiry(r),
    addSubkey: (r) => get().addSubkey(r),
    revokeSubkey: (r) => get().revokeSubkey(r),
    changePassphrase: (r) => get().changePassphrase(r),
    exportBackupFull: (r) => get().exportBackupFull(r),
    restoreBackup: (r) => get().restoreBackup(r),
    pgpMimeEncrypt: (r) => get().pgpMimeEncrypt(r),
    pgpMimeSign: (r) => get().pgpMimeSign(r),
    pgpMimeOpen: (r) => get().pgpMimeOpen(r),
    pgpMimeVerify: (r) => get().pgpMimeVerify(r),
    analyzeMessage: (r) => get().analyzeMessage(r),
    autocryptHeader: (r) => get().autocryptHeader(r),
    parseAutocrypt: (r) => get().parseAutocrypt(r),
    autocryptRecommend: (r) => get().autocryptRecommend(r),
    detectAttachedKeys: (r) => get().detectAttachedKeys(r),
    wkdUrls: (r) => get().wkdUrls(r),
  };
}

/**
 * The process-wide crypto worker accessor. The browser gets the real wasm-pack
 * worker (lazily spawned); unit tests get the deterministic stub. Memoized so the
 * keys slice + compose-crypto share one worker instance.
 */
let instance: CryptoWorkerApi | null = null;
export function getCryptoWorker(): CryptoWorkerApi {
  instance ??= useRealWorker() ? createLazyCryptoWorker() : createStubCryptoWorker();
  return instance;
}

/** Reset the cached worker (tests). */
export function __resetCryptoWorker(): void {
  instance = null;
}
