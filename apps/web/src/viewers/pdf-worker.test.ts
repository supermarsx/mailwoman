import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { PDF_WORKER_SRC, isSelfHosted } from './pdfWorkerSrc.ts';

// pdfjs' main build can't be imported under jsdom (needs DOMMatrix), so the
// self-hosted-worker contract lives in a pdfjs-free module we CAN load, plus a
// source-level check that PdfViewer registers exactly that path on pdfjs.

describe('PDF.js worker is self-hosted (plan §1.7 / §7.13)', () => {
  it('points workerSrc at an origin-served path, never a CDN', () => {
    expect(PDF_WORKER_SRC.endsWith('pdf.worker.mjs')).toBe(true);
    expect(isSelfHosted(PDF_WORKER_SRC)).toBe(true);
  });

  it('rejects remote / CDN worker URLs', () => {
    expect(isSelfHosted('https://cdn.jsdelivr.net/npm/pdfjs-dist/pdf.worker.mjs')).toBe(false);
    expect(isSelfHosted('http://evil.example/pdf.worker.mjs')).toBe(false);
    expect(isSelfHosted('//cdn.example/pdf.worker.mjs')).toBe(false);
    expect(isSelfHosted('/pdf.worker.mjs')).toBe(true);
    expect(isSelfHosted('./pdf.worker.mjs')).toBe(true);
    expect(isSelfHosted('pdf.worker.mjs')).toBe(true);
  });

  it('PdfViewer registers the self-hosted worker on pdfjs at module load', () => {
    // cwd is apps/web under vitest; read the source directly.
    const src = readFileSync('src/viewers/PdfViewer.tsx', 'utf8');
    expect(src).toMatch(/GlobalWorkerOptions\.workerSrc\s*=\s*PDF_WORKER_SRC/);
    // and never a hardcoded CDN
    expect(src).not.toMatch(/https?:\/\/[^'"]*pdf\.worker/);
  });
});
