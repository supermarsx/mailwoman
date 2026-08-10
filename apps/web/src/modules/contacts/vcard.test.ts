// vCard 3.0/4.0 reader + writer (t19-e12, tag 26.19).
//
// `vcard.ts` had no test before this file. It parses UNTRUSTED input — contact
// files a user drags in, and CardDAV payloads from a foreign server — so the
// cases that matter are the malformed and adversarial ones as much as the happy
// path. It stays deliberately lenient (unknown properties ignored, 3.0 and 4.0
// spellings both accepted); "lenient" is a design choice that needs pinning,
// because the alternative reading is "silently drops data", and the two are only
// distinguishable by saying which is which.
//
// Two round-trip defects are RECORDED rather than fixed — `vcard.ts` is outside
// this lane's locks. Both are called out inline and in the lane log.

import { describe, it, expect } from 'vitest';
import { parseVCards, toVCard, toVCardDocument, birthday } from './vcard.ts';
import type { ContactCard } from '../../api/pim-types.ts';

/** A full card, so a round-trip exercises every emitted property. */
function card(over: Partial<ContactCard> = {}): ContactCard {
  return {
    id: 'c1',
    addressBookId: 'ab1',
    uid: 'urn-less-uid',
    kind: 'individual',
    name: { full: 'Ada Lovelace', given: 'Ada', surname: 'Lovelace', prefix: 'Dr', suffix: 'PhD' },
    nicknames: [],
    organizations: [],
    titles: [],
    emails: [],
    phones: [],
    onlineServices: [],
    addresses: [],
    anniversaries: [],
    notes: '',
    photoBlobId: null,
    isFavorite: false,
    groupIds: [],
    pgpKey: null,
    smimeCert: null,
    etag: null,
    ...over,
  };
}

const MINIMAL = ['BEGIN:VCARD', 'VERSION:4.0', 'FN:Ada Lovelace', 'END:VCARD'].join('\r\n');

describe('parseVCards — structure', () => {
  it('reads a minimal card', () => {
    const [c] = parseVCards(MINIMAL);
    expect(c?.name.full).toBe('Ada Lovelace');
    expect(c?.kind).toBe('individual');
    expect(c?.emails).toEqual([]);
  });

  it('reads several cards from one document', () => {
    const doc = `${MINIMAL}\r\n${MINIMAL.replace('Ada Lovelace', 'Grace Hopper')}`;
    expect(parseVCards(doc).map((c) => c.name.full)).toEqual(['Ada Lovelace', 'Grace Hopper']);
  });

  it('accepts CRLF, bare CR and bare LF line endings alike', () => {
    // Exports in the wild use all three; a reader that only handled CRLF would
    // return a single unusable card from a Mac-classic or Unix-written file.
    for (const eol of ['\r\n', '\n', '\r']) {
      const doc = MINIMAL.replace(/\r\n/g, eol);
      expect(parseVCards(doc)).toHaveLength(1);
    }
  });

  it('unfolds RFC 6350 continuation lines, space- or tab-prefixed', () => {
    const folded = 'BEGIN:VCARD\r\nFN:Ada Lo\r\n vela\r\n\tce\r\nEND:VCARD';
    expect(parseVCards(folded)[0]?.name.full).toBe('Ada Lovelace');
  });

  it('strips a property grouping prefix', () => {
    // Apple Contacts emits `item1.EMAIL` + `item1.X-ABLabel`. Without the strip,
    // the property name never matches and the email is silently dropped.
    const doc = 'BEGIN:VCARD\r\nFN:A\r\nitem1.EMAIL:a@example.com\r\nEND:VCARD';
    expect(parseVCards(doc)[0]?.emails[0]?.value).toBe('a@example.com');
  });
});

