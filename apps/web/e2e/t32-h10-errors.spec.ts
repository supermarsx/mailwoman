import { test, expect, type Page } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import { login } from './helpers';

/**
 * Unknown pages and unknown in-app routes (t32-h10), against a live stack.
 *
 *   - A path the server does not serve (`/does-not-exist`, `/admin/bogus`) gets
 *     mw-server's error page: the real 404, the app's name, one sentence, a link
 *     back to the mail app under the deploy prefix, no CSP violation and no axe
 *     finding, in the dark and the light scheme.
 *   - An unknown mail route (`/#/bogus`) is replaced by the inbox with a notice.
 *   - An unknown admin section (`/admin#/bogus`) is replaced by the dashboard
 *     with a notice.
 *
 * The project `baseURL` may carry a sub-path. `MW_E2E_SUBPATH_BASE_URL` names a
 * second server started with `MW_BASE_PATH` (for example `/mail`); its checks skip
 * when it is unset. The admin checks sign in with `MW_ADMIN_USER` /
 * `MW_ADMIN_PASSWORD` (default `root` / `hunter2`, the CI stacks' values).
 */

const ADMIN = {
  username: process.env['MW_ADMIN_USER'] ?? 'root',
  password: process.env['MW_ADMIN_PASSWORD'] ?? 'hunter2',
};

const WCAG_22_AA = ['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa', 'wcag22aa'];

/** The origin and the deploy prefix (no trailing slash) of a base URL. */
function split(base: string): { origin: string; prefix: string } {
  const u = new URL(base);
  return { origin: u.origin, prefix: u.pathname.replace(/\/+$/, '') };
}

/** Collect CSP violations the page reports, from before its first script runs. */
async function watchCsp(page: Page): Promise<() => Promise<string[]>> {
  const consoleHits: string[] = [];
  page.on('console', (m) => {
    if (/Content[- ]Security[- ]Policy/i.test(m.text())) consoleHits.push(m.text());
  });
  await page.addInitScript(() => {
    const w = window as unknown as { __mwCsp: string[] };
    w.__mwCsp = [];
    document.addEventListener('securitypolicyviolation', (e) => {
      w.__mwCsp.push(`${e.violatedDirective} ${e.blockedURI}`);
    });
  });
  return async () => [
    ...consoleHits,
    ...(await page.evaluate(() => (window as unknown as { __mwCsp?: string[] }).__mwCsp ?? [])),
  ];
}

async function axeClean(page: Page, screen: string): Promise<void> {
  const results = await new AxeBuilder({ page }).withTags(WCAG_22_AA).analyze();
  const lines = results.violations.map((v) => `${v.id}: ${v.help} (${v.nodes.map((n) => n.target.join(' ')).join(', ')})`);
  expect(lines, `axe on ${screen}`).toEqual([]);
}

/** The error page for `path` under `base`, checked in both colour schemes. */
async function expectErrorPage(page: Page, base: string, path: string, opts: { adminLink: boolean }): Promise<void> {
  const { origin, prefix } = split(base);
  const csp = await watchCsp(page);
  for (const scheme of ['dark', 'light'] as const) {
    await page.emulateMedia({ colorScheme: scheme });
    const res = await page.goto(`${origin}${prefix}${path}`);
    expect(res?.status(), `${prefix}${path} status`).toBe(404);
    expect(res?.headers()['content-type'] ?? '').toContain('text/html');
    expect(res?.headers()['cache-control']).toBe('no-store');
    expect(res?.headers()['content-security-policy'] ?? '').toContain("default-src 'none'");
    await expect(page.getByRole('heading', { level: 1, name: 'This page does not exist.' })).toBeVisible();
    await expect(page.getByText('Mailwoman', { exact: true })).toBeVisible();
    const mail = page.getByRole('link', { name: 'Go to your mail' });
    await expect(mail).toHaveAttribute('href', `${prefix}/`);
    const admin = page.getByRole('link', { name: 'Go to the admin console' });
    if (opts.adminLink) await expect(admin).toHaveAttribute('href', `${prefix}/admin`);
    else await expect(admin).toHaveCount(0);
    // The stylesheet applied: the page background follows the scheme.
    const bg = await page.evaluate(() => getComputedStyle(document.body).backgroundColor);
    expect(bg, `${scheme} background`).toBe(scheme === 'dark' ? 'rgb(17, 17, 17)' : 'rgb(244, 241, 234)');
    await axeClean(page, `error page (${scheme}) ${prefix}${path}`);
  }
  expect(await csp(), 'CSP violations on the error page').toEqual([]);

  // The link goes back into the app.
  await page.getByRole('link', { name: 'Go to your mail' }).click();
  await expect(page).toHaveURL(`${origin}${prefix}/`);
  await expect(page.getByRole('button', { name: 'Sign in', exact: true }).or(page.getByRole('button', { name: 'Compose' }))).toBeVisible();
}

