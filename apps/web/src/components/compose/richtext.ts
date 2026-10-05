// Rich-text (ProseMirror) schema + serialization for the composer (W1).
//
// This module is DOM-serializer-based (ProseMirror `DOMSerializer`/`DOMParser`),
// so it runs in the browser and in the jsdom test environment — it is the pure,
// view-independent half of the editor and is unit-tested directly. The stateful
// `EditorView` lives in `RichTextEditor.tsx`.
//
// The schema is deliberately email-safe and small: paragraphs, headings,
// blockquote, code blocks, horizontal rules, ordered/bulleted lists, hard
// breaks, and the inline marks bold / italic / code / underline /
// strikethrough / link (http, https and mailto targets only). There is no
// image node, no table, and no attribute other than a link's `href`/`title`,
// so `htmlFromDoc(docFromHtml(x))` reduces any HTML to that subset: an `<img>`
// is dropped, a `javascript:` link keeps its text and loses the link, and
// `style`/`on*` attributes do not survive. The composer relies on this for
// HTML it did not author (an identity's stored signature).

import { Schema, DOMParser, DOMSerializer, type MarkSpec, type Node as PMNode } from 'prosemirror-model';
import { schema as basicSchema } from 'prosemirror-schema-basic';
import { addListNodes } from 'prosemirror-schema-list';

// paragraph + heading + blockquote + code_block + horizontal_rule + lists: the
// basic nodes without `image`, which would let parsed HTML carry a remote
// `<img src>` into the outgoing body.
const nodes = addListNodes(basicSchema.spec.nodes.remove('image'), 'paragraph block*', 'block');

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

/** The composer's rich-text schema (basic block/inline set + underline + strike). */
export const richSchema = new Schema({
  nodes,
  marks: basicSchema.spec.marks.update('link', link).append({
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
  }),
});

/** True only where a real DOM (browser or jsdom) is available for (de)serialization. */
function hasDom(): boolean {
  return typeof document !== 'undefined';
}

/** Serialize a ProseMirror document to an HTML string for the send payload. */
export function htmlFromDoc(doc: PMNode): string {
  if (!hasDom()) return '';
  const serializer = DOMSerializer.fromSchema(richSchema);
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

/** Parse an element's content into a ProseMirror document under the composer schema. */
export function docFromElement(root: HTMLElement): PMNode {
  return DOMParser.fromSchema(richSchema).parse(root);
}

/** Parse an HTML string into a ProseMirror document under the composer schema. */
export function docFromHtml(html: string): PMNode {
  return docFromElement(inertBody(html));
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
