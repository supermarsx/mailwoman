import { test, expect, type APIRequestContext } from '@playwright/test';

/**
 * Deploy guard (t30-e2): the admin panel of an installation made by `deploy.sh`,
 * in a browser.
 *
 * `deploy.sh` writes `MW_CSRF_STRICT=true` into every new installation. In that
 * mode the server refuses a write that a cookie authorises unless the request
 * repeats the `mw_csrf` cookie in `X-CSRF-Token`. Until 26.20 the web client
 * never sent it, so the generated admin password was answered 403 and a default
 * installation could not be administered.
 *
 * What this spec shows, in order:
 *   1. the installation really is strict — the same write, sent from this
 *      browser's session without the header, is refused and stores nothing. Without
 *      this the rest would pass against a server that checks nothing;
 *   2. sign-in, a write (add a ban), its undo (remove it) and sign-out all succeed
 *      through the UI, each confirmed by asking the server, not by the page alone;
 *   3. a reload keeps the admin signed in.
 *
 * What it does not cover: the mailbox app (the `mock`, `engine` and `phone`
 * projects run that against strict stacks), and any admin screen other than
 * Observability.
 *
 * Run by `scripts/test-deploy.sh`, which sets `MW_DEPLOY_BASE_URL` to the
 * installation it deployed and `MW_ADMIN_USER` / `MW_ADMIN_PASSWORD` from that
 * installation's `app.env`.
 */

const BASE = (process.env['MW_DEPLOY_BASE_URL'] ?? '').replace(/\/+$/, '');
const ADMIN_USER = process.env['MW_ADMIN_USER'] ?? '';
const ADMIN_PASSWORD = process.env['MW_ADMIN_PASSWORD'] ?? '';

/** The addresses the server lists as banned, asked with the browser's own cookies. */
async function bannedIps(request: APIRequestContext): Promise<string[]> {
  const res = await request.get(`${BASE}/admin/bans`);
  expect(res.status(), 'GET /admin/bans').toBe(200);
  return ((await res.json()) as Array<{ ip: string }>).map((b) => b.ip);
}

test.describe('admin panel on a deployed installation', () => {
  test.skip(BASE === '', 'MW_DEPLOY_BASE_URL is not set: there is no installation to drive');

  test('sign in, add and remove a ban, reload, sign out — with strict CSRF on', async ({ page }) => {
    expect(ADMIN_USER, 'MW_ADMIN_USER must be set').not.toBe('');
    expect(ADMIN_PASSWORD, 'MW_ADMIN_PASSWORD must be set').not.toBe('');

    // ── sign in through the form ────────────────────────────────────────────
    await page.goto(`${BASE}/admin`);
    const loginForm = page.getByRole('form', { name: 'Admin sign in' });
    await expect(loginForm).toBeVisible();
    await loginForm.getByLabel('Admin username').fill(ADMIN_USER);
    await loginForm.getByLabel('Password', { exact: true }).fill(ADMIN_PASSWORD);
    await loginForm.getByRole('button', { name: 'Sign in' }).click();

    const nav = page.getByRole('navigation', { name: 'Admin sections' });
    await expect(nav, 'the generated password signs in').toBeVisible();

    // A documentation address (TEST-NET-3) that is not on the list yet.
    const before = await bannedIps(page.request);
    const ip = Array.from({ length: 254 }, (_, i) => `203.0.113.${i + 1}`).find((c) => !before.includes(c));
    expect(ip, 'a free 203.0.113.x address').toBeDefined();
    if (ip === undefined) return;

    // ── 1. precondition: this installation is strict ────────────────────────
    // `page.request` shares the browser's cookies (admin session and mw_csrf) and
    // adds no header of its own: this is the request the web client used to send.
    const bare = await page.request.post(`${BASE}/admin/bans`, {
      data: { ip, reason: 'deploy guard: must be refused', expiresAt: null },
    });
    expect(bare.status(), 'a write without X-CSRF-Token is refused: strict mode is on').toBe(403);
    expect(await bare.json()).toEqual({ error: 'missing or invalid CSRF token' });
    expect(await bannedIps(page.request), 'the refused write stored nothing').not.toContain(ip);

    // ── 2. the same write through the UI ────────────────────────────────────
    await nav.getByRole('link', { name: 'Observability' }).click();
    const addBan = page.getByRole('form', { name: 'Add ban' });
    await addBan.getByLabel('IP address').fill(ip);
    await addBan.getByLabel('Reason').fill('deploy guard');
    await addBan.getByRole('button', { name: 'Ban IP' }).click();

    const unban = page.getByRole('button', { name: `Unban ${ip}` });
    await expect(unban, 'the ban is listed').toBeVisible();
    await expect(page.getByText('Could not add the ban')).toHaveCount(0);
    expect(await bannedIps(page.request), 'the server stored the ban').toContain(ip);

    // …and its undo.
    await unban.click();
    await expect(unban, 'the ban is gone from the list').toHaveCount(0);
    await expect(page.getByText('Could not remove the ban')).toHaveCount(0);
    expect(await bannedIps(page.request), 'the server removed the ban').not.toContain(ip);

    // ── 3. a reload keeps the session ───────────────────────────────────────
    await page.reload();
    await expect(page.getByRole('navigation', { name: 'Admin sections' })).toBeVisible();
    await expect(page.getByRole('form', { name: 'Admin sign in' })).toHaveCount(0);

    // Sign-out is a write too.
    await page.getByRole('button', { name: 'Sign out' }).click();
    await expect(page.getByRole('form', { name: 'Admin sign in' })).toBeVisible();
    expect((await page.request.get(`${BASE}/admin/session`)).status(), 'the session ended on the server').toBe(401);
  });
});
