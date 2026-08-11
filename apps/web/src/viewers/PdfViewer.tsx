// PDF viewer — pdfjs-dist 6.1.200 (Apache-2.0), rendered to <canvas>.
//
// This module statically imports pdfjs (~1 MB). It is ONLY ever reached through
// `lazy(() => import('./PdfViewer.tsx'))` in AttachmentViewer, so the bundler
// code-splits pdfjs into this chunk and it stays OFF the login→inbox critical
// path (plan §1.7, §23 bundle gate). Do not import this module eagerly.
//
// The worker is SELF-HOSTED: `GlobalWorkerOptions.workerSrc` points at an
// origin-served `pdf.worker.mjs` (vendored into `public/`), never a CDN — the
// per-message CSP is `worker-src 'self' / script-src 'self'` (plan §2.4, §7.13).
// pdfjs is a pure parser here: v6 renders without any `eval`/`Function` codepath
// (eval-based rendering was removed upstream) so no PDF-embedded active content
// executes; output is only canvas pixels.
//
// PAGE VIRTUALIZATION (t22 L6). Every page gets a fixed-height SLOT so the
// document's full scroll height and every page offset are known from page 1's
// viewport alone — no page is loaded to find out where it sits. Only the pages
// inside the scroll window (see `pdfWindow.ts`) hold a `<canvas>`; a page that
// leaves has its render cancelled, its backing store zeroed (`width = 0`) and
// its canvas removed. The number of live canvases is therefore bounded by the
// VIEWPORT, not by the page count: a 500-page document allocates the same
// handful of canvases as a 5-page one.

import {
  getDocument,
  GlobalWorkerOptions,
  type PDFDocumentLoadingTask,
  type PDFDocumentProxy,
} from 'pdfjs-dist';
import { onCleanup, onMount, Show, createSignal, type JSX } from 'solid-js';
import type { ViewerProps } from '../contracts/viewer.ts';
import { PDF_WORKER_SRC, isSelfHosted } from './pdfWorkerSrc.ts';
import { maxWindowPages, visiblePageRange } from './pdfWindow.ts';

export { PDF_WORKER_SRC, isSelfHosted };

// Self-host the worker at module load so every getDocument uses the origin copy.
GlobalWorkerOptions.workerSrc = PDF_WORKER_SRC;

/** Render scale, unchanged from the eager implementation. */
const SCALE = 1.3;
/** Pages kept rendered beyond each edge of the visible range. */
const OVERSCAN = 2;
/** Must match `.mw-viewer__pdf-pages { gap }` in viewers.css. */
const SLOT_GAP = 12;

/** Minimal structural view of the pdfjs objects this component uses, so the
 *  windowing logic is expressible without leaking pdfjs types into the signature. */
interface RenderHandle {
  promise: Promise<unknown>;
  cancel: () => void;
}

