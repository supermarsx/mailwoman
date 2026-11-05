// "Recall and edit" for a message waiting in the Outbox: cancel the submission
// (a server-side compare-and-set), then rebuild the message as a local draft so
// the composer reopens with its recipients, body, attachments and identity.
//
// The cancel comes first. If the engine has already claimed the message the
// cancel is refused and nothing is opened, so a message is never both sent and
// reopened for editing.

import { CAP_CORE, CAP_MAIL, type EmailAddress, type EmailBodyPart, type EmailBodyValue, type Id, type JmapRequest, type JmapResponse } from '../api/jmap-types.ts';
import { cancelSubmission, request, responseFor } from '../api/jmap.ts';
import { preferenceAccount } from '../api/accountPreferenceLifecycle.ts';
import { newDraftId, saveDraft, type StoredDraft } from '../components/compose/drafts-store.ts';
import { formatAddressList } from '../components/compose/reply.ts';
import { updateApplied } from './queuedSend.ts';
import { offerRecalledDraft } from './recalledDraft.ts';

interface FetchedPart extends EmailBodyPart {
  name?: string | null;
  cid?: string | null;
  disposition?: string | null;
}

interface FetchedEmail {
  id: Id;
  to?: EmailAddress[] | null;
  cc?: EmailAddress[] | null;
  bcc?: EmailAddress[] | null;
  subject?: string | null;
  messageId?: string[] | null;
  inReplyTo?: string[] | null;
  references?: string[] | null;
  htmlBody?: FetchedPart[];
  textBody?: FetchedPart[];
  attachments?: FetchedPart[];
  bodyValues?: Record<string, EmailBodyValue>;
}

export type RecallResult = 'recalled' | 'too-late' | 'unavailable';

function emailGetForRecall(accountId: Id, id: Id): JmapRequest {
  return request([CAP_CORE, CAP_MAIL], [[
    'Email/get',
    {
      accountId,
      ids: [id],
      properties: ['id', 'to', 'cc', 'bcc', 'subject', 'inReplyTo', 'references', 'htmlBody', 'textBody', 'attachments', 'bodyValues'],
      fetchHTMLBodyValues: true,
      fetchTextBodyValues: true,
      maxBodyValueBytes: 2_000_000,
    },
    'g',
  ]]);
}

function bodyOf(parts: FetchedPart[] | undefined, values: Record<string, EmailBodyValue> | undefined): string {
  return (parts ?? []).map((p) => (p.partId !== null ? values?.[p.partId]?.value ?? '' : '')).join('');
}

/** The stored draft for a fetched message. Pure, so it can be tested alone. */
export function draftFromEmail(email: FetchedEmail, identityId: string | null, accountId: string, storageAuthority: string, now = Date.now()): StoredDraft {
  const html = bodyOf(email.htmlBody, email.bodyValues);
  const text = bodyOf(email.textBody, email.bodyValues);
  const draft: StoredDraft = {
    id: newDraftId(),
    accountId,
    storageAuthority,
    to: formatAddressList(email.to ?? []),
    subject: email.subject ?? '',
    bodyHtml: html !== '' ? html : '',
    bodyText: text,
    bodyFormat: html !== '' ? 'html' : 'plain',
    savedAt: now,
  };
  const cc = formatAddressList(email.cc ?? []);
  const bcc = formatAddressList(email.bcc ?? []);
  if (cc !== '') draft.cc = cc;
  if (bcc !== '') draft.bcc = bcc;
  if (email.inReplyTo?.length) draft.inReplyTo = email.inReplyTo;
  if (email.references?.length) draft.references = email.references;
  if (identityId !== null) draft.identityId = identityId;
  const files = (email.attachments ?? []).filter((p) => p.blobId !== null);
  const inline = files.filter((p) => p.disposition === 'inline' && p.cid);
  const plain = files.filter((p) => !(p.disposition === 'inline' && p.cid));
  if (plain.length) {
    draft.attachments = plain.map((p) => ({ name: p.name ?? 'attachment', blobId: p.blobId as string, size: p.size, contentType: p.type }));
  }
  if (inline.length) {
    draft.inline = inline.map((p) => ({ cid: p.cid as string, blobId: p.blobId as string, type: p.type, name: p.name ?? 'image', size: p.size }));
  }
  return draft;
}

/** Cancel the submission and reopen its message as a draft. */
export async function recallFromOutbox(
  call: (body: JmapRequest) => Promise<JmapResponse>,
  accountId: Id,
  submission: { id: Id; emailId: Id; identityId?: Id | null },
): Promise<RecallResult> {
  const cancel = await call(cancelSubmission(accountId, submission.id));
  if (!updateApplied(cancel, submission.id)) return 'too-late';
  const owner = preferenceAccount();
  if (owner === null || !owner.storageAuthority) return 'unavailable';
  const res = await call(emailGetForRecall(accountId, submission.emailId));
  const email = responseFor<{ list?: FetchedEmail[] }>(res, 'g').list?.[0];
  if (email === undefined) return 'unavailable';
  const draft = draftFromEmail(email, submission.identityId ?? null, owner.id, owner.storageAuthority);
  saveDraft(draft);
  offerRecalledDraft(draft.id);
  return 'recalled';
}
