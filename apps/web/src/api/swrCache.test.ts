import { describe, it, expect, vi } from 'vitest';
import { createSwrCache, scopedKey } from './swrCache.ts';

function deferred<T>() {
  let resolve!: (v: T) => void;
  let reject!: (e: unknown) => void;
  const promise = new Promise<T>((res, rej) => { resolve = res; reject = rej; });
  return { promise, resolve, reject };
}

describe('swrCache (t99)', () => {
  it('serves a fresh value without asking again, and shares one request between callers', async () => {
    let now = 0;
    const cache = createSwrCache<number>({ freshMs: 1000, maxAgeMs: 5000, now: () => now });
    const fetcher = vi.fn(async () => 1);
    const [a, b] = await Promise.all([cache.get('k', fetcher), cache.get('k', fetcher)]);
    expect([a, b]).toEqual([1, 1]);
    now = 999;
    expect(await cache.get('k', fetcher)).toBe(1);
    expect(fetcher).toHaveBeenCalledTimes(1);
  });

  it('answers a stale value at once and refreshes it in the background, telling subscribers', async () => {
    let now = 0;
    const cache = createSwrCache<string>({ freshMs: 1000, maxAgeMs: 5000, now: () => now });
    await cache.get('k', async () => 'old');
    const heard: string[] = [];
    cache.subscribe('k', (v) => heard.push(v));
    now = 2000;
    const next = deferred<string>();
    const fetcher = vi.fn(() => next.promise);
    expect(await cache.get('k', fetcher)).toBe('old');
    expect(fetcher).toHaveBeenCalledTimes(1);
    // A second stale read while the refresh runs does not start another.
    expect(await cache.get('k', fetcher)).toBe('old');
    expect(fetcher).toHaveBeenCalledTimes(1);
    next.resolve('new');
    await next.promise;
    await Promise.resolve();
    expect(heard).toEqual(['new']);
    expect(cache.peek('k')).toBe('new');
  });

  it('fetches again once a value is older than maxAge', async () => {
    let now = 0;
    const cache = createSwrCache<number>({ freshMs: 10, maxAgeMs: 100, now: () => now });
    await cache.get('k', async () => 1);
    now = 100;
    expect(cache.peek('k')).toBeUndefined();
    expect(await cache.get('k', async () => 2)).toBe(2);
  });

  it('does not cache a failure', async () => {
    const cache = createSwrCache<number>({ freshMs: 1000, maxAgeMs: 5000 });
    await expect(cache.get('k', async () => { throw new Error('down'); })).rejects.toThrow('down');
    expect(await cache.get('k', async () => 3)).toBe(3);
  });

  it('invalidate drops the value, and a read in flight before it is not stored', async () => {
    const cache = createSwrCache<string>({ freshMs: 1000, maxAgeMs: 5000 });
    const slow = deferred<string>();
    const pending = cache.get('k', () => slow.promise);
    cache.invalidate('k');
    slow.resolve('before-write');
    expect(await pending).toBe('before-write');
    expect(cache.peek('k')).toBeUndefined();
    expect(await cache.get('k', async () => 'after-write')).toBe('after-write');
  });

  it('set (a write answer) wins over a read that was in flight', async () => {
    const cache = createSwrCache<string>({ freshMs: 1000, maxAgeMs: 5000 });
    const slow = deferred<string>();
    void cache.get('k', () => slow.promise);
    cache.set('k', 'written');
    slow.resolve('read');
    await slow.promise;
    await Promise.resolve();
    expect(cache.peek('k')).toBe('written');
  });

  it('keeps each account apart and invalidates by predicate', async () => {
    const cache = createSwrCache<string>({ freshMs: 1000, maxAgeMs: 5000 });
    const a = scopedKey('folders', 'account-a', 'auth-a');
    const b = scopedKey('folders', 'account-b', 'auth-b');
    await cache.get(a, async () => 'A');
    await cache.get(b, async () => 'B');
    expect(cache.peek(a)).toBe('A');
    expect(cache.peek(b)).toBe('B');
    cache.invalidate((key) => key.includes('account-a'));
    expect(cache.peek(a)).toBeUndefined();
    expect(cache.peek(b)).toBe('B');
    cache.clear();
    expect(cache.size()).toBe(0);
  });

  it('scopedKey cannot be forged by a part containing the separator', () => {
    expect(scopedKey('s', 'a|b', 'c')).not.toBe(scopedKey('s', 'a', 'b|c'));
  });
});
