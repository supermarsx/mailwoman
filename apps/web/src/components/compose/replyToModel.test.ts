import { expect, it } from 'vitest';
import { checkReplyTo, domainOfAddress, replyToForSend, replyToRowVisible, replyToToggleAvailable } from './replyToModel.ts';

const base = { show: 'auto' as const, hasValue: false, toggled: false, repliesToReplyTo: false };

it('the row follows the show setting', () => {
  expect(replyToRowVisible(base)).toBe(false);
  expect(replyToRowVisible({ ...base, toggled: true })).toBe(true);
  expect(replyToRowVisible({ ...base, hasValue: true })).toBe(true);
  expect(replyToRowVisible({ ...base, repliesToReplyTo: true })).toBe(true);
  expect(replyToRowVisible({ ...base, show: 'always' })).toBe(true);
  expect(replyToRowVisible({ ...base, show: 'never', hasValue: true, toggled: true })).toBe(false);
  expect(replyToToggleAvailable('auto')).toBe(true);
  expect(replyToToggleAvailable('always')).toBe(false);
  expect(replyToToggleAvailable('never')).toBe(false);
});

it('validates addresses and flags a different organisation domain', () => {
  expect(checkReplyTo('boss@example.com', 'me@example.com')).toEqual({ invalid: [], external: [] });
  expect(checkReplyTo('x@mail.example.com', 'me@example.com').external).toEqual([]);
  expect(checkReplyTo('Team <team@other.org>, x@third.net, y@other.org', 'me@example.com').external).toEqual(['other.org', 'third.net']);
  const bad = checkReplyTo('not-an-address, ok@example.com', 'me@example.com');
  expect(bad.invalid).toEqual(['not-an-address']);
  expect(bad.external).toEqual([]);
  expect(domainOfAddress('A@Example.COM')).toBe('example.com');
});

it('send value: untouched or forbidden keeps the identity default, an edit wins, empty removes', () => {
  expect(replyToForSend({ edited: false, text: 'a@b.c', allowOverride: true })).toBeUndefined();
  expect(replyToForSend({ edited: true, text: 'a@b.c', allowOverride: false })).toBeUndefined();
  expect(replyToForSend({ edited: true, text: ' a@b.c ', allowOverride: true })).toBe('a@b.c');
  expect(replyToForSend({ edited: true, text: '', allowOverride: true })).toBe('');
});
