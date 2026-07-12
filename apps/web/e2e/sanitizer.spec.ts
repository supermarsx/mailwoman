import { test, expect } from '@playwright/test';
import { login, messageRow } from './helpers.ts';

/**
 * Proves the sanitize-through-render-child path is genuinely wired: the hostile
 * seeded message ("Your invoice is ready") carries a <script> sentinel, a remote
 * tracking pixel, and a javascript: link. After going through mw-server ->
 * mw-render, none of that must survive into the reader, and the reader iframe
 * must be sandboxed so nothing could execute even if it did.
 */
test.describe('sanitizer wiring', () => {
  test('hostile message is neutralized and rendered in a locked-down iframe', async ({ page }) => {
    await login(page);

    await messageRow(page, 'Your invoice is ready').click();
    const frame = page.locator('iframe[title="Message body"]');
    await expect(frame).toBeVisible();

    // The reader iframe is sandboxed WITHOUT allow-scripts / allow-same-origin.
    const sandbox = await frame.getAttribute('sandbox');
    expect(sandbox).not.toBeNull();
    expect(sandbox).not.toContain('allow-scripts');
    expect(sandbox).not.toContain('allow-same-origin');

    // Wait for the sanitized body to be injected, then inspect it.
    await expect.poll(async () => await frame.getAttribute('srcdoc')).toContain('Please review.');
    const srcdoc = (await frame.getAttribute('srcdoc')) ?? '';

    // Hostile content is gone from the sanitized DOM.
    expect(srcdoc).not.toContain('<script');
    expect(srcdoc).not.toContain('__mw_pwned');
    expect(srcdoc).not.toContain('tracker.evil.example');
    expect(srcdoc).not.toContain('javascript:');
    // Legit content survived.
    expect(srcdoc).toContain('Please review.');

    // The script never executed on the TOP page: its sentinel global is unset.
    // (Any escape would have run at render time, which has already happened.)
    const pwned = await page.evaluate(() => (window as unknown as { __mw_pwned?: unknown }).__mw_pwned);
    expect(pwned).toBeUndefined();

    // The sanitized (script-less) body still renders visibly inside the frame.
    await expect(
      page.frameLocator('iframe[title="Message body"]').getByText('Please review.'),
    ).toBeVisible();
  });
});
