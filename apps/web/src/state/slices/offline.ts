// Offline slice (plan §3 e5). Wires the offline surface into `AppState`: the
// honest offline-queue pending count, queue actions (replayed on reconnect), a
// cached header slice, and the reduced offline search over it. The Service
// Worker (public/sw.js) + OPFS encrypted cache + IndexedDB queue live under
// offline/** and sw/**; this slice is the store-facing seam.
//
// NOTE (boundary for e7): `offlineQueuePending` counts the OFFLINE REPLAY queue
// (mutations captured while offline), NOT e7's submission Outbox
// (`EmailSubmission/query`, server-held send-later / undo-send). See
// offline/outbox.ts for the full boundary note.

import { createSignal, type Accessor } from 'solid-js';
import type { Email } from '../../api/jmap-types.ts';
import type { OutboundType } from '../../contracts/offline.ts';
import { idbAvailable, idbOutboxStore } from '../../offline/idb.ts';
import {
  drainOutbox,
  enqueueOutbound,
  memoryOutboxStore,
  type DrainResult,
  type OutboxStore,
} from '../../offline/outbox.ts';
import { offlineSearch, type OfflineQuery } from '../../offline/search.ts';
import { registerServiceWorker } from '../../sw/register.ts';
import type { SliceContext } from './context.ts';

export interface OfflineSlice {
  /** Mutations queued while offline, awaiting replay. Distinct from e7's
   *  submission Outbox (server-held send-later / undo-send). */
  offlineQueuePending: Accessor<number>;
  /** Queue a mutation to replay on reconnect (use when a write happens offline). */
  enqueueOffline(type: OutboundType, payload: unknown): Promise<void>;
  /** Drain the offline queue FIFO against the server; returns a summary. */
  replayOffline(): Promise<DrainResult>;
  /** Reduced field/substring search over the cached header slice ("limited offline"). */
  searchOffline(query: OfflineQuery): Email[];
  /** Replace the in-memory cached header slice offline search reads from. */
  cacheHeaders(headers: Email[]): void;
}

export function createOfflineSlice(ctx: SliceContext): OfflineSlice {
  const { client, showToast } = ctx;

  const [offlineQueuePending, setPending] = createSignal(0);
  const [cachedHeaders, setCachedHeaders] = createSignal<Email[]>([]);

  // IndexedDB in the browser; an in-memory queue under jsdom / unsupported envs.
  const store: OutboxStore = idbAvailable() ? idbOutboxStore() : memoryOutboxStore();

  async function refreshCount(): Promise<void> {
    const items = await store.all();
    setPending(items.filter((i) => i.state !== 'sent').length);
  }

  async function enqueueOffline(type: OutboundType, payload: unknown): Promise<void> {
    await enqueueOutbound(store, { type, payload });
    await refreshCount();
  }

  async function replayOffline(): Promise<DrainResult> {
    const result = await drainOutbox(store, client);
    await refreshCount();
    if (result.sent > 0) {
      showToast('success', `Sent ${result.sent} queued ${result.sent === 1 ? 'change' : 'changes'}`);
    }
    if (result.failed > 0) {
      showToast('error', `${result.failed} queued ${result.failed === 1 ? 'change' : 'changes'} failed`);
    }
    return result;
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
  // The browser's own reconnect signal, independent of in-flight requests.
  if (typeof window !== 'undefined') {
    window.addEventListener('online', () => void replayOffline());
  }

  // Hydrate the pending count + register the SW; both no-op under jsdom tests.
  if (idbAvailable()) void refreshCount();
  void registerServiceWorker();

  return { offlineQueuePending, enqueueOffline, replayOffline, searchOffline, cacheHeaders };
}
