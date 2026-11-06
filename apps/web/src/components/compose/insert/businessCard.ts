// Business card for the Insert tab: a compact, mail-safe HTML card (a table with
// inline styles only, no images unless the caller passes a cid) and the vCard 4.0
// that goes with it. The vCard reuses the contacts share builder.

import type { ContactCard } from '../../../api/pim-types.ts';
import { cardForAddress, contactVCard, vcardFileName } from '../../../modules/contacts/share.ts';

export type CardMode = 'both' | 'card' | 'vcard';

export interface CardFields {
  name: string;
  title: string;
  org: string;
  phone: string;
  email: string;
  address: string;
  /** A `cid:` source for the photo, when the host attached one. */
  photoSrc?: string;
}

export const emptyCard = (): CardFields => ({ name: '', title: '', org: '', phone: '', email: '', address: '' });

const esc = (s: string): string =>
  s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');

/** The card's fields from a saved contact. */
export function fieldsFromContact(card: ContactCard): CardFields {
  const address = card.addresses[0];
  const parts = address === undefined ? [] : [address['street'], address['locality'], address['postcode'], address['country']];
  const formatted = address === undefined
    ? ''
    : String(address['full'] ?? address['formatted'] ?? parts.filter((v) => typeof v === 'string' && v !== '').join(', '));
  return {
    name: card.name.full,
    title: card.titles[0] ?? '',
    org: card.organizations[0] ?? '',
    phone: card.phones[0]?.value ?? '',
    email: card.emails[0]?.value ?? '',
    address: formatted,
  };
}

/** The card's fields from the sending identity. */
export function fieldsFromIdentity(identity: { name: string; email: string; organization?: string | null } | null): CardFields {
  return { ...emptyCard(), name: identity?.name ?? '', email: identity?.email ?? '', org: identity?.organization ?? '' };
}

/** A ContactCard built from the dialog's fields (for the vCard). */
export function contactFromFields(fields: CardFields): ContactCard {
  const base = cardForAddress(fields.email || 'unknown@invalid', fields.name);
  return {
    ...base,
    emails: fields.email ? base.emails : [],
    titles: fields.title ? [fields.title] : [],
    organizations: fields.org ? [fields.org] : [],
    phones: fields.phone ? [{ context: '', value: fields.phone }] : [],
    addresses: fields.address ? [{ full: fields.address }] : [],
  };
}

export function businessCardVcf(fields: CardFields, now: Date = new Date()): { name: string; text: string; type: string } {
  return { name: vcardFileName(fields.name || fields.email), text: contactVCard(contactFromFields(fields), now), type: 'text/vcard' };
}

const FONT = 'font-family:Arial,Helvetica,sans-serif';

/** The visual card: one table, inline styles, readable in light and dark clients. */
export function businessCardHtml(fields: CardFields): string {
  const row = (html: string, style: string): string => (html === '' ? '' : `<tr><td style="${style}">${html}</td></tr>`);
  const photo = fields.photoSrc
    ? `<td style="padding:12px 0 12px 12px;vertical-align:top" width="64"><img src="${esc(fields.photoSrc)}" alt="" width="64" height="64" style="display:block;border-radius:32px"></td>`
    : '';
  const lines = [
    row(fields.name ? `<strong>${esc(fields.name)}</strong>` : '', `${FONT};font-size:16px;line-height:22px`),
    row(esc([fields.title, fields.org].filter(Boolean).join(', ')), `${FONT};font-size:13px;line-height:18px;color:#555555`),
    row(fields.phone ? `<a href="tel:${esc(fields.phone.replace(/[^+\d]/g, ''))}" style="color:#1a56b0;text-decoration:none">${esc(fields.phone)}</a>` : '', `${FONT};font-size:13px;line-height:18px;padding-top:6px`),
    row(fields.email ? `<a href="mailto:${esc(fields.email)}" style="color:#1a56b0;text-decoration:none">${esc(fields.email)}</a>` : '', `${FONT};font-size:13px;line-height:18px`),
    row(esc(fields.address), `${FONT};font-size:12px;line-height:17px;color:#555555;padding-top:2px`),
  ].join('');
  return `<table role="presentation" data-mw-card="1" cellpadding="0" cellspacing="0" border="0" style="border:1px solid #c8c8c8;border-radius:6px;background:#ffffff;color:#222222;max-width:420px"><tr>${photo}<td style="padding:12px 16px"><table role="presentation" cellpadding="0" cellspacing="0" border="0">${lines}</table></td></tr></table>`;
}

/** Plain-text form for the plain body mode. */
export function businessCardText(fields: CardFields): string {
  return [fields.name, [fields.title, fields.org].filter(Boolean).join(', '), fields.phone, fields.email, fields.address].filter((v) => v !== '').join('\n');
}
