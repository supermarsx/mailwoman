// List markers that no mail client draws on its own (FORMAT).
//
// A dash, arrow or tick bullet, `1)`, `01.`, `1.1.1` and a checklist are drawn by
// the editors from a `data-mw-list` attribute on the list (and `data-mw-checked`
// on a checklist item). What goes out is plain: the list gets
// `list-style-type:none` and each item starts with its marker as text
// (`<span>1.1&nbsp;</span>`), which renders the same in Gmail, Outlook and Apple
// Mail. `materializeLists` makes that form; `dematerializeLists` reads it back
// into the attribute form when HTML is loaded, so a saved draft reopens as a
// list of the same type. Neither leaves a `data-mw-*` attribute behind in the
// sent form.

import { CHECK_OFF, CHECK_ON, listStyleDef, type ListMarker } from './formatModel.ts';

export const LIST_ATTRIBUTE = 'data-mw-list';
export const CHECKED_ATTRIBUTE = 'data-mw-checked';

const BLOCK_TAG = /^(P|DIV|H[1-6])$/;
const MARKERS: Record<string, ListMarker> = { '–': 'dash', '→': 'arrow', '✓': 'tick', [CHECK_OFF]: 'check', [CHECK_ON]: 'check' };

/** The text one item's marker shows. `path` is the item's outline number when the list is `legal`. */
export function markerText(marker: ListMarker, ordinal: number, checked = false, path = String(ordinal)): string {
  switch (marker) {
    case 'dash': return '–';
    case 'arrow': return '→';
    case 'tick': return '✓';
    case 'check': return checked ? CHECK_ON : CHECK_OFF;
    case 'paren': return `${ordinal})`;
    case 'zero': return `${String(ordinal).padStart(2, '0')}.`;
    case 'legal': return path;
  }
}

const items = (list: Element): HTMLElement[] => Array.from(list.children).filter((child): child is HTMLElement => child.tagName === 'LI');

/** Where an item's marker text goes: in front of its first block's text, else in front of the item's own. */
function markerHost(item: Element): Element {
  const first = item.firstElementChild;
  return first && BLOCK_TAG.test(first.tagName) && item.firstChild === first ? first : item;
}

/** Turn the attribute form into the sent form, inside `root`. */
export function materializeLists(root: ParentNode): void {
  const lists = Array.from(root.querySelectorAll<HTMLElement>(`ul[${LIST_ATTRIBUTE}],ol[${LIST_ATTRIBUTE}]`));
  const paths = new Map<Element, string>();
  for (const list of lists) {
    const id = list.getAttribute(LIST_ATTRIBUTE);
    list.removeAttribute(LIST_ATTRIBUTE);
    const marker = listStyleDef(id)?.marker;
    for (const item of items(list)) {
      if (marker === undefined) { item.removeAttribute(CHECKED_ATTRIBUTE); continue; }
    }
    if (marker === undefined) continue;
    list.style.setProperty('list-style-type', 'none');
    const start = list.tagName === 'OL' ? Number(list.getAttribute('start') ?? '1') || 1 : 1;
    if (list.tagName === 'OL') list.removeAttribute('start');
    const parentPath = list.parentElement?.tagName === 'LI' ? paths.get(list.parentElement) : undefined;
    items(list).forEach((item, index) => {
      const ordinal = start + index;
      const path = parentPath !== undefined && marker === 'legal' ? `${parentPath}.${ordinal}` : String(ordinal);
      paths.set(item, path);
      const checked = item.getAttribute(CHECKED_ATTRIBUTE) === 'true';
      item.removeAttribute(CHECKED_ATTRIBUTE);
      const span = list.ownerDocument.createElement('span');
      span.textContent = `${markerText(marker, ordinal, checked, path)}\u00a0`;
      const host = markerHost(item);
      host.insertBefore(span, host.firstChild);
    });
  }
}

function hasNoMarker(list: HTMLElement): boolean {
  if (list.style.listStyleType === 'none') return true;
  return /list-style-type\s*:\s*none/i.test(list.getAttribute('style') ?? '');
}

function dropNoMarker(list: HTMLElement): void {
  list.style.removeProperty('list-style-type');
  const text = list.getAttribute('style');
  if (text !== null) {
    const next = text.replace(/list-style-type\s*:\s*none\s*;?/gi, '').trim();
    if (next === '') list.removeAttribute('style');
    else list.setAttribute('style', next);
  }
}

/** The marker an item starts with: its text, and the span that holds it. */
function leadingMarker(item: Element): { span: Element; text: string } | null {
  const host = markerHost(item);
  const span = host.firstChild;
  if (!span || span.nodeType !== 1 || (span as Element).tagName !== 'SPAN') return null;
  const text = span.textContent ?? '';
  return text.endsWith('\u00a0') ? { span: span as Element, text: text.slice(0, -1) } : null;
}

function kindOf(text: string): ListMarker | null {
  if (MARKERS[text]) return MARKERS[text]!;
  if (/^\d{1,4}\)$/.test(text)) return 'paren';
  if (/^0\d{1,3}\.$/.test(text)) return 'zero';
  if (/^\d{1,4}(?:\.\d{1,4})*$/.test(text)) return 'legal';
  return null;
}

/** Turn the sent form into the attribute form, inside `root`. */
export function dematerializeLists(root: ParentNode): void {
  for (const list of Array.from(root.querySelectorAll<HTMLElement>('ul,ol'))) {
    if (list.hasAttribute(LIST_ATTRIBUTE) || !hasNoMarker(list)) continue;
    const entries = items(list);
    if (entries.length === 0) continue;
    const found = entries.map((item) => leadingMarker(item));
    if (found.some((entry) => entry === null)) continue;
    const kinds = new Set(found.map((entry) => kindOf(entry!.text)));
    const kind = [...kinds][0];
    if (kinds.size !== 1 || !kind) continue;
    // A `1`-style number needs the outline shape to be a list of ours: top-level items are `n` alone.
    dropNoMarker(list);
    list.setAttribute(LIST_ATTRIBUTE, kind);
    if (list.tagName === 'OL' && (kind === 'paren' || kind === 'zero' || kind === 'legal')) {
      const first = parseInt(found[0]!.text.split('.').pop() ?? '1', 10);
      if (Number.isFinite(first) && first > 1) list.setAttribute('start', String(first));
    }
    entries.forEach((item, index) => {
      if (kind === 'check') item.setAttribute(CHECKED_ATTRIBUTE, found[index]!.text === CHECK_ON ? 'true' : 'false');
      found[index]!.span.remove();
    });
  }
}
