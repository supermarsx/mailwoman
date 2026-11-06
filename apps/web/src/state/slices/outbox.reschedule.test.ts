import { describe, it, expect, vi } from 'vitest';
import { createRoot } from 'solid-js';
import { createOutboxSlice, type OutboxSlice, type OutboxSubmission } from './outbox.ts';
import type { SliceContext } from './context.ts';
import type { Client } from '../../api/client.ts';
import { CAP_MAIL, type JmapRequest, type JmapResponse, type JmapSession } from '../../api/jmap-types.ts';

// The slice's edit-send-time, send-again and delete paths against stubbed JMAP
// answers shaped like the engine's (t28_submission_hold.rs asserts the engine side).

const SESSION: JmapSession = {
  capabilities: {},
  accounts: { acct1: { name: 'T', isPersonal: true, isReadOnly: false, accountCapabilities: {} } },
  primaryAccounts: { [CAP_MAIL]: 'acct1' },
  username: 'me@example.org',
  apiUrl: '/a', downloadUrl: '/d', uploadUrl: '/u', eventSourceUrl: '/e', state: 's0',
};
const sub = (id: string, over: Partial<OutboxSubmission> = {}): OutboxSubmission => ({ id, emailId: `e-${id}`, identityId: 'id1', sendAt: null, undoStatus: 'pending', mailwomanHoldSeconds: 0, ...over });

type SetAnswer = Record<string, unknown>;
function run(
  initial: OutboxSubmission[],
  answers: { submissionSet?: SetAnswer; emailSet?: SetAnswer; later?: OutboxSubmission[] },
  body: (o: OutboxSlice, ctx: { jmap: ReturnType<typeof vi.fn>; toast: ReturnType<typeof vi.fn>; setQueue: (subs: OutboxSubmission[]) => void }) => Promise<void>,
): Promise<void> {
  let queue = initial;
  const jmap = vi.fn(async (req: JmapRequest): Promise<JmapResponse> => {
    const call = req.methodCalls[0]!;
    if (call[0] === 'EmailSubmission/set') return { methodResponses: [['EmailSubmission/set', { accountId: 'acct1', ...answers.submissionSet }, call[2]]], sessionState: 's' };
    if (call[0] === 'Email/set') return { methodResponses: [['Email/set', { accountId: 'acct1', ...answers.emailSet }, call[2]]], sessionState: 's' };
    if (call[0] === 'Email/get') {
      const email = { id: 'e-a', subject: 'Hi', from: [], to: [{ name: null, email: 'x@example.net' }], cc: [], textBody: [{ partId: '1' }, { partId: '2' }], bodyValues: { '1': { value: 'one' }, '2': { value: 'two' } } };
      return { methodResponses: [['Email/get', { accountId: 'acct1', state: 's', list: [email], notFound: [] }, call[2]]], sessionState: 's' };
    }
    return {
      methodResponses: [
        ['EmailSubmission/query', { accountId: 'acct1', ids: queue.map((s) => s.id) }, 'q'],
        ['EmailSubmission/get', { accountId: 'acct1', state: 's', list: queue, notFound: [] }, 'g'],
      ],
      sessionState: 's',
    };
  });
  const client = { session: vi.fn(async () => SESSION), jmap } as unknown as Client;
  const toast = vi.fn();
  const ctx: SliceContext = { client, showToast: toast };
  return createRoot(async (dispose) => {
    const o = createOutboxSlice(ctx);
    await body(o, { jmap, toast, setQueue: (subs) => { queue = subs; } });
    dispose();
  });
}

const sentPatch = (jmap: ReturnType<typeof vi.fn>): Record<string, unknown> => {
  const call = (jmap.mock.calls.find((c) => (c[0] as JmapRequest).methodCalls[0]![0] === 'EmailSubmission/set')![0] as JmapRequest).methodCalls[0]!;
  return call[1] as Record<string, unknown>;
};
const FIRST = '2027-01-01T10:00:00.000Z';
const SECOND = '2027-01-02T10:00:00.000Z';

