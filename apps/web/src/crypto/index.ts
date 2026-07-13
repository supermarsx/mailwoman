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
    generateKey: (r) => get().generateKey(r),
    encrypt: (r) => get().encrypt(r),
    decrypt: (r) => get().decrypt(r),
    sign: (r) => get().sign(r),
    verify: (r) => get().verify(r),
    importPkcs12: (r) => get().importPkcs12(r),
    importArmored: (r) => get().importArmored(r),
    exportPublic: (r) => get().exportPublic(r),
    exportBackup: (r) => get().exportBackup(r),
    unlockKey: (r) => get().unlockKey(r),
    lockKey: (r) => get().lockKey(r),
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
