// Outbox slice (plan §3 e7, §1.3, §2.1). Owns the *visible* send queue backed by
// `EmailSubmission/query` — the honest Outbox — plus the sending identities the
// compose picker offers. Undo-send itself (the 10-second Cancel toast) lives in
// the mail slice (it rides `sendMessage`); this slice is the durable view of what
// the engine is holding: send-later rows waiting for their `sendAt`, held rows in
// their undo window, canceled rows, and finalized (sent) rows.
//
// BOUNDARY (vs e5): this is the SERVER-held submission queue (undo-send /
// send-later), NOT the offline replay queue (`offlineQueuePending`, offline/**).

import { createSignal, type Accessor } from 'solid-js';
import {
  cancelSubmission,
  identityGet,
  outboxQuery,
  responseFor,
  sendSubmissionNow,
} from '../../api/jmap.ts';
import {
  CAP_MAIL,
  type EmailSubmission,
  type EmailSubmissionGetResponse,
  type Id,
  type Identity,
  type IdentityGetResponse,
} from '../../api/jmap-types.ts';
import type { SliceContext } from './context.ts';

/** A submission's user-facing lifecycle state (derived from the raw row). */
export type OutboxState = 'scheduled' | 'holding' | 'sent' | 'canceled';

/** Classify a submission for the Outbox UI (§1.3). */
export function outboxStateOf(sub: EmailSubmission, now = Date.now()): OutboxState {
  if (sub.undoStatus === 'canceled') return 'canceled';
  if (sub.undoStatus === 'final') return 'sent';
  // pending: either scheduled for a future time (send-later) or inside the
  // engine-held undo window (holding), waiting to be dispatched.
  if (sub.sendAt !== null && new Date(sub.sendAt).getTime() > now) return 'scheduled';
  return 'holding';
}

export interface OutboxSlice {
  /** The submission queue newest-first (`EmailSubmission/query` + get). */
  outbox: Accessor<EmailSubmission[]>;
  outboxLoading: Accessor<boolean>;
  /** Submissions the user can still stop (scheduled or holding). */
  cancelableOutbox: Accessor<EmailSubmission[]>;
  /** Sending identities (configured + server-pulled allowed-froms, §2.1). */
  identities: Accessor<Identity[]>;
  /** (Re)load the Outbox from the server. */
  refreshOutbox(): Promise<void>;
  /** Load the sending identities (once per session; used by compose). */
  loadIdentities(): Promise<void>;
  /** Cancel a pending/scheduled submission before it dispatches. */
  cancelOutbox(id: Id): Promise<void>;
  /** Send a held/scheduled submission immediately (clear its delay). */
  sendOutboxNow(id: Id): Promise<void>;
}

export function createOutboxSlice(ctx: SliceContext): OutboxSlice {
  const { client, showToast } = ctx;

  const [outbox, setOutbox] = createSignal<EmailSubmission[]>([]);
  const [outboxLoading, setLoading] = createSignal(false);
  const [identities, setIdentities] = createSignal<Identity[]>([]);

  let cachedAccount: Id | null = null;
  async function resolveAccount(): Promise<Id | null> {
    if (cachedAccount !== null) return cachedAccount;
    const session = await client.session();
    cachedAccount = session.primaryAccounts[CAP_MAIL] ?? Object.keys(session.accounts)[0] ?? null;
    return cachedAccount;
  }

  async function refreshOutbox(): Promise<void> {
    const acct = await resolveAccount();
    if (acct === null) return;
    setLoading(true);
    try {
      const res = await client.jmap(outboxQuery(acct));
      setOutbox(responseFor<EmailSubmissionGetResponse>(res, 'g').list);
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

  async function cancelOutbox(id: Id): Promise<void> {
    const acct = await resolveAccount();
    if (acct === null) return;
    await client.jmap(cancelSubmission(acct, id));
    setOutbox((subs) => subs.map((s) => (s.id === id ? { ...s, undoStatus: 'canceled' } : s)));
    showToast('info', 'Send canceled');
  }

  async function sendOutboxNow(id: Id): Promise<void> {
    const acct = await resolveAccount();
    if (acct === null) return;
    await client.jmap(sendSubmissionNow(acct, id));
    setOutbox((subs) =>
      subs.map((s) => (s.id === id ? { ...s, sendAt: null, mailwomanHoldSeconds: 0 } : s)),
    );
    showToast('success', 'Sending now');
  }

  const cancelableOutbox: Accessor<EmailSubmission[]> = () =>
    outbox().filter((s) => {
      const st = outboxStateOf(s);
      return st === 'scheduled' || st === 'holding';
    });

  return {
    outbox,
    outboxLoading,
    cancelableOutbox,
    identities,
    refreshOutbox,
    loadIdentities,
    cancelOutbox,
    sendOutboxNow,
  };
}
