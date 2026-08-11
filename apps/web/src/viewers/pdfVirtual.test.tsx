// t22 L6 — the PDF viewer allocates a BOUNDED number of canvases.
//
// The document under test has 500 pages and the viewport fits four, so a
// document that "fits entirely in the window" cannot make this assertion pass by
// accident — that is the failure shape this file is written against. The other
// one is a count taken only at load: the window is scrolled to the middle and to
// the end, and the count is re-asserted each time, so a viewer that renders on
// demand and never releases is caught (it would climb, not hold).
//
// On master the same fixture allocates one canvas per page: `PdfViewer` looped
// `for (let n = 1; n <= pdf.numPages; n++)` and appended each canvas to the host.
// The master-failing value for this fixture is 500.
//
// pdfjs itself cannot be imported under jsdom (it needs `DOMMatrix` — see
// `pdf-worker.test.ts`), so the module is mocked. What is NOT mocked is the
// component: the slot building, the window reconciliation and the canvas
// lifecycle are the shipping code.

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, waitFor } from '@solidjs/testing-library';
import type { EmailBodyPart } from '../api/jmap-types.ts';

const fixture = vi.hoisted(() => ({
  numPages: 500,
  pageWidth: 800,
  pageHeight: 1000,
  /** Pages whose data pdfjs was asked for — proves the viewer does not load all. */
  loaded: new Set<number>(),
  cancelled: [] as number[],
  destroyed: 0,
}));

vi.mock('pdfjs-dist', () => {
  const makePage = (n: number): unknown => ({
    getViewport: () => ({ width: fixture.pageWidth, height: fixture.pageHeight }),
    render: () => ({
      promise: Promise.resolve(),
      cancel: () => fixture.cancelled.push(n),
    }),
  });
  return {
    GlobalWorkerOptions: { workerSrc: '' },
    getDocument: () => ({
      promise: Promise.resolve({
        numPages: fixture.numPages,
        getPage: (n: number) => {
          fixture.loaded.add(n);
          return Promise.resolve(makePage(n));
        },
        cleanup: () => true,
      }),
      // The loading task is what owns the worker; destroying it is the real
      // teardown, so that is what the component holds and what is counted here.
      destroy: () => {
        fixture.destroyed += 1;
        return Promise.resolve();
      },
    }),
  };
});

const { PdfViewer, pdfCanvasBound } = await import('./PdfViewer.tsx');

const part: EmailBodyPart = { partId: '1', blobId: 'b', size: 1, type: 'application/pdf' };

/** jsdom has no layout: give the scroller a real viewport height and a
 *  settable scroll offset so the window arithmetic has something to work with. */
function drive(container: HTMLElement, viewportHeight: number): (top: number) => void {
  const scroller = container.querySelector('.mw-viewer__pdf');
  if (scroller === null) throw new Error('no scroller');
  Object.defineProperty(scroller, 'clientHeight', { value: viewportHeight, configurable: true });
  let top = 0;
  Object.defineProperty(scroller, 'scrollTop', {
    get: () => top,
    set: (v: number) => {
      top = v;
    },
    configurable: true,
  });
  return (to: number) => {
    (scroller as HTMLElement & { scrollTop: number }).scrollTop = to;
    scroller.dispatchEvent(new Event('scroll'));
  };
}

beforeEach(() => {
  // jsdom throws "not implemented" from getContext; the component already treats
  // a null context as "canvas allocated, nothing painted", which is the state the
  // count assertions measure. Stubbing it keeps the run's output honest-looking.
  vi.spyOn(HTMLCanvasElement.prototype, 'getContext').mockReturnValue(null);
  fixture.numPages = 500;
  fixture.loaded.clear();
  fixture.cancelled = [];
  fixture.destroyed = 0;
});

afterEach(() => {
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

describe('PdfViewer page virtualization (t22 L6)', () => {
  it('renders 500 slots but a canvas count bounded by the viewport', async () => {
    const { container, unmount } = render(() => (
      <PdfViewer part={part} blobUrl="blob:pdf" mime="application/pdf" name="big.pdf" />
    ));

    await waitFor(() => expect(container.querySelectorAll('.mw-viewer__pdf-slot').length).toBe(500));

    // 4 pages fit in 4 000 px of viewport at 1 012 px per row.
    const scrollTo = drive(container, 4000);
    scrollTo(0);

    const bound = pdfCanvasBound(4000, fixture.pageHeight, 500);
    expect(bound).toBeLessThan(20);
    expect(bound).toBeLessThan(fixture.numPages);

    await waitFor(() => expect(container.querySelectorAll('canvas').length).toBeGreaterThan(0));
    expect(container.querySelectorAll('canvas').length).toBeLessThanOrEqual(bound);

    // Far into the document — the count must HOLD, not climb.
    scrollTo(250 * 1012);
    await waitFor(() =>
      expect(container.querySelector('.mw-viewer__pdf-slot[data-page="251"] canvas')).not.toBeNull(),
    );
    expect(container.querySelectorAll('canvas').length).toBeLessThanOrEqual(bound);
    // and the pages left behind gave their canvases back
    expect(container.querySelector('.mw-viewer__pdf-slot[data-page="1"] canvas')).toBeNull();

    // The end of the document.
    scrollTo(499 * 1012);
    await waitFor(() =>
      expect(container.querySelector('.mw-viewer__pdf-slot[data-page="500"] canvas')).not.toBeNull(),
    );
    expect(container.querySelectorAll('canvas').length).toBeLessThanOrEqual(bound);

    // Page DATA is fetched on demand too: nothing like 500 pages was parsed.
    expect(fixture.loaded.size).toBeLessThan(50);

    unmount();
    expect(container.querySelectorAll('canvas').length).toBe(0);
    expect(fixture.destroyed).toBe(1);
  });

  it('a document that fits entirely in the window still renders every page', async () => {
    // The control for the assertion above: with 3 pages the bound IS the page
    // count, so a "bounded" result here proves nothing — which is exactly why
    // the real assertion uses 500.
    fixture.numPages = 3;
    const { container } = render(() => (
      <PdfViewer part={part} blobUrl="blob:pdf" mime="application/pdf" name="small.pdf" />
    ));
    await waitFor(() => expect(container.querySelectorAll('.mw-viewer__pdf-slot').length).toBe(3));
    const scrollTo = drive(container, 4000);
    scrollTo(0);
    await waitFor(() => expect(container.querySelectorAll('canvas').length).toBe(3));
    expect(pdfCanvasBound(4000, fixture.pageHeight, 3)).toBe(3);
  });

  it('slots reserve each page height before anything renders (no layout shift)', async () => {
    const { container } = render(() => (
      <PdfViewer part={part} blobUrl="blob:pdf" mime="application/pdf" name="big.pdf" />
    ));
    await waitFor(() => expect(container.querySelectorAll('.mw-viewer__pdf-slot').length).toBe(500));
    const slot = container.querySelector<HTMLElement>('.mw-viewer__pdf-slot[data-page="400"]');
    expect(slot?.style.height).toBe(`${fixture.pageHeight}px`);
    expect(slot?.style.width).toBe(`${fixture.pageWidth}px`);
    expect(slot?.querySelector('canvas')).toBeNull();
  });
});
