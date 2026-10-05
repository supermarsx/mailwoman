import { describe, it, expect, vi } from 'vitest';
import { NetworkError, type Client } from '../api/client.ts';
import type { Invocation, JmapResponse } from '../api/jmap-types.ts';
import type { OutboundItem } from '../contracts/offline.ts';
import {
  MAX_ATTEMPTS,
  discardOutbound,
  drainOutbox,
  enqueueOutbound,
  failedOutbound,
  memoryOutboxStore,
  outboundApplied,
  outboundRejection,
  outboundToRequest,
  removeQueued,
  retryOutbound,
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

  it('a queued reply replays with its Cc, Bcc and thread ids, as a draft and as a send', () => {
    const reply = {
      accountId: 'acct1',
      draft: {
        from: { name: null, email: 'me@example.org' },
        to: 'alice@example.org',
        cc: 'bob@example.org',
        bcc: 'carol@example.org',
        inReplyTo: ['orig@example.org'],
        references: ['root@example.org', 'orig@example.org'],
        subject: 'Re: Hi',
        htmlBody: '<p>x</p>',
        draftMailboxId: 'drafts',
      },
    };
    const created = {
      mailboxIds: { drafts: true },
      keywords: { $draft: true, $seen: true },
      from: [{ name: null, email: 'me@example.org' }],
      to: [{ name: null, email: 'alice@example.org' }],
      cc: [{ name: null, email: 'bob@example.org' }],
      bcc: [{ name: null, email: 'carol@example.org' }],
      inReplyTo: ['orig@example.org'],
      references: ['root@example.org', 'orig@example.org'],
      subject: 'Re: Hi',
      htmlBody: [{ partId: 'body', type: 'text/html' }],
      bodyValues: { body: { value: '<p>x</p>' } },
    };
    expect(outboundToRequest(item('draft', reply)).methodCalls).toEqual([
      ['Email/set', { accountId: 'acct1', create: { draft: created } }, 'set'],
    ]);
    expect(outboundToRequest(item('send', reply)).methodCalls).toEqual([
      ['Email/set', { accountId: 'acct1', create: { draft: created } }, 'set'],
      [
        'EmailSubmission/set',
        {
          accountId: 'acct1',
          create: {
            send: {
              emailId: '#draft',
              envelope: {
                mailFrom: { email: 'me@example.org' },
                rcptTo: [{ email: 'alice@example.org' }, { email: 'bob@example.org' }, { email: 'carol@example.org' }],
              },
            },
          },
        },
        'submit',
      ],
    ]);
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

  it('marks a server-refused item failed with the reason, and never replays it again', async () => {
    const queued = item('flag', flag, 1);
    const store = memoryOutboxStore([queued]);
    const client = fakeClient(
      vi.fn(async () =>
        setResponse({ notUpdated: { m1: { type: 'notFound', description: 'no such message' } } }),
      ),
    );
    expect(await drainOutbox(store, client)).toEqual({ sent: 0, failed: 1 });
    expect(await store.all()).toEqual([{ ...queued, state: 'failed', attempts: 1, lastError: 'no such message' }]);

    // A second reconnect must not send the refused mutation again.
    expect(await drainOutbox(store, client)).toEqual({ sent: 0, failed: 0 });
    expect(client.jmap).toHaveBeenCalledTimes(1);
  });

  it('retries an item whose replay errors, up to MAX_ATTEMPTS, then gives up', async () => {
    const queued = item('move', move, 1);
    const store = memoryOutboxStore([queued]);
    const client = fakeClient(
      vi.fn(async () => {
        throw new Error('HTTP 503');
      }),
    );
    // Precondition for the loop below: the cap leaves room for a retry.
    expect(MAX_ATTEMPTS).toBe(3);

    expect(await drainOutbox(store, client)).toEqual({ sent: 0, failed: 0 });
    expect(await store.all()).toEqual([{ ...queued, state: 'queued', attempts: 1, lastError: 'HTTP 503' }]);
    expect(await drainOutbox(store, client)).toEqual({ sent: 0, failed: 0 });
    expect(await store.all()).toEqual([{ ...queued, state: 'queued', attempts: 2, lastError: 'HTTP 503' }]);
    expect(await drainOutbox(store, client)).toEqual({ sent: 0, failed: 1 });
    expect(await store.all()).toEqual([{ ...queued, state: 'failed', attempts: 3, lastError: 'HTTP 503' }]);

    // Given up on: a fourth drain does not touch it.
    expect(await drainOutbox(store, client)).toEqual({ sent: 0, failed: 0 });
    expect(client.jmap).toHaveBeenCalledTimes(3);
  });

  it('an item that errored once is applied on the next drain and leaves the queue', async () => {
    const store = memoryOutboxStore([item('flag', flag, 1)]);
    const jmap = vi
      .fn<Client['jmap']>()
      .mockRejectedValueOnce(new Error('HTTP 502'))
      .mockResolvedValueOnce(setResponse({ updated: { m1: null } }));
    const client = fakeClient(jmap);
    expect(await drainOutbox(store, client)).toEqual({ sent: 0, failed: 0 });
    expect(await store.all()).toHaveLength(1);
    expect(await drainOutbox(store, client)).toEqual({ sent: 1, failed: 0 });
    expect(await store.all()).toEqual([]);
  });

  it('one bad item does not hold up the ones behind it', async () => {
    const store = memoryOutboxStore([item('flag', flag, 1), item('move', move, 2)]);
    const jmap = vi
      .fn<Client['jmap']>()
      .mockResolvedValueOnce(setResponse({ notUpdated: { m1: { type: 'forbidden' } } }))
      .mockResolvedValueOnce(setResponse({ updated: { m1: null } }));
    expect(await drainOutbox(store, fakeClient(jmap))).toEqual({ sent: 1, failed: 1 });
    expect((await store.all()).map((i) => [i.type, i.state])).toEqual([['flag', 'failed']]);
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
    // Only the first item was attempted; both remain queued (FIFO preserved),
    // and being offline is not counted as an attempt.
    expect(client.jmap).toHaveBeenCalledTimes(1);
    expect(await store.all()).toEqual([item('flag', flag, 1), item('move', move, 2)]);
  });
});

describe('drainOutbox — one drain at a time', () => {
  // Two things call for a drain on reconnect, at the same moment: the client's
  // first successful request and the browser's `online` event. Each drain reads
  // the queue before the other has deleted anything.
  it('two drains started together send each item once', async () => {
    const store = memoryOutboxStore([item('send', send, 1), item('flag', flag, 2)]);
    const client = fakeClient(
      vi.fn(async (body) => {
        await new Promise((r) => setTimeout(r, 5));
        return body.methodCalls.length === 2
          ? jmapResponse(
              ['Email/set', { created: { draft: { id: 'e9' } } }, 'set'],
              ['EmailSubmission/set', { created: { send: { id: 's1' } }, notCreated: null }, 'submit'],
            )
          : setResponse({ updated: { m1: null } });
      }),
    );
    const [first, second] = await Promise.all([drainOutbox(store, client), drainOutbox(store, client)]);
    expect(client.jmap).toHaveBeenCalledTimes(2);
    expect(first).toEqual({ sent: 2, failed: 0 });
    // The second caller is told what the one drain did, not a second tally.
    expect(second).toEqual({ sent: 2, failed: 0 });
    expect(await store.all()).toEqual([]);
  });

  it('a drain started after the first finished runs normally', async () => {
    const store = memoryOutboxStore([item('flag', flag, 1)]);
    const client = fakeClient(vi.fn(async () => setResponse({ updated: { m1: null } })));
    expect(await drainOutbox(store, client)).toEqual({ sent: 1, failed: 0 });
    await store.add(item('move', move, 2));
    expect(await drainOutbox(store, client)).toEqual({ sent: 1, failed: 0 });
    expect(client.jmap).toHaveBeenCalledTimes(2);
  });

  it('skips the drain when another tab holds the queue lock', async () => {
    const store = memoryOutboxStore([item('flag', flag, 1)]);
    const client = fakeClient(vi.fn(async () => setResponse({ updated: { m1: null } })));
    // Web Locks with `ifAvailable`: the callback gets `null` when the lock is taken.
    vi.stubGlobal('navigator', {
      locks: { request: async (_n: string, _o: unknown, cb: (lock: unknown) => unknown) => cb(null) },
    });
    try {
      expect(await drainOutbox(store, client)).toEqual({ sent: 0, failed: 0 });
      expect(client.jmap).not.toHaveBeenCalled();
      expect(await store.all()).toHaveLength(1);
    } finally {
      vi.unstubAllGlobals();
    }
  });
});

describe('outboundRejection', () => {
  it('reads the SetError for the item, preferring its description', () => {
    expect(
      outboundRejection(item('move', move), setResponse({ notUpdated: { m1: { type: 'notFound' } } })),
    ).toBe('notFound');
    const refusedSend = jmapResponse(
      [
        'Email/set',
        { created: null, notCreated: { draft: { type: 'invalidProperties', description: 'to: "bob": no @' } } },
        'set',
      ],
      ['EmailSubmission/set', { created: null, notCreated: { send: { type: 'invalidProperties' } } }, 'submit'],
    );
    expect(outboundRejection(item('send', send), refusedSend)).toBe('to: "bob": no @');
    const refusedSubmission = jmapResponse(
      ['Email/set', { created: { draft: { id: 'e9' } }, notCreated: null }, 'set'],
      [
        'EmailSubmission/set',
        { created: null, notCreated: { send: { type: 'invalidProperties', description: 'sendAt: is in the past' } } },
        'submit',
      ],
    );
    expect(outboundRejection(item('send', send), refusedSubmission)).toBe('sendAt: is in the past');
  });

  it('is null when the response names no error for the item', () => {
    expect(outboundRejection(item('flag', flag), setResponse({ updated: { m1: null } }))).toBeNull();
  });
});

describe('queue management', () => {
  const failed = (id: string, createdAt: number): OutboundItem & { attempts: number; lastError: string } => ({
    ...item('flag', flag, createdAt),
    id,
    state: 'failed',
    attempts: 3,
    lastError: 'HTTP 503',
  });

  it('removeQueued takes an unsent item out, so a later drain sends nothing for it', async () => {
    const store = memoryOutboxStore();
    const queued = await enqueueOutbound(store, { type: 'move', payload: move });
    const client = fakeClient(vi.fn(async () => setResponse({ updated: { m1: null } })));

    expect(await removeQueued(store, queued.id)).toBe(true);
    expect(await store.all()).toEqual([]);
    expect(await drainOutbox(store, client)).toEqual({ sent: 0, failed: 0 });
    expect(client.jmap).not.toHaveBeenCalled();
  });

  it('removeQueued reports false for an item the queue no longer holds', async () => {
    const store = memoryOutboxStore();
    const queued = await enqueueOutbound(store, { type: 'move', payload: move });
    await drainOutbox(store, fakeClient(vi.fn(async () => setResponse({ updated: { m1: null } }))));
    expect(await removeQueued(store, queued.id)).toBe(false);
  });

  it('failedOutbound lists only the items given up on, oldest first', async () => {
    const store = memoryOutboxStore([failed('f2', 5), item('move', move, 3), failed('f1', 1)]);
    expect((await failedOutbound(store)).map((i) => i.id)).toEqual(['f1', 'f2']);
  });

  it('retryOutbound re-queues a failed item with a fresh count, and a drain then sends it', async () => {
    const store = memoryOutboxStore([failed('f1', 1)]);
    const client = fakeClient(vi.fn(async () => setResponse({ updated: { m1: null } })));
    // Precondition: as it stands, a drain leaves the failed item alone.
    expect(await drainOutbox(store, client)).toEqual({ sent: 0, failed: 0 });
    expect(client.jmap).not.toHaveBeenCalled();

    expect(await retryOutbound(store, 'f1')).toBe(true);
    expect(await store.all()).toEqual([{ ...item('flag', flag, 1), id: 'f1', state: 'queued', attempts: 0 }]);
    expect(await drainOutbox(store, client)).toEqual({ sent: 1, failed: 0 });
    expect(await store.all()).toEqual([]);
  });

  it('retryOutbound does nothing for an id that is not a failed item', async () => {
    const queued = item('flag', flag, 1);
    const store = memoryOutboxStore([queued]);
    expect(await retryOutbound(store, queued.id)).toBe(false);
    expect(await retryOutbound(store, 'nope')).toBe(false);
    expect(await store.all()).toEqual([queued]);
  });

  it('discardOutbound removes a failed item without sending it', async () => {
    const store = memoryOutboxStore([failed('f1', 1), failed('f2', 2)]);
    await discardOutbound(store, 'f1');
    expect((await store.all()).map((i) => i.id)).toEqual(['f2']);
  });
});
