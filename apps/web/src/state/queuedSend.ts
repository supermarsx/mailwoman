// What the user sees after Send while the engine holds the message: a toast
// with a live countdown, Undo (recall) and Send now. The hold is server-side,
// so closing the tab neither sends early nor loses the message.
//
// Recall is a compare-and-set on the server (pending -> canceled). A message
// the engine has already claimed (`sending`) or sent refuses it, and the toast
// says so instead of pretending it was stopped.

import { toast } from '../components/toast/index.ts';
import { newDraftId, saveDraft, type StoredDraft } from '../components/compose/drafts-store.ts';
import { preferenceAccount } from '../api/accountPreferenceLifecycle.ts';
import { cancelSubmission, responseFor, sendSubmissionNow } from '../api/jmap.ts';
import type { EmailSubmissionSetResponse, JmapRequest, JmapResponse } from '../api/jmap-types.ts';
import { loadCatalog, t } from '../i18n';
import { offerRecalledDraft } from './recalledDraft.ts';
import type { SendInput } from './slices/mail.ts';

export const QUEUED_TOAST_ID = 'mail-queued-send';

/** Whether an `EmailSubmission/set` update of `id` was applied. */
export function updateApplied(res: JmapResponse, id: string): boolean {
  const set = responseFor<EmailSubmissionSetResponse>(res, 'set');
  if (set.notUpdated?.[id] !== undefined) return false;
  return set.updated !== undefined && set.updated !== null && id in set.updated;
}

/** The message as a local draft, with everything the composer needs to resume it. */
export function draftFromSend(input: SendInput, now = Date.now()): StoredDraft | null {
  const owner = preferenceAccount();
  if (owner === null || !owner.storageAuthority) return null;
  const draft: StoredDraft = {
    id: newDraftId(),
    accountId: owner.id,
    storageAuthority: owner.storageAuthority,
    to: input.to,
    subject: input.subject,
    bodyHtml: input.htmlBody,
    bodyText: input.textBody ?? '',
    bodyFormat: input.textBody !== undefined ? 'plain' : 'html',
    savedAt: now,
  };
  if (input.cc) draft.cc = input.cc;
  if (input.bcc) draft.bcc = input.bcc;
  if (input.inReplyTo?.length) draft.inReplyTo = input.inReplyTo;
  if (input.references?.length) draft.references = input.references;
  if (input.identity?.id) draft.identityId = input.identity.id;
  if (input.fromOverride) draft.fromOverride = input.fromOverride;
  if (input.importance) draft.importance = input.importance;
  if (input.sentMailboxId) draft.sentMailboxId = input.sentMailboxId;
  if (input.replyTo !== undefined) draft.replyTo = input.replyTo;
  if (input.requestReadReceipt) draft.requestReceipt = true;
  if (input.requestDeliveryReceipt) draft.requestDeliveryReceipt = true;
  if (input.openTracking) draft.openTracking = true;
  if (input.attachments?.length) {
    draft.attachments = input.attachments.map((a) => ({ name: a.name, blobId: a.blobId, size: a.size ?? 0, contentType: a.type }));
  }
  if (input.inline?.length) {
    draft.inline = input.inline.map((i) => ({ cid: i.cid, blobId: i.blobId, type: i.type, name: i.name, size: 0 }));
  }
  return draft;
}

export interface QueuedSend {
  accountId: string;
  submissionId: string;
  holdSeconds: number;
  input: SendInput;
  call: (body: JmapRequest) => Promise<JmapResponse>;
  /** After a recall: undo anything done on acceptance (the answered mark). */
  onRecalled?: () => Promise<void>;
  /** Whether this session is still the one that sent. */
  current: () => boolean;
}

/** Recall: returns true when the message was stopped and reopened as a draft. */
export async function recallQueued(q: QueuedSend): Promise<boolean> {
  const res = await q.call(cancelSubmission(q.accountId, q.submissionId));
  if (!q.current()) return false;
  if (!updateApplied(res, q.submissionId)) {
    toast.error(t('send-queued-too-late'), { id: QUEUED_TOAST_ID });
    return false;
  }
  await q.onRecalled?.().catch(() => undefined);
  const draft = draftFromSend(q.input);
  if (draft !== null) {
    saveDraft(draft);
    offerRecalledDraft(draft.id);
  }
  toast.info(t('send-queued-recalled'));
  return true;
}

/** Skip the rest of the window. */
export async function sendQueuedNow(q: QueuedSend): Promise<boolean> {
  const res = await q.call(sendSubmissionNow(q.accountId, q.submissionId));
  if (!q.current()) return false;
  if (!updateApplied(res, q.submissionId)) {
    toast.error(t('send-queued-too-late'), { id: QUEUED_TOAST_ID });
    return false;
  }
  toast.success(t('send-queued-sent-now'));
  return true;
}

/** Show the queued toast for a send held for `holdSeconds`. */
export async function announceQueuedSend(q: QueuedSend): Promise<void> {
  await loadCatalog('send-delay');
  toast.info(t('send-queued'), {
    id: QUEUED_TOAST_ID,
    duration: q.holdSeconds * 1000,
    countdown: true,
    pausable: false,
    testId: 'send-queued-toast',
    action: { label: t('send-queued-undo'), run: async () => { await recallQueued(q); } },
    secondaryAction: { label: t('send-queued-now'), run: async () => { await sendQueuedNow(q); } },
  });
}

/** Confirmation for a send scheduled for a later time. */
export async function announceScheduled(sendAtIso: string, zone: string, format: (iso: string) => string): Promise<void> {
  await loadCatalog('send-delay');
  toast.success(t('send-queued-scheduled', { when: format(sendAtIso), zone }));
}