describe('parseVCards — malformed and hostile input', () => {
  it('returns nothing for input that is not a vCard at all', () => {
    expect(parseVCards('')).toEqual([]);
    expect(parseVCards('just some text\r\nwith no colon')).toEqual([]);
    expect(parseVCards('{"json":true}')).toEqual([]);
  });

  it('drops a card with no END, rather than emitting a half-read one', () => {
    // Truncated file: better to import nothing than to import a partial contact
    // the user then has to notice is wrong.
    expect(parseVCards('BEGIN:VCARD\r\nFN:Ada\r\nEMAIL:a@example.com')).toEqual([]);
  });

  it('ignores properties that appear before any BEGIN', () => {
    expect(parseVCards('FN:Nobody\r\nBEGIN:VCARD\r\nFN:Ada\r\nEND:VCARD')[0]?.name.full).toBe('Ada');
  });

  it('ignores unknown and X- properties instead of failing', () => {
    const doc = [
      'BEGIN:VCARD',
      'FN:Ada',
      'X-MADE-UP-THING:whatever',
      'PRODID:-//Example//EN',
      'REV:20260810T000000Z',
      'END:VCARD',
    ].join('\r\n');
    const [c] = parseVCards(doc);
    expect(c?.name.full).toBe('Ada');
    expect(c?.notes).toBe('');
  });

  it('survives a value containing colons and semicolons', () => {
    const doc = 'BEGIN:VCARD\r\nFN:A\r\nNOTE:see http://example.com/a:b\r\nEND:VCARD';
    // Only the FIRST colon separates name from value — the rest belong to it.
    expect(parseVCards(doc)[0]?.notes).toBe('see http://example.com/a:b');
  });

  it('does not let a property name collide with Object.prototype', () => {
    // `params` is a bare-ish record built from attacker-controlled keys; a
    // `__proto__` param must not reach the prototype chain.
    const doc = 'BEGIN:VCARD\r\nFN:A\r\nEMAIL;__proto__=polluted:a@example.com\r\nEND:VCARD';
    const [c] = parseVCards(doc);
    expect(c?.emails[0]?.value).toBe('a@example.com');
    expect(({} as Record<string, unknown>)['polluted']).toBeUndefined();
    expect(Object.prototype).not.toHaveProperty('polluted');
  });
});

describe('parseVCards — values', () => {
  it('unescapes TEXT escapes, including the uppercase \\N form', () => {
    const doc = 'BEGIN:VCARD\r\nFN:A\r\nNOTE:line1\\nline2\\Nline3 a\\, b\\; c \\\\ d\r\nEND:VCARD';
    expect(parseVCards(doc)[0]?.notes).toBe('line1\nline2\nline3 a, b; c \\ d');
  });

  it('splits the structured N property into its components', () => {
    const doc = 'BEGIN:VCARD\r\nN:Lovelace;Ada;Augusta;Dr;PhD\r\nEND:VCARD';
    const [c] = parseVCards(doc);
    expect(c?.name).toMatchObject({ surname: 'Lovelace', given: 'Ada', prefix: 'Dr', suffix: 'PhD' });
    // FN absent → derived from N in prefix/given/surname/suffix order.
    expect(c?.name.full).toBe('Dr Ada Lovelace PhD');
  });

  it('falls back to the organisation when neither FN nor N names anyone', () => {
    const doc = 'BEGIN:VCARD\r\nKIND:org\r\nORG:Analytical Engines;Research\r\nEND:VCARD';
    const [c] = parseVCards(doc);
    expect(c?.kind).toBe('org');
    expect(c?.organizations).toEqual(['Analytical Engines · Research']);
    expect(c?.name.full).toBe('Analytical Engines · Research');
  });

  it('reads TYPE params in both vCard 2.1 and 3.0/4.0 spellings', () => {
    const doc = [
      'BEGIN:VCARD',
      'FN:A',
      'EMAIL;TYPE=WORK:work@example.com', // 3.0/4.0
      'EMAIL;HOME:home@example.com', // 2.1 bare param
      'TEL;TYPE=CELL:+15551234', // CELL is normalised to `mobile`
      'END:VCARD',
    ].join('\r\n');
    const [c] = parseVCards(doc);
    expect(c?.emails.map((e) => e.context)).toEqual(['work', 'home']);
    expect(c?.phones[0]?.context).toBe('mobile');
  });

  it('reads PREF from both the 4.0 numeric and 3.0 TYPE spellings', () => {
    const doc = [
      'BEGIN:VCARD',
      'FN:A',
      'EMAIL;PREF=1:first@example.com',
      'EMAIL;TYPE=WORK,PREF:second@example.com',
      'EMAIL:third@example.com',
      'END:VCARD',
    ].join('\r\n');
    const [c] = parseVCards(doc);
    expect(c?.emails.map((e) => e.pref)).toEqual([1, 1, 0]);
    // `PREF` is a preference marker, not a context — it must not become one.
    expect(c?.emails[1]?.context).toBe('work');
  });

  it('honours quoted param values containing the separator', () => {
    const doc = 'BEGIN:VCARD\r\nFN:A\r\nEMAIL;TYPE="work,urgent":a@example.com\r\nEND:VCARD';
    expect(parseVCards(doc)[0]?.emails[0]?.context).toBe('work,urgent');
  });

  it('normalises compact vCard dates but passes partial ones through', () => {
    const doc = [
      'BEGIN:VCARD',
      'FN:A',
      'BDAY:19901215',
      'ANNIVERSARY:2015-06-01',
      'BDAY:--1215', // vCard 4.0 year-less birthday
      'END:VCARD',
    ].join('\r\n');
    expect(parseVCards(doc)[0]?.anniversaries).toEqual([
      { kind: 'birthday', date: '1990-12-15' },
      { kind: 'anniversary', date: '2015-06-01' },
      { kind: 'birthday', date: '--1215' },
    ]);
  });

  it('strips the urn:uuid: prefix from UID', () => {
    const doc = 'BEGIN:VCARD\r\nFN:A\r\nUID:urn:uuid:1234-5678\r\nEND:VCARD';
    expect(parseVCards(doc)[0]?.uid).toBe('1234-5678');
  });

  it('joins repeated NOTE properties rather than keeping only the last', () => {
    const doc = 'BEGIN:VCARD\r\nFN:A\r\nNOTE:one\r\nNOTE:two\r\nEND:VCARD';
    expect(parseVCards(doc)[0]?.notes).toBe('one\ntwo');
  });

  it('reads addresses into their seven structured components', () => {
    const doc = 'BEGIN:VCARD\r\nFN:A\r\nADR;TYPE=WORK:;Suite 4;12 Main St;Springfield;IL;62701;USA\r\nEND:VCARD';
    expect(parseVCards(doc)[0]?.addresses[0]).toEqual({
      context: 'work',
      pobox: '',
      ext: 'Suite 4',
      street: '12 Main St',
      locality: 'Springfield',
      region: 'IL',
      postcode: '62701',
      country: 'USA',
    });
  });

  it('maps IMPP and X-SOCIALPROFILE onto online services', () => {
    const doc = [
      'BEGIN:VCARD',
      'FN:A',
      'IMPP;TYPE=xmpp:xmpp:a@example.com',
      'X-SOCIALPROFILE:https://example.com/@a', // no TYPE → derived from the name
      'END:VCARD',
    ].join('\r\n');
    expect(parseVCards(doc)[0]?.onlineServices).toEqual([
      { context: 'xmpp', value: 'xmpp:a@example.com' },
      { context: 'socialprofile', value: 'https://example.com/@a' },
    ]);
  });
});

