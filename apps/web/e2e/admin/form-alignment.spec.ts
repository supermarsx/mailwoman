import { test, expect } from '../csrf.ts';

test('horizontal form actions align below labels and settings cells use available width @phone', async ({ page }) => {
  await page.goto('/admin');
  await page.getByLabel('Admin username').fill('root');
  await page.getByLabel('Password', { exact: true }).fill('hunter2');
  await page.getByRole('button', { name: 'Sign in', exact: true }).click();
  await expect(page.getByRole('button', { name: 'Sign in', exact: true })).toHaveCount(0);
  await page.goto('/admin#/observability');
  const form = page.getByRole('form', { name: 'Add ban' });
  const button = form.getByRole('button', { name: 'Ban IP' });
  await expect(button).toBeVisible();
  const inputBox = (await form.getByRole('textbox').last().boundingBox())!;
  const buttonBox = (await button.boundingBox())!;
  if (Math.abs(buttonBox.y - inputBox.y) < 50) expect(Math.abs(buttonBox.y + buttonBox.height - inputBox.y - inputBox.height)).toBeLessThanOrEqual(2);
  else expect(buttonBox.y).toBeGreaterThanOrEqual(inputBox.y + inputBox.height);
  await page.goto('/admin#/settings');
  const table = page.getByRole('table', { name: 'Settings', exact: true });
  await expect(table).toBeVisible();
  const name = table.locator('tbody td').first().locator('span').first();
  expect(await name.evaluate(node => getComputedStyle(node).maxInlineSize)).toBe('none');
  const value = table.locator('tbody tr').first().locator('td').nth(1).locator('span').first();
  expect(await value.evaluate(node => getComputedStyle(node).textOverflow)).not.toBe('ellipsis');
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  await page.screenshot({ path: test.info().outputPath('settings-fluid-columns.png'), fullPage: true });
});
