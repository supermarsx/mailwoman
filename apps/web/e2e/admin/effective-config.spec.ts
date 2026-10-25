import { test, expect } from '../csrf.ts';

test('effective configuration has readable source filters and opens the exact editor @phone', async ({ page }) => {
  await page.goto('/admin');
  await page.getByLabel('Admin username').fill('root');
  await page.getByLabel('Password', { exact: true }).fill('hunter2');
  await page.getByRole('button', { name: 'Sign in', exact: true }).click();
  await expect(page.getByRole('button', { name: 'Sign in', exact: true })).toHaveCount(0);
  await page.goto('/admin#/config');
  await page.getByRole('searchbox', { name: 'Search configuration' }).fill('reader.colorMode');
  const table = page.getByRole('table', { name: 'Configuration visible to your administrator role' });
  await expect(table.getByRole('row')).toHaveCount(2);
  await expect(table.getByText('Message colors', { exact: true })).toBeVisible();
  await expect(page.getByRole('combobox', { name: 'Source', exact: true })).toBeVisible();
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  await page.screenshot({ path: test.info().outputPath('effective-configuration.png'), fullPage: true });
  await table.getByRole('link', { name: 'Open settings editor' }).click();
  await expect(page).toHaveURL(/#\/settings\/reader.colorMode$/);
  await expect(page.getByTestId('setting-detail')).toBeVisible();
});
