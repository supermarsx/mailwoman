import { test, expect, type Page } from '@playwright/test';
import { readFileSync } from 'node:fs';
import { createServer, type Server } from 'node:http';
import type { AddressInfo } from 'node:net';
import { fileURLToPath } from 'node:url';
import { engineLogin, injectViaSmtp, messageRow, waitForInboxMessage } from './helpers.ts';

/**
 * Service-worker cache scope and logout purge (t28-e3, audit §10.4 / SEC-8).
 *
 * Cache Storage is plaintext on disk. Before t28 the worker stored every 200 from
 * `/jmap/*` and `/api/*` — attachment bytes, whole `.eml` source, `/api/me` — and
 * nothing ever deleted them, so they outlived the session. These specs assert, in
 * a real browser with the real Cache API, that those responses are not stored and
 * that logging out empties the caches.
 *
 * Two groups:
 *   - "shipped worker, stub origin": serves the real `public/sw.js` from a
 *     throwaway HTTP server started by the spec. Needs no Mailwoman stack, so it
 *     runs anywhere Chromium does. The stub stands in for the SERVER only.
 *   - "engine mode": the built bundle on the engine stack (:8090), through the
 *     real UI — a real attachment download, a real `.eml` export, the real
 *     "Log out" button.
 */

/** Every URL held in any cache of the page's origin. */
function cachedUrls(page: Page): Promise<string[]> {
  return page.evaluate(async () => {
    const urls: string[] = [];
    for (const name of await caches.keys()) {
      const cache = await caches.open(name);
      for (const request of await cache.keys()) urls.push(request.url);
    }
    return urls;
  });
}

function mailwomanCacheNames(page: Page): Promise<string[]> {
  return page.evaluate(async () => (await caches.keys()).filter((name) => name.startsWith('mw-')));
}

test.describe('service-worker cache: shipped worker, stub origin', () => {
  const SW_SOURCE = readFileSync(fileURLToPath(new URL('../public/sw.js', import.meta.url)));
  const ASSET = '/assets/app-a1b2c3d4.js';
  const DOWNLOAD = '/jmap/download/acct1/blob9/report-2026final.pdf';
  const EML = '/jmap/download/acct1/blob7/message.eml';

  let server: Server;
  let origin: string;
  /** Paths the stub actually served — proof a request reached the network. */
  let served: string[];

  test.beforeAll(async () => {
    server = createServer((req, res) => {
      const path = (req.url ?? '/').split('?')[0] ?? '/';
      served.push(`${req.method ?? ''} ${path}`);
      const send = (type: string, body: string | Buffer, status = 200): void => {
        res.writeHead(status, { 'content-type': type });
        res.end(body);
      };
      if (path === '/sw.js') return send('text/javascript', SW_SOURCE);
      if (path === ASSET) return send('text/javascript', 'export {};');
      if (path.startsWith('/jmap/download/')) return send('application/octet-stream', 'ATTACHMENT-BYTES');
      if (path === '/api/me') return send('application/json', '{"username":"testuser"}');
      if (path === '/api/logout') return send('application/json', '{"ok":true}');
      return send('text/html', '<!doctype html><title>stub</title>');
    });
    await new Promise<void>((resolve) => server.listen(0, '127.0.0.1', resolve));
    // `localhost` (not the bare IP) so the page is a secure context in every engine.
    origin = `http://localhost:${(server.address() as AddressInfo).port}`;
  });

  test.afterAll(async () => {
    await new Promise((resolve) => server.close(resolve));
  });

  test.beforeEach(() => {
    served = [];
  });

  /** Register the worker and wait until it controls the page. */
  async function register(page: Page): Promise<void> {
    await page.evaluate(async () => {
      await navigator.serviceWorker.register('/sw.js', { scope: '/' });
      await navigator.serviceWorker.ready;
      if (navigator.serviceWorker.controller === null) {
        await new Promise((resolve) =>
          navigator.serviceWorker.addEventListener('controllerchange', resolve, { once: true }),
        );
      }
    });
  }

  const get = (page: Page, path: string): Promise<number> =>
    page.evaluate(async (p) => (await fetch(p)).status, path);

  test('an asset is cached; attachment, .eml and /api/me responses are not', async ({ page }) => {
    await page.goto(origin);
    await register(page);

    // Precondition / negative control: this worker, in this browser, does cache.
    expect(await get(page, ASSET)).toBe(200);
    await expect.poll(() => cachedUrls(page)).toContain(`${origin}${ASSET}`);

    for (const path of [DOWNLOAD, EML, '/api/me']) {
      expect(await get(page, path)).toBe(200);
      // The request really went out and came back — it was not short-circuited.
      expect(served).toContain(`GET ${path}`);
    }

    const urls = await cachedUrls(page);
    expect(urls.filter((url) => url.includes('/jmap/') || url.includes('/api/'))).toEqual([]);
    // Still holding the asset: the absence above is not an empty cache.
    expect(urls).toContain(`${origin}${ASSET}`);
  });

  test('the logout request deletes every mw-* cache', async ({ page }) => {
    await page.goto(origin);
    await register(page);
    await get(page, ASSET);
    await expect.poll(() => cachedUrls(page)).toContain(`${origin}${ASSET}`);
    // Precondition: there is a Mailwoman cache to delete.
    expect(await mailwomanCacheNames(page)).toEqual(['mw-shell-v1']);

    // A write that is not a logout leaves it alone.
    await page.evaluate(() => fetch('/jmap/api', { method: 'POST' }));
    expect(await mailwomanCacheNames(page)).toEqual(['mw-shell-v1']);

    await page.evaluate(() => fetch('/api/logout', { method: 'POST' }));

    expect(served).toContain('POST /api/logout');
    await expect.poll(() => mailwomanCacheNames(page)).toEqual([]);
  });

  test('upgrading removes what the previous worker stored', async ({ page }) => {
    await page.goto(origin);
    // What a pre-t28 worker left in `mw-shell-v1`, written before this worker exists.
    await page.evaluate(
      async ([download, eml]) => {
        const cache = await caches.open('mw-shell-v1');
        for (const path of [download, eml, '/api/me']) {
          await cache.put(path as string, new Response('legacy plaintext'));
        }
      },
      [DOWNLOAD, EML],
    );
    expect(await cachedUrls(page)).toHaveLength(3);

    await register(page);

    await expect
      .poll(async () =>
        (await cachedUrls(page)).filter((url) => url.includes('/jmap/') || url.includes('/api/')),
      )
      .toEqual([]);
  });
});

