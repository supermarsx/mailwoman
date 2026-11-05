import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { JmapRequest, JmapResponse } from '../api/jmap-types.ts';

vi.mock('../api/accountPreferenceLifecycle.ts', () => ({
  preferenceAccount: () => ({ id: 'acct1', generation: 1, storageAuthority: 'authority-0123456789' }),
}));
const toast = vi.hoisted(() => ({ info: vi.fn(), error: vi.fn(), success: vi.fn() }));
vi.mock('../components/toast/index.ts', () => ({ toast }));

import { listDrafts } from '../components/compose/drafts-store.ts';
import { draftFromSend, recallQueued, sendQueuedNow, updateApplied, type QueuedSend } from './queuedSend.ts';
import { recalledDraftId, takeRecalledDraft } from './recalledDraft.ts';

const input = {
  to: 'a@example.org', cc: 'c@example.org', bcc: 'b@example.org', subject: 'Hello', htmlBody: '<p>x</p>',
  identity: { id: 'id-2', name: 'Me', email: 'me@example.org', replyTo: null, signatureHtml: null, signatureText: null, sentMailboxId: null },
  attachments: [{ blobId: 'blob1', name: 'a.pdf', type: 'application/pdf', size: 10 }],
};
const applied: JmapResponse = { methodResponses: [['EmailSubmission/set', { accountId: 'acct1', updated: { sub1: null } }, 'set']], sessionState: 's' };
const refused: JmapResponse = { methodResponses: [['EmailSubmission/set', { accountId: 'acct1', notUpdated: { sub1: { type: 'forbidden', description: 'sending' } } }, 'set']], sessionState: 's' };

function queued(res: JmapResponse, extra: Partial<QueuedSend> = {}): QueuedSend & { calls: JmapRequest[] } {
  const calls: JmapRequest[] = [];
  return {
    accountId: 'acct1', submissionId: 'sub1', holdSeconds: 10, input,
    call: async (body) => { calls.push(body); return res; },
    current: () => true, calls, ...extra,
  };
}

beforeEach(() => {
  localStorage.clear();
  toast.info.mockClear(); toast.error.mockClear(); toast.success.mockClear();
  takeRecalledDraft();
});

describe('updateApplied', () => {
  it('is true only when the update is in `updated`', () => {
    expect(updateApplied(applied, 'sub1')).toBe(true);
    expect(updateApplied(refused, 'sub1')).toBe(false);
    expect(updateApplied({ methodResponses: [['EmailSubmission/set', {}, 'set']], sessionState: 's' }, 'sub1')).toBe(false);
  });
});

describe('recallQueued', () => {
  it('cancels, saves a draft with recipients, identity and attachments, and offers it', async () => {
    const onRecalled = vi.fn(async () => undefined);
    const q = queued(applied, { onRecalled });
    expect(await recallQueued(q)).toBe(true);
    expect(q.calls[0]!.methodCalls[0]![1]).toMatchObject({ update: { sub1: { undoStatus: 'canceled' } } });
    expect(onRecalled).toHaveBeenCalled();
    const id = recalledDraftId();
    const draft = listDrafts('acct1', 'authority-0123456789').find((d) => d.id === id)!;
    expect(draft).toMatchObject({ to: 'a@example.org', cc: 'c@example.org', bcc: 'b@example.org', subject: 'Hello', identityId: 'id-2' });
    expect(draft.attachments).toEqual([{ name: 'a.pdf', blobId: 'blob1', size: 10, contentType: 'application/pdf' }]);
  });
  it('says too late and opens nothing when the engine already claimed the send', async () => {
    const onRecalled = vi.fn();
    expect(await recallQueued(queued(refused, { onRecalled }))).toBe(false);
    expect(toast.error).toHaveBeenCalled();
    expect(onRecalled).not.toHaveBeenCalled();
    expect(recalledDraftId()).toBeNull();
    expect(listDrafts('acct1', 'authority-0123456789')).toHaveLength(0);
  });
  it('is idempotent: a second recall of the same message is refused, not duplicated', async () => {
    const responses = [applied, refused];
    const q = queued(applied, { call: async () => responses.shift()! });
    expect(await recallQueued(q)).toBe(true);
    expect(await recallQueued(q)).toBe(false);
    expect(listDrafts('acct1', 'authority-0123456789')).toHaveLength(1);
  });
});

describe('sendQueuedNow', () => {
  it('clears the hold; a refusal is reported', async () => {
    const q = queued(applied);
    expect(await sendQueuedNow(q)).toBe(true);
    expect(q.calls[0]!.methodCalls[0]![1]).toMatchObject({ update: { sub1: { sendAt: null, mailwomanHoldSeconds: 0 } } });
    expect(await sendQueuedNow(queued(refused))).toBe(false);
  });
});

describe('draftFromSend', () => {
  it('keeps plain-text bodies as plain', () => {
    expect(draftFromSend({ ...input, textBody: 'hi' })).toMatchObject({ bodyFormat: 'plain', bodyText: 'hi' });
  });
});
