// PDF.js's complete, pinned generic viewer is fetched only for a PDF preview.
// The iframe receives authorized attachment bytes, never account credentials or
// an arbitrary PDF URL. Its own CSP and bootstrap disable PDF scripting/network.
//
// Why the frame keeps `allow-scripts allow-same-origin` (Firefox warns that such a
// frame "can remove its sandboxing"): the pinned upstream viewer is built to run
// same-origin with its host. The bootstrap hands options to the viewer through
// `window.parent.document` (the `webviewerloaded` event), the viewer uses
// localStorage, creates its module Worker from a same-origin URL, and fetches
// cmaps/fonts/wasm with credentialed same-origin requests; an opaque origin breaks
// each of these (cross-origin module/worker/CORS, `parent.document` throws). The
// frame only ever loads our fixed, embedded, script-free-of-inline document under
// a tight CSP (script-src 'self', Trusted Types, no PDF scripting, connect-src
// 'self' blob:), so the "remove its sandbox" path needs a code-execution bug in
// pinned PDF.js first. Dropping the token needs the viewer reworked and verified
// in real browsers; until then this is the documented, accepted residual.
import { createMemo, createSignal, onCleanup, onMount, Show, type JSX } from 'solid-js';
import type { ViewerProps } from '../contracts/viewer.ts';
import { shellBase } from '../api/basePath.ts';
import { apiFetch } from '../api/http.ts';
import { t } from '../i18n/index.ts';
import { ConfirmDialog } from '../screens/Admin/shell/ConfirmDialog.tsx';
import { authorizedPdfBlob, PDF_CHANNEL, safePdfLink, validPdfMessage } from './pdfBridge.ts';

export { PDF_WORKER_SRC, isSelfHosted } from './pdfWorkerSrc.ts';

export function PdfViewer(props: ViewerProps): JSX.Element {
  const document = createMemo(() => ({ url: props.blobUrl, name: props.name }));
  return <Show when={document()} keyed>{attachment => <PdfDocument url={attachment.url} name={attachment.name}/>}</Show>;
}

function PdfDocument(props: { url: string; name: string }): JSX.Element {
  let frame!: HTMLIFrameElement;
  let ready = false;
  let sent = false;
  let disposed = false;
  let failed = false;
  let instance: string|undefined;
  let reloads = 0;
  let host = window;
  let observer: MutationObserver|undefined;
  let deadline: ReturnType<typeof setTimeout>|undefined;
  let bytes: ArrayBuffer | undefined;
  let abort = new AbortController();
  let nonce = crypto.randomUUID();
  const [status, setStatus] = createSignal<'loading'|'ready'|'error'>('loading');
  const [link, setLink] = createSignal<string>();
  function fail(): void {
    if (disposed || failed) return;
    failed = true;
    abort.abort();
    bytes = undefined;
    setStatus('error');
    if (sent) post('destroy');
  }

  function theme(): Record<string, string|boolean> {
    const root = frame.ownerDocument.documentElement;
    const style = host.getComputedStyle(root);
    const result: Record<string, string|boolean> = { dark: root.dataset.appearance === 'dark', 'font-sans': style.fontFamily };
    for (const key of ['bg', 'bg-alt', 'surface', 'text', 'text-dim', 'border', 'accent', 'accent-text']) result[key] = style.getPropertyValue(`--${key}`).trim();
    return result;
  }
  function post(type: string, details: Record<string, unknown> = {}, transfer: Transferable[] = []): void {
    frame.contentWindow?.postMessage({ channel: PDF_CHANNEL, nonce, instance, type, ...details }, host.location.origin, transfer);
  }
  function open(): void {
    if (!ready || !bytes || sent || disposed || failed) return;
    sent = true;
    post('open', { bytes, name: props.name, locale: frame.ownerDocument.documentElement.lang || 'en', theme: theme() }, [bytes]);
    bytes = undefined;
  }
  function receive(event: MessageEvent): void {
    if (disposed || !validPdfMessage(event, frame.contentWindow, host.location.origin)) return;
    if (event.data.type === 'ready' && typeof event.data.instance === 'string' && event.data.instance.length <= 100) {
      if (event.data.instance === instance || (failed && instance === undefined)) return;
      if (instance !== undefined) {
        if (++reloads > 5) { fail(); return; }
        abort.abort(); abort = new AbortController(); nonce = crypto.randomUUID(); sent = false; bytes = undefined; failed = false;
        clearTimeout(deadline); deadline = setTimeout(fail, 45_000); setStatus('loading'); setLink(undefined);
        fetchBytes();
      }
      instance = event.data.instance;
      ready = true; open(); return;
    }
    if (failed || event.data.instance !== instance || event.data.nonce !== nonce || !sent) return;
    if (event.data.type === 'loaded' || event.data.type === 'password') { clearTimeout(deadline); setStatus('ready'); }
    else if (event.data.type === 'error') fail();
    else if (event.data.type === 'link') {
      const url = safePdfLink(event.data.url);
      if (url) setLink(url);
    }
  }
  function fetchBytes(): void {
    if (!authorizedPdfBlob(props.url, host.location.origin)) { fail(); return; }
    const request = abort;
    void apiFetch(props.url, { signal: request.signal }).then(async response => {
      if (!response.ok) throw new Error('PDF unavailable');
      const result = await response.arrayBuffer();
      if (!disposed && !failed && request === abort) { bytes = result; open(); }
    }).catch(() => { if (request === abort) fail(); });
  }
  function bindHost(): void {
    host.removeEventListener('message', receive);
    host = frame.ownerDocument.defaultView ?? window;
    host.addEventListener('message', receive);
    observer?.disconnect();
    observer = new MutationObserver(() => { if (sent) post('theme', { theme: theme() }); });
    observer.observe(frame.ownerDocument.documentElement, { attributes: true, attributeFilter: ['data-theme', 'data-accent', 'data-appearance', 'style', 'class'] });
  }
  onMount(() => {
    bindHost();
    deadline = setTimeout(fail, 45_000);
    fetchBytes();
  });
  onCleanup(() => {
    disposed = true;
    abort.abort();
    clearTimeout(deadline);
    if (sent) post('destroy');
    host.removeEventListener('message', receive);
    observer?.disconnect();
    bytes = undefined;
  });
  return <div class="mw-viewer__pdf-full" data-pdf-state={status()}>
    <Show when={status() === 'loading'}><p role="status" class="mw-viewer__loading">{t('common-viewer-rendering-pdf')}</p></Show>
    <Show when={status() === 'error'}><p role="alert" class="mw-viewer__error">{t('common-viewer-pdf-failed')}</p></Show>
    <iframe ref={frame} class="mw-viewer__pdf-frame" src={`${shellBase()}pdfjs/web/mailwoman.html`} title={t('common-pdf-preview') + ': ' + props.name}
      onLoad={() => { bindHost(); post('probe'); }}
      sandbox="allow-scripts allow-same-origin allow-downloads allow-modals" allowfullscreen referrerPolicy="no-referrer" />
    <ConfirmDialog open={link() !== undefined} title={t('common-pdf-link-title')} body={t('common-pdf-link-help')} danger={false}
      confirmLabel={t('common-pdf-link-open')} cancelLabel={t('common-cancel')} onCancel={() => setLink(undefined)} mount={frame?.ownerDocument.body}
      onConfirm={() => { const url = safePdfLink(link()); if (url) host.open(url, '_blank', 'noopener,noreferrer'); setLink(undefined); }}>
      <p class="mw-viewer__pdf-link">{link()}</p>
    </ConfirmDialog>
  </div>;
}

export default PdfViewer;
