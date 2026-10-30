// t104 live: a signed-in admin runs several diagnostics in a row with no 429,
// and a forced 429 (a low rateLimit.diagnostics* limit) is the corner toast
// with Retry, not a line in the page, and Retry works once the wait is over.
//
// Needs a server built from the tree with the console on (root / hunter2) at
// baseURL, and the sign-in retry cooldown at 0 or the fixture's 3 s (adminLogin
// waits it out). Screenshots go to MW_T104_SHOTS when set.
import { test, expect, type Page } from '@playwright/test';
import { ADMIN } from './admin-path.ts';
import { adminLogin } from './admin-login.ts';

const shots = process.env['MW_T104_SHOTS'];
const shot = async (page: Page, name: string): Promise<void> => {
  if (shots) await page.screenshot({ path: `${shots}/${name}.png`, fullPage: false });
};

/** A console API call from the page, with its session and CSRF token. */
async function api(page: Page, method: string, path: string, body?: unknown): Promise<{ status: number; body: unknown }> {
  return page.evaluate(async ({ method, url, body }) => {
    const token = document.cookie.split('; ').find((c) => c.startsWith('mw_admin_csrf=') || c.startsWith('mw_csrf='))?.split('=')[1];
    const response = await fetch(url, {
      method,
      credentials: 'same-origin',
      headers: { 'content-type': 'application/json', ...(token ? { 'x-csrf-token': decodeURIComponent(token) } : {}) },
      ...(body === undefined ? {} : { body: JSON.stringify(body) }),
    });
    return { status: response.status, body: await response.json().catch(() => null) };
  }, { method, url: `${ADMIN}${path}`, body });
}

test('several diagnostics in a row, then a forced 429 as the corner toast with a working Retry', async ({ page }, info) => {
  test.setTimeout(180_000);
  const scheme = info.project.name.includes('dark') ? 'dark' : 'light';
  await page.emulateMedia({ colorScheme: scheme });
  const statuses: number[] = [];
  page.on('response', (r) => { if (r.url().endsWith(`${ADMIN}/diag/run`) && r.request().method() === 'POST') statuses.push(r.status()); });
  await adminLogin(page);
  // t105: the server suite runs from a card on the Server sub-tab.
  await page.goto(`${ADMIN}#/diagnostics/server`);
  const run = page.getByRole('article', { name: 'Database connection' }).getByRole('button', { name: /^Run( again)?$/ });
  for (let n = 0; n < 4; n += 1) {
    await expect(run).toBeEnabled({ timeout: 60_000 });
    const answered = page.waitForResponse((r) => r.url().endsWith(`${ADMIN}/diag/run`) && r.request().method() === 'POST');
    await run.click();
    expect((await answered).status(), `run ${n + 1}`).toBe(200);
  }
  expect(statuses).toEqual([200, 200, 200, 200]);
  await expect(page.getByTestId('admin-request-error')).toHaveCount(0);
  await shot(page, `runs-in-a-row-${info.project.name}`);

  // Force a 429: one run per 10 s for this console (step-up first).
  const stepUp = await api(page, 'POST', '/step-up', { password: 'hunter2' });
  expect(stepUp.status, JSON.stringify(stepUp.body)).toBe(200);
  const low = await api(page, 'PUT', '/rate-limits', { diagnosticsBurst: 1, diagnosticsPerMinute: 6 });
  expect(low.status, JSON.stringify(low.body)).toBe(200);
  try {
    // A fresh session starts with a full (now one-run) bucket.
    await page.context().clearCookies();
    await adminLogin(page);
    await page.goto(`${ADMIN}#/diagnostics/server`);
    await expect(run).toBeEnabled({ timeout: 60_000 });
    await run.click();
    await expect(run).toBeEnabled({ timeout: 60_000 });
    const refused = page.waitForResponse((r) => r.url().endsWith(`${ADMIN}/diag/run`) && r.request().method() === 'POST');
    await run.click();
    const reply = await refused;
    expect(reply.status()).toBe(429);
    expect(Number(reply.headers()['retry-after'])).toBeGreaterThan(0);
    const toast = page.getByTestId('toast-region').getByTestId('admin-request-error');
    await expect(toast).toBeVisible();
    await expect(toast).toContainText(/Too many requests, try again in \d+ s\./);
    const section = page.getByRole('region', { name: 'Diagnostics' });
    await expect(section.getByRole('alert')).toHaveCount(0);
    // The toast sits in the fixed corner region, outside the page flow.
    expect(await page.getByTestId('toast-region').evaluate((el) => getComputedStyle(el).position)).toBe('fixed');
    await shot(page, `toast-429-${info.project.name}`);
    const retried = page.waitForResponse((r) => r.url().endsWith(`${ADMIN}/diag/run`) && r.request().method() === 'POST', { timeout: 30_000 });
    await toast.getByRole('button', { name: 'Retry' }).click();
    expect((await retried).status()).toBe(200);
    await expect(page.getByTestId('admin-request-error')).toHaveCount(0);
    await shot(page, `retry-done-${info.project.name}`);
  } finally {
    await api(page, 'POST', '/step-up', { password: 'hunter2' });
    await api(page, 'PUT', '/rate-limits', { diagnosticsBurst: null, diagnosticsPerMinute: null });
  }
});
