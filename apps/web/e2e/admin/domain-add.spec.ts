import { test, expect } from '../csrf.ts';

test('quick and guided domain creation use dedicated pages and save after review @phone', async ({ page }) => {
  await page.goto('/admin');
  await page.getByLabel('Admin username').fill('root');
  await page.getByLabel('Password', { exact: true }).fill('hunter2');
  await page.getByRole('button', { name: 'Sign in', exact: true }).click();
  await expect(page.locator('#admin-main')).toBeVisible();
  const names = [`quick-${Date.now()}.example.test`, `guided-${Date.now()}.example.test`];
  try {
    await page.goto('/admin#/domains');
    await page.getByRole('button', { name: 'Quick add', exact: true }).click();
    await expect(page.getByRole('table', { name: 'Domains' })).toHaveCount(0);
    const quick = page.getByRole('form', { name: 'Add domain' });
    await quick.getByLabel('Domain name', { exact: true }).fill(names[0]!);
    await quick.getByRole('button', { name: 'Create and configure', exact: true }).click();
    await expect(page).toHaveURL(new RegExp(`/domains/${names[0]}$`));
    await page.getByRole('button', { name: 'Back to domains' }).click();
    await page.getByRole('button', { name: 'Guided add', exact: true }).click();
    await expect(page).toHaveURL(/\/domains\/new\/guided$/);
    await page.getByLabel('Domain name', { exact: true }).fill(names[1]!);
    await page.getByRole('button', { name: 'Next', exact: true }).click();
    await expect(page.getByText('Server defaults use the mail routing configured by your administrator. You can review this domain before creating it.')).toBeVisible();
    expect((await page.request.get(`/admin/domains/${names[1]}`)).status()).toBe(404);
    await page.getByRole('button', { name: 'Next', exact: true }).click();
    await expect(page.getByRole('heading', { name: /Review/ })).toBeVisible();
    await page.screenshot({ path: test.info().outputPath('guided-domain-review.png'), fullPage: true });
    await page.getByRole('button', { name: 'Create domain', exact: true }).click();
    await expect(page).toHaveURL(new RegExp(`/domains/${names[1]}/connections$`));
    expect((await page.request.get(`/admin/domains/${names[1]}`)).ok()).toBe(true);
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  } finally {
    for (const name of names) {
      const response = await page.request.get(`/admin/domains/${name}`);
      if (response.ok()) expect((await page.request.delete(`/admin/domains/${name}`, { headers: { 'if-match': response.headers()['etag']! } })).ok()).toBe(true);
    }
  }
});
