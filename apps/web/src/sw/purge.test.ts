// @vitest-environment node
import { afterEach, describe, expect, it, vi } from 'vitest';
import { purgeShellCaches } from './purge.ts';
import { FakeCacheStorage } from './swHarness.ts';

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('purgeShellCaches', () => {
  it('deletes every mw-* cache and leaves other caches alone', async () => {
    const storage = new FakeCacheStorage('https://mail.example.com/');
    await storage.open('mw-shell-v1');
    await storage.open('mw-shell-v0');
    await storage.open('someone-elses');
    vi.stubGlobal('caches', storage);
    // Precondition: the caches the purge is about to remove are really there.
    expect(await storage.keys()).toEqual(['mw-shell-v1', 'mw-shell-v0', 'someone-elses']);

    await purgeShellCaches();

    expect(await storage.keys()).toEqual(['someone-elses']);
  });

  it('is a no-op where the Cache API does not exist', async () => {
    vi.stubGlobal('caches', undefined);
    await expect(purgeShellCaches()).resolves.toBeUndefined();
  });

  it('does not reject when Cache Storage throws', async () => {
    vi.stubGlobal('caches', {
      keys: async () => {
        throw new DOMException('blocked', 'SecurityError');
      },
    });
    await expect(purgeShellCaches()).resolves.toBeUndefined();
  });
});
