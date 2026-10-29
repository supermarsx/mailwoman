import { test, expect, type Page } from '../csrf.ts';
import { ADMIN } from './admin-path.ts';
import { signInManually } from '../helpers.ts';
async function adminLogin(page: Page): Promise<void> {
  await page.goto(ADMIN); await page.getByLabel('Admin username').fill('root');
  await page.getByLabel('Password', { exact: true }).fill('hunter2');
  const signedIn = page.waitForResponse((response) => response.url().endsWith(`${ADMIN}/login`) && response.request().method() === 'POST');
  await page.getByRole('button', { name: 'Sign in', exact: true }).click();
  expect((await signedIn).ok()).toBe(true);
  await expect(page.getByRole('button', { name: 'Sign in', exact: true })).toHaveCount(0);
}
test('mailbox and administrator session tables revoke only selected sessions @phone', async ({ page, browser, baseURL }) => {
  const marker = `SessionProof-${Date.now()}`;
  const context = await browser.newContext({ ...(baseURL ? { baseURL } : {}), userAgent: marker, viewport: page.viewportSize() });
  const other = await context.newPage();
  try {
    await other.goto('/'); await signInManually(other, { username: 'testuser', password: 'testpass' });
    await expect(other.getByRole('toolbar', { name: 'Workspace layout' })).toBeVisible();
    const showFolders = other.getByRole('button', { name: 'Show folders sidebar' });
    if (await showFolders.isVisible()) await showFolders.click();
    if (test.info().project.name === 'phone') await other.getByRole('button', { name: 'Open folders and apps' }).click();
    await other.getByRole('button', { name: 'Settings', exact: true }).click();
    await other.getByRole('navigation', { name: 'Settings sections' }).getByRole('button', { name: 'Security centre', exact: true }).click();
    // 2b1695c0 split the Security centre into tabs; the sessions table is on "Sessions & access".
    await other.getByRole('tablist', { name: 'Security centre' }).getByRole('tab', { name: 'Sessions & access', exact: true }).click();
    await expect(other.getByRole('table', { name: 'Active sessions' })).toBeVisible();
    await expect(other.getByText('This session', { exact: true }).first()).toBeVisible();
    await other.screenshot({ path: test.info().outputPath('account-sessions-table.png'), fullPage: true });
    expect(await other.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
    await adminLogin(page); await page.goto(`${ADMIN}#/sessions`);
    await page.getByRole('searchbox', { name: 'Search account, device or IP' }).fill(marker);
    const mailbox = page.getByRole('table', { name: 'Mailbox sessions' });
    await expect(mailbox.locator('tbody tr')).toHaveCount(1);
    await expect(mailbox).toContainText(marker);
    if (test.info().project.name !== 'phone') expect(await mailbox.locator('..').evaluate((node) => node.scrollWidth <= node.clientWidth)).toBe(true);
    await page.screenshot({ path: test.info().outputPath('mailbox-sessions-table.png'), fullPage: true });
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
    await mailbox.getByRole('button', { name: /Revoke session for/ }).click();
    await page.getByRole('alertdialog').getByRole('button', { name: 'Revoke session', exact: true }).click();
    await expect(mailbox.locator('tbody tr')).toHaveCount(0);
    expect((await other.request.get('/api/me')).status()).toBe(401);
    await adminLogin(other);
    const rows = await (await other.request.get(`${ADMIN}/admin-sessions`)).json();
    const target = rows.sessions.find((row: { current: boolean }) => row.current);
    expect(target).toBeTruthy();
    await page.goto(`${ADMIN}#/admins`);
    const admins = page.getByRole('table', { name: 'Administrator sessions' });
    const row = admins.locator(`[data-row-key="${target.id}"]`);
    await expect(row).toBeVisible();
    await expect(admins.getByRole('columnheader', { name: 'Idle expiry' })).toBeVisible();
    await page.screenshot({ path: test.info().outputPath('administrator-sessions-table.png'), fullPage: true });
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
    await row.getByRole('button', { name: /Revoke session for/ }).click();
    await page.getByRole('alertdialog').getByRole('button', { name: 'Confirm change', exact: true }).click();
    await expect(row).toHaveCount(0);
    expect((await other.request.get(`${ADMIN}/me`)).status()).toBe(401);
    expect((await page.request.get(`${ADMIN}/me`)).ok()).toBe(true);
  } finally { await context.close(); }
});
