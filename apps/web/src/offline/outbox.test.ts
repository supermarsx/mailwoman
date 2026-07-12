import { describe, it, expect, vi } from 'vitest';
import { NetworkError, type Client } from '../api/client.ts';
import type { Invocation, JmapResponse } from '../api/jmap-types.ts';
import type { OutboundItem } from '../contracts/offline.ts';
import {
  drainOutbox,
  enqueueOutbound,
  memoryOutboxStore,
  outboundApplied,
  outboundToRequest,
  type DraftPayload,
  type FlagPayload,
  type MovePayload,
  type SendPayload,
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

function setResponse(args: Record<string, unknown>): JmapResponse {
  return jmapResponse([
    'Email/set',
    { accountId: 'acct1', created: null, updated: null, notCreated: null, notUpdated: null, ...args },
    'set',
  ]);
}

const flag: FlagPayload = { accountId: 'acct1', emailId: 'm1', keyword: '$flagged', value: true };
const move: MovePayload = { accountId: 'acct1', emailId: 'm1', mailboxIds: { archive: true } };
const draftInput = {
  from: { name: null, email: 'me@x.org' },
  to: 'you@y.org',
  subject: 'Hi',
  htmlBody: '<p>hi</p>',
  draftMailboxId: 'drafts1',
};
const send: SendPayload = { accountId: 'acct1', draft: draftInput };
const draft: DraftPayload = { accountId: 'acct1', draft: draftInput };

function item(type: OutboundItem['type'], payload: unknown, createdAt = 0): OutboundItem {
  return { id: `${type}-${createdAt}`, type, payload, createdAt, state: 'queued' };
}

describe('enqueueOutbound', () => {
  it('appends a queued item with an id + timestamp', async () => {
    const store = memoryOutboxStore();
    const created = await enqueueOutbound(store, { type: 'flag', payload: flag });
    expect(created.state).toBe('queued');
    expect(created.id).toBeTruthy();
    expect(created.createdAt).toBeGreaterThan(0);
    expect(await store.all()).toHaveLength(1);
  });
});

describe('outboundToRequest', () => {
  it('flag → Email/set keyword patch', () => {
    const [call] = outboundToRequest(item('flag', flag)).methodCalls as [Invocation];
    expect(call[0]).toBe('Email/set');
    expect(call[1]['update']).toEqual({ m1: { 'keywords/$flagged': true } });
  });

  it('flag value:false → keyword removal (null)', () => {
    const [call] = outboundToRequest(item('flag', { ...flag, value: false })).methodCalls as [Invocation];
    expect(call[1]['update']).toEqual({ m1: { 'keywords/$flagged': null } });
  });

  it('move → Email/set mailboxIds patch', () => {
    const [call] = outboundToRequest(item('move', move)).methodCalls as [Invocation];
    expect(call[1]['update']).toEqual({ m1: { mailboxIds: { archive: true } } });
  });

  it('draft → Email/set create with $draft keyword, no submission', () => {
    const req = outboundToRequest(item('draft', draft));
    expect(req.methodCalls).toHaveLength(1);
    const [call] = req.methodCalls as [Invocation];
    const create = call[1]['create'] as Record<string, Record<string, unknown>>;
    expect(create['draft']!['keywords']).toMatchObject({ $draft: true });
    expect(create['draft']!['subject']).toBe('Hi');
  });

  it('send → Email/set + EmailSubmission/set (compose + submit)', () => {
    const req = outboundToRequest(item('send', send));
    const names = req.methodCalls.map((c) => c[0]);
    expect(names).toEqual(['Email/set', 'EmailSubmission/set']);
  });
});

describe('outboundApplied', () => {
  it('flag applied when the id is in updated', () => {
    expect(outboundApplied(item('flag', flag), setResponse({ updated: { m1: null } }))).toBe(true);
  });
  it('flag not applied when the id is in notUpdated', () => {
    expect(
      outboundApplied(item('flag', flag), setResponse({ notUpdated: { m1: { type: 'notFound' } } })),
    ).toBe(false);
  });
  it('draft applied when created.draft exists', () => {
    expect(outboundApplied(item('draft', draft), setResponse({ created: { draft: { id: 'e9' } } }))).toBe(true);
  });
  it('send applied when the submission is created', () => {
    const res = jmapResponse(
      ['Email/set', { created: { draft: { id: 'e9' } } }, 'set'],
      ['EmailSubmission/set', { created: { send: { id: 's1' } }, notCreated: null }, 'submit'],
    );
    expect(outboundApplied(item('send', send), res)).toBe(true);
  });
});

describe('drainOutbox', () => {
  it('replays FIFO, deleting applied items and counting them sent', async () => {
    const store = memoryOutboxStore([item('flag', flag, 1), item('move', move, 2)]);
    const seen: string[] = [];
    const client = fakeClient(
      vi.fn(async (body) => {
        const update = (body.methodCalls[0]![1]['update'] ?? {}) as Record<string, unknown>;
        seen.push(Object.keys(update)[0]!);
        return setResponse({ updated: { m1: null } });
      }),
    );
    const result = await drainOutbox(store, client);
    expect(result).toEqual({ sent: 2, failed: 0 });
    expect(await store.all()).toHaveLength(0);
    // Oldest first.
    expect(seen).toEqual(['m1', 'm1']);
    expect(client.jmap).toHaveBeenCalledTimes(2);
  });

  it('marks a server-rejected item failed and keeps it', async () => {
    const store = memoryOutboxStore([item('flag', flag, 1)]);
    const client = fakeClient(vi.fn(async () => setResponse({ notUpdated: { m1: { type: 'forbidden' } } })));
    const result = await drainOutbox(store, client);
    expect(result).toEqual({ sent: 0, failed: 1 });
    const [remaining] = await store.all();
    expect(remaining!.state).toBe('failed');
  });

  it('stops on a network error, leaving the item queued for the next reconnect', async () => {
    const store = memoryOutboxStore([item('flag', flag, 1), item('move', move, 2)]);
    const client = fakeClient(
      vi.fn(async () => {
        throw new NetworkError('offline');
      }),
    );
    const result = await drainOutbox(store, client);
    expect(result).toEqual({ sent: 0, failed: 0 });
    // Only the first item was attempted; both remain queued (FIFO preserved).
    expect(client.jmap).toHaveBeenCalledTimes(1);
    const all = await store.all();
    expect(all).toHaveLength(2);
    expect(all.every((i) => i.state === 'queued')).toBe(true);
  });
});
