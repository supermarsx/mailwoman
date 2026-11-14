import {filePreviews,previewToolEnabled} from './filePreviewPreferences.ts';
// PDF.js's complete, pinned generic viewer is fetched only for a PDF preview.
// The iframe receives authorized attachment bytes, never account credentials or
// an arbitrary PDF URL. Its own CSP and bootstrap disable PDF scripting/network.
//
// The viewer frame is sandboxed with `allow-scripts` only, so it has an opaque
// origin and cannot touch this page, its storage or its cookies (and Firefox no
// longer warns that it "can remove its sandboxing"). The upstream viewer expects
// to run same-origin with its host, so the frame's bootstrap adapts it instead:
// options come from the bootstrap (not `parent.document`), storage is in memory,
// the worker is started through a blob wrapper, and every `/pdfjs/*` file is
// served with CORS + CORP for the opaque origin. The PDF bytes never come from a
// URL the frame can choose: this component fetches them through the
// authenticated attachment route and posts them to the frame, which is addressed
// by window (an opaque origin cannot be a postMessage target, so '*') and
// answers only to the frame window with the origin 'null' and the per-frame nonce.
import { createEffect, createMemo, createSignal, For, onCleanup, onMount, Show, type JSX } from 'solid-js';
import type { ViewerDetails, ViewerProps } from '../contracts/viewer.ts';
import { shellBase } from '../api/basePath.ts';
import { apiFetch } from '../api/http.ts';
import { t } from '../i18n/index.ts';
import {mailFeature} from '../state/mailFeatures.ts';
import {pdfDownload} from './pdfBridge.ts';
import {toast} from '../components/toast/index.ts';
import { ConfirmDialog } from '../screens/Admin/shell/ConfirmDialog.tsx';
import { reportBoundaryError } from '../observability/clientErrors.ts';
import { viewerDetailsFrom } from './attachmentDetails.ts';
import { authorizedPdfBlob, pdfFailureContext, pdfFailureLines, pdfFrameError, pdfHostFailure, PDF_CHANNEL, PDF_ERROR_BOUNDARY, PDF_FRAME_ORIGIN, PDF_FRAME_SANDBOX, safePdfLink, validPdfMessage, type PdfFailure, type PdfStage } from './pdfBridge.ts';

export { PDF_WORKER_SRC, isSelfHosted } from './pdfWorkerSrc.ts';

export function PdfViewer(props: ViewerProps): JSX.Element {
  const document = createMemo(() => ({ url: props.blobUrl, name: props.name }));
  return <Show when={document()} keyed>{attachment => <PdfDocument url={attachment.url} name={attachment.name} onDetails={props.onDetails}/>}</Show>;
}