describe('toVCard', () => {
  it('emits a well-formed 4.0 card with CRLF endings', () => {
    const out = toVCard(card());
    expect(out.startsWith('BEGIN:VCARD\r\nVERSION:4.0\r\n')).toBe(true);
    expect(out.endsWith('\r\nEND:VCARD')).toBe(true);
    expect(out).toContain('\r\nFN:Ada Lovelace\r\n');
    expect(out).toContain('\r\nN:Lovelace;Ada;;Dr;PhD\r\n');
  });

  it('escapes TEXT values so a value can never forge a new property line', () => {
    const out = toVCard(card({ notes: 'a,b;c\\d\nSUMMARY:injected' }));
    expect(out).toContain('NOTE:a\\,b\\;c\\\\d\\nSUMMARY:injected');
    // The newline is escaped, so the forged property is part of the NOTE value
    // and never becomes a line of its own.
    expect(out.split('\r\n').some((l) => l.startsWith('SUMMARY:'))).toBe(false);
  });

  it('folds long lines to 75 characters with a leading-space continuation', () => {
    const out = toVCard(card({ notes: 'x'.repeat(300) }));
    const lines = out.split('\r\n');
    for (const line of lines) expect(line.length).toBeLessThanOrEqual(75);
    expect(lines.filter((l) => l.startsWith(' ')).length).toBeGreaterThan(0);
    // Folding is reversible: the reader gets the value back intact.
    expect(parseVCards(out)[0]?.notes).toBe('x'.repeat(300));
  });

  it('omits optional properties rather than emitting empty ones', () => {
    const out = toVCard(card({ uid: '', notes: '', pgpKey: '' }));
    expect(out).not.toContain('UID:');
    expect(out).not.toContain('NOTE:');
    expect(out).not.toContain('KEY:');
    expect(out).not.toContain('KIND:');
  });

  it('emits KIND only for organisations', () => {
    expect(toVCard(card({ kind: 'org' }))).toContain('KIND:org');
    expect(toVCard(card({ kind: 'individual' }))).not.toContain('KIND:');
  });

  it('emits email TYPE and PREF params', () => {
    const out = toVCard(
      card({ emails: [{ context: 'work', value: 'a@example.com', pref: 2 }] }),
    );
    expect(out).toContain('EMAIL;TYPE=work;PREF=2:a@example.com');
  });

  it('separates cards with CRLF and terminates the document', () => {
    const doc = toVCardDocument([card(), card({ name: { ...card().name, full: 'Grace' } })]);
    expect(doc.endsWith('\r\n')).toBe(true);
    expect(parseVCards(doc)).toHaveLength(2);
  });

  it('exports the anniversary helper used by callers', () => {
    expect(birthday('1990-12-15')).toEqual({ kind: 'birthday', date: '1990-12-15' });
  });
});

