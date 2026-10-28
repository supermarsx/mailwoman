import { test, expect } from '../csrf.ts';
import { signInManually, messageRow, sidebarInbox, drawerButton, storageAuthorityHeaders, ribbonControl } from '../helpers.ts';
import { sendRawSmtp } from '../crypto-helpers.ts';

test('opted-in quick reply opens a compact tab and expands without losing text @phone', async ({ page, context }) => {
  const subject = `Quick reply ${Date.now()}`;
  await sendRawSmtp({ from: 'sender@example.test', message: `From: sender@example.test\r\nTo: testuser@example.org\r\nSubject: ${subject}\r\nMessage-ID: <quick-${Date.now()}@example.test>\r\nDate: ${new Date().toUTCString()}\r\nContent-Type: text/plain; charset=utf-8\r\n\r\nOriginal message for a compact reply.\r\n` });
  await signInManually(page, { username: 'testuser', password: 'testpass' });
  await expect(page.getByRole('button', { name: 'Compose', exact: true })).toBeVisible();
  const csrf = (await context.cookies()).find(cookie => cookie.name === 'mw_csrf')!.value;
  const headers = { 'X-CSRF-Token': csrf, ...(await storageAuthorityHeaders(page.request)) };
  const initial = (await (await page.request.get('/api/settings')).json()).settings['mail.quickReply'];
  try {
    const response = await page.request.put('/api/settings/mail.quickReply', { headers, data: { value: true } });
    expect(response.ok(), await response.text()).toBe(true);
    await page.reload();
    await expect(async () => {
      if (test.info().project.name === 'phone') await drawerButton(page).click();
      await sidebarInbox(page).click();
      await expect(messageRow(page, subject)).toBeVisible({ timeout: 3000 });
    }).toPass({ timeout: 30000 });
    await messageRow(page, subject).click();
    await page.getByRole('button', { name: 'Quick reply', exact: true }).click();
    await expect(page.getByRole('tab', { name: `Re: ${subject}`, exact: true })).toHaveAttribute('aria-selected', 'true');
    const composer = page.getByRole('region', { name: 'Compose message', exact: true });
    await expect(composer.getByLabel('To', { exact: true })).toHaveCount(0);
    await expect(composer.getByRole('region', { name: 'Quick reply', exact: true })).toContainText('sender@example.test');
    // On a phone the Message type group folds into a dropdown (t38-e5).
    await (await ribbonControl(composer, 'Message type', composer.getByTestId('format-toggle'))).selectOption('plain'); // the format toggle is a message-type select now (plain | rich | html)
    await composer.getByLabel('Body', { exact: true }).fill('My preserved quick reply');
    await composer.getByRole('button', { name: 'Edit recipients and subject' }).click();
    await expect(composer.getByLabel('To', { exact: true })).toHaveValue(/sender@example.test/);
    await expect(composer.getByLabel('Body', { exact: true })).toHaveValue('My preserved quick reply');
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
    await page.screenshot({ path: test.info().outputPath('quick-reply-expanded.png'), fullPage: true });
  } finally {
    const restore = initial?.userValue === undefined
      ? await page.request.delete('/api/settings/mail.quickReply', { headers })
      : await page.request.put('/api/settings/mail.quickReply', { headers, data: { value: initial.userValue } });
    expect(restore.ok(), await restore.text()).toBe(true);
  }
});
