import { test, expect, type Page } from '../csrf.ts';
import { signInManually } from '../helpers.ts';
async function openNotifications(page: Page): Promise<void> {
  const drawer = page.getByRole('button', { name: 'Open folders and apps' });
  if (await drawer.isVisible()) await drawer.click();
  const show = page.getByRole('button', { name: 'Show folders sidebar' });
  if (await show.isVisible()) await show.click();
  await page.getByRole('button', { name: 'Settings', exact: true }).click();
  await page.getByRole('navigation', { name: 'Settings sections' }).getByRole('button', { name: 'Notifications', exact: true }).click();
  await expect(page.getByRole('region', { name: 'Notifications', exact: true })).toBeVisible();
}
test('notification settings persist with explicit device permissions and rule limits @phone', async ({ page }) => {
  await page.goto('/'); await signInManually(page, { username: 'testuser', password: 'testpass' });
  await expect(page.getByRole('toolbar', { name: 'Workspace layout' })).toBeVisible({ timeout: 60_000 });
  const me = await (await page.request.get('/api/me')).json();
  expect(me.storageAuthority).toMatch(/^msa1_/);
  const reset = await page.request.put('/api/account/notifications', { headers: { 'X-Mailwoman-Storage-Authority': me.storageAuthority }, data: { enabled: false, rules: [], quietHours: { enabled: false, start: '22:00', end: '07:00', timezone: 'UTC' } } });
  expect(reset.ok()).toBe(true); await openNotifications(page);
  const section = page.getByRole('region', { name: 'Notifications', exact: true });
  await expect(section.getByRole('checkbox', { name: 'Show notifications for new mail' })).not.toBeChecked();
  await section.getByRole('checkbox', { name: 'Show notifications for new mail' }).check();
  await section.getByRole('checkbox', { name: 'Silence notifications during a set time' }).check();
  await section.getByLabel('Quiet-hours time zone (IANA)').fill('Europe/Lisbon');
  await section.getByTestId('notif-add-rule').click();
  await section.getByTestId('notif-rule-list').locator('input').fill('private');
  await section.getByTestId('notif-rule-list').getByRole('combobox').selectOption('mute');
  const saved = page.waitForResponse(response => response.url().endsWith('/api/account/notifications') && response.request().method() === 'PUT');
  await section.getByTestId('notif-save').click(); expect((await saved).status()).toBe(200);
  const value = await (await page.request.get('/api/account/notifications')).json();
  expect(value.enabled).toBe(true); expect(value.quietHours.timezone).toBe('Europe/Lisbon'); expect(value.rules[0].action).toBe('mute');
  await expect(section.getByText(/Background system notifications are suppressed/)).toBeVisible();
  const storage = section.getByRole('button', { name: 'Request persistent storage' });
  if (await storage.isVisible()) await expect(storage).toBeDisabled();
  await page.reload(); await expect(page.getByRole('toolbar', { name: 'Workspace layout' })).toBeVisible({ timeout: 60_000 }); await openNotifications(page);
  await expect(page.getByLabel('Quiet-hours time zone (IANA)')).toHaveValue('Europe/Lisbon');
  await page.screenshot({ path: test.info().outputPath('notifications-permissions.png'), fullPage: true });
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
});
