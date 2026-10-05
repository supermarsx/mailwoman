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
import {
  cancelSubmission,
  identityGet,
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
  type Id,
  type Identity,
  type IdentityGetResponse,
  type JmapResponse,
  type SetError,
} from '../../api/jmap-types.ts';
import { t } from '../../i18n/index.ts';
import type { SliceContext } from './context.ts';

/** What created a submission, when it was not the owner's own client. */
export interface SubmissionOrigin {
  /** `apiKey` (name = the key's prefix) or `oauthClient` (name = the client id). */
  kind: string;
  name: string;
}

/**
 * A submission as the engine returns it from `EmailSubmission/get`
 * (`crates/mw-engine/src/jmap.rs`, `submission_json`): the RFC 8621 object plus
 * the engine's hold and retry fields. They are optional here because a JMAP
 * server that is not the engine (proxy mode) does not send them.
 */
export interface OutboxSubmission extends EmailSubmission {
  /** `"manual"` while the submission waits for the owner to release it. */
  mailwomanHold?: 'manual' | null;
  mailwomanOrigin?: SubmissionOrigin | null;
  /** The engine stopped trying; nothing was delivered. Reported with `canceled`. */
  mailwomanFailed?: boolean;
  /** Attempts in which the mail server accepted nothing. */
  mailwomanAttempts?: number;
  /** The latest failure. On a sent row: what went wrong filing the copy. */
  mailwomanLastError?: string | null;
  mailwomanNextAttemptAt?: string | null;
}

/** The message behind a waiting submission, enough to recognise it by. */
export interface OutboxMessage {
  subject: string | null;
  to: EmailAddress[];
}

/** A submission's user-facing lifecycle state (derived from the raw row). */
export type OutboxState = 'held' | 'scheduled' | 'holding' | 'sent' | 'canceled' | 'failed';

/** Classify a submission for the Outbox UI (§1.3). */
export function outboxStateOf(sub: OutboxSubmission, now = Date.now()): OutboxState {
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

/** The `EmailSubmission/set` update result (RFC 8620 §5.3). */
interface SubmissionUpdateResponse {
  /** The server-set properties of each updated id (`null` when none). */
  updated?: Record<Id, Partial<OutboxSubmission> | null> | null;
  notUpdated?: Record<Id, SetError> | null;
}

/** What one update did: the server-set properties, or why it was refused. */
type UpdateOutcome =
  | { ok: true; changed: Partial<OutboxSubmission> }
  | { ok: false; reason: string };

function updateOutcome(res: JmapResponse, id: Id): UpdateOutcome {
  const set = responseFor<SubmissionUpdateResponse>(res, 'set');
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
  loadIdentities(): Promise<void>;
  /** Cancel a waiting submission before it is sent ("Cancel" / "Discard"). */
  cancelOutbox(id: Id): Promise<void>;
  /** Send a waiting submission now ("Send now" / "Release"). */
  sendOutboxNow(id: Id): Promise<void>;
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

  /** Subject and recipients for the waiting rows: what "Release" would send. */
  async function loadMessages(acct: Id, subs: OutboxSubmission[]): Promise<void> {
    const ids = [...new Set(subs.filter(isWaiting).map((s) => s.emailId))];
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

  async function refreshOutbox(): Promise<void> {
    const acct = await resolveAccount();
    if (acct === null) return;
    setLoading(true);
    try {
      const res = await client.jmap(outboxQuery(acct));
      const subs: OutboxSubmission[] = responseFor<EmailSubmissionGetResponse>(res, 'g').list;
      setOutbox(subs);
      await loadMessages(acct, subs);
    } finally {
      setLoading(false);
    }
  }

  async function loadIdentities(): Promise<void> {
    const acct = await resolveAccount();
    if (acct === null) return;
    try {
      const res = await client.jmap(identityGet(acct));
      setIdentities(responseFor<IdentityGetResponse>(res, 'i').list ?? []);
    } catch {
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
  };
}
