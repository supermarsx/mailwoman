// The quoted original of a reply or a forward.
//
// The input is what the reader displayed: the sanitised HTML of the message
// (server-sanitised cleartext, or an encrypted message's plaintext sanitised in
// the crypto worker), or its plain text. It is parsed into a document that has
// no browsing context, stripped of everything that embeds or loads, and then
// reduced to the composer's schema (`richtext.ts`) — paragraphs, headings,
// lists, blockquotes, code, and http/https/mailto links. The schema has no
// image node and keeps no `style`, `class` or event attribute, so no remote
// reference of the original is carried into the outgoing body.
//
// ProseMirror is imported on demand: this module is reached from the reader,
// which is on the entry chunk, and the schema's libraries are not.

/** What the reader displayed for the message being quoted. `html` wins when
 *  both are given; neither (an encrypted message not yet decrypted) quotes
 *  nothing. */
export interface QuoteSource {
  html?: string;
  text?: string;
}

/** A quoted block in both forms the composer keeps. */
export interface Quote {
  html: string;
  text: string;
}

/** Elements removed whole, with their content, before the schema sees the
 *  markup. The schema would drop the tags anyway; this also drops what is
 *  inside them (fallback text of an `<iframe>`, the label of a `<button>`). */
const DROPPED = [
  'script', 'style', 'link', 'meta', 'base', 'title', 'head', 'template', 'noscript',
  'iframe', 'frame', 'frameset', 'object', 'embed', 'applet',
  'img', 'picture', 'source', 'video', 'audio', 'track', 'canvas', 'svg', 'math', 'map', 'area',
  'form', 'input', 'button', 'select', 'textarea',
].join(',');

function escapeHtml(s: string): string {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}

/** The source reduced to the compose schema, as HTML and as text. Empty when
 *  the source holds nothing to quote. */
async function reduce(source: QuoteSource): Promise<Quote> {
  const rt = await import('./richtext.ts');
  if (source.html !== undefined) {
    const body = rt.inertBody(source.html);
    for (const el of Array.from(body.querySelectorAll(DROPPED))) el.remove();
    const doc = rt.docFromElement(body);
    const text = rt.textFromDoc(doc);
    return text.trim() === '' ? { html: '', text: '' } : { html: rt.htmlFromDoc(doc), text };
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

/**
 * The body a reply opens with: an empty first paragraph for the answer, the
 * attribution line, and the original inside a blockquote. Returns an empty
 * quote when there is nothing to quote.
 */
export async function quoteForReply(source: QuoteSource, attribution: string): Promise<Quote> {
  const original = await reduce(source);
  if (original.html === '') return original;
  return {
    html: `<p></p><p>${escapeHtml(attribution)}</p><blockquote>${original.html}</blockquote>`,
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
    html: `<p></p><p>${header}</p>${original.html}`,
    text: `\n\n${headerLines.join('\n')}\n\n${original.text}`,
  };
}
