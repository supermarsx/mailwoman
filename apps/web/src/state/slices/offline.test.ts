import { describe, it, expect, vi } from 'vitest';
import { createRoot } from 'solid-js';
import { createOfflineSlice, type OfflineSlice } from './offline.ts';
import type { SliceContext } from './context.ts';
import type { Client } from '../../api/client.ts';
import type { JmapRequest, JmapResponse } from '../../api/jmap-types.ts';
import { memoryOutboxStore, type MovePayload, type OutboxStore, type SendPayload } from '../../offline/outbox.ts';
import { createAppState } from '../store.ts';
import { makeClient, mkEmail } from '../../components/appHarness.tsx';

const move: MovePayload = { accountId: 'acct1', emailId: 'm1', mailboxIds: { archive: true } };
const send: SendPayload = {
  accountId: 'acct1',
  draft: {
    from: { name: null, email: 'me@example.org' },
    to: 'you@example.org',
    subject: 'Hi',
    htmlBody: '<p>hi</p>',
    draftMailboxId: 'drafts1',
  },
};

const applied = (): JmapResponse => ({
  methodResponses: [['Email/set', { updated: { m1: null }, notUpdated: null }, 'set']],
  sessionState: 's',
});
const refused = (): JmapResponse => ({
  methodResponses: [
    ['Email/set', { updated: null, notUpdated: { m1: { type: 'notFound', description: 'no such message' } } }, 'set'],
  ],
  sessionState: 's',
});
const sentOk = (): JmapResponse => ({
  methodResponses: [
    ['Email/set', { created: { draft: { id: 'e1' } }, notCreated: null }, 'set'],
    ['EmailSubmission/set', { created: { send: { id: 's1' } }, notCreated: null }, 'submit'],
  ],
  sessionState: 's',
});

interface Rig {
  offline: OfflineSlice;
  store: OutboxStore;
  jmap: ReturnType<typeof vi.fn<Client['jmap']>>;
  toast: ReturnType<typeof vi.fn>;
  /** Report a network state change the way the client does. */
  network(up: boolean): void;
}

async function withSlice(run: (rig: Rig) => Promise<void>): Promise<void> {
  const listeners: ((up: boolean) => void)[] = [];
  const jmap = vi.fn<Client['jmap']>(async () => applied());
  const client = {
    jmap,
    onNetwork: (l: (up: boolean) => void) => {
      listeners.push(l);
      return () => undefined;
    },
  } as unknown as Client;
  const toast = vi.fn();
  const ctx: SliceContext = { client, showToast: toast };
  const store = memoryOutboxStore();
  await createRoot(async (dispose) => {
    const offline = createOfflineSlice(ctx, { store });
    await run({ offline, store, jmap, toast, network: (up) => listeners.forEach((l) => l(up)) });
    dispose();
  });
}

/** The compose+submit requests the client was handed. */
function submissions(jmap: Rig['jmap']): JmapRequest[] {
  return jmap.mock.calls.map((c) => c[0]).filter((r) => r.methodCalls.some((m) => m[0] === 'EmailSubmission/set'));
}

describe('offline slice — queue and undo', () => {
  it('enqueue resolves to the item id and counts it; dequeue removes it before it is sent', async () => {
    await withSlice(async ({ offline, store, jmap }) => {
      const id = await offline.enqueueOffline('move', move);
      expect((await store.all()).map((i) => i.id)).toEqual([id]);
      expect(offline.offlineQueuePending()).toBe(1);

      expect(await offline.dequeueOffline(id)).toBe(true);
      expect(offline.offlineQueuePending()).toBe(0);
      expect(await offline.replayOffline()).toEqual({ sent: 0, failed: 0 });
      expect(jmap).not.toHaveBeenCalled();
    });
  });

  it('dequeue says false once the item has been replayed', async () => {
    await withSlice(async ({ offline, jmap }) => {
      const id = await offline.enqueueOffline('move', move);
      await offline.replayOffline();
      expect(jmap).toHaveBeenCalledTimes(1);
      expect(await offline.dequeueOffline(id)).toBe(false);
    });
  });
});

