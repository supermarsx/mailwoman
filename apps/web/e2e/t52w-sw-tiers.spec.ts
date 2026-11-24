import { expect, test, type Page, type Request } from '@playwright/test';
import { engineLogin } from './helpers.ts';

/**
 * t52w: the service worker's install set (`web.offline.precacheMode`, default
 * "app"; apps/web/src/sw/offlineTiers.ts) and the offline start it serves.
 *
 * Real server, real build, real CSP (the `engine` project). The default
 * "app" install is checked against the list the build wrote into sw.js; the
 * idle fill then adds the active locale's pack; an offline reload renders the
 * shell, the message list and a message opened online.
 *
 * An older server (enum all/core, default all) runs this only with
 * web.offline.precacheMode set to "core", which the worker reads as "app".
 */

const assetPath = (url: string): string | null => {
  const path = new URL(url).pathname;
  const at = path.search(/\/(?:assets|pdfjs)\//);
  return at < 0 ? null : path.slice(at);
};

/** The worker reaches activated; resolves with its state. */
async function activated(page: Page): Promise<string> {
  return page.evaluate(async () => {
    const base = new URL('./', location.href).pathname;
    const registration = await navigator.serviceWorker.register(`${base}sw.js`, { type: 'classic', scope: base });
    const worker = registration.installing ?? registration.waiting ?? registration.active;
    if (worker === null) return 'none';
    if (worker.state !== 'activated') {
      await new Promise<void>((resolve) => {
        const changed = (): void => { if (worker.state === 'activated' || worker.state === 'redundant') { worker.removeEventListener('statechange', changed); resolve(); } };
        worker.addEventListener('statechange', changed);
        changed();
      });
    }
    return worker.state;
  });
}

test.describe('t52w service-worker tiers', () => {
  test.describe.configure({ mode: 'serial' });

  test('install fetches the app set and nothing first-use', async ({ page, context, request }) => {
    const config = await (await request.get('api/public-config')).json() as { webPerformance?: { precacheMode?: string } };
    expect(['app', 'core'], 'this check needs the default (or the old core) install mode').toContain(config.webPerformance?.precacheMode);
    const sw = await (await request.get('sw.js')).text();
    const app = JSON.parse(/const OFFLINE_APP_ASSETS = (\[[^\n]*?\]);/.exec(sw)![1]!) as string[];
    expect(app.length).toBeGreaterThan(10);

    const fetched = new Set<string>();
    const record = (r: Request): void => { if (r.serviceWorker() !== null) { const path = assetPath(r.url()); if (path !== null) fetched.add(path); } };
    context.on('request', record);
    await page.goto('./');
    expect(await activated(page)).toBe('activated');
    context.off('request', record);

    for (const path of app) expect(fetched, `installed ${path}`).toContain(path);
    const first = [...fetched].filter((p) => !app.includes(p));
    // After activation the worker also answers the page's own requests (the sign-in
    // screen's font and pack); none of the first-use files is installed.
    for (const path of first) expect(path, 'not an install-time first-use file').not.toMatch(/^\/pdfjs\/|mw_crypto_bg-|\/assets\/admin-section|\/assets\/catalog\.[^.]+\.admin-/);
    const cached = await page.evaluate(async () => (await (await caches.open('mw-shell-v1')).keys()).map((r) => r.url));
    expect(cached.filter((u) => u.includes('/pdfjs/'))).toEqual([]);
  });

  test('offline reload renders the shell, the list and a message opened online', async ({ page, context }) => {
    test.setTimeout(120_000);
    await engineLogin(page);
    expect(await activated(page)).toBe('activated');
    await expect.poll(() => page.evaluate(() => navigator.serviceWorker.controller !== null)).toBe(true);
    // The idle fill of this signed-in page stores the active locale's pack through
    // the worker, then the Settings / PIM files; never the admin pack.
    await expect.poll(async () => page.evaluate(async () => {
      const lang = document.documentElement.lang;
      const urls = (await (await caches.open('mw-shell-v1')).keys()).map((r) => r.url);
      return {
        pack: urls.some((u) => u.includes(`/assets/catalog.${lang}.app-`)),
        settings: urls.some((u) => /\/assets\/settings-/.test(u)),
        pim: urls.some((u) => /\/assets\/pim-/.test(u)),
        admin: urls.some((u) => /\.admin-[^/]+\.json$/.test(u)),
      };
    }), { timeout: 40_000 }).toEqual({ pack: true, settings: true, pim: true, admin: false });

    // Offline access is off by default (docs/user/offline-sync.md): turn it on for this account.
    await page.evaluate(async () => {
      const csrf = /(?:^|;\s*)mw_csrf=([^;]+)/.exec(document.cookie)?.[1] ?? '';
      const me = await (await fetch('api/me', { credentials: 'same-origin' })).json() as { storageAuthority?: string };
      const res = await fetch('api/settings/offline.enabled', { method: 'PUT', credentials: 'same-origin', headers: { 'content-type': 'application/json', 'x-csrf-token': decodeURIComponent(csrf), 'x-mailwoman-storage-authority': me.storageAuthority ?? '' }, body: JSON.stringify({ value: true }) });
      if (!res.ok) throw new Error(`settings ${res.status} ${await res.text()}`);
    });
    await page.reload();
    const row = page.locator('.list__row').first();
    await expect(row).toBeVisible();
    await row.click();
    const reader = page.locator('.reader--open');
    await expect(reader).toBeVisible();
    // A line of the open message, to find again offline.
    await expect.poll(async () => (await reader.innerText()).trim().length).toBeGreaterThan(20);
    const line = (await reader.innerText()).split('\n').map((l) => l.trim()).find((l) => l.length > 8) ?? '';

    await context.setOffline(true);
    try {
      await page.reload();
      await expect(page.locator('.list__row').first()).toBeVisible({ timeout: 20_000 });
      await page.locator('.list__row').first().click();
      await expect(reader).toBeVisible();
      expect(line.length).toBeGreaterThan(8);
      await expect(reader).toContainText(line);
      await expect(page.getByText('This page could not load')).toHaveCount(0);
    } finally {
      await context.setOffline(false);
    }
  });
});
