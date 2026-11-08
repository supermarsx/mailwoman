import { test, expect, type ConsoleMessage, type Page, type Response, type TestInfo } from '@playwright/test';
import { writeFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { login, messageRow } from './helpers.ts';
import { buildEmbeddedFontPdf, buildEncryptedPdf, buildLargePdf, buildPdf, buildScannedPdf } from './pdf-fixtures.ts';

export { buildPdf };

/**
 * A real PDF attachment opens end to end in the PDF.js viewer: the first page
 * canvas renders, no "Could not render this PDF." appears, every /pdfjs/* (and
 * the worker) request succeeds with the right type, and the browser raises no
 * sandbox warning or CSP violation. Run in Chromium (`mock`) and Firefox
 * (`mock-firefox`) against the same stack, over plain HTTP in CI and, by setting
 * PLAYWRIGHT_MOCK_BASE_URL, through a TLS/HTTP-2 reverse proxy (docs/admin/pdf-viewer.md,
 * "PDF troubleshooting").
 *
 * Runs against the mock-JMAP proxy stack. The mock has no attachments, so the
 * browser-side JMAP reply for one message is extended with a PDF part and its
 * download is answered with real PDF bytes; everything else (the mw-server, the
 * embedded viewer assets, the headers, the sandboxed frame) is the real thing.
 */

const BLOB_ID = 'blob-pdf-attachment';

async function seedPdfAttachment(page: Page, pdf: Buffer): Promise<void> {
  await page.route(/\/jmap\/download\/[^/]+\/blob-pdf-attachment\//, (route) =>
    route.fulfill({ status: 200, contentType: 'application/pdf', body: pdf }));
  await page.route(/\/jmap(\/api)?$/, async (route) => {
    const request = route.request();
    if (request.method() !== 'POST') return route.continue();
    const response = await route.fetch();
    let body = await response.text();
    try {
      const json = JSON.parse(body) as { methodResponses?: [string, { list?: Record<string, unknown>[] }, string][] };
      for (const [name, args] of json.methodResponses ?? []) {
        if (name !== 'Email/get') continue;
        for (const email of args.list ?? []) {
          if (email['subject'] !== 'Lunch tomorrow?') continue;
          email['hasAttachment'] = true;
          email['attachments'] = [{ partId: '2', blobId: BLOB_ID, type: 'application/pdf', name: 'report.pdf', size: pdf.length, disposition: 'attachment' }];
        }
      }
      body = JSON.stringify(json);
    } catch { /* not JSON */ }
    return route.fulfill({ response, body });
  });
}

/** What the browser reported while the viewer ran. */
interface Capture { console_: string[]; failures: string[]; pdfjs: { url: string; status: number; type: string; cors: string; corp: string }[] }
function capture(page: Page): Capture {
  const c: Capture = { console_: [], failures: [], pdfjs: [] };
  page.on('console', (m: ConsoleMessage) => c.console_.push(`${m.type()} [${m.location().url.replace(/^https?:..[^/]*/, '')}:${m.location().lineNumber}]: ${m.text()}`));
  page.on('pageerror', (e) => c.console_.push(`pageerror: ${e.message}`));
  page.on('requestfailed', (r) => /\/pdfjs\/|pdf\.worker/.test(r.url()) && !/ABORTED/.test(r.failure()?.errorText ?? '') && c.failures.push(`${r.url()} ${r.failure()?.errorText}`));
  page.on('response', (r: Response) => {
    const u = new URL(r.url());
    if (/\/pdfjs\/|\/pdf\.worker\.mjs$/.test(u.pathname)) c.pdfjs.push({ url: u.pathname, status: r.status(), type: r.headers()['content-type'] ?? '', cors: r.headers()['access-control-allow-origin'] ?? '', corp: r.headers()['cross-origin-resource-policy'] ?? '' });
  });
  return c;
}

async function openAttachment(page: Page): Promise<ReturnType<Page['getByRole']>> {
  await login(page);
  await messageRow(page, 'Lunch tomorrow?').click();
  const bar = page.getByTestId('reader-attachments-bar');
  await expect(bar).toBeVisible();
  // Attachments start collapsed; open the list.
  if (!(await page.getByRole('option', { name: 'report.pdf' }).isVisible())) await bar.getByRole('button').last().click();
  await expect(page.getByTestId('reader-attachments')).toBeVisible();
  await page.getByRole('option', { name: 'report.pdf' }).click();
  const tab = page.getByRole('tabpanel', { name: 'report.pdf', exact: true });
  await expect(tab).toBeVisible();
  return tab;
}

async function save(c: Capture, page: Page, info: TestInfo): Promise<void> {
  writeFileSync(info.outputPath('console.txt'), c.console_.join(String.fromCharCode(10)));
  writeFileSync(info.outputPath('pdfjs-requests.json'), JSON.stringify({ pdfjs: c.pdfjs, failures: c.failures }, null, 1));
  await page.screenshot({ path: info.outputPath('pdf-viewer.png') });
}

/** Every viewer request answered, nothing blocked, no sandbox warning, no policy/CORS/script error from the frame or its worker. */
function expectClean(c: Capture): void {
  for (const r of c.pdfjs) {
    expect(r.status, r.url).toBeLessThan(400);
    if (!r.url.endsWith('/locale.json')) expect(r.type, r.url).not.toContain('json');
    // The frame has an opaque origin: every file it loads (not the document itself) is a CORS / cross-origin request.
    if (r.url.endsWith('/pdfjs/web/mailwoman.html')) continue;
    expect(r.cors, `${r.url} access-control-allow-origin`).toBe('*');
    expect(r.corp, `${r.url} cross-origin-resource-policy`).toBe('cross-origin');
  }
  expect(c.failures).toEqual([]);
  expect(c.console_.filter((l) => /can remove its sandboxing|allow-scripts and allow-same-origin/i.test(l))).toEqual([]);
  expect(c.console_.filter((l) => /\[\/(pdfjs\/|pdf\.worker)/.test(l) && /Content[- ]Security[- ]Policy|violates|Refused to|blocked by CORS|has been blocked|SecurityError|Failed to load resource|Failed to fetch|Untrusted|may not load or link to file/i.test(l))).toEqual([]);
  // No file: access is requested by the page, the message frames or the viewer frame.
  expect(c.console_.filter((l) => /may not load or link to file:/i.test(l))).toEqual([]);
}

test.describe('PDF attachment', () => {
  test.setTimeout(120_000);

  test('opens, renders page 1, and the browser reports no sandbox warning or CSP violation', async ({ page }, info) => {
    const c = capture(page);
    try {
      await seedPdfAttachment(page, Buffer.from(buildPdf(3), 'latin1'));
      const tab = await openAttachment(page);
      const frame = tab.locator('iframe.mw-viewer__pdf-frame');
      // Least privilege: scripts, downloads and the print dialog; never the app's origin.
      expect(await frame.getAttribute('sandbox')).toBe('allow-scripts allow-downloads allow-modals');
      const inner = tab.frameLocator('iframe.mw-viewer__pdf-frame');
      await expect(inner.locator('.page canvas').first()).toBeVisible({ timeout: 30_000 });
      await expect(inner.locator('#numPages')).toContainText('3');
      await expect(tab.getByText('Could not render this PDF.')).toHaveCount(0);
      // The file facts stay behind a button; opening it shows the actual bytes'
      // fingerprint and the page count reported by the sandboxed viewer.
      await expect(page.getByTestId('attachment-details')).toHaveCount(0);
      const detailsButton = tab.getByTestId('attachment-details-button');
      await detailsButton.click();
      const details = page.getByTestId('attachment-details');
      await expect(details).toBeVisible();
      await expect(details).toContainText('report.pdf');
      const digest = createHash('sha256').update(Buffer.from(buildPdf(3), 'latin1')).digest('hex');
      await expect(details.getByTestId('attachment-fingerprint')).toHaveText(digest.match(/.{1,8}/g)!.join(' '));
      await expect(details.locator('dt').filter({ hasText: /^Pages$/ }).locator('+ dd')).toHaveText('3');
      await page.keyboard.press('Escape');
      await expect(details).toHaveCount(0);
      await expect(detailsButton).toBeFocused();
      // The worker really runs in a worker (not the main-thread fallback).
      expect(c.console_.filter((l) => /Setting up fake worker/i.test(l))).toEqual([]);
    } finally { await save(c, page, info); }
    expectClean(c);
  });

  const variants: [string, () => Buffer, (inner: ReturnType<Page['frameLocator']>) => Promise<void>][] = [
    ['embedded fonts', buildEmbeddedFontPdf, async (inner) => { await expect(inner.locator('#numPages')).toContainText('2'); await expect(inner.locator('.textLayer span').first()).toBeAttached(); }],
    ['a scanned image page', buildScannedPdf, async (inner) => { await expect(inner.locator('#numPages')).toContainText('1'); }],
  ];
  for (const [name, build, check] of variants) {
    test(`renders a PDF with ${name}`, async ({ page }, info) => {
      const c = capture(page);
      try {
        await seedPdfAttachment(page, build());
        const tab = await openAttachment(page);
        const inner = tab.frameLocator('iframe.mw-viewer__pdf-frame');
        await expect(inner.locator('.page canvas').first()).toBeVisible({ timeout: 30_000 });
        await check(inner);
        await expect(tab.getByText('Could not render this PDF.')).toHaveCount(0);
      } finally { await save(c, page, info); }
      expectClean(c);
    });
  }

  test('asks for the password of an encrypted PDF and renders it once given', async ({ page }, info) => {
    const c = capture(page);
    try {
      await seedPdfAttachment(page, buildEncryptedPdf('mailwoman'));
      const tab = await openAttachment(page);
      const inner = tab.frameLocator('iframe.mw-viewer__pdf-frame');
      await expect(inner.locator('#password')).toBeVisible({ timeout: 30_000 });
      await expect(tab.getByText('Could not render this PDF.')).toHaveCount(0);
      await inner.locator('#password').fill('mailwoman');
      await inner.locator('#passwordSubmit').click();
      await expect(inner.locator('.page canvas').first()).toBeVisible({ timeout: 30_000 });
      await expect(inner.locator('#numPages')).toContainText('1');
    } finally { await save(c, page, info); }
    expectClean(c);
  });

  test('renders a large (about 20 MB) PDF', async ({ page }, info) => {
    test.setTimeout(240_000);
    const c = capture(page);
    try {
      const pdf = buildLargePdf(20);
      expect(pdf.length).toBeGreaterThan(19 * 1024 * 1024);
      await seedPdfAttachment(page, pdf);
      const tab = await openAttachment(page);
      const inner = tab.frameLocator('iframe.mw-viewer__pdf-frame');
      await expect(inner.locator('.page canvas').first()).toBeVisible({ timeout: 120_000 });
      await expect(tab.getByText('Could not render this PDF.')).toHaveCount(0);
    } finally { await save(c, page, info); }
    expectClean(c);
  });

  // A browser a few releases old lacks built-ins PDF.js calls without a fallback
  // (Promise.try, Map.prototype.getOrInsertComputed, Math.sumPrecise, URL.parse,
  // RegExp.escape, Uint8Array base64/hex). Before the polyfills this showed "Could
  // not render this PDF". Removed here in the page, the viewer frame and (through
  // the polyfill module both load first) the worker.
  test('renders in a browser without the newer built-ins PDF.js uses', async ({ page }, info) => {
    const strip = 'try{delete Map.prototype.getOrInsert;delete Map.prototype.getOrInsertComputed;delete WeakMap.prototype.getOrInsert;delete WeakMap.prototype.getOrInsertComputed;delete Math.sumPrecise;delete Promise.try;delete RegExp.escape;delete URL.parse;delete Uint8Array.prototype.toHex;delete Uint8Array.prototype.toBase64;delete Uint8Array.fromBase64;delete Uint8Array.fromHex;}catch(e){}';
    await page.addInitScript(strip);
    await page.route(/\/pdfjs\/polyfills\.mjs$/, async (route) => {
      const response = await route.fetch();
      await route.fulfill({ response, body: `${strip}\n${await response.text()}` });
    });
    const c = capture(page);
    try {
      await seedPdfAttachment(page, Buffer.from(buildPdf(2), 'latin1'));
      const tab = await openAttachment(page);
      const inner = tab.frameLocator('iframe.mw-viewer__pdf-frame');
      await expect(inner.locator('.page canvas').first()).toBeVisible({ timeout: 30_000 });
      await expect(inner.locator('#numPages')).toContainText('2');
      await expect(tab.getByText('Could not render this PDF.')).toHaveCount(0);
      const inFrame = await page.frames().find((f) => f.url().endsWith('/pdfjs/web/mailwoman.html'))!.evaluate('[typeof Promise.try, typeof Map.prototype.getOrInsertComputed, typeof Math.sumPrecise].join()');
      expect(inFrame).toBe('function,function,function');
    } finally { await save(c, page, info); }
    expectClean(c);
  });

  test('shows the actual reason, with a technical-details section, when the viewer cannot start', async ({ page }, info) => {
    // The library fails to load (a proxy answering it with an error page, a blocked request).
    await page.route(/\/pdfjs\/build\/pdf\.mjs$/, (route) => route.fulfill({ status: 200, contentType: 'text/javascript', headers: { 'access-control-allow-origin': '*', 'cross-origin-resource-policy': 'cross-origin' }, body: 'throw new TypeError("Promise.try is not a function");' }));
    const c = capture(page);
    try {
      await seedPdfAttachment(page, Buffer.from(buildPdf(1), 'latin1'));
      const tab = await openAttachment(page);
      await expect(tab.getByText('Could not render this PDF.')).toBeVisible({ timeout: 30_000 });
      const details = tab.getByTestId('pdf-failure-details');
      await details.locator('summary').click();
      await expect(details).toContainText('library');
      await expect(details).toContainText('Promise.try is not a function');
      await expect(details).toContainText(await page.evaluate(() => navigator.userAgent));
    } finally { await save(c, page, info); }
  });
});
