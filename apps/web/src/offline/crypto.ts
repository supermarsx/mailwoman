// AES-256-GCM device-at-rest crypto for the OPFS cache (plan §1.8, §2.5).
//
// This is DEVICE-AT-REST protection, NOT zero-access: the key is a
// non-extractable per-profile `CryptoKey` and the server still sends plaintext
// in V2. V6 swaps the key source for the user-derived hierarchy (§9.1).

import { GCM_IV_BYTES } from '../contracts/offline.ts';

/** The single profile key's slot in the `mw-keys` store. */
export const PROFILE_KEY_ID = 'profile-aes-gcm';

/** Persistence for the profile `CryptoKey`. Injected so unit tests avoid IDB. */
export interface KeyStore {
  get(id: string): Promise<CryptoKey | undefined>;
  put(id: string, key: CryptoKey): Promise<void>;
}

/**
 * Generate a fresh AES-GCM 256 key. `extractable: false` is the device-at-rest
 * guarantee — the raw key material can never be exported out of the browser.
 */
export function generateProfileKey(): Promise<CryptoKey> {
  return crypto.subtle.generateKey({ name: 'AES-GCM', length: 256 }, false, ['encrypt', 'decrypt']);
}

/** Fetch the profile key, generating + persisting one on first run. */
export async function getOrCreateProfileKey(store: KeyStore): Promise<CryptoKey> {
  const existing = await store.get(PROFILE_KEY_ID);
  if (existing !== undefined) return existing;
  const key = await generateProfileKey();
  await store.put(PROFILE_KEY_ID, key);
  return key;
}

/** Encrypt bytes → a `[12-byte IV | ciphertext]` blob (contract `GCM_IV_BYTES`). */
export async function encryptBytes(key: CryptoKey, plaintext: Uint8Array): Promise<Uint8Array> {
  const iv = crypto.getRandomValues(new Uint8Array(GCM_IV_BYTES));
  // `plaintext` is always byte-backed at runtime; the cast satisfies the DOM
  // BufferSource requirement (Uint8Array now defaults its buffer to ArrayBufferLike).
  const ct = await crypto.subtle.encrypt({ name: 'AES-GCM', iv }, key, plaintext as BufferSource);
  const out = new Uint8Array(GCM_IV_BYTES + ct.byteLength);
  out.set(iv, 0);
  out.set(new Uint8Array(ct), GCM_IV_BYTES);
  return out;
}

/** Decrypt a `[12-byte IV | ciphertext]` blob produced by `encryptBytes`. */
export async function decryptBytes(key: CryptoKey, blob: Uint8Array): Promise<Uint8Array> {
  if (blob.byteLength <= GCM_IV_BYTES) throw new Error('offline: ciphertext too short for IV');
  const iv = blob.subarray(0, GCM_IV_BYTES);
  const ct = blob.subarray(GCM_IV_BYTES);
  const pt = await crypto.subtle.decrypt(
    { name: 'AES-GCM', iv: iv as BufferSource },
    key,
    ct as BufferSource,
  );
  return new Uint8Array(pt);
}

const textEncoder = new TextEncoder();
const textDecoder = new TextDecoder();

/** Encrypt a JSON-serialisable value. */
export async function encryptJson(key: CryptoKey, value: unknown): Promise<Uint8Array> {
  return encryptBytes(key, textEncoder.encode(JSON.stringify(value)));
}

/** Decrypt a blob written by `encryptJson`. */
export async function decryptJson<T>(key: CryptoKey, blob: Uint8Array): Promise<T> {
  return JSON.parse(textDecoder.decode(await decryptBytes(key, blob))) as T;
}
