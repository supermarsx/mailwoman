import { describe, expect, it, vi } from 'vitest';
import { accountOutbox, outboundAccount } from './accountOutbox.ts';
import { drainOutbox, memoryOutboxStore } from './outbox.ts';
import type { OutboundItem } from '../contracts/offline.ts';
import type { Client } from '../api/client.ts';
const move = (id: string, accountId: string): OutboundItem => ({ id, type: 'move', payload: { accountId, emailId: id, mailboxIds: { archive: true } }, createdAt: 1, state: 'queued',storageAuthority:'test-storage-authority-a' });
describe('offline account isolation', () => {
  it('quarantines unknown/mixed legacy rows without deleting their bytes', async () => {
    const mixed: OutboundItem = { ...move('mixed', 'a'), type: 'pim', payload: { request: { methodCalls: [['Note/set', { accountId: 'a' }, 'a'], ['Note/set', { accountId: 'b' }, 'b']] }, callId: 'a' } };
    const unknown: OutboundItem = { ...move('unknown', 'a'), payload: {} };
    const store = memoryOutboxStore([move('a', 'a'), move('b', 'b'), mixed, unknown]);
    expect(outboundAccount(mixed)).toBeNull();
    expect(outboundAccount(unknown)).toBeNull();
    const a = accountOutbox(store, 'a','test-storage-authority-a');
    expect((await a.all()).map(row => row.id)).toEqual(['a']);
    await a.delete('b'); await a.delete('unknown');
    await expect(a.put(move('b', 'a'))).rejects.toThrow('another account');
    expect(await store.all()).toHaveLength(4);
  });
  it('stops later dispatches after an account boundary but acknowledges the original successful mutation', async () => {
    const store = memoryOutboxStore([move('one', 'a'), move('two', 'a'), move('other', 'b')]);
    let active = true;
    const jmap = vi.fn(async () => {
      active = false;
      return { methodResponses: [['Email/set', { updated: { one: null }, notUpdated: null }, 'set']], sessionState: 's' };
    });
    const result = await drainOutbox(accountOutbox(store, 'a','test-storage-authority-a'), { jmap } as unknown as Client, undefined, () => active);
    expect(result).toEqual({ sent: 1, failed: 0 });
    expect(jmap).toHaveBeenCalledOnce();
    expect((await store.all()).map(row => row.id)).toEqual(['two', 'other']);
  });
  it('does not spend retry attempts or dispatch when authority expired before draining', async () => {
    const store = memoryOutboxStore([move('one', 'a')]); const jmap = vi.fn();
    await drainOutbox(accountOutbox(store, 'a','test-storage-authority-a'), { jmap } as unknown as Client, undefined, () => false);
    expect(jmap).not.toHaveBeenCalled();
    expect(await store.all()).toEqual([move('one', 'a')]);
  });
});

it('never replays a same-ID foreign realm or unproven legacy queue',async()=>{
 const legacy=move('legacy','1');delete legacy.storageAuthority;
 const raw=memoryOutboxStore([legacy]);const a=accountOutbox(raw,'1','verified-storage-authority-a'),b=accountOutbox(raw,'1','verified-storage-authority-b');
 const item=move('owned','1');delete item.storageAuthority;await a.add(item);
 expect((await raw.all()).find(row=>row.id==='owned')?.storageAuthority).toBe('verified-storage-authority-a');
 const jmap=vi.fn();expect(await drainOutbox(b,{jmap} as unknown as Client)).toEqual({sent:0,failed:0});expect(jmap).not.toHaveBeenCalled();expect(await b.all()).toEqual([]);
 await expect(b.add(item)).rejects.toThrow('another account');await b.delete('owned');expect(await raw.all()).toHaveLength(2);expect((await a.all()).map(row=>row.id)).toEqual(['owned']);
});
