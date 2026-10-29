import { test, expect } from '../csrf.ts';
import { ADMIN } from './admin-path.ts';
import {adminLogin} from './admin-login.ts';

test('settings actions are icons and the detail fits the scrolling viewport @phone', async ({ page }) => {
  await adminLogin(page);
  await page.goto(`${ADMIN}#/settings`);
  const open = page.getByRole('link', { name: /^Open / }).first();
  await expect(open).toBeVisible();
  expect((await open.boundingBox())!.width).toBeLessThanOrEqual(48);
  await expect(open.locator('svg[data-icon="edit"]')).toHaveCount(1);
  await open.click();
  const detail = page.getByTestId('list-detail');
  await expect(detail).toBeVisible();
  await expect(detail.getByRole('link', { name: 'Close', exact: true })).toBeVisible();
  // A detail occupies an exclusive page at every viewport (a5292a0f): the list is not rendered beside it.
  await expect(page.getByRole('table', { name: 'Settings', exact: true })).toHaveCount(0);
  const box = (await detail.boundingBox())!;
  expect(box.x).toBeGreaterThanOrEqual(0);
  expect(box.x + box.width).toBeLessThanOrEqual(page.viewportSize()!.width);
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  await page.screenshot({ path: test.info().outputPath('settings-detail.png'), fullPage: true });
  await detail.getByRole('link', { name: 'Close', exact: true }).click();
  await expect(page.getByTestId('list-detail')).toHaveCount(0);
  await expect(page.getByRole('table', { name: 'Settings', exact: true })).toBeVisible();
});
