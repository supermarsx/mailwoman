import { describe, it, expect, vi } from 'vitest';
import { createRoot } from 'solid-js';
import { createOutboxSlice, outboxStateOf, type OutboxSlice, type OutboxSubmission } from './outbox.ts';
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

function sub(id: string, over: Partial<OutboxSubmission> = {}): OutboxSubmission {
  return { id, emailId: `e-${id}`, identityId: null, sendAt: null, undoStatus: 'pending', mailwomanHoldSeconds: 10, ...over };
}

/** The entry for one id in an `EmailSubmission/set` update response. */
type SetResult = { updated: Partial<OutboxSubmission> | null } | { notUpdated: { type: string; description?: string } };

/**
 * What the engine answers a release with when the mail server accepted the
 * message: the object `release_submission` builds in
 * `crates/mw-engine/src/jmap.rs` (the `updated` entry of `submission_set`).
 */
const RELEASED_AND_SENT: SetResult = {
  updated: {
    undoStatus: 'final', sendAt: null, mailwomanHoldSeconds: 0, mailwomanHold: null,
    mailwomanFailed: false, mailwomanAttempts: 0, mailwomanLastError: null, mailwomanNextAttemptAt: null,
  },
};
/** The same, when the mail server accepted nothing: released, pending, backing off. */
const RELEASED_NOT_ACCEPTED: SetResult = {
  updated: {
    undoStatus: 'pending', sendAt: null, mailwomanHoldSeconds: 0, mailwomanHold: null,
    mailwomanFailed: false, mailwomanAttempts: 1, mailwomanLastError: 'transport error: connection refused',
    mailwomanNextAttemptAt: '2026-01-01T00:00:30Z',
  },
};
/** `cancel_submission` / `release_submission` on a row that is no longer pending. */
const REFUSED_FINAL: SetResult = {
  notUpdated: { type: 'serverFail', description: 'backend protocol error: submission a is final and cannot be released' },
};
/** A cancel that took effect: `submission_set` inserts `null` for the id. */
const CANCELED: SetResult = { updated: null };

const IDENTITY: Identity = {
  id: 'id1', name: 'Work', email: 'work@example.org', replyTo: null,
  signatureHtml: '<b>W</b>', signatureText: 'Cheers', sentMailboxId: 'sent',
};

