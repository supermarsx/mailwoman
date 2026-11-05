import { isMailbox, parseRecipients, splitAddressList } from '../../api/jmap.ts';

export type RecipientSeparator = 'comma' | 'semicolon';
export function isRecipientSeparator(value: unknown): value is RecipientSeparator {
  return value === 'comma' || value === 'semicolon';
}
export const sepChar = (sep: RecipientSeparator): string => (sep === 'semicolon' ? ';' : ',');

/** Whether `text` is exactly one deliverable recipient: `a@b.c` or `Name <a@b.c>`. */
export function isCompleteRecipient(text: string): boolean {
  const trimmed = text.trim();
  if (trimmed === '') return false;
  if (splitAddressList(trimmed).length !== 1) return false;
  const address = parseRecipients(trimmed)[0];
  if (!address || !isMailbox(address.email)) return false;
  return trimmed.includes('<') || !/\s/u.test(trimmed);
}

/** One finished recipient as the field model holds it. `text` is what is serialised. */
export interface Recipient { text: string; email: string; name: string | null; valid: boolean }

export function recipientOf(text: string): Recipient {
  const trimmed = text.trim();
  const address = parseRecipients(trimmed)[0];
  const email = address?.email ?? trimmed;
  return { text: trimmed, email, name: address?.name ?? null, valid: isMailbox(email) };
}

/** The non-empty entries of an address list, trimmed. */
export function entriesOf(value: string): string[] {
  return splitAddressList(value).map(entry => entry.trim()).filter(entry => entry !== '');
}

/** Finished recipients followed by the text still being typed (the draft). */
export function serializeRecipients(list: readonly { text: string }[], draft: string, sep: RecipientSeparator): string {
  const head = list.length === 0 ? '' : `${list.map(item => item.text).join(`${sepChar(sep)} `)}${sepChar(sep)} `;
  return head + draft;
}

/** Pasted text as separate entries (comma, semicolon, newline or whitespace separated, `Name <addr>` kept whole). */
export function splitPastedEntries(text: string): string[] {
  const entries: string[] = [];
  for (const entry of splitAddressList(text.replace(/[\r\n\t]+/gu, ','))) {
    const trimmed = entry.trim();
    if (trimmed === '') continue;
    const parts = !trimmed.includes('<') && /\s/u.test(trimmed) ? trimmed.split(/\s+/u) : [trimmed];
    entries.push(...(parts.every(isMailbox) ? parts : [trimmed]));
  }
  return entries;
}
