// Cookie consent: the browser side of the rules. Scanning what the page has stored
// (names only, never values), blocking writes of storage the visitor has not
// allowed, and removing what is already there.
//
// Blocking wraps `Storage.prototype.setItem`, the `document.cookie` setter and
// `IDBFactory.prototype.open`, so the application's own modules need no change:
// a blocked write is dropped as it would be in a browser with storage turned off,
// and the application already tolerates that.

import { basePath } from '../api/basePath.ts';
import { classify, mayStore, type ConsentConfig, type Effective, type StorageKind } from './policy.ts';

export interface Detected { kind: StorageKind; name: string }

const nativeSetItem = typeof Storage === 'undefined' ? null : Storage.prototype.setItem;
const nativeRemoveItem = typeof Storage === 'undefined' ? null : Storage.prototype.removeItem;
const cookieDescriptor = typeof Document === 'undefined' ? undefined : Object.getOwnPropertyDescriptor(Document.prototype, 'cookie');

/** Write to localStorage bypassing the gate (the consent record itself). */
export function rawSet(storage: Storage, key: string, value: string): void {
  try { (nativeSetItem ?? storage.setItem).call(storage, key, value); } catch { /* storage may be unavailable */ }
}

function cookieNames(): string[] {
  try {
    const jar = (cookieDescriptor?.get?.call(document) as string | undefined) ?? document.cookie;
    return jar.split(';').map((part) => part.split('=')[0]!.trim()).filter(Boolean).map((name) => { try { return decodeURIComponent(name); } catch { return name; } });
  } catch { return []; }
}

function storageKeys(storage: () => Storage): string[] {
  try {
    const s = storage();
    const out: string[] = [];
    for (let i = 0; i < s.length; i += 1) { const key = s.key(i); if (key) out.push(key); }
    return out;
  } catch { return []; }
}

/** Names the page has stored now, by kind. Values are never read. */
export async function scanStorage(): Promise<Detected[]> {
  const out: Detected[] = [];
  for (const name of cookieNames()) out.push({ kind: 'cookie', name });
  for (const name of storageKeys(() => localStorage)) out.push({ kind: 'localStorage', name });
  for (const name of storageKeys(() => sessionStorage)) out.push({ kind: 'sessionStorage', name });
  try {
    const dbs = await (indexedDB as IDBFactory & { databases?: () => Promise<{ name?: string }[]> }).databases?.();
    for (const db of dbs ?? []) if (db.name) out.push({ kind: 'indexedDB', name: db.name });
  } catch { /* IndexedDB is optional */ }
  return out;
}

/** The names nothing classifies, deduplicated. */
export function unclassified(items: readonly Detected[], config: ConsentConfig): Detected[] {
  const seen = new Set<string>();
  const out: Detected[] = [];
  for (const item of items) {
    const id = `${item.kind}\u0000${item.name}`;
    if (seen.has(id) || classify(item.kind, item.name, config)) continue;
    seen.add(id);
    out.push(item);
  }
  return out;
}

function expireCookie(name: string): void {
  const set = (value: string): void => {
    try { if (cookieDescriptor?.set) cookieDescriptor.set.call(document, value); else document.cookie = value; } catch { /* ignore */ }
  };
  const base = basePath() || '/';
  for (const path of new Set(['/', base])) set(`${encodeURIComponent(name)}=; Max-Age=0; Path=${path}`);
}

/** Remove everything the page has stored that the visitor has not allowed. Returns what was removed. */
export async function purge(config: ConsentConfig, effective: Effective): Promise<Detected[]> {
  if (!config.enabled) return [];
  const removed: Detected[] = [];
  for (const item of await scanStorage()) {
    if (mayStore(item.kind, item.name, config, effective)) continue;
    removed.push(item);
    try {
      if (item.kind === 'localStorage') (nativeRemoveItem ?? localStorage.removeItem).call(localStorage, item.name);
      else if (item.kind === 'sessionStorage') (nativeRemoveItem ?? sessionStorage.removeItem).call(sessionStorage, item.name);
      else if (item.kind === 'cookie') expireCookie(item.name);
      else if (item.kind === 'indexedDB') indexedDB.deleteDatabase(item.name);
    } catch { /* removal is best effort */ }
  }
  return removed;
}

export interface GateState { config: ConsentConfig; effective: Effective }

/**
 * Block writes of storage the visitor has not allowed. `current()` is read at
 * each write, so a later decision takes effect without installing again.
 * Returns a function that removes the gate (tests).
 */
export function installGate(current: () => GateState | null): () => void {
  const undo: (() => void)[] = [];
  const blocked = (kind: StorageKind, name: string): boolean => {
    const state = current();
    return state !== null && !mayStore(kind, name, state.config, state.effective);
  };
  if (typeof Storage !== 'undefined' && nativeSetItem) {
    const original = Storage.prototype.setItem;
    Storage.prototype.setItem = function setItem(this: Storage, key: string, value: string): void {
      let kind: StorageKind = 'localStorage';
      try { if (this === window.sessionStorage) kind = 'sessionStorage'; } catch { /* sandboxed */ }
      if (blocked(kind, String(key))) return;
      original.call(this, key, value);
    };
    undo.push(() => { Storage.prototype.setItem = original; });
  }
  if (cookieDescriptor?.set && cookieDescriptor.get) {
    const { get, set } = cookieDescriptor;
    Object.defineProperty(Document.prototype, 'cookie', {
      configurable: true,
      enumerable: cookieDescriptor.enumerable ?? true,
      get() { return get.call(this); },
      set(value: string) {
        const name = String(value).split('=')[0]!.trim();
        let decoded = name;
        try { decoded = decodeURIComponent(name); } catch { /* keep the raw name */ }
        // Removing a cookie (an expiry) is never blocked.
        const removal = /(?:^|;\s*)max-age=0|expires=thu, 01 jan 1970/i.test(String(value));
        if (!removal && blocked('cookie', decoded)) return;
        set.call(this, value);
      },
    });
    undo.push(() => { Object.defineProperty(Document.prototype, 'cookie', cookieDescriptor); });
  }
  if (typeof IDBFactory !== 'undefined') {
    const original = IDBFactory.prototype.open;
    IDBFactory.prototype.open = function open(this: IDBFactory, name: string, version?: number): IDBOpenDBRequest {
      if (blocked('indexedDB', String(name))) throw new DOMException('Storage is not allowed by the cookie preferences.', 'SecurityError');
      return original.call(this, name, version);
    };
    undo.push(() => { IDBFactory.prototype.open = original; });
  }
  return () => { for (const fn of undo.reverse()) fn(); };
}
