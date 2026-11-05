import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { JmapRequest, JmapResponse } from '../api/jmap-types.ts';

vi.mock('../api/accountPreferenceLifecycle.ts', () => ({
  preferenceAccount: () => ({ id: 'acct1', generation: 1, storageAuthority: 'authority-0123456789' }),
}));

import { listDrafts } from '../components/compose/drafts-store.ts';
import { draftFromEmail, recallFromOutbox } from './recallFromOutbox.ts';
import { recalledDraftId, takeRecalledDraft } from './recalledDraft.ts';

const email = {
  id: 'e1',
  to: [{ name: 'Ann', email: 'ann@example.org' }],
  cc: [{ name: null, email: 'cc@example.org' }],
  bcc: [{ name: null, email: 'bcc@example.org' }],
  subject: 'Plan',
  inReplyTo: ['orig@example.org'],
  htmlBody: [{ partId: '1', blobId: 'b', size: 5, type: 'text/html' }],
  textBody: [{ partId: '2', blobId: 'c', size: 5, type: 'text/plain' }],
  attachments: [
    { partId: '3', blobId: 'f1', size: 99, type: 'application/pdf', name: 'plan.pdf', disposition: 'attachment' },
    { partId: '4', blobId: 'f2', size: 9, type: 'image/png', name: 'logo.png', disposition: 'inline', cid: 'logo@x' },
  ],
  bodyValues: { '1': { value: '<p>hi</p>' }, '2': { value: 'hi' } },
};
const ok = (extra: object): JmapResponse => ({ methodResponses: [['EmailSubmission/set', { accountId: 'acct1', updated: { sub1: null }, ...extra }, 'set']], sessionState: 's' });

beforeEach(() => { localStorage.clear(); takeRecalledDraft(); });

describe('draftFromEmail', () => {
  it('keeps recipients, bodies, attachments, inline images and the identity', () => {
    const d = draftFromEmail(email, 'ident-2', 'acct1', 'authority-0123456789', 5);
    expect(d).toMatchObject({
      to: 'Ann <ann@example.org>', cc: 'cc@example.org', bcc: 'bcc@example.org', subject: 'Plan',
      bodyHtml: '<p>hi</p>', bodyText: 'hi', bodyFormat: 'html', identityId: 'ident-2', inReplyTo: ['orig@example.org'],
    });
    expect(d.attachments).toEqual([{ name: 'plan.pdf', blobId: 'f1', size: 99, contentType: 'application/pdf' }]);
    expect(d.inline).toEqual([{ cid: 'logo@x', blobId: 'f2', type: 'image/png', name: 'logo.png', size: 9 }]);
  });
});

describe('recallFromOutbox', () => {
  it('cancels first, then reopens the message as a draft', async () => {
    const calls: JmapRequest[] = [];
    const call = async (body: JmapRequest): Promise<JmapResponse> => {
      calls.push(body);
      return body.methodCalls[0]![0] === 'Email/get'
        ? { methodResponses: [['Email/get', { accountId: 'acct1', list: [email] }, 'g']], sessionState: 's' }
        : ok({});
    };
    expect(await recallFromOutbox(call, 'acct1', { id: 'sub1', emailId: 'e1', identityId: 'ident-2' })).toBe('recalled');
    expect(calls.map((c) => c.methodCalls[0]![0])).toEqual(['EmailSubmission/set', 'Email/get']);
    expect(listDrafts('acct1', 'authority-0123456789').some((d) => d.id === recalledDraftId())).toBe(true);
  });

  it('does not touch the message when the send was already claimed', async () => {
    const calls: JmapRequest[] = [];
    const call = async (body: JmapRequest): Promise<JmapResponse> => {
      calls.push(body);
      return { methodResponses: [['EmailSubmission/set', { accountId: 'acct1', notUpdated: { sub1: { type: 'forbidden' } } }, 'set']], sessionState: 's' };
    };
    expect(await recallFromOutbox(call, 'acct1', { id: 'sub1', emailId: 'e1' })).toBe('too-late');
    expect(calls).toHaveLength(1);
    expect(recalledDraftId()).toBeNull();
  });
});
