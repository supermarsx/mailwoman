import { test, expect } from '../csrf.ts';
import { signInManually, messageRow } from '../helpers.ts';
import { sendRawSmtp } from '../crypto-helpers.ts';

test('webmail reader and module entries preserve geometry and respect reduced motion @phone', async ({ page }) => {
  const subject = `Motion ${Date.now()}-${test.info().workerIndex}`;
  await sendRawSmtp({ from: 'sender@example.test', message: `From: sender@example.test\r\nTo: testuser@example.org\r\nDate: ${new Date().toUTCString()}\r\nMessage-ID: <motion-${Date.now()}@example.test>\r\nSubject: ${subject}\r\nContent-Type: text/plain; charset=utf-8\r\n\r\nMotion verification.\r\n` });
  await page.emulateMedia({ reducedMotion: 'no-preference' });
  await page.goto('/');
  const login = page.locator('main.login');
  await expect(login).toBeVisible();
  expect(await login.evaluate(node => getComputedStyle(node).animationName)).not.toBe('none');
  await signInManually(page, { username: 'testuser', password: 'testpass' });
  const row = messageRow(page, subject);
  await expect(row).toBeVisible({ timeout: 30_000 });
  const reader = page.locator('section.reader');
  await row.click();
  await expect(reader).toHaveAttribute('data-mw-entering', 'true');
  expect(await reader.evaluate(node => getComputedStyle(node).animationName)).not.toBe('none');
  expect(await reader.evaluate(node => getComputedStyle(node).transform)).toBe('none');
  await page.emulateMedia({ reducedMotion: 'reduce' });
  await expect.poll(() => reader.evaluate(node => getComputedStyle(node).animationName)).toBe('none');

  // Hash navigation uses the live shell router, retaining the authenticated app.
  await page.evaluate(() => { location.hash = '#/calendar'; });
  const calendar = page.locator('main.module-pane[data-surface="calendar"]');
  await expect(calendar).toBeVisible();
  expect(await calendar.evaluate(node => getComputedStyle(node).animationName)).toBe('none');
  await page.emulateMedia({ reducedMotion: 'no-preference' });
  await page.evaluate(() => { location.hash = '#/contacts'; });
  const contacts = page.locator('main.module-pane[data-surface="contacts"]');
  await expect(contacts).toBeVisible();
  expect(await contacts.evaluate(node => getComputedStyle(node).animationName)).not.toBe('none');
  expect(await contacts.evaluate(node => getComputedStyle(node).transform)).toBe('none');
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  await page.screenshot({ path: test.info().outputPath('webmail-motion-contacts.png'), fullPage: true });
});
