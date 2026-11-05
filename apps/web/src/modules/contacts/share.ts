// Sharing a contact: the vCard 4.0 (RFC 6350) of a saved card or of a bare
// address, the short text summary that goes in a message body, and the three
// ways out of the browser (clipboard, a .vcf download, the Web Share API).
// Nothing here touches the network; attaching the file to a message is the
// caller's job (`uploadComposeBlob` in the contacts slice).

import type { ContactCard } from '../../api/pim-types.ts';
import type { ComposeInitial } from '../../components/compose/reply.ts';
import { normalizeContactAddress } from './contactAddress.ts';
import { toVCard } from './vcard.ts';

export const VCARD_MIME = 'text/vcard';

/** A contact card for an address that is not saved: only what the message told us. */
export function cardForAddress(address: string, name = ''): ContactCard {
  const email = normalizeContactAddress(address) ?? address.trim();
  const full = name.trim() || email;
  return {
    id: '', addressBookId: '', uid: `urn:uuid:${crypto.randomUUID()}`, kind: 'individual',
    name: { full, given: '', surname: '', prefix: '', suffix: '' },
    nicknames: [], organizations: [], titles: [], emails: [{ context: '', value: email, pref: 1 }], phones: [], onlineServices: [],
    addresses: [], anniversaries: [], notes: '', photoBlobId: null, isFavorite: false, groupIds: [], pgpKey: null, smimeCert: null, etag: null,
  };
}

/** The vCard of a card, CRLF-terminated, with a `REV` stamp. The favorite flag is a local choice and is not shared. */
export function contactVCard(card: ContactCard, now: Date = new Date()): string {
  const outgoing = card.uid.length > 0 ? card : { ...card, uid: `urn:uuid:${crypto.randomUUID()}` };
  return `${toVCard({ ...outgoing, isFavorite: false }, { rev: now })}\r\n`;
}

/** `Ada Lovelace` -> `Ada Lovelace.vcf`; characters a file system refuses are dropped. */
export function vcardFileName(name: string): string {
  // eslint-disable-next-line no-control-regex
  const clean = name.replace(/[\u0000-\u001f\\/:*?"<>|]+/g, ' ').replace(/\s+/g, ' ').trim().slice(0, 80);
  return `${clean.length > 0 ? clean : 'contact'}.vcf`;
}

/** The lines of the plain-text summary: name, role, then each address and number. */
export function contactSummary(card: ContactCard): string {
  const lines = [card.name.full || card.emails[0]?.value || ''];
  const role = [card.titles[0], card.organizations[0]].filter((value): value is string => !!value).join(', ');
  if (role) lines.push(role);
  for (const email of card.emails) lines.push(email.value);
  for (const phone of card.phones) lines.push(phone.value);
  return lines.filter((line) => line.length > 0).join('\n');
}

/** A composer opened on a contact: the summary in the body, the .vcf attached when it could be stored. */
export function contactShareDraft(card: ContactCard, attachment: ComposeInitial['attachments'][number] | null): ComposeInitial {
  const summary = contactSummary(card);
  const name = card.name.full || card.emails[0]?.value || '';
  return {
    mode: 'new', to: '', cc: '', subject: name, bodyHtml: '',
    bodyText: attachment ? summary : `${summary}\n\n${contactVCard(card).replace(/\r\n/g, '\n')}`,
    inReplyTo: [], references: [], attachments: attachment ? [attachment] : [], quotesDecrypted: false,
  };
}

function legacyCopy(doc: Document, text: string): boolean {
  try {
    const area = doc.createElement('textarea');
    area.value = text; area.setAttribute('readonly', ''); area.style.position = 'fixed'; area.style.opacity = '0';
    doc.body.append(area); area.select();
    const ok = typeof doc.execCommand === 'function' && doc.execCommand('copy');
    area.remove();
    return ok;
  } catch { return false; }
}

/** Put `text` on the clipboard, falling back to a selection copy where the async API is missing or refused. */
export async function copyText(doc: Document, text: string): Promise<boolean> {
  try { await (doc.defaultView?.navigator ?? navigator).clipboard.writeText(text); return true; } catch { return legacyCopy(doc, text); }
}

/** Offer `text` as a file download named `filename`. */
export function downloadText(doc: Document, filename: string, text: string, mime = VCARD_MIME): void {
  const view = doc.defaultView ?? window;
  const url = view.URL.createObjectURL(new Blob([text], { type: `${mime};charset=utf-8` }));
  const link = doc.createElement('a');
  link.href = url; link.download = filename; link.rel = 'noopener'; link.style.display = 'none';
  doc.body.append(link); link.click(); link.remove();
  setTimeout(() => view.URL.revokeObjectURL(url), 0);
}

/** Whether the Web Share API can send this file here (it can on most phones, few desktops). */
export function canShareFile(nav: Navigator | undefined, file: File): boolean {
  return !!nav && typeof nav.share === 'function' && (typeof nav.canShare !== 'function' || nav.canShare({ files: [file] }));
}

export type ShareResult = 'shared' | 'cancelled' | 'unsupported' | 'failed';

/** Hand the .vcf to the system share sheet. A dismissed sheet is `cancelled`, not an error. */
export async function shareVCardFile(nav: Navigator | undefined, filename: string, vcf: string, title: string, text: string): Promise<ShareResult> {
  const file = new File([vcf], filename, { type: VCARD_MIME });
  if (!nav || !canShareFile(nav, file)) return 'unsupported';
  try { await nav.share({ files: [file], title, text }); return 'shared'; }
  catch (error) { return error instanceof DOMException && error.name === 'AbortError' ? 'cancelled' : 'failed'; }
}
