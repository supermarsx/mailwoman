// @vitest-environment node
//
// These tests run the REAL `public/sw.js` (see swHarness.ts) — the worker has no
// second implementation in src/ to drift from. Every "is NOT cached" assertion is
// paired, in the same harness, with a request that IS cached, so a harness that
// cached nothing at all could not pass.

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { shellCacheName } from '../contracts/offline.ts';
import { bootWorker, type Worker } from './swHarness.ts';

const ORIGIN = 'https://mail.example.com';
const ASSET = '/assets/index-a1b2c3d4.js';
const DAY_MS = 24 * 60 * 60 * 1000;
const T0 = Date.UTC(2026, 9, 4);

function reply(body: string, contentType: string, status = 200): Response {
  return new Response(body, { status, headers: { 'content-type': contentType } });
}

const js = (): Response => reply('export{}', 'text/javascript');
const offline = async (): Promise<Response> => {
  throw new TypeError('Failed to fetch');
};

beforeEach(() => {
  vi.useFakeTimers({ toFake: ['Date'] });
  vi.setSystemTime(T0);
});

afterEach(() => {
  vi.useRealTimers();
});

function boot(scriptUrl?: string): Worker {
  const sw = bootWorker(scriptUrl);
  sw.network.fetch = async () => js();
  return sw;
}

describe('cache name', () => {
  it('installs into the cache the offline contract names', async () => {
    const sw = boot();
    sw.network.fetch = async () => reply('<!doctype html>', 'text/html');
    await sw.install();
    expect(await sw.caches.keys()).toEqual([shellCacheName()]);
    expect(sw.caches.urls()).toEqual([`${ORIGIN}/`]);
  });
});

