import { test, expect } from '../csrf.ts';
import { ADMIN } from './admin-path.ts';

test('dashboard serves real data and personal widget settings survive reload @phone', async ({ page }) => {
  await page.goto(ADMIN);
  await page.getByLabel('Admin username').fill('root');
  await page.getByLabel('Password', { exact: true }).fill('hunter2');
  const login = page.waitForResponse(r => r.url().endsWith(`${ADMIN}/login`) && r.request().method() === 'POST');
  await page.getByRole('button', { name: 'Sign in', exact: true }).click();
  expect((await login).status()).toBe(200);
  const response = await page.request.get(`${ADMIN}/dashboard`);
  expect(response.status(), await response.text()).toBe(200);
  const dashboard = await response.json();
  expect(dashboard.counts.domains).toBeGreaterThanOrEqual(0);
  expect(dashboard.health.dbPingMs).toBeGreaterThanOrEqual(0);
  const saved = await page.request.get(`${ADMIN}/me/preferences`);
  const original = await saved.json();
  try {
    const baseline = await page.request.put(`${ADMIN}/me/preferences`, { headers: { 'If-Match': saved.headers()['etag']! }, data: { ...original, dashboard: { hidden: [], refreshSecs: 0 } } });
    expect(baseline.ok()).toBe(true);
    await page.reload();
    await page.getByRole('link', { name: 'Customize dashboard' }).click();
    const prefs = page.locator('#dashboard-preferences');
    await expect(prefs).toBeVisible();
    await prefs.getByRole('checkbox', { name: 'HTTP traffic', exact: true }).uncheck();
    await prefs.getByRole('checkbox', { name: 'Server information', exact: true }).check();
    await prefs.getByLabel('Automatic refresh').selectOption('60');
    await prefs.getByRole('button', { name: 'Reorder Server information' }).press('ArrowUp');
    await expect.poll(async () => (await (await page.request.get(`${ADMIN}/me/preferences`)).json()).dashboard?.refreshSecs).toBe(60);
    await expect.poll(async () => (await (await page.request.get(`${ADMIN}/me/preferences`)).json()).dashboard?.hidden).toContain('traffic');
    await expect.poll(async () => (await (await page.request.get(`${ADMIN}/me/preferences`)).json()).dashboard?.order?.indexOf('server')).toBe(6);
    await page.goto(`${ADMIN}#/dashboard`);
    await expect(page.getByTestId('tile-users')).toBeVisible();
    await expect(page.locator('[data-dashboard-widget="traffic"]')).toHaveCount(0);
    await expect(page.locator('[data-dashboard-widget="server"]')).toBeVisible();
    expect(await page.locator('[data-dashboard-widget="server"]').evaluate(node => getComputedStyle(node).order)).toBe('7');
    await page.reload();
    await expect(page.getByTestId('tile-users')).toBeVisible();
    await expect(page.locator('[data-dashboard-widget="traffic"]')).toHaveCount(0);
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
    await page.screenshot({ path: test.info().outputPath('dashboard.png'), fullPage: true });
  } finally {
    const current = await page.request.get(`${ADMIN}/me/preferences`);
    const restored = await page.request.put(`${ADMIN}/me/preferences`, { headers: { 'If-Match': current.headers()['etag']! }, data: original });
    expect(restored.ok()).toBe(true);
  }
});
