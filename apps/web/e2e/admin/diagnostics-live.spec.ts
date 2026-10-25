import { test, expect } from '../csrf.ts';

test('server diagnostics execute real checks and retain history @phone', async ({ page }) => {
  await page.goto('/admin');
  await page.getByLabel('Admin username').fill('root');
  await page.getByLabel('Password', { exact: true }).fill('hunter2');
  await page.getByRole('button', { name: 'Sign in', exact: true }).click();
  await expect(page.getByRole('button', { name: 'Sign in', exact: true })).toHaveCount(0);
  await page.goto('/admin#/diagnostics');
  const run = page.getByRole('button', { name: 'Run server checks', exact: true });
  await expect(run).toBeEnabled();
  const response = page.waitForResponse(r => r.url().endsWith('/admin/diag/run') && r.request().method() === 'POST');
  await run.click();
  const reply = await response;
  expect(reply.status()).toBe(200);
  const report = await reply.json() as { run: { id: string; status: string; results: unknown[] }; historySaved: boolean; auditSaved: boolean };
  expect(report.run.status).toBe('pass');
  expect(report.run.results).toHaveLength(5);
  expect(report.historySaved).toBe(true);
  expect(report.auditSaved).toBe(true);
  await expect(page.locator('main tbody tr')).toHaveCount(5);
  await expect(page.locator('main tbody')).toContainText('Server key seals and opens a synthetic sample');
  await page.reload();
  await expect(page.locator('main tbody tr')).toHaveCount(5);
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  await page.screenshot({ path: test.info().outputPath('diagnostics.png'), fullPage: true });
});