export function PdfViewer(props: ViewerProps): JSX.Element {
  let scroller: HTMLDivElement | undefined;
  let host: HTMLDivElement | undefined;
  const [status, setStatus] = createSignal<'loading' | 'ready' | 'error'>('loading');
  const [pageCount, setPageCount] = createSignal(0);
  let destroyed = false;

  let pdf: PDFDocumentProxy | null = null;
  /** Held so cleanup can `destroy()` it — that is what aborts in-flight network
   *  requests and tears down the pdfjs worker, releasing the parsed document. */
  let loadingTask: PDFDocumentLoadingTask | null = null;
  /** Per-page slot elements, index 0 = page 1. */
  const slots: HTMLDivElement[] = [];
  /** Pages the current window wants rendered — the intent the async render checks. */
  let claimed = new Set<number>();
  /** Pages that currently own a canvas. Its size is the memory bound. */
  const canvases = new Map<number, HTMLCanvasElement>();
  /** Pages whose render is in flight (cancelled when they leave the window). */
  const tasks = new Map<number, RenderHandle>();
  /** Pages whose async render is between awaits — prevents double-starting. */
  const starting = new Set<number>();
  let slotHeight = 0;

  /** Drop everything page `n` holds: cancel its render, zero and detach its canvas. */
  function release(n: number): void {
    const task = tasks.get(n);
    if (task !== undefined) {
      try {
        task.cancel();
      } catch {
        /* already settled */
      }
      tasks.delete(n);
    }
    const canvas = canvases.get(n);
    if (canvas !== undefined) {
      // Zeroing the dimensions frees the backing store immediately rather than
      // waiting for the element to be collected.
      canvas.width = 0;
      canvas.height = 0;
      canvas.remove();
      canvases.delete(n);
    }
  }

  async function renderPage(n: number): Promise<void> {
    if (destroyed || pdf === null) return;
    if (canvases.has(n) || starting.has(n)) return;
    const slot = slots[n - 1];
    if (slot === undefined) return;
    starting.add(n);
    try {
      const page = await pdf.getPage(n);
      // The window may have moved past this page while it loaded.
      if (destroyed || !claimed.has(n) || canvases.has(n)) return;
      const viewport = page.getViewport({ scale: SCALE });
      const canvas = document.createElement('canvas');
      canvas.className = 'mw-viewer__pdf-page';
      canvas.width = Math.ceil(viewport.width);
      canvas.height = Math.ceil(viewport.height);
      slot.replaceChildren(canvas);
      canvases.set(n, canvas);
      const cx = canvas.getContext('2d');
      if (cx === null) return;
      const task = page.render({ canvas, canvasContext: cx, viewport }) as RenderHandle;
      tasks.set(n, task);
      try {
        await task.promise;
      } catch {
        /* cancelled, or a page that will not render — the slot stays blank */
      }
      tasks.delete(n);
    } catch {
      if (!destroyed) setStatus('error');
    } finally {
      starting.delete(n);
    }
  }

  /** Recompute the window for the current scroll offset and reconcile canvases. */
  function update(): void {
    if (destroyed || pdf === null || slotHeight <= 0) return;
    const numPages = pageCount();
    const range = visiblePageRange({
      scrollTop: scroller?.scrollTop ?? 0,
      viewportHeight: scroller?.clientHeight ?? 0,
      pageHeight: slotHeight,
      gap: SLOT_GAP,
      numPages,
      overscan: OVERSCAN,
    });
    const next = new Set<number>();
    for (let n = range.first; n <= range.last; n++) next.add(n);
    for (const n of [...canvases.keys(), ...tasks.keys()]) {
      if (!next.has(n)) release(n);
    }
    claimed = next;
    for (const n of next) void renderPage(n);
  }

  onMount(() => {
    void (async () => {
      try {
        loadingTask = getDocument({ url: props.blobUrl });
        pdf = await loadingTask.promise;
        if (destroyed || host === undefined) return;
        const numPages: number = pdf.numPages;
        // Page 1 sizes every slot. Mail PDFs are overwhelmingly uniform, and a
        // per-page measurement would mean loading all N pages — the very cost
        // this virtualization exists to avoid.
        const first = await pdf.getPage(1);
        if (destroyed || host === undefined) return;
        const viewport = first.getViewport({ scale: SCALE });
        slotHeight = Math.ceil(viewport.height);
        const slotWidth = Math.ceil(viewport.width);
        for (let n = 1; n <= numPages; n++) {
          const slot = document.createElement('div');
          slot.className = 'mw-viewer__pdf-slot';
          slot.dataset['page'] = String(n);
          slot.style.height = `${slotHeight}px`;
          slot.style.width = `${slotWidth}px`;
          host.appendChild(slot);
          slots.push(slot);
        }
        setPageCount(numPages);
        setStatus('ready');
        update();
      } catch {
        if (!destroyed) setStatus('error');
      }
    })();
  });

  onCleanup(() => {
    destroyed = true;
    for (const n of [...canvases.keys(), ...tasks.keys()]) release(n);
    claimed = new Set();
    slots.length = 0;
    if (loadingTask !== null) {
      try {
        void loadingTask.destroy();
      } catch {
        /* nothing further to release */
      }
      loadingTask = null;
    }
    pdf = null;
  });

  return (
    <div ref={scroller} class="mw-viewer__pdf" onScroll={() => update()}>
      <Show when={status() === 'loading'}>
        <p class="mw-viewer__loading">Rendering PDF…</p>
      </Show>
      <Show when={status() === 'error'}>
        <p class="mw-viewer__error">Could not render this PDF.</p>
      </Show>
      <div ref={host} class="mw-viewer__pdf-pages" aria-label={props.name} />
    </div>
  );
}

/** The canvas ceiling for a viewport — exported so the count assertion and the
 *  component share one definition of "bounded by the visible window". */
export function pdfCanvasBound(viewportHeight: number, pageHeight: number, numPages: number): number {
  return maxWindowPages({
    scrollTop: 0,
    viewportHeight,
    pageHeight,
    gap: SLOT_GAP,
    numPages,
    overscan: OVERSCAN,
  });
}

export default PdfViewer;
