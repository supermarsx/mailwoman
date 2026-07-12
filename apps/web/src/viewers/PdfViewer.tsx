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

import { getDocument, GlobalWorkerOptions } from 'pdfjs-dist';
import { onCleanup, onMount, Show, createSignal, type JSX } from 'solid-js';
import type { ViewerProps } from '../contracts/viewer.ts';
import { PDF_WORKER_SRC, isSelfHosted } from './pdfWorkerSrc.ts';

export { PDF_WORKER_SRC, isSelfHosted };

// Self-host the worker at module load so every getDocument uses the origin copy.
GlobalWorkerOptions.workerSrc = PDF_WORKER_SRC;

export function PdfViewer(props: ViewerProps): JSX.Element {
  let host: HTMLDivElement | undefined;
  const [status, setStatus] = createSignal<'loading' | 'ready' | 'error'>('loading');
  let destroyed = false;

  onMount(() => {
    void (async () => {
      try {
        const task = getDocument({ url: props.blobUrl });
        const pdf = await task.promise;
        if (destroyed || host === undefined) return;
        for (let n = 1; n <= pdf.numPages; n++) {
          const page = await pdf.getPage(n);
          if (destroyed || host === undefined) return;
          const viewport = page.getViewport({ scale: 1.3 });
          const canvas = document.createElement('canvas');
          canvas.className = 'mw-viewer__pdf-page';
          canvas.width = Math.ceil(viewport.width);
          canvas.height = Math.ceil(viewport.height);
          const cx = canvas.getContext('2d');
          if (cx !== null) {
            host.appendChild(canvas);
            await page.render({ canvas, canvasContext: cx, viewport }).promise;
          }
        }
        if (!destroyed) setStatus('ready');
      } catch {
        if (!destroyed) setStatus('error');
      }
    })();
  });

  onCleanup(() => {
    destroyed = true;
  });

  return (
    <div class="mw-viewer__pdf">
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

export default PdfViewer;