describe('what the worker stores', () => {
  it('caches a hashed asset and a font (the control for everything below)', async () => {
    const sw = boot();
    const asset = await sw.fetch(ASSET);
    expect(asset.handled).toBe(true);
    expect(await asset.response?.text()).toBe('export{}');
    sw.network.fetch = async () => reply('wOF2', 'font/woff2');
    await sw.fetch('/fonts/inter-400.woff2');
    expect(sw.caches.urls()).toEqual([`${ORIGIN}${ASSET}`, `${ORIGIN}/fonts/inter-400.woff2`]);
  });

  // Every one of these was `network-first` (stored on every 200) before t28-e3.
  it.each([
    ['an attachment', '/jmap/download/acct1/blob9/photo.png', 'cors'],
    ['an attachment whose name looks like a hashed asset', '/jmap/download/acct1/blob9/report-2026final.pdf', 'cors'],
    ['a whole message exported as .eml', '/jmap/download/acct1/blob9/message.eml', 'cors'],
    ['an attachment opened as a navigation', '/jmap/download/acct1/blob9/doc.pdf', 'navigate'],
    ['the server-side export', '/api/export/m123', 'cors'],
    ['the signed-in identity', '/api/me', 'cors'],
    ['the JMAP session', '/jmap/session', 'cors'],
    ['the event stream', '/jmap/eventsource', 'cors'],
    ['an API path that is not on the allowlist', '/api/password/policy', 'cors'],
    ['an allowlisted path with a query string', '/api/sso/providers?domain=example.org', 'cors'],
  ])('leaves %s to the browser and stores nothing', async (_what, path, mode) => {
    const sw = boot();
    // Control, same worker instance: a cacheable request is handled and stored.
    await sw.fetch(ASSET);
    expect(sw.caches.urls()).toEqual([`${ORIGIN}${ASSET}`]);

    const out = await sw.fetch(path, { mode });

    expect(out.handled).toBe(false);
    expect(sw.network.calls).toEqual([`${ORIGIN}${ASSET}`]);
    expect(sw.caches.urls()).toEqual([`${ORIGIN}${ASSET}`]);
  });

  it('stores the two public config endpoints, network-first', async () => {
    const sw = boot();
    sw.network.fetch = async () => reply('{"providers":[]}', 'application/json');
    await sw.fetch('/api/sso/providers');
    await sw.fetch('/api/push/vapid');
    expect(sw.caches.urls()).toEqual([`${ORIGIN}/api/sso/providers`, `${ORIGIN}/api/push/vapid`]);

    // Network-first: a later answer replaces what the caller sees.
    sw.network.fetch = async () => reply('{"providers":[1]}', 'application/json');
    expect(await (await sw.fetch('/api/sso/providers')).response?.text()).toBe('{"providers":[1]}');
    // Offline: the stored copy is the fallback.
    sw.network.fetch = offline;
    expect(await (await sw.fetch('/api/sso/providers')).response?.text()).toBe('{"providers":[1]}');
  });

  it('does not handle a cross-origin request, whatever its shape', async () => {
    const sw = boot();
    const out = await sw.fetch('https://cdn.example.net/fonts/inter-400.woff2');
    expect(out.handled).toBe(false);
    expect(sw.caches.urls()).toEqual([]);
  });

  it('never handles a write', async () => {
    const sw = boot();
    expect((await sw.fetch('/jmap/api', { method: 'POST' })).handled).toBe(false);
    expect((await sw.fetch(ASSET, { method: 'POST' })).handled).toBe(false);
    expect(sw.caches.urls()).toEqual([]);
  });

  it('returns a 206 without storing it and without failing the request', async () => {
    const sw = boot();
    sw.network.fetch = async () => reply('part', 'font/woff2', 206);
    const out = await sw.fetch('/fonts/inter-400.woff2');
    expect(out.error).toBeUndefined();
    expect(out.response?.status).toBe(206);
    expect(sw.caches.urls()).toEqual([]);
  });

  it('returns the response when the cache write itself fails', async () => {
    const sw = boot();
    (await sw.caches.open(shellCacheName())).failPuts = true;
    const out = await sw.fetch(ASSET);
    expect(out.error).toBeUndefined();
    expect(await out.response?.text()).toBe('export{}');
    expect(sw.caches.urls()).toEqual([]);
  });

  // mw-server answers an unmatched path with index.html under a 200 (t24-e13).
  it('does not store the SPA fallback returned for a font or an asset', async () => {
    const sw = boot();
    sw.network.fetch = async () => reply('<!doctype html>', 'text/html; charset=utf-8');
    const out = await sw.fetch('/fonts/inter-400.woff2');
    await sw.fetch(ASSET);
    await sw.fetch('/api/push/vapid');
    // The response is still returned to the caller — only the caching is refused.
    expect(await out.response?.text()).toBe('<!doctype html>');
    expect(sw.caches.urls()).toEqual([]);
  });
});

describe('strategies', () => {
  it('cache-first serves a stored asset without touching the network', async () => {
    const sw = boot();
    await sw.fetch(ASSET);
    sw.network.calls.length = 0;
    sw.network.fetch = offline;
    const out = await sw.fetch(ASSET);
    expect(await out.response?.text()).toBe('export{}');
    expect(sw.network.calls).toEqual([]);
  });

  it('network-first fails when offline with nothing stored', async () => {
    const sw = boot();
    sw.network.fetch = offline;
    const out = await sw.fetch('/api/push/vapid');
    expect(out.handled).toBe(true);
    expect(out.error).toBeInstanceOf(TypeError);
  });

  it('an offline navigation gets the stored app shell', async () => {
    const sw = boot();
    sw.network.fetch = async () => reply('<!doctype html><title>shell</title>', 'text/html');
    await sw.install();
    sw.network.fetch = offline;
    const out = await sw.fetch('/inbox', { mode: 'navigate' });
    expect(await out.response?.text()).toBe('<!doctype html><title>shell</title>');
  });

  it('an online navigation to the shell URL replaces the stored shell', async () => {
    const sw = boot();
    sw.network.fetch = async () => reply('old shell', 'text/html');
    await sw.install();
    sw.network.fetch = async () => reply('new shell', 'text/html');
    await sw.fetch('/', { mode: 'navigate' });
    // A deep link is not stored under its own URL.
    await sw.fetch('/inbox', { mode: 'navigate' });
    expect(sw.caches.urls()).toEqual([`${ORIGIN}/`]);
    sw.network.fetch = offline;
    expect(await (await sw.fetch('/inbox', { mode: 'navigate' })).response?.text()).toBe('new shell');
  });
});

