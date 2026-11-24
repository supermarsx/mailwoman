import { expect, test } from '@playwright/test';

/**
 * t59: the service worker must install under the server's real CSP.
 *
 * mw-server sends /sw.js with the shell CSP, which carries
 * `require-trusted-types-for 'script'`. The built sw.js loads the
 * offline-background bundle with importScripts, a TrustedScriptURL sink. With a
 * plain string the worker threw during evaluation ("This document requires
 * 'TrustedScriptURL' assignment"), went redundant, and never installed, so there
 * was no offline start, no cached shell and no background replay. The stub-origin
 * group of sw-cache.spec.ts serves the unbuilt public/sw.js without a CSP, so it
 * could not see this.
 *
 * Needs the real server serving the real build (the `engine` project's baseURL).
 * Signed out: the sign-in shell is enough to register the worker.
 */

test.describe('service worker under the real CSP', () => {
  test('the server sends Trusted Types with /sw.js and the worker reaches activated', async ({ page, request }) => {
    const sw = await request.get('sw.js');
    expect(sw.status()).toBe(200);
    // Precondition: this is the configuration that broke it.
    expect(sw.headers()['content-security-policy'] ?? '').toContain("require-trusted-types-for 'script'");

    const errors: string[] = [];
    page.on('console', (message) => { if (message.type() === 'error') errors.push(message.text()); });
    await page.goto('./');

    const state = await page.evaluate(async () => {
      // Same call as src/sw/register.ts; it is a no-op if the app already registered.
      const base = new URL('./', location.href).pathname;
      const registration = await navigator.serviceWorker.register(`${base}sw.js`, { type: 'classic', scope: base });
      const worker = registration.installing ?? registration.waiting ?? registration.active;
      if (worker === null) return 'none';
      if (worker.state !== 'activated') {
        await new Promise<void>((resolve) => {
          const changed = (): void => {
            if (worker.state === 'activated' || worker.state === 'redundant') { worker.removeEventListener('statechange', changed); resolve(); }
          };
          worker.addEventListener('statechange', changed);
          changed();
        });
      }
      return worker.state;
    });
    expect(state, `worker state; console errors: ${errors.join(' | ')}`).toBe('activated');
    await page.evaluate(() => navigator.serviceWorker.ready);
    expect(errors.filter((text) => text.includes('TrustedScriptURL'))).toEqual([]);
  });

  test('with the worker installed, a reload with the network off still renders the shell', async ({ page, context }) => {
    await page.goto('./');
    await page.evaluate(async () => {
      const base = new URL('./', location.href).pathname;
      await navigator.serviceWorker.register(`${base}sw.js`, { type: 'classic', scope: base });
      await navigator.serviceWorker.ready;
    });
    // The first load ran before the worker existed; load once through it.
    await page.reload();
    await expect.poll(() => page.evaluate(() => navigator.serviceWorker.controller !== null)).toBe(true);
    const title = await page.title();

    await context.setOffline(true);
    try {
      await page.reload();
      await expect(page).toHaveTitle(title);
      await expect(page.locator('#root, body').first()).not.toBeEmpty();
      expect(await page.evaluate(() => document.querySelector('script[src*="assets/"]') !== null)).toBe(true);
    } finally {
      await context.setOffline(false);
    }
  });
});
