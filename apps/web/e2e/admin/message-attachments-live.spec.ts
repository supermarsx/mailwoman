import { readFileSync } from 'node:fs';
import { test, expect, ANONYMOUS_PAIR } from '../csrf.ts';
import { signInManually, messageRow, sidebarInbox, drawerButton } from '../helpers.ts';
import { sendRawSmtp } from '../crypto-helpers.ts';

test('EML and Outlook attachments use the isolated preview worker @phone', async ({ page, request }) => {
  const eml = 'From: Alice <alice@example.test>\r\nTo: testuser@example.org\r\nSubject: Nested preview\r\nMIME-Version: 1.0\r\nContent-Type: text/html; charset=utf-8\r\n\r\n<p>Nested body</p><script>alert(1)</script><img src="https://preview-leak.invalid/pixel">';
  expect((await request.post('/api/attachments/preview', { headers: ANONYMOUS_PAIR, data: { format: 'eml', bytesBase64: Buffer.from(eml).toString('base64') } })).status()).toBe(401);
  const subject = `Preview attachments ${Date.now()}`;
  const boundary = `preview-${Date.now()}`;
  await sendRawSmtp({ from: 'sender@example.test', message: `From: sender@example.test\r\nTo: testuser@example.org\r\nDate: ${new Date().toUTCString()}\r\nMessage-ID: <${boundary}@example.test>\r\nSubject: ${subject}\r\nMIME-Version: 1.0\r\nContent-Type: multipart/mixed; boundary="${boundary}"\r\n\r\n--${boundary}\r\nContent-Type: text/plain\r\n\r\nPreview fixture\r\n--${boundary}\r\nContent-Type: message/rfc822\r\nContent-Disposition: attachment; filename="forwarded.eml"\r\nContent-Transfer-Encoding: base64\r\n\r\n${Buffer.from(eml).toString('base64')}\r\n--${boundary}--\r\n` });
  await signInManually(page, { username: 'testuser', password: 'testpass' });
  await expect(page.getByRole('button', { name: 'Compose', exact: true })).toBeVisible();
  const response = await page.request.post('/api/attachments/preview', { data: { format: 'eml', bytesBase64: Buffer.from(eml).toString('base64') } });
  expect(response.status(), await response.text()).toBe(200);
  expect(response.headers()['cache-control']).toBe('private, no-store');
  const body = await response.json();
  expect(body.preview.subject).toBe('Nested preview');
  expect(body.preview.html).not.toContain('<script');
  // The sanitizer records blocked hosts as inert metadata for diagnostics.
  expect(body.preview.html).not.toMatch(/(?:src|href)=["']https:\/\/preview-leak\.invalid/);
  const tnef = readFileSync(new URL('../../../../crates/mw-media-wasm/fixtures/tnef/rtf.tnef', import.meta.url));
  const outlook = await page.request.post('/api/attachments/preview', { data: { format: 'tnef', bytesBase64: tnef.toString('base64') } });
  expect(outlook.status(), await outlook.text()).toBe(200);
  expect((await outlook.json()).preview.text.length).toBeGreaterThan(0);
  expect((await page.request.post('/api/attachments/preview', { data: { format: 'eml', bytesBase64: '!!!' } })).status()).toBe(400);
  expect((await page.request.post('/api/attachments/preview', { headers: { 'x-csrf-token': 'wrong' }, data: { format: 'eml', bytesBase64: 'YWJj' } })).status()).toBe(403);
  await expect(async () => {
    if (test.info().project.name === 'phone') await drawerButton(page).click();
    await sidebarInbox(page).click();
    await expect(messageRow(page, subject)).toBeVisible({ timeout: 3000 });
  }).toPass({ timeout: 30_000 });
  await messageRow(page, subject).click();
  await page.getByRole('option', { name: /forwarded.eml/ }).click();
  const tab = page.getByRole('tabpanel', { name: 'forwarded.eml', exact: true });
  await expect(page.getByRole('tab', { name: 'forwarded.eml', exact: true })).toHaveAttribute('aria-selected', 'true');
  await expect(tab.getByRole('heading', { name: 'Nested preview' })).toBeVisible();
  await expect(tab.locator('iframe')).toHaveAttribute('sandbox', '');
  await expect(page.frameLocator('.attachment-tab iframe').getByText('Nested body')).toBeVisible();
  await tab.getByText('File details and fingerprint', {exact:true}).click();
  await expect(tab.locator('code').first()).toHaveText(/^[a-f0-9]{64}$/);
  await page.getByRole('tab', { name: subject, exact: true }).click();
  await expect(page.getByRole('heading', { name: subject, exact: true })).toBeVisible();
  await page.getByRole('tab', { name: 'forwarded.eml', exact: true }).click();
  await expect(tab.getByRole('heading', { name: 'Nested preview' })).toBeVisible();
  await page.screenshot({ path: test.info().outputPath('nested-message-preview.png'), fullPage: true });
});