describe('logout purge', () => {
  async function populated(scriptUrl?: string): Promise<Worker> {
    const sw = boot(scriptUrl);
    const base = new URL('./', scriptUrl ?? `${ORIGIN}/sw.js`).pathname;
    sw.network.fetch = async () => reply('shell', 'text/html');
    await sw.install();
    sw.network.fetch = async () => js();
    await sw.fetch(`${base}assets/index-a1b2c3d4.js`);
    await sw.caches.open('unrelated');
    // Precondition for every purge test: there is something to purge.
    expect(await sw.caches.keys()).toEqual([shellCacheName(), 'unrelated']);
    expect(sw.caches.urls()).toHaveLength(2);
    return sw;
  }

  it.each(['/api/logout', '/api/sso/logout'])('POST %s deletes every mw-* cache', async (path) => {
    const sw = await populated();
    const out = await sw.fetch(path, { method: 'POST' });
    // The logout request itself still goes to the network untouched.
    expect(out.handled).toBe(false);
    expect(await sw.caches.keys()).toEqual(['unrelated']);
  });

  it.each([
    ['another write', '/jmap/api', 'POST'],
    ['a GET of the logout path', '/api/logout', 'GET'],
    ['the admin logout', '/admin/logout', 'POST'],
    ['a logout sent to another origin', 'https://other.example.net/api/logout', 'POST'],
  ])('%s does not purge', async (_what, path, method) => {
    const sw = await populated();
    await sw.fetch(path, { method });
    expect(await sw.caches.keys()).toEqual([shellCacheName(), 'unrelated']);
    expect(sw.caches.urls()).toHaveLength(2);
  });

  it('purges under a sub-path, and only for that prefix', async () => {
    const sw = await populated(`${ORIGIN}/mail/sw.js`);
    await sw.fetch('/mailbox/api/logout', { method: 'POST' });
    expect(await sw.caches.keys()).toEqual([shellCacheName(), 'unrelated']);
    await sw.fetch('/mail/api/logout', { method: 'POST' });
    expect(await sw.caches.keys()).toEqual(['unrelated']);
  });

  it('the shell is stored again by the next online navigation', async () => {
    const sw = await populated();
    await sw.fetch('/api/logout', { method: 'POST' });
    sw.network.fetch = async () => reply('shell again', 'text/html');
    await sw.fetch('/', { mode: 'navigate' });
    sw.network.fetch = offline;
    expect(await (await sw.fetch('/', { mode: 'navigate' })).response?.text()).toBe('shell again');
  });
});

describe('sub-path hosting', () => {
  it('classifies prefixed paths as their root equivalents', async () => {
    const sw = boot(`${ORIGIN}/mail/sw.js`);
    await sw.fetch('/mail/assets/index-a1b2c3d4.js');
    expect((await sw.fetch('/mail/jmap/download/acct1/blob9/photo.png')).handled).toBe(false);
    expect((await sw.fetch('/mail/api/me')).handled).toBe(false);
    // Only strips on a segment boundary — /mailbox is not /mail + box.
    expect((await sw.fetch('/mailbox/api/push/vapid')).handled).toBe(false);
    expect((await sw.fetch('/mail/api/push/vapid')).handled).toBe(true);
    expect(sw.caches.urls()).toEqual([
      `${ORIGIN}/mail/assets/index-a1b2c3d4.js`,
      `${ORIGIN}/mail/api/push/vapid`,
    ]);
  });

  it('precaches the prefixed shell', async () => {
    const sw = boot(`${ORIGIN}/mail/sw.js`);
    sw.network.fetch = async () => reply('shell', 'text/html');
    await sw.install();
    expect(sw.caches.urls()).toEqual([`${ORIGIN}/mail/`]);
  });
});

