// t76 — the admin console answers only at its configured path, live. Runs on the
// receipts stack (scripts/e2e-receipts-stack.mjs) started with MW_E2E_ADMIN_PATH
// set to a path of its own; the stack passes it to the server as MW_ADMIN_PATH.
//
// For Chromium and Firefox, at 1440 and 390 px:
//   - the console signs in at the path, and Security > Console address shows it;
//   - /admin (page, sign-in, API) gets exactly what an unknown path gets;
//   - a mailbox user's session (no admin) never sees the path: not in the DOM,
//     not in any response it loads (bodies and headers), not in sw.js or the
//     manifest.
import { test, expect, type Page, type Response } from '../csrf.ts';
import { signInManually } from '../helpers.ts';

const ADMIN = process.env['MW_E2E_ADMIN_PATH'] ?? '';
const SEGMENT = ADMIN.replace(/^\/+/, '');
const SIZES = [{ width: 1440, height: 900 }, { width: 390, height: 844 }];

test.skip(ADMIN === '' || ADMIN === '/admin', 'run with MW_E2E_ADMIN_PATH set to a path other than /admin');

async function adminSignIn(page: Page): Promise<void> {
  await page.goto(ADMIN);
  await page.getByLabel('Admin username').fill('root');
  await page.getByLabel('Password', { exact: true }).fill('hunter2');
  const login = page.waitForResponse((r) => r.url().endsWith(`${ADMIN}/login`) && r.request().method() === 'POST');
  await page.getByRole('button', { name: 'Sign in', exact: true }).click();
  expect((await login).status()).toBe(200);
  await expect(page.locator('#admin-main')).toBeVisible();
}

for (const size of SIZES) {
  test(`console at its path; /admin is unknown (${size.width})`, async ({ page }, info) => {
    await page.setViewportSize(size);
    await adminSignIn(page);
    const cookies = await page.context().cookies();
    const session = cookies.find((c) => c.name === 'mw_admin_session');
    expect(session?.path, 'the admin cookie is scoped to the console').toBe(ADMIN);
    await page.goto(`${ADMIN}#/security/console`);
    await expect(page.getByTestId('console-path-current')).toContainText(ADMIN);
    await expect(page.getByText(/does not separate the console from the mail app/)).toBeVisible();
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1)).toBe(true);
    await page.screenshot({ path: info.outputPath(`console-${size.width}.png`) });

    // Exactly the unknown-path answer, for the page and the API, signed in or not.
    const nav = { headers: { accept: 'text/html', 'sec-fetch-dest': 'document', 'sec-fetch-mode': 'navigate' } };
    for (const [path, unknown] of [['/admin', '/zz-t76-none'], ['/admin/session', '/zz-t76-none/session'], ['/admin/users', '/zz-t76-none/users']] as const) {
      const a = await page.request.get(path, nav);
      const b = await page.request.get(unknown, nav);
      expect(a.status(), path).toBe(404);
      expect(await a.text(), path).toBe(await b.text());
    }
    const signIn = await page.request.post('/admin/login', { data: { username: 'root', password: 'hunter2' } });
    const unknownSignIn = await page.request.post('/zz-t76-none/login', { data: { username: 'root', password: 'hunter2' } });
    expect(signIn.status()).toBe(unknownSignIn.status());
    await page.goto('/admin');
    await expect(page.getByRole('heading', { level: 1 })).toHaveText('This page does not exist.');
    await expect(page.locator('#admin-main')).toHaveCount(0);
    await page.screenshot({ path: info.outputPath(`admin-404-${size.width}.png`) });
  });

  test(`a mailbox user never sees the console path (${size.width})`, async ({ browser }, info) => {
    // A browser that was never used for the console.
    const context = await browser.newContext({ viewport: size, serviceWorkers: 'block' });
    const page = await context.newPage();
    const seen: string[] = [];
    const leaks: string[] = [];
    page.on('response', (res: Response) => {
      void (async () => {
        const url = res.url();
        seen.push(url);
        if (url.includes(SEGMENT)) leaks.push(`url ${url}`);
        for (const [k, v] of Object.entries(res.headers())) if (v.includes(SEGMENT)) leaks.push(`header ${k} of ${url}`);
        const type = res.headers()['content-type'] ?? '';
        if (/json|text|javascript|html|xml|css|manifest/.test(type)) {
          const body = await res.text().catch(() => '');
          if (body.includes(SEGMENT)) leaks.push(`body of ${url}`);
        }
      })();
    });
    await signInManually(page, { username: 'testuser', password: 'testpass' });
    await expect(page.locator('.list__row').first()).toBeVisible({ timeout: 60_000 });
    await page.goto('/#/settings');
    await page.waitForLoadState('networkidle');
    for (const extra of ['/sw.js', '/manifest.webmanifest', '/robots.txt', '/api/me', '/api/public-config']) {
      const r = await page.request.get(extra);
      const text = await r.text();
      expect(text.includes(SEGMENT), `${extra} names the console`).toBe(false);
    }
    const html = await page.content();
    expect(html.includes(SEGMENT), 'the mail app DOM names the console').toBe(false);
    expect(await page.evaluate((seg) => JSON.stringify({ ...localStorage }).includes(seg) || JSON.stringify({ ...sessionStorage }).includes(seg), SEGMENT)).toBe(false);
    await page.waitForTimeout(500);
    expect(seen.length).toBeGreaterThan(5);
    expect(leaks, leaks.join('\n')).toEqual([]);
    const cookies = await context.cookies();
    expect(cookies.some((c) => c.name === 'mw_admin_session')).toBe(false);
    await page.screenshot({ path: info.outputPath(`mail-${size.width}.png`) });
    await context.close();
  });
}
