import { test, expect, type Page } from '@playwright/test';
import { ENGINE_CREDS, sidebarInbox } from './helpers.ts';

/**
 * The sign-in screen without client-side server entry (t32-h9), against the REAL
 * engine stack (mw-server in MW_MODE=engine over Greenmail — the `engine`
 * project). The file keeps its t28-e4 name so the project's testMatch is
 * unchanged; the server lookup it used to test is gone.
 *
 * The screen asks for the email address (or user name) and the password, and
 * nothing else. The server signs the address in to the mail server it is
 * configured with (`MW_UPSTREAM_DEFAULT=imap://greenmail:3143` in the engine
 * stack). It refuses a sign-in that names a server, and `/api/discover`, which
 * used to hand the browser the hosts found for a domain, answers 410.
 *
 * What is real here and what is not: every case talks to the real server except
 * the "not set up" message, which answers `/api/login` with the server's 404
 * body itself, because the stack has a default upstream and so places every
 * address.
 */

async function openLogin(page: Page): Promise<void> {
  await page.goto('/');
  await expect(page.getByRole('button', { name: 'Sign in', exact: true })).toBeVisible();
}

test.describe('Sign-in without a server field (engine mode)', () => {
  test('the form is the address and the password, with no server, URL or manual entry', async ({ page }) => {
    await openLogin(page);
    const form = page.getByRole('form', { name: 'Sign in' });
    await expect(form.getByLabel('Email address')).toBeVisible();
    await expect(form.getByLabel('Password', { exact: true })).toBeVisible();
    await expect(form.locator('input')).toHaveCount(2);
    await expect(form.locator('input[type=url]')).toHaveCount(0);
    await expect(page.getByLabel('JMAP server URL')).toHaveCount(0);
    await expect(page.getByRole('button', { name: /server|manual/i })).toHaveCount(0);
    await expect(page.getByText(/JMAP|IMAP/)).toHaveCount(0);
    // The mock backend's credentials are not printed on the screen.
    await expect(page.getByText('testpass')).toHaveCount(0);
  });

  test('signing in sends the address and the password only and reaches the configured server', async ({ page }) => {
    const loginBodies: unknown[] = [];
    const discoverCalls: string[] = [];
    page.on('request', (req) => {
      const path = new URL(req.url()).pathname;
      if (path.endsWith('/api/login') && req.method() === 'POST') loginBodies.push(req.postDataJSON());
      if (path.endsWith('/api/discover')) discoverCalls.push(path);
    });
    await openLogin(page);
    await page.getByLabel('Email address').fill(ENGINE_CREDS.username);
    await page.getByLabel('Password', { exact: true }).fill(ENGINE_CREDS.password);
    await page.getByRole('button', { name: 'Sign in', exact: true }).click();
    // The real IMAP session: the sidebar's Inbox comes from Greenmail's LIST.
    await expect(sidebarInbox(page)).toBeVisible();
    expect(loginBodies).toEqual([{ username: ENGINE_CREDS.username, password: ENGINE_CREDS.password }]);
    expect(discoverCalls).toEqual([]);
  });

  test('an address the server cannot place gets a plain message and still no server field', async ({ page }) => {
    await page.route('**/api/login', (route) =>
      route.fulfill({
        status: 404,
        contentType: 'application/json',
        body: JSON.stringify({ error: 'this address is not set up on this server', code: 'addressNotConfigured' }),
      }),
    );
    await openLogin(page);
    await page.getByLabel('Email address').fill('ada@nowhere.invalid');
    await page.getByLabel('Password', { exact: true }).fill('pw');
    await page.getByRole('button', { name: 'Sign in', exact: true }).click();
    await expect(page.getByRole('alert')).toHaveText(
      'This address is not set up on this server. Contact your administrator.',
    );
    await expect(page.getByRole('form', { name: 'Sign in' }).locator('input')).toHaveCount(2);
  });
});

test.describe('The server decides the upstream (engine mode)', () => {
  test('a sign-in that names a server is refused, even the configured one', async ({ request }) => {
    for (const body of [
      { jmapUrl: ENGINE_CREDS.imapUrl, username: ENGINE_CREDS.username, password: ENGINE_CREDS.password },
      { host: 'greenmail', port: 3143, username: ENGINE_CREDS.username, password: ENGINE_CREDS.password },
    ]) {
      const res = await request.post('/api/login', { data: body });
      expect(res.status(), JSON.stringify(Object.keys(body))).toBe(400);
      expect(await res.json()).toMatchObject({ code: 'clientServerNotAllowed' });
      expect(res.headers()['set-cookie'] ?? '').not.toContain('mw_session=');
    }
    // Control: the same credentials without the server field sign in.
    const ok = await request.post('/api/login', {
      data: { username: ENGINE_CREDS.username, password: ENGINE_CREDS.password },
    });
    expect(ok.status()).toBe(200);
  });

  test('/api/discover is gone and names no server', async ({ request }) => {
    const res = await request.post('/api/discover', { data: { email: 'ada@example.org' } });
    expect(res.status()).toBe(410);
    const text = await res.text();
    expect(text).not.toMatch(/greenmail|imap|jmapSrv/i);
  });
});
