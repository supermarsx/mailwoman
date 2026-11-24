import { expect, test, type APIRequest, type Page } from '@playwright/test';
import { ENGINE_ADMIN, engineLogin, postSignIn } from './helpers.ts';

/**
 * t52w: the boot script preloads the first screen's files from the build
 * manifest in the shell (src/boot/boot.js, src/boot/bootManifest.ts): the
 * sign-in screen's chunk, or the mailbox's when a session was open last time,
 * the one UI font file in use and the active locale's translation pack.
 *
 * Real server, real build, real CSP (the `engine` project). Checked here:
 *   - no CSP violation and no console warning about a preload (unused, or not
 *     matching the request that should use it, which downloads the file twice);
 *   - every preloaded file is requested once;
 *   - the font preload is the URL the stylesheet asks for. Run this file also
 *     with PLAYWRIGHT_ENGINE_BASE_URL pointing at a server under MW_BASE_PATH
 *     (for example http://localhost:8090/mail/) to check that under a prefix.
 */

type Watch = { problems: string[]; requests: Map<string, number> };

/**
 * Console problems, and the /assets/ requests that reached the network (a
 * memory or disk cache hit, or a preload consumed by its request, is not one),
 * through the DevTools protocol: Playwright's own request events include cache
 * hits.
 */
async function watch(page: Page): Promise<Watch> {
  const watched: Watch = { problems: [], requests: new Map() };
  page.on('console', (message) => {
    const text = message.text();
    if (/preload/i.test(text) || /Content Security Policy|Trusted Type/i.test(text)) watched.problems.push(`${message.type()}: ${text}`);
  });
  const cdp = await page.context().newCDPSession(page);
  await cdp.send('Network.enable');
  const cached = new Set<string>();
  cdp.on('Network.requestServedFromCache', (event: { requestId: string }) => { cached.add(event.requestId); });
  cdp.on('Network.responseReceived', (event: { requestId: string; response: { url: string; fromDiskCache?: boolean; fromServiceWorker?: boolean; fromPrefetchCache?: boolean } }) => {
    const { url, fromDiskCache, fromServiceWorker, fromPrefetchCache } = event.response;
    if (!url.includes('/assets/') || cached.has(event.requestId) || fromDiskCache || fromServiceWorker || fromPrefetchCache) return;
    watched.requests.set(url, (watched.requests.get(url) ?? 0) + 1);
  });
  // A CSP report the console does not show (a blocked link, for one).
  await page.addInitScript(() => {
    document.addEventListener('securitypolicyviolation', (event) => {
      console.error(`Content Security Policy violation: ${event.violatedDirective} ${event.blockedURI}`);
    });
  });
  return watched;
}

/** The boot manifest's file lists, resolved against the page. */
async function manifest(page: Page): Promise<{ mail: string[]; login: string[] }> {
  return page.evaluate(() => {
    const value = JSON.parse(document.getElementById('mw-boot-manifest')?.textContent ?? '{}') as { mail?: string[]; login?: string[] };
    const abs = (list: string[] | undefined): string[] => (list ?? []).map((href) => new URL(href, document.baseURI).href);
    return { mail: abs(value.mail), login: abs(value.login) };
  });
}

/** The preload links the boot script added (font, pack), resolved. */
async function preloads(page: Page): Promise<{ as: string | null; href: string }[]> {
  return page.evaluate(() => [...document.head.querySelectorAll('link[rel="preload"]')]
    .map((link) => ({ as: link.getAttribute('as'), href: (link as HTMLLinkElement).href })));
}

/** Let the page go idle long enough for Chrome's unused-preload check (load + 3 s). */
async function settle(page: Page): Promise<void> {
  await page.waitForLoadState('load');
  await page.waitForTimeout(4500);
}

const prefixed = new URL(process.env['PLAYWRIGHT_ENGINE_BASE_URL'] ?? 'http://localhost:8090').pathname.replace(/\/+$/, '') !== '';

