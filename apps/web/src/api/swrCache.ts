// Stale-while-revalidate cache for slow-changing reads (t99).
//
// A read through `get(key, fetcher)` answers from memory when it can:
//
//   * younger than `freshMs`   -> the cached value, no request;
//   * younger than `maxAgeMs`  -> the cached value at once, and ONE background
//                                 request refreshes it (subscribers hear the new
//                                 value, so a screen updates in place);
//   * older, or never fetched  -> a request; concurrent callers share it.
//
// Keys are the caller's: a cache that holds account data MUST put the account in
// the key (`scopedKey`), so one mailbox's value is never served to another
// (linked mailboxes, t87). Failures are not cached. Nothing here persists: the
// cache lives in this page's memory only and is gone on reload or sign-out
// (`clear`).
//
// An entry is dropped by `invalidate` (after a write, or when a live-update
// event says the data changed); a fetch that was already in flight when it was
// invalidated is not stored, so a value read before a write never lands after it.

export interface SwrOptions {
  /** How long a value is served without asking the server again. */
  freshMs: number;
  /** How long a value may be served while a background refresh runs. */
  maxAgeMs: number;
  /** Clock, for tests. */
  now?: () => number;
}

interface Entry<T> {
  value?: T;
  at: number;
  inflight?: Promise<T>;
  /** Bumped by invalidate; a fetch started under an older one is not stored. */
  generation: number;
}

export type SwrListener<T> = (value: T) => void;

export interface SwrCache<T> {
  /** The value for `key`, from memory when fresh enough (see the module comment). */
  get(key: string, fetcher: () => Promise<T>): Promise<T>;
  /** Always ask the server (shared with a request already in flight) and keep the answer. */
  refresh(key: string, fetcher: () => Promise<T>): Promise<T>;
  /** The cached value without any request (`undefined` when none, or too old). */
  peek(key: string): T | undefined;
  /** Store a value the caller already has (a write's answer). */
  set(key: string, value: T): void;
  /** Drop one key, or every key the predicate accepts. */
  invalidate(key: string | ((key: string) => boolean)): void;
  /** Drop everything (sign-in, sign-out). */
  clear(): void;
  /** Hear every value stored for `key` (a background refresh included). */
  subscribe(key: string, listener: SwrListener<T>): () => void;
  /** Number of keys held (tests, diagnostics). */
  size(): number;
}

export function createSwrCache<T>(options: SwrOptions): SwrCache<T> {
  const now = options.now ?? (() => Date.now());
  const entries = new Map<string, Entry<T>>();
  const listeners = new Map<string, Set<SwrListener<T>>>();
  let epoch = 0;

  function entry(key: string): Entry<T> {
    let e = entries.get(key);
    if (!e) {
      e = { at: 0, generation: epoch };
      entries.set(key, e);
    }
    return e;
  }

  function publish(key: string, value: T): void {
    for (const listener of listeners.get(key) ?? []) {
      try { listener(value); } catch { /* a listener's failure stays its own */ }
    }
  }

  function start(key: string, fetcher: () => Promise<T>): Promise<T> {
    const e = entry(key);
    if (e.inflight) return e.inflight;
    const generation = e.generation;
    const run = fetcher().then(
      (value) => {
        const current = entries.get(key);
        if (current === e && e.generation === generation) {
          e.value = value;
          e.at = now();
          publish(key, value);
        }
        return value;
      },
    ).finally(() => {
      if (e.inflight === run) delete e.inflight;
    });
    e.inflight = run;
    // A background refresh's failure must not surface as an unhandled rejection.
    run.catch(() => undefined);
    return run;
  }

  function age(e: Entry<T>): number {
    return now() - e.at;
  }

  return {
    get(key, fetcher) {
      const e = entries.get(key);
      if (e && e.value !== undefined) {
        const a = age(e);
        if (a < options.freshMs) return Promise.resolve(e.value);
        if (a < options.maxAgeMs) {
          void start(key, fetcher);
          return Promise.resolve(e.value);
        }
      }
      return start(key, fetcher);
    },
    refresh(key, fetcher) {
      return start(key, fetcher);
    },
    peek(key) {
      const e = entries.get(key);
      if (!e || e.value === undefined || age(e) >= options.maxAgeMs) return undefined;
      return e.value;
    },
    set(key, value) {
      const e = entry(key);
      // A write's answer outranks a read that was in flight before it.
      e.generation++;
      delete e.inflight;
      e.value = value;
      e.at = now();
      publish(key, value);
    },
    invalidate(key) {
      const match = typeof key === 'string' ? (k: string) => k === key : key;
      for (const [k, e] of entries) {
        if (!match(k)) continue;
        e.generation++;
        delete e.inflight;
        entries.delete(k);
      }
    },
    clear() {
      epoch++;
      for (const e of entries.values()) e.generation++;
      entries.clear();
    },
    subscribe(key, listener) {
      let set = listeners.get(key);
      if (!set) { set = new Set(); listeners.set(key, set); }
      set.add(listener);
      return () => {
        set!.delete(listener);
        if (set!.size === 0) listeners.delete(key);
      };
    },
    size() {
      return entries.size;
    },
  };
}

/** A cache key that names the account it belongs to: `<scope>|<part>|<part>…`. */
export function scopedKey(scope: string, ...parts: readonly (string | null | undefined)[]): string {
  return [scope, ...parts.map((p) => p ?? '')].map((p) => encodeURIComponent(p)).join('|');
}

/** Whether module-level caches run. Off under the unit-test runner, where every
 *  test stubs its own server and a value cached by an earlier test would leak
 *  into the next; the cache's own tests turn it on. */
let cachesEnabled = !(typeof import.meta !== 'undefined' && import.meta.env?.MODE === 'test');
export function swrCachesEnabled(): boolean {
  return cachesEnabled;
}
export function setSwrCachesEnabled(on: boolean): void {
  cachesEnabled = on;
}
