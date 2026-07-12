// Unit tests for the contacts module (plan §3 e7 acceptance): vCard/CSV parse +
// mapping, merge projection + duplicate detection, autocomplete ranking, and the
// store slice's list/create/favorite/import/merge/group behaviour over the mock
// backend. Component (UI) coverage lives in `contacts.test.tsx`.

import { describe, it, expect } from 'vitest';
import { createRoot } from 'solid-js';
import { rankSuggestions, suggestionDisplay } from './autocomplete.ts';
import { mergeCards, findDuplicateClusters } from './merge.ts';
import { parseCsv, guessMapping, csvToContacts, contactsToCsv, type CsvMapping } from './csv.ts';
import { parseVCards, toVCard } from './vcard.ts';
import { makeContactsClient, defaultSeed } from './mockClient.ts';
import {
  createContactsSlice,
  contactDisplayName,
  contactMatches,
  draftToCard,
  type ContactsSlice,
} from '../../state/slices/contacts.ts';
import type { SliceContext } from '../../state/slices/context.ts';
import type { ContactCard } from '../../api/pim-types.ts';

function card(id: string, over: Partial<ContactCard> = {}): ContactCard {
  return draftToCard(id, 'ab1', over);
}

// ── vCard ────────────────────────────────────────────────────────────────────

describe('vcard parse/emit', () => {
  it('parses a vCard 3.0 card with N, EMAIL and TEL', () => {
    const text = [
      'BEGIN:VCARD', 'VERSION:3.0', 'FN:Grace Hopper', 'N:Hopper;Grace;;;',
      'EMAIL;TYPE=WORK:grace@example.org', 'TEL;TYPE=CELL:+1-555-0100', 'END:VCARD',
    ].join('\r\n');
    const [c] = parseVCards(text);
    expect(c!.name.full).toBe('Grace Hopper');
    expect(c!.name.surname).toBe('Hopper');
    expect(c!.emails[0]).toMatchObject({ value: 'grace@example.org', context: 'work' });
    expect(c!.phones[0]!.context).toBe('mobile'); // CELL → mobile
  });

  it('derives FN from N when FN is absent, and round-trips through emit', () => {
    const [c] = parseVCards(['BEGIN:VCARD', 'VERSION:4.0', 'N:Lovelace;Ada;;;', 'END:VCARD'].join('\r\n'));
    expect(c!.name.full).toBe('Ada Lovelace');
    const out = toVCard(card('c1', { name: c!.name, emails: [{ context: 'work', value: 'ada@x.org', pref: 1 }] }));
    expect(out).toContain('FN:Ada Lovelace');
    expect(out).toContain('EMAIL;TYPE=work;PREF=1:ada@x.org');
    // Re-parsing the emitted card recovers the same identity.
    expect(parseVCards(out)[0]!.name.full).toBe('Ada Lovelace');
  });
});

// ── CSV import + mapping ─────────────────────────────────────────────────────

describe('csv import', () => {
  it('parses quoted fields and guesses a sensible column mapping', () => {
    const csv = 'Full Name,E-mail,Company\r\n"Hopper, Grace",grace@example.org,Navy\r\n';
    const parsed = parseCsv(csv);
    expect(parsed.headers).toEqual(['Full Name', 'E-mail', 'Company']);
    expect(parsed.rows[0]).toEqual(['Hopper, Grace', 'grace@example.org', 'Navy']);
    expect(guessMapping(parsed.headers)).toEqual(['fullName', 'email', 'organization']);
  });

  it('materializes contacts from rows under an explicit mapping', () => {
    const parsed = parseCsv('First,Last,Mail\nAlan,Turing,alan@example.org\n');
    // These headers lack the "name" suffix the guesser keys on, so map by hand.
    const mapping = ['given', 'surname', 'email'] as CsvMapping;
    const out = csvToContacts(parsed, mapping);
    expect(out).toHaveLength(1);
    expect(out[0]!.name.full).toBe('Alan Turing'); // derived from given + surname
    expect(out[0]!.emails[0]!.value).toBe('alan@example.org');
  });

  it('exports cards to a CSV sheet with a header row', () => {
    const sheet = contactsToCsv([card('c1', { name: { full: 'Ada', given: 'Ada', surname: '', prefix: '', suffix: '' }, emails: [{ context: '', value: 'ada@x.org', pref: 0 }] })]);
    expect(sheet.split('\r\n')[0]).toContain('Full Name');
    expect(sheet).toContain('ada@x.org');
  });
});

// ── Merge ────────────────────────────────────────────────────────────────────

