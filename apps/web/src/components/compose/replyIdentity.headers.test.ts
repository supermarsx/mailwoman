import { describe, expect, it } from 'vitest';
import { chooseReplyIdentity, matchesPattern, parseHeaderPriority, type FromCandidate } from './replyIdentity.ts';

const work: FromCandidate = { id: 'work', email: 'me@work.example', isDefault: true };
const home: FromCandidate = { id: 'home', email: 'Me@Home.example' };
const sales: FromCandidate = { id: 'sales', email: 'sales@work.example' };
const all = [work, home, sales];

describe('chooseReplyIdentity: delivery headers and their precedence', () => {
  it('Delivered-To beats To by default, even when To names another identity', () => {
    const got = chooseReplyIdentity(all, { to: ['me@work.example'], headers: { 'delivered-to': ['sales@work.example'] } }, 'match');
    expect(got).toMatchObject({ identityId: 'sales', reason: 'match', source: 'delivered-to', address: 'sales@work.example' });
  });
  it('X-Original-To is next, and its plus tag is understood', () => {
    const got = chooseReplyIdentity(all, { to: ['list@elsewhere.test'], headers: { 'x-original-to': ['me+news@home.example'] } }, 'match');
    expect(got).toMatchObject({ identityId: 'home', reason: 'match-plus', source: 'x-original-to' });
  });
  it('honours a configured order: To first puts the visible recipient ahead of the header', () => {
    const received = { to: ['me@home.example'], headers: { 'delivered-to': ['sales@work.example'] } } as const;
    expect(chooseReplyIdentity(all, received, 'match', ['to', 'delivered-to']).identityId).toBe('home');
    expect(chooseReplyIdentity(all, received, 'match', ['delivered-to', 'to']).identityId).toBe('sales');
  });
  it('a header left out of the priority list is not consulted', () => {
    const received = { to: ['x@y.z'], headers: { 'envelope-to': ['me@home.example'] } } as const;
    expect(chooseReplyIdentity(all, received, 'match').identityId).toBe('work');
    expect(chooseReplyIdentity(all, received, 'match', ['envelope-to']).identityId).toBe('home');
  });
  it('X-Forwarded-To and Resent-To work when listed', () => {
    for (const source of ['x-forwarded-to', 'resent-to'] as const) {
      expect(chooseReplyIdentity(all, { to: [], headers: { [source]: ['Me <me@home.example>'] } }, 'match', [source])).toMatchObject({ identityId: 'home', source });
    }
  });
  it('the legacy delivered list still counts as Delivered-To', () => {
    expect(chooseReplyIdentity(all, { to: [], delivered: ['me@home.example'] }, 'match')).toMatchObject({ identityId: 'home', source: 'delivered-to' });
  });
  it('falls back to the default identity when no header matches', () => {
    const got = chooseReplyIdentity(all, { to: ['a@b.test'], cc: ['c@d.test'], headers: { 'delivered-to': ['z@z.test'] } }, 'match');
    expect(got).toMatchObject({ identityId: 'work', reason: 'default' });
    expect(got.source).toBeUndefined();
  });
});

describe('chooseReplyIdentity: patterns, catch-all and subdomains', () => {
  it('a *@domain identity is a catch-all for that domain, including plus tags', () => {
    const catchAll: FromCandidate = { id: 'all', email: '*@shop.example' };
    expect(chooseReplyIdentity([work, catchAll], { to: [], headers: { 'delivered-to': ['anything+x@shop.example'] } }, 'match')).toMatchObject({ identityId: 'all', reason: 'match-pattern' });
    expect(chooseReplyIdentity([work, catchAll], { to: ['x@other.example'] }, 'match').identityId).toBe('work');
  });
  it('wildcards stay inside their half of the address', () => {
    expect(matchesPattern('me+*@home.example', 'me+tag@home.example')).toBe(true);
    expect(matchesPattern('me+*@home.example', 'you+tag@home.example')).toBe(false);
    expect(matchesPattern('me@*.home.example', 'me@mail.home.example')).toBe(true);
    expect(matchesPattern('me@*.home.example', 'me@a.b.home.example')).toBe(true);
    expect(matchesPattern('me@*.home.example', 'me@home.example')).toBe(false);
    expect(matchesPattern('*@home.example', 'x@evilhome.example')).toBe(false);
    expect(matchesPattern('*@home.example', 'a.b@home.example')).toBe(true);
  });
  it('an exact address beats a pattern, and a plus match beats a pattern', () => {
    const catchAll: FromCandidate = { id: 'all', email: '*@work.example' };
    expect(chooseReplyIdentity([catchAll, work], { to: ['me@work.example'] }, 'match')).toMatchObject({ identityId: 'work', reason: 'match' });
    expect(chooseReplyIdentity([catchAll, work], { to: ['me+x@work.example'] }, 'match')).toMatchObject({ identityId: 'work', reason: 'match-plus' });
  });
  it('mail delivered to a catch-all alias or subdomain replies from the identity at that domain', () => {
    expect(chooseReplyIdentity([home, work], { to: [], headers: { 'delivered-to': ['whatever@work.example'] } }, 'match')).toMatchObject({ identityId: 'work', reason: 'match-domain' });
    expect(chooseReplyIdentity([home, work], { to: [], headers: { 'delivered-to': ['x@mail.work.example'] } }, 'match')).toMatchObject({ identityId: 'work', reason: 'match-domain' });
  });
  it('a shared domain in To or Cc proves nothing: a colleague is not me', () => {
    expect(chooseReplyIdentity([home, work], { to: ['colleague@work.example'], cc: ['colleague@home.example'] }, 'match')).toMatchObject({ identityId: 'work', reason: 'default' });
    expect(chooseReplyIdentity([{ ...home, isDefault: true }, { ...work, isDefault: false }], { to: ['colleague@work.example'] }, 'match')).toMatchObject({ identityId: 'home', reason: 'default' });
  });
  it('a top-level domain alone is never a parent domain', () => {
    const tld: FromCandidate = { id: 'tld', email: 'me@example' };
    expect(chooseReplyIdentity([work, tld], { to: [], headers: { 'delivered-to': ['x@foo.example'] } }, 'match').reason).toBe('default');
  });
});

describe('parseHeaderPriority', () => {
  const standard = ['delivered-to', 'x-original-to', 'to', 'cc'];
  it('defaults when unset or invalid', () => {
    expect(parseHeaderPriority(undefined)).toEqual(standard);
    expect(parseHeaderPriority([])).toEqual(standard);
    expect(parseHeaderPriority(['Subject', 7])).toEqual(standard);
  });
  it('reads names case-insensitively, drops unknowns and repeats, and keeps order', () => {
    expect(parseHeaderPriority(['CC', 'X-Forwarded-To', 'cc', 'bogus', 'Delivered-To'])).toEqual(['cc', 'x-forwarded-to', 'delivered-to']);
  });
});
