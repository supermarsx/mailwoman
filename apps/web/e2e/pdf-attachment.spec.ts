import { test, expect, type ConsoleMessage, type Page, type Response } from '@playwright/test';
import { writeFileSync } from 'node:fs';
import { login, messageRow } from './helpers.ts';

/**
 * A real PDF attachment opens end to end in the PDF.js viewer: the first page
 * canvas renders, no "Could not render this PDF." appears, every /pdfjs/* (and
 * the worker) request succeeds with the right type, and the browser raises no
 * sandbox warning or CSP violation.
 *
 * Runs against the mock-JMAP proxy stack (the `mock` project). The mock has no
 * attachments, so the browser-side JMAP reply for one message is extended with a
 * PDF part and its download is answered with real PDF bytes; everything else
 * (the mw-server, the embedded viewer assets, the headers, the sandboxed frame)
 * is the real thing.
 */

/** A valid multi-page PDF with a correct xref computed from real byte offsets. */
export function buildPdf(pages: number): string {
  const objs: string[] = [];
  const kids = Array.from({ length: pages }, (_, i) => `${4 + i * 2} 0 R`).join(' ');
  objs.push('<< /Type /Catalog /Pages 2 0 R >>');
  objs.push(`<< /Type /Pages /Kids [${kids}] /Count ${pages} >>`);
  objs.push('<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>');
  for (let i = 0; i < pages; i += 1) {
    const text = `BT /F1 24 Tf 72 700 Td (Mailwoman PDF page ${i + 1} of ${pages}) Tj ET`;
    objs.push(`<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 3 0 R >> >> /Contents ${5 + i * 2} 0 R >>`);
    objs.push(`<< /Length ${text.length} >>\nstream\n${text}\nendstream`);
  }
  let pdf = '%PDF-1.4\n';
  const offsets: number[] = [];
  objs.forEach((body, i) => { offsets.push(pdf.length); pdf += `${i + 1} 0 obj\n${body}\nendobj\n`; });
  const xref = pdf.length;
  pdf += `xref\n0 ${objs.length + 1}\n0000000000 65535 f \n`;
  for (const off of offsets) pdf += `${String(off).padStart(10, '0')} 00000 n \n`;
  return `${pdf}trailer\n<< /Root 1 0 R /Size ${objs.length + 1} >>\nstartxref\n${xref}\n%%EOF`;
}

const PDF = Buffer.from(buildPdf(3), 'latin1');
const BLOB_ID = 'blob-pdf-attachment';

async function seedPdfAttachment(page: Page): Promise<void> {
  await page.route(/\/jmap\/download\/[^/]+\/blob-pdf-attachment\//, (route) =>
    route.fulfill({ status: 200, contentType: 'application/pdf', body: PDF }));
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
          email['attachments'] = [{ partId: '2', blobId: BLOB_ID, type: 'application/pdf', name: 'report.pdf', size: PDF.length, disposition: 'attachment' }];
        }
      }
      body = JSON.stringify(json);
    } catch { /* not JSON */ }
    return route.fulfill({ response, body });
  });
}

test.describe('PDF attachment', () => {
  test.setTimeout(90_000);
  test('opens, renders page 1, and the browser reports no sandbox warning or CSP violation', async ({ page }, info) => {
    const console_: string[] = [];
    const failures: string[] = [];
    const pdfjs: { url: string; status: number; type: string }[] = [];
    page.on('console', (m: ConsoleMessage) => console_.push(`${m.type()} [${m.location().url.replace(/^https?:..[^/]*/, '')}:${m.location().lineNumber}]: ${m.text()}`));
    page.on('pageerror', (e) => console_.push(`pageerror: ${e.message}`));
    page.on('requestfailed', (r) => /\/pdfjs\/|pdf\.worker/.test(r.url()) && !/ABORTED/.test(r.failure()?.errorText ?? '') && failures.push(`${r.url()} ${r.failure()?.errorText}`));
    page.on('response', (r: Response) => {
      const u = new URL(r.url());
      if (u.pathname.startsWith('/pdfjs/') || u.pathname === '/pdf.worker.mjs') {
        pdfjs.push({ url: u.pathname, status: r.status(), type: r.headers()['content-type'] ?? '' });
      }
    });
    try {
      await seedPdfAttachment(page);
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

      const frame = tab.locator('iframe.mw-viewer__pdf-frame');
      // Least privilege: scripts only, never the app's origin.
      await expect(frame).toHaveAttribute('sandbox', /allow-scripts/);
      expect(await frame.getAttribute('sandbox')).not.toContain('allow-same-origin');

      const inner = tab.frameLocator('iframe.mw-viewer__pdf-frame');
      await expect(inner.locator('.page canvas').first()).toBeVisible({ timeout: 30_000 });
      await expect(inner.locator('#numPages')).toContainText('3');
      await expect(tab.getByText('Could not render this PDF.')).toHaveCount(0);
    } finally {
      writeFileSync(info.outputPath('console.txt'), console_.join(String.fromCharCode(10)));
      writeFileSync(info.outputPath('pdfjs-requests.json'), JSON.stringify({ pdfjs, failures }, null, 1));
      await page.screenshot({ path: info.outputPath('pdf-viewer.png') });
    }
    for (const r of pdfjs) {
      expect(r.status, r.url).toBeLessThan(400);
      if (!r.url.endsWith('/locale.json')) expect(r.type, r.url).not.toContain('json');
    }
    expect(failures).toEqual([]);
    // The browser's own warning about a frame that could remove its sandbox, anywhere
    // on the page; and any policy/CORS/script failure raised by the viewer frame or its worker.
    const sandboxWarning = console_.filter((l) => /can remove its sandboxing|allow-scripts and allow-same-origin/i.test(l));
    expect(sandboxWarning).toEqual([]);
    const bad = console_.filter((l) => /\[\/(pdfjs\/|pdf\.worker)/.test(l) && /Content[- ]Security[- ]Policy|violates|Refused to|blocked by CORS|has been blocked|SecurityError|Failed to load resource|Failed to fetch|Untrusted/i.test(l));
    expect(bad).toEqual([]);
  });
});
