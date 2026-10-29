// @vitest-environment node
//
// These tests run the REAL `public/sw.js` (see swHarness.ts) — the worker has no
// second implementation in src/ to drift from. Every "is NOT cached" assertion is
// paired, in the same harness, with a request that IS cached, so a harness that
// cached nothing at all could not pass.

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { shellCacheName } from '../contracts/offline.ts';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { bootWorker, defaultDigests, digestOf, TEST_SHELL, type Worker } from './swHarness.ts';

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
  it('can install and activate when browser storage is unavailable', async () => {
    const sw = boot();
    vi.spyOn(sw.caches, 'open').mockRejectedValue(new Error('Storage denied'));
    vi.spyOn(sw.caches, 'keys').mockRejectedValue(new Error('Storage denied'));
    await expect(sw.install()).resolves.toBeUndefined();
    await expect(sw.activate()).resolves.toBeUndefined();
    expect(await (await sw.fetch(ASSET)).response?.text()).toBe('export{}');
  });
  it('installs into the cache the offline contract names', async () => {
    const sw = boot();
    sw.network.fetch = async () => reply(TEST_SHELL, 'text/html');
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
    sw.network.fetch = async () => reply('{"providers":[{"id":"corp","kind":"oidc","displayName":"Corp"}]}', 'application/json');
    expect(await (await sw.fetch('/api/sso/providers')).response?.text()).toBe('{"providers":[{"id":"corp","kind":"oidc","displayName":"Corp"}]}');
    // Offline: the stored copy is the fallback.
    sw.network.fetch = offline;
    expect(await (await sw.fetch('/api/sso/providers')).response?.text()).toBe('{"providers":[{"id":"corp","kind":"oidc","displayName":"Corp"}]}');
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
  it.each([ASSET, '/api/push/vapid', '/'])('keeps %s online when Cache Storage cannot open', async path => {
    const sw = boot();
    vi.spyOn(sw.caches, 'open').mockRejectedValue(new DOMException('Storage unavailable', 'SecurityError'));
    sw.network.fetch = async () => path === '/' ? reply('<!doctype html>', 'text/html') : js();
    const result = await sw.fetch(path, { mode: path === '/' ? 'navigate' : 'cors' });
    expect(result.error).toBeUndefined();
    expect(result.response?.status).toBe(200);
    expect(sw.network.calls).toHaveLength(1);
  });

  it('fetches an asset when its cache lookup fails', async () => {
    const sw = boot();
    const cache = await sw.caches.open(shellCacheName());
    vi.spyOn(cache, 'match').mockRejectedValue(new Error('Damaged cache'));
    const result = await sw.fetch(ASSET);
    expect(result.error).toBeUndefined();
    expect(await result.response?.text()).toBe('export{}');
  });

  it('replaces an old cached HTML error instead of serving it as a module', async () => {
    const sw = boot();
    const cache = await sw.caches.open(shellCacheName());
    await cache.put(ASSET, reply('<html>old fallback</html>', 'text/html'));
    expect(await (await sw.fetch(ASSET)).response?.text()).toBe('export{}');
    expect(sw.network.calls).toHaveLength(1);
  });

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
    const out = await sw.fetch('/', { mode: 'navigate' });
    expect(await out.response?.text()).toBe('<!doctype html><title>shell</title>');
  });

  it('an online navigation to the shell URL replaces the stored shell', async () => {
    // A new build: only its own index.html matches the manifest.
    const sw = bootWorker(undefined, { digests: defaultDigests({ '/index.html': digestOf('new shell') }) });
    sw.network.fetch = async () => reply('old shell', 'text/html');
    await sw.install();
    sw.network.fetch = async () => reply('new shell', 'text/html');
    await sw.fetch('/', { mode: 'navigate' });
    // The shell's other name is not stored under its own URL.
    await sw.fetch('/index.html', { mode: 'navigate' });
    expect(sw.caches.urls()).toEqual([`${ORIGIN}/`]);
    sw.network.fetch = offline;
    expect(await (await sw.fetch('/index.html', { mode: 'navigate' })).response?.text()).toBe('new shell');
  });
});

