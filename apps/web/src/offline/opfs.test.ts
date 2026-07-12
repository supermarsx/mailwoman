import { describe, it, expect, beforeAll, vi } from 'vitest';
import { webcrypto } from 'node:crypto';
import type { Email } from '../api/jmap-types.ts';
import { opfsMessagePath } from '../contracts/offline.ts';
import { generateProfileKey } from './crypto.ts';
import { EncryptedCache, memoryBackend } from './opfs.ts';

beforeAll(() => {
  if (globalThis.crypto?.subtle === undefined) {
    vi.stubGlobal('crypto', webcrypto);
  }
});

function email(id: string, subject: string): Email {
  return {
    id,
    mailboxIds: { inbox: true },
    from: [{ name: 'A', email: 'a@x.org' }],
    to: [{ name: null, email: 'b@y.org' }],
    subject,
    receivedAt: '2026-07-12T00:00:00Z',
    preview: 'preview text',
  };
}

describe('EncryptedCache', () => {
  it('round-trips a single message through the encrypted store', async () => {
    const backend = memoryBackend();
    const cache = new EncryptedCache(backend, await generateProfileKey());
    const msg = email('m1', 'hello');
    await cache.putMessage('acct1', 'm1', msg);
    expect(await cache.getMessage('acct1', 'm1')).toEqual(msg);
  });

  it('round-trips a header slice and a search slice', async () => {
    const cache = new EncryptedCache(memoryBackend(), await generateProfileKey());
    const headers = [email('m1', 'one'), email('m2', 'two')];
    await cache.putHeaders('acct1', 'inbox', headers);
    await cache.putSearchSlice('acct1', headers);
    expect(await cache.getHeaders('acct1', 'inbox')).toEqual(headers);
    expect(await cache.getSearchSlice('acct1')).toEqual(headers);
  });

  it('stores ciphertext, never plaintext JSON, at the contract path', async () => {
    const backend = memoryBackend();
    const cache = new EncryptedCache(backend, await generateProfileKey());
    await cache.putMessage('acct1', 'm1', email('m1', 'secret-subject'));
    const raw = await backend.read(opfsMessagePath('acct1', 'm1'));
    expect(raw).not.toBeNull();
    const asText = new TextDecoder().decode(raw!);
    expect(asText).not.toContain('secret-subject');
  });

  it('returns null for an uncached entry', async () => {
    const cache = new EncryptedCache(memoryBackend(), await generateProfileKey());
    expect(await cache.getMessage('acct1', 'missing')).toBeNull();
    expect(await cache.getHeaders('acct1', 'nope')).toBeNull();
  });
});