test.describe('t52w boot preloads', () => {
  // The first visit: no service worker answers (it is checked in t52w-sw-tiers.spec.ts).
  test.use({ serviceWorkers: 'block' });
  test('sign-in: the login chunk, the UI font and the translation pack, each requested once', async ({ page }) => {
    const watched = await watch(page);
    await page.goto('./');
    await expect(page.locator('#login-username')).toBeVisible();
    await settle(page);
    const files = await manifest(page);
    expect(files.login.length, 'login JS + CSS').toBe(2);
    for (const url of files.login) expect(watched.requests.get(url) ?? 0, url).toBe(1);
    for (const url of files.mail) expect(watched.requests.get(url) ?? 0, `no mailbox file before a session: ${url}`).toBe(0);
    const links = await preloads(page);
    const font = links.filter((l) => l.as === 'font');
    expect(font.map((l) => /\/assets\/(jost-latin-wght-normal)-/.exec(new URL(l.href).pathname)?.[1])).toEqual(['jost-latin-wght-normal']);
    // The stylesheet asks for exactly the preloaded font URL (else it downloads twice).
    expect(watched.requests.get(font[0]!.href), font[0]!.href).toBe(1);
    const pack = links.filter((l) => l.as === 'fetch');
    expect(pack.map((l) => /\/assets\/catalog\.([^.]+)\.app-/.exec(new URL(l.href).pathname)?.[1])).toEqual(['en']);
    for (const link of links) expect(watched.requests.get(link.href) ?? 0, link.href).toBe(1);
    expect(watched.problems).toEqual([]);
  });

  test('mailbox: with a session last time, the mailbox chunks are fetched once and used', async ({ page }) => {
    test.skip(prefixed, 'engineLogin signs in at the origin root');
    await engineLogin(page);
    const watched = await watch(page);
    // Cold files, warm session: what a returning user's first load after a deploy looks like.
    const cdp = await page.context().newCDPSession(page);
    await cdp.send('Network.clearBrowserCache');
    await page.reload();
    await expect(page.locator('.list__row').first()).toBeVisible();
    await settle(page);
    const files = await manifest(page);
    expect(files.mail.length, 'mail + shared, JS + CSS').toBe(4);
    for (const url of files.mail) expect(watched.requests.get(url) ?? 0, url).toBe(1);
    for (const url of files.login) expect(watched.requests.get(url) ?? 0, `no sign-in file with a session: ${url}`).toBe(0);
    for (const link of await preloads(page)) expect(watched.requests.get(link.href) ?? 0, link.href).toBe(1);
    expect(watched.problems).toEqual([]);
  });
});

/** Set a global setting through the admin API (test stacks only); answers the value it replaced. */
async function setGlobal(factory: APIRequest, baseURL: string, key: string, value: unknown): Promise<unknown> {
  const ctx = await factory.newContext({ baseURL });
  try {
    expect((await postSignIn(ctx, '/admin/login', ENGINE_ADMIN)).status(), 'admin API sign-in').toBe(200);
    const url = `/admin/settings/${key}`;
    const current = await ctx.get(url);
    expect(current.status(), `GET ${url}`).toBe(200);
    const before = ((await current.json()) as { effective?: { value?: unknown } }).effective?.value;
    const csrf = (await ctx.storageState()).cookies.find((c) => c.name === 'mw_csrf')?.value ?? '';
    const put = await ctx.put(url, { headers: { 'if-match': current.headers()['etag'] ?? '', 'x-csrf-token': csrf }, data: { globalValue: value } });
    expect(put.status(), `PUT ${url}`).toBe(200);
    return before;
  } finally { await ctx.dispose(); }
}

test.describe('t52w restored session', () => {
  test.use({ serviceWorkers: 'block' });
  // A reload with a session sends the boot reads over the WebSocket action channel
  // (the CSRF cookie is readable before /api/me answers), and the account's
  // activation clears that channel. A read caught by it is sent over HTTP; before
  // the fix it was refused, the performance policy stayed unknown and the idle
  // prefetch never started. This checks the outcome on a real stack; the race
  // itself is forced in src/api/http.channelChange.test.ts.
  test('after a reload the performance policy is applied and the idle prefetch starts', async ({ page, playwright, baseURL }) => {
    test.skip(prefixed, 'engineLogin signs in at the origin root');
    const modules = await setGlobal(playwright.request, baseURL!, 'web.prefetch.modules', true);
    const delay = await setGlobal(playwright.request, baseURL!, 'web.prefetch.idleDelayMs', 200);
    try {
      const refused: string[] = [];
      page.on('console', (message) => { if (/action channel changed|actionNotDispatched/i.test(message.text())) refused.push(message.text()); });
      await engineLogin(page);
      await expect(page.locator('.list__row').first()).toBeVisible();
      // Every request, memory and disk cache hits included: the sign-in load above
      // already prefetched these files, so the reload may read them from the cache.
      const requested: string[] = [];
      page.on('request', (request) => { requested.push(new URL(request.url()).pathname); });
      await page.reload();
      await expect(page.locator('.list__row').first()).toBeVisible();
      // Idle prefetch with modules on loads the Compose and Settings panes nobody opened.
      await expect.poll(() => requested.some((path) => /\/assets\/(compose|settings)-[\w-]+\.js$/.test(path)), { timeout: 20_000 }).toBe(true);
      expect(refused).toEqual([]);
    } finally {
      await setGlobal(playwright.request, baseURL!, 'web.prefetch.modules', modules ?? false);
      await setGlobal(playwright.request, baseURL!, 'web.prefetch.idleDelayMs', delay ?? 2500);
    }
  });
});
