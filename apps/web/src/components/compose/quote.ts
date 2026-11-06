// The quoted original of a reply or a forward.
//
// The input is what the reader displayed: the sanitised HTML of the message
// (server-sanitised cleartext, or an encrypted message's plaintext sanitised in
// the crypto worker), or its plain text. HTML is re-run through the Rust
// policy (`normalizeHtmlSource`) and kept as native markup: tables, safe inline
// styles, fonts and spacing survive, local-file and other dangerous URL
// schemes are removed, and only images this reply owns (freshly uploaded CIDs)
// remain. Plain text is quoted as paragraphs.
//
// ProseMirror is imported on demand: this module is reached from the reader,
// which is on the entry chunk, and the schema's libraries are not.

/** What the reader displayed for the message being quoted. `html` wins when
 *  both are given; neither (an encrypted message not yet decrypted) quotes
 *  nothing. */
export interface QuoteSource {
  html?: string;
  text?: string;
  /** Only images freshly uploaded for this reply may enter its quoted HTML. */
  ownedCids?:ReadonlySet<string>;
}

/** A quoted block in both forms the composer keeps. */
export interface Quote {
  html: string;
  text: string;
}

/** Elements removed whole, with their content, before the Rust policy runs:
 *  the policy would drop the tags, but their fallback text (an `<iframe>`'s,
 *  a `<button>`'s label, a `<title>`) must not become visible quote text. */
const DROPPED = [
  'script', 'link', 'meta', 'base', 'title', 'template', 'noscript',
  'iframe', 'frame', 'frameset', 'object', 'embed', 'applet',
  'picture', 'source', 'video', 'audio', 'track', 'canvas', 'svg', 'math', 'map', 'area',
  'form', 'input', 'button', 'select', 'textarea',
].join(',');

function escapeHtml(s: string): string {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}


/** Spacing and overflow tokens of the quote. Em values keep the margins
 *  proportional to the sender's font size in every mail client. */
const QUOTE_BLOCKQUOTE_STYLE = 'margin:0 0 0 .8em;padding:0 0 0 1em;border-left:2px solid #c8c8c8;max-width:100%';

/** Keep quoted images and tables inside the message width. Existing
 *  `max-width` declarations of the original are respected. */
export function fitQuoteHtml(safeHtml: string): string {
  const template = document.createElement('template');
  template.innerHTML = safeHtml;
  // Placeholders of removed content leave empty spans behind.
  for (const span of Array.from(template.content.querySelectorAll('span'))) if (span.attributes.length === 0 && span.childNodes.length === 0) span.remove();
  // A link whose href the policy removed is plain text.
  for (const a of Array.from(template.content.querySelectorAll('a:not([href])'))) a.replaceWith(...Array.from(a.childNodes));
  // An image whose source the policy removed is dead: keep its alt text only.
  for (const img of Array.from(template.content.querySelectorAll('img'))) {
    if ((img.getAttribute('src') ?? '').trim() !== '') continue;
    const alt = (img.getAttribute('alt') ?? '').trim();
    if (alt !== '') img.replaceWith(alt); else img.remove();
  }
  for (const el of Array.from(template.content.querySelectorAll('img,table')) as HTMLElement[]) {
    const style = el.getAttribute('style') ?? '';
    if (/max-width\s*:/i.test(style)) continue;
    el.setAttribute('style', `${style.replace(/;?\s*$/, style ? ';' : '')}max-width:100%`);
  }
  return template.innerHTML;
}

/** The source reduced to the compose schema, as HTML and as text. Empty when
 *  the source holds nothing to quote. */
async function reduce(source: QuoteSource): Promise<Quote> {
  const rt = await import('./richtext.ts');
  if (source.html !== undefined) {
    // The original stays native HTML: the Rust policy allowlists it (tables,
    // safe inline styles, fonts, spacing, classes scoped to the quote), and
    // only images this reply owns survive. No schema reduction, so a
    // newsletter or Outlook layout reaches the editor and the sent MIME intact.
    const owned = source.ownedCids ?? new Set<string>();
    const {normalizeHtmlSource,scopeQuoteHtml}=await import('./htmlSource.ts');
    const {referencedCids}=await import('./inlineImages.ts');
    // A parsed-but-inert document: its head stylesheets and body attributes
    // reach the policy together with the body.
    const cleaned=new DOMParser().parseFromString(source.html,'text/html');
    for (const el of Array.from(cleaned.querySelectorAll(DROPPED))) el.remove();
    const original=await normalizeHtmlSource(cleaned.documentElement.outerHTML,owned);
    const html=fitQuoteHtml(scopeQuoteHtml(original.html));
    const hasOwnedImage=referencedCids(html).some(cid=>owned.has(cid));
    return original.text.trim()!==''||hasOwnedImage?{html,text:original.text}:{html:'',text:''};
  }
  const text = source.text ?? '';
  if (text.trim() === '') return { html: '', text: '' };
  return { html: rt.htmlFromDoc(rt.docFromText(text)), text };
}

/** Each line of `text` behind the plain-text quote marker. */
function quoteLines(text: string): string {
  return text
    .split('\n')
    .map((line) => (line === '' ? '>' : `> ${line}`))
    .join('\n');
}

/** Empty paragraphs the answer is written in, above the quote. Real
 * paragraphs, not CSS spacing, so Enter makes ordinary ones; the caret opens
 * in the first. */
export const REPLY_SPACER_PARAGRAPHS = 2;
export const REPLY_SPACER_HTML = '<p><br></p>'.repeat(REPLY_SPACER_PARAGRAPHS);

/**
 * The body a reply opens with: empty paragraphs for the answer, the
 * attribution line, and the original inside a blockquote. Returns an empty
 * quote when there is nothing to quote.
 */
export async function quoteForReply(source: QuoteSource, attribution: string): Promise<Quote> {
  const original = await reduce(source);
  if (original.html === '') return original;
  return {
    html: `${REPLY_SPACER_HTML}<p>${escapeHtml(attribution)}</p><blockquote style="${QUOTE_BLOCKQUOTE_STYLE}">${original.html}</blockquote>`,
    text: `\n\n${attribution}\n${quoteLines(original.text)}`,
  };
}

/**
 * The body a forward opens with: an empty first paragraph, the header lines
 * of the original (`headerLines`: a title line, then From / Date / Subject /
 * To as the caller worded them), and the original below them, unquoted.
 */
export async function quoteForForward(source: QuoteSource, headerLines: string[]): Promise<Quote> {
  const original = await reduce(source);
  const header = headerLines.map(escapeHtml).join('<br>');
  return {
    html: `${REPLY_SPACER_HTML}<p>${header}</p>${original.html}`,
    text: `\n\n${headerLines.join('\n')}\n\n${original.text}`,
  };
}

/** Outlook-style metadata header; originals still pass through the inert quote schema. */
export async function quoteForOutlook(source: QuoteSource, fields: { label: string; value: string }[]): Promise<Quote> {
  const original = await reduce(source);
  const header = fields.map(field => `<strong>${escapeHtml(field.label)}</strong> ${escapeHtml(field.value)}`).join('<br>');
  return {
    html: `${REPLY_SPACER_HTML}<hr><p>${header}</p>${original.html}`,
    text: `\n\n________________________________\n${fields.map(field => `${field.label} ${field.value}`).join('\n')}\n\n${original.text}`,
  };
}
