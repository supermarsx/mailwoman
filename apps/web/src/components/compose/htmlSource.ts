import { hydrateInlineStyles, renameStyleAttributes } from './richtext.ts';

type Sanitizer = typeof import('../../wasm/mw-sanitize/mw_sanitize.js');
let ready: Promise<Sanitizer> | undefined;

/** Load only when source HTML is edited; opening the mailbox does not fetch WASM. */
function sanitizer(): Promise<Sanitizer> {
  return ready ??= import('../../wasm/mw-sanitize/mw_sanitize.js').then(async (module) => {
    await module.default();
    return module;
  }).catch((error: unknown) => {
    ready = undefined;
    throw error;
  });
}

/** Parse sanitized fragments inside a template's inert document. Keeping its
 * nodes there prevents stylesheet evaluation (and CSP reports) during editing.
 * Inline declarations are restored through CSSOM, as in the visual composer. */
export function nativeHtmlBody(safeHtml: string): HTMLElement {
  const template = document.createElement('template');
  template.innerHTML = `<div>${renameStyleAttributes(safeHtml)}</div>`;
  const body = template.content.firstElementChild as HTMLElement;
  hydrateInlineStyles(body);
  return body;
}

const CONTAINER = 'mw-email-body';

function ownedUrl(value: string, owned: ReadonlySet<string>): boolean {
  return !/^cid:/i.test(value) || owned.has(value.slice(4));
}

/** Rust has already parsed and allowlisted the CSS. This narrows its CID URLs
 * to this draft's attachments; it does not admit any additional CSS or URL. */
