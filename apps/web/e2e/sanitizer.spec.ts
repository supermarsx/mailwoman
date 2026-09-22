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
    expect(srcdoc).not.toContain('javascript:');

    // The tracker host must survive in exactly ONE place: the hidden block marker
    // the sanitizer appends on purpose (`data-mw-blocked-host`, t16 S9 —
    // crates/mw-sanitize/src/lib.rs). That marker is how the reader can say
    // "N trackers blocked" without a second round-trip
    // (analyzeBlockedContent, src/api/remote-images.ts), and by construction it
    // "never carries a loadable URL".
    //
    // This assertion used to be a flat `not.toContain('tracker.evil.example')`,
    // which failed on the breadcrumb and so called a working sanitizer broken. The
    // property that actually matters is that nothing can LOAD from the host, so
    // that is what is asserted: strip the breadcrumb attributes, then require the
    // host to be absent from everything that remains.
    expect(srcdoc, 'the block marker must be present and inert').toMatch(
      /<span[^>]*\bhidden\b[^>]*data-mw-blocked-host="tracker\.evil\.example"/,
    );
    const withoutMarkers = srcdoc.replace(/\sdata-mw-blocked-host="[^"]*"/g, '');
    expect(
      withoutMarkers,
      'outside the hidden marker the tracker host must not appear at all',
    ).not.toContain('tracker.evil.example');
    // And belt-and-braces: no loadable attribute ever names it.
    expect(srcdoc).not.toMatch(/(?:src|href|srcset|action|poster)\s*=\s*["'][^"']*tracker\.evil/i);

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
