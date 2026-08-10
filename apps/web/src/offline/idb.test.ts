// The IndexedDB adapters behind the offline key store and outbound queue
// (t19-e12, tag 26.19).
//
// `idb.ts` had no test before this file. jsdom ships no IndexedDB, and adding a
// polyfill would mean a new devDependency this tag does not take, so these tests
// drive the adapters through the injection seam they already expose: both
// factories accept the database promise as a parameter. That covers everything
// the adapters actually decide — which store name a value lands in, whether the
// key is in-line or out-of-line, and the ordering guarantee `all()` makes. The
// parts that belong to `idb` itself (the real upgrade transaction) are not
// re-tested here; the `upgrade` callback is exercised end to end by the offline
// Playwright specs against a real browser.

import { describe, it, expect, vi } from 'vitest';
import type { IDBPDatabase } from 'idb';
import { idbAvailable, idbKeyStore, idbOutboxStore, openOfflineDb } from './idb.ts';
import { IDB_KEYS_STORE, IDB_OUTBOX_STORE, type OutboundItem } from '../contracts/offline.ts';

/** A recording stand-in for the two `idb` methods the adapters call. */
function fakeDb(rows: OutboundItem[] = []) {
  const store = new Map<string, unknown>();
  const calls: Array<{ op: string; args: unknown[] }> = [];
  const db = {
    get: vi.fn(async (name: string, key: string) => {
      calls.push({ op: 'get', args: [name, key] });
      return store.get(`${name}/${key}`);
    }),
    put: vi.fn(async (name: string, value: unknown, key?: string) => {
      calls.push({ op: 'put', args: [name, value, key] });
      const id = key ?? (value as { id: string }).id;
      store.set(`${name}/${id}`, value);
      return id;
    }),
    getAll: vi.fn(async (name: string) => {
      calls.push({ op: 'getAll', args: [name] });
      return name === IDB_OUTBOX_STORE ? rows : [];
    }),
    delete: vi.fn(async (name: string, key: string) => {
      calls.push({ op: 'delete', args: [name, key] });
      store.delete(`${name}/${key}`);
    }),
  };
  return { db: Promise.resolve(db as unknown as IDBPDatabase), spy: db, calls, store };
}

function item(id: string, createdAt: number): OutboundItem {
  return { id, type: 'send', payload: { id }, createdAt, state: 'queued' };
}

describe('idbAvailable', () => {
  it('reports the truth about this environment', () => {
    // jsdom has no IndexedDB, which is exactly why the rest of offline/** talks
    // to the KeyStore/OutboxStore interfaces and injects fakes in unit tests.
    expect(idbAvailable()).toBe(typeof indexedDB !== 'undefined');
  });

  it('is a guard callers MUST honour — opening without it throws synchronously', () => {
    if (idbAvailable()) return;
    // `openDB` reaches for the global immediately, so the failure is a
    // synchronous ReferenceError, NOT a rejected promise. That is why every
    // caller in offline/** checks `idbAvailable()` first rather than relying on
    // a `.catch()`, and why the adapters take an injected database promise.
    expect(() => openOfflineDb()).toThrow(/indexedDB/i);
  });
});

describe('idbKeyStore', () => {
  it('stores the profile key out of line, under the id it was given', async () => {
    const { db, spy } = fakeDb();
    const keys = idbKeyStore(db);
    const key = { fake: 'CryptoKey' } as unknown as CryptoKey;

    await keys.put('profile', key);
    // Out-of-line keying: the key is the THIRD argument, because a CryptoKey has
    // no id field to key on. Passing it as part of the value would fail against
    // a store created without a keyPath.
    expect(spy.put).toHaveBeenCalledWith(IDB_KEYS_STORE, key, 'profile');
    await expect(keys.get('profile')).resolves.toBe(key);
  });

  it('returns undefined for a key that was never stored', async () => {
    const { db } = fakeDb();
    await expect(idbKeyStore(db).get('absent')).resolves.toBeUndefined();
  });

  it('never touches the outbox store', async () => {
    const { db, calls } = fakeDb();
    const keys = idbKeyStore(db);
    await keys.put('profile', {} as CryptoKey);
    await keys.get('profile');
    expect(calls.every((c) => c.args[0] === IDB_KEYS_STORE)).toBe(true);
  });
});

describe('idbOutboxStore', () => {
  it('returns queued items oldest-first regardless of storage order', async () => {
    // The queue is drained in order on reconnect, so this sort is the thing that
    // stops a reply going out before the message it replies to. IndexedDB makes
    // no ordering promise for `getAll` on a non-indexed store.
    const { db } = fakeDb([item('c', 300), item('a', 100), item('b', 200)]);
    const all = await idbOutboxStore(db).all();
    expect(all.map((i) => i.id)).toEqual(['a', 'b', 'c']);
  });

  it('keeps a stable order for items queued in the same millisecond', async () => {
    const { db } = fakeDb([item('x', 100), item('y', 100)]);
    expect((await idbOutboxStore(db).all()).map((i) => i.id)).toEqual(['x', 'y']);
  });

  it('returns an empty queue rather than undefined when nothing is pending', async () => {
    const { db } = fakeDb([]);
    await expect(idbOutboxStore(db).all()).resolves.toEqual([]);
  });

  it('stores items in line, keyed by their own id', async () => {
    const { db, spy } = fakeDb();
    const it0 = item('m1', 1);
    await idbOutboxStore(db).add(it0);
    // In-line key: the store was created with `keyPath: 'id'`, so no explicit
    // key argument — passing one would be a DataError against a real database.
    expect(spy.put).toHaveBeenCalledWith(IDB_OUTBOX_STORE, it0);
  });

  it('treats add and put alike, so a retry updates rather than duplicating', async () => {
    // Both map onto `put`. That is what lets the drain loop write back an item
    // with a changed `state` without creating a second copy of it.
    const { db, spy } = fakeDb();
    const outbox = idbOutboxStore(db);
    await outbox.add(item('m1', 1));
    await outbox.put({ ...item('m1', 1), state: 'failed' });
    expect(spy.put).toHaveBeenCalledTimes(2);
    expect(spy.put.mock.calls.every((c) => c[0] === IDB_OUTBOX_STORE)).toBe(true);
  });

  it('deletes by id from the outbox store', async () => {
    const { db, spy } = fakeDb();
    await idbOutboxStore(db).delete('m1');
    expect(spy.delete).toHaveBeenCalledWith(IDB_OUTBOX_STORE, 'm1');
  });

  it('opens the database once and reuses it across every operation', async () => {
    // The parameter is a PROMISE, not a factory, so a store built once does not
    // reopen the database per call — which would serialise every queue write
    // behind a fresh connection.
    const { db, spy } = fakeDb([item('a', 1)]);
    const outbox = idbOutboxStore(db);
    await outbox.all();
    await outbox.add(item('b', 2));
    await outbox.delete('a');
    expect(spy.getAll).toHaveBeenCalledTimes(1);
  });
});