describe('logout purge', () => {
  async function populated(scriptUrl?: string): Promise<Worker> {
    const sw = boot(scriptUrl);
    const base = new URL('./', scriptUrl ?? `${ORIGIN}/sw.js`).pathname;
    sw.network.fetch = async () => reply(TEST_SHELL, 'text/html');
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
    sw.network.fetch = async () => reply(TEST_SHELL, 'text/html');
    await sw.fetch('/', { mode: 'navigate' });
    sw.network.fetch = offline;
    expect(await (await sw.fetch('/', { mode: 'navigate' })).response?.text()).toBe(TEST_SHELL);
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
    sw.network.fetch = async () => reply(TEST_SHELL, 'text/html');
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
    sw.network.fetch = async () => reply('{"publicKey":"BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB"}', 'application/json');
    await sw.fetch('/api/push/vapid');
    sw.network.fetch = offline;
    vi.setSystemTime(T0 + DAY_MS - 1000);
    // Precondition: inside the limit the offline fallback works.
    expect(await (await sw.fetch('/api/push/vapid')).response?.text()).toBe('{"publicKey":"BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB"}');

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

// t32-h8: under a controlling worker `/admin` showed mw-server's plain-text 404.
// The worker re-fetched the navigation itself, and a fetch made from a worker
// goes out with `Sec-Fetch-Dest: empty` instead of `document`, which the server's
// SPA fallback (`wants_spa_shell`) correctly refuses. Admin paths are now left to
// the browser entirely: its own navigation carries `Sec-Fetch-Dest: document`,
// nothing the admin session fetches is stored, and offline it gets the browser's
// error page rather than the mail shell.
//
// t32-h15: the console can be at any path (`MW_ADMIN_PATH`) and the worker is never
// told which, so it owns only the mail shell's navigations (an allowlist) and
// stores only what is under `/assets/` and `/fonts/`. `/ops-7f3k` stands for an
// operator's path.
describe('the admin console is never handled by the worker', () => {
  async function withShell(scriptUrl?: string): Promise<Worker> {
    const sw = boot(scriptUrl);
    sw.network.fetch = async () => reply('<!doctype html><title>shell</title>', 'text/html');
    await sw.install();
    sw.network.calls.length = 0;
    return sw;
  }

  it.each([
    ['/admin', 'navigate'],
    ['/admin/', 'navigate'],
    ['/admin?next=users', 'navigate'],
    ['/admin/users', 'navigate'],
    ['/oauth/authorize?client_id=c1&state=s', 'navigate'],
    ['/admin/session', 'cors'],
    ['/admin/audit/export-a1b2c3d4e5.csv', 'cors'],
    ['/ops-7f3k', 'navigate'],
    ['/ops-7f3k?next=users', 'navigate'],
    ['/ops/console', 'navigate'],
    ['/ops-7f3k/session', 'cors'],
    ['/ops-7f3k/audit/export-a1b2c3d4e5.csv', 'cors'],
    ['/ops-7f3k/fonts/inter-400.woff2', 'cors'],
    ['/ops-7f3k/assets-a1b2c3d4.js', 'cors'],
  ])('leaves %s (%s) to the browser and stores nothing', async (path, mode) => {
    const sw = await withShell();
    // Control, same worker: a navigation to the mail shell is still answered by the worker.
    expect((await sw.fetch('/', { mode: 'navigate' })).handled).toBe(true);
    sw.network.calls.length = 0;

    const out = await sw.fetch(path, { mode });

    expect(out.handled).toBe(false);
    expect(sw.network.calls).toEqual([]);
    expect(sw.caches.urls()).toEqual([`${ORIGIN}/`]);
  });

  it('offline, an admin navigation is not answered with the mail shell', async () => {
    const sw = await withShell();
    sw.network.fetch = offline;
    // Control: the mail app still gets its shell offline.
    expect(await (await sw.fetch('/', { mode: 'navigate' })).response?.text()).toBe(
      '<!doctype html><title>shell</title>',
    );
    expect((await sw.fetch('/admin', { mode: 'navigate' })).handled).toBe(false);
    expect((await sw.fetch('/ops-7f3k', { mode: 'navigate' })).handled).toBe(false);
  });

  it('owns only the mail shell navigations', async () => {
    const sw = await withShell();
    for (const path of ['/', '/?x=1', '/index.html']) {
      expect((await sw.fetch(path, { mode: 'navigate' })).handled, path).toBe(true);
    }
    // An unknown page goes to the browser, which gets the server's 404 page (t32-h10).
    for (const path of ['/administrator', '/inbox', '/index.htm', '/x/index.html', '/oauth/authorized']) {
      expect((await sw.fetch(path, { mode: 'navigate' })).handled, path).toBe(false);
    }
  });

  it('under a sub-path, only the prefixed mail shell is handled', async () => {
    const sw = await withShell(`${ORIGIN}/mail/sw.js`);
    expect((await sw.fetch('/mail/admin', { mode: 'navigate' })).handled).toBe(false);
    expect((await sw.fetch('/mail/admin/', { mode: 'navigate' })).handled).toBe(false);
    expect((await sw.fetch('/mail/ops-7f3k', { mode: 'navigate' })).handled).toBe(false);
    expect((await sw.fetch('/mail/oauth/authorize', { mode: 'navigate' })).handled).toBe(false);
    expect((await sw.fetch('/mail/admin/session')).handled).toBe(false);
    expect((await sw.fetch('/mail/ops-7f3k/session')).handled).toBe(false);
    // Control: the prefixed mail app is still handled.
    expect((await sw.fetch('/mail/', { mode: 'navigate' })).handled).toBe(true);
    expect(sw.caches.urls()).toEqual([`${ORIGIN}/mail/`]);
  });

  it('activation removes an admin response an older worker stored', async () => {
    const sw = boot();
    const legacy = await sw.caches.open(shellCacheName());
    // Both stamped as fresh, so neither is removed for its age.
    const fresh = (body: string): Response =>
      new Response(body, { headers: { 'x-mw-cached-at': String(T0) } });
    await legacy.put('/admin/audit/export-a1b2c3d4e5.csv', fresh('audit rows'));
    await legacy.put('/ops-7f3k/audit/export-a1b2c3d4e5.csv', fresh('audit rows'));
    await legacy.put(ASSET, fresh('export{}'));

    await sw.activate();

    expect(sw.caches.urls()).toEqual([`${ORIGIN}${ASSET}`]);
  });
});

describe('background push authority recheck', () => {
  it('checks only an endpoint hash under the deployment prefix before notifying', async () => {
    const sw=boot('https://mail.example.com/mail/sw.js');
    sw.network.fetch=async()=>reply('{"allowed":true}','application/json');
    await sw.push();
    expect(sw.pushState.notifications).toEqual(['Mailwoman']);
    expect(sw.network.calls).toHaveLength(1);
    expect(sw.network.calls[0]).toMatch(/^https:\/\/mail\.example\.com\/mail\/api\/push\/permission\?endpointHash=[a-f0-9]{64}$/);
    expect(sw.network.calls[0]).not.toContain('capability-secret');
    expect(sw.caches.urls()).toEqual([]);
  });
  it('suppresses delayed wakes after logout, policy refusal, or network failure', async () => {
    for (const response of [async()=>reply('{}','application/json',401),async()=>reply('{"allowed":false}','application/json'),offline]) {
      const sw=boot();sw.network.fetch=response;await sw.push();expect(sw.pushState.notifications).toEqual([]);
    }
  });
  it('does not display or request authority when a window is visible or the subscription was removed', async () => {
    const sw=boot();sw.pushState.visible=true;await sw.push();expect(sw.network.calls).toEqual([]);
    sw.pushState.visible=false;sw.pushState.endpoint=null;await sw.push();expect(sw.network.calls).toEqual([]);
    expect(sw.pushState.notifications).toEqual([]);
  });
});

// t63 F4: a shell served with a browser analytics tag, and the tag's loader, are
// per-user and per-consent: never stored, always fetched.
describe('browser analytics tag', () => {
  const tagged = (body: string): Response => new Response(body, { status: 200, headers: { 'content-type': 'text/html', 'x-mw-analytics-tag': '1' } });
  it('does not store a tagged shell, and keeps the untagged one for offline use', async () => {
    const sw = boot();
    sw.network.fetch = async () => reply(TEST_SHELL, 'text/html');
    await sw.install();
    sw.network.fetch = async () => tagged('tagged shell');
    expect(await (await sw.fetch('/', { mode: 'navigate' })).response?.text()).toBe('tagged shell');
    sw.network.fetch = offline;
    expect(await (await sw.fetch('/', { mode: 'navigate' })).response?.text()).toBe(TEST_SHELL);
  });
  it('does not precache a tagged shell at install', async () => {
    const sw = boot();
    sw.network.fetch = async () => tagged('tagged shell');
    await sw.install();
    expect(sw.caches.urls()).toEqual([]);
  });
  it('passes the loader through to the network and never stores it', async () => {
    const sw = boot();
    const loader = `/assets/analytics/${'a'.repeat(64)}.js`;
    await sw.fetch(loader, { mode: 'no-cors' });
    await sw.fetch(ASSET);
    expect(sw.caches.urls()).toEqual([`${ORIGIN}${ASSET}`]);
    sw.network.fetch = offline;
    expect((await sw.fetch(loader, { mode: 'no-cors' })).response).toBeUndefined();
  });
});

// t63 R1: any script of this origin can write Cache Storage, so nothing the
// worker serves from it is trusted on sight. Each of these fails on the worker
// before t63 R1.
describe('a poisoned cache cannot run in the console or the shell', () => {
  const EVIL = 'globalThis.__poisoned = true;';
  const poison = async (sw: Worker, path: string, body: string, headers: Record<string, string> = {}) => {
    const cache = await sw.caches.open(shellCacheName(1));
    await cache.put(`${ORIGIN}${path}`, new Response(body, { status: 200, headers: { 'content-type': 'text/javascript', 'x-mw-cached-at': '9000000000000000', ...headers } }));
  };
  it('the admin console never gets a cached asset, even a valid one', async () => {
    const sw = boot();
    await sw.fetch(ASSET); // stored for the mail shell
    sw.network.calls.length = 0;
    await poison(sw, ASSET, EVIL);
    const out = await sw.fetch(ASSET, { client: '/admin' });
    expect(await out.response?.text()).toBe('export{}');
    expect(sw.network.calls).toEqual([`${ORIGIN}${ASSET}`]);
  });
  it('the mail shell drops a cached asset that does not match the build manifest', async () => {
    const sw = boot();
    await poison(sw, ASSET, EVIL);
    const out = await sw.fetch(ASSET);
    expect(await out.response?.text()).toBe('export{}');
    sw.network.fetch = offline;
    expect(await (await sw.fetch(ASSET)).response?.text()).toBe('export{}');
  });
  it('without a build manifest no /assets/ file is served from the cache', async () => {
    const sw = bootWorker(undefined, { digests: null });
    sw.network.fetch = async () => js();
    await sw.fetch(ASSET);
    expect(sw.caches.urls()).toEqual([]);
    await poison(sw, ASSET, 'export{}');
    sw.network.calls.length = 0;
    await sw.fetch(ASSET);
    expect(sw.network.calls).toHaveLength(1);
  });
  it('a stamp from the future does not keep a config entry fresh', async () => {
    const sw = boot();
    await poison(sw, '/api/push/vapid', '{"key":"attacker"}', { 'content-type': 'application/json' });
    sw.network.fetch = offline;
    const out = await sw.fetch('/api/push/vapid');
    expect(out.response).toBeUndefined();
    expect(out.error).toBeDefined();
  });
  it('the offline shell is served under the fixed shell policy, not the stored headers', async () => {
    const sw = boot();
    const cache = await sw.caches.open(shellCacheName(1));
    await cache.put(`${ORIGIN}/`, new Response(TEST_SHELL, { status: 200, headers: { 'content-type': 'text/plain', 'content-security-policy': "script-src 'unsafe-inline'" } }));
    sw.network.fetch = offline;
    const out = await sw.fetch('/', { mode: 'navigate' });
    expect(out.response?.headers.get('content-security-policy')).toBe(rustShellCsp());
    expect(out.response?.headers.get('content-type')).toBe('text/html; charset=utf-8');
    expect(out.response?.headers.get('x-content-type-options')).toBe('nosniff');
  });
  it('the offline policy is the server shell CSP, byte for byte', () => {
    const sw = readFileSync(fileURLToPath(new URL('../../public/sw.js', import.meta.url)), 'utf8');
    expect(sw).toContain(`const OFFLINE_SHELL_CSP = "${rustShellCsp()}";`);
  });
});

function rustShellCsp(): string {
  const lib = readFileSync(fileURLToPath(new URL('../../../../crates/mw-server/src/lib.rs', import.meta.url)), 'utf8');
  const raw = /const CSP: &str = "([^"]+)";/.exec(lib)?.[1];
  if (raw === undefined) throw new Error('CSP const not found in lib.rs');
  return raw.replace(/\\\r?\n\s*/g, '');
}

// t63 T1/T2/T6: stored headers are never served, every build file and the
// shell body are checked against the manifest, and the app's own workers still
// get their wasm offline. Each of these fails on the worker of 6885ae2c.
describe('nothing stored is trusted, and the app still works offline', () => {
  const put = async (sw: Worker, path: string, body: string, headers: Record<string, string>) => {
    const cache = await sw.caches.open(shellCacheName(1));
    await cache.put(`${new URL(path, ORIGIN).href}`, new Response(body, { status: 200, headers: { 'x-mw-cached-at': String(Date.now()), ...headers } }));
  };
  it('a poisoned stylesheet with script in it is dropped, and is never served as script', async () => {
    const sw = boot();
    await put(sw, '/assets/index-x.css', 'globalThis.__poisoned = true;', { 'content-type': 'text/javascript' });
    sw.network.fetch = offline;
    expect((await sw.fetch('/assets/index-x.css')).response).toBeUndefined();
  });
  it('a genuine stored file is served with headers from its extension, not the stored ones', async () => {
    const sw = boot();
    await put(sw, '/assets/index-x.css', 'export{}', { 'content-type': 'text/javascript' });
    sw.network.fetch = offline;
    const out = await sw.fetch('/assets/index-x.css');
    expect(out.response?.headers.get('content-type')).toBe('text/css; charset=utf-8');
    expect(out.response?.headers.get('x-content-type-options')).toBe('nosniff');
  });
  it('an installed font under /fonts/ is served only as a font', async () => {
    const sw = boot();
    await put(sw, '/fonts/jost.woff2', 'globalThis.__poisoned = true;', { 'content-type': 'text/javascript' });
    sw.network.fetch = offline;
    const out = await sw.fetch('/fonts/jost.woff2');
    expect(out.response?.headers.get('content-type')).toBe('font/woff2');
    expect(out.response?.headers.get('x-content-type-options')).toBe('nosniff');
  });
  it('a poisoned shell body is not served offline', async () => {
    const sw = boot();
    await put(sw, '/', '<!doctype html><script src="/assets/x.css"></script>', { 'content-type': 'text/html' });
    sw.network.fetch = offline;
    const out = await sw.fetch('/', { mode: 'navigate' });
    expect(out.response).toBeUndefined();
  });
  it('a stored public API answer is served only as JSON, with fixed headers', async () => {
    const sw = boot();
    await put(sw, '/api/push/vapid', '<script>evil()</script>', { 'content-type': 'text/html' });
    sw.network.fetch = offline;
    expect((await sw.fetch('/api/push/vapid')).response).toBeUndefined();
    await put(sw, '/api/push/vapid', '{"publicKey":"BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB"}', { 'content-type': 'text/html' });
    const out = await sw.fetch('/api/push/vapid');
    expect(out.response?.headers.get('content-type')).toBe('application/json');
    expect(await out.response?.json()).toEqual({"publicKey":"BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB"});
  });
  it('the app\'s own worker gets its wasm from the cache offline; a blob or unknown worker does not', async () => {
    const sw = boot();
    const wasm = '/assets/mw_crypto_bg-a1.wasm';
    await sw.fetch(wasm, { client: '/assets/worker.entry-b2.js', clientType: 'worker' });
    sw.network.fetch = offline;
    expect(await (await sw.fetch(wasm, { client: '/assets/worker.entry-b2.js', clientType: 'worker' })).response?.text()).toBe('export{}');
    expect((await sw.fetch(wasm, { client: '/elsewhere/w.js', clientType: 'worker' })).response).toBeUndefined();
  });
  it('under a deploy prefix, a stylesheet with prefixed font URLs and the shimmed shell still verify', async () => {
    const css = 'a{src:url(/fonts/a.woff2)}';
    const shell = '<!doctype html><head><title>shell</title></head>';
    const sw = bootWorker('https://mail.example.com/mail/sw.js', { digests: defaultDigests({ '/assets/index-x.css': digestOf(css), '/index.html': digestOf(shell) }) });
    sw.network.fetch = async (req) => {
      const url = typeof req === 'string' ? req : req.url;
      if (url.endsWith('.css')) return reply('a{src:url(/mail/fonts/a.woff2)}', 'text/css');
      return reply('<!doctype html><head><script src="/mail/__mw_base.js"></script><title>shell</title></head>', 'text/html');
    };
    await sw.fetch('/mail/assets/index-x.css');
    await sw.fetch('/mail/', { mode: 'navigate' });
    sw.network.fetch = offline;
    expect(await (await sw.fetch('/mail/assets/index-x.css')).response?.text()).toBe('a{src:url(/mail/fonts/a.woff2)}');
    expect((await sw.fetch('/mail/', { mode: 'navigate' })).response?.status).toBe(200);
  });
});

// t63 F5/F6: the cached public answers keep only the server's shape, and the
// asset cap follows the build. Both fail on the 6679e250 worker.
describe('final-check hardening', () => {
  const put = async (sw: Worker, path: string, body: string) => {
    const cache = await sw.caches.open(shellCacheName(1));
    await cache.put(`${ORIGIN}${path}`, new Response(body, { status: 200, headers: { 'content-type': 'application/json', 'x-mw-cached-at': String(Date.now()) } }));
  };
  it('a stored provider list with made-up fields or shapes is not served offline', async () => {
    const sw = boot();
    sw.network.fetch = offline;
    await put(sw, '/api/sso/providers', '{"providers":[{"id":"x","kind":"oidc","displayName":"Sign in","href":"https://evil.example"}],"banner":"Your session expired"}');
    expect(await (await sw.fetch('/api/sso/providers')).response?.json()).toEqual({ providers: [{ id: 'x', kind: 'oidc', displayName: 'Sign in' }] });
    await put(sw, '/api/sso/providers', '{"providers":[{"id":"x","kind":"oidc","displayName":{"html":"<b>"}}]}');
    expect((await sw.fetch('/api/sso/providers')).response).toBeUndefined();
    await put(sw, '/api/push/vapid', '{"publicKey":"short","extra":1}');
    expect((await sw.fetch('/api/push/vapid')).response).toBeUndefined();
  });
  it('the asset cap is twice the manifest, so a large build is not evicted while current', async () => {
    const digests: Record<string, string> = {};
    for (let i = 0; i < 400; i += 1) digests[`/assets/a${i}.js`] = digestOf('export{}');
    const sw = bootWorker(undefined, { digests: defaultDigests(digests) });
    sw.network.fetch = async () => js();
    for (let i = 0; i < 600; i += 1) await sw.fetch(`/assets/b${i}.js`);
    vi.setSystemTime(T0 + 61_000);
    await sw.fetch('/assets/last.js'); // runs the sweep
    expect(sw.caches.urls().length).toBe(601);
  }, 30_000);
});
