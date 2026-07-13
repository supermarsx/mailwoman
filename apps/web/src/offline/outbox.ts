// Offline outbound queue (contract store `mw-outbox`, plan §2.5): capture
// mutations made while the network is down, then drain them FIFO on reconnect →
// JMAP → reconcile against the set/submission response.
//
// ── BOUNDARY vs e7's Outbox (documented for e7) ────────────────────────────
// THIS queue = the OFFLINE REPLAY queue. It holds mutations (send / flag / move
// / draft) captured while the browser was offline and re-applies them verbatim
// once the connection returns. It is engine/client-side and empties on replay.
//
// e7's Outbox = the SUBMISSION Outbox (`EmailSubmission/query`) — the visible,
// server-held list of messages awaiting send-later / inside the undo-send hold
// window. That is server state, not a local replay log.
//
// They meet at exactly one point: a message COMPOSED while offline is queued
// here as a `send` item; when replayed it becomes a normal `EmailSubmission` and
// (if send-later) then appears in e7's Outbox. Keep the two counts separate —
// `offlineQueuePending` (this) vs the submission Outbox size (e7).

import { NetworkError, type Client } from '../api/client.ts';
import { parseRecipients, request, responseFor, type DraftInput } from '../api/jmap.ts';
import { sendEnvelope } from '../api/jmap.ts';
import {
  CAP_CORE,
  CAP_MAIL,
  type EmailSetResponse,
  type EmailSubmissionSetResponse,
  type Id,
  type JmapRequest,
  type JmapResponse,
} from '../api/jmap-types.ts';
import type { OutboundItem, OutboundType } from '../contracts/offline.ts';

// ── Per-type payloads. `payload` is `unknown` in the frozen contract; these are
//    the shapes this module reads back when building the replay request. ──
export interface FlagPayload {
  accountId: Id;
  emailId: Id;
  keyword: string;
  value: boolean;
}
export interface MovePayload {
  accountId: Id;
  emailId: Id;
  mailboxIds: Record<Id, boolean>;
}
export interface SendPayload {
  accountId: Id;
  draft: DraftInput;
}
export interface DraftPayload {
  accountId: Id;
  draft: DraftInput;
}
/** A PIM mutation captured offline (plan §1.8): the built JMAP request is stored
 *  verbatim and replayed on reconnect; `callId` names the mutation response whose
 *  `notCreated`/`notUpdated`/`notDestroyed` decide whether it applied. */
export interface PimPayload {
  request: JmapRequest;
  callId: string;
}

/** The set-shaped response a replayed PIM mutation is reconciled against. */
interface PimSetResponse {
  created?: Record<string, unknown> | null;
  updated?: Record<string, unknown> | null;
  destroyed?: string[] | null;
  notCreated?: Record<string, unknown> | null;
  notUpdated?: Record<string, unknown> | null;
  notDestroyed?: Record<string, unknown> | null;
}

/** Persistence for the queue. Injected so unit tests avoid a real IndexedDB. */
export interface OutboxStore {
  add(item: OutboundItem): Promise<void>;
  put(item: OutboundItem): Promise<void>;
  /** All items, oldest first (FIFO). */
  all(): Promise<OutboundItem[]>;
  delete(id: string): Promise<void>;
}

const MAIL_USING = [CAP_CORE, CAP_MAIL];

function newId(): string {
  if (typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function') {
    return crypto.randomUUID();
  }
  return `ob-${Date.now().toString(36)}-${Math.random().toString(16).slice(2)}`;
}

/** Append a mutation to the queue (state `queued`). */
export async function enqueueOutbound(
  store: OutboxStore,
  input: { type: OutboundType; payload: unknown },
): Promise<OutboundItem> {
  const item: OutboundItem = {
    id: newId(),
    type: input.type,
    payload: input.payload,
    createdAt: Date.now(),
    state: 'queued',
  };
  await store.add(item);
  return item;
}

