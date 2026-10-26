import { test, expect } from '../csrf.ts';
import { signInManually, messageRow, sidebarInbox, drawerButton } from '../helpers.ts';
import { sendRawSmtp } from '../crypto-helpers.ts';

test('message spam scores show header results and expandable rules @phone', async ({ page }) => {
  const subject = `Spam scores ${Date.now()}`;
  await sendRawSmtp({ from: 'sender@example.test', message: [
    'From: sender@example.test', 'To: testuser@example.org',
    `Date: ${new Date().toUTCString()}`, `Message-ID: <scores-${Date.now()}@example.test>`,
    `Subject: ${subject}`, 'X-Spam-Status: No, score=-1.2 required=5.0 tests=DKIM_VALID,SPF_PASS',
    'X-Spamd-Result: default: False [-0.50 / 15.00]; DKIM_ALLOW(-0.20)[]; SPF_ALLOW(-0.30)[];',
    'Content-Type: text/plain; charset=utf-8', '', 'Message with reported spam analysis.', '',
  ].join('\r\n') });
  await signInManually(page, { username: 'testuser', password: 'testpass' });
  await expect(async () => {
    if (test.info().project.name === 'phone') await drawerButton(page).click();
    await sidebarInbox(page).click();
    await expect(messageRow(page, subject)).toBeVisible({ timeout: 3000 });
  }).toPass({ timeout: 30_000 });
  await messageRow(page, subject).click();
  const widget = page.locator('.spam-score');
  await expect(widget).toContainText('SpamAssassin -1.2 / 5');
  await expect(widget).toContainText('Rspamd -0.5 / 15');
  await widget.locator('summary').filter({ hasText: 'Rspamd' }).click();
  await expect(widget.getByText('DKIM_ALLOW (-0.20)', { exact: true })).toBeVisible();
  await expect(widget.getByText(/not independently verified/).last()).toBeVisible();
  await expect(page.getByRole('heading', { name: subject, exact: true })).toBeVisible();
  await page.screenshot({ path: test.info().outputPath('spam-score-widget.png'), fullPage: true });
});