describe('offline slice — reconnect', () => {
  it('the network edge and the browser online event together submit a queued send exactly once', async () => {
    await withSlice(async ({ offline, store, jmap, toast, network }) => {
      jmap.mockImplementation(async () => {
        await new Promise((r) => setTimeout(r, 5));
        return sentOk();
      });
      await offline.enqueueOffline('send', send);
      network(false);
      // Precondition: nothing has been sent while offline.
      expect(jmap).not.toHaveBeenCalled();

      // Both triggers, in the same tick, as a real reconnect produces them.
      network(true);
      window.dispatchEvent(new Event('online'));

      await vi.waitFor(async () => expect(await store.all()).toEqual([]));
      await vi.waitFor(() => expect(offline.offlineQueuePending()).toBe(0));
      expect(submissions(jmap)).toHaveLength(1);
      expect(jmap).toHaveBeenCalledTimes(1);
      expect(toast).toHaveBeenCalledWith('success', 'Sent 1 queued change');
    });
  });
});

describe('offline slice — failed items', () => {
  it('a refused item leaves the pending count and is listed with its reason', async () => {
    await withSlice(async ({ offline, jmap, toast }) => {
      jmap.mockResolvedValueOnce(refused());
      const id = await offline.enqueueOffline('move', move);
      // Precondition: it is pending, not failed, before the replay.
      expect(offline.offlineQueuePending()).toBe(1);
      expect(offline.offlineFailed()).toEqual([]);

      expect(await offline.replayOffline()).toEqual({ sent: 0, failed: 1 });

      expect(offline.offlineQueuePending()).toBe(0);
      expect(offline.offlineFailed().map((i) => [i.id, i.type, i.state, i.lastError])).toEqual([
        [id, 'move', 'failed', 'no such message'],
      ]);
      expect(toast).toHaveBeenCalledWith('error', '1 queued change failed');
    });
  });

  it('retry replays it at once; when the server now applies it, it leaves the list', async () => {
    await withSlice(async ({ offline, store, jmap, toast }) => {
      jmap.mockResolvedValueOnce(refused());
      const id = await offline.enqueueOffline('move', move);
      await offline.replayOffline();
      expect(offline.offlineFailed()).toHaveLength(1);
      // A plain replay leaves a failed item alone.
      await offline.replayOffline();
      expect(jmap).toHaveBeenCalledTimes(1);

      await offline.retryOffline(id);

      expect(jmap).toHaveBeenCalledTimes(2);
      expect(offline.offlineFailed()).toEqual([]);
      expect(await store.all()).toEqual([]);
      expect(toast).toHaveBeenLastCalledWith('success', 'Sent 1 queued change');
    });
  });

  it('discard drops it without sending it again', async () => {
    await withSlice(async ({ offline, store, jmap, toast }) => {
      jmap.mockResolvedValueOnce(refused());
      const id = await offline.enqueueOffline('move', move);
      await offline.replayOffline();
      expect(offline.offlineFailed()).toHaveLength(1);

      await offline.discardOffline(id);

      expect(offline.offlineFailed()).toEqual([]);
      expect(await store.all()).toEqual([]);
      expect(jmap).toHaveBeenCalledTimes(1);
      expect(toast).toHaveBeenLastCalledWith('info', 'Discarded. The change was not applied.');
    });
  });
});

describe('the composed store — an offline move and its undo', () => {
  it('undo takes the queued move out again: nothing pending, nothing sent', async () => {
    const client = makeClient({ emails: [mkEmail('a'), mkEmail('b')] });
    await createRoot(async (dispose) => {
      const app = createAppState(client);
      await app.login({ jmapUrl: 'x', username: 'me@example.org', password: 'p' });
      // Go offline the way the client reports it.
      for (const call of vi.mocked(client.onNetwork).mock.calls) call[0](false);
      expect(app.online()).toBe(false);
      vi.mocked(client.jmap).mockClear();

      await app.archiveMessage('a');
      // Precondition: the move went into the offline queue, not to the server.
      expect(app.offlineQueuePending()).toBe(1);
      expect(app.messages().map((m) => m.id)).toEqual(['b']);

      await app.undoNow();

      expect(app.offlineQueuePending()).toBe(0);
      expect(app.messages().map((m) => m.id)).toEqual(['a', 'b']);
      expect(await app.replayOffline()).toEqual({ sent: 0, failed: 0 });
      expect(client.jmap).not.toHaveBeenCalled();
      dispose();
    });
  });
});
