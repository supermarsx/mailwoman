import { test, expect } from './csrf.ts';
import { login } from './helpers.ts';
import { ADMIN_PATH, V6 } from './v6-helpers.ts';

/**
 * t32-h15 — the admin console at the operator's path (`MW_ADMIN_PATH` on the
 * server, `MW_E2E_ADMIN_PATH` here; `/admin` when unset).
 *
 * Signs in to the console through its form, opens the dashboard and one section,
 * and signs out, all at the configured path; every admin request the page makes
 * goes there and none to `/admin`. When the console has moved, `/admin` is the
 * same 404 page as any unknown path. The mail client signs in as usual.
 *
 * Runs against a proxy-mode server fronting the JMAP mock, with
 * `MW_ADMIN_USER`/`MW_ADMIN_PASSWORD` set (the `v6` stack shape).
 */
const MOVED = ADMIN_PATH !== '/admin';

test.describe('admin console at MW_ADMIN_PATH', () => {
  test('sign in, dashboard, a section and sign out, all at the configured path', async ({ page }) => {
    const adminRequests: string[] = [];
    page.on('request', (req) => {
      const path = new URL(req.url()).pathname;
      if (path === ADMIN_PATH || path.startsWith(`${ADMIN_PATH}/`) || path === '/admin' || path.startsWith('/admin/')) {
        adminRequests.push(`${req.method()} ${path}`);
      }
    });

    const resp = await page.goto(ADMIN_PATH);
    expect(resp?.status(), 'the console page').toBe(200);
    const loginForm = page.getByRole('form', { name: 'Admin sign in' });
    await expect(loginForm).toBeVisible();
    await loginForm.getByLabel('Admin username').fill(V6.adminUser);
    await loginForm.getByLabel('Password').fill(V6.adminPass);
    await loginForm.getByRole('button', { name: 'Sign in' }).click();

    const nav = page.getByRole('navigation', { name: 'Admin sections' });
    await expect(nav).toBeVisible();
    expect(new URL(page.url()).pathname, 'the dashboard is the console page').toBe(ADMIN_PATH);

    await nav.getByRole('link', { name: 'Users' }).click();
    await expect(page).toHaveURL(new RegExp(`${ADMIN_PATH.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}#/users$`));
    await expect(page.getByRole('heading', { name: 'Users' }).first()).toBeVisible();

    // A reload at the section keeps the session and the section.
    await page.reload();
    await expect(page.getByRole('navigation', { name: 'Admin sections' })).toBeVisible();
    await expect(page).toHaveURL(/#\/users$/);

    await page.getByRole('button', { name: 'Sign out' }).click();
    await expect(page.getByRole('form', { name: 'Admin sign in' })).toBeVisible();
    expect((await page.request.get(`${ADMIN_PATH}/session`)).status(), 'the session ended on the server').toBe(401);

    expect(adminRequests.some((r) => r === `POST ${ADMIN_PATH}/login`), adminRequests.join('\n')).toBe(true);
    expect(adminRequests.some((r) => r === `GET ${ADMIN_PATH}/users`), adminRequests.join('\n')).toBe(true);
    if (MOVED) {
      expect(
        adminRequests.filter((r) => / \/admin(\/|$)/.test(r)),
        'nothing is asked of /admin once the console has moved',
      ).toEqual([]);
    }
  });

  test('/admin is the unknown-path 404 once the console has moved', async ({ page, request }) => {
    test.skip(!MOVED, 'the console is at /admin on this stack');
    for (const path of ['/admin', '/admin/']) {
      const old = await page.goto(path);
      expect(old?.status(), path).toBe(404);
      const oldBody = await page.content();
      await page.goto('/zz-h15-none');
      expect(await page.content(), `${path} shows the unknown-path page`).toBe(oldBody);
      await expect(page.getByRole('link', { name: 'Go to the admin console' })).toHaveCount(0);
    }
    const api = await request.get('/admin/session');
    const unknown = await request.get('/zz-h15-none/session');
    expect(api.status()).toBe(unknown.status());
    expect(await api.text()).toBe(await unknown.text());
  });

  test('the mail client signs in and shows the inbox', async ({ page }) => {
    await page.goto('/');
    await login(page);
    const html = await (await page.request.get('/')).text();
    expect(html, 'the mail page is not the console').not.toContain('mw-console');
    if (MOVED) expect(html).not.toContain(ADMIN_PATH);
  });
});
