// The composer's managed signature region (ribbon Signature control).
//
// A signature that the ribbon applied is wrapped in `<div data-mw-signature="id">`.
// The marker survives in the HTML/native editors and in saved drafts; the visual
// (schema) editor flattens unknown wrappers, so a region is also found by its
// text: the run of top-level blocks whose text equals the signature's text.
// `stripSignatureMarker` removes the attribute at send, so recipients get a plain
// `div`. Plain text uses the standard `-- ` delimiter line.

import { inertBody } from './richtext.ts';

export const SIGNATURE_ATTR = 'data-mw-signature';
export type SignaturePosition = 'above-quote' | 'below-quote' | 'end';
export const SIGNATURE_POSITIONS: readonly SignaturePosition[] = ['above-quote', 'below-quote', 'end'];
export const PLAIN_DELIMITER = '-- ';

const squash = (value: string): string => value.replace(/\s+/g, ' ').trim();
const escapeAttr = (value: string): string => value.replace(/&/g, '&amp;').replace(/"/g, '&quot;').replace(/</g, '&lt;');

/** The signature wrapped in its marker. */
export function wrapSignature(html: string, id: string): string {
  return `<div ${SIGNATURE_ATTR}="${escapeAttr(id)}">${html}</div>`;
}

/** The marker attribute removed from sent HTML (the wrapper stays, as a plain div). */
export function stripSignatureMarker(html: string): string {
  return html.replace(/\sdata-mw-signature(?:=("[^"]*"|'[^']*'|[^\s>]*))?/gi, '');
}

export interface Located {
  /** The marker's id, when the marker is there. */
  id: string | null;
  nodes: ChildNode[];
}

/** The top-level node that starts a quote: the block whose text starts with `anchor`. */
function quoteStart(nodes: ChildNode[], anchor: string | undefined): number {
  const wanted = anchor === undefined ? '' : squash(anchor);
  if (wanted === '') return -1;
  return nodes.findIndex((node) => node.nodeType === 1 && squash(node.textContent ?? '').startsWith(wanted));
}

/**
 * Where a signature region sits in `body`: the marked element, else (when
 * `text` is given) the shortest run of top-level blocks with exactly that text.
 */
export function locate(root: HTMLElement, text?: string): Located | null {
  const marked = Array.from(root.childNodes).find((node) => node.nodeType === 1 && (node as Element).hasAttribute(SIGNATURE_ATTR));
  if (marked) return { id: (marked as Element).getAttribute(SIGNATURE_ATTR), nodes: [marked] };
  const wanted = text === undefined ? '' : squash(text);
  if (wanted === '') return null;
  const nodes = Array.from(root.childNodes);
  for (let end = nodes.length - 1; end >= 0; end--) {
    let seen = '';
    for (let start = end; start >= 0 && end - start < 12; start--) {
      seen = squash(`${nodes[start]!.textContent ?? ''} ${seen}`);
      if (seen === wanted) return { id: null, nodes: nodes.slice(start, end + 1) };
      if (seen.length > wanted.length) break;
    }
  }
  return null;
}

/** Whether `html` holds a marked signature region. */
export function hasSignatureRegion(html: string): boolean {
  return locate(inertBody(html)) !== null;
}

/** The id of the marked region, or null (none, or flattened by the visual editor). */
export function markedSignatureId(html: string): string | null {
  return locate(inertBody(html))?.id ?? null;
}

/** The squashed text of the marked region (to tell an edited signature from the stored one). */
export function markedSignatureText(html: string): string | null {
  const found = locate(inertBody(html));
  return found && found.id !== null ? squash(found.nodes.map((n) => n.textContent ?? '').join(' ')) : null;
}

export interface PlaceOptions {
  /** The text of the quote's header, to put the signature relative to the quote. */
  quoteAnchor?: string | undefined;
  /** Quote kind: a reply quote ends at its blockquote; a tail quote runs to the end. */
  quoteKind?: 'reply' | 'tail' | undefined;
  position: SignaturePosition;
  /** The text of the signature the region should be found by when it carries no marker. */
  knownText?: string | undefined;
}

/** Remove the managed region. Returns the body unchanged when there is none. */
export function removeSignatureHtml(body: string, knownText?: string): string {
  const root = inertBody(body);
  const found = locate(root, knownText);
  if (!found) return body;
  for (const node of found.nodes) node.remove();
  return root.innerHTML;
}

/**
 * `body` with the signature applied: replaced where the managed region stands,
 * or inserted by `position` when there is none. The signature never goes into
 * the quote: above-quote puts it before the quote's header; below-quote and end
 * put it after everything.
 */
export function applySignatureHtml(body: string, signatureHtml: string, id: string, opts: PlaceOptions): string {
  const root = inertBody(body);
  const block = inertBody(wrapSignature(signatureHtml, id)).firstChild as ChildNode;
  const found = locate(root, opts.knownText);
  if (found) {
    const first = found.nodes[0]!;
    first.parentNode!.insertBefore(root.ownerDocument.importNode(block, true), first);
    for (const node of found.nodes) node.remove();
    return root.innerHTML;
  }
  const nodes = Array.from(root.childNodes);
  const at = opts.position === 'above-quote' ? quoteStart(nodes, opts.quoteAnchor) : -1;
  if (at >= 0) {
    // Keep the empty paragraph that precedes a quote (the answer spacer) above the signature.
    root.insertBefore(root.ownerDocument.importNode(block, true), nodes[at]!);
  } else root.appendChild(root.ownerDocument.importNode(block, true));
  return root.innerHTML;
}

// ── plain text ───────────────────────────────────────────────────────────────

interface PlainSpan { start: number; end: number }

/** The `-- ` block: from the last delimiter line to the quote's header (if it follows) or the end. */
function plainSpan(body: string, anchor?: string): PlainSpan | null {
  let start = -1;
  if (body.startsWith(PLAIN_DELIMITER + '\n')) start = 0;
  const found = body.lastIndexOf('\n' + PLAIN_DELIMITER + '\n');
  if (found >= 0) start = found + 1;
  if (start < 0) return null;
  const quote = anchor === undefined || anchor === '' ? -1 : body.indexOf(anchor, start);
  return { start, end: quote >= 0 ? quote : body.length };
}

/** Whether a plain body holds a `-- ` signature. */
export function hasPlainSignature(body: string, anchor?: string): boolean {
  return plainSpan(body, anchor) !== null;
}

/** The text after the delimiter line, or null. */
export function plainSignatureText(body: string, anchor?: string): string | null {
  const span = plainSpan(body, anchor);
  return span === null ? null : body.slice(span.start + PLAIN_DELIMITER.length + 1, span.end).replace(/\s+$/, '');
}

const join = (head: string, tail: string): string => (head.trim() === '' ? tail : tail.trim() === '' ? head : `${head}\n\n${tail}`);

export function removeSignaturePlain(body: string, anchor?: string): string {
  const span = plainSpan(body, anchor);
  if (span === null) return body;
  return join(body.slice(0, span.start).replace(/\s+$/, ''), body.slice(span.end));
}

/** `body` with `signature` applied as the `-- ` block: replaced where it stands, else added by position. */
export function applySignaturePlain(body: string, signature: string, opts: PlaceOptions): string {
  const block = `${PLAIN_DELIMITER}\n${signature.replace(/\s+$/, '')}`;
  const span = opts.knownText ? plainSpan(body, opts.quoteAnchor) : null;
  if (span !== null) {
    const head = body.slice(0, span.start).replace(/\s+$/, ''), tail = body.slice(span.end);
    return tail.trim() === '' ? join(head, block) : join(join(head, block), tail);
  }
  const anchor = opts.quoteAnchor === undefined || opts.quoteAnchor === '' ? -1 : body.indexOf(opts.quoteAnchor);
  if (opts.position === 'above-quote' && anchor >= 0) {
    const head = body.slice(0, anchor).replace(/\s+$/, '');
    return `${head === '' ? '' : `${head}\n\n`}${block}\n\n${body.slice(anchor)}`;
  }
  return join(body.replace(/\s+$/, ''), block);
}
