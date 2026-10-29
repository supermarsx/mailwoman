import {styledBlock,colorMark,composeTableNodes,boundTableHtml} from './richFormatting.ts';
import {COMPOSER_FONTS,normalizeComposerFont} from './fontCatalog.ts';
// Rich-text (ProseMirror) schema + serialization for the composer (W1).
//
// This module is DOM-serializer-based (ProseMirror `DOMSerializer`/`DOMParser`),
// so it runs in the browser and in the jsdom test environment — it is the pure,
// view-independent half of the editor and is unit-tested directly. The stateful
// `EditorView` lives in `RichTextEditor.tsx`.
//
// The schema is deliberately email-safe and small: paragraphs, headings,
// blockquote, code blocks, horizontal rules, ordered/bulleted lists, hard
// breaks, inline images of this draft, and the inline marks bold / italic /
// code / underline / strikethrough / link (http, https and mailto targets
// only), bounded tables, and explicit paragraph/color formatting. Attributes
// are constrained to owned CID images, safe links, finite paragraph styles,
// colors and allowlisted font family/point size.
// `htmlFromDoc(docFromHtml(x))` reduces any HTML
// to that subset: a `javascript:` link keeps its text and loses the link, and
// Unsupported style properties and all event handlers do not survive. The composer relies on this for
// HTML it did not author (an identity's stored signature).
//
// Images (t29-e12): the `image` node has no parse rule of its own. Only a parse
// given the set of Content-IDs this draft uploaded (`docFromHtml(html, owned)`)
// keeps an `<img src="cid:<id>">` whose id is in that set. Every other `<img>` —
// a remote URL, `data:`, a `cid:` the draft does not own, or any image at all
// when no set is given — is dropped at parse. Quoted originals are parsed under
// `quoteSchema`, which has no image node at all.
//
// Embedded images (t49): a signature has no upload of its own, so its images
// live in its HTML as `data:` URLs of a raster type (`EMBEDDED_IMAGE`). Only a
// parse asked for them (`docFromHtml(html, owned, true)`, the signature editor)
// keeps one; the composer turns each into an uploaded `cid:` image of its draft
// before the signature enters the body (`inlineImages.ts`).

import {
  Schema,
  DOMParser,
  DOMSerializer,
  type MarkSpec,
  type NodeSpec,
  type Node as PMNode,
  type TagParseRule,
} from 'prosemirror-model';
import { schema as basicSchema } from 'prosemirror-schema-basic';
import { addListNodes } from 'prosemirror-schema-list';

// paragraph + heading + blockquote + code_block + horizontal_rule + lists: the
// basic nodes without the basic `image`, whose parse rule would let parsed HTML
// carry a remote `<img src>` into the outgoing body.
const basicNodes=basicSchema.spec.nodes.remove('image').update('paragraph',styledBlock(basicSchema.spec.nodes.get('paragraph')!)).update('heading',styledBlock(basicSchema.spec.nodes.get('heading')!,true));
const nodes = addListNodes(basicNodes, 'paragraph block*', 'block').append(composeTableNodes);

/** An image this draft uploaded, referenced by its Content-ID. Serialises as
 *  `<img src="cid:<id>">`; the editor shows a local preview instead (a node view
 *  in `RichTextEditor.tsx`). No `parseDOM`: see `docFromElement`. */
const image: NodeSpec = {
  inline: true,
  group: 'inline',
  draggable: true,
  attrs: { cid: { default: '' }, alt: { default: '' }, src: { default: null } },
  toDOM(node) {
    const embedded = node.attrs['src'] as string | null;
    return ['img', { src: embedded ?? `cid:${node.attrs['cid'] as string}`, alt: node.attrs['alt'] as string }];
  },
};

/** Link targets a parsed `<a href>` may keep. Anything else (`javascript:`,
 *  `data:`, a relative path) is not a link the recipient can use. */
const LINK_HREF = /^(https?:|mailto:)/i;

// The basic link mark, except that parsing refuses an href outside LINK_HREF:
// `false` means "this rule does not match", so the text stays and the mark is
// not applied.
const link: MarkSpec = {
  ...basicSchema.spec.marks.get('link'),
  parseDOM: [
    {
      tag: 'a[href]',
      getAttrs(dom: HTMLElement) {
        const href = (dom.getAttribute('href') ?? '').trim();
        return LINK_HREF.test(href) ? { href, title: dom.getAttribute('title') } : false;
      },
    },
  ],
};

