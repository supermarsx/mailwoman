import { describe, it, expect, beforeEach } from 'vitest';
import {
  deleteDraft,
  draftHasContent,
  listDrafts,
  newDraftId,
  saveDraft,
  type StoredDraft,
  type StoredInlineImage,
} from './drafts-store.ts';

function draft(id: string, over: Partial<StoredDraft> = {}): StoredDraft {
  return { id, accountId:'account-a', to: 'a@b.c', subject: 's', bodyHtml: '<p>h</p>', bodyText: 'h', savedAt: 1, ...over };
}

describe('drafts-store (W9)', () => {
  beforeEach(() => localStorage.clear());

  it('saves and lists a draft', () => {
    saveDraft(draft('d1'));
    expect(listDrafts('account-a').map((d) => d.id)).toEqual(['d1']);
  });

  it('keeps format, identity, signature and attachment state within its account', () => {
    const state = draft('complete', {identityId:'primary', bodyFormat:'plain', insertedSignatures:['primary'],
      attachments:[{name:'report.pdf',blobId:'owned-upload',size:42,contentType:'application/pdf'}]});
    saveDraft(state);
    expect(listDrafts('account-a')).toEqual([state]);
    expect(listDrafts('account-b')).toEqual([]);
    localStorage.setItem('mw.compose.drafts.v1',JSON.stringify([draft('bad',{attachments:[{name:'x',blobId:'x',size:-1,contentType:null}]})]));
    expect(listDrafts('account-a')).toEqual([]);
  });

  it('retains an attachment-only composition before recipients are entered', () => {
    const attachmentOnly=draft('attachment-only',{to:'',subject:'',bodyText:'',bodyHtml:'',attachments:[{name:'x.pdf',blobId:'owned',size:1,contentType:'application/pdf'}]});
    saveDraft(attachmentOnly);
    expect(listDrafts('account-a')).toEqual([attachmentOnly]);
  });

  it('lists newest first and updates in place by id', () => {
    saveDraft(draft('d1', { savedAt: 10 }));
    saveDraft(draft('d2', { savedAt: 20 }));
    saveDraft(draft('d1', { savedAt: 30, subject: 'updated' }));
    const list = listDrafts('account-a');
    expect(list.map((d) => d.id)).toEqual(['d1', 'd2']);
    expect(list[0]?.subject).toBe('updated');
  });

  // t29-e12: a draft keeps its send options and inline images for itself.
  it('keeps the send options and inline images of a draft', () => {
    const inline = [{ cid: 'c1@mailwoman', blobId: 'b1', type: 'image/png', name: 'a.png', size: 3 }];
    saveDraft(draft('d1', { requestReceipt: true, requestDeliveryReceipt: true, openTracking: true, inline }));
    expect(listDrafts('account-a')).toEqual([draft('d1', { requestReceipt: true, requestDeliveryReceipt: true, openTracking: true, inline })]);
  });

  it('ignores a stored draft whose options or images are malformed', () => {
    localStorage.setItem(
      'mw.compose.drafts.v1',
      JSON.stringify([
        draft('bad-flag', { requestReceipt: 'yes' as unknown as boolean }),
        draft('bad-dsn', { requestDeliveryReceipt: 'yes' as unknown as boolean }),
        draft('bad-inline', { inline: [{ cid: 'c', blobId: 'b' }] as unknown as StoredInlineImage[] }),
        draft('ok'),
      ]),
    );
    expect(listDrafts('account-a').map((d) => d.id)).toEqual(['ok']);
  });

  it('deletes a draft by id', () => {
    saveDraft(draft('d1'));
    saveDraft(draft('d2'));
    deleteDraft('d1', 'account-a');
    expect(listDrafts('account-a').map((d) => d.id)).toEqual(['d2']);
  });

  it('skips empty compositions', () => {
    saveDraft(draft('empty', { to: '', subject: '', bodyText: '' }));
    expect(listDrafts('account-a')).toHaveLength(0);
  });

  it('draftHasContent treats any non-blank field as content', () => {
    expect(draftHasContent({ to: '', subject: '', bodyText: '' })).toBe(false);
    expect(draftHasContent({ to: 'x', subject: '', bodyText: '' })).toBe(true);
    expect(draftHasContent({ to: '', subject: '', bodyText: 'hi' })).toBe(true);
  });

  it('survives corrupt storage without throwing', () => {
    localStorage.setItem('mw.compose.drafts.v1', 'not json');
    expect(listDrafts('account-a')).toEqual([]);
    saveDraft(draft('d1'));
    expect(listDrafts('account-a').map((d) => d.id)).toEqual(['d1']);
  });

  it('keeps a draft stored before Cc, Bcc and replies existed, and stores the new fields', () => {
    const old = { id: 'old', accountId:'account-a', to: 'a@x.org', subject: 's', bodyHtml: '<p>b</p>', bodyText: 'b', savedAt: 1 };
    localStorage.setItem('mw.compose.drafts.v1', JSON.stringify([old]));
    expect(listDrafts('account-a')).toEqual([old]);

    const reply = { ...old, id: 'new', savedAt: 2, cc: 'c@x.org', bcc: 'd@x.org', inReplyTo: ['o@x'], references: ['r@x', 'o@x'] };
    saveDraft(reply);
    expect(listDrafts('account-a')).toEqual([reply, old]);
  });

  it('a draft whose only content is a Cc or Bcc recipient is saved', () => {
    saveDraft(draft('cc-only', { to: '', subject: '', bodyText: '', cc: 'c@x.org' }));
    saveDraft(draft('bcc-only', { to: '', subject: '', bodyText: '', bcc: 'd@x.org' }));
    expect(listDrafts('account-a').map((d) => d.id).sort()).toEqual(['bcc-only', 'cc-only']);
  });

  it('drops a stored entry whose Cc or thread ids are not the expected type', () => {
    const ok = { id: 'ok', accountId:'account-a', to: 'a@x.org', subject: 's', bodyHtml: '', bodyText: 'b', savedAt: 1 };
    localStorage.setItem(
      'mw.compose.drafts.v1',
      JSON.stringify([ok, { ...ok, id: 'bad-cc', cc: 5 }, { ...ok, id: 'bad-refs', references: ['a', 1] }]),
    );
    expect(listDrafts('account-a').map((d) => d.id)).toEqual(['ok']);
  });

  it('mints distinct draft ids', () => {
    expect(newDraftId()).not.toBe(newDraftId());
  });
});

it('isolates saved compositions by authenticated account even when draft IDs collide', () => {
  const forAccount = listDrafts;
  saveDraft({...draft('same', {subject:'Account A'}), accountId:'account-a'} as StoredDraft);
  saveDraft({...draft('same', {subject:'Account B'}), accountId:'account-b'} as StoredDraft);
  expect(forAccount('account-a').map(row=>row.subject)).toEqual(['Account A']);
  expect(forAccount('account-b').map(row=>row.subject)).toEqual(['Account B']);
});

it('does not attribute a legacy unowned draft to a new account', () => {
  const unowned: Record<string,unknown> = {...draft('old')}; delete unowned['accountId'];
  localStorage.setItem('mw.compose.drafts.v1',JSON.stringify([unowned]));
  expect(listDrafts('account-a')).toEqual([]);
  expect(listDrafts(null)).toEqual([]);
});
