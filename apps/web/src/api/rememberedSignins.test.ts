import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { rememberedSignins, rememberSuccessfulSignin, forgetSignin, rememberSuccessfulSigninIfAllowed } from './rememberedSignins.ts';
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

afterEach(() => vi.unstubAllGlobals());
it('checks current server policy before saving and purges disabled history', async () => {
  vi.stubGlobal('fetch', vi.fn(async () => new Response(JSON.stringify({rememberSignins:true}))));
  await rememberSuccessfulSigninIfAllowed('alice@example.test');
  expect(rememberedSignins()).toEqual(['alice@example.test']);
  vi.stubGlobal('fetch', vi.fn(async () => new Response(JSON.stringify({rememberSignins:false}))));
  await rememberSuccessfulSigninIfAllowed('bob@example.test');
  expect(rememberedSignins()).toEqual([]);
});
it('does not save on missing policy or configuration failures', async () => {
  vi.stubGlobal('fetch', vi.fn(async () => new Response('{}')));
  await rememberSuccessfulSigninIfAllowed('alice@example.test');
  expect(rememberedSignins()).toEqual([]);
  vi.stubGlobal('fetch', vi.fn(async () => { throw new Error('offline'); }));
  await expect(rememberSuccessfulSigninIfAllowed('alice@example.test')).resolves.toBeUndefined();
  expect(rememberedSignins()).toEqual([]);
});
