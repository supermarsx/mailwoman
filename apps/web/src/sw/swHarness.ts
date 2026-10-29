// Test harness that runs the REAL service worker (`public/sw.js`) — the file is
// read from disk and evaluated as-is against a fake ServiceWorkerGlobalScope.
// Nothing here restates the worker's routing: the fakes stand in for the BROWSER
// (Cache Storage, the network, event dispatch), never for the worker.

import { createHash } from 'node:crypto';
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
  push(): Promise<void>;
  pushState: { endpoint: string | null; visible: boolean; notifications: string[] };
  install(): Promise<void>;
  activate(): Promise<void>;
  /** `client` is the page that made the request (default: the mail shell, `/`). */
  fetch(path: string, init?: { method?: string; mode?: string; client?: string; clientType?: string }): Promise<FetchOutcome>;
}

/** The SHA-256 the build would record for `body` (t63 R1). */
export function digestOf(body: string): string {
  return createHash('sha256').update(body).digest('hex');
}

/**
 * The asset manifest the build writes into dist/sw.js. By default every
 * /assets/ path is recorded with the digest of the body the tests' default
 * network serves (`export{}`), so a stored copy of that body verifies and any
 * other body does not.
 */
export function defaultDigests(overrides: Record<string, string> = {}): Record<string, string> {
  const hash = digestOf('export{}');
  const shell = digestOf(TEST_SHELL);
  return new Proxy(overrides, { get: (target, key) => {
    if (typeof key !== 'string') return undefined;
    if (key in target) return target[key];
    if (key === '/index.html') return shell;
    return key.startsWith('/assets/') ? hash : undefined;
  } });
}

/** The shell body the default manifest records for /index.html. */
export const TEST_SHELL = '<!doctype html><title>shell</title>';

type Listener = (event: Record<string, unknown>) => void;

/**
 * Evaluate `public/sw.js` with `self`, `caches` and `fetch` bound to fakes.
 * `scriptUrl` is where the worker is served from — its directory is the deploy
 * prefix the worker derives for itself.
 */
export function bootWorker(scriptUrl = 'https://mail.example.com/sw.js', options: { digests?: Record<string, string> | null } = {}): Worker {
  const location = new URL(scriptUrl);
  const caches = new FakeCacheStorage(location.href);
  const listeners = new Map<string, Listener>();
  const clientUrls = new Map<string, { type: string; url: string }>();
  const network: Worker['network'] = {
    calls: [],
    fetch: async () => new Response('ok', { status: 200 }),
  };
  const pushState: Worker['pushState'] = { endpoint: 'https://push.example/capability-secret', visible: false, notifications: [] };
  const self = {
    crypto: globalThis.crypto, TextEncoder, TextDecoder,
    location,
    addEventListener: (type: string, fn: Listener) => listeners.set(type, fn),
    skipWaiting: async () => undefined,
    __assetDigests: options.digests === undefined ? defaultDigests() : options.digests,
    clients: { claim: async () => undefined, get: async (id: string) => clientUrls.get(id), matchAll: async () => pushState.visible ? [{ visibilityState: 'visible', postMessage: () => undefined }] : [] },
    registration: { pushManager: { getSubscription: async () => pushState.endpoint ? { endpoint: pushState.endpoint } : null }, showNotification: async (title: string) => { pushState.notifications.push(title); } },
  };
  const fetchFromWorker = (req: FakeRequest | string): Promise<Response> => {
    network.calls.push(new URL(typeof req === 'string' ? req : req.url, location.href).href);
    return network.fetch(req);
  };
  const source = SW_SOURCE.replace('/* @mw-asset-digests */ null', 'self.__assetDigests');
  if (source === SW_SOURCE) throw new Error('sw.js lost its asset-digest placeholder');
  new Function('self', 'caches', 'fetch', source)(self, caches, fetchFromWorker);

  async function lifecycle(type: string): Promise<void> {
    const waits: Promise<unknown>[] = [];
    listeners.get(type)?.({ waitUntil: (p: Promise<unknown>) => waits.push(p) });
    await Promise.all(waits);
  }

  return {
    origin: location.origin,
    caches,
    network,
    push: () => lifecycle('push'),
    pushState,
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
      const clientId = `client-${clientUrls.size + 1}`;
      clientUrls.set(clientId, { type: init.clientType ?? 'window', url: new URL(init.client ?? '/', location.href).href });
      listeners.get('fetch')?.({
        request,
        clientId: request.mode === 'navigate' ? '' : clientId,
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
