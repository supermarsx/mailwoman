// Outbox slice (plan §3 e7, §1.3, §2.1). Owns the *visible* send queue backed by
// `EmailSubmission/query` — the honest Outbox — plus the sending identities the
// compose picker offers. Undo-send itself (the 10-second Cancel toast) lives in
// the mail slice (it rides `sendMessage`); this slice is the durable view of what
// the engine is holding: send-later rows waiting for their `sendAt`, rows in
// their undo window, rows held until the owner releases them (an MCP `mail.send`
// from a key without unattended send), canceled rows, rows the engine gave up on,
// and finalized (sent) rows.
//
// Cancel and release report what the server answered, not what was asked: the
// `EmailSubmission/set` response is read, the list is reloaded, and a refusal or
// a send that SMTP did not accept is shown as that.
//
// BOUNDARY (vs e5): this is the SERVER-held submission queue (undo-send /
// send-later / held), NOT the offline replay queue (`offlineQueuePending`,
// offline/**).

import { createSignal, type Accessor } from 'solid-js';
import { recallFromOutbox } from '../recallFromOutbox.ts';
import {
  cancelSubmission,
  identityGet,
  rescheduleSubmission,
  resendSubmission,
  outboxQuery,
  request,
  responseFor,
  sendSubmissionNow,
} from '../../api/jmap.ts';
import {
  CAP_CORE,
  CAP_MAIL,
  type EmailAddress,
  type EmailSubmission,
  type EmailSubmissionGetResponse,
  type EmailSubmissionSetResponse,
  type Id,
  type Identity,
  type IdentityGetResponse,
  type JmapResponse,
} from '../../api/jmap-types.ts';
import { loadCatalog, t } from '../../i18n/index.ts';
import type { SliceContext } from './context.ts';

export type { SubmissionOrigin } from '../../api/jmap-types.ts';

/** A row of the Outbox: the shared `EmailSubmission`, which carries the
 *  engine's hold and retry fields (`api/jmap-types.ts`). */
export type OutboxSubmission = EmailSubmission;

/** The message behind a waiting submission, enough to recognise it by. */
export interface OutboxMessage {
  subject: string | null;
  to: EmailAddress[];
}

/** A queued message as the Outbox reader shows it. */
export interface OutboxBody {
  subject: string | null;
  from: EmailAddress[];
  to: EmailAddress[];
  cc: EmailAddress[];
  text: string;
}

/** How an edit of the send time ended. `conflict`: the row moved since it was read. */
export type RescheduleResult = { ok: true; sendAt: string } | { ok: false; kind: 'conflict' | 'refused'; reason: string };

/** A submission's user-facing lifecycle state (derived from the raw row). */
export type OutboxState = 'held' | 'scheduled' | 'holding' | 'sent' | 'canceled' | 'failed' | 'uncertain';

/** Classify a submission for the Outbox UI (§1.3). */
export function outboxStateOf(sub: OutboxSubmission, now = Date.now()): OutboxState {
  if (sub.mailwomanDeliveryUncertain === true) return 'uncertain';
  if (sub.undoStatus === 'canceled') return sub.mailwomanFailed === true ? 'failed' : 'canceled';
  if (sub.undoStatus === 'final') return 'sent';
  // pending: held until released, scheduled for a future time (send-later), or
  // inside the engine-held undo window / awaiting a retry (holding). A hold
  // outranks the times: a held row is not sent whatever its `sendAt` says.
  if (sub.mailwomanHold === 'manual') return 'held';
  if (sub.sendAt !== null && new Date(sub.sendAt).getTime() > now) return 'scheduled';
  return 'holding';
}

/** Whether the owner can still stop this submission. */
function isWaiting(sub: OutboxSubmission): boolean {
  const st = outboxStateOf(sub);
  return st === 'held' || st === 'scheduled' || st === 'holding';
}

/** What one update did: the server-set properties, or why it was refused. */
type UpdateOutcome =
  | { ok: true; changed: Partial<OutboxSubmission> }
  | { ok: false; reason: string };

function updateOutcome(res: JmapResponse, id: Id): UpdateOutcome {
  const set = responseFor<EmailSubmissionSetResponse>(res, 'set');
  const refused = set.notUpdated?.[id];
  if (refused !== undefined) {
    return { ok: false, reason: refused.description ?? refused.type };
  }
  // An id in neither map was not updated either: do not report it as done.
  if (set.updated === undefined || set.updated === null || !(id in set.updated)) {
    return { ok: false, reason: 'no result' };
  }
  return { ok: true, changed: set.updated[id] ?? {} };
}

