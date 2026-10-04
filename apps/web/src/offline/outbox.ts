// Offline outbound queue (contract store `mw-outbox`, plan §2.5): capture
// mutations made while the network is down, then drain them FIFO on reconnect →
// JMAP → reconcile against the set/submission response.
//
// An item leaves the queue in one of three ways: the server applied it (it is
// deleted), the user removed it (`removeQueued` before it was sent,
// `discardOutbound` after it failed), or it is given up on — state `failed`,
// which no drain touches again until the user asks (`retryOutbound`). A drain
// only ever replays `queued` items, so a refused item is not re-sent on every
// reconnect.
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

/**
 * How many times an item is replayed when the replay itself errors (an HTTP 5xx,
 * an expired session, a JMAP method-level error) before it is marked `failed`.
 * A refusal of the item — the server answered and said no — is not retried at
 * all: asking again gets the same answer.
 */
export const MAX_ATTEMPTS = 3;

/**
 * A queue row as this module stores it: the frozen contract's `OutboundItem`
 * plus two optional fields the contract has no place for. Rows written before
 * these existed simply lack them (`attempts` reads as 0).
 */
export interface QueuedItem extends OutboundItem {
  /** Replays that errored so far (see MAX_ATTEMPTS). */
  attempts?: number;
  /** Why the last replay did not apply: the server's SetError, or the error thrown. */
  lastError?: string;
}

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

/**
 * Why the server did not apply a replayed item, from the response: the
 * SetError's description when there is one, else its type. `null` when the
 * response carries no SetError for the item (it simply is not in `created` /
 * `updated`).
 */
export function outboundRejection(item: OutboundItem, res: JmapResponse): string | null {
  const reason = (err: { type?: string; description?: string | null } | undefined): string | null => {
    if (err === undefined) return null;
    if (typeof err.description === 'string' && err.description.length > 0) return err.description;
    return typeof err.type === 'string' ? err.type : null;
  };
  try {
    switch (item.type) {
      case 'flag':
      case 'move': {
        const p = item.payload as FlagPayload | MovePayload;
        return reason(responseFor<EmailSetResponse>(res, 'set').notUpdated?.[p.emailId]);
      }
      case 'draft':
        return reason(responseFor<EmailSetResponse>(res, 'set').notCreated?.['draft']);
      case 'send':
        return (
          reason(responseFor<EmailSetResponse>(res, 'set').notCreated?.['draft']) ??
          reason(responseFor<EmailSubmissionSetResponse>(res, 'submit').notCreated?.['send'])
        );
      case 'pim':
        return null;
    }
  } catch (err) {
    // A method-level error for the call (`responseFor` throws on those).
    return err instanceof Error ? err.message : null;
  }
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
  /** Applied by the server and removed from the queue. */
  sent: number;
  /** Given up on in this drain (now state `failed`; kept until retried or discarded). */
  failed: number;
}

/**
 * Drain the queue FIFO. Only `queued` items are replayed. For each:
 *  - applied → delete it, count `sent`;
 *  - the server answered and refused it → `failed` at once, with the reason;
 *  - the replay errored (not a network failure) → one more attempt recorded;
 *    `failed` once MAX_ATTEMPTS is reached, otherwise left `queued` for the
 *    next drain (and in neither count);
 *  - the network is still down → nothing recorded, and STOP, preserving FIFO
 *    order for the next reconnect. Being offline is not an attempt.
 *
 * Only one drain runs at a time. Two callers fire together on every reconnect
 * (the client's first successful request and the browser's `online` event);
 * each would read the queue before the other had deleted anything, and every
 * queued send went out twice. A call made while a drain of the same store is
 * running gets that drain's result. Across tabs, which share the IndexedDB
 * queue, a Web Lock does the same job where the browser has one: a tab that
 * finds the lock taken skips its drain, since the holder is draining the same
 * rows.
 */
export function drainOutbox(store: OutboxStore, client: Client): Promise<DrainResult> {
  const running = draining.get(store);
  if (running !== undefined) return running;
  const run = withQueueLock(() => drainOnce(store, client)).finally(() => draining.delete(store));
  draining.set(store, run);
  return run;
}

/** The drain in flight for each store, if any. */
const draining = new WeakMap<OutboxStore, Promise<DrainResult>>();

/** Name of the cross-tab lock held for the length of a drain. */
const DRAIN_LOCK = 'mw-outbox-drain';

/** The slice of the Web Locks API used here (absent in jsdom and older WebViews). */
interface LockManagerLike {
  request<T>(name: string, options: { ifAvailable: boolean }, callback: (lock: unknown) => Promise<T>): Promise<T>;
}

async function withQueueLock(drain: () => Promise<DrainResult>): Promise<DrainResult> {
  const locks =
    typeof navigator !== 'undefined' ? (navigator as { locks?: LockManagerLike }).locks : undefined;
  if (locks === undefined) return drain();
  return locks.request(DRAIN_LOCK, { ifAvailable: true }, (lock) =>
    lock === null ? Promise.resolve({ sent: 0, failed: 0 }) : drain(),
  );
}

async function drainOnce(store: OutboxStore, client: Client): Promise<DrainResult> {
  const pending = ((await store.all()) as QueuedItem[]).filter((i) => i.state === 'queued');
  let sent = 0;
  let failed = 0;
  for (const item of pending) {
    try {
      const res = await client.jmap(outboundToRequest(item));
      if (outboundApplied(item, res)) {
        await store.delete(item.id);
        sent += 1;
      } else {
        const lastError = outboundRejection(item, res);
        const refused: QueuedItem = {
          ...item,
          state: 'failed',
          attempts: (item.attempts ?? 0) + 1,
          ...(lastError !== null ? { lastError } : {}),
        };
        await store.put(refused);
        failed += 1;
      }
    } catch (err) {
      if (err instanceof NetworkError) break;
      const attempts = (item.attempts ?? 0) + 1;
      const gaveUp = attempts >= MAX_ATTEMPTS;
      const errored: QueuedItem = {
        ...item,
        state: gaveUp ? 'failed' : 'queued',
        attempts,
        lastError: err instanceof Error ? err.message : String(err),
      };
      await store.put(errored);
      if (gaveUp) failed += 1;
    }
  }
  return { sent, failed };
}

/** The items the queue has given up on, oldest first. */
export async function failedOutbound(store: OutboxStore): Promise<QueuedItem[]> {
  return ((await store.all()) as QueuedItem[]).filter((i) => i.state === 'failed');
}

/**
 * Take an item out of the queue before it has been sent (offline undo). Returns
 * `true` if it was still there and is now gone; `false` if it is not in the
 * queue any more — it was already replayed, so the caller has to reverse the
 * change on the server instead.
 */
export async function removeQueued(store: OutboxStore, id: string): Promise<boolean> {
  const present = (await store.all()).some((i) => i.id === id);
  if (present) await store.delete(id);
  return present;
}

/** Put a failed item back in line with a fresh attempt count. No-op for an id
 *  that is not a failed item. Returns whether anything was re-queued. */
export async function retryOutbound(store: OutboxStore, id: string): Promise<boolean> {
  const item = ((await store.all()) as QueuedItem[]).find((i) => i.id === id && i.state === 'failed');
  if (item === undefined) return false;
  const { lastError: _dropped, ...rest } = item;
  const requeued: QueuedItem = { ...rest, state: 'queued', attempts: 0 };
  await store.put(requeued);
  return true;
}

/** Drop a failed item for good. The change it carried is not applied. */
export async function discardOutbound(store: OutboxStore, id: string): Promise<void> {
  await store.delete(id);
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
