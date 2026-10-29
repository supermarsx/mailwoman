import { test, expect } from '../csrf.ts';
import { ADMIN } from './admin-path.ts';
import { signInManually } from '../helpers.ts';
async function admin(page: import('@playwright/test').Page): Promise<void> {
  await page.goto(ADMIN); await page.getByLabel('Admin username').fill('root'); await page.getByLabel('Password', { exact: true }).fill('hunter2'); await page.getByRole('button', { name: 'Sign in', exact: true }).click();
  await expect(page.getByRole('button', { name: 'Sign in', exact: true })).toHaveCount(0);
  // The LDAP editor is the Integrations "directory" tab (a5292a0f).
  await page.goto(`${ADMIN}#/integrations/directory`);
}
// The runtime leg needs what CI's compose stack does not have: an LDAPS server at
// localhost:42412 (dc=example,dc=com, cn=admin/adminpassword, an alice entry) and a server
// restart between the two tests, since directory changes apply only after a restart.
// MW_E2E_LDAP_RUNTIME=1 marks such a run; the configure leg then keeps its saved entry.
const ldapRuntime = Boolean(process.env['MW_E2E_LDAP_RUNTIME']);
test('configure a real LDAP TLS directory with sealed credentials @configure', async ({ page }) => {
  await admin(page);
  const section = page.getByRole('region', { name: 'LDAP directory configuration', exact: true });
  await expect(section.getByRole('button', { name: 'Add directory' })).toBeVisible();
  await section.getByRole('button', { name: 'Add directory' }).click();
  await section.getByLabel('Server URL', { exact: true }).fill('ldaps://localhost:42412/');
  await section.getByLabel('Search base DN', { exact: true }).fill('dc=example,dc=com');
  await section.getByLabel('Service bind DN', { exact: true }).fill('cn=admin,dc=example,dc=com');
  await section.getByRole('combobox', { name: 'Connection policy', exact: true }).selectOption('direct');
  await section.getByLabel('Include in directory searches', { exact: true }).check();
  await section.getByLabel('Allow private LAN and loopback addresses', { exact: true }).check();
  await section.getByRole('combobox', { name: 'Stored password action', exact: true }).selectOption('set');
  await section.getByLabel('New service password', { exact: true }).fill('adminpassword');
  const response = page.waitForResponse(r => r.url().endsWith('/integrations/directory') && r.request().method() === 'PUT');
  await section.getByRole('button', { name: 'Save directory configuration', exact: true }).click();
  const saved = await response; expect(saved.status()).toBe(200); expect(await saved.text()).not.toContain('adminpassword');
  await expect(section.getByText('Configuration saved. Restart the server to apply it.')).toBeVisible();
  await expect(section.getByLabel('New service password')).toHaveCount(0);
  await section.screenshot({ path: test.info().outputPath('directory-saved-restart-required.png') });
  if (!ldapRuntime) {
    // Without the restart leg, leave the saved configuration as it was: remove the entry again.
    const entries = section.getByRole('group');
    await entries.last().getByRole('button', { name: 'Remove from configuration', exact: true }).click();
    const cleared = page.waitForResponse(r => r.url().endsWith('/integrations/directory') && r.request().method() === 'PUT');
    await section.getByRole('button', { name: 'Save directory configuration', exact: true }).click();
    expect((await cleared).status()).toBe(200);
    await expect(section.getByText('Configuration saved. Restart the server to apply it.')).toBeVisible();
    await expect(section.getByLabel('Server URL', { exact: true })).toHaveCount(0);
  }
});
test('restarted directory performs a real TLS search from the saved editor configuration @runtime @phone', async ({ page }) => {
  test.skip(!ldapRuntime, 'needs an LDAPS fixture at localhost:42412 and a server restart after the configure leg (set MW_E2E_LDAP_RUNTIME=1)');
  await admin(page);
  const section = page.getByRole('region', { name: 'LDAP directory configuration', exact: true });
  await expect(section.getByText('A sealed service password is stored.')).toBeVisible();
  await expect(section.getByText('Saved configuration differs from the running directory. A server restart is required.')).toHaveCount(0);
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  await section.screenshot({ path: test.info().outputPath('directory-active.png') });
  const save = section.getByRole('button', { name: 'Save directory configuration', exact: true });
  await save.scrollIntoViewIfNeeded();
  expect((await save.boundingBox())!.width).toBeLessThan(300);
  expect((await section.getByRole('checkbox', { name: 'Include in directory searches', exact: true }).boundingBox())!.width).toBeLessThanOrEqual(26);
  await page.screenshot({ path: test.info().outputPath('directory-compact-actions.png') });
  await signInManually(page, { username: 'testuser', password: 'testpass' });
  await expect(page.getByRole('button', { name: 'Compose', exact: true })).toBeVisible();
  const response = await page.request.get('/api/directory/search?q=alice');
  expect(response.status()).toBe(200);
  const value = await response.json() as { entries: { mail: string }[] };
  expect(value.entries.some(row => row.mail.includes('alice'))).toBe(true);
  await test.info().attach('real-directory-search', { body: JSON.stringify({ status: response.status(), count: value.entries.length }), contentType: 'application/json' });
});
