import { test, expect } from '../csrf.ts';
import { signInManually } from '../helpers.ts';
test('genuine mailbox interaction renews activity but polling and admin usage do not @phone', async ({ page }) => {
  await page.clock.install();
  const renewals: string[] = [];
  page.on('request', (request) => { if (request.url().endsWith('/api/account/session/activity')) renewals.push(request.method()); });
  await page.goto('/'); await signInManually(page, { username: 'testuser', password: 'testpass' });
  await expect(page.getByRole('toolbar', { name: 'Workspace layout' })).toBeVisible();
  const initial = renewals.length;
  await page.evaluate(() => document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Shift', bubbles: true })));
  await page.clock.fastForward(31_000);
  expect(renewals).toHaveLength(initial);
  for (let index = 0; index < 2; index += 1) {
    const renewed = page.waitForResponse((response) => response.url().endsWith('/api/account/session/activity') && response.request().method() === 'POST');
    await page.keyboard.press('Shift');
    expect((await renewed).status()).toBe(200);
    await page.clock.fastForward(31_000);
    expect(renewals).toHaveLength(initial + index + 1);
  }
  await page.screenshot({ path: test.info().outputPath('mailbox-active-session.png'), fullPage: true });
  await page.goto('/admin'); await page.getByLabel('Admin username').fill('root');
  await page.getByLabel('Password', { exact: true }).fill('hunter2');
  const signedIn = page.waitForResponse(response => response.url().endsWith('/admin/login') && response.request().method() === 'POST');
  await page.getByRole('button', { name: 'Sign in', exact: true }).click(); expect((await signedIn).status()).toBe(200);
  await page.clock.fastForward(31_000); await page.keyboard.press('Shift');
  expect(renewals).toHaveLength(initial + 2);
});
