import { describe, expect, it, vi } from 'vitest';
import { cardForAddress, canShareFile, contactShareDraft, contactSummary, contactVCard, shareVCardFile, vcardFileName } from './share.ts';
import { parseVCards, foldLine } from './vcard.ts';
import { defaultSeed } from './mockClient.ts';
import type { ContactCard } from '../../api/pim-types.ts';

const ada = (): ContactCard => ({
  ...defaultSeed().contacts[0]!,
  uid: 'urn:uuid:ada', titles: ['Analyst'], organizations: ['Analytical Engines'], phones: [{ context: 'work', value: '+44 20 7946 0000' }],
  notes: 'Likes semicolons; commas, and\nnew lines', isFavorite: true,
  addresses: [{ context: 'work', pobox: '', ext: '', street: '12 St James\'s Sq', locality: 'London', region: '', postcode: 'SW1Y 4JH', country: 'UK' }],
});

describe('contact vCard for sharing', () => {
  it('is a vCard 4.0 with identity, contact points, organisation, address, note and a REV stamp', () => {
    const out = contactVCard(ada(), new Date('2026-11-05T10:20:30Z'));
    expect(out.startsWith('BEGIN:VCARD\r\nVERSION:4.0\r\n')).toBe(true);
    expect(out.endsWith('END:VCARD\r\n')).toBe(true);
    for (const line of ['UID:urn:uuid:ada', 'FN:Ada Lovelace', 'N:Lovelace;Ada;;;', 'ORG:Analytical Engines', 'TITLE:Analyst', 'EMAIL;TYPE=work;PREF=1:ada@example.org', 'TEL;TYPE=work:+44 20 7946 0000', 'ADR;TYPE=work:;;12 St James\'s Sq;London;;SW1Y 4JH;UK', 'REV:20261105T102030Z']) expect(out).toContain(`\r\n${line}\r\n`);
    expect(out).toContain('NOTE:Likes semicolons\\; commas\\, and\\nnew lines');
    const [back] = parseVCards(out);
    expect(back?.name.full).toBe('Ada Lovelace'); expect(back?.addresses).toHaveLength(1); expect(back?.notes).toBe(ada().notes);
  });
  it('does not export the local favorite flag to whoever receives it', () => {
    expect(contactVCard(ada())).not.toContain('CATEGORIES');
    expect(parseVCards(contactVCard(ada()))[0]?.isFavorite).toBe(false);
  });
  it('gives a card without a UID one, so the receiving address book can merge a re-share', () => {
    expect(contactVCard({ ...ada(), uid: '' })).toMatch(/\r\nUID:urn:uuid:[0-9a-f-]{36}\r\n/);
  });
  it('escapes a value so it can never start a new property, and folds multi-byte text on octet limits', () => {
    const out = contactVCard({ ...ada(), notes: 'x\r\nEND:VCARD\r\nBEGIN:VCARD', name: { ...ada().name, full: 'Zoë 😀 '.repeat(20) } });
    expect(out.split('\r\n').filter((line) => line === 'END:VCARD')).toHaveLength(1);
    const encoder = new TextEncoder();
    for (const line of out.split('\r\n')) { expect(encoder.encode(line).length).toBeLessThanOrEqual(75); expect(line).not.toContain('�'); }
    expect(parseVCards(out)[0]?.name.full).toBe('Zoë 😀 '.repeat(20).trim());
  });
  it('folds only past 75 octets and keeps a short line whole', () => {
    expect(foldLine('FN:short')).toBe('FN:short');
    expect(foldLine(`FN:${'é'.repeat(60)}`).split('\r\n ').every((part) => new TextEncoder().encode(part).length <= 75)).toBe(true);
  });
  it('builds a card from a bare address and display name, and falls back to the address as the name', () => {
    const named = cardForAddress('Bob@Example.org', 'Bob Builder');
    expect(named.name.full).toBe('Bob Builder'); expect(named.emails).toEqual([{ context: '', value: 'Bob@example.org', pref: 1 }]);
    expect(cardForAddress('bob@example.org').name.full).toBe('bob@example.org');
    const out = contactVCard(named);
    expect(out).toContain('FN:Bob Builder'); expect(out).toContain('EMAIL;PREF=1:Bob@example.org');
  });
  it('names the file after the contact without characters a file system refuses', () => {
    expect(vcardFileName('Ada Lovelace')).toBe('Ada Lovelace.vcf');
    expect(vcardFileName('a/b\\c:d*e?"f<g>h|i')).toBe('a b c d e f g h i.vcf');
    expect(vcardFileName('   ')).toBe('contact.vcf');
    expect(vcardFileName('x'.repeat(200))).toHaveLength(84);
  });
});

describe('share message draft', () => {
  it('summarises the contact in the body and attaches the file when it was stored', () => {
    const file = { name: 'Ada Lovelace.vcf', blobId: 'b1', size: 10, contentType: 'text/vcard' };
    const draft = contactShareDraft(ada(), file);
    expect(draft).toMatchObject({ mode: 'new', to: '', subject: 'Ada Lovelace', attachments: [file], quotesDecrypted: false });
    expect(draft.bodyText).toBe('Ada Lovelace\nAnalyst, Analytical Engines\nada@example.org\n+44 20 7946 0000');
    expect(contactSummary(ada())).toBe(draft.bodyText);
  });
  it('carries the whole vCard in the text when nothing could be attached', () => {
    const draft = contactShareDraft(ada(), null);
    expect(draft.attachments).toEqual([]);
    expect(draft.bodyText).toContain('Ada Lovelace\nAnalyst');
    expect(draft.bodyText).toContain('\n\nBEGIN:VCARD\nVERSION:4.0\n');
    expect(draft.bodyText).toContain('END:VCARD');
  });
});

describe('system share sheet', () => {
  const file = new File(['x'], 'a.vcf', { type: 'text/vcard' });
  it('needs navigator.share, and canShare must accept the file', () => {
    expect(canShareFile(undefined, file)).toBe(false);
    expect(canShareFile({} as Navigator, file)).toBe(false);
    expect(canShareFile({ share: vi.fn() } as unknown as Navigator, file)).toBe(true);
    expect(canShareFile({ share: vi.fn(), canShare: () => false } as unknown as Navigator, file)).toBe(false);
  });
  it('reports shared, cancelled, failed and unsupported distinctly', async () => {
    const make = (share: () => Promise<void>) => ({ share, canShare: () => true }) as unknown as Navigator;
    expect(await shareVCardFile(make(async () => undefined), 'a.vcf', 'v', 't', 'x')).toBe('shared');
    expect(await shareVCardFile(make(() => Promise.reject(new DOMException('no', 'AbortError'))), 'a.vcf', 'v', 't', 'x')).toBe('cancelled');
    expect(await shareVCardFile(make(() => Promise.reject(new Error('boom'))), 'a.vcf', 'v', 't', 'x')).toBe('failed');
    expect(await shareVCardFile(undefined, 'a.vcf', 'v', 't', 'x')).toBe('unsupported');
  });
});
