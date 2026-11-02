import { isMailbox } from '../../api/jmap.ts';
/** Exact mailbox identity: local-part bytes remain intact; DNS is case-insensitive. */
export function normalizeContactAddress(value: string): string|null {
  const address = value.trim();
  if (!isMailbox(address)) return null;
  const at = address.lastIndexOf('@');
  const domain = address.slice(at + 1);
  return address.slice(0, at + 1) + (domain.startsWith('[') ? domain : domain.toLowerCase());
}
/** A contact action concerns one recipient, never mailto query/header actions. */
export function contactAddressFromMailto(value: string): string|null {
  if (!/^mailto:/i.test(value)) return null;
  try { return normalizeContactAddress(decodeURIComponent(value.slice(7).split(/[?#]/, 1)[0]!)); } catch { return null; }
}
/** Resolve only the token under a caret, not another address in its paragraph. */
export function emailAtTextPoint(text: string, offset: number): string|null {
  if (!Number.isSafeInteger(offset) || offset < 0 || offset > text.length) return null;
  const start = Math.max(0, offset - 256), end = Math.min(text.length, offset + 256);
  const input = text.slice(start, end);
  const candidates = /(?:"(?:[^"\\\r\n]|\\.)+"|[^\s<>()[\],;:"\\]+)@(?:\[[^\]\s]+\]|[^\s<>()[\],;:"\\]+)/gu;
  for (const match of input.matchAll(candidates)) {
    const candidate = match[0].replace(/[.!?]+$/, '');
    const left = start + match.index;
    if (offset < left || offset > left + candidate.length) continue;
    // URL user-info is not a plain email target, despite '/' being legal in an
    // SMTP local part. Keep the existing web link's context actions intact.
    if (candidate.startsWith('//') && left > 0 && text[left - 1] === ':') continue;
    const address = normalizeContactAddress(candidate);
    if (address) return address;
  }
  return null;
}
