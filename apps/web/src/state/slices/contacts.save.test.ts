import { afterEach, describe, expect, it, vi } from 'vitest';
import { createRoot } from 'solid-js';
import type { Client } from '../../api/client.ts';
import { activateAccountPreferences, suspendAccountPreferences } from '../../api/accountPreferenceLifecycle.ts';
import { defaultSeed, makeContactsClient } from '../../modules/contacts/mockClient.ts';
import { createContactsSlice } from './contacts.ts';

const disposers: (() => void)[] = [];
afterEach(() => { disposers.splice(0).forEach(dispose => dispose()); suspendAccountPreferences(); });
function setup(mode: 'reject' | 'missing' | 'delay', method = 'ContactCard/set') {
  activateAccountPreferences('acct1');
  const inner = makeContactsClient(defaultSeed());
  let release: () => void = () => undefined;
  const held = new Promise<void>(resolve => { release = resolve; });
  let reached: () => void = () => undefined;
  const waiting = new Promise<void>(resolve => { reached = resolve; });
  const client: Client = { ...inner, jmap: async request => {
    const response = await inner.jmap(request);
    if (request.methodCalls.some(([name]) => name === method)) {
      if (mode === 'delay') { reached(); await held; }
      else for (const call of response.methodResponses) {
        if (call[0] !== method) continue;
        Object.assign(call[1], {created: null, updated: null,
          notUpdated: mode === 'reject' ? {c1: {type: 'forbidden'}} : null,
          notCreated: mode === 'reject' ? {new: {type: 'forbidden'}} : null});
      }
    }
    return response;
  }};
  const broadcast = vi.fn();
  const slice = createRoot(dispose => { disposers.push(dispose); return createContactsSlice({client, showToast: vi.fn(), broadcastChange: broadcast}); });
  return {slice, broadcast, release, waiting};
}
describe('contact save acknowledgement and account fence', () => {
  for (const mode of ['reject', 'missing'] as const) it(`does not invent a group on ${mode} acknowledgement`, async () => {
    const {slice,broadcast}=setup(mode,'ContactGroup/set');await slice.loadContacts();const before=slice.contactGroups();
    expect(await slice.createGroup('Unconfirmed')).toBeNull();expect(slice.contactGroups()).toEqual(before);expect(broadcast).not.toHaveBeenCalled();
  });
  it('ignores a group acknowledgement after account replacement',async()=>{
    const {slice,broadcast,release,waiting}=setup('delay','ContactGroup/set');await slice.loadContacts();const save=slice.createGroup('Old account');await waiting;
    suspendAccountPreferences();activateAccountPreferences('acct2');release();expect(await save).toBeNull();expect(slice.contactGroups()).toEqual([]);expect(broadcast).not.toHaveBeenCalled();
  });
  for (const mode of ['reject', 'missing'] as const) it(`does not mutate or invent contacts on ${mode} acknowledgement`, async () => {
    const {slice, broadcast} = setup(mode); await slice.loadContacts();
    expect(await slice.updateContact('c1', {notes:'unsaved'})).toBe(false);
    expect(slice.contactById('c1')?.notes).not.toBe('unsaved');
    expect(await slice.createContact({notes:'new'})).toBeNull();
    expect(slice.contacts()).toHaveLength(2); expect(broadcast).not.toHaveBeenCalled();
  });
  it('clears old data and ignores a late successful save after account replacement', async () => {
    const {slice, broadcast, release, waiting} = setup('delay'); await slice.loadContacts();
    const save = slice.updateContact('c1', {notes:'old account'}); await waiting;
    suspendAccountPreferences(); activateAccountPreferences('acct2'); release();
    expect(await save).toBe(false); expect(slice.contacts()).toEqual([]); expect(broadcast).not.toHaveBeenCalled();
  });
});
