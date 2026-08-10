// Self-hosted PDF.js worker path (plan §1.7 / §7.13). Kept in a pdfjs-free module
// so the "origin-served, never a CDN" contract is unit-testable without importing
// pdfjs itself (whose main build needs a real browser — DOMMatrix et al. — and
// won't load under jsdom). `PdfViewer` imports this and registers it on pdfjs's
// `GlobalWorkerOptions` at module load.

import { shellBase } from '../api/basePath.ts';

/**
 * Origin-served worker path (never a CDN).
 *
 * Sub-path hosting (t20 B4): this is derived from the server-injected deploy
 * prefix, NOT from `import.meta.env.BASE_URL`. Since `vite.config.ts` now sets
 * `base: './'` so the same bundle can be served from any prefix, `BASE_URL` is the
 * literal `'./'` and no longer names the deploy path. `shellBase()` is `/` at the
 * origin root — the same URL this produced before — and `/mail/` under a prefix.
 *
 * A function because the prefix is a runtime value; {@link PDF_WORKER_SRC} below
 * keeps the const form existing importers use.
 */
export function pdfWorkerSrc(): string {
  return `${shellBase()}pdf.worker.mjs`;
}

/** {@link pdfWorkerSrc} resolved at module load. `PdfViewer.tsx` is itself a lazy
 *  chunk, so by the time this module initialises the server-injected
 *  `__MW_BASE__` in `index.html` has long been read. */
export const PDF_WORKER_SRC = pdfWorkerSrc();

/** True when `src` is a same-origin (self-hosted) path, not a remote/CDN URL. */
export function isSelfHosted(src: string): boolean {
  if (/^https?:\/\//i.test(src) || src.startsWith('//')) return false;
  return src.startsWith('/') || src.startsWith('./') || !src.includes(':');
}