function ownedCss(value: string, owned: ReadonlySet<string>): string {
  return value.replace(/url\(([^)]*)\)/gi, (url: string, argument: string) => {
    const target = argument.trim().replace(/^(['"])(.*)\1$/, '$2').trim();
    return /^cid:/i.test(target) && ownedUrl(target, owned) ? url : 'none';
  });
}

/** Schemes that reach the local machine or browser internals. The Rust policy
 * already refuses them; this is the second line for every compose path, since
 * Firefox reports each such reference as "may not load or link to file:///". */
const LOCAL_SCHEME = /^(?:file|filesystem|ms-help|ms-its|mk|res|chrome|resource|view-source|javascript|vbscript|jar|wyciwyg|moz-extension|chrome-extension)$/i;
const URL_ATTRIBUTES = ['src', 'srcset', 'href', 'background', 'poster', 'action', 'formaction', 'xlink:href', 'data', 'longdesc', 'usemap'];

/** True when `value` names a scheme that must not appear in outgoing mail. */
export function localUrl(value: string): boolean {
  // Control characters are what is being removed here.
  // eslint-disable-next-line no-control-regex
  const compact = value.replace(/[\u0000-\u0020\u007f]+/g, '');
  const scheme = /^([a-z][a-z0-9+.-]*):/i.exec(compact)?.[1];
  if (scheme === undefined) return false;
  if (LOCAL_SCHEME.test(scheme)) return true;
  if (/^about$/i.test(scheme)) return !/^about:blank$/i.test(compact);
  if (/^data$/i.test(scheme)) return !/^data:image\/(?:png|jpe?g|gif|webp);base64,/i.test(compact);
  return false;
}

/** Remove local-scheme references from `body`. Dead images lose the attribute
 * and keep their alt text; links keep their text and lose the href. */
export function stripLocalUrls(body: HTMLElement): void {
  for (const element of Array.from(body.querySelectorAll('*'))) {
    for (const name of URL_ATTRIBUTES) {
      const value = element.getAttribute(name);
      if (value === null) continue;
      const candidates = name === 'srcset' ? value.split(',').map((part) => part.trim().split(/\s+/)[0] ?? '') : [value];
      if (candidates.some(localUrl)) element.removeAttribute(name);
    }
    const style = element.getAttribute('style');
    if (style !== null && /(?:file|ms-help|ms-its|mk|res|chrome|resource|view-source|javascript|vbscript)\s*:/i.test(style)) {
      element.setAttribute('style', style.replace(/url\(\s*(['"]?)\s*(?:file|ms-help|ms-its|mk|res|chrome|resource|view-source|javascript|vbscript)\s*:[^)]*\)/gi, 'none'));
    }
    if (element.tagName === 'STYLE') {
      element.textContent = (element.textContent ?? '')
        .replace(/@import[^;]*(?:;|$)/gi, '')
        .replace(/url\(\s*(['"]?)\s*(?:file|ms-help|ms-its|mk|res|chrome|resource|view-source|javascript|vbscript)\s*:[^)]*\)/gi, 'none');
    }
  }
}

function collapseContainer(body: HTMLElement): void {
  // The Rust sanitizer emits a fresh stylesheet wrapper. Reuse an existing
  // root wrapper when normalizing its own output, preserving its presentation.
  for (;;) {
    const outer = body.children.length === 1 ? body.firstElementChild : null;
    if (outer?.tagName !== 'DIV' || outer.getAttribute('class') !== 'mw-email-body' || outer.attributes.length !== 1) return;
    const content = Array.from(outer.childNodes).filter((node) => node.nodeType === Node.ELEMENT_NODE
      ? (node as Element).tagName !== 'STYLE' : (node.textContent ?? '').trim() !== '');
    const inner = content.length === 1 ? content[0] : null;
    if (!(inner instanceof HTMLElement) || !inner.classList.contains('mw-email-body')) return;
    const styles = Array.from(outer.children).filter((child) => child.tagName === 'STYLE');
    inner.prepend(...styles);
    outer.replaceWith(inner);
  }
}

function plainText(body: HTMLElement): string {
  const copy = body.cloneNode(true) as HTMLElement;
  copy.querySelectorAll('style,[hidden]').forEach((element) => element.remove());
  copy.querySelectorAll('br').forEach((element) => element.replaceWith('\n'));
  copy.querySelectorAll('p,div,h1,h2,h3,h4,h5,h6,li,tr,blockquote,pre').forEach((element) => element.append('\n'));
  copy.querySelectorAll('td,th').forEach((element) => element.append('\t'));
  return (copy.textContent ?? '').replace(/[^\S\n]+\n/g, '\n').replace(/\n{3,}/g, '\n\n').trim();
}

/** Native source editing uses the same Rust policy as stored mail/signatures,
 * without reducing tables and styling through the visual editor's schema.
 * Embedded raster images are allowed only when the signature caller opts in.
 * Loading/sanitizing errors reject so callers never publish unchecked HTML. */
export async function normalizeHtmlSource(html: string, owned: ReadonlySet<string> = new Set(), embedded = false): Promise<{ html: string; text: string }> {
  if (html.length > 8 * 1024 * 1024) throw new Error('HTML source is too large');
  const module = await sanitizer();
  const body = nativeHtmlBody(embedded ? module.sanitizeSignatureHtml(html) : module.sanitizeEmailHtml(html));
  body.querySelectorAll('span[hidden][data-mw-blocked-host]').forEach((marker) => marker.remove());
  for (const element of body.querySelectorAll('*')) {
    for (const name of ['src', 'href', 'background']) {
      const value = element.getAttribute(name);
      if (value !== null && !ownedUrl(value, owned)) element.removeAttribute(name);
    }
    const style = element.getAttribute('style');
    if (style !== null && element instanceof HTMLElement) element.style.cssText = ownedCss(style, owned);
    if (element.tagName === 'STYLE') element.textContent = ownedCss(element.textContent ?? '', owned);
  }
  stripLocalUrls(body);
  collapseContainer(body);
  return { html: body.innerHTML, text: plainText(body) };
}

/** Class of the container that holds a quoted original in a reply or forward.
 *  The sanitizer scopes every rule of the original under `.mw-email-body`, but
 *  once the quote sits inside our own message that class also names the
 *  message-level wrapper any later pass emits, so `body{...}` / `p{...}` of the
 *  original would style our own paragraphs. A class of its own keeps the rules
 *  (and the wrapper's inline font, colour, background, dir) inside the quote. */
export const QUOTE_CLASS = 'mw-quote-body';

/** Rename the sanitizer's container to `QUOTE_CLASS` in the markup and in the
 *  selectors of the original's stylesheet. Only text is rewritten; the HTML was
 *  already allowlisted and is not widened. */
export function scopeQuoteHtml(safeHtml: string): string {
  const body = nativeHtmlBody(safeHtml);
  for (const element of body.querySelectorAll('[class]')) {
    const tokens = (element.getAttribute('class') ?? '').split(/\s+/).filter(Boolean);
    if (tokens.includes(CONTAINER)) element.setAttribute('class', tokens.map((token) => (token === CONTAINER ? QUOTE_CLASS : token)).join(' '));
  }
  for (const style of body.querySelectorAll('style')) style.textContent = (style.textContent ?? '').replace(/\.mw-email-body(?![\w-])/g, `.${QUOTE_CLASS}`);
  return body.innerHTML;
}
