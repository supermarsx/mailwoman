import { test, expect } from '../csrf.ts';
import { ADMIN } from './admin-path.ts';

test('privacy diagnostics use readable contained tables and settings detail groups have room @phone', async ({ page }) => {
  await page.goto(ADMIN);
  await page.getByLabel('Admin username').fill('root');
  await page.getByLabel('Password', { exact: true }).fill('hunter2');
  await page.getByRole('button', { name: 'Sign in', exact: true }).click();
  await expect(page.getByRole('button', { name: 'Sign in', exact: true })).toHaveCount(0);
  await page.goto(`${ADMIN}#/privacy`);
  const cache = page.getByRole('table', { name: 'Cache and tracking protection', exact: true });
  const traffic = page.getByRole('table', { name: 'Outbound traffic processing', exact: true });
  await expect(cache).toBeVisible();
  await expect(cache.getByRole('row')).toHaveCount(4);
  await expect(traffic.getByRole('columnheader', { name: 'Total time (ms)' })).toBeVisible();
  expect(await cache.locator('td').first().evaluate(node => parseFloat(getComputedStyle(node).paddingInlineStart))).toBeGreaterThanOrEqual(12);
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  await page.screenshot({ path: test.info().outputPath('privacy-tables.png'), fullPage: true });
  await page.goto(`${ADMIN}#/settings/privacy.remoteContent`);
  const detail = page.getByTestId('setting-detail');
  await expect(detail).toBeVisible();
  const switches = detail.locator('#setting-override-help');
  await expect(switches).toBeVisible();
  expect(await switches.evaluate(node => parseFloat(getComputedStyle(node.parentElement!).rowGap))).toBeGreaterThanOrEqual(8);
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  await page.screenshot({ path: test.info().outputPath('settings-detail-spacing.png'), fullPage: true });
});
