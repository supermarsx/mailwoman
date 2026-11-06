// Client-side sanitising: cache -> worker pool -> fallback.
//
//   sanitizeBody(req)  answers from the content-hash cache when it can, otherwise from the
//                      sanitiser worker pool (`pool.ts`, separate from the crypto worker),
//                      otherwise from the fallback for that op (strip: the server's
//                      /api/sanitize; reader ops: the crypto worker's sanitizeReader).
//
// The pool and the server run the same `mw-sanitize` code (the wasm build of the crate the
// server links), so the output is the same from either path. Decrypted end-to-end
// encrypted mail never comes through here: it is sanitised inside the crypto worker.
// The cache is memory only and is emptied on sign-out and account switch.

import { getCryptoWorker } from '../crypto/index.ts';
import { observePreferenceAccount } from '../api/accountPreferenceLifecycle.ts';
import { SanitizeCache, cacheKey } from './cache.ts';
import { SanitizerPool, poolSize, type Priority, type WorkerLike } from './pool.ts';
import type { SanitizeRequest } from './protocol.ts';

export type { SanitizeRequest } from './protocol.ts';
export type { Priority } from './pool.ts';

/** Default cache budget: 16 MiB of sanitised output. */
export const DEFAULT_CACHE_BYTES = 16 * 1024 * 1024;

export const sanitizeCache = new SanitizeCache(DEFAULT_CACHE_BYTES);

/** Per-op fallbacks, installed by the owners of the server client and the crypto worker. */
export interface Fallbacks {
  /** `strip` op: the server's POST /api/sanitize. */
  server?: (html: string) => Promise<string>;
  /** Reader ops: the crypto worker's `sanitizeReader`. */
  reader?: (req: SanitizeRequest) => Promise<string>;
}
let fallbacks: Fallbacks = {
  // No sanitiser worker could run (no Worker support, wasm blocked): the crypto worker
  // hosts the same wasm. Slower (shared with key operations) but correct.
  reader: (req) => getCryptoWorker().sanitizeReader({
    html: req.html,
    blockTrackers: req.blockTrackers !== false,
    cleanLinks: req.cleanLinks !== false,
    ...(req.op === 'readerSafe' ? { allowSanitizedHtml: true } : {}),
  }),
};
export function setSanitizeFallbacks(next: Fallbacks): void {
  fallbacks = { ...fallbacks, ...next };
}

function fallback(req: SanitizeRequest): Promise<string> {
  if (req.op === 'strip') {
    return fallbacks.server ? fallbacks.server(req.html) : Promise.reject(new Error('no server sanitizer'));
  }
  return fallbacks.reader ? fallbacks.reader(req) : Promise.reject(new Error('no reader sanitizer'));
}

let pool: SanitizerPool | null = null;

function createPool(): SanitizerPool {
  return new SanitizerPool({
    size: poolSize(typeof navigator === 'undefined' ? undefined : navigator.hardwareConcurrency),
    // The inline `new Worker(new URL(...), { type: 'module' })` form is what Vite detects to
    // bundle the worker and its wasm as a separate chunk (same as crypto/worker.ts).
    createWorker: () => new Worker(new URL('./worker.entry.ts', import.meta.url), { type: 'module' }) as unknown as WorkerLike,
    loadModule: async () => {
      // A static build asset, not an API request: there is no session or CSRF token to send.
      // eslint-disable-next-line mw/no-global-fetch
      const res = await fetch(new URL('../wasm/mw-sanitize/mw_sanitize_bg.wasm', import.meta.url));
      if (!res.ok) throw new Error(`sanitizer wasm: ${res.status}`);
      return WebAssembly.compile(await res.arrayBuffer());
    },
    // No pool-level fallback: `sanitizeBody` answers any pool failure from `fallback` once.
  });
}

/** The shared pool (created on first use). Tests install their own with `setSanitizePool`. */
export function sanitizePool(): SanitizerPool {
  pool ??= createPool();
  return pool;
}
export function setSanitizePool(next: SanitizerPool | null): void {
  pool?.dispose();
  pool = next;
}

let lifecycleHooked = false;
function hookLifecycle(): void {
  if (lifecycleHooked) return;
  lifecycleHooked = true;
  // Fires on sign-out (null) and on every account change; either empties the cache.
  let first = true;
  observePreferenceAccount(() => {
    if (first) { first = false; return; }
    sanitizeCache.clear();
  });
}

function optionsKey(req: SanitizeRequest): string {
  return `${req.op}:${req.blockTrackers === false ? 0 : 1}${req.cleanLinks === false ? 0 : 1}`;
}

const inflight = new Map<string, Promise<string>>();

function sizeBucket(n: number): string {
  return n < 16_384 ? 'xs' : n < 131_072 ? 's' : n < 1_048_576 ? 'm' : 'l';
}

function mark(path: 'cache' | 'worker' | 'fallback', bytes: number, start: number): void {
  try {
    performance.measure(`mw:sanitize:${path}:${sizeBucket(bytes)}`, { start, end: performance.now() });
  } catch { /* measures are diagnostics only */ }
}

export interface SanitizeOptions {
  /** `high` (default): the message being opened. `low`: prefetch. */
  priority?: Priority;
  signal?: AbortSignal;
}

/** Sanitise `req.html`; identical requests share one result. */
export async function sanitizeBody(req: SanitizeRequest, options: SanitizeOptions = {}): Promise<string> {
  hookLifecycle();
  const start = typeof performance === 'undefined' ? 0 : performance.now();
  const key = cacheKey(optionsKey(req), req.html);
  const hit = sanitizeCache.get(key);
  if (hit !== undefined) {
    mark('cache', req.html.length, start);
    return hit;
  }
  const shared = options.signal === undefined ? inflight.get(key) : undefined;
  if (shared) return shared;
  const work = (async () => {
    let path: 'worker' | 'fallback' = 'worker';
    let html: string;
    try {
      html = await sanitizePool().run(req, {
        ...(options.priority !== undefined ? { priority: options.priority } : {}),
        ...(options.signal !== undefined ? { signal: options.signal } : {}),
      });
    } catch (error) {
      // Aborts and a full prefetch queue are answers; anything else is a pool failure
      // the fallback can still serve.
      if (options.signal?.aborted || (error instanceof Error && (error.name === 'AbortError' || error.name === 'QueueFullError'))) throw error;
      path = 'fallback';
      html = await fallback(req);
    }
    sanitizeCache.set(key, html);
    mark(path, req.html.length, start);
    return html;
  })();
  if (options.signal === undefined) {
    inflight.set(key, work);
    const done = () => { if (inflight.get(key) === work) inflight.delete(key); };
    work.then(done, done);
  }
  return work;
}

/** Strip-mode body (remote images removed): the message-load result the reader falls back to. */
export function sanitizeStrip(html: string, options?: SanitizeOptions): Promise<string> {
  return sanitizeBody({ op: 'strip', html }, options);
}

/** Warm the cache for a message the user is likely to open next (lowest priority, never throws). */
export function prefetchSanitized(req: SanitizeRequest): void {
  void sanitizeBody(req, { priority: 'low' }).catch(() => undefined);
}

export function clearSanitizeCache(): void {
  sanitizeCache.clear();
  inflight.clear();
}
