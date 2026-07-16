import { test, expect } from '@playwright/test';
import { engineLogin, uid, waitForInboxMessage, openMessage, injectHtmlViaSmtp } from './crypto-helpers.ts';

/**
 * 26.12 sanitizer CSS-rewrite live E2E (audit #4, SPEC §7.2 item 3, plan §2 Batch-4).
 *
 * The 26.12 mw-sanitize no longer wholesale-strips CSS — it PARSES it, keeps an
 * allowlist of visual properties, namespaces every selector under the message
 * container (`.mw-email-body`), and drops `position:fixed`/`sticky`, `@import`, and
 * external `url()` references (clamping z-index). This proves that rewrite runs on a
 * REAL server-rendered message: an HTML email is delivered over SMTP into the
 * Greenmail account, ingested by the engine, and rendered through mw-server →
 * mw-render (native `sanitize_email_html`) into the reader's sandboxed iframe. We
 * then inspect the iframe `srcdoc` (the sanitized DOM the user actually sees):
 *
 *   • a benign, namespace-able rule SURVIVES and is scoped under `.mw-email-body`;
 *   • `position:fixed` is DROPPED (overlay/clickjacking hardening);
 *   • `@import` is DROPPED (no remote stylesheet pull);
 *   • an external `url(https://…)` (tracking pixel / remote asset) is DROPPED,
 *     while the visible text still renders.
 *
 * This is the browser end of the §7.2 CSS gate; the property-allowlist / z-index-clamp
 * matrix is unit-proven in `crates/mw-sanitize` (26 tests).
 */

const readerFrame = 'iframe.reader__frame';

test('sanitizer keeps namespaced CSS but drops position:fixed / @import / external url() in a real render', async ({
  page,
}) => {
  test.setTimeout(90_000); // SMTP delivery + engine ingest + server-side render

  const token = uid();
  const subject = `CSS sanitize ${token}`;
  const promo = `PROMO_${token}`;
  const sticky = `STICKY_${token}`;
  const pixel = `PIXEL_${token}`;
  const importHost = `import-${token}.evil.example`;
  const trackerHost = `tracker-${token}.evil.example`;

  // The message CSS mixes a benign namespace-able rule with the three things the
  // rewrite must strip: a fixed-position overlay, an @import, and an external
  // url() background. `.promo` also proves class selectors survive namespacing.
  const html =
    `<html><head><style>` +
    `@import url("https://${importHost}/inject.css");` +
    `.promo { color: rgb(10, 120, 200); font-weight: bold; }` +
    `.overlay { position: fixed; top: 0; left: 0; z-index: 2147483647; }` +
    `.bg { background: url("https://${trackerHost}/pixel.png") no-repeat; padding: 4px; }` +
    `</style></head><body>` +
    `<div class="promo">${promo}</div>` +
    `<div class="overlay">${sticky}</div>` +
    `<div class="bg">${pixel}</div>` +
    `</body></html>`;

  await engineLogin(page);
  await injectHtmlViaSmtp({
    from: 'newsletter@example.com',
    subject,
    text: `${promo} ${sticky} ${pixel}`,
    html,
  });

  await waitForInboxMessage(page, subject);
  await openMessage(page, subject);

  // The body renders in the locked-down reader iframe (sandbox omits allow-scripts
  // AND allow-same-origin — the standing §7.2 guarantee).
  const frame = page.locator(readerFrame);
  await expect(frame).toBeVisible();
  const sandbox = await frame.getAttribute('sandbox');
  expect(sandbox).not.toBeNull();
  expect(sandbox).not.toContain('allow-scripts');
  expect(sandbox).not.toContain('allow-same-origin');

  // Wait for the sanitized body to be injected (the visible marker appears).
  await expect.poll(async () => (await frame.getAttribute('srcdoc')) ?? '').toContain(promo);
  const srcdoc = (await frame.getAttribute('srcdoc')) ?? '';

  // The benign rule SURVIVED and is namespaced under the message container.
  expect(srcdoc, 'namespaced container present').toContain('mw-email-body');
  expect(srcdoc, 'benign selector survives').toContain('.promo');
  expect(srcdoc, 'benign declaration survives').toMatch(/color\s*:/);
  // The namespacing scopes `.promo` under `.mw-email-body` (not a bare top-level rule).
  expect(srcdoc).toMatch(/\.mw-email-body[^{]*\.promo/);

  // `position:fixed` was DROPPED (no fixed-position overlay reaches the DOM).
  expect(srcdoc, 'position:fixed dropped').not.toMatch(/position\s*:\s*fixed/i);
  // `@import` was DROPPED — no remote stylesheet pull, and the import host is gone.
  expect(srcdoc, '@import dropped').not.toContain('@import');
  expect(srcdoc, 'import host dropped').not.toContain(importHost);
  // The external `url(https://…)` (tracking pixel) was DROPPED.
  expect(srcdoc, 'external url() dropped').not.toContain(trackerHost);
  expect(srcdoc, 'no external https url in css').not.toMatch(/url\(\s*["']?https?:/i);

  // The visible text still renders inside the frame (sanitizing CSS didn't eat content).
  const inFrame = page.frameLocator(readerFrame);
  await expect(inFrame.getByText(promo)).toBeVisible();
  await expect(inFrame.getByText(pixel)).toBeVisible();
});
