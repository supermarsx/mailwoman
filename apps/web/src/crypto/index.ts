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
  CryptoWorkerApi,
  DecryptResult,
  EncryptResult,
  ExportBackupResult,
  GenerateKeyResult,
  ImportArmoredResult,
  ImportPkcs12Result,
  KeyRef,
  SignResult,
} from '../contracts/crypto.ts';
import type { SignatureVerdict } from '../api/security-types.ts';

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
    async generateKey(): Promise<GenerateKeyResult> {
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
  };
}

/**
 * The process-wide crypto worker accessor. e0 returns the stub; e8 lazy-loads
 * (dynamic `import()`, off the login→inbox critical path, plan risk #12) the real
 * wasm-pack worker from `worker.ts` and returns that instead.
 */
let instance: CryptoWorkerApi | null = null;
export function getCryptoWorker(): CryptoWorkerApi {
  instance ??= createStubCryptoWorker();
  return instance;
}

/** Reset the cached worker (tests). */
export function __resetCryptoWorker(): void {
  instance = null;
}
