import { test, expect } from '../csrf.ts';

test('last Super admin is protected unless explicitly confirmed policy permits loss @phone', async ({ page }) => {
  test.setTimeout(120_000);
  await new Promise(resolve => setTimeout(resolve, 3200));
  await page.goto('/admin');
  await page.getByLabel('Admin username').fill('root');
  await page.getByLabel('Password', { exact: true }).fill('hunter2');
  await page.getByRole('button', { name: 'Sign in', exact: true }).click();
  await expect.poll(async () => (await page.request.get('/admin/session')).status()).toBe(200);
  const path = '/admin/settings/admin.allowLastSuperadminLoss';
  const original = await page.request.get(path);
  expect(original.status()).toBe(200);
  const previous = (await original.json()).globalValue;
  await page.request.post('/admin/step-up', { data: { password: 'hunter2' } });
  const set = async (value: unknown): Promise<void> => {
    const row = await page.request.get(path);
    const saved = await page.request.put(path, { headers: { 'if-match': row.headers()['etag']! }, data: { globalValue: value, confirmLastSuperadminLoss: true } });
    expect(saved.status(), await saved.text()).toBe(200);
  };
  try {
    await set(false);
    const accounts = await (await page.request.get('/admin/admins')).json();
    const root = accounts.admins.find((account: { username: string }) => account.username === 'root');
    expect(root.lastSuperadminProtected).toBe(true);
    await page.goto(`/admin#/admins/${encodeURIComponent(root.id)}/edit`);
    await expect(page.getByRole('button', { name: 'Delete', exact: true })).toBeDisabled();
    await expect(page.getByRole('button', { name: 'Disable account', exact: true })).toBeDisabled();
    await page.getByRole('button', { name: 'Delete', exact: true }).scrollIntoViewIfNeeded();
    await page.screenshot({ path: test.info().outputPath('last-superadmin-protected.png'), fullPage: true });
    await page.goto('/admin#/settings/admin.allowLastSuperadminLoss');
    await page.getByRole('checkbox', { name: /Allow losing the last Super admin/ }).check();
    await page.getByRole('button', { name: 'Save', exact: true }).click();
    const confirmation = page.getByRole('alertdialog');
    await expect(confirmation).toContainText('leave nobody able to administer');
    await expect(confirmation.getByRole('button', { name: 'Allow loss of access' })).toBeDisabled();
    await confirmation.getByRole('textbox').fill('ALLOW LAST SUPERADMIN LOSS');
    await confirmation.getByRole('button', { name: 'Allow loss of access' }).click();
    await expect(confirmation).toHaveCount(0);
    await page.goto(`/admin#/admins/${encodeURIComponent(root.id)}/edit`);
    await page.getByRole('button', { name: 'Delete', exact: true }).click();
    await expect(page.getByRole('alertdialog')).toContainText('without any Super admin access');
    await page.screenshot({ path: test.info().outputPath('last-superadmin-loss-confirmation.png'), fullPage: true });
    await page.getByRole('alertdialog').getByRole('button', { name: 'Cancel', exact: true }).click();
    expect((await page.request.get('/admin/session')).status()).toBe(200);
  } finally { await set(previous); }
});