describe('outbox slice: edit send time', () => {
  it('sends the new time guarded by the time the row had, and updates the row', async () => {
    await run([sub('a', { sendAt: FIRST, mailwomanDueAt: FIRST })], { submissionSet: { updated: { a: { sendAt: SECOND, mailwomanDueAt: SECOND } } } }, async (o, { jmap }) => {
      await o.refreshOutbox();
      const result = await o.rescheduleOutbox('a', SECOND);
      expect(result).toEqual({ ok: true, sendAt: SECOND });
      expect(sentPatch(jmap)['update']).toEqual({ a: { sendAt: SECOND, mailwomanIfSendAt: FIRST } });
      expect(o.outbox()[0]!.sendAt).toBe(SECOND);
      expect(o.outbox()[0]!.mailwomanDueAt).toBe(SECOND);
    });
  });

  it('reports a stateMismatch as a conflict and reloads the queue', async () => {
    await run([sub('a', { sendAt: FIRST })], { submissionSet: { updated: {}, notUpdated: { a: { type: 'stateMismatch', description: 'the send time changed since it was read' } } } }, async (o, { jmap, setQueue }) => {
      await o.refreshOutbox();
      setQueue([sub('a', { sendAt: SECOND })]);
      const result = await o.rescheduleOutbox('a', '2027-01-03T10:00:00.000Z');
      expect(result).toMatchObject({ ok: false, kind: 'conflict' });
      expect(o.outbox()[0]!.sendAt).toBe(SECOND);
      expect(jmap.mock.calls.length).toBeGreaterThan(2);
    });
  });

  it.each([
    ['past', 'invalidProperties', 'sendAt: "2020-01-01T00:00:00Z" is in the past'],
    ['over the maximum', 'invalidProperties', 'sendAt: cannot be more than 90 days ahead'],
    ['already sent', 'invalidArguments', 'submission a is sent and cannot be rescheduled'],
  ])('a %s refusal is returned with the server reason and the row is kept', async (_name, type, description) => {
    await run([sub('a', { sendAt: FIRST })], { submissionSet: { updated: {}, notUpdated: { a: { type, description } } } }, async (o) => {
      await o.refreshOutbox();
      const result = await o.rescheduleOutbox('a', SECOND);
      expect(result).toEqual({ ok: false, kind: 'refused', reason: description });
      expect(o.outbox()[0]!.sendAt).toBe(FIRST);
    });
  });

  it('an id in neither map is not reported as done', async () => {
    await run([sub('a', { sendAt: FIRST })], { submissionSet: { updated: {} } }, async (o) => {
      await o.refreshOutbox();
      expect(await o.rescheduleOutbox('a', SECOND)).toMatchObject({ ok: false });
      expect(o.outbox()[0]!.sendAt).toBe(FIRST);
    });
  });
});

describe('outbox slice: failed sends, send again, delete, body', () => {
  it('announces a row that stops waiting and fails, once, but not failures present at first load', async () => {
    const failed = (id: string): OutboxSubmission => sub(id, { undoStatus: 'canceled', mailwomanFailed: true, mailwomanLastError: '550 no such user' });
    await run([failed('old'), sub('a', { mailwomanHoldSeconds: 5 })], {}, async (o, { toast, setQueue }) => {
      await o.refreshOutbox();
      expect(toast).not.toHaveBeenCalled();
      setQueue([failed('old'), failed('a')]);
      await o.refreshOutbox();
      await o.refreshOutbox();
      expect(toast).toHaveBeenCalledTimes(1);
      expect(toast).toHaveBeenCalledWith('error', expect.stringContaining('550 no such user'));
    });
  });

  it('send again queues the same message and identity as a new submission', async () => {
    await run([sub('a', { undoStatus: 'canceled', mailwomanFailed: true })], { submissionSet: { created: { again: { id: 'sub-2' } } } }, async (o, { jmap, toast }) => {
      await o.refreshOutbox();
      expect(await o.sendOutboxAgain('a')).toBe(true);
      expect(sentPatch(jmap)['create']).toEqual({ again: { emailId: 'e-a', identityId: 'id1' } });
      expect(toast).toHaveBeenCalledWith('info', expect.stringContaining('queued'));
    });
  });

  it('send again says why when the server refuses', async () => {
    await run([sub('a', { undoStatus: 'canceled', mailwomanFailed: true })], { submissionSet: { created: {}, notCreated: { again: { type: 'overQuota', description: 'too many queued' } } } }, async (o, { toast }) => {
      await o.refreshOutbox();
      expect(await o.sendOutboxAgain('a')).toBe(false);
      expect(toast).toHaveBeenCalledWith('error', expect.stringContaining('too many queued'));
    });
  });

  it('delete cancels a waiting submission first, then destroys the message', async () => {
    await run([sub('a', { mailwomanHoldSeconds: 30 })], { submissionSet: { updated: { a: null } }, emailSet: { destroyed: ['e-a'] } }, async (o, { jmap }) => {
      await o.refreshOutbox();
      expect(await o.deleteOutbox('a')).toBe(true);
      const names = jmap.mock.calls.map((c) => (c[0] as JmapRequest).methodCalls[0]![0]);
      expect(names.indexOf('EmailSubmission/set')).toBeLessThan(names.indexOf('Email/set'));
    });
  });

  it('delete leaves the message alone when the cancel is refused (already being sent)', async () => {
    await run([sub('a', { mailwomanHoldSeconds: 30 })], { submissionSet: { updated: {}, notUpdated: { a: { type: 'invalidArguments', description: 'submission a is sent and cannot be canceled' } } } }, async (o, { jmap, toast }) => {
      await o.refreshOutbox();
      expect(await o.deleteOutbox('a')).toBe(false);
      expect(jmap.mock.calls.some((c) => (c[0] as JmapRequest).methodCalls[0]![0] === 'Email/set')).toBe(false);
      expect(toast).toHaveBeenCalledWith('error', expect.any(String));
    });
  });

  it('reads the plain-text body of the queued message', async () => {
    await run([sub('a')], {}, async (o) => {
      expect(await o.outboxBody('e-a')).toMatchObject({ subject: 'Hi', text: 'one\ntwo' });
    });
  });
});
