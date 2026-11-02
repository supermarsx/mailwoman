/* global location, window, document, history, URL, matchMedia, crypto */
// Mailwoman bridge for the unmodified, pinned Mozilla PDF.js generic viewer.
// PDF-embedded code and external resource fetching are never enabled.
const CHANNEL = 'mailwoman-pdf-v1';
const origin = location.origin;
const instance = crypto.randomUUID();
let nonce = null;
let opened = false;
let application = null;
let configuration = null;
const worker = new URL('../pdf.worker.mjs', import.meta.url).href;

// Only the fixed self-hosted worker can enter a script-URL sink. HTML is used
// by upstream Fluent's local-translation overlay, which sanitizes its DOM.
globalThis.trustedTypes?.createPolicy('default', {
  createHTML: value => value,
  createScriptURL: value => {
    if (new URL(value, location.href).href !== worker) throw new TypeError('Untrusted PDF worker');
    return value;
  },
});

// The generic viewer normally accepts file= and hash/debug preferences. This
// embedded viewer accepts bytes from its owning app only, never a URL parameter.
history.replaceState(null, '', location.pathname);
function send(type, detail = {}) {
  if (window.parent !== window) window.parent.postMessage({ channel: CHANNEL, type, nonce, instance, ...detail }, origin);
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
  const keys = ['bg', 'bg-alt', 'surface', 'text', 'text-dim', 'border', 'accent', 'accent-text', 'font-sans'];
  for (const key of keys) {
    const value = theme[key];
    if (typeof value === 'string' && value.length <= 300 && !/[;{}<>]|url\s*\(/i.test(value)) document.documentElement.style.setProperty(`--mw-${key}`, value);
  }
  document.documentElement.style.colorScheme = theme.dark ? 'dark' : 'light';
}

const initialization = new Promise(resolve => {
  function receive(event) {
    if (event.source !== window.parent || event.origin !== origin || event.data?.channel !== CHANNEL) return;
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
    enableUpdatedAddImage: false, enablePermissions: true, workerSrc: worker,
    localeProperties: { lang: configuration?.locale || 'en-US' },
    enableAutoLinking: true,
  };
  for (const [key, value] of Object.entries(settings)) options.set(key, value);
}

async function start() {
  send('ready');
  const message = await initialization;
  const hostDocument = window.parent.document;
  hostDocument.addEventListener('webviewerloaded', configure);
  try {
    await import('./build/pdf.mjs');
    await import('./web/viewer.mjs');
    application = window.PDFViewerApplication;
    await application.initializedPromise;
  } finally {
    hostDocument.removeEventListener('webviewerloaded', configure);
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
    application.downloadManager.openOrDownloadData = (data, filename) => {
      application.downloadManager.downloadData(data, filename, 'application/octet-stream');
      return false;
    };
  }
  const open = application.open.bind(application);
  application.open = args => {
    if (!args?.data && !(typeof args?.url === 'string' && args.url.startsWith(`blob:${origin}/`))) return Promise.reject(new Error('PDF URLs are not permitted'));
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
  await application.open({ data: new Uint8Array(message.bytes) });
  application._contentDispositionFilename = [...String(message.name || 'document.pdf')].map(char => char === '/' || char === '\\' || char.charCodeAt(0) < 32 ? '_' : char).join('');
  document.title = application._contentDispositionFilename;
  opened = true;
  send('loaded', { pages: application.pdfDocument.numPages });
}
start().catch(() => send('error'));
window.addEventListener('unhandledrejection', () => { if (!opened) send('error'); });
