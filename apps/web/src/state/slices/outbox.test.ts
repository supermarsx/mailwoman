import { describe, it, expect, vi } from 'vitest';
import { createRoot } from 'solid-js';
import { createOutboxSlice, outboxStateOf, type OutboxSlice } from './outbox.ts';
import type { SliceContext } from './context.ts';
import type { Client } from '../../api/client.ts';
import {
  CAP_MAIL,
  type EmailSubmission,
  type Identity,
  type JmapRequest,
  type JmapResponse,
  type JmapSession,
} from '../../api/jmap-types.ts';

const SESSION: JmapSession = {
  capabilities: {},
  accounts: { acct1: { name: 'T', isPersonal: true, isReadOnly: false, accountCapabilities: {} } },
  primaryAccounts: { [CAP_MAIL]: 'acct1' },
  username: 'me@example.org',
  apiUrl: '/a', downloadUrl: '/d', uploadUrl: '/u', eventSourceUrl: '/e', state: 's0',
};

function sub(id: string, over: Partial<EmailSubmission> = {}): EmailSubmission {
  return { id, emailId: `e-${id}`, identityId: null, sendAt: null, undoStatus: 'pending', mailwomanHoldSeconds: 10, ...over };
}

const IDENTITY: Identity = {
  id: 'id1', name: 'Work', email: 'work@example.org', replyTo: null,
  signatureHtml: '<b>W</b>', signatureText: 'Cheers', sentMailboxId: 'sent',
};

function makeClient(subs: EmailSubmission[]): { client: Client; jmap: ReturnType<typeof vi.fn> } {
  const jmap = vi.fn(async (body: JmapRequest): Promise<JmapResponse> => {
    const names = body.methodCalls.map((c) => c[0]);
    if (names.includes('EmailSubmission/query')) {
      return {
        methodResponses: [
          ['EmailSubmission/query', { accountId: 'acct1', ids: subs.map((s) => s.id) }, 'q'],
          ['EmailSubmission/get', { accountId: 'acct1', state: 's', list: subs, notFound: [] }, 'g'],
        ],
        sessionState: 's',
      };
    }
    if (names.includes('Identity/get')) {
      return { methodResponses: [['Identity/get', { accountId: 'acct1', state: 's', list: [IDENTITY], notFound: [] }, 'i']], sessionState: 's' };
    }
    return { methodResponses: body.methodCalls.map((c) => [c[0], {}, c[2]] as JmapResponse['methodResponses'][number]), sessionState: 's' };
  });
  const client = {
    login: vi.fn(), logout: vi.fn(), me: vi.fn(),
    session: vi.fn(async () => SESSION),
    jmap, sanitize: vi.fn(async (h: string) => h), onNetwork: vi.fn(() => () => undefined),
  } as unknown as Client;
  return { client, jmap };
}

function withOutbox(
  subs: EmailSubmission[],
  run: (o: OutboxSlice, ctx: { jmap: ReturnType<typeof vi.fn>; toast: ReturnType<typeof vi.fn> }) => Promise<void>,
): Promise<void> {
  const { client, jmap } = makeClient(subs);
  const toast = vi.fn();
  const ctx: SliceContext = { client, showToast: toast };
  return createRoot(async (dispose) => {
    const o = createOutboxSlice(ctx);
    await run(o, { jmap, toast });
    dispose();
  });
}

describe('outboxStateOf', () => {
  const now = Date.UTC(2026, 0, 1);
  it('classifies canceled / final / scheduled / holding', () => {
    expect(outboxStateOf(sub('a', { undoStatus: 'canceled' }), now)).toBe('canceled');
    expect(outboxStateOf(sub('b', { undoStatus: 'final' }), now)).toBe('sent');
    expect(outboxStateOf(sub('c', { sendAt: new Date(now + 3_600_000).toISOString() }), now)).toBe('scheduled');
    expect(outboxStateOf(sub('d', { sendAt: null }), now)).toBe('holding');
    expect(outboxStateOf(sub('e', { sendAt: new Date(now - 1000).toISOString() }), now)).toBe('holding');
  });
});

describe('outbox slice', () => {
  it('loads the submission queue', async () => {
    await withOutbox([sub('a'), sub('b', { undoStatus: 'final' })], async (o) => {
      await o.refreshOutbox();
      expect(o.outbox().map((s) => s.id)).toEqual(['a', 'b']);
    });
  });

  it('exposes only cancelable (scheduled/holding) submissions', async () => {
    await withOutbox(
      [sub('hold'), sub('sched', { sendAt: new Date(Date.now() + 3_600_000).toISOString() }), sub('done', { undoStatus: 'final' })],
      async (o) => {
        await o.refreshOutbox();
        expect(o.cancelableOutbox().map((s) => s.id)).toEqual(['hold', 'sched']);
      },
    );
  });

  it('cancel flips the row to canceled and calls EmailSubmission/set', async () => {
    await withOutbox([sub('a')], async (o, { jmap }) => {
      await o.refreshOutbox();
      jmap.mockClear();
      await o.cancelOutbox('a');
      expect(o.outbox()[0]!.undoStatus).toBe('canceled');
      expect(jmap.mock.calls.some((call) => (call[0] as JmapRequest).methodCalls[0]![0] === 'EmailSubmission/set')).toBe(true);
    });
  });

  it('send-now clears the row delay', async () => {
    await withOutbox([sub('a', { sendAt: new Date(Date.now() + 3_600_000).toISOString() })], async (o) => {
      await o.refreshOutbox();
      await o.sendOutboxNow('a');
      expect(o.outbox()[0]!.sendAt).toBeNull();
      expect(o.outbox()[0]!.mailwomanHoldSeconds).toBe(0);
    });
  });

  it('loads sending identities', async () => {
    await withOutbox([], async (o) => {
      await o.loadIdentities();
      expect(o.identities().map((i) => i.email)).toEqual(['work@example.org']);
    });
  });
});
