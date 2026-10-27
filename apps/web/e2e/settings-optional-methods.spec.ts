import { test, expect } from '@playwright/test';
import { login, settingsSurface, sidebarInbox } from './helpers.ts';

// t32-h6: in proxy mode every JMAP method goes to the upstream, and the in-repo
// mock (like third-party JMAP servers) does not implement `ServerMetadata/get`.
// Opening Settings used to replace the whole mailbox with "Something went wrong";
// now the annotations section says the server does not provide them.

test.describe('settings against an upstream without Mailwoman extensions', () => {
  test('opening Settings keeps the mailbox and contains the missing ServerMetadata/get', async ({ page }) => {
    await login(page);

    await page.getByRole('button', { name: 'Settings' }).click();
    const dialog = settingsSurface(page);
    await expect(dialog).toBeVisible();

    // Settings is sectioned (35fdc0ec): the annotations live under "Server
    // information" and load only when that section opens.
    const answered = page.waitForResponse(
      async (res) => res.url().endsWith('/jmap/api') && (res.request().postData() ?? '').includes('ServerMetadata/get'),
    );
    await dialog.getByRole('navigation', { name: 'Settings sections' }).getByRole('button', { name: 'Server information' }).click();

    // The upstream really answered unknownMethod (the case this guards).
    expect(await (await answered).text()).toContain('unknownMethod');

    await expect(dialog.getByTestId('metadata-unavailable')).toHaveText(
      'This mail server does not provide annotations.',
    );
    await expect(page.getByText('Something went wrong')).toHaveCount(0);

    // Close Settings: the mailbox shell is still there. Settings is a workspace
    // tab now (24fb6149), closed by its own button rather than Escape.
    await dialog.getByRole('button', { name: 'Close settings' }).click();
    await expect(dialog).toBeHidden();
    await expect(page.getByRole('button', { name: 'Compose' })).toBeVisible();
    await expect(sidebarInbox(page)).toBeVisible();
  });
});
