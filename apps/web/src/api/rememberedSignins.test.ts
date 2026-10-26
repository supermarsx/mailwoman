import { beforeEach, expect, it, vi } from 'vitest';
import { rememberedSignins, rememberSuccessfulSignin, forgetSignin } from './rememberedSignins.ts';
beforeEach(() => { localStorage.clear(); vi.restoreAllMocks(); });
it('remembers bounded unique identifiers in recent-use order and supports removal', () => {
  for (let i = 0; i < 12; i++) rememberSuccessfulSignin(`person${i}@example.test`);
  rememberSuccessfulSignin('person8@example.test');
  expect(rememberedSignins()).toHaveLength(10);
  expect(rememberedSignins()[0]).toBe('person8@example.test');
  forgetSignin('person8@example.test');
  expect(rememberedSignins()).not.toContain('person8@example.test');
});
it('does not store control characters or throw when storage is unavailable', () => {
  rememberSuccessfulSignin('bad\naddress');
  expect(rememberedSignins()).toEqual([]);
  vi.spyOn(Storage.prototype, 'setItem').mockImplementation(() => { throw new Error('disabled'); });
  expect(() => rememberSuccessfulSignin('alice@example.test')).not.toThrow();
});
