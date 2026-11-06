// In-place patching of the message document (no reload, no flicker).
//
// Toggling "Show images", "Retry", or cancelling image loading only changes
// image-bearing attributes of the already sanitized document (src, srcset,
// background, style and <style> text). The parent computes those differences
// here and posts them to the bridge, which relays them over the nonce-bound
// channel to the message document's fixed, hash-authorized handler. The handler
// re-validates every operation with the same rules (reader-address-handler.js
// carries a copy), so a patch can only ever point an image at one of this
// server's signed /api/rc/{i,p}/ handles or at an inline data: image. Anything
// else in the document differing (text, links, structure) returns `null` and
// the caller falls back to a full, double-buffered re-render.

export type PatchAttr = 'src' | 'srcset' | 'background' | 'style';
export interface PatchOp { s: 'b' | 'h'; i: number; a: PatchAttr | 'text'; v: string | null }
export const PATCH_MAX_OPS = 4096;
const PATCH_ATTRS: readonly PatchAttr[] = ['src', 'srcset', 'background', 'style'];
const DATA_IMAGE = /^data:image\/(?:gif|png|jpeg|webp|avif);base64,[A-Za-z0-9+/=]*$/;
// A NUL byte and a backslash are what this rejects.
// eslint-disable-next-line no-control-regex
const CSS_FORBIDDEN = /[\x00\x5c]|@import|image-set|cross-fade|-webkit-image|element\(|\bimage\(|expression\(/i;

/** Signed-handle prefixes (`…/api/rc/i/`, `…/api/rc/p/`) for a server base URL. */
export function patchPrefixes(base: string): string[] {
  const root = base.replace(/\/$/, '') + '/api/rc/';
  return [root + 'i/', root + 'p/'];
}
export function patchUrlAllowed(raw: unknown, prefixes: readonly string[]): boolean {
  if (typeof raw !== 'string' || raw.length > 4096) return false;
  if (raw.length > 5 && raw.startsWith('data:')) return DATA_IMAGE.test(raw);
  return prefixes.some(prefix => raw.startsWith(prefix) && /^[A-Za-z0-9_.-]{1,512}$/.test(raw.slice(prefix.length)));
}
function cssAllowed(value: string, prefixes: readonly string[]): boolean {
  if (value.length > 65536 || CSS_FORBIDDEN.test(value)) return false;
  const all = value.match(/url\s*\(/gi)?.length ?? 0;
  let seen = 0, ok = true;
  value.replace(/url\(\s*(?:"([^"]*)"|'([^']*)'|([^)"'\s]*))\s*\)/gi, (_m, a: string | undefined, b: string | undefined, c: string | undefined) => { seen++; ok = ok && patchUrlAllowed(a ?? b ?? c ?? '', prefixes); return ''; });
  return ok && seen === all;
}
export function patchValueAllowed(attr: string, value: unknown, prefixes: readonly string[]): boolean {
  if (value === null) return attr !== 'text';
  if (typeof value !== 'string') return false;
  if (attr === 'src' || attr === 'background') return patchUrlAllowed(value, prefixes);
  if (attr === 'srcset') {
    const parts = value.split(',').map(part => part.trim());
    return parts.length > 0 && parts.length <= 64 && parts.every(part => { const [url, descriptor, ...rest] = part.split(/\s+/); return rest.length === 0 && patchUrlAllowed(url, prefixes) && (descriptor === undefined || /^\d+(?:\.\d+)?[wx]$/.test(descriptor)); });
  }
  if (attr === 'style' || attr === 'text') return cssAllowed(value, prefixes);
  return false;
}
export function patchOpAllowed(op: unknown, prefixes: readonly string[]): op is PatchOp {
  if (!op || typeof op !== 'object') return false;
  const { s, i, a, v } = op as Record<string, unknown>;
  return (s === 'b' || s === 'h') && Number.isSafeInteger(i) && (i as number) >= 0 && (i as number) < 1_000_000
    && (a === 'text' ? true : PATCH_ATTRS.includes(a as PatchAttr)) && (s === 'b' || a === 'text') && patchValueAllowed(a as string, v, prefixes);
}

interface Slot { s: 'b' | 'h'; el: Element; attrs: Partial<Record<PatchAttr, string>>; text?: string }
/** Strips patchable values out of `doc` (in place) and returns them by slot. */
function neutralise(doc: Document): Slot[] {
  const slots: Slot[] = [];
  const take = (s: 'b' | 'h', el: Element): void => {
    const slot: Slot = { s, el, attrs: {} };
    if (s === 'b') for (const attr of PATCH_ATTRS) { const value = el.getAttribute(attr); if (value !== null) { slot.attrs[attr] = value; el.removeAttribute(attr); } }
    if (el.tagName === 'STYLE') { slot.text = el.textContent ?? ''; el.textContent = ''; }
    slots.push(slot);
  };
  doc.head.querySelectorAll('style').forEach(el => take('h', el));
  doc.body.querySelectorAll('*').forEach(el => take('b', el));
  return slots;
}

/**
 * Operations turning `oldHtml` into `newHtml`, or `null` when the documents
 * differ in anything but patchable image references (or a new value would not be
 * accepted by the document's own validation). `[]` means nothing to change.
 */
export function diffReaderDocument(oldHtml: string, newHtml: string, prefixes: readonly string[]): PatchOp[] | null {
  if (oldHtml === newHtml) return [];
  if (oldHtml.length > 8 * 1024 * 1024 || newHtml.length > 8 * 1024 * 1024) return null;
  const parser = new DOMParser();
  const a = parser.parseFromString(oldHtml, 'text/html'), b = parser.parseFromString(newHtml, 'text/html');
  const left = neutralise(a), right = neutralise(b);
  if (left.length !== right.length || a.documentElement.outerHTML !== b.documentElement.outerHTML) return null;
  const ops: PatchOp[] = [];
  let hi = 0, bi = 0;
  for (let n = 0; n < left.length; n++) {
    const before = left[n]!, after = right[n]!;
    const index = after.s === 'h' ? hi++ : bi++;
    for (const attr of PATCH_ATTRS) {
      const was = before.attrs[attr] ?? null, now = after.attrs[attr] ?? null;
      if (was !== now) ops.push({ s: after.s, i: index, a: attr, v: now });
    }
    if (before.text !== after.text) ops.push({ s: after.s, i: index, a: 'text', v: after.text ?? '' });
  }
  if (ops.length > PATCH_MAX_OPS || !ops.every(op => patchOpAllowed(op, prefixes))) return null;
  return ops;
}
