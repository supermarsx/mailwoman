import { test, expect, type Locator } from '@playwright/test';
import { engineLogin, injectViaSmtp, messageRow, waitForInboxMessage } from './helpers.ts';

/**
 * V2 attachment viewers (plan §10.2 / DoD). Seed ONE message carrying an image,
 * a PDF, and a video; open it and assert each renders inside its sandboxed
 * container: image/video in an `<iframe sandbox="">` (no allow-scripts /
 * allow-same-origin), the PDF as a pdfjs `<canvas>` (self-hosted worker) inside
 * the sandboxed modal. The bytes are fetched over the real
 * GET /jmap/download/{accountId}/{blobId}/{name} route (e14).
 */

// A minimal single-page (blank) PDF with a CORRECT xref computed from real byte
// offsets, so pdfjs parses it cleanly and renders a canvas. All bytes are ASCII
// (offset == char index), and the attachment is base64-transported verbatim.
function buildMinimalPdf(): string {
  const bodies = [
    '<< /Type /Catalog /Pages 2 0 R >>',
    '<< /Type /Pages /Kids [3 0 R] /Count 1 >>',
    '<< /Type /Page /Parent 2 0 R /MediaBox [0 0 300 144] >>',
  ];
  let pdf = '%PDF-1.4\n';
  const offsets: number[] = [];
  bodies.forEach((body, i) => {
    offsets.push(pdf.length);
    pdf += `${i + 1} 0 obj\n${body}\nendobj\n`;
  });
  const xrefStart = pdf.length;
  pdf += `xref\n0 ${bodies.length + 1}\n0000000000 65535 f \n`;
  for (const off of offsets) pdf += `${String(off).padStart(10, '0')} 00000 n \n`;
  pdf += `trailer\n<< /Root 1 0 R /Size ${bodies.length + 1} >>\nstartxref\n${xrefStart}\n%%EOF`;
  return pdf;
}
const MINIMAL_PDF = buildMinimalPdf();

// A 1x1 transparent PNG (real image bytes so the image frame has something valid).
const PNG_1x1_B64 =
  'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==';

test.describe.configure({ mode: 'serial', retries: 2 });

test.describe('V2 attachment viewers (engine mode)', () => {
  test('image + PDF + video each render inside a sandboxed container', async ({ page }) => {
    test.slow();
    await engineLogin(page);

    const subject = `Viewers ${Date.now()}`;
    await injectViaSmtp({
      from: 'Files Bot <files@example.org>',
      subject,
      text: 'three attachments to preview',
      attachments: [
        { filename: 'photo.png', contentType: 'image/png', base64: PNG_1x1_B64 },
        { filename: 'doc.pdf', contentType: 'application/pdf', content: MINIMAL_PDF },
        { filename: 'clip.mp4', contentType: 'video/mp4', content: '\x00\x00\x00\x18ftypmp42fake-video-bytes' },
      ],
    });
    await waitForInboxMessage(page, subject, 150_000);

    // Open the message -> the Reader shows the attachment strip with all three.
    await messageRow(page, subject).first().click();
    await expect(page.getByTestId('reader-attachments')).toBeVisible();
    for (const name of ['photo.png', 'doc.pdf', 'clip.mp4']) {
      await expect(page.getByRole('option', { name })).toBeVisible();
    }

    const modal = page.getByTestId('attachment-viewer');
    const openViewer = async (name: string): Promise<void> => {
      await page.getByRole('option', { name }).click();
      await expect(modal).toBeVisible();
    };
    const closeViewer = async (): Promise<void> => {
      await page.getByRole('button', { name: 'Close attachment' }).click();
      await expect(modal).toBeHidden();
    };
    const assertSandboxed = async (frame: Locator): Promise<void> => {
      await expect(frame).toBeVisible();
      const sandbox = await frame.getAttribute('sandbox');
      expect(sandbox).not.toBeNull();
      expect(sandbox).not.toContain('allow-scripts');
      expect(sandbox).not.toContain('allow-same-origin');
    };

    // PDF: pdfjs renders a real canvas page (self-hosted worker) — not the error.
    await openViewer('doc.pdf');
    await expect(modal.locator('.mw-viewer[data-viewer-kind="pdf"]')).toBeVisible();
    await expect(modal.locator('.mw-viewer__pdf-page')).toBeVisible({ timeout: 20_000 });
    await expect(modal.getByText('Could not render this PDF.')).toHaveCount(0);
    await closeViewer();

    // Image: sandboxed iframe.
    await openViewer('photo.png');
    await assertSandboxed(modal.locator('iframe.mw-viewer__frame--image'));
    await closeViewer();

    // Video: sandboxed iframe.
    await openViewer('clip.mp4');
    await assertSandboxed(modal.locator('iframe.mw-viewer__frame--video'));
    await closeViewer();
  });
});
