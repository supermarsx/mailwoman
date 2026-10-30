import { afterEach, expect, it } from 'vitest';
import { activateAccountPreferences, suspendAccountPreferences } from './accountPreferenceLifecycle.ts';
import { installLinkedMailbox, setPrimaryMailboxAuthority } from './linkedMailboxSession.ts';
import { personStorageAuthorityHeaders, storageAuthorityHeaders } from './storageAuthority.ts';
afterEach(() => { suspendAccountPreferences(); installLinkedMailbox(null); setPrimaryMailboxAuthority(undefined); });
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
it('t87 D4: person-scoped requests carry the sign-in authority while a linked mailbox is open', () => {
  setPrimaryMailboxAuthority('msa1_realm_primary_valid');
  activateAccountPreferences('parent', 'msa1_realm_primary_valid');
  expect(personStorageAuthorityHeaders()).toEqual({'x-mailwoman-storage-authority':'msa1_realm_primary_valid'});
  installLinkedMailbox({linkId:'11111111-1111-4111-8111-111111111111', accountId:'child', storageAuthority:'msa1_realm_child_valid'});
  activateAccountPreferences('child', 'msa1_realm_child_valid');
  expect(storageAuthorityHeaders()).toEqual({'x-mailwoman-storage-authority':'msa1_realm_child_valid'});
  expect(personStorageAuthorityHeaders()).toEqual({'x-mailwoman-storage-authority':'msa1_realm_primary_valid'});
  setPrimaryMailboxAuthority(undefined);
  expect(() => personStorageAuthorityHeaders()).toThrow();
});