export const COMPOSE_FONTS=COMPOSER_FONTS.map(font=>font.label);
export const COMPOSE_FONT_SIZES=[8,9,10,11,12,14,16,18,20,24,28,32,36,48,72] as const;
function textStyle(property:'font-family'|'font-size'):MarkSpec {
 const valid=(value:unknown):string|false=>{
  if(typeof value!=='string')return false;
  const clean=value.replace(/^["']|["']$/g,'').trim();
  return property==='font-family' ? normalizeComposerFont(value)??normalizeComposerFont(clean)??false : COMPOSE_FONT_SIZES.some(size=>clean===`${size}pt`)?clean:false;
 };
 return {attrs:{value:{}},parseDOM:[{style:property,getAttrs:value=>{const checked=valid(value);return checked?{value:checked}:false;}}],toDOM:mark=>{
  const span=document.createElement('span'),value=valid(mark.attrs['value']);
  if(value)span.style.setProperty(property,value);
  return {dom:span,contentDOM:span};
 }};
}

const marks = basicSchema.spec.marks.update('link', link).append({
    textColor:colorMark('color'),
    highlight:colorMark('background-color'),
    fontFamily:textStyle('font-family'),
    fontSize:textStyle('font-size'),
    underline: {
      parseDOM: [{ tag: 'u' }, { style: 'text-decoration=underline' }],
      toDOM(): ['u', 0] {
        return ['u', 0];
      },
    },
    strikethrough: {
      parseDOM: [{ tag: 's' }, { tag: 'del' }, { style: 'text-decoration=line-through' }],
      toDOM(): ['s', 0] {
        return ['s', 0];
      },
    },
  });

/** The composer's rich-text schema (basic block/inline set + underline + strike
 *  + the draft's own inline images). */
export const richSchema = new Schema({ nodes: nodes.append({ image }), marks });

/** The schema a quoted original is reduced to: the composer's, without images. */
export const quoteSchema = new Schema({ nodes, marks });

/** A Content-ID the composer generates and the engine accepts (atext, `.`, `@`). */
const CID = /^[A-Za-z0-9!#$%&'*+/=?^_`{|}~.@-]{1,255}$/;

/** The parse rule that keeps an `<img src="cid:<id>">` naming an id in `owned`. */
function ownedImageRule(owned: ReadonlySet<string>): TagParseRule {
  return {
    tag: 'img[src]',
    node: 'image',
    getAttrs(dom: HTMLElement) {
      const m = /^cid:(.+)$/i.exec((dom.getAttribute('src') ?? '').trim());
      const cid = m?.[1];
      if (cid === undefined || !CID.test(cid) || !owned.has(cid)) return false;
      return { cid, alt: dom.getAttribute('alt') ?? '' };
    },
  };
}

/** A `data:` URL of a raster image type, base64-encoded: the one embedded image
 *  form a signature keeps. SVG is not raster and can carry script. Mirrors
 *  `mw_sanitize::sanitize_signature_html`. */
export const EMBEDDED_IMAGE = /^data:image\/(?:png|jpeg|gif|webp);base64,[A-Za-z0-9+/]+={0,2}$/;

/** The parse rule that keeps an `<img src="data:image/...;base64,...">`. */
const embeddedImageRule: TagParseRule = {
  tag: 'img[src]',
  node: 'image',
  getAttrs(dom: HTMLElement) {
    const src = (dom.getAttribute('src') ?? '').trim();
    if (!EMBEDDED_IMAGE.test(src)) return false;
    return { src, alt: dom.getAttribute('alt') ?? '' };
  },
};

/** True only where a real DOM (browser or jsdom) is available for (de)serialization. */
function hasDom(): boolean {
  return typeof document !== 'undefined';
}

/** Serialize a ProseMirror document to an HTML string for the send payload. */
export function htmlFromDoc(doc: PMNode): string {
  if (!hasDom()) return '';
  // Serialise into a document of its own: an <img src="cid:..."> built in the page's
  // document would be fetched (and refused by img-src, t49) just to be read back as text.
  const inert = document.implementation.createHTMLDocument('');
  const serializer = DOMSerializer.fromSchema(doc.type.schema);
  const fragment = serializer.serializeFragment(doc.content, { document: inert });
  const container = inert.createElement('div');
  container.appendChild(fragment);
  return container.innerHTML;
}

/**
 * Parse `html` into the body of a document of its own. That document has no
 * browsing context, so nothing in it runs or loads: assigning the same markup
 * to `innerHTML` of an element of the page would start fetching every
 * `<img src>` in it, attached or not.
 */
export function inertBody(html: string): HTMLElement {
  // The parser never sees a `style` attribute: Chromium reports every one it
  // parses as a refused inline style (t67: ~65 console reports per composer
  // and signature run), even in a DOMParser document. The declarations go in
  // through the CSSOM instead, which the CSP allows.
  const body = new window.DOMParser().parseFromString(renameStyleAttributes(html), 'text/html').body;
  hydrateInlineStyles(body);
  return body;
}

/** The attribute `style` carries while markup is parsed (see `inertBody`). */
export const PARSED_STYLE_ATTRIBUTE = 'data-mw-parsed-style';
const RAW_TEXT = new Set(['script', 'style', 'textarea', 'title', 'xmp', 'iframe', 'noembed', 'noframes', 'plaintext']);

/**
 * Rename every `style` attribute in `html` to `data-mw-parsed-style` (`data-mw-style` is the paragraph preset), following the
 * HTML tokenizer closely enough that text, comments, raw-text element content
 * and other attributes' values are left exactly as they are: only attribute
 * names inside start tags are looked at.
 */
export function renameStyleAttributes(html: string): string {
  if (!/style/i.test(html)) return html;
  let out = '';
  let i = 0;
  const n = html.length;
  const space = (c: string | undefined): boolean => c === ' ' || c === '\t' || c === '\n' || c === '\r' || c === '\f';
  while (i < n) {
    const lt = html.indexOf('<', i);
    if (lt < 0) { out += html.slice(i); break; }
    out += html.slice(i, lt);
    i = lt;
    if (html.startsWith('<!--', i)) {
      const end = html.indexOf('-->', i + 4);
      const stop = end < 0 ? n : end + 3;
      out += html.slice(i, stop); i = stop; continue;
    }
    const next = html[i + 1];
    if (next === '!' || next === '?' || (next === '/' && /[a-z]/i.test(html[i + 2] ?? ''))) {
      const end = html.indexOf('>', i + 1);
      const stop = end < 0 ? n : end + 1;
      out += html.slice(i, stop); i = stop; continue;
    }
    if (!/[a-z]/i.test(next ?? '')) { out += '<'; i += 1; continue; }
    // A start tag: its name, then attributes up to '>'.
    let j = i + 1;
    while (j < n && !space(html[j]) && html[j] !== '/' && html[j] !== '>') j++;
    const tag = html.slice(i + 1, j).toLowerCase();
    out += html.slice(i, j);
    while (j < n && html[j] !== '>') {
      if (space(html[j]) || html[j] === '/') { out += html[j]; j++; continue; }
      let k = j + 1;
      while (k < n && !space(html[k]) && html[k] !== '/' && html[k] !== '>' && html[k] !== '=') k++;
      const name = html.slice(j, k);
      out += name.toLowerCase() === 'style' ? PARSED_STYLE_ATTRIBUTE : name;
      j = k;
      let m = j;
      while (m < n && space(html[m])) m++;
      if (html[m] !== '=') continue;
      m++;
      while (m < n && space(html[m])) m++;
      let end: number;
      if (html[m] === '"' || html[m] === "'") { const close = html.indexOf(html[m]!, m + 1); end = close < 0 ? n : close + 1; }
      else { end = m; while (end < n && !space(html[end]) && html[end] !== '>') end++; }
      out += html.slice(j, end); j = end;
    }
    if (j < n) { out += '>'; j++; }
    i = j;
    if (RAW_TEXT.has(tag)) {
      const close = html.toLowerCase().indexOf(`</${tag}`, i);
      const stop = close < 0 ? n : close;
      out += html.slice(i, stop); i = stop;
    }
  }
  return out;
}

/** The `prop: value` declarations of a `style` attribute, split on `;` outside quotes and brackets. */
export function styleDeclarations(text: string): [string, string, string][] {
  const out: [string, string, string][] = [];
  let current = '';
  let quote = '';
  let depth = 0;
  const flush = (): void => {
    const colon = current.indexOf(':');
    if (colon > 0) {
      const name = current.slice(0, colon).trim().toLowerCase();
      let value = current.slice(colon + 1).trim();
      const important = /!\s*important$/i.test(value);
      if (important) value = value.replace(/!\s*important$/i, '').trim();
      if (/^-?[a-z][a-z0-9-]*$/.test(name) && value !== '') out.push([name, value, important ? 'important' : '']);
    }
    current = '';
  };
  for (let i = 0; i < text.length; i++) {
    const ch = text[i]!;
    if (quote !== '') {
      current += ch;
      if (ch === '\\' && i + 1 < text.length) current += text[++i]!;
      else if (ch === quote) quote = '';
    } else if (ch === '"' || ch === "'") {
      quote = ch;
      current += ch;
    } else if (ch === '(') {
      depth++;
      current += ch;
    } else if (ch === ')') {
      depth = Math.max(0, depth - 1);
      current += ch;
    } else if (ch === ';' && depth === 0) {
      flush();
    } else {
      current += ch;
    }
  }
  flush();
  return out;
}

/**
 * Under the production CSP (`style-src 'self'`, t49), Chromium does not apply a
 * `style` attribute that arrives in parsed markup, so `element.style` is empty
 * and every style parse rule (font, size, colour, paragraph format) would drop
 * its formatting on load. Setting the same declarations through the CSSOM is
 * allowed, so the attribute's text is applied that way before the parse. The
 * schema's rules still decide what is kept.
 */
export function hydrateInlineStyles(root: HTMLElement): void {
  for (const element of Array.from(root.querySelectorAll<HTMLElement>(`[${PARSED_STYLE_ATTRIBUTE}]`))) {
    const text = element.getAttribute(PARSED_STYLE_ATTRIBUTE) ?? '';
    element.removeAttribute(PARSED_STYLE_ATTRIBUTE);
    for (const [name, value, priority] of styleDeclarations(text)) element.style.setProperty(name, value, priority);
  }
  for (const element of Array.from(root.querySelectorAll<HTMLElement>('[style]'))) {
    if (element.style.length > 0) continue;
    for (const [name, value, priority] of styleDeclarations(element.getAttribute('style') ?? '')) {
      element.style.setProperty(name, value, priority);
    }
  }
}

/** Parse an element's content into a ProseMirror document under the composer
 *  schema. `owned` is the set of Content-IDs this draft uploaded; an image is
 *  kept only when its `cid:` names one of them. `embedded` also keeps raster
 *  `data:` images (`EMBEDDED_IMAGE`; the signature editor only). */
export function docFromElement(root: HTMLElement, owned?: ReadonlySet<string>, embedded = false): PMNode {
  hydrateInlineStyles(root);
  boundTableHtml(root);
  const extra: TagParseRule[] = [];
  if (owned !== undefined && owned.size > 0) extra.push(ownedImageRule(owned));
  if (embedded) extra.push(embeddedImageRule);
  if (extra.length === 0) return DOMParser.fromSchema(richSchema).parse(root);
  return new DOMParser(richSchema, [...extra, ...DOMParser.fromSchema(richSchema).rules]).parse(root);
}

/** Parse an HTML string into a ProseMirror document under the composer schema
 *  (`owned` and `embedded` as in `docFromElement`). */
export function docFromHtml(html: string, owned?: ReadonlySet<string>, embedded = false): PMNode {
  return docFromElement(inertBody(html), owned, embedded);
}

/** Parse a quoted original's content under `quoteSchema` (no images). */
export function quoteDocFromElement(root: HTMLElement): PMNode {
  hydrateInlineStyles(root);
  boundTableHtml(root);
  return DOMParser.fromSchema(quoteSchema).parse(root);
}

/** Plain-text projection of a document: blocks joined by blank lines, hard breaks
 *  as single newlines. This is what the crypto / DLP / dictation paths read. */
export function textFromDoc(doc: PMNode): string {
  return doc.textBetween(0, doc.content.size, '\n\n', (leaf) =>
    leaf.type.name === 'hard_break' ? '\n' : '',
  );
}

/** Build a document from plain text: newlines become hard breaks inside one
 *  paragraph. Paired with `textFromDoc` (hard breaks → `\n`) this makes the
 *  plain-text ⇄ rich toggle round-trip the exact text, blank lines included. */
export function docFromText(text: string): PMNode {
  const container = document.createElement('div');
  const p = document.createElement('p');
  const lines = text.split('\n');
  lines.forEach((line, i) => {
    if (i > 0) p.appendChild(document.createElement('br'));
    p.appendChild(document.createTextNode(line));
  });
  container.appendChild(p);
  return DOMParser.fromSchema(richSchema).parse(container);
}

/** Rich HTML → plain text (via the document), for the rich → plain-text toggle. */
export function htmlToText(html: string): string {
  return textFromDoc(docFromHtml(html));
}
