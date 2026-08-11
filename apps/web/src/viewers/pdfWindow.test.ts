// t22 L6 — the PDF page-window arithmetic (`pdfWindow.ts`).
//
// The component test (`pdfVirtual.test.tsx`) counts canvases; this file pins the
// arithmetic those counts rest on, including the two edges a windowing bug hides
// in: the top of a document (where the range must not address page 0) and the
// bottom (where it must not address page numPages + 1).

import { describe, it, expect } from 'vitest';
import { maxWindowPages, visiblePageRange, type PdfWindowInput } from './pdfWindow.ts';

/** A 500-page document at ~1000 px per page in an 800 px viewport. */
function doc(over: Partial<PdfWindowInput> = {}): PdfWindowInput {
  return {
    scrollTop: 0,
    viewportHeight: 800,
    pageHeight: 1000,
    gap: 12,
    numPages: 500,
    overscan: 2,
    ...over,
  };
}

describe('visiblePageRange', () => {
  it('at the top, clamps to page 1 rather than addressing page 0', () => {
    expect(visiblePageRange(doc())).toEqual({ first: 1, last: 4 });
  });

  it('at the bottom, clamps to the last page', () => {
    const i = doc({ scrollTop: 499 * 1012 });
    const r = visiblePageRange(i);
    expect(r.last).toBe(500);
    expect(r.first).toBe(498);
  });

  it('moves with the scroll position instead of growing', () => {
    // Both positions are mid-document, so neither window is clipped by the
    // page-1 clamp and the two spans are directly comparable.
    const a = visiblePageRange(doc({ scrollTop: 50 * 1012 }));
    const b = visiblePageRange(doc({ scrollTop: 100 * 1012 }));
    expect(b.first).toBeGreaterThan(a.last);
    expect(b.last - b.first).toBe(a.last - a.first);
  });

  it('a taller viewport widens the window; the page count does not', () => {
    const short = visiblePageRange(doc({ viewportHeight: 800 }));
    const tall = visiblePageRange(doc({ viewportHeight: 4000 }));
    expect(tall.last - tall.first).toBeGreaterThan(short.last - short.first);
    const more = visiblePageRange(doc({ numPages: 200_000, scrollTop: 50 * 1012 }));
    const fewer = visiblePageRange(doc({ numPages: 500, scrollTop: 50 * 1012 }));
    expect(more.last - more.first).toBe(fewer.last - fewer.first);
  });

  it('an empty document yields an empty range', () => {
    expect(visiblePageRange(doc({ numPages: 0 }))).toEqual({ first: 1, last: 0 });
  });
});

describe('maxWindowPages — the ceiling the canvas count is asserted against', () => {
  it('is a function of the viewport, not the document length', () => {
    expect(maxWindowPages(doc({ numPages: 500 }))).toBe(maxWindowPages(doc({ numPages: 200_000 })));
  });

  it('never exceeds a document shorter than the window', () => {
    expect(maxWindowPages(doc({ numPages: 3 }))).toBe(3);
  });

  it('bounds every reachable scroll position of a 500-page document', () => {
    const i = doc();
    const bound = maxWindowPages(i);
    for (let top = 0; top <= 500 * 1012; top += 977) {
      const r = visiblePageRange({ ...i, scrollTop: top });
      expect(r.last - r.first + 1).toBeLessThanOrEqual(bound);
    }
  });
});
