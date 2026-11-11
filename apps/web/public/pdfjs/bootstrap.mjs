/* global location, window, document, history, URL, matchMedia, crypto */
// Mailwoman bridge for the unmodified, pinned Mozilla PDF.js generic viewer.
// PDF-embedded code and external resource fetching are never enabled.
import { installed as polyfilled } from './polyfills.mjs';
const CHANNEL = 'mailwoman-pdf-v1';
// The frame is sandboxed WITHOUT allow-same-origin, so its own origin is opaque
// ('null') and it cannot read the embedding page or its storage. It can neither
// name nor read the host's origin up front: the host is `window.parent`, and the
// origin of the first message that window delivers is pinned (see `receive`).
const host = window.parent;
let hostOrigin = null;
const instance = crypto.randomUUID();
let nonce = null;
let opened = false;
let application = null;
let configuration = null;
const worker = new URL('../pdf.worker.mjs', import.meta.url).href;
const polyfills = new URL('./polyfills.mjs', import.meta.url).href;
// Where the open stopped, for the error detail the host shows and reports.
let stage = 'script';
const workerStatus = { mode: 'pending', reason: '' };
const base = new URL('./web/', import.meta.url).href;
// A data: module worker has an opaque origin like this document and may import the
// real worker over CORS (a blob: worker is refused outright from an opaque origin,
// and so is a direct http(s) worker URL). The wrapper is the only script URL ever
// handed to the Worker constructor.
// The same polyfills the frame uses come first: the worker is a separate realm.
const workerWrapper = `data:text/javascript,${encodeURIComponent(`await import(${JSON.stringify(polyfills)});await import(${JSON.stringify(worker)});`)}`;
let workerPort = null;

// An opaque-origin document has no localStorage (the getter throws). The viewer
// reads it for signatures and history, both disabled here; give it a private
// in-memory store so nothing throws and nothing is ever persisted.
function memoryStorage() {
  const map = new Map();
  return {
    get length() { return map.size; },
    key: index => [...map.keys()][index] ?? null,
    getItem: key => map.get(String(key)) ?? null,
    setItem: (key, value) => { map.set(String(key), String(value)); },
    removeItem: key => { map.delete(String(key)); },
    clear: () => map.clear(),
  };
}
try { Object.defineProperty(window, 'localStorage', { value: memoryStorage(), configurable: true }); } catch { /* keep the native one */ }
// The viewer announces `webviewerloaded` on `parent.document` so a same-origin
// host can set options. That read throws here (and logs an error), so make the
// frame its own announcement target; options are set from this file instead.
try { Object.defineProperty(window, 'parent', { value: { document }, configurable: true }); } catch { /* falls back to document.dispatchEvent */ }

// Only the fixed self-hosted worker and its data: wrapper can enter a script-URL
// sink. HTML is used
// by upstream Fluent's local-translation overlay, which sanitizes its DOM.
globalThis.trustedTypes?.createPolicy('default', {
  createHTML: value => value,
  createScriptURL: value => {
    if (value === workerWrapper || new URL(value, location.href).href === worker) return value;
    throw new TypeError('Untrusted PDF worker');
    return value;
  },
});

