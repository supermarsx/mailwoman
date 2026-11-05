import { isMailbox, parseRecipients, splitAddressList } from '../../api/jmap.ts';
import { recipientToken } from '../../modules/contacts/autocomplete.ts';

export type RecipientSeparator = 'comma' | 'semicolon';
export function isRecipientSeparator(value: unknown): value is RecipientSeparator {
  return value === 'comma' || value === 'semicolon';
}
const sepChar = (sep: RecipientSeparator): string => (sep === 'semicolon' ? ';' : ',');

export type RecipientTrigger = 'space' | 'comma' | 'semicolon' | 'blur' | 'enter';

/** Whether `text` is exactly one deliverable recipient: `a@b.c` or `Name <a@b.c>`. */
export function isCompleteRecipient(text: string): boolean {
  const trimmed = text.trim();
  if (trimmed === '') return false;
  if (splitAddressList(trimmed).length !== 1) return false;
  const address = parseRecipients(trimmed)[0];
  if (!address || !isMailbox(address.email)) return false;
  return trimmed.includes('<') || !/\s/u.test(trimmed);
}

/**
 * Turn the recipient being typed into a finished one. `caret` is the caret after the
 * trigger character was typed (for `space`, `comma` and `semicolon`) or the caret
 * itself (for `blur` and `enter`). Returns the new field text with the configured
 * separator inserted, or null when nothing should change (the text is not yet a
 * valid recipient, so it stays as typed).
 */
export function commitRecipient(value: string, caret: number, trigger: RecipientTrigger, sep: RecipientSeparator): { value: string; caret: number } | null {
  const typed = trigger === 'space' || trigger === 'comma' || trigger === 'semicolon';
  const end = typed ? Math.max(0, caret - 1) : caret;
  const token = recipientToken(value, end);
  const text = value.slice(token.start, typed ? end : token.end).trim();
  if (!isCompleteRecipient(text)) return null;
  const head = value.slice(0, token.start).replace(/\s+$/u, '') + (token.start > 0 ? ' ' : '');
  const tail = value.slice(typed ? caret : token.end).replace(/^[\s,;]+/u, '');
  const lead = head + text + sepChar(sep) + ' ';
  const next = lead + tail;
  return next === value ? null : { value: next, caret: lead.length };
}

/** Pasted text with several addresses becomes a separated list; null when it is a single entry. */
export function splitPasted(text: string, sep: RecipientSeparator): string | null {
  const entries: string[] = [];
  for (const entry of splitAddressList(text.replace(/[\r\n\t]+/gu, ','))) {
    const trimmed = entry.trim();
    if (trimmed === '') continue;
    const parts = !trimmed.includes('<') && /\s/u.test(trimmed) ? trimmed.split(/\s+/u) : [trimmed];
    entries.push(...(parts.every(isMailbox) ? parts : [trimmed]));
  }
  if (entries.length < 2) return null;
  return entries.join(sepChar(sep) + ' ') + sepChar(sep) + ' ';
}

export interface RecipientChip { index: number; text: string; email: string; name: string | null; valid: boolean }
export function chipsOf(value: string): RecipientChip[] {
  return splitAddressList(value).flatMap((entry, index) => {
    const text = entry.trim();
    if (text === '') return [];
    const address = parseRecipients(text)[0]!;
    return [{ index, text, email: address.email, name: address.name, valid: isMailbox(address.email) }];
  });
}

/** Remove the entry at `index` (as numbered by `chipsOf`). */
export function removeRecipient(value: string, index: number, sep: RecipientSeparator): string {
  const kept = splitAddressList(value).filter((entry, i) => i !== index && entry.trim() !== '').map(entry => entry.trim());
  return kept.length === 0 ? '' : kept.join(sepChar(sep) + ' ') + sepChar(sep) + ' ';
}
