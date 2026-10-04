// Test harness that runs the REAL service worker (`public/sw.js`) — the file is
// read from disk and evaluated as-is against a fake ServiceWorkerGlobalScope.
// Nothing here restates the worker's routing: the fakes stand in for the BROWSER
// (Cache Storage, the network, event dispatch), never for the worker.

import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

const SW_SOURCE = readFileSync(
  fileURLToPath(new URL('../../public/sw.js', import.meta.url)),
  'utf8',
);

/** What the worker reads off a `Request`; `mode: 'navigate'` cannot be constructed. */
export interface FakeRequest {
  url: string;
  method: string;
  mode: string;
}

type CacheKey = string | { url: string };

/** One named cache. Mirrors the two `Cache.put` rules the worker depends on. */
export class FakeCache {
  readonly entries = new Map<string, Response>();
  /** Make every `put` reject, as a full quota or a `Vary: *` response does. */
  failPuts = false;

  constructor(private readonly base: string) {}

  private key(key: CacheKey): string {
    return new URL(typeof key === 'string' ? key : key.url, this.base).href;
  }

  async match(key: CacheKey): Promise<Response | undefined> {
    return this.entries.get(this.key(key))?.clone();
  }

  async put(key: CacheKey, res: Response): Promise<void> {
    if (this.failPuts) throw new TypeError('put rejected');
    // The real Cache API refuses partial content.
    if (res.status === 206) throw new TypeError('Partial response (status code 206) is unsupported');
    this.entries.set(this.key(key), res);
  }

  async delete(key: CacheKey): Promise<boolean> {
    return this.entries.delete(this.key(key));
  }

  async keys(): Promise<FakeRequest[]> {
    return [...this.entries.keys()].map((url) => ({ url, method: 'GET', mode: '' }));
  }
}

export class FakeCacheStorage {
  readonly stores = new Map<string, FakeCache>();

  constructor(private readonly base: string) {}

  async open(name: string): Promise<FakeCache> {
    let cache = this.stores.get(name);
    if (cache === undefined) {
      cache = new FakeCache(this.base);
      this.stores.set(name, cache);
    }
    return cache;
  }

  async keys(): Promise<string[]> {
    return [...this.stores.keys()];
  }

  async delete(name: string): Promise<boolean> {
    return this.stores.delete(name);
  }

  /** Every cached URL across every cache, for assertions. */
  urls(): string[] {
    return [...this.stores.values()].flatMap((cache) => [...cache.entries.keys()]);
  }
}

export interface FetchOutcome {
  /** Whether the worker called `respondWith` — false means the browser handles it. */
  handled: boolean;
  response: Response | undefined;
  /** Set when the promise handed to `respondWith` rejected (a failed fetch). */
  error: unknown;
}

export interface Worker {
  origin: string;
  caches: FakeCacheStorage;
  /** The network as the worker sees it; replace per test. */
  network: { fetch: (req: FakeRequest | string) => Promise<Response>; calls: string[] };
  install(): Promise<void>;
  activate(): Promise<void>;
  fetch(path: string, init?: { method?: string; mode?: string }): Promise<FetchOutcome>;
}

type Listener = (event: Record<string, unknown>) => void;

/**
 * Evaluate `public/sw.js` with `self`, `caches` and `fetch` bound to fakes.
 * `scriptUrl` is where the worker is served from — its directory is the deploy
 * prefix the worker derives for itself.
 */
export function bootWorker(scriptUrl = 'https://mail.example.com/sw.js'): Worker {
  const location = new URL(scriptUrl);
  const caches = new FakeCacheStorage(location.href);
  const listeners = new Map<string, Listener>();
  const network: Worker['network'] = {
    calls: [],
    fetch: async () => new Response('ok', { status: 200 }),
  };
  const self = {
    location,
    addEventListener: (type: string, fn: Listener) => listeners.set(type, fn),
    skipWaiting: async () => undefined,
    clients: { claim: async () => undefined },
  };
  const fetchFromWorker = (req: FakeRequest | string): Promise<Response> => {
    network.calls.push(new URL(typeof req === 'string' ? req : req.url, location.href).href);
    return network.fetch(req);
  };
  new Function('self', 'caches', 'fetch', SW_SOURCE)(self, caches, fetchFromWorker);

  async function lifecycle(type: string): Promise<void> {
    const waits: Promise<unknown>[] = [];
    listeners.get(type)?.({ waitUntil: (p: Promise<unknown>) => waits.push(p) });
    await Promise.all(waits);
  }

  return {
    origin: location.origin,
    caches,
    network,
    install: () => lifecycle('install'),
    activate: () => lifecycle('activate'),
    async fetch(path, init = {}) {
      let responded: Promise<Response> | undefined;
      const waits: Promise<unknown>[] = [];
      const request: FakeRequest = {
        url: new URL(path, location.origin).href,
        method: init.method ?? 'GET',
        mode: init.mode ?? 'cors',
      };
      listeners.get('fetch')?.({
        request,
        respondWith: (p: Promise<Response>) => {
          responded = p;
        },
        waitUntil: (p: Promise<unknown>) => waits.push(p),
      });
      let response: Response | undefined;
      let error: unknown;
      try {
        response = await responded;
      } catch (err) {
        error = err;
      }
      await Promise.all(waits);
      return { handled: responded !== undefined, response, error };
    },
  };
}