export interface OutboxSlice {
  /** The submission queue newest-first (`EmailSubmission/query` + get). */
  outbox: Accessor<OutboxSubmission[]>;
  outboxLoading: Accessor<boolean>;
  /** Submissions the user can still stop (held, scheduled or holding). */
  cancelableOutbox: Accessor<OutboxSubmission[]>;
  /** Subject and recipients of each waiting submission's message, by `emailId`. */
  outboxMessages: Accessor<Record<Id, OutboxMessage>>;
  /** Sending identities (configured + server-pulled allowed-froms, §2.1). */
  identities: Accessor<Identity[]>;
  /** (Re)load the Outbox from the server. */
  refreshOutbox(): Promise<void>;
  /** Load the sending identities (once per session; used by compose). */
  loadIdentities(options?:{strict?:boolean}): Promise<void>;
  /** Cancel a waiting submission before it is sent ("Cancel" / "Discard"). */
  cancelOutbox(id: Id): Promise<void>;
  /** Send a waiting submission now ("Send now" / "Release"). */
  sendOutboxNow(id: Id): Promise<void>;
  /** Cancel a waiting submission and reopen its message as a draft in the composer. */
  recallOutbox(id: Id): Promise<void>;
  /** Move a scheduled submission to `sendAt` (ISO), guarded by the time it was read at. */
  rescheduleOutbox(id: Id, sendAt: string): Promise<RescheduleResult>;
  /** Queue the message of a failed submission again. */
  sendOutboxAgain(id: Id): Promise<boolean>;
  /** The message behind a submission, for the Outbox reader. */
  outboxBody(emailId: Id): Promise<OutboxBody | null>;
  /** Stop a waiting submission (if still waiting) and delete its message. */
  deleteOutbox(id: Id): Promise<boolean>;
}

