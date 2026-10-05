import { describe, it, expect, beforeEach } from 'vitest';
import {
  deleteDraft,
  draftHasContent,
  listDrafts,
  newDraftId,
  saveDraft,
  type StoredDraft,
} from './drafts-store.ts';

function draft(id: string, over: Partial<StoredDraft> = {}): StoredDraft {
  return { id, to: 'a@b.c', subject: 's', bodyHtml: '<p>h</p>', bodyText: 'h', savedAt: 1, ...over };
}

describe('drafts-store (W9)', () => {
  beforeEach(() => localStorage.clear());

  it('saves and lists a draft', () => {
    saveDraft(draft('d1'));
    expect(listDrafts().map((d) => d.id)).toEqual(['d1']);
  });

  it('lists newest first and updates in place by id', () => {
    saveDraft(draft('d1', { savedAt: 10 }));
    saveDraft(draft('d2', { savedAt: 20 }));
    saveDraft(draft('d1', { savedAt: 30, subject: 'updated' }));
    const list = listDrafts();
    expect(list.map((d) => d.id)).toEqual(['d1', 'd2']);
    expect(list[0]?.subject).toBe('updated');
  });

  it('deletes a draft by id', () => {
    saveDraft(draft('d1'));
    saveDraft(draft('d2'));
    deleteDraft('d1');
    expect(listDrafts().map((d) => d.id)).toEqual(['d2']);
  });

  it('skips empty compositions', () => {
    saveDraft(draft('empty', { to: '', subject: '', bodyText: '' }));
    expect(listDrafts()).toHaveLength(0);
  });

  it('draftHasContent treats any non-blank field as content', () => {
    expect(draftHasContent({ to: '', subject: '', bodyText: '' })).toBe(false);
    expect(draftHasContent({ to: 'x', subject: '', bodyText: '' })).toBe(true);
    expect(draftHasContent({ to: '', subject: '', bodyText: 'hi' })).toBe(true);
  });

  it('survives corrupt storage without throwing', () => {
    localStorage.setItem('mw.compose.drafts.v1', 'not json');
    expect(listDrafts()).toEqual([]);
    saveDraft(draft('d1'));
    expect(listDrafts().map((d) => d.id)).toEqual(['d1']);
  });

  it('keeps a draft stored before Cc, Bcc and replies existed, and stores the new fields', () => {
    const old = { id: 'old', to: 'a@x.org', subject: 's', bodyHtml: '<p>b</p>', bodyText: 'b', savedAt: 1 };
    localStorage.setItem('mw.compose.drafts.v1', JSON.stringify([old]));
    expect(listDrafts()).toEqual([old]);

    const reply = { ...old, id: 'new', savedAt: 2, cc: 'c@x.org', bcc: 'd@x.org', inReplyTo: ['o@x'], references: ['r@x', 'o@x'] };
    saveDraft(reply);
    expect(listDrafts()).toEqual([reply, old]);
  });

  it('a draft whose only content is a Cc or Bcc recipient is saved', () => {
    saveDraft(draft('cc-only', { to: '', subject: '', bodyText: '', cc: 'c@x.org' }));
    saveDraft(draft('bcc-only', { to: '', subject: '', bodyText: '', bcc: 'd@x.org' }));
    expect(listDrafts().map((d) => d.id).sort()).toEqual(['bcc-only', 'cc-only']);
  });

  it('drops a stored entry whose Cc or thread ids are not the expected type', () => {
    const ok = { id: 'ok', to: 'a@x.org', subject: 's', bodyHtml: '', bodyText: 'b', savedAt: 1 };
    localStorage.setItem(
      'mw.compose.drafts.v1',
      JSON.stringify([ok, { ...ok, id: 'bad-cc', cc: 5 }, { ...ok, id: 'bad-refs', references: ['a', 1] }]),
    );
    expect(listDrafts().map((d) => d.id)).toEqual(['ok']);
  });

  it('mints distinct draft ids', () => {
    expect(newDraftId()).not.toBe(newDraftId());
  });
});
