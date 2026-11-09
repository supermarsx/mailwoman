import { expect, it } from 'vitest';
import { chipsOf, offlineQueryOf, parseQuery, serializeQuery, toggleTerm } from './searchQuery.ts';
import { offlineSearch } from '../offline/search.ts';
import { mkEmail } from '../components/appHarness.tsx';
it('recognizes follow-up aliases, toggle negation and mention-address chips',()=>{
  const query=parseQuery('is:follow-up is:completed mentions:me');
  expect(chipsOf(query).map(chip=>chip.kind)).toEqual(['followup','finished','mentions']);
  expect(serializeQuery(query)).toBe('is:followup is:finished mentions:me');
  expect(serializeQuery({terms:[toggleTerm(query.terms[0]!)]})).toBe('-is:followup');
});
it('searches cached reminder metadata and flags while excluding finished entries from active follow-ups',()=>{
  const emails=[mkEmail('plain'),mkEmail('flag',{keywords:{$flagged:true}}),mkEmail('reminder',{followUpAt:'2026-11-20T10:00:00Z'}),mkEmail('done',{keywords:{$flagged:true,'mw-followup-completed':true},followUpAt:'2026-11-20T10:00:00Z'})];
  expect(offlineSearch(emails,offlineQueryOf('is:followup')).map(email=>email.id)).toEqual(['flag','reminder']);
  expect(offlineSearch(emails,offlineQueryOf('is:finished')).map(email=>email.id)).toEqual(['done']);
  expect(offlineSearch(emails,offlineQueryOf('-is:followup')).map(email=>email.id)).toEqual(['plain','done']);
});
