// Rich-text (ProseMirror) schema + serialization for the composer (W1).
//
// This module is DOM-serializer-based (ProseMirror `DOMSerializer`/`DOMParser`),
// so it runs in the browser and in the jsdom test environment — it is the pure,
// view-independent half of the editor and is unit-tested directly. The stateful
// `EditorView` lives in `RichTextEditor.tsx`.
//
// The schema is deliberately email-safe and small: paragraphs, headings,
// blockquote, ordered/bulleted lists, hard breaks, and the inline marks bold /
// italic / underline / strikethrough / link. No images, tables, or arbitrary
// HTML — the outgoing body stays a compact, predictable subset that the reader
// pane's sanitizer already understands.

import { Schema, DOMParser, DOMSerializer, type Node as PMNode } from 'prosemirror-model';
import { schema as basicSchema } from 'prosemirror-schema-basic';
import { addListNodes } from 'prosemirror-schema-list';

// paragraph + heading + blockquote + code_block + lists over the basic nodes.
const nodes = addListNodes(basicSchema.spec.nodes, 'paragraph block*', 'block');

/** The composer's rich-text schema (basic block/inline set + underline + strike). */
export const richSchema = new Schema({
  nodes,
  marks: basicSchema.spec.marks.append({
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

/** Parse an HTML string into a ProseMirror document under the composer schema. */
export function docFromHtml(html: string): PMNode {
  const container = document.createElement('div');
  container.innerHTML = html;
  return DOMParser.fromSchema(richSchema).parse(container);
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
