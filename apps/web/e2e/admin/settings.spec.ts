import { test, expect } from '../csrf.ts';
import { drawerButton, engineLogin, enginePhoneLogin, settingsSurface } from '../helpers.ts';
import { adminPath, ADMIN_PATH } from '../v6-helpers.ts';

/**
 * Admin console › Settings, end to end (t32-admin §3.4 e16 Settings part; worker f01).
 *
 *  - Sign in to the console, open Settings, find a setting by search, open it.
 *  - Lock the theme for every user from the console (the lock preview opens
 *    when users have their own value) and save.
 *  - The webmail's Settings shows the theme held ("Set by your administrator").
 *  - The audit log lists the change with the target setting:appearance.theme.
 *
 * Stack: engine mode with MW_UPSTREAM_DEFAULT and the dev admin credentials
 * (MW_ADMIN_USER / MW_ADMIN_PASSWORD, default root / hunter2; DEV/CI ONLY). The
 * console path follows MW_E2E_ADMIN_PATH. The lock is removed in `finally`.
 */

const ADMIN_USER = process.env['MW_ADMIN_USER'] ?? 'root';
const ADMIN_PASSWORD = process.env['MW_ADMIN_PASSWORD'] ?? 'hunter2';
const KEY = 'appearance.theme';

test('lock the theme in the console; the webmail shows it held; the audit lists it @phone', async ({
  page,
  browser,
  playwright,
  baseURL,
}) => {
  const api = await playwright.request.newContext({ baseURL: baseURL ?? '' });
  const login = await api.post(adminPath('/login'), { data: { username: ADMIN_USER, password: ADMIN_PASSWORD } });
  test.skip(login.status() !== 200, `the stack refused the dev admin credentials (${login.status()})`);
  try {
    // The console.
    await page.goto(ADMIN_PATH);
    await page.getByLabel('Admin username').fill(ADMIN_USER);
    await page.getByLabel('Password', { exact: true }).fill(ADMIN_PASSWORD);
    await page.getByRole('button', { name: 'Sign in', exact: true }).click();
    await page.goto(`${ADMIN_PATH}#/settings`);
    await page.getByRole('searchbox', { name: 'Search settings' }).fill('theme');
    await page.getByRole('link', { name: `Open ${KEY}` }).click();
    const detail = page.getByTestId('setting-detail');
    await expect(detail).toBeVisible();
    await detail.getByRole('textbox', { name: /^Theme/ }).fill('hc-dark');
    await detail.getByRole('switch').click();
    await detail.getByRole('button', { name: 'Save', exact: true }).click();
    const preview = page.getByRole('alertdialog');
    if (await preview.isVisible().catch(() => false)) {
      await preview.getByRole('button', { name: 'Lock' }).click();
    }
    // A result is a toast, not text in the detail (t32 f39).
    await expect(page.getByText('Saved. It applies within 30 seconds on every server process.')).toBeVisible();
    await expect(detail.getByTestId('source-badge').first()).toHaveAttribute('data-source', 'locked');

    // The webmail, in a second browser context.
    const mail = await browser.newContext();
    const user = await mail.newPage();
    // browser.newContext() takes the project's viewport: at phone width the folders and Settings sit in the drawer.
    if (test.info().project.name === 'phone') {
      await enginePhoneLogin(user);
      await drawerButton(user).click();
    } else await engineLogin(user);
    await user.getByRole('button', { name: 'Settings', exact: true }).click();
    const dialog = settingsSurface(user);
    await expect(dialog.locator('#settings-theme-lock')).toHaveText('Set by your administrator');
    await expect(dialog.getByRole('button', { name: 'Grove Dark' })).toBeDisabled();
    await mail.close();

    // The audit.
    const audit = await (await api.get(adminPath('/audit?limit=20'))).json();
    expect(
      (audit as { action: string; target: string }[]).some(
        (e) => e.action === 'setting-changed' && e.target === `setting:${KEY}`,
      ),
    ).toBe(true);
  } finally {
    await api.delete(adminPath(`/settings/${KEY}`));
    await api.dispose();
  }
});
