import { describe, expect, it } from 'vitest';
import { bareAddress, chooseReplyIdentity, stripPlusTag, type FromCandidate } from './replyIdentity.ts';

const work: FromCandidate = { id: 'work', email: 'me@work.example', isDefault: true };
const home: FromCandidate = { id: 'home', email: 'Me@Home.example' };
const formal: FromCandidate = { id: 'formal', email: 'me@work.example', variantOf: 'work' };
const ids = [work, home];

describe('chooseReplyIdentity', () => {
  it('matches To case-insensitively and parses display names', () => {
    expect(chooseReplyIdentity(ids, { to: ['Me <ME@home.EXAMPLE>'] }, 'match')).toMatchObject({ identityId: 'home', reason: 'match' });
  });
  it('falls through To to Cc to delivered headers', () => {
    expect(chooseReplyIdentity(ids, { to: ['x@y.z'], cc: ['me@home.example'] }, 'match').identityId).toBe('home');
    expect(chooseReplyIdentity(ids, { to: ['list@y.z'], delivered: ['me@home.example'] }, 'match').identityId).toBe('home');
  });
  it('supports plus-addressing and prefers exact over plus', () => {
    expect(chooseReplyIdentity(ids, { to: ['me+news@home.example'] }, 'match')).toMatchObject({ identityId: 'home', reason: 'match-plus' });
    const tagged = { id: 'tag', email: 'me+news@home.example' };
    expect(chooseReplyIdentity([...ids, tagged], { to: ['me+news@home.example'] }, 'match')).toMatchObject({ identityId: 'tag', reason: 'match' });
  });
  it('uses the default when nothing matches', () => {
    expect(chooseReplyIdentity(ids, { to: ['stranger@elsewhere.test'] }, 'match')).toMatchObject({ identityId: 'work', reason: 'default' });
  });
  it('prefers a matching variant, and reports alternatives for ask', () => {
    expect(chooseReplyIdentity([work, formal, home], { to: ['me@work.example'] }, 'match').identityId).toBe('formal');
    const ask = chooseReplyIdentity([work, formal, home], { to: ['me@work.example'] }, 'ask');
    expect(ask.reason).toBe('ambiguous');
    expect(ask.alternatives.map((item) => item.id)).toEqual(['work']);
  });
  it('honours default and keep modes', () => {
    expect(chooseReplyIdentity(ids, { to: ['me@home.example'] }, 'default')).toMatchObject({ identityId: 'work', reason: 'forced-default' });
    expect(chooseReplyIdentity(ids, { to: ['me@home.example'] }, 'keep').identityId).toBeNull();
  });
  it('helpers', () => {
    expect(bareAddress('A <B@C.d>')).toBe('b@c.d');
    expect(stripPlusTag('a+b@c.d')).toBe('a@c.d');
    expect(stripPlusTag('+b@c.d')).toBe('+b@c.d');
  });
});
