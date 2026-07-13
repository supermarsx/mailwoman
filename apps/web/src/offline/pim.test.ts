import { describe, it, expect, vi } from 'vitest';
import { type Client } from '../api/client.ts';
import type { Invocation, JmapRequest, JmapResponse } from '../api/jmap-types.ts';
import { opfsPimPath } from '../contracts/offline.ts';
import {
  drainOutbox,
  enqueuePimMutation,
  memoryOutboxStore,
  outboundApplied,
  outboundToRequest,
  pimOutbound,
} from './outbox.ts';

function fakeClient(jmap: Client['jmap']): Client {
  return {
    login: vi.fn(),
    logout: vi.fn(),
    me: vi.fn(),
    session: vi.fn(),
    jmap,
    sanitize: vi.fn(async (h: string) => h),
    onNetwork: vi.fn(() => () => undefined),
  } as unknown as Client;
}

function jmapResponse(...responses: Invocation[]): JmapResponse {
  return { methodResponses: responses, sessionState: 's1' };
}

// A representative PIM mutation: create a note.
const noteSetRequest: JmapRequest = {
  using: ['urn:mailwoman:notes'],
  methodCalls: [
    ['Note/set', { accountId: 'acct1', create: { new: { title: 'Groceries' } } }, 'set'],
  ],
};

describe('offline PIM registration', () => {
  it('lays out one AES-GCM blob per object under the pim/{type} namespace', () => {
    expect(opfsPimPath('acct1', 'Note', 'n1')).toBe('/acct1/pim/Note/n1.enc');
    expect(opfsPimPath('acct1', 'CalendarEvent', 'e9')).toBe('/acct1/pim/CalendarEvent/e9.enc');
    expect(opfsPimPath('a2', 'ContactCard', 'c-3')).toBe('/a2/pim/ContactCard/c-3.enc');
  });

  it('replays a queued PIM mutation verbatim', () => {
    const payload = pimOutbound(noteSetRequest, 'set');
    const req = outboundToRequest({ id: 'p1', type: 'pim', payload, createdAt: 0, state: 'queued' });
    expect(req).toEqual(noteSetRequest);
  });

  it('reconciles applied vs rejected PIM replays by the set response', () => {
    const applied = { id: 'p1', type: 'pim' as const, payload: pimOutbound(noteSetRequest, 'set'), createdAt: 0, state: 'queued' as const };
    const okRes = jmapResponse(['Note/set', { accountId: 'acct1', created: { new: { id: 'n1' } }, notCreated: null }, 'set']);
    const rejRes = jmapResponse(['Note/set', { accountId: 'acct1', created: null, notCreated: { new: { type: 'invalidProperties' } } }, 'set']);
    expect(outboundApplied(applied, okRes)).toBe(true);
    expect(outboundApplied(applied, rejRes)).toBe(false);
  });

  it('drains a PIM item off the queue on reconnect (type "pim")', async () => {
    const store = memoryOutboxStore();
    await enqueuePimMutation(store, noteSetRequest, 'set');
    expect((await store.all())[0]?.type).toBe('pim');

    const client = fakeClient(
      vi.fn(async () =>
        jmapResponse(['Note/set', { accountId: 'acct1', created: { new: { id: 'n1' } }, notCreated: null }, 'set']),
      ),
    );
    const result = await drainOutbox(store, client);
    expect(result).toEqual({ sent: 1, failed: 0 });
    expect(await store.all()).toHaveLength(0);
  });
});