/** Build the JMAP request that replays one queued item. */
export function outboundToRequest(item: OutboundItem): JmapRequest {
  switch (item.type) {
    case 'flag': {
      const p = item.payload as FlagPayload;
      return request(MAIL_USING, [
        [
          'Email/set',
          { accountId: p.accountId, update: { [p.emailId]: { [`keywords/${p.keyword}`]: p.value ? true : null } } },
          'set',
        ],
      ]);
    }
    case 'move': {
      const p = item.payload as MovePayload;
      return request(MAIL_USING, [
        ['Email/set', { accountId: p.accountId, update: { [p.emailId]: { mailboxIds: p.mailboxIds } } }, 'set'],
      ]);
    }
    case 'draft': {
      const p = item.payload as DraftPayload;
      return request(MAIL_USING, [
        [
          'Email/set',
          {
            accountId: p.accountId,
            create: {
              draft: {
                mailboxIds: { [p.draft.draftMailboxId]: true },
                keywords: { $draft: true, $seen: true },
                from: [p.draft.from],
                to: parseRecipients(p.draft.to),
                subject: p.draft.subject,
                htmlBody: [{ partId: 'body', type: 'text/html' }],
                bodyValues: { body: { value: p.draft.htmlBody } },
              },
            },
          },
          'set',
        ],
      ]);
    }
    case 'send': {
      const p = item.payload as SendPayload;
      return sendEnvelope(p.accountId, p.draft);
    }
    case 'pim': {
      // Replay the captured PIM request verbatim (Calendar/Task/Note/Contact set).
      return (item.payload as PimPayload).request;
    }
  }
}

/** Build a `pim` outbound item's payload from a prepared JMAP mutation request. */
export function pimOutbound(request: JmapRequest, callId: string): PimPayload {
  return { request, callId };
}

/** Queue a PIM mutation for offline replay (plan §1.8 outbound queue `type:"pim"`). */
export function enqueuePimMutation(
  store: OutboxStore,
  request: JmapRequest,
  callId: string,
): Promise<OutboundItem> {
  return enqueueOutbound(store, { type: 'pim', payload: pimOutbound(request, callId) });
}

/** Did the server actually apply the replayed item? Reconciles vs the response. */
export function outboundApplied(item: OutboundItem, res: JmapResponse): boolean {
  switch (item.type) {
    case 'flag':
    case 'move': {
      const r = responseFor<EmailSetResponse>(res, 'set');
      const p = item.payload as FlagPayload | MovePayload;
      return (
        r.updated !== null &&
        p.emailId in r.updated &&
        !(r.notUpdated !== null && p.emailId in r.notUpdated)
      );
    }
    case 'draft': {
      const r = responseFor<EmailSetResponse>(res, 'set');
      return r.created !== null && 'draft' in r.created;
    }
    case 'send': {
      const r = responseFor<EmailSubmissionSetResponse>(res, 'submit');
      return r.created !== null && 'send' in r.created && !(r.notCreated !== null && 'send' in r.notCreated);
    }
    case 'pim': {
      const p = item.payload as PimPayload;
      let r: PimSetResponse;
      try {
        r = responseFor<PimSetResponse>(res, p.callId);
      } catch {
        // Missing response or a JMAP method-level error → the item didn't apply.
        return false;
      }
      const empty = (m: Record<string, unknown> | null | undefined): boolean =>
        m === null || m === undefined || Object.keys(m).length === 0;
      return empty(r.notCreated) && empty(r.notUpdated) && empty(r.notDestroyed);
    }
  }
}

export interface DrainResult {
  sent: number;
  failed: number;
}

/**
 * Drain the queue FIFO. For each item: replay → if applied, delete it and count
 * `sent`; if the server rejected it, mark `failed` and keep it; if the network
 * is still down, re-queue it and STOP (preserving FIFO order for next reconnect).
 */
export async function drainOutbox(store: OutboxStore, client: Client): Promise<DrainResult> {
  const pending = (await store.all()).filter((i) => i.state !== 'sent');
  let sent = 0;
  let failed = 0;
  for (const item of pending) {
    try {
      const res = await client.jmap(outboundToRequest(item));
      if (outboundApplied(item, res)) {
        await store.delete(item.id);
        sent += 1;
      } else {
        await store.put({ ...item, state: 'failed' });
        failed += 1;
      }
    } catch (err) {
      if (err instanceof NetworkError) {
        // Still offline: leave it queued and stop; retry on the next reconnect.
        await store.put({ ...item, state: 'queued' });
        break;
      }
      // A JMAP-level / server error: the item itself is bad — mark it failed.
      await store.put({ ...item, state: 'failed' });
      failed += 1;
    }
  }
  return { sent, failed };
}

/** In-memory queue: the unit-test fake and the graceful fallback when IDB is absent. */
export function memoryOutboxStore(seed: OutboundItem[] = []): OutboxStore {
  const items = new Map<string, OutboundItem>(seed.map((i) => [i.id, i]));
  return {
    async add(item) {
      items.set(item.id, item);
    },
    async put(item) {
      items.set(item.id, item);
    },
    async all() {
      return [...items.values()].sort((a, b) => a.createdAt - b.createdAt);
    },
    async delete(id) {
      items.delete(id);
    },
  };
}