test.describe('service-worker cache: engine mode', () => {
  test.describe.configure({ mode: 'serial', retries: 2 });

  // A 1x1 transparent PNG.
  const PNG_1x1_B64 =
    'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==';

  const isUserData = (url: string): boolean => {
    const path = new URL(url).pathname;
    if (path.endsWith('/api/sso/providers') || path.endsWith('/api/push/vapid')) return false;
    return path.includes('/jmap/') || path.includes('/api/');
  };

  test('attachment and .eml downloads are not cached, and logout empties the caches', async ({ page }) => {
    test.slow();
    await engineLogin(page);

    // The first load ran before the worker existed, so nothing went through it.
    // Reload once it is active: the session cookie keeps us signed in.
    await page.evaluate(() => navigator.serviceWorker.ready);
    await page.reload();
    await expect(page.getByRole('button', { name: 'Compose' })).toBeVisible();
    // Precondition: the worker controls the page and has cached the build assets.
    expect(await page.evaluate(() => navigator.serviceWorker.controller !== null)).toBe(true);
    await expect
      .poll(async () => (await cachedUrls(page)).filter((url) => url.includes('/assets/')).length)
      .toBeGreaterThan(0);

    const subject = `SW cache ${Date.now()}`;
    await injectViaSmtp({
      from: 'Files Bot <files@example.org>',
      subject,
      text: `service worker cache ${subject}`,
      attachments: [{ filename: 'photo.png', contentType: 'image/png', base64: PNG_1x1_B64 }],
    });
    await waitForInboxMessage(page, subject, 150_000);
    await messageRow(page, subject).first().click();
    await expect(page.getByTestId('reader-attachments')).toBeVisible();

    // Open the attachment: its bytes travel over GET /jmap/download/….
    const [attachment] = await Promise.all([
      page.waitForResponse((res) => res.url().includes('/jmap/download/') && res.status() === 200),
      page.getByRole('option', { name: 'photo.png' }).click(),
    ]);
    expect(attachment.ok()).toBe(true);
    await expect(page.getByTestId('attachment-viewer')).toBeVisible();
    await page.getByRole('button', { name: 'Close attachment' }).click();

    // Export the whole message as .eml — also a GET /jmap/download/….
    const [download] = await Promise.all([
      page.waitForEvent('download'),
      page.getByTestId('reader-export').click(),
    ]);
    expect(download.suggestedFilename()).toMatch(/\.eml$/i);

    const before = await cachedUrls(page);
    expect(before.filter(isUserData)).toEqual([]);
    // Negative control: assets are still there, so the list above is not empty by accident.
    expect(before.filter((url) => url.includes('/assets/')).length).toBeGreaterThan(0);

    await page.locator('.sidebar__logout').click();
    await expect(page.getByRole('button', { name: 'Sign in' })).toBeVisible();

    // Nothing that was cached while signed in survives. (The login screen may
    // fetch — and the worker may store — assets it had not loaded before.)
    await expect
      .poll(async () => {
        const after = new Set(await cachedUrls(page));
        return before.filter((url) => after.has(url));
      })
      .toEqual([]);
  });
});
