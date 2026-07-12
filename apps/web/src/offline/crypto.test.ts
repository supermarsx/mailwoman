import { describe, it, expect, beforeAll, vi } from 'vitest';
import { webcrypto } from 'node:crypto';
import { GCM_IV_BYTES } from '../contracts/offline.ts';
import {
  decryptBytes,
  decryptJson,
  encryptBytes,
  encryptJson,
  generateProfileKey,
  getOrCreateProfileKey,
  type KeyStore,
} from './crypto.ts';

// jsdom's Crypto has getRandomValues but no SubtleCrypto; use Node's webcrypto.
beforeAll(() => {
  if (globalThis.crypto?.subtle === undefined) {
    vi.stubGlobal('crypto', webcrypto);
  }
});

function memoryKeyStore(): KeyStore {
  const keys = new Map<string, CryptoKey>();
  return {
    async get(id) {
      return keys.get(id);
    },
    async put(id, key) {
      keys.set(id, key);
    },
  };
}

describe('profile key', () => {
  it('is a non-extractable AES-GCM 256 key', async () => {
    const key = await generateProfileKey();
    expect(key.extractable).toBe(false);
    expect(key.type).toBe('secret');
    const algo = key.algorithm as AesKeyAlgorithm;
    expect(algo.name).toBe('AES-GCM');
    expect(algo.length).toBe(256);
    expect(key.usages).toEqual(expect.arrayContaining(['encrypt', 'decrypt']));
  });

  it('cannot be exported (device-at-rest, not zero-access)', async () => {
    const key = await generateProfileKey();
    await expect(crypto.subtle.exportKey('raw', key)).rejects.toThrow();
  });

  it('generates once then reuses the persisted key', async () => {
    const store = memoryKeyStore();
    const first = await getOrCreateProfileKey(store);
    const second = await getOrCreateProfileKey(store);
    expect(second).toBe(first);
  });
});

describe('encrypt/decrypt round-trip', () => {
  it('round-trips bytes with a 12-byte IV prefix', async () => {
    const key = await generateProfileKey();
    const plaintext = new TextEncoder().encode('the quick brown fox');
    const blob = await encryptBytes(key, plaintext);
    // [IV | ciphertext+tag] — strictly longer than IV + plaintext.
    expect(blob.byteLength).toBeGreaterThan(GCM_IV_BYTES + plaintext.byteLength);
    const back = await decryptBytes(key, blob);
    expect(new TextDecoder().decode(back)).toBe('the quick brown fox');
  });

  it('round-trips JSON', async () => {
    const key = await generateProfileKey();
    const value = { id: 'm1', subject: 'hi', flags: ['$seen'], n: 42 };
    const blob = await encryptJson(key, value);
    expect(await decryptJson(key, blob)).toEqual(value);
  });

  it('produces a fresh IV per call (ciphertexts differ)', async () => {
    const key = await generateProfileKey();
    const a = await encryptBytes(key, new Uint8Array([1, 2, 3]));
    const b = await encryptBytes(key, new Uint8Array([1, 2, 3]));
    expect(Array.from(a)).not.toEqual(Array.from(b));
  });

  it('rejects a blob shorter than the IV', async () => {
    const key = await generateProfileKey();
    await expect(decryptBytes(key, new Uint8Array(GCM_IV_BYTES))).rejects.toThrow(/too short/);
  });

  it('fails to decrypt under a different key', async () => {
    const k1 = await generateProfileKey();
    const k2 = await generateProfileKey();
    const blob = await encryptBytes(k1, new Uint8Array([9, 9, 9]));
    await expect(decryptBytes(k2, blob)).rejects.toThrow();
  });
});