describe('mergeCards', () => {
  it('unions contact points, keeps the primary identity, favorites if any source is', () => {
    const a = card('c1', { name: { full: 'Ada Lovelace', given: 'Ada', surname: 'Lovelace', prefix: '', suffix: '' }, emails: [{ context: 'work', value: 'ada@work.org', pref: 1 }] });
    const b = card('c2', { emails: [{ context: 'home', value: 'ada@home.org', pref: 0 }, { context: 'work', value: 'ADA@work.org', pref: 0 }], isFavorite: true, phones: [{ context: '', value: '555' }] });
    const merged = mergeCards(a, [b]);
    expect(merged.id).toBe('c1'); // survivor keeps primary id
    expect(merged.name.full).toBe('Ada Lovelace');
    // ada@work.org deduped case-insensitively; both distinct addresses kept.
    expect(merged.emails.map((e) => e.value.toLowerCase()).sort()).toEqual(['ada@home.org', 'ada@work.org']);
    expect(merged.phones).toHaveLength(1);
    expect(merged.isFavorite).toBe(true);
  });
});

describe('findDuplicateClusters', () => {
  it('clusters cards that share an email or an identical full name', () => {
    const cards = [
      card('c1', { name: { full: 'Grace Hopper', given: '', surname: '', prefix: '', suffix: '' }, emails: [{ context: '', value: 'grace@x.org', pref: 0 }] }),
      card('c2', { name: { full: 'Grace Hopper', given: '', surname: '', prefix: '', suffix: '' }, emails: [] }),
      card('c3', { name: { full: 'Someone Else', given: '', surname: '', prefix: '', suffix: '' }, emails: [{ context: '', value: 'grace@x.org', pref: 0 }] }),
      card('c4', { name: { full: 'Unique Person', given: '', surname: '', prefix: '', suffix: '' }, emails: [{ context: '', value: 'u@x.org', pref: 0 }] }),
    ];
    const clusters = findDuplicateClusters(cards);
    expect(clusters).toHaveLength(1);
    // c1↔c2 by name, c1↔c3 by email — one transitive cluster of three.
    expect(clusters[0]!.map((c) => c.id).sort()).toEqual(['c1', 'c2', 'c3']);
  });
});

// ── Autocomplete ranking ─────────────────────────────────────────────────────

describe('rankSuggestions', () => {
  const cards = [
    card('c1', { name: { full: 'Ada Lovelace', given: 'Ada', surname: 'Lovelace', prefix: '', suffix: '' }, emails: [{ context: 'work', value: 'ada@example.org', pref: 1 }], isFavorite: true }),
    card('c2', { name: { full: 'Alan Turing', given: 'Alan', surname: 'Turing', prefix: '', suffix: '' }, emails: [{ context: 'home', value: 'alan@example.org', pref: 0 }] }),
    card('c3', { name: { full: 'Ada Byron', given: 'Ada', surname: 'Byron', prefix: '', suffix: '' }, emails: [{ context: '', value: 'byron@example.org', pref: 0 }] }),
  ];

  it('returns nothing for a blank prefix', () => {
    expect(rankSuggestions(cards, '   ')).toEqual([]);
  });

  it('ranks a favorite name-prefix match ahead of a non-favorite one', () => {
    const out = rankSuggestions(cards, 'ada');
    expect(out.map((s) => s.cardId)).toEqual(['c1', 'c3']); // both "Ada*", favorite first
    expect(out[0]!.display).toBe('Ada Lovelace <ada@example.org>');
  });

  it('matches on email prefix and caps at the limit', () => {
    const out = rankSuggestions(cards, 'alan@', 1);
    expect(out).toHaveLength(1);
    expect(out[0]!.email).toBe('alan@example.org');
  });

  it('formats a bare email when the card is unnamed', () => {
    expect(suggestionDisplay('', 'x@y.org')).toBe('x@y.org');
  });
});

// ── Slice ────────────────────────────────────────────────────────────────────

function withSlice(seed: Parameters<typeof makeContactsClient>[0], run: (s: ContactsSlice) => Promise<void>): Promise<void> {
  return new Promise<void>((resolve, reject) => {
    createRoot((dispose) => {
      const ctx: SliceContext = { client: makeContactsClient(seed), showToast: () => undefined };
      const slice = createContactsSlice(ctx);
      run(slice).then(resolve, reject).finally(dispose);
    });
  });
}

