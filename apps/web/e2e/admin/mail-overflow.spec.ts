import { test, expect } from '../csrf.ts';
import { signInManually, messageRow } from '../helpers.ts';
import { sendRawSmtp } from '../crypto-helpers.ts';

test('large virtual folder and long message stay inside the viewport @phone', async ({ page }) => {
  const subject = `Overflow ${Date.now()}-${test.info().workerIndex}`;
  await sendRawSmtp({ from: 'sender@example.test', message: `From: sender@example.test\r\nTo: testuser@example.org\r\nDate: ${new Date().toUTCString()}\r\nMessage-ID: <overflow-${Date.now()}@example.test>\r\nSubject: ${subject}\r\nContent-Type: text/html; charset=utf-8\r\n\r\n${'<p>Long message content with ordinary text.</p>'.repeat(1000)}\r\n` });
  await signInManually(page, { username: 'testuser', password: 'testpass' });
  const row = messageRow(page, subject);
  await expect(row).toBeVisible({ timeout: 30_000 });
  // Stress the exact virtual-list spacer used for large folders, independent of
  // upstream mailbox count. It must never become the grid's minimum height.
  const original = await page.locator('.list__items').evaluate(node => {
    const list = node as HTMLElement; const value = list.style.height;
    list.style.height = '1000000px'; return value;
  });
  const geometry = await page.evaluate(() => {
    const shell = document.querySelector('.shell')!, scroll = document.querySelector('.list__scroll')!;
    return { height: shell.getBoundingClientRect().height, viewport: innerHeight, document: document.documentElement.scrollHeight, scroll: scroll.scrollHeight, client: scroll.clientHeight };
  });
  expect(geometry.height).toBeLessThanOrEqual(geometry.viewport + 1);
  expect(geometry.document).toBeLessThanOrEqual(geometry.viewport + 1);
  expect(geometry.scroll).toBeGreaterThan(geometry.client * 10);
  expect(geometry.client).toBeGreaterThan(50);
  await page.locator('.list__items').evaluate((node, value) => { (node as HTMLElement).style.height = value; }, original);
  await row.click();
  await expect(page.locator('iframe.reader__frame')).toBeVisible();
  expect(await page.evaluate(() => document.documentElement.scrollHeight <= innerHeight + 1)).toBe(true);
  expect(await page.locator('.reader').evaluate(node => node.getBoundingClientRect().height <= innerHeight + 1)).toBe(true);
  await page.screenshot({ path: test.info().outputPath('bounded-message-pane.png'), fullPage: true });
});
