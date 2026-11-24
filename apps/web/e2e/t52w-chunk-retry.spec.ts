import { expect, test, type Page, type Route } from '@playwright/test';
import { engineLogin } from './helpers.ts';

/**
 * t52w: a lazy chunk that is answered 503 once (the server's asset cache says
 * `assetLoadBusy` with `Retry-After: 1` under a burst) must not end on "This
 * page could not load". Measured before the change (t52w-e0, Docker layout):
 * 2 and 4 of 7 cold mailbox loads ended there, because the client never asked
 * again, and a browser keeps a failed module fetch for the life of the page
 * (src/boot/chunkRetry.ts). A missing chunk (404) must still reach the error
 * page, so a real "file gone after a deploy" case is not hidden.
 *
 * Real server, real build, real CSP (the `engine` project). Requests are
 * intercepted with page.route, which also turns the browser's HTTP cache off,
 * so each file is answered by the server on every request after the first.
 */

type Injected = { status: number; urls: string[]; count: Map<string, number> };

/** Answer the FIRST request for each URL matching `pattern` with `status`; pass the rest. */
async function failFirst(page: Page, pattern: RegExp, status: number): Promise<Injected> {
  const injected: Injected = { status, urls: [], count: new Map() };
  await page.route(pattern, async (route: Route) => {
    const url = route.request().url();
    const n = (injected.count.get(url) ?? 0) + 1;
    injected.count.set(url, n);
    if (n === 1 || (status === 404)) {
      if (n === 1) injected.urls.push(url);
      await route.fulfill({ status, headers: { 'retry-after': '1', 'cache-control': 'no-store', 'content-type': 'text/plain' }, body: status === 503 ? 'assetLoadBusy' : 'Not Found' });
      return;
    }
    await route.continue();
  });
  return injected;
}

test.describe('t52w chunk-load retry', () => {
  // A service worker would answer from its cache, past page.route.
  test.use({ serviceWorkers: 'block' });
  test('sign-in renders when the login chunk and its stylesheet are answered 503 once', async ({ page }) => {
    const injected = await failFirst(page, /\/assets\/login-[^/]+\.(?:js|css)$/, 503);
    await page.goto('./');
    await expect(page.locator('#login-username')).toBeVisible({ timeout: 20_000 });
    await expect(page.getByText('This page could not load')).toHaveCount(0);
    expect(injected.urls.length, 'the login JS and CSS were each refused once').toBe(2);
    for (const url of injected.urls) expect(injected.count.get(url), url).toBeGreaterThanOrEqual(2);
  });

  test('the mailbox renders when its lazy chunks are answered 503 once', async ({ page }) => {
    await engineLogin(page);
    const injected = await failFirst(page, /\/assets\/(?:mail|shared)-[^/]+\.(?:js|css)$/, 503);
    await page.reload();
    await expect(page.getByRole('button', { name: 'Compose', exact: true })).toBeVisible({ timeout: 20_000 });
    await expect(page.locator('.list__row').first()).toBeVisible();
    await expect(page.getByText('This page could not load')).toHaveCount(0);
    expect(injected.urls.length, 'mail and shared, JS and CSS').toBeGreaterThanOrEqual(3);
  });

  test('a missing chunk (404) is not retried and still shows the error page', async ({ page }) => {
    const injected = await failFirst(page, /\/assets\/login-[^/]+\.js$/, 404);
    await page.goto('./');
    await expect(page.getByText('This page could not load')).toBeVisible({ timeout: 10_000 });
    const [url] = injected.urls;
    expect(url).toBeDefined();
    // The boot script's download, the import's one attempt (a 404 is not
    // retried) and the module loader's own request.
    expect(injected.count.get(url!) ?? 0).toBeLessThanOrEqual(3);
  });
});
