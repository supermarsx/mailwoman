import { test, expect } from '../csrf.ts';
import type { APIRequestContext, Page } from '@playwright/test';
import { ENGINE_CREDS, engineLogin, enginePhoneLogin, signInManually } from '../helpers.ts';
import { adminPath } from '../v6-helpers.ts';

/**
 * The settings model C in a real browser (t32-admin §2.8, §3.4 e17; worker f01).
 *
 *  - The sign-in form asks for the address and the password and nothing else, at
 *    desktop and phone width: no server, URL, host or port field.
 *  - A setting an administrator holds is shown as such in Settings ("Set by your
 *    administrator"), its control is disabled, and a direct API write is refused
 *    403 `setting_locked` while the stored value stays as it was.
 *  - A held appearance field is replaced in what the server stores.
 *
 * Runs against an engine-mode stack with MW_UPSTREAM_DEFAULT set and the dev admin
 * credentials (MW_ADMIN_USER / MW_ADMIN_PASSWORD, default root / hunter2; DEV/CI
 * ONLY). The admin writes go through the REST API and are undone in `finally`.
 */

const ADMIN_USER = process.env['MW_ADMIN_USER'] ?? 'root';
const ADMIN_PASSWORD = process.env['MW_ADMIN_PASSWORD'] ?? 'hunter2';

async function adminSignIn(request: APIRequestContext): Promise<void> {
  const r = await request.post(adminPath('/login'), { data: { username: ADMIN_USER, password: ADMIN_PASSWORD } });
  test.skip(r.status() !== 200, `the stack refused the dev admin credentials (${r.status()})`);
}

async function hold(request: APIRequestContext, key: string, value: unknown): Promise<void> {
  const r = await request.put(adminPath(`/settings/${key}`), { data: { globalValue: value, locked: true } });
  expect(r.status(), await r.text()).toBe(200);
}

async function release(request: APIRequestContext, key: string): Promise<void> {
  await request.delete(adminPath(`/settings/${key}`));
}

async function expectNoServerField(page: Page): Promise<void> {
  await page.goto('/');
  await expect(page.getByRole('button', { name: 'Sign in', exact: true })).toBeVisible();
  await expect(page.getByLabel('Email address', { exact: true })).toBeVisible();
  await expect(page.getByLabel('Password', { exact: true })).toBeVisible();
  const form = page.getByRole('form', { name: 'Sign in' });
  // One text field (the address) and one password field; nothing for a server.
  await expect(form.locator('input:not([type=hidden]):not([type=password])')).toHaveCount(1);
  for (const name of [/server/i, /url/i, /host/i, /port/i]) {
    await expect(form.getByLabel(name)).toHaveCount(0);
  }
}

test.describe('settings model C (t32 f01)', () => {
  test('the sign-in form has no server field @phone', async ({ page }) => {
    await expectNoServerField(page);
  });

  test('a held setting is marked, not editable, and refused by the API', async ({ page, playwright, baseURL }) => {
    const admin = await playwright.request.newContext({ baseURL: baseURL ?? '' });
    await adminSignIn(admin);
    try {
      await hold(admin, 'ui.language', 'en');
      await hold(admin, 'appearance.density', 'relaxed');
      await engineLogin(page);

      await page.getByRole('button', { name: 'Settings' }).click();
      const dialog = page.getByRole('dialog', { name: 'Settings' });
      await expect(dialog).toBeVisible();
      const language = dialog.getByRole('combobox', { name: 'Language' });
      await expect(language).toBeDisabled();
      await expect(language).toHaveValue('en');
      await expect(dialog.locator('#settings-language-lock')).toHaveText('Set by your administrator');
      await expect(dialog.getByRole('button', { name: 'Compact', exact: true })).toBeDisabled();
      await expect(dialog.getByRole('button', { name: 'Relaxed', exact: true })).toHaveAttribute('aria-pressed', 'true');
      await expect(dialog.locator('#settings-density-lock')).toBeVisible();

      // The browser's own session cannot change it either.
      const refused = await page.request.put('/api/settings/ui.language', { data: { value: 'de' } });
      expect(refused.status()).toBe(403);
      expect((await refused.json()).code).toBe('setting_locked');
      const all = await (await page.request.get('/api/settings')).json();
      expect(all.settings['ui.language']).toMatchObject({ value: 'en', locked: true, lockedBy: 'global' });

      // A held appearance field is replaced in what the server stores.
      const put = await page.request.put('/api/account/appearance', {
        data: { appearance: { mode: 'fixed', theme: 'dark', density: 'compact', accentId: '' } },
      });
      expect(put.status()).toBe(200);
      expect((await put.json()).lockedFields).toEqual(['density']);
      const stored = await (await page.request.get('/api/account/appearance')).json();
      expect(stored.appearance.density).toBe('relaxed');
    } finally {
      await release(admin, 'ui.language');
      await release(admin, 'appearance.density');
      await admin.dispose();
    }
  });

  test('released, the same control is free again @phone', async ({ page, playwright, baseURL }, testInfo) => {
    const admin = await playwright.request.newContext({ baseURL: baseURL ?? '' });
    await adminSignIn(admin);
    await release(admin, 'ui.language');
    await admin.dispose();
    if (testInfo.project.name === 'phone') {
      await enginePhoneLogin(page);
    } else {
      await signInManually(page, { username: ENGINE_CREDS.username, password: ENGINE_CREDS.password });
    }
    const ok = await page.request.put('/api/settings/ui.language', { data: { value: 'browser' } });
    expect(ok.status(), await ok.text()).toBe(200);
    expect((await ok.json()).locked).toBe(false);
  });
});