function PdfDocument(props: { url: string; name: string; onDetails: ((details: ViewerDetails) => void) | undefined }): JSX.Element {
  let frame!: HTMLIFrameElement;
  let ready = false;
  let sent = false;
  let disposed = false;
  let failed = false;
  let passwordAsked = false;
  let instance: string|undefined;
  let reloads = 0;
  let host = window;
  let observer: MutationObserver|undefined;
  let deadline: ReturnType<typeof setTimeout>|undefined;
  let bytes: ArrayBuffer | undefined;
  let abort = new AbortController();
  const downloads=new Map<string,ReturnType<typeof setTimeout>>();
  let nonce = crypto.randomUUID();
  const [status, setStatus] = createSignal<'loading'|'ready'|'error'>('loading');
  const [link, setLink] = createSignal<string>();
  const [failure, setFailure] = createSignal<PdfFailure>();
  const browser = (): string => host.navigator?.userAgent ?? '';
  /** `stage` names the host-side step that failed; a frame error carries its own. */
  function fail(reason?: unknown, stage: PdfStage = 'unknown'): void {
    if (disposed || failed) return;
    failed = true;
    const error = reason ?? new Error('PDF viewer failed');
    const detail = (error as { pdf?: PdfFailure }).pdf ?? pdfHostFailure(stage, error);
    setFailure(detail);
    reportBoundaryError(error, PDF_ERROR_BOUNDARY, pdfFailureContext(detail, browser()));
    abort.abort();
    bytes = undefined;
    setStatus('error');
    if (sent) post('destroy');
  }

  function theme(): Record<string, string|boolean> {
    const root = frame.ownerDocument.documentElement;
    const style = host.getComputedStyle(root);
    const body=host.getComputedStyle(frame.ownerDocument.body);
    const appearance=root.dataset.appearance;
    const result: Record<string, string|boolean> = { dark: appearance==='light'?false:appearance==='dark'?true:style.colorScheme==='dark', 'font-sans': body.fontFamily,'font-size':body.fontSize,direction:body.direction };
    for (const key of ['bg', 'bg-alt', 'surface', 'text', 'text-dim', 'border', 'accent', 'accent-text','danger','warning','success','radius-sm','radius-md']) result[key] = style.getPropertyValue(`--${key}`).trim();
    return result;
  }
  function post(type: string, details: Record<string, unknown> = {}, transfer: Transferable[] = []): void {
    frame.contentWindow?.postMessage({ channel: PDF_CHANNEL, nonce, instance, type, ...details }, '*', transfer);
  }
  function open(): void {
    if (!ready || !bytes || sent || disposed || failed) return;
    sent = true;
    post('open', { bytes, name: props.name, locale: frame.ownerDocument.documentElement.lang || 'en', theme: theme(),downloadAllowed:mailFeature('allowDownload')&&previewToolEnabled('pdf.download')&&previewToolEnabled('download'),printAllowed:mailFeature('allowPrint')&&previewToolEnabled('pdf.print'),tools:filePreviews().tools }, [bytes]);
    bytes = undefined;
  }
  function receive(event: MessageEvent): void {
    if (disposed || !validPdfMessage(event, frame.contentWindow, PDF_FRAME_ORIGIN)) return;
    if (event.data.type === 'ready' && typeof event.data.instance === 'string' && event.data.instance.length <= 100) {
      if (event.data.instance === instance || (failed && instance === undefined)) return;
      if (instance !== undefined) {
        if (++reloads > 5) { fail(new Error('PDF viewer reloaded too many times'), 'handshake'); return; }
        abort.abort(); abort = new AbortController(); nonce = crypto.randomUUID(); sent = false; bytes = undefined; failed = false;
        clearTimeout(deadline); deadline = setTimeout(timedOut, 45_000); setStatus('loading'); setLink(undefined); setFailure(undefined);
        fetchBytes();
      }
      instance = event.data.instance;
      ready = true; open(); return;
    }
    if (failed || event.data.instance !== instance || event.data.nonce !== nonce || !sent) return;
    if(event.data.type==='download'){
      if(!mailFeature('allowDownload')||!previewToolEnabled('pdf.download')||!previewToolEnabled('download')){toast.error(t('reader-attachments-download-off'));return;}
      const file=pdfDownload(event.data);if(!file)return;
      try{
        const url=host.URL.createObjectURL(new Blob([file.bytes],{type:file.mime}));
        const anchor=frame.ownerDocument.createElement('a');anchor.href=url;anchor.download=file.name;frame.ownerDocument.body.append(anchor);anchor.click();anchor.remove();
        downloads.set(url,setTimeout(()=>{host.URL.revokeObjectURL(url);downloads.delete(url);},60_000));
      }catch{toast.error(t('mail-attachment-download-failed'));}
      return;
    }
    if (event.data.type === 'password') passwordAsked = true;
    if (event.data.type === 'loaded' || event.data.type === 'password') { clearTimeout(deadline); setStatus('ready'); }
    if (event.data.type === 'loaded') props.onDetails?.({ ...viewerDetailsFrom(event.data), passwordRequired: passwordAsked });
    else if (event.data.type === 'error') fail(pdfFrameError(event.data));
    else if (event.data.type === 'link') {
      const url = safePdfLink(event.data.url);
      if (url) setLink(url);
    }
  }
  function fetchBytes(): void {
    if (!authorizedPdfBlob(props.url, host.location.origin)) { fail(new Error('PDF attachment URL not authorized'), 'authorize'); return; }
    const request = abort;
    void apiFetch(props.url, { signal: request.signal }).then(async response => {
      if (!response.ok) throw new Error(`PDF unavailable (HTTP ${response.status})`);
      const result = await response.arrayBuffer();
      if (!disposed && !failed && request === abort) { bytes = result; open(); }
    }).catch((error: unknown) => { if (request === abort) fail(error, 'fetch'); });
  }
  /** No frame answer at all (its script was blocked or failed to load) versus an answer that never became a document. */
  function timedOut(): void { fail(new Error(ready ? 'PDF viewer did not finish opening the document' : 'PDF viewer frame did not answer'), ready ? 'timeout' : 'handshake'); }
  function bindHost(): void {
    host.removeEventListener('message', receive);
    host = frame.ownerDocument.defaultView ?? window;
    host.addEventListener('message', receive);
    observer?.disconnect();
    observer = new MutationObserver(() => { if (sent) post('theme', { theme: theme() }); });
    observer.observe(frame.ownerDocument.documentElement, { attributes: true, attributeFilter: ['data-theme', 'data-accent', 'data-appearance', 'style', 'class','dir'] });
    observer.observe(frame.ownerDocument.body,{attributes:true,attributeFilter:['style','class','dir']});
  }
  createEffect(()=>{const allowed=mailFeature('allowDownload')&&previewToolEnabled('pdf.download')&&previewToolEnabled('download'),printAllowed=mailFeature('allowPrint')&&previewToolEnabled('pdf.print'),tools=filePreviews().tools;if(sent&&!disposed)post('permissions',{downloadAllowed:allowed,printAllowed,tools});});
  onMount(() => {
    bindHost();
    deadline = setTimeout(timedOut, 45_000);
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
    for(const [url,timer] of downloads){clearTimeout(timer);host.URL.revokeObjectURL(url);}downloads.clear();
  });
  return <div class="mw-viewer__pdf-full" data-pdf-state={status()}>
    <Show when={status() === 'loading'}><p role="status" class="mw-viewer__loading">{t('common-viewer-rendering-pdf')}</p></Show>
    <Show when={status() === 'error'}><div role="alert" class="mw-viewer__error">
      <p>{t('common-viewer-pdf-failed')}</p>
      <Show when={failure()}>{detail => <details class="mw-viewer__error-details" data-testid="pdf-failure-details"><summary>{t('common-viewer-pdf-details')}</summary>
        <dl>{<For each={pdfFailureLines(detail(), browser())}>{([label, value]) => <><dt>{label}</dt><dd>{value}</dd></>}</For>}</dl>
        <p class="mw-viewer__error-help">{t('common-viewer-pdf-details-help')}</p>
      </details>}</Show>
    </div></Show>
    <iframe ref={frame} class="mw-viewer__pdf-frame" src={`${shellBase()}pdfjs/web/mailwoman.html`} title={t('common-pdf-preview') + ': ' + props.name}
      onLoad={() => { bindHost(); post('probe'); }}
      sandbox={PDF_FRAME_SANDBOX} allowfullscreen referrerPolicy="no-referrer" />
    <ConfirmDialog open={link() !== undefined} title={t('common-pdf-link-title')} body={t('common-pdf-link-help')} danger={false}
      confirmLabel={t('common-pdf-link-open')} cancelLabel={t('common-cancel')} onCancel={() => setLink(undefined)} mount={frame?.ownerDocument.body}
      onConfirm={() => { const url = safePdfLink(link()); if (url) host.open(url, '_blank', 'noopener,noreferrer'); setLink(undefined); }}>
      <p class="mw-viewer__pdf-link">{link()}</p>
    </ConfirmDialog>
  </div>;
}

export default PdfViewer;
