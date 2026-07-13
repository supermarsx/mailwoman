// The client-side private-key vault (plan §2.5 / §1.2) — the passphrase-wrapped
// store for own private keys. Private material lives ONLY here + in the crypto
// worker session cache; the server holds only the opaque `encryptedPrivateBackup`
// blob (for cross-device restore), never plaintext (plan §1.2 / risk #4).
//
// The at-rest wrapping reuses the V2 OPFS AES-GCM primitive in
// `apps/web/src/offline/crypto.ts` (`encryptBytes`/`decryptBytes`) with a key
// derived from the user's key passphrase (Argon2/PBKDF2 — e8 picks the KDF). e0
// authors this interface + an in-memory stub so e2 (key-mgmt) can build against
// it; e8 backs it with OPFS/IndexedDB persistence + the real KDF.

/** An opaque, passphrase-wrapped private-key bundle (never leaves the client
 *  except as `CryptoKey.encryptedPrivateBackup`, the same opaque bytes). */
export type EncryptedPrivateBundle = string;

/** A stored vault entry: the opaque wrapped bundle keyed by the key fingerprint. */
export interface VaultEntry {
  fingerprint: string;
  kind: 'pgp' | 'smime';
  encryptedPrivateBundle: EncryptedPrivateBundle;
  addresses: string[];
}

/**
 * The private-key vault (§2.5). All values are opaque wrapped bundles; the vault
 * never holds unwrapped private material (unwrapping happens transiently in the
 * crypto worker, zeroized on lock). e8 persists this to OPFS/IndexedDB.
 */
export interface KeyVault {
  /** Store (or replace) a wrapped private bundle for a key. */
  put(entry: VaultEntry): Promise<void>;
  /** Fetch the wrapped bundle for a fingerprint, or `null`. */
  get(fingerprint: string): Promise<VaultEntry | null>;
  /** List all stored own-key fingerprints. */
  list(): Promise<VaultEntry[]>;
  /** Remove a key from the vault. */
  remove(fingerprint: string): Promise<void>;
}

/**
 * An in-memory [`KeyVault`] stub (e0) — non-persistent, for component tests +
 * the pre-e8 build. e8 swaps in the OPFS/IndexedDB-backed, passphrase-wrapped
 * implementation reusing `offline/crypto.ts`.
 */
export function createInMemoryVault(): KeyVault {
  const entries = new Map<string, VaultEntry>();
  return {
    async put(entry: VaultEntry): Promise<void> {
      entries.set(entry.fingerprint, entry);
    },
    async get(fingerprint: string): Promise<VaultEntry | null> {
      return entries.get(fingerprint) ?? null;
    },
    async list(): Promise<VaultEntry[]> {
      return [...entries.values()];
    },
    async remove(fingerprint: string): Promise<void> {
      entries.delete(fingerprint);
    },
  };
}
