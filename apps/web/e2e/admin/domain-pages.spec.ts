import { test, expect } from '../csrf.ts';
import { ADMIN } from './admin-path.ts';

test('domain list actions open exclusive editors and clone without enabling mail @phone', async ({ page }) => {
  await page.goto(ADMIN);
  await page.getByLabel('Admin username').fill('root');
  await page.getByLabel('Password', { exact: true }).fill('hunter2');
  const login = page.waitForResponse(r => r.url().endsWith(`${ADMIN}/login`) && r.request().method() === 'POST');
  await page.getByRole('button', { name: 'Sign in', exact: true }).click();
  expect((await login).ok()).toBe(true);
  await expect(page.locator('#admin-main')).toBeVisible();
  const name = `pages-${Date.now()}.example.test`, copy = `copy-${name}`;
  const body = { enabled: false, upstream: { version: 2, backends: [] }, allowlist: [], blocklist: [] };
  expect((await page.request.put(`${ADMIN}/domains/${name}`, { data: body })).ok()).toBe(true);
  try {
    const detail = await (await page.request.get(`${ADMIN}/domains/${name}`)).json();
    const alias = `alias-${name}`;
    expect((await page.request.put(`${ADMIN}/domains/${name}/aliases/${alias}`, { headers: { 'if-match': detail.mappingEtag } })).ok()).toBe(true);
    await page.goto(`${ADMIN}#/domains`);
    await expect(page.getByRole('table', { name: 'Domains' })).toBeVisible();
    await page.getByRole('searchbox', { name: 'Find a domain' }).fill(alias);
    const row = page.getByRole('row').filter({ has: page.getByRole('button', { name, exact: true }) });
    await expect(row.getByText(alias, { exact: true })).toBeVisible();
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
    await page.screenshot({ path: test.info().outputPath('domain-table-aliases.png'), fullPage: true });
    await expect(row.getByRole('button', { name: 'Enable domain', exact: true })).toBeVisible();
    await expect(row.getByRole('button', { name: 'Delete', exact: true })).toBeVisible();
    await row.getByRole('link', { name: 'Clone', exact: true }).click();
    await expect(page).toHaveURL(new RegExp(`/domains/${name}/clone$`));
    const form = page.getByRole('form', { name: 'Clone', exact: true });
    await form.getByLabel('Domain name', { exact: true }).fill(copy);
    await form.getByRole('button', { name: 'Clone', exact: true }).click();
    await expect(page).toHaveURL(new RegExp(`/domains/${copy}/connections$`));
    const saved = await page.request.get(`${ADMIN}/domains/${copy}`);
    expect(saved.ok()).toBe(true);
    expect((await saved.json()).enabled).toBe(false);
    await page.getByRole('link', { name: 'Sign-in policy', exact: true }).click();
    await expect(page.getByLabel('Allowlist', { exact: true })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Add backend', exact: true })).toHaveCount(0);
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
    await page.screenshot({ path: test.info().outputPath('domain-policy-page.png'), fullPage: true });
    await page.getByRole('link', { name: 'Overview', exact: true }).click();
    await expect(page.locator('#admin-main form')).toHaveCount(0);
  } finally {
    for (const domain of [copy, name]) {
      const response = await page.request.get(`${ADMIN}/domains/${domain}`);
      if (response.ok()) expect((await page.request.delete(`${ADMIN}/domains/${domain}`, { headers: { 'if-match': response.headers()['etag']! } })).ok()).toBe(true);
    }
  }
});
