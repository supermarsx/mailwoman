// IndexedDB backing for the offline profile key (`mw-keys`) + outbound queue
// (`mw-outbox`), per the frozen contract store names. Thin wrapper over `idb`;
// the rest of offline/** talks to the `KeyStore` / `OutboxStore` interfaces so
// unit tests inject in-memory fakes and never need a real IndexedDB.

import { openDB, type IDBPDatabase } from 'idb';
import { IDB_KEYS_STORE, IDB_OUTBOX_STORE, type OutboundItem } from '../contracts/offline.ts';
import type { KeyStore } from './crypto.ts';
import type { OutboxStore } from './outbox.ts';

const DB_NAME = 'mailwoman';
const DB_VERSION = 1;

/** Open (or upgrade) the offline DB, creating both contract stores. */
export function openOfflineDb(): Promise<IDBPDatabase> {
  return openDB(DB_NAME, DB_VERSION, {
    upgrade(db) {
      if (!db.objectStoreNames.contains(IDB_KEYS_STORE)) {
        // Out-of-line keys: the CryptoKey is stored under an explicit id.
        db.createObjectStore(IDB_KEYS_STORE);
      }
      if (!db.objectStoreNames.contains(IDB_OUTBOX_STORE)) {
        db.createObjectStore(IDB_OUTBOX_STORE, { keyPath: 'id' });
      }
    },
  });
}

/** Is IndexedDB usable here? (Absent under the jsdom unit-test environment.) */
export function idbAvailable(): boolean {
  return typeof indexedDB !== 'undefined';
}

export function idbKeyStore(dbp: Promise<IDBPDatabase> = openOfflineDb()): KeyStore {
  return {
    async get(id) {
      const value = await (await dbp).get(IDB_KEYS_STORE, id);
      return value as CryptoKey | undefined;
    },
    async put(id, key) {
      await (await dbp).put(IDB_KEYS_STORE, key, id);
    },
  };
}

export function idbOutboxStore(dbp: Promise<IDBPDatabase> = openOfflineDb()): OutboxStore {
  return {
    async add(item) {
      await (await dbp).put(IDB_OUTBOX_STORE, item);
    },
    async put(item) {
      await (await dbp).put(IDB_OUTBOX_STORE, item);
    },
    async all() {
      const items = (await (await dbp).getAll(IDB_OUTBOX_STORE)) as OutboundItem[];
      return items.sort((a, b) => a.createdAt - b.createdAt);
    },
    async delete(id) {
      await (await dbp).delete(IDB_OUTBOX_STORE, id);
    },
  };
}
