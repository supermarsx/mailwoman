import { test, expect } from '../csrf.ts';
test('effective configuration renders the actual endpoint on desktop and phone @phone', async ({ page }) => {
  await page.goto('/admin'); await page.getByLabel('Admin username').fill('root'); await page.getByLabel('Password', { exact: true }).fill('hunter2'); await page.getByRole('button', { name: 'Sign in', exact: true }).click();
  await expect(page.getByRole('button', { name: 'Sign in', exact: true })).toHaveCount(0);
  const response = page.waitForResponse(r => r.url().includes('/config/effective') && r.request().method() === 'GET');
  await page.goto('/admin#/config');
  const api = await response; expect(api.status()).toBe(200);
  const rows = await api.json() as { id: string; valueStatus: string; value: unknown }[];
  expect(rows.length).toBeGreaterThan(100);
  await expect(page.getByRole('heading', { name: 'Effective configuration', exact: true })).toBeVisible();
  await expect(page.locator('main tbody tr').first()).toBeVisible();
  await page.getByRole('searchbox', { name: 'Search configuration' }).fill('session.idleSecs');
  await expect(page.locator('main tbody')).toContainText('session.idleSecs');
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  await page.screenshot({ path: test.info().outputPath('effective-configuration.png'), fullPage: true });
  await test.info().attach('actual-config-response-summary', { body: JSON.stringify({ count: rows.length, statuses: [...new Set(rows.map(r => r.valueStatus))] }), contentType: 'application/json' });
});
