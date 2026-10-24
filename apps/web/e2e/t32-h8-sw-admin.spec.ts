import { test, expect, type Page } from '@playwright/test';
import { readFileSync } from 'node:fs';
import { createServer, type Server } from 'node:http';
import type { AddressInfo } from 'node:net';
import { fileURLToPath } from 'node:url';
import { ADMIN_PATH } from './v6-helpers.ts';

/**
 * The admin console under a controlling service worker (t32-h8).
 *
 * With the worker in control, navigating to `/admin` showed mw-server's plain-text
 * 404. The worker answered the navigation by fetching it again itself, and a
 * fetch made from a worker carries `Sec-Fetch-Dest: empty` rather than
 * `document`, which the server's SPA fallback (`wants_spa_shell` in mw-server's
 * lib.rs) refuses. The worker now leaves admin requests to the browser.
 *
 * t32-h15: the console may be at any path (`MW_ADMIN_PATH`), so the worker owns
 * only the mail shell's navigations; `/ops-7f3k` below stands for a moved console,
 * and the live stack's console is at `MW_E2E_ADMIN_PATH` (default `/admin`).
 *
 * Two groups:
 *   - "stub origin": serves the real `public/sw.js` from an HTTP server started by
 *     the spec, at the origin root and under `/mail/`. The stub stands in for the
 *     SERVER only: like mw-server, an unknown path gets the page when the request
 *     says it is a document and a plain-text 404 otherwise. Needs no stack.
 *   - "live stack": the built bundle on the project's server (`baseURL`, which may
 *     carry a sub-path such as `/mail`), through the real UI. With
 *     `MW_E2E_SUBPATH_BASE_URL` set (a second server started with
 *     `MW_BASE_PATH`), the same check also runs there.
 */

/** Wait until the page has a worker in control, reloading once if needed. */
async function controlled(page: Page): Promise<void> {
  await page.evaluate(() => navigator.serviceWorker.ready);
  if (!(await page.evaluate(() => navigator.serviceWorker.controller !== null))) {
    await page.reload();
  }
  expect(await page.evaluate(() => navigator.serviceWorker.controller !== null)).toBe(true);
}

test.describe('service worker and the admin console: stub origin', () => {
  const SW_SOURCE = readFileSync(fileURLToPath(new URL('../public/sw.js', import.meta.url)));
  const ADMIN_PAGE = '<!doctype html><title>stub</title><h1>admin console</h1>';
  const MAIL_PAGE = '<!doctype html><title>stub</title><h1>mail shell</h1>';

  let server: Server;
  let origin: string;
  /** `<path> <Sec-Fetch-Dest>` of every request the stub served. */
  let served: string[];

  test.beforeAll(async () => {
    server = createServer((req, res) => {
      const path = (req.url ?? '/').split('?')[0] ?? '/';
      const dest = String(req.headers['sec-fetch-dest'] ?? '');
      served.push(`${path} ${dest}`);
      const send = (type: string, body: string | Buffer, status = 200): void => {
        res.writeHead(status, { 'content-type': type });
        res.end(body);
      };
      if (path === '/sw.js' || path === '/mail/sw.js') return send('text/javascript', SW_SOURCE);
      if (path === '/' || path === '/mail/') return send('text/html', MAIL_PAGE);
      // mw-server's SPA fallback: the page only for a document request.
      if (dest !== 'document') return send('text/plain', 'not found', 404);
      return send('text/html', /\/(admin|ops-7f3k)(\/|$)/.test(path) ? ADMIN_PAGE : MAIL_PAGE);
    });
    await new Promise<void>((resolve) => server.listen(0, '127.0.0.1', resolve));
    // `localhost` (not the bare IP) so the page is a secure context.
    origin = `http://localhost:${(server.address() as AddressInfo).port}`;
  });

  test.afterAll(async () => {
    await new Promise((resolve) => server.close(resolve));
  });

  test.beforeEach(() => {
    served = [];
  });

  for (const base of ['/', '/mail/']) {
    test(`navigating to ${base}admin under the worker shows the admin page`, async ({ page }) => {
      await page.goto(`${origin}${base}`);
      await page.evaluate(
        async (b) => navigator.serviceWorker.register(`${b}sw.js`, { scope: b }),
        base,
      );
      await controlled(page);

      // Control: the mail shell is still answered by the worker (its own fetch,
      // `Sec-Fetch-Dest: empty`), which is what made the admin console 404.
      served = [];
      await page.goto(`${origin}${base}`);
      expect(served).toContain(`${base} empty`);

      for (const path of [
        `${base}admin`,
        `${base}admin/`,
        `${base}admin#/users`,
        // Fragment first: from `ops-7f3k` itself it would be a same-document move.
        `${base}ops-7f3k#/users`,
        `${base}ops-7f3k`,
      ]) {
        served = [];
        const res = await page.goto(`${origin}${path}`);
        expect(res?.status(), path).toBe(200);
        await expect(page.getByRole('heading', { name: 'admin console' })).toBeVisible();
        expect(served).toContain(`${path.split('#')[0]} document`);
      }
      // Still controlled: the admin page was left to the browser, not unregistered.
      await page.goto(`${origin}${base}`);
      expect(await page.evaluate(() => navigator.serviceWorker.controller !== null)).toBe(true);
    });
  }
});

test.describe('service worker and the admin console: live stack', () => {
  async function adminUnderWorker(page: Page, base: string): Promise<void> {
    // The app registers its worker on start, signed in or not.
    await page.goto(base);
    await controlled(page);

    const res = await page.goto(`${base}${ADMIN_PATH.slice(1)}`);
    expect(res?.status()).toBe(200);
    await expect(page.getByRole('form', { name: 'Admin sign in' })).toBeVisible();

    // Reloaded with the worker still in control, the console loads again.
    expect(await page.evaluate(() => navigator.serviceWorker.controller !== null)).toBe(true);
    await page.reload();
    await expect(page.getByRole('form', { name: 'Admin sign in' })).toBeVisible();
  }

  test('the admin sign-in card loads with the worker in control', async ({ page, baseURL }) => {
    test.skip(baseURL === undefined, 'needs a stack (the project baseURL)');
    await adminUnderWorker(page, `${baseURL!.replace(/\/+$/, '')}/`);
  });

  test('and under a sub-path', async ({ page }) => {
    const sub = process.env['MW_E2E_SUBPATH_BASE_URL'];
    test.skip(sub === undefined, 'MW_E2E_SUBPATH_BASE_URL names no server started with MW_BASE_PATH');
    await adminUnderWorker(page, `${sub!.replace(/\/+$/, '')}/`);
  });
});
