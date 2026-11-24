import { expect, test } from '@playwright/test';
import { engineLogin } from './helpers.ts';

/**
 * t52w: registering the service worker is an enhancement. A failed fetch of
 * /sw.js (429, 503, a network error) must leave the sign-in screen and the
 * mailbox working, with the failure only on the console (src/sw/register.ts
 * swallows it). Real server and CSP (`engine` project); the failure is
 * injected with context.route, which also sees the worker script request.
 */

const kinds = [
  { name: '429', fail: { status: 429, headers: { 'retry-after': '300' }, body: '{"error":"rateLimited"}' } },
  { name: '503', fail: { status: 503, headers: { 'retry-after': '1' }, body: 'assetLoadBusy' } },
  { name: 'network error', fail: null },
] as const;

for (const kind of kinds) {
  test.describe(`service worker script ${kind.name}`, () => {
    test('sign-in still renders and no worker is registered', async ({ page, context }) => {
      let hits = 0;
      await context.route(/\/sw\.js(?:\?|$)/, async (route) => {
        hits += 1;
        if (kind.fail === null) await route.abort('connectionreset');
        else await route.fulfill({ ...kind.fail, contentType: 'text/plain' });
      });
      await page.goto('./');
      await expect(page.locator('#login-username')).toBeVisible();
      await expect.poll(() => hits).toBeGreaterThan(0);
      await expect(page.getByText(/Something went wrong|This page could not load/)).toHaveCount(0);
      expect(await page.evaluate(async () => (await navigator.serviceWorker.getRegistrations()).length)).toBe(0);
    });

    test('the mailbox still renders after sign-in', async ({ page, context }) => {
      await context.route(/\/sw\.js(?:\?|$)/, async (route) => {
        if (kind.fail === null) await route.abort('connectionreset');
        else await route.fulfill({ ...kind.fail, contentType: 'text/plain' });
      });
      await engineLogin(page);
      await page.reload();
      await expect(page.locator('.list__row').first()).toBeVisible();
      await expect(page.getByText(/Something went wrong|This page could not load/)).toHaveCount(0);
    });
  });
}