describe('contacts slice', () => {
  it('loads address books, groups and contacts', async () => {
    await withSlice(defaultSeed(), async (s) => {
      await s.loadContacts();
      expect(s.addressBooks().map((b) => b.id)).toEqual(['ab1']);
      expect(s.contactGroups().map((g) => g.id)).toEqual(['g1']);
      expect(s.contacts().map((c) => c.id).sort()).toEqual(['c1', 'c2']);
      // Sorted by display name (Ada before Alan).
      expect(s.filteredContacts()[0]!.name.full).toBe('Ada Lovelace');
    });
  });

  it('search + favorites + group filters narrow the list', async () => {
    await withSlice(defaultSeed(), async (s) => {
      await s.loadContacts();
      s.setContactSearch('turing');
      expect(s.filteredContacts().map((c) => c.id)).toEqual(['c2']);
      s.setContactSearch('');
      s.setFavoritesOnly(true);
      expect(s.filteredContacts().map((c) => c.id)).toEqual(['c1']);
      s.setFavoritesOnly(false);
      s.setSelectedGroup('g1');
      expect(s.filteredContacts().map((c) => c.id)).toEqual(['c1']); // only member of Colleagues
    });
  });

  it('creates a contact and toggles its favorite flag', async () => {
    await withSlice(defaultSeed(), async (s) => {
      await s.loadContacts();
      const id = await s.createContact({ name: { full: 'Grace Hopper', given: 'Grace', surname: 'Hopper', prefix: '', suffix: '' } });
      expect(id).not.toBeNull();
      expect(s.contacts().some((c) => c.id === id)).toBe(true);
      await s.toggleFavorite(id!);
      expect(s.contactById(id!)!.isFavorite).toBe(true);
    });
  });

  it('imports parsed drafts and reports the count', async () => {
    await withSlice(defaultSeed(), async (s) => {
      await s.loadContacts();
      const drafts = parseVCards(['BEGIN:VCARD', 'VERSION:4.0', 'FN:New Person', 'EMAIL:new@example.org', 'END:VCARD'].join('\r\n'));
      const n = await s.importContacts(drafts);
      expect(n).toBe(1);
      expect(s.contacts().some((c) => c.name.full === 'New Person')).toBe(true);
    });
  });

  it('merges duplicates non-destructively into the survivor', async () => {
    const seed = defaultSeed();
    seed.contacts = [
      { ...seed.contacts[0]!, id: 'd1', name: { full: 'Grace Hopper', given: '', surname: '', prefix: '', suffix: '' }, emails: [{ context: 'work', value: 'grace@x.org', pref: 1 }], isFavorite: false },
      { ...seed.contacts[1]!, id: 'd2', name: { full: 'Grace Hopper', given: '', surname: '', prefix: '', suffix: '' }, emails: [{ context: 'home', value: 'grace@home.org', pref: 0 }], isFavorite: true },
    ];
    await withSlice(seed, async (s) => {
      await s.loadContacts();
      const kept = await s.mergeContacts('d1', ['d2']);
      expect(kept).toBe('d1');
      expect(s.contacts().map((c) => c.id)).toEqual(['d1']); // d2 tombstoned
      const survivor = s.contactById('d1')!;
      expect(survivor.emails).toHaveLength(2); // unioned
      expect(survivor.isFavorite).toBe(true); // inherited
    });
  });

  it('creates a group and toggles membership on both edges', async () => {
    await withSlice(defaultSeed(), async (s) => {
      await s.loadContacts();
      const gid = await s.createGroup('Friends');
      expect(gid).not.toBeNull();
      await s.setGroupMembership('c2', gid!, true);
      expect(s.contactGroups().find((g) => g.id === gid)!.memberIds).toContain('c2');
      expect(s.contactById('c2')!.groupIds).toContain(gid);
      await s.setGroupMembership('c2', gid!, false);
      expect(s.contactGroups().find((g) => g.id === gid)!.memberIds).not.toContain('c2');
    });
  });
});

describe('contact helpers', () => {
  it('contactDisplayName falls back through org and email', () => {
    expect(contactDisplayName(card('c1', { organizations: ['ACME'] }))).toBe('ACME');
    expect(contactDisplayName(card('c1', { emails: [{ context: '', value: 'x@y.org', pref: 0 }] }))).toBe('x@y.org');
    expect(contactDisplayName(card('c1'))).toBe('No name');
  });

  it('contactMatches searches across name, org, email', () => {
    const c = card('c1', { name: { full: 'Ada Lovelace', given: 'Ada', surname: 'Lovelace', prefix: '', suffix: '' }, organizations: ['Analytical Engine'], emails: [{ context: '', value: 'ada@example.org', pref: 0 }] });
    expect(contactMatches(c, 'lovelace')).toBe(true);
    expect(contactMatches(c, 'engine')).toBe(true);
    expect(contactMatches(c, 'example.org')).toBe(true);
    expect(contactMatches(c, 'zzz')).toBe(false);
  });
});