test.describe('unknown pages get the error page', () => {
  test('/does-not-exist', async ({ page, baseURL }) => {
    test.skip(baseURL === undefined, 'needs a stack (the project baseURL)');
    await expectErrorPage(page, baseURL!, '/does-not-exist', { adminLink: false });
  });

  test('/admin/bogus links the admin console too', async ({ page, baseURL }) => {
    test.skip(baseURL === undefined, 'needs a stack (the project baseURL)');
    await expectErrorPage(page, baseURL!, '/admin/bogus', { adminLink: true });
  });

  test('/mail/does-not-exist under MW_BASE_PATH', async ({ page }) => {
    const sub = process.env['MW_E2E_SUBPATH_BASE_URL'];
    test.skip(sub === undefined, 'MW_E2E_SUBPATH_BASE_URL names no server started with MW_BASE_PATH');
    await expectErrorPage(page, sub!, '/does-not-exist', { adminLink: false });
  });

  test('an API path answers JSON, not the page', async ({ request, baseURL }) => {
    test.skip(baseURL === undefined, 'needs a stack (the project baseURL)');
    const { prefix } = split(baseURL!);
    const res = await request.get(`${prefix}/api/does-not-exist`, { headers: { accept: 'application/json' } });
    expect(res.status()).toBe(404);
    expect(res.headers()['content-type']).toContain('application/json');
    expect(await res.json()).toEqual({ error: 'This page does not exist.', code: 'not_found' });
  });
});

test.describe('unknown in-app routes', () => {
  test('/#/bogus lands on the inbox with a notice', async ({ page }) => {
    await login(page);
    const csp = await watchCsp(page);
    await page.goto('/#/bogus');
    await expect(page.getByRole('button', { name: 'Compose' })).toBeVisible();
    await expect(page).toHaveURL(/#\/mail$/);
    await expect(page.getByRole('status').filter({ hasText: 'That page does not exist. Showing your inbox.' })).toBeVisible();
    // The mail screens already trip `style-src` with static `style=` attributes in
    // compiled templates (MessageList, Ribbon; a known gap in check-size's literal
    // style scan, reported by t32-e7 and t32-h10). Those are not this redirect's;
    // anything else is.
    const own = (await csp()).filter((v) => !/Applying inline style/i.test(v) && !/^style-src-attr /.test(v));
    expect(own).toEqual([]);
  });

  test('/admin#/bogus lands on the dashboard with a notice', async ({ page, baseURL }) => {
    test.skip(baseURL === undefined, 'needs a stack (the project baseURL)');
    const { prefix } = split(baseURL!);
    const signIn = await page.request.post(`${prefix}/admin/login`, { data: ADMIN });
    // A stack without the admin panel answers 401 to every admin route; the CI
    // mock and engine stacks set no MW_ADMIN_USER. The console's unknown-section
    // rule is also covered by vitest (src/screens/Admin/unknownSection.test.tsx).
    test.skip(
      signIn.status() === 401 && process.env['MW_ADMIN_USER'] === undefined,
      'the admin panel is off on this stack (set MW_ADMIN_USER/MW_ADMIN_PASSWORD to run this)',
    );
    expect(signIn.status(), 'admin login').toBe(200);
    const csp = await watchCsp(page);
    await page.goto(`${prefix}/admin#/bogus`);
    await expect(page.getByRole('region', { name: 'Dashboard' })).toBeVisible();
    expect(new URL(page.url()).hash).toBe('');
    await expect(page.getByTestId('admin-unknown-section')).toContainText('Showing the dashboard');
    expect(await csp()).toEqual([]);
  });
});