export function createOutboxSlice(ctx: SliceContext): OutboxSlice {
  const { client, showToast } = ctx;

  const [outbox, setOutbox] = createSignal<OutboxSubmission[]>([]);
  const [outboxLoading, setLoading] = createSignal(false);
  const [outboxMessages, setMessages] = createSignal<Record<Id, OutboxMessage>>({});
  const [identities, setIdentities] = createSignal<Identity[]>([]);

  let cachedAccount: Id | null = null;
  async function resolveAccount(): Promise<Id | null> {
    if (cachedAccount !== null) return cachedAccount;
    const session = await client.session();
    cachedAccount = session.primaryAccounts[CAP_MAIL] ?? Object.keys(session.accounts)[0] ?? null;
    return cachedAccount;
  }

  /** Subject and recipients for waiting or uncertain rows: identify the affected message. */
  async function loadMessages(acct: Id, subs: OutboxSubmission[]): Promise<void> {
    const ids = [...new Set(subs.filter((s) => isWaiting(s) || outboxStateOf(s) === 'uncertain').map((s) => s.emailId))];
    if (ids.length === 0) {
      setMessages({});
      return;
    }
    try {
      const res = await client.jmap(
        request(
          [CAP_CORE, CAP_MAIL],
          [['Email/get', { accountId: acct, ids, properties: ['id', 'subject', 'to'] }, 'e']],
        ),
      );
      const list = responseFor<{ list?: { id: Id; subject?: string | null; to?: EmailAddress[] | null }[] }>(
        res,
        'e',
      ).list;
      const byId: Record<Id, OutboxMessage> = {};
      for (const m of list ?? []) byId[m.id] = { subject: m.subject ?? null, to: m.to ?? [] };
      setMessages(byId);
    } catch {
      // The rows still list; they just cannot say what they hold.
      setMessages({});
    }
  }

  // A row that was waiting when last seen and is now a failure is announced
  // once; a first load announces nothing (those failures are old news).
  const seenStates = new Map<Id, OutboxState>();
  let seenOnce = false;
  function announceFailures(subs: OutboxSubmission[]): void {
    const now = Date.now();
    for (const sub of subs) {
      const state = outboxStateOf(sub, now);
      const before = seenStates.get(sub.id);
      if (seenOnce && before !== undefined && before !== state && (state === 'failed' || state === 'uncertain')) {
        const error = sub.mailwomanLastError ?? '';
        showToast('error', state === 'uncertain' ? t('outbox-failed-uncertain-toast') : t('outbox-failed-toast', { error }));
      }
      seenStates.set(sub.id, state);
    }
    seenOnce = true;
  }

  async function refreshOutbox(): Promise<void> {
    setLoading(true);
    try {
      const acct = await resolveAccount();
      if (acct === null) return;
      const res = await client.jmap(outboxQuery(acct));
      const subs: OutboxSubmission[] = responseFor<EmailSubmissionGetResponse>(res, 'g').list;
      announceFailures(subs);
      setOutbox(subs);
      await loadMessages(acct, subs);
    } finally {
      setLoading(false);
    }
  }

  async function loadIdentities(options?:{strict?:boolean}): Promise<void> {
    const acct = await resolveAccount();
    if (acct === null) return;
    try {
      const res = await client.jmap(identityGet(acct));
      const unsupported=res.methodResponses.some(([name,args,id])=>id==='i'&&name==='error'&&args['type']==='unknownMethod');
      if(unsupported){setIdentities([]);return;}
      const result=responseFor<IdentityGetResponse>(res,'i');
      if(options?.strict&&!Array.isArray(result.list))throw new Error('Invalid Identity/get response');
      setIdentities(result.list??[]);
    } catch (cause) {
      if(options?.strict)throw cause;
      // Server lacks Identity/get (e.g. a bare IMAP server or the V0 mock):
      // fall back to no configured identities rather than breaking compose.
      setIdentities([]);
    }
  }

  /** Apply the server-set properties of one update to its row. */
  function patchRow(id: Id, changed: Partial<OutboxSubmission>): void {
    setOutbox((subs) => subs.map((s) => (s.id === id ? { ...s, ...changed } : s)));
  }

  async function cancelOutbox(id: Id): Promise<void> {
    const acct = await resolveAccount();
    if (acct === null) return;
    const wasHeld = outbox().some((s) => s.id === id && outboxStateOf(s) === 'held');
    const outcome = updateOutcome(await client.jmap(cancelSubmission(acct, id)), id);
    if (!outcome.ok) {
      showToast('error', t('mail-outbox-toast-not-canceled', { error: outcome.reason }));
      // The row is not what the list shows (already sent, most likely).
      await refreshOutbox();
      return;
    }
    patchRow(id, { undoStatus: 'canceled', mailwomanHold: null, ...outcome.changed });
    showToast('info', t(wasHeld ? 'mail-outbox-toast-discarded' : 'mail-outbox-toast-canceled'));
  }

  async function sendOutboxNow(id: Id): Promise<void> {
    const acct = await resolveAccount();
    if (acct === null) return;
    const outcome = updateOutcome(await client.jmap(sendSubmissionNow(acct, id)), id);
    if (!outcome.ok) {
      showToast('error', t('mail-outbox-toast-not-released', { error: outcome.reason }));
      await refreshOutbox();
      return;
    }
    const { changed } = outcome;
    patchRow(id, changed);
    const error = changed.mailwomanLastError ?? '';
    if (changed.undoStatus === 'final') {
      showToast('success', t('mail-outbox-toast-sent'));
    } else if (changed.mailwomanFailed === true) {
      showToast('error', t('mail-outbox-toast-failed', { error }));
    } else if (changed.undoStatus === 'pending' && error !== '') {
      // Released, handed to the mail server, not accepted: the engine retries.
      showToast('error', t('mail-outbox-toast-retrying', { error }));
    } else {
      // A server that reports no outcome for the update (not the engine): the
      // row was released; whether it has been sent is what the reload shows.
      await refreshOutbox();
    }
  }

  async function recallOutbox(id: Id): Promise<void> {
    const acct = await resolveAccount();
    if (acct === null) return;
    const sub = outbox().find((s) => s.id === id);
    if (sub === undefined) return;
    await loadCatalog('send-delay');
    try {
      const result = await recallFromOutbox((body) => client.jmap(body), acct, sub);
      if (result === 'too-late') showToast('error', t('outbox-recall-too-late'));
      else if (result === 'recalled') patchRow(id, { undoStatus: 'canceled', mailwomanHold: null });
      else showToast('error', t('mail-outbox-toast-not-canceled', { error: '' }));
    } catch {
      showToast('error', t('mail-outbox-toast-not-canceled', { error: '' }));
    }
    await refreshOutbox();
  }

  async function rescheduleOutbox(id: Id, sendAt: string): Promise<RescheduleResult> {
    const acct = await resolveAccount();
    if (acct === null) return { ok: false, kind: 'refused', reason: '' };
    const sub = outbox().find((s) => s.id === id);
    const res = await client.jmap(rescheduleSubmission(acct, id, sendAt, sub?.sendAt ?? null));
    const set = responseFor<EmailSubmissionSetResponse>(res, 'set');
    const refused = set.notUpdated?.[id];
    if (refused !== undefined) {
      await refreshOutbox();
      return { ok: false, kind: refused.type === 'stateMismatch' ? 'conflict' : 'refused', reason: refused.description ?? refused.type };
    }
    if (set.updated === undefined || set.updated === null || !(id in set.updated)) {
      return { ok: false, kind: 'refused', reason: 'no result' };
    }
    patchRow(id, { sendAt, mailwomanDueAt: sendAt, mailwomanHoldSeconds: 0, ...(set.updated[id] ?? {}) });
    return { ok: true, sendAt };
  }

  async function sendOutboxAgain(id: Id): Promise<boolean> {
    const acct = await resolveAccount();
    if (acct === null) return false;
    const sub = outbox().find((s) => s.id === id);
    if (sub === undefined) return false;
    const res = await client.jmap(resendSubmission(acct, sub.emailId, sub.identityId));
    const set = responseFor<EmailSubmissionSetResponse>(res, 'set');
    const refused = set.notCreated?.['again'];
    if (refused !== undefined || set.created === null || set.created === undefined || set.created['again'] === undefined) {
      showToast('error', t('outbox-send-again-failed', { error: refused?.description ?? refused?.type ?? '' }));
      return false;
    }
    showToast('info', t('outbox-send-again-queued'));
    await refreshOutbox();
    return true;
  }

  async function outboxBody(emailId: Id): Promise<OutboxBody | null> {
    const acct = await resolveAccount();
    if (acct === null) return null;
    try {
      const res = await client.jmap(
        request(
          [CAP_CORE, CAP_MAIL],
          [['Email/get', { accountId: acct, ids: [emailId], properties: ['id', 'subject', 'from', 'to', 'cc', 'textBody', 'bodyValues'], fetchTextBodyValues: true, maxBodyValueBytes: 200_000 }, 'b']],
        ),
      );
      const email = responseFor<{ list?: { subject?: string | null; from?: EmailAddress[] | null; to?: EmailAddress[] | null; cc?: EmailAddress[] | null; textBody?: { partId: string | null }[]; bodyValues?: Record<string, { value: string }> }[] }>(res, 'b').list?.[0];
      if (email === undefined) return null;
      const text = (email.textBody ?? []).map((p) => (p.partId !== null ? email.bodyValues?.[p.partId]?.value ?? '' : '')).join('\n');
      return { subject: email.subject ?? null, from: email.from ?? [], to: email.to ?? [], cc: email.cc ?? [], text };
    } catch {
      return null;
    }
  }

  async function deleteOutbox(id: Id): Promise<boolean> {
    const acct = await resolveAccount();
    if (acct === null) return false;
    const sub = outbox().find((s) => s.id === id);
    if (sub === undefined) return false;
    if (isWaiting(sub)) {
      const outcome = updateOutcome(await client.jmap(cancelSubmission(acct, id)), id);
      if (!outcome.ok) {
        // Already on its way (or sent): the message must not be deleted under it.
        showToast('error', t('mail-outbox-toast-not-canceled', { error: outcome.reason }));
        await refreshOutbox();
        return false;
      }
    }
    try {
      const res = await client.jmap(request([CAP_CORE, CAP_MAIL], [['Email/set', { accountId: acct, destroy: [sub.emailId] }, 'd']]));
      const set = responseFor<{ destroyed?: Id[] | null; notDestroyed?: Record<Id, { description?: string; type: string }> | null }>(res, 'd');
      const failed = set.notDestroyed?.[sub.emailId];
      if (failed !== undefined) {
        showToast('error', t('outbox-delete-failed', { error: failed.description ?? failed.type }));
        await refreshOutbox();
        return false;
      }
    } catch {
      showToast('error', t('outbox-delete-failed', { error: '' }));
      await refreshOutbox();
      return false;
    }
    showToast('info', t('outbox-deleted'));
    await refreshOutbox();
    return true;
  }

  const cancelableOutbox: Accessor<OutboxSubmission[]> = () => outbox().filter(isWaiting);

  return {
    outbox,
    outboxLoading,
    cancelableOutbox,
    outboxMessages,
    identities,
    refreshOutbox,
    loadIdentities,
    cancelOutbox,
    sendOutboxNow,
    recallOutbox,
    rescheduleOutbox,
    sendOutboxAgain,
    outboxBody,
    deleteOutbox,
  };
}
