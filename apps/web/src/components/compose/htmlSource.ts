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
  collapseContainer(body);
  return { html: body.innerHTML, text: plainText(body) };
}