describe('round trip', () => {
  it('preserves names, emails, phones, orgs, titles, notes, dates and key', () => {
    const original = card({
      uid: 'uid-1',
      organizations: ['Analytical Engines'],
      titles: ['Mathematician'],
      emails: [
        { context: 'work', value: 'ada@example.com', pref: 1 },
        { context: '', value: 'ada@home.example', pref: 0 },
      ],
      phones: [{ context: 'mobile', value: '+15551234' }],
      onlineServices: [{ context: 'xmpp', value: 'xmpp:ada@example.com' }],
      anniversaries: [birthday('1815-12-10')],
      notes: 'first programmer',
      pgpKey: 'ABC123',
    });
    const [back] = parseVCards(toVCard(original));

    expect(back?.uid).toBe(original.uid);
    expect(back?.name).toEqual(original.name);
    expect(back?.emails).toEqual(original.emails);
    expect(back?.phones).toEqual(original.phones);
    expect(back?.onlineServices).toEqual(original.onlineServices);
    expect(back?.organizations).toEqual(original.organizations);
    expect(back?.titles).toEqual(original.titles);
    expect(back?.anniversaries).toEqual(original.anniversaries);
    expect(back?.notes).toBe(original.notes);
    expect(back?.pgpKey).toBe(original.pgpKey);
  });

  it('preserves values that contain the vCard delimiters', () => {
    const original = card({ notes: 'semi; comma, backslash \\ newline\nend', titles: ['Head, R&D'] });
    const [back] = parseVCards(toVCard(original));
    expect(back?.notes).toBe(original.notes);
    expect(back?.titles).toEqual(original.titles);
  });

  // ── RECORDED DEFECTS ────────────────────────────────────────────────────
  // Both are in `vcard.ts`, outside this lane's locks. They are pinned as the
  // CURRENT behaviour, named as wrong, so a fix turns these red and gets them
  // rewritten as ordinary round-trip assertions.

  it('KNOWN BUG: export drops postal addresses entirely', () => {
    // `applyProperty` parses ADR into seven components, but `toVCard` has no ADR
    // branch at all — so exporting a contact loses every postal address without
    // any warning. Fix = an ADR emit block mirroring the parse side.
    const original = card({
      addresses: [
        {
          context: 'work',
          pobox: '',
          ext: '',
          street: '12 Main St',
          locality: 'Springfield',
          region: 'IL',
          postcode: '62701',
          country: 'USA',
        },
      ],
    });
    const out = toVCard(original);
    expect(out).not.toContain('ADR');
    expect(parseVCards(out)[0]?.addresses).toEqual([]);
  });

  it('KNOWN BUG: multiple nicknames collapse into one on export', () => {
    // `toVCard` joins nicknames with `,` and then hands the joined string to
    // `emitProp`, which escapes it again — so the LIST separator is emitted as
    // `\,`, a literal comma. The reader correctly unescapes it back into a
    // single nickname. Fix = emit the pre-escaped list without a second pass
    // (the `N:` property already does exactly that).
    const out = toVCard(card({ nicknames: ['Countess', 'Enchantress'] }));
    expect(out).toContain('NICKNAME:Countess\\,Enchantress');
    expect(parseVCards(out)[0]?.nicknames).toEqual(['Countess,Enchantress']);
    // A single nickname is unaffected, which is why this has gone unnoticed.
    expect(parseVCards(toVCard(card({ nicknames: ['Countess'] })))[0]?.nicknames).toEqual([
      'Countess',
    ]);
  });
});
