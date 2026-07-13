import { test, expect } from '@playwright/test';
import { engineLogin, injectHtmlViaSmtp, uid, waitForInboxMessage, openMessage } from './crypto-helpers.ts';

/**
 * V4 max-security opening live E2E (plan §3 e10 item 6 / SPEC §7.2): the three-
 * position Reader toolbar switch (Full / No media / Plain text) changes the ACTUAL
 * body render. Flipping to "No media" pins the sandboxed body frame to a media-free
 * CSP (images/media dropped); flipping to "Plain text" renders the message as escaped
 * plain text with no HTML at all. Pure client-side (no WASM worker) — the switch
 * drives the `bodyFrameDoc` CSP/sanitize mode in `viewers/sandbox.ts`.
 */
test('Max-security: the switch strips media (no-media) and renders plain text', async ({ page }) => {
  const token = uid();
  const subject = `MaxSec ${token}`;
  const htmlMark = `HTMLMARK_${token}`;
  const plainMark = `PLAINMARK_${token}`;

  // A message with both a text/plain part (used by plain-text mode) and a text/html
  // part carrying an <img> (used to prove media stripping).
  await injectHtmlViaSmtp({
    from: 'Newsletter <news@remote-example.net>',
    subject,
    text: `${plainMark} plain body`,
    html: `<p>${htmlMark}</p><img src="http://remote-example.net/tracker.png" alt="tracker">`,
  });

  await engineLogin(page);
  await waitForInboxMessage(page, subject);
  await openMessage(page, subject);

  const frame = page.locator('iframe.reader__frame');
  await expect(frame).toBeVisible();
  const switchEl = page.locator('[data-testid="max-security-switch"]');
  // Default posture is full-sanitized (the pre-V4 body render).
  await expect(switchEl).toHaveAttribute('data-mode', 'full-sanitized');

  // Flip to No media → the body frame is pinned to a media-free CSP (no img-src).
  await page.locator('[data-testid="max-security-opt-sanitized-no-media"]').click();
  await expect(switchEl).toHaveAttribute('data-mode', 'sanitized-no-media');
  await expect(async () => {
    const srcdoc = (await frame.getAttribute('srcdoc')) ?? '';
    expect(srcdoc).toContain("default-src 'none'");
    // The media-free CSP has NO img-src/media-src source at all.
    expect(srcdoc).not.toContain('img-src');
    expect(srcdoc).not.toContain('media-src');
  }).toPass({ timeout: 10_000 });

  // Flip to Plain text → the body renders as escaped plain text (no live HTML).
  await page.locator('[data-testid="max-security-opt-plain-text"]').click();
  await expect(switchEl).toHaveAttribute('data-mode', 'plain-text');
  await expect(async () => {
    const srcdoc = (await frame.getAttribute('srcdoc')) ?? '';
    expect(srcdoc).toContain('<pre>');
    expect(srcdoc).toContain(plainMark);
    // No live <img> element — media cannot render in plain-text mode.
    expect(srcdoc).not.toContain('<img');
  }).toPass({ timeout: 10_000 });
});
