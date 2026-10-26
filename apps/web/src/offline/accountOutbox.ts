import type { OutboundItem } from '../contracts/offline.ts';
import { outboundToRequest, type OutboxStore } from './outbox.ts';

/** Ownership comes from every captured JMAP call, never from the current login.
 * Unknown legacy/mixed-account rows remain on disk but cannot be replayed. */
export function outboundAccount(item: OutboundItem): string | null {
  try {
    const calls = outboundToRequest(item).methodCalls;
    if (!Array.isArray(calls) || calls.length === 0) return null;
    const accounts = calls.map(call => call[1].accountId);
    const account = accounts[0];
    return typeof account === 'string' && account.length > 0 && accounts.every(id => id === account) ? account : null;
  } catch { return null; }
}

export function accountOutbox(store: OutboxStore, account: string): OutboxStore {
  const owns = (item: OutboundItem): boolean => outboundAccount(item) === account;
  return {
    async all() { return (await store.all()).filter(owns); },
    async add(item) {
      if (!owns(item)) throw new Error('Queued change does not belong to the active account');
      await store.add(item);
    },
    async put(item) {
      if (!owns(item)) throw new Error('Queued change does not belong to the active account');
      const old = (await store.all()).find(row => row.id === item.id);
      if (old && !owns(old)) throw new Error('Queued change belongs to another account');
      await store.put(item);
    },
    async delete(id) {
      if ((await store.all()).some(item => item.id === id && owns(item))) await store.delete(id);
    },
  };
}
