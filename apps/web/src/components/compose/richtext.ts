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
// only). There is no table and no attribute other than a link's `href`/`title`
// and an image's `cid`/`alt`, plus allowlisted font family/point size.
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
const nodes = addListNodes(basicSchema.spec.nodes.remove('image'), 'paragraph block*', 'block');

/** An image this draft uploaded, referenced by its Content-ID. Serialises as
 *  `<img src="cid:<id>">`; the editor shows a local preview instead (a node view
 *  in `RichTextEditor.tsx`). No `parseDOM`: see `docFromElement`. */
const image: NodeSpec = {
  inline: true,
  group: 'inline',
  draggable: true,
  attrs: { cid: {}, alt: { default: '' } },
  toDOM(node) {
    return ['img', { src: `cid:${node.attrs['cid'] as string}`, alt: node.attrs['alt'] as string }];
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

/** True only where a real DOM (browser or jsdom) is available for (de)serialization. */
function hasDom(): boolean {
  return typeof document !== 'undefined';
}

/** Serialize a ProseMirror document to an HTML string for the send payload. */
export function htmlFromDoc(doc: PMNode): string {
  if (!hasDom()) return '';
  const serializer = DOMSerializer.fromSchema(doc.type.schema);
  const fragment = serializer.serializeFragment(doc.content);
  const container = document.createElement('div');
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
  return new window.DOMParser().parseFromString(html, 'text/html').body;
}

/** Parse an element's content into a ProseMirror document under the composer
 *  schema. `owned` is the set of Content-IDs this draft uploaded; an image is
 *  kept only when its `cid:` names one of them. */
export function docFromElement(root: HTMLElement, owned?: ReadonlySet<string>): PMNode {
  if (owned === undefined || owned.size === 0) return DOMParser.fromSchema(richSchema).parse(root);
  return new DOMParser(richSchema, [ownedImageRule(owned), ...DOMParser.fromSchema(richSchema).rules]).parse(root);
}

/** Parse an HTML string into a ProseMirror document under the composer schema
 *  (`owned` as in `docFromElement`). */
export function docFromHtml(html: string, owned?: ReadonlySet<string>): PMNode {
  return docFromElement(inertBody(html), owned);
}

/** Parse a quoted original's content under `quoteSchema` (no images). */
export function quoteDocFromElement(root: HTMLElement): PMNode {
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
