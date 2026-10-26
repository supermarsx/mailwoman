import { test, expect } from '../csrf.ts';
import { signInManually } from '../helpers.ts';

test('account recipient policy persists and resets confirmation after an edit @phone', async ({ page, context }) => {
  await signInManually(page, { username: 'testuser', password: 'testpass' });
  await expect(page.getByRole('button', { name: 'Compose', exact: true })).toBeVisible();
  const csrf = (await context.cookies()).find(cookie => cookie.name === 'mw_csrf')?.value;
  expect(csrf).toBeTruthy();
  const headers = { 'X-CSRF-Token': csrf! };
  const initial = (await (await page.request.get('/api/settings')).json()).settings;
  const keys = ['mail.externalRecipientWarning', 'mail.trustedRecipientDomains'];
  try {
    for (const [key, value] of [['mail.externalRecipientWarning', 'confirm'], ['mail.trustedRecipientDomains', ['partner.test']]] as const) {
      const response = await page.request.put(`/api/settings/${key}`, { headers, data: { value } });
      expect(response.ok(), await response.text()).toBe(true);
    }
    await page.reload();
    await page.getByRole('button', { name: 'Compose', exact: true }).click();
    await page.getByLabel('To', { exact: true }).fill('recipient@outside.test');
    await expect(page.getByText('1 recipient(s) outside trusted domains')).toBeVisible();
    await page.getByRole('button', { name: 'Send', exact: true }).click();
    await expect(page.getByText('Review and confirm the external recipients before sending.')).toBeVisible();
    const review = page.getByLabel('I have checked these external recipients.');
    await review.check();
    await page.getByLabel('To', { exact: true }).fill('another@outside.test');
    await expect(review).not.toBeChecked();
    await page.screenshot({ path: test.info().outputPath('external-recipient-policy.png'), fullPage: true });
    await page.getByLabel('To', { exact: true }).fill('person@partner.test');
    await expect(review).toHaveCount(0);
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  } finally {
    for (const key of keys) {
      const previous = initial[key]?.userValue;
      const response = previous === undefined
        ? await page.request.delete(`/api/settings/${key}`, { headers })
        : await page.request.put(`/api/settings/${key}`, { headers, data: { value: previous } });
      expect(response.ok(), await response.text()).toBe(true);
    }
  }
});
