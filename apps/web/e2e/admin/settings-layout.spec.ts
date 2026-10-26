import { test, expect } from '../csrf.ts';
import {adminLogin} from './admin-login.ts';

test('settings actions are icons and the detail fits the scrolling viewport @phone', async ({ page }) => {
  await adminLogin(page);
  await page.goto('/admin#/settings');
  const open = page.getByRole('link', { name: /^Open / }).first();
  await expect(open).toBeVisible();
  expect((await open.boundingBox())!.width).toBeLessThanOrEqual(48);
  await expect(open.locator('svg[data-icon="edit"]')).toHaveCount(1);
  await open.click();
  const detail = page.getByTestId('list-detail');
  await expect(detail).toBeVisible();
  await expect(detail.getByRole('link', { name: 'Close', exact: true })).toBeVisible();
  if (page.viewportSize()!.width >= 1200) {
    const box = (await detail.boundingBox())!;
    expect(box.height).toBeLessThan(page.viewportSize()!.height - 40);
    const table = page.getByRole('table', { name: 'Settings', exact: true });
    expect((await table.boundingBox())!.height).toBeGreaterThan(box.height);
    await page.locator('#admin-main').evaluate((node) => { node.scrollTop = 600; });
    await expect.poll(async () => (await detail.boundingBox())!.y).toBeLessThan(60);
    const after = (await detail.boundingBox())!;
    expect(after.y).toBeGreaterThanOrEqual(0);
    expect(after.y + after.height).toBeLessThanOrEqual(page.viewportSize()!.height);
  }
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  await page.screenshot({ path: test.info().outputPath('settings-detail.png'), fullPage: true });
  await detail.getByRole('link', { name: 'Close', exact: true }).click();
  await expect(page.getByTestId('list-detail')).toHaveCount(0);
});