function makeClient(
  subs: EmailSubmission[],
  setResult: SetResult = CANCELED,
): { client: Client; jmap: ReturnType<typeof vi.fn> } {
  const jmap = vi.fn(async (body: JmapRequest): Promise<JmapResponse> => {
    const names = body.methodCalls.map((c) => c[0]);
    if (names.includes('EmailSubmission/set')) {
      const call = body.methodCalls[0]!;
      const id = Object.keys((call[1] as { update: Record<string, unknown> }).update)[0]!;
      const result =
        'updated' in setResult
          ? { accountId: 'acct1', updated: { [id]: setResult.updated } }
          : { accountId: 'acct1', updated: {}, notUpdated: { [id]: setResult.notUpdated } };
      return { methodResponses: [['EmailSubmission/set', result, call[2]]], sessionState: 's' };
    }
    if (names.includes('Email/get')) {
      const call = body.methodCalls[0]!;
      const ids = (call[1] as { ids: string[] }).ids;
      const list = ids.map((id) => ({ id, subject: `Subject of ${id}`, to: [{ name: null, email: `${id}@example.net` }] }));
      return { methodResponses: [['Email/get', { accountId: 'acct1', state: 's', list, notFound: [] }, call[2]]], sessionState: 's' };
    }
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
  setResult: SetResult = CANCELED,
): Promise<void> {
  const { client, jmap } = makeClient(subs, setResult);
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

  it('classifies a manual hold as held whatever its times say, and only while pending', () => {
    const later = new Date(now + 3_600_000).toISOString();
    expect(outboxStateOf(sub('a', { mailwomanHold: 'manual' }), now)).toBe('held');
    expect(outboxStateOf(sub('b', { mailwomanHold: 'manual', sendAt: later }), now)).toBe('held');
    expect(outboxStateOf(sub('c', { mailwomanHold: null, sendAt: later }), now)).toBe('scheduled');
    expect(outboxStateOf(sub('d', { mailwomanHold: 'manual', undoStatus: 'canceled' }), now)).toBe('canceled');
    expect(outboxStateOf(sub('e', { mailwomanHold: 'manual', undoStatus: 'final' }), now)).toBe('sent');
  });

  it('tells a submission the engine gave up on from one the user canceled', () => {
    expect(outboxStateOf(sub('a', { undoStatus: 'canceled', mailwomanFailed: true }), now)).toBe('failed');
    expect(outboxStateOf(sub('b', { undoStatus: 'canceled', mailwomanFailed: false }), now)).toBe('canceled');
  });
});

describe('outbox slice', () => {
  it('loads the submission queue', async () => {
    await withOutbox([sub('a'), sub('b', { undoStatus: 'final' })], async (o) => {
      await o.refreshOutbox();
      expect(o.outbox().map((s) => s.id)).toEqual(['a', 'b']);
    });
  });

  it('exposes the waiting submissions (held, scheduled, holding) as cancelable', async () => {
    await withOutbox(
      [
        sub('hold'),
        sub('sched', { sendAt: new Date(Date.now() + 3_600_000).toISOString() }),
        sub('held', { mailwomanHold: 'manual' }),
        sub('done', { undoStatus: 'final' }),
        sub('gone', { undoStatus: 'canceled' }),
      ],
      async (o) => {
        await o.refreshOutbox();
        expect(o.cancelableOutbox().map((s) => s.id)).toEqual(['hold', 'sched', 'held']);
      },
    );
  });

  it('loads subject and recipients for the waiting rows only', async () => {
    await withOutbox([sub('held', { mailwomanHold: 'manual' }), sub('done', { undoStatus: 'final' })], async (o, { jmap }) => {
      await o.refreshOutbox();
      expect(o.outboxMessages()).toEqual({
        'e-held': { subject: 'Subject of e-held', to: [{ name: null, email: 'e-held@example.net' }] },
      });
      const emailGet = jmap.mock.calls
        .map((call) => (call[0] as JmapRequest).methodCalls[0]!)
        .find((c) => c[0] === 'Email/get');
      expect((emailGet![1] as { ids: string[] }).ids).toEqual(['e-held']);
    });
  });

  it('cancel sends undoStatus: canceled and marks the row canceled once the server accepts', async () => {
    await withOutbox([sub('a')], async (o, { jmap, toast }) => {
      await o.refreshOutbox();
      jmap.mockClear();
      await o.cancelOutbox('a');
      const call = (jmap.mock.calls[0]![0] as JmapRequest).methodCalls[0]!;
      expect(call[0]).toBe('EmailSubmission/set');
      expect((call[1] as { update: unknown }).update).toEqual({ a: { undoStatus: 'canceled' } });
      expect(o.outbox()[0]!.undoStatus).toBe('canceled');
      expect(toast).toHaveBeenCalledWith('info', 'Send canceled');
    });
  });

  it('discarding a held row says the message was not sent', async () => {
    await withOutbox([sub('a', { mailwomanHold: 'manual' })], async (o, { toast }) => {
      await o.refreshOutbox();
      await o.cancelOutbox('a');
      expect(outboxStateOf(o.outbox()[0]!)).toBe('canceled');
      expect(toast).toHaveBeenCalledWith('info', 'Discarded. The message was not sent.');
    });
  });

  it('a cancel the server refuses leaves the row alone and says why', async () => {
    await withOutbox(
      [sub('a')],
      async (o, { toast }) => {
        await o.refreshOutbox();
        await o.cancelOutbox('a');
        expect(o.outbox()[0]!.undoStatus).toBe('pending');
        expect(toast).toHaveBeenCalledTimes(1);
        expect(toast.mock.calls[0]![0]).toBe('error');
        expect(toast.mock.calls[0]![1]).toContain('is final and cannot be');
      },
      REFUSED_FINAL,
    );
  });

  it('release sends the release patch and shows the row as sent when the server says so', async () => {
    await withOutbox(
      [sub('a', { mailwomanHold: 'manual', mailwomanOrigin: { kind: 'apiKey', name: 'abcd' } })],
      async (o, { jmap, toast }) => {
        await o.refreshOutbox();
        expect(outboxStateOf(o.outbox()[0]!)).toBe('held');
        jmap.mockClear();
        await o.sendOutboxNow('a');
        const call = (jmap.mock.calls[0]![0] as JmapRequest).methodCalls[0]!;
        expect((call[1] as { update: unknown }).update).toEqual({ a: { sendAt: null, mailwomanHoldSeconds: 0 } });
        expect(outboxStateOf(o.outbox()[0]!)).toBe('sent');
        expect(o.outbox()[0]!.mailwomanHold).toBeNull();
        // The origin is not in the update and must survive it.
        expect(o.outbox()[0]!.mailwomanOrigin).toEqual({ kind: 'apiKey', name: 'abcd' });
        expect(toast).toHaveBeenCalledWith('success', 'Sent');
      },
      RELEASED_AND_SENT,
    );
  });

  it('a release the mail server did not accept is not reported as sent', async () => {
    await withOutbox(
      [sub('a', { mailwomanHold: 'manual' })],
      async (o, { toast }) => {
        await o.refreshOutbox();
        await o.sendOutboxNow('a');
        const row = o.outbox()[0]!;
        expect(row.undoStatus).toBe('pending');
        expect(outboxStateOf(row)).toBe('holding');
        expect(row.mailwomanLastError).toContain('connection refused');
        expect(toast).toHaveBeenCalledTimes(1);
        expect(toast.mock.calls[0]![0]).toBe('error');
        expect(toast.mock.calls[0]![1]).toContain('connection refused');
      },
      RELEASED_NOT_ACCEPTED,
    );
  });

  it('a release the server refuses changes nothing and says why', async () => {
    await withOutbox(
      [sub('a', { mailwomanHold: 'manual' })],
      async (o, { toast }) => {
        await o.refreshOutbox();
        await o.sendOutboxNow('a');
        expect(outboxStateOf(o.outbox()[0]!)).toBe('held');
        expect(toast.mock.calls[0]![0]).toBe('error');
        expect(toast.mock.calls[0]![1]).toContain('cannot be released');
      },
      REFUSED_FINAL,
    );
  });

  it('loads sending identities', async () => {
    await withOutbox([], async (o) => {
      await o.loadIdentities();
      expect(o.identities().map((i) => i.email)).toEqual(['work@example.org']);
    });
  });
});
