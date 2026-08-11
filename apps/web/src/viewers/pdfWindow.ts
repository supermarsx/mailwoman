// Page windowing arithmetic for the PDF viewer (t22 L6, media memory).
//
// Before this, `PdfViewer` looped `1..numPages` and rendered EVERY page to its
// own `<canvas>` at scale 1.3. A 500-page document allocated 500 canvases, each
// with a backing store of width × height × 4 bytes — hundreds of megabytes that
// the tab never gives back while the viewer is open. The fix is to render only
// the pages near the scroll position and drop the canvases of pages that leave.
//
// The arithmetic lives here, separate from the pdfjs-importing component, for
// two reasons: pdfjs cannot be imported under jsdom (it needs `DOMMatrix`, see
// `pdf-worker.test.ts`), and the bound this module computes is the instrument
// the canvas-count assertion is written against. Every page is laid out in a
// fixed-height slot, so page N's offset is exact arithmetic rather than a
// measurement — no page needs to be loaded to know where it sits.

export interface PdfWindowInput {
  /** Scroll offset of the scrolling container, in px. */
  scrollTop: number;
  /** Visible height of the scrolling container, in px. */
  viewportHeight: number;
  /** Height of one page slot, in px (every slot is the same height). */
  pageHeight: number;
  /** Vertical gap between slots, in px (the flex `gap` in viewers.css). */
  gap: number;
  /** Total pages in the document. */
  numPages: number;
  /** Pages rendered beyond each edge of the visible range. */
  overscan: number;
}

/** 1-based, inclusive page range. `first > last` is never returned. */
export interface PdfPageRange {
  first: number;
  last: number;
}

function rowHeight(i: PdfWindowInput): number {
  return Math.max(1, i.pageHeight + i.gap);
}

/** How many pages the visible viewport can show at once (at least one). */
function visibleRows(i: PdfWindowInput): number {
  if (i.viewportHeight <= 0) return 1;
  return Math.max(1, Math.ceil(i.viewportHeight / rowHeight(i)) + 1);
}

/**
 * The 1-based page range to keep rendered for a scroll position.
 *
 * Clamped to `1..numPages`, so scrolling to either end shrinks the window
 * rather than addressing pages that do not exist.
 */
export function visiblePageRange(i: PdfWindowInput): PdfPageRange {
  if (i.numPages <= 0) return { first: 1, last: 0 };
  const row = rowHeight(i);
  const firstVisible = Math.floor(Math.max(0, i.scrollTop) / row) + 1;
  const first = Math.max(1, firstVisible - i.overscan);
  const last = Math.min(i.numPages, firstVisible + visibleRows(i) - 1 + i.overscan);
  return { first, last: Math.max(first, last) };
}

/**
 * The largest number of pages [`visiblePageRange`] can ever return for these
 * dimensions — the ceiling the rendered-canvas count is asserted against. It is
 * a function of the VIEWPORT, not of `numPages`, except where the document is
 * shorter than the window.
 */
export function maxWindowPages(i: PdfWindowInput): number {
  return Math.min(i.numPages, visibleRows(i) + 2 * i.overscan);
}
