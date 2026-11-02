import { afterEach, describe, expect, it, vi } from 'vitest';
import { createRoot } from 'solid-js';
import type { Client } from '../../api/client.ts';
import { activateAccountPreferences, suspendAccountPreferences } from '../../api/accountPreferenceLifecycle.ts';
import { defaultSeed, makeContactsClient } from '../../modules/contacts/mockClient.ts';
import { CAP_CONTACTS } from '../../api/pim-types.ts';
import { createContactsSlice } from './contacts.ts';
const disposers: (() => void)[] = [];
afterEach(() => { disposers.splice(0).forEach(dispose => dispose()); suspendAccountPreferences(); });
function sliceFor(client: Client) {
  activateAccountPreferences('acct1');
  return createRoot(dispose => { disposers.push(dispose); return createContactsSlice({ client, showToast: vi.fn() }); });
}
describe('address contact lookup', () => {
  it('searches every book, matches exact local-part identity and preserves all contact screen state', async () => {
    const seed = defaultSeed(); seed.contacts![0]!.addressBookId = 'other-book'; seed.contacts![0]!.emails[0]!.value = 'Ada+Team@EXAMPLE.org';
    const client = makeContactsClient(seed), jmap = vi.spyOn(client, 'jmap'), slice = sliceFor(client);
    slice.selectAddressBook('selected-book'); slice.setContactSearch('unchanged'); slice.selectContact('c2');
    const result = await slice.lookupContactAddress('Ada+Team@example.org');
    expect(result?.contact?.id).toBe('c1');
    expect(jmap.mock.calls[0]![0].methodCalls.find(([method]) => method === 'ContactCard/query')?.[1]).toMatchObject({ filter: { text: 'Ada+Team@example.org' } });
    expect(jmap.mock.calls[0]![0].methodCalls.find(([method]) => method === 'ContactCard/query')?.[1].filter).not.toHaveProperty('addressBookId');
    expect(slice.selectedAddressBookId()).toBe('selected-book'); expect(slice.contactSearch()).toBe('unchanged'); expect(slice.selectedContactId()).toBe('c2'); expect(slice.contacts()).toEqual([]);
    expect((await slice.lookupContactAddress('ada+team@example.org'))?.contact).toBeNull();
  });
  it('does not read unsupported contacts or dispatch invalid addresses', async () => {
    const inner = makeContactsClient(defaultSeed());
    const client: Client = { ...inner, session: async () => { const session = await inner.session(), primaryAccounts = { ...session.primaryAccounts }; delete primaryAccounts[CAP_CONTACTS]; return { ...session, primaryAccounts }; } };
    const jmap = vi.spyOn(client, 'jmap'), slice = sliceFor(client);
    expect(await slice.lookupContactAddress('person@example.org')).toBeNull(); expect(await slice.lookupContactAddress('bad\n@example.org')).toBeNull(); expect(jmap).not.toHaveBeenCalled();
  });
  it('drops a lookup that resolves after an account replacement', async () => {
    const inner = makeContactsClient(defaultSeed()); let release!: () => void, reached!: () => void;
    const gate = new Promise<void>(resolve => { release = resolve; }), waiting = new Promise<void>(resolve => { reached = resolve; });
    const client: Client = { ...inner, jmap: async request => { const result = await inner.jmap(request); reached(); await gate; return result; } };
    const slice = sliceFor(client), lookup = slice.lookupContactAddress('ada@example.org'); await waiting;
    suspendAccountPreferences(); activateAccountPreferences('acct2'); release();
    expect(await lookup).toBeNull(); expect(slice.contacts()).toEqual([]); expect(slice.addressBooks()).toEqual([]);
  });
  it('refuses a mismatched account response instead of offering contact creation', async () => {
    const inner = makeContactsClient(defaultSeed());
    const client: Client = { ...inner, jmap: async request => { const result = await inner.jmap(request); for (const [method, args] of result.methodResponses) if (method === 'ContactCard/get') Object.assign(args, { accountId: 'another-account' }); return result; } };
    await expect(sliceFor(client).lookupContactAddress('ada@example.org')).rejects.toThrow('Contact lookup failed');
  });
});
