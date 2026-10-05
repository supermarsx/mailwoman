// Offline slice (plan §3 e5). Wires the offline surface into `AppState`: the
// offline-queue pending count, the items the queue has given up on, queue
// actions (replayed on reconnect), a cached header slice, and the reduced
// offline search over it. The Service Worker (public/sw.js) + OPFS encrypted
// cache + IndexedDB queue live under offline/** and sw/**; this slice is the
// store-facing seam.
//
// NOTE (boundary for e7): `offlineQueuePending` counts the OFFLINE REPLAY queue
// (mutations captured while offline), NOT e7's submission Outbox
// (`EmailSubmission/query`, server-held send-later / undo-send). See
// offline/outbox.ts for the full boundary note.

import { createSignal, type Accessor } from 'solid-js';
import type { Email } from '../../api/jmap-types.ts';
import type { OutboundType } from '../../contracts/offline.ts';
import { t } from '../../i18n/index.ts';
import { idbAvailable, idbOutboxStore } from '../../offline/idb.ts';
import {
  discardOutbound,
  drainOutbox,
  enqueueOutbound,
  memoryOutboxStore,
  removeQueued,
  retryOutbound,
  type DrainResult,
  type OutboxStore,
  type QueuedItem,
} from '../../offline/outbox.ts';
import { offlineSearch, type OfflineQuery } from '../../offline/search.ts';
import { registerServiceWorker } from '../../sw/register.ts';
import type { SliceContext } from './context.ts';

export interface OfflineSlice {
  /** Mutations queued while offline that are still waiting to be replayed.
   *  Items the queue has given up on are in `offlineFailed`, not counted here.
   *  Distinct from e7's submission Outbox (server-held send-later / undo-send). */
  offlineQueuePending: Accessor<number>;
  /** Queued mutations the server refused, or whose replay kept erroring, oldest
   *  first. They stay until the user retries or discards them. */
  offlineFailed: Accessor<QueuedItem[]>;
  /** Queue a mutation to replay on reconnect (use when a write happens offline).
   *  Resolves to the queued item's id, which `dequeueOffline` takes. */
  enqueueOffline(type: OutboundType, payload: unknown): Promise<string>;
  /** Take a queued mutation back out before it is replayed. Resolves `false` if
   *  it is no longer in the queue (it was replayed already). */
  dequeueOffline(id: string): Promise<boolean>;
  /** Put a failed item back in line and replay the queue now. */
  retryOffline(id: string): Promise<void>;
  /** Drop a failed item. The change it carried is not applied. */
  discardOffline(id: string): Promise<void>;
  /** Drain the offline queue FIFO against the server; returns a summary. */
  replayOffline(): Promise<DrainResult>;
  /** Reduced field/substring search over the cached header slice ("limited offline"). */
  searchOffline(query: OfflineQuery): Email[];
  /** Replace the in-memory cached header slice offline search reads from. */
  cacheHeaders(headers: Email[]): void;
}

/** What a slice needs beyond the shared context; `store` is injectable so a
 *  test can seed and inspect the queue. */
export interface OfflineSliceDeps {
  store?: OutboxStore;
}

export function createOfflineSlice(ctx: SliceContext, deps: OfflineSliceDeps = {}): OfflineSlice {
  const { client, showToast } = ctx;

  const [offlineQueuePending, setPending] = createSignal(0);
  const [offlineFailed, setFailed] = createSignal<QueuedItem[]>([]);
  const [cachedHeaders, setCachedHeaders] = createSignal<Email[]>([]);

  // IndexedDB in the browser; an in-memory queue under jsdom / unsupported envs.
  const store: OutboxStore = deps.store ?? (idbAvailable() ? idbOutboxStore() : memoryOutboxStore());

  /** Re-read the queue into the two signals the UI shows. */
  async function refresh(): Promise<void> {
    const items = (await store.all()) as QueuedItem[];
    setPending(items.filter((i) => i.state === 'queued').length);
    setFailed(items.filter((i) => i.state === 'failed'));
  }

  async function enqueueOffline(type: OutboundType, payload: unknown): Promise<string> {
    const item = await enqueueOutbound(store, { type, payload });
    await refresh();
    return item.id;
  }

  async function dequeueOffline(id: string): Promise<boolean> {
    const removed = await removeQueued(store, id);
    await refresh();
    return removed;
  }

  async function replayOffline(): Promise<DrainResult> {
    const result = await drainOutbox(store, client);
    await refresh();
    if (result.sent > 0) showToast('success', t('mail-queue-sent', { count: result.sent }));
    if (result.failed > 0) showToast('error', t('mail-queue-failed', { count: result.failed }));
    return result;
  }

  async function retryOffline(id: string): Promise<void> {
    if (await retryOutbound(store, id)) await replayOffline();
    else await refresh();
  }

  async function discardOffline(id: string): Promise<void> {
    await discardOutbound(store, id);
    await refresh();
    showToast('info', t('mail-queue-discarded'));
  }

  function searchOffline(query: OfflineQuery): Email[] {
    return offlineSearch(cachedHeaders(), query);
  }

  function cacheHeaders(headers: Email[]): void {
    setCachedHeaders(headers);
  }

  // Replay only on an offline→online transition (client.onNetwork fires `up` on
  // every successful request, so gate on the recovery edge).
  let wasOffline = false;
  client.onNetwork((up) => {
    if (!up) {
      wasOffline = true;
      return;
    }
    if (wasOffline) {
      wasOffline = false;
      void replayOffline();
    }
  });
  // The browser's own reconnect signal, independent of in-flight requests. It
  // usually fires together with the edge above; `drainOutbox` runs one drain at
  // a time, so the two do not replay the queue twice.
  if (typeof window !== 'undefined') {
    window.addEventListener('online', () => void replayOffline());
  }

  // Hydrate the counts + register the SW; both no-op under jsdom tests.
  if (deps.store !== undefined || idbAvailable()) void refresh();
  void registerServiceWorker();

  return {
    offlineQueuePending,
    offlineFailed,
    enqueueOffline,
    dequeueOffline,
    retryOffline,
    discardOffline,
    replayOffline,
    searchOffline,
    cacheHeaders,
  };
}
