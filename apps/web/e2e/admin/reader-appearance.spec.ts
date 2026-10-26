import { test, expect } from '../csrf.ts';
import { signInManually, messageRow } from '../helpers.ts';
import { sendRawSmtp } from '../crypto-helpers.ts';

test('isolated messages adapt colors, preserve original mode and invert without changing sandbox @phone', async ({ page }) => {
  const subject = `Reader colors ${Date.now()}`;
  await sendRawSmtp({ from: 'sender@example.test', message: `From: sender@example.test\r\nTo: testuser@example.org\r\nDate: ${new Date().toUTCString()}\r\nMessage-ID: <reader-colors-${Date.now()}@example.test>\r\nSubject: ${subject}\r\nContent-Type: text/html; charset=utf-8\r\n\r\n<table bgcolor="white"><tr><td style="color:black;background-color:white">Reader appearance fixture</td></tr></table>\r\n` });
  // Exercise renderer policies against a controlled effective-settings response;
  // the message, sanitizer, frame bridge and account are the real runtime.
  let mode = 'adaptive';
  await page.route('**/api/settings', async route => {
    const response = await route.fetch(), body = await response.json();
    body.settings['reader.colorMode'] = { value: mode, source: 'global', locked: false };
    body.settings['reader.fontSize'] = { value: 18, source: 'user', locked: false };
    await route.fulfill({ response, json: body });
  });
  // Other admin specs share this source IP; respect the initial-login cooldown.
  await new Promise(resolve => setTimeout(resolve, 3100));
  await signInManually(page, { username: 'testuser', password: 'testpass' });
  await expect(page.getByRole('button', { name: 'Compose', exact: true })).toBeVisible();
  for (const colorMode of ['adaptive', 'original', 'invert']) {
    mode = colorMode;
    if (colorMode !== 'adaptive') await page.reload();
    await messageRow(page, subject).click();
    const outer = page.frameLocator('iframe.reader__frame');
    const inner = outer.frameLocator('iframe');
    await expect(inner.getByText('Reader appearance fixture')).toBeVisible();
    await expect.poll(() => inner.locator('body').evaluate(node => getComputedStyle(node).fontSize)).toBe('18px');
    const html = await inner.locator('html').evaluate(node => ({ filter: getComputedStyle(node).filter, background: getComputedStyle(node).backgroundColor }));
    if (colorMode === 'adaptive') {
      expect(await inner.locator('td').evaluate(node => getComputedStyle(node).backgroundColor)).toBe('rgba(0, 0, 0, 0)');
      expect(html.filter).toBe('none');
    } else if (colorMode === 'original') {
      expect(html.background).toBe('rgb(255, 255, 255)');
      expect(html.filter).toBe('none');
    } else expect(html.filter).toContain('invert(1)');
    await expect(outer.locator('iframe')).toHaveAttribute('sandbox', '');
    expect(await page.evaluate(() => document.documentElement.scrollHeight <= innerHeight + 1)).toBe(true);
    await page.screenshot({ path: test.info().outputPath(`reader-${colorMode}.png`), fullPage: true });
  }
});
