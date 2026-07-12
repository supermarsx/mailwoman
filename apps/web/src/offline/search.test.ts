import { describe, it, expect } from 'vitest';
import type { Email } from '../api/jmap-types.ts';
import { matchesOffline, offlineSearch, type OfflineQuery } from './search.ts';

function email(over: Partial<Email> & Pick<Email, 'id'>): Email {
  return {
    mailboxIds: { inbox: true },
    from: [{ name: 'Alice', email: 'alice@work.example' }],
    to: [{ name: null, email: 'me@home.example' }],
    subject: 'Quarterly report',
    receivedAt: '2026-07-12T00:00:00Z',
    preview: 'Please review the attached numbers.',
    ...over,
  } as Email;
}

const corpus: Email[] = [
  email({ id: 'a' }),
  email({
    id: 'b',
    from: [{ name: 'Bob', email: 'bob@vendor.example' }],
    subject: 'Lunch?',
    preview: 'want to grab food',
    keywords: { $seen: true, $flagged: true },
    hasAttachment: false,
  }),
  email({ id: 'c', subject: 'Invoice', hasAttachment: true, keywords: { $seen: true } }),
];

describe('matchesOffline', () => {
  it('free text matches across from/to/subject/preview, case-insensitively', () => {
    expect(matchesOffline(corpus[0]!, { text: 'QUARTERLY' })).toBe(true);
    expect(matchesOffline(corpus[0]!, { text: 'alice@work' })).toBe(true);
    expect(matchesOffline(corpus[0]!, { text: 'nonsense' })).toBe(false);
  });

  it('field filters target the right field', () => {
    expect(matchesOffline(corpus[1]!, { from: 'bob' })).toBe(true);
    expect(matchesOffline(corpus[1]!, { from: 'alice' })).toBe(false);
    expect(matchesOffline(corpus[0]!, { subject: 'report' })).toBe(true);
    expect(matchesOffline(corpus[0]!, { to: 'home.example' })).toBe(true);
  });

  it('keyword filters honour hasKeyword / notKeyword', () => {
    expect(matchesOffline(corpus[1]!, { hasKeyword: '$flagged' })).toBe(true);
    expect(matchesOffline(corpus[0]!, { hasKeyword: '$flagged' })).toBe(false);
    expect(matchesOffline(corpus[1]!, { notKeyword: '$flagged' })).toBe(false);
    expect(matchesOffline(corpus[0]!, { notKeyword: '$flagged' })).toBe(true);
  });

  it('hasAttachment filters on the cached flag', () => {
    expect(matchesOffline(corpus[2]!, { hasAttachment: true })).toBe(true);
    expect(matchesOffline(corpus[1]!, { hasAttachment: true })).toBe(false);
    expect(matchesOffline(corpus[0]!, { hasAttachment: false })).toBe(true);
  });

  it('conjoins every present field (all must match)', () => {
    const q: OfflineQuery = { text: 'invoice', hasAttachment: true };
    expect(matchesOffline(corpus[2]!, q)).toBe(true);
    expect(matchesOffline(corpus[2]!, { text: 'invoice', hasAttachment: false })).toBe(false);
  });

  it('an empty query matches everything', () => {
    expect(matchesOffline(corpus[0]!, {})).toBe(true);
  });
});

describe('offlineSearch', () => {
  it('filters the slice preserving order', () => {
    const hits = offlineSearch(corpus, { hasKeyword: '$seen' });
    expect(hits.map((e) => e.id)).toEqual(['b', 'c']);
  });

  it('returns an empty array on no match', () => {
    expect(offlineSearch(corpus, { text: 'zzzzz' })).toEqual([]);
  });
});
