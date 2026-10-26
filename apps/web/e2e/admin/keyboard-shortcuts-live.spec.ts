import { test, expect } from '../csrf.ts';
import { signInManually, drawerButton } from '../helpers.ts';

test('dedicated shortcut editor persists a custom binding and the live dispatcher uses it @phone', async ({ page, context }) => {
  await signInManually(page, { username: 'testuser', password: 'testpass' });
  await expect(page.getByRole('button', { name: 'Compose', exact: true })).toBeVisible();
  const csrf = (await context.cookies()).find(cookie => cookie.name === 'mw_csrf')!.value;
  const headers = { 'X-CSRF-Token': csrf };
  const initial = (await (await page.request.get('/api/settings')).json()).settings;
  const key = 'shortcuts.compose';
  try {
    if (test.info().project.name === 'phone') await drawerButton(page).click();
    await page.locator('.sidebar__settings').click();
    await page.getByRole('button', { name: 'Keyboard shortcuts', exact: true }).click();
    await page.getByRole('button', { name: 'Edit Compose shortcut', exact: true }).click();
    const editor = page.getByRole('region', { name: 'Edit shortcut', exact: true });
    await expect(editor.getByRole('table')).toHaveCount(0);
    await editor.getByLabel('Binding', { exact: true }).fill('Ctrl+Shift+K');
    await editor.getByRole('button', { name: 'Save shortcut', exact: true }).click();
    await expect(page.getByRole('region', { name: 'Keyboard shortcuts', exact: true }).getByRole('table')).toBeVisible();
    const stored = (await (await page.request.get('/api/settings')).json()).settings[key];
    expect(stored.userValue).toBe('Ctrl+Shift+K');
    const settingsLoaded = page.waitForResponse(response => new URL(response.url()).pathname === '/api/settings' && response.request().method() === 'GET' && response.status() === 200);
    await page.reload(); await settingsLoaded;
    const mailTab = page.getByRole('tab', { name: 'Mail', exact: true });
    await expect(mailTab).toBeVisible();
    await mailTab.focus();
    await page.keyboard.press('Control+Shift+K');
    await expect(page.getByRole('region', { name: 'Compose message', exact: true })).toBeVisible();
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
    await page.screenshot({ path: test.info().outputPath('custom-shortcut-composer.png'), fullPage: true });
  } finally {
    const previous = initial[key]?.userValue;
    const restored = previous === undefined
      ? await page.request.delete(`/api/settings/${key}`, { headers })
      : await page.request.put(`/api/settings/${key}`, { headers, data: { value: previous } });
    expect(restored.ok(), await restored.text()).toBe(true);
  }
});