// The generic viewer normally accepts file= and hash/debug preferences. This
// embedded viewer accepts bytes from its owning app only, never a URL parameter.
history.replaceState(null, '', location.pathname);
// Messages go to the host window only. Until the host has spoken first the origin
// is unknown, and the only thing sent then is this frame's random instance id.
function send(type, detail = {}, transfer = []) {
  if (host !== window) host.postMessage({ channel: CHANNEL, type, nonce, instance, ...detail }, hostOrigin ?? '*', transfer);
}
function externalLink(url) {
  try {
    const parsed = new URL(url);
    if (!['https:', 'http:', 'mailto:', 'tel:'].includes(parsed.protocol) || parsed.username || parsed.password || [...url].some(char => char.charCodeAt(0) < 32 || char.charCodeAt(0) === 127)) return;
    send('link', { url: parsed.href });
  } catch { /* Invalid or active URL schemes are inert. */ }
}
function applyTheme(theme) {
  if (!theme || typeof theme !== 'object') return;
  const keys = ['bg', 'bg-alt', 'surface', 'text', 'text-dim', 'border', 'accent', 'accent-text', 'font-sans','font-size','danger','warning','success','radius-sm','radius-md'];
  for (const key of keys) {
    const value = theme[key];
    if (typeof value === 'string' && value.length <= 300 && !/[;{}<>]|url\s*\(/i.test(value)) document.documentElement.style.setProperty(`--mw-${key}`, value);
  }
  document.documentElement.style.colorScheme = theme.dark ? 'dark' : 'light';
  document.documentElement.dir=theme.direction==='rtl'?'rtl':'ltr';
}
function applyDownloadPermission() {
  for(const id of ['downloadButton','secondaryDownload']){const button=document.getElementById(id);if(button)button.disabled=configuration?.downloadAllowed===false;}
}

// PDF.js cannot start its own worker from an opaque origin (it judges "same
// origin" by URL, builds a direct http(s) worker, and silently drops to parsing on
// the main thread). Start it here and hand the port over; when it does not come up
// (blocked request, no module-worker support) PDF.js's main-thread fallback runs.
function startWorker() {
  return new Promise(resolve => {
    let worker;
    try { worker = new Worker(workerWrapper, { type: 'module' }); } catch (error) { workerStatus.mode = 'main-thread'; workerStatus.reason = `constructor: ${error?.name}: ${error?.message}`; resolve(null); return; }
    const done = (value, reason = '') => {
      clearTimeout(timer); worker.removeEventListener('message', ready); worker.removeEventListener('error', failed);
      workerStatus.mode = value ? 'worker' : 'main-thread'; workerStatus.reason = reason;
      if (!value) worker.terminate();
      resolve(value ? worker : null);
    };
    const ready = () => done(true);
    const failed = event => done(false, event?.message ? `error event: ${event.message}` : 'no ready message within 8 s');
    const timer = setTimeout(failed, 8000);
    worker.addEventListener('message', ready);
    worker.addEventListener('error', failed);
  });
}

const initialization = new Promise(resolve => {
  function receive(event) {
    if (event.source !== host || event.data?.channel !== CHANNEL) return;
    if (hostOrigin === null) hostOrigin = event.origin;
    else if (event.origin !== hostOrigin) return;
    const message = event.data;
    if (message.type === 'probe') { send('ready'); return; }
    if (message.instance !== instance) return;
    if (message.type === 'open' && nonce === null && typeof message.nonce === 'string' && message.nonce.length <= 100 && message.bytes instanceof ArrayBuffer) {
      nonce = message.nonce;
      configuration = message;
      applyTheme(message.theme);
      resolve(message);
    } else if (message.nonce === nonce && nonce !== null) {
      if (message.type === 'theme') applyTheme(message.theme);
      if (message.type === 'permissions') { configuration.downloadAllowed=message.downloadAllowed!==false;applyDownloadPermission(); }
      else if (message.type === 'destroy') void application?.close();
    }
  }
  window.addEventListener('message', receive);
  window.addEventListener('pagehide', () => { window.removeEventListener('message', receive); void application?.close(); }, { once: true });
});

// Register before the viewer imports: upstream dispatches this synchronous event
// in the parent document, specifically to allow embedded hosts to set options.
function configure(event) {
  if (event.detail?.source !== window) return;
  const options = window.PDFViewerApplicationOptions;
  const settings = {
    defaultUrl: '', disablePreferences: true, disableHistory: true, pdfBugEnabled: false,
    enableScripting: false, isEvalSupported: false, enableXfa: false,
    enableAltText: false, enableGuessAltText: false, enableAltTextModelDownload: false,
    enableNewAltTextWhenAddingImage: false, enableSignatureEditor: false,
    enableUpdatedAddImage: false, enablePermissions: true, workerSrc: worker, workerPort,
    localeProperties: { lang: configuration?.locale || 'en-US' },
    enableAutoLinking: true,
    // Absolute: the worker runs from a blob URL and cannot resolve relative ones.
    cMapUrl: `${base}cmaps/`, standardFontDataUrl: `${base}standard_fonts/`, wasmUrl: `${base}wasm/`, iccUrl: `${base}iccs/`,
  };
  for (const [key, value] of Object.entries(settings)) options.set(key, value);
}

async function start() {
  send('ready');
  stage = 'handshake';
  const message = await initialization;
  stage = 'worker';
  workerPort = await startWorker();
  stage = 'stylesheets';
  // The viewer measures layout as it starts. Let our stylesheets finish first, or the
  // browser warns that layout was forced before the page was fully loaded (Firefox) and
  // the first measurements are taken against unstyled content.
  await Promise.all([...document.querySelectorAll('link[rel="stylesheet"]')].map(link => link.sheet ? null : new Promise(resolve => {
    link.addEventListener('load', resolve, { once: true });
    link.addEventListener('error', resolve, { once: true });
  })));
  if (document.readyState !== 'complete') await new Promise(resolve => window.addEventListener('load', resolve, { once: true }));
  document.addEventListener('webviewerloaded', configure);
  try {
    stage = 'library';
    await import('./build/pdf.mjs');
    stage = 'viewer';
    await import('./web/viewer.mjs');
    application = window.PDFViewerApplication;
    stage = 'initialize';
    await application.initializedPromise;
  } finally {
    document.removeEventListener('webviewerloaded', configure);
  }
  // Never place an external URL in a live anchor (including context-menu and
  // middle-click navigation); all supported URL schemes pass app confirmation.
  application.pdfLinkService.addLinkAttributes = (link, url) => {
    link.href = '#';
    link.title = url;
    link.dataset.mailwomanExternalLink = url;
    link.onclick = event => { event.preventDefault(); externalLink(url); return false; };
  };
  document.addEventListener('click', event => {
    const anchor = event.target.closest?.('a[href]');
    if (!anchor || anchor.hasAttribute('download')) return;
    const url = anchor.dataset.mailwomanExternalLink;
    if (url) { event.preventDefault(); event.stopImmediatePropagation(); externalLink(url); return; }
    const href = anchor.getAttribute('href');
    if (href && !href.startsWith('#') && !href.startsWith('blob:')) {
      event.preventDefault(); event.stopImmediatePropagation(); externalLink(anchor.href);
    }
  }, true);
  document.addEventListener('auxclick', event => {
    const anchor = event.target.closest?.('a[href]');
    if (anchor && !anchor.hasAttribute('download')) { event.preventDefault(); if (anchor.dataset.mailwomanExternalLink) externalLink(anchor.dataset.mailwomanExternalLink); }
  }, true);
  // Embedded attachments are downloads, not new viewer windows taking arbitrary
  // file= inputs. The main PDF's save/download retains annotations and form data.
  if (application.downloadManager) {
    // An opaque-origin blob cannot reliably download through upstream's
    // target="_parent" (Firefox). The owning app saves a fresh copy instead.
    const saveData = (data, filename, mime) => {
      if(configuration?.downloadAllowed===false)return;
      const bytes = data instanceof ArrayBuffer ? data.slice(0) : ArrayBuffer.isView(data) ? data.buffer.slice(data.byteOffset, data.byteOffset + data.byteLength) : null;
      if (bytes?.byteLength) send('download', { bytes, name: filename, mime }, [bytes]);
    };
    application.downloadManager.download = (data, _url, filename) => saveData(data, filename, 'application/pdf');
    application.downloadManager.downloadData = (data, filename, mime) => saveData(data, filename, mime || 'application/octet-stream');
    application.downloadManager.openOrDownloadData = (data, filename) => {
      application.downloadManager.downloadData(data, filename, 'application/octet-stream');
      return false;
    };
  }
  const open = application.open.bind(application);
  application.open = args => {
    if (!args?.data && !(typeof args?.url === 'string' && args.url.startsWith('blob:'))) return Promise.reject(new Error('PDF URLs are not permitted'));
    return open(args);
  };
  // Reuse the live upstream scale control in the overflow menu on small
  // screens; its listeners, labels and fit/page-width choices stay intact.
  const scale = document.getElementById('scaleSelectContainer');
  const menu = document.getElementById('secondaryToolbarButtonContainer');
  const position = document.createComment('PDF zoom placement');
  scale.before(position);
  const compact = matchMedia('(max-width: 690px)');
  const placeScale = () => { if (compact.matches) menu.prepend(scale); else position.after(scale); };
  compact.addEventListener('change', placeScale);
  window.addEventListener('pagehide', () => compact.removeEventListener('change', placeScale), { once: true });
  placeScale();
  // A PDF's OpenAction/JavaScript Print action must never trigger a dialog.
  // The toolbar/keyboard print command remains available after a user gesture.
  application._initializeAutoPrint = async () => {};
  const passwordPrompt = application.passwordPrompt;
  const openPasswordPrompt = passwordPrompt.open.bind(passwordPrompt);
  passwordPrompt.open = (...args) => { send('password'); return openPasswordPrompt(...args); };
  stage = 'document';
  await application.open({ data: new Uint8Array(message.bytes) });
  stage = 'loaded';
  application._contentDispositionFilename = [...String(message.name || 'document.pdf')].map(char => char === '/' || char === '\\' || char.charCodeAt(0) < 32 ? '_' : char).join('');
  applyDownloadPermission();
  document.title = application._contentDispositionFilename;
  opened = true;
  send('loaded', { pages: application.pdfDocument.numPages, ...(await documentMeta()) });
}
// Document facts for the File details dialog: what the PDF itself declares, bounded by the host.
async function documentMeta() {
  try {
    const { info } = await application.pdfDocument.getMetadata();
    const text = value => (typeof value === 'string' ? value.slice(0, 200) : undefined);
    return { producer: text(info?.Producer), creator: text(info?.Creator), created: text(info?.CreationDate), pdfVersion: text(info?.PDFFormatVersion) };
  } catch { return {}; }
}
// The error detail is what the host files with the client error reporter; it is
// bounded here and scrubbed again by the host (no document text is ever included).
function errorDetail(reason) {
  const e = reason instanceof Error ? reason : null;
  return {
    name: String(e?.name ?? 'PdfViewerError').slice(0, 80),
    message: String(e?.message ?? (typeof reason === 'string' ? reason : 'PDF viewer failed')).slice(0, 300),
    stack: String(e?.stack ?? '').slice(0, 2000),
    // Where it stopped and what this browser lacked (never document content).
    stage,
    pdfjs: String(window.pdfjsLib?.version ?? '').slice(0, 40),
    polyfilled: polyfilled.join(',').slice(0, 400),
    worker: `${workerStatus.mode}${workerStatus.reason ? `: ${workerStatus.reason}` : ''}`.slice(0, 300),
  };
}
start().catch(reason => send('error', errorDetail(reason)));
window.addEventListener('unhandledrejection', event => { if (!opened) send('error', errorDetail(event.reason)); });