describe('bounds', () => {
  it('activation removes what an older worker stored and this one would not', async () => {
    const sw = boot();
    const legacy = await sw.caches.open(shellCacheName());
    // What the pre-t28 worker left behind: unstamped entries, user data included.
    for (const path of [
      '/',
      '/jmap/download/acct1/blob9/photo.png',
      '/jmap/download/acct1/blob9/message.eml',
      '/api/me',
      '/jmap/session',
      ASSET,
    ]) {
      await legacy.put(path, reply('legacy', 'application/octet-stream'));
    }
    await sw.caches.open('mw-shell-v0');
    await sw.caches.open('unrelated');
    expect(sw.caches.urls()).toHaveLength(6);

    await sw.activate();

    // Only the shell survives; the old asset is refetched (and stamped) on demand.
    expect(sw.caches.urls()).toEqual([`${ORIGIN}/`]);
    expect(await sw.caches.keys()).toEqual([shellCacheName(), 'unrelated']);
  });

  it('drops an asset once it is older than 30 days', async () => {
    const sw = boot();
    await sw.fetch(ASSET);
    vi.setSystemTime(T0 + 29 * DAY_MS);
    await sw.fetch('/assets/chunk-b1b2c3d4.js');
    // Precondition: inside the limit, a sweep keeps it.
    expect(sw.caches.urls()).toEqual([`${ORIGIN}${ASSET}`, `${ORIGIN}/assets/chunk-b1b2c3d4.js`]);

    vi.setSystemTime(T0 + 31 * DAY_MS);
    await sw.fetch('/assets/chunk-c1b2c3d4.js');

    expect(sw.caches.urls()).toEqual([
      `${ORIGIN}/assets/chunk-b1b2c3d4.js`,
      `${ORIGIN}/assets/chunk-c1b2c3d4.js`,
    ]);
  });

  it('does not serve a config response older than a day', async () => {
    const sw = boot();
    sw.network.fetch = async () => reply('{"key":"k"}', 'application/json');
    await sw.fetch('/api/push/vapid');
    sw.network.fetch = offline;
    vi.setSystemTime(T0 + DAY_MS - 1000);
    // Precondition: inside the limit the offline fallback works.
    expect(await (await sw.fetch('/api/push/vapid')).response?.text()).toBe('{"key":"k"}');

    vi.setSystemTime(T0 + DAY_MS + 1000);

    expect((await sw.fetch('/api/push/vapid')).error).toBeInstanceOf(TypeError);
  });

  it('keeps at most 512 assets, evicting the oldest', async () => {
    const sw = boot();
    const url = (i: number): string => `${ORIGIN}/assets/chunk-${String(i).padStart(8, '0')}.js`;
    for (let i = 0; i < 512; i += 1) {
      vi.setSystemTime(T0 + i * 1000);
      await sw.fetch(url(i));
    }
    // Precondition: the cap itself is reachable — 512 entries all stay.
    vi.setSystemTime(T0 + 600 * 1000);
    await sw.activate();
    expect(sw.caches.urls()).toHaveLength(512);

    for (let i = 512; i < 520; i += 1) {
      vi.setSystemTime(T0 + i * 60_000);
      await sw.fetch(url(i));
    }

    const urls = sw.caches.urls();
    expect(urls).toHaveLength(512);
    expect(urls).not.toContain(url(0));
    expect(urls).not.toContain(url(7));
    expect(urls).toContain(url(8));
    expect(urls).toContain(url(519));
  });
});
