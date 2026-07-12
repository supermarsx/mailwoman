// Self-hosted PDF.js worker path (plan §1.7 / §7.13). Kept in a pdfjs-free module
// so the "origin-served, never a CDN" contract is unit-testable without importing
// pdfjs itself (whose main build needs a real browser — DOMMatrix et al. — and
// won't load under jsdom). `PdfViewer` imports this and registers it on pdfjs's
// `GlobalWorkerOptions` at module load.

/** Origin-served worker path (never a CDN). `BASE_URL` is the app's deploy base. */
export const PDF_WORKER_SRC = `${import.meta.env.BASE_URL}pdf.worker.mjs`;

/** True when `src` is a same-origin (self-hosted) path, not a remote/CDN URL. */
export function isSelfHosted(src: string): boolean {
  if (/^https?:\/\//i.test(src) || src.startsWith('//')) return false;
  return src.startsWith('/') || src.startsWith('./') || !src.includes(':');
}
