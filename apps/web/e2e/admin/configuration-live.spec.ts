import { test, expect } from '../csrf.ts';
import { ADMIN } from './admin-path.ts';
import {adminLogin} from './admin-login.ts';
test('effective configuration renders the actual endpoint on desktop and phone @phone', async ({ page }) => {
  await adminLogin(page);
  const response = page.waitForResponse(r => r.url().includes('/config/effective') && r.request().method() === 'GET');
  await page.goto(`${ADMIN}#/config`);
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
