import { afterEach, expect, it } from 'vitest';
import { activateAccountPreferences, suspendAccountPreferences } from './accountPreferenceLifecycle.ts';
import { storageAuthorityHeaders } from './storageAuthority.ts';
afterEach(suspendAccountPreferences);
it('refuses missing authority and a stale same-ID realm', () => {
  activateAccountPreferences('1');
  expect(() => storageAuthorityHeaders()).toThrow();
  activateAccountPreferences('1', 'short');
  expect(() => storageAuthorityHeaders()).toThrow(); // Invalid short server value is not promoted.
  activateAccountPreferences('1', 'msa1_realm_first_valid');
  expect(storageAuthorityHeaders()).toEqual({'x-mailwoman-storage-authority':'msa1_realm_first_valid'});
  activateAccountPreferences('1', 'msa1_realm_second_valid');
  expect(() => storageAuthorityHeaders('msa1_realm_first_valid')).toThrow();
});
