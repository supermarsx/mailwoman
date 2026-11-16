import {offlineServerBase} from '../../offline/cacheScope.ts';
import {publishOfflineChange,observeOfflineChanges} from '../../offline/events.ts';
import {activeLinkedMailbox} from '../../api/linkedMailboxSession.ts';
import {basePath} from '../../api/basePath.ts';
import {isOffline} from '../../offline/runtime.ts';
import {encryptedOutboxStore} from '../../offline/encryptedOutbox.ts';
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

import { createSignal, getOwner, onCleanup, type Accessor } from 'solid-js';
import { observePreferenceAccount, preferenceAccount, preferenceAccountIsCurrent, type PreferenceAccount } from '../../api/accountPreferenceLifecycle.ts';
import { accountOutbox } from '../../offline/accountOutbox.ts';
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
import { registerServiceWorker,requestBackgroundReplay } from '../../sw/register.ts';
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
  let disposed=false;
  const activeReplays=new Set<AbortController>();

  const [offlineQueuePending, setPending] = createSignal(0);
  const [offlineFailed, setFailed] = createSignal<QueuedItem[]>([]);
  const [cachedHeaders, setCachedHeaders] = createSignal<Email[]>([]);

  // IndexedDB in the browser; an in-memory queue under jsdom / unsupported envs.
  const store: OutboxStore = deps.store ?? (idbAvailable() ? encryptedOutboxStore(idbOutboxStore()) : memoryOutboxStore());

  const accountStores = new Map<string, OutboxStore>();
  function scoped(account: PreferenceAccount): OutboxStore {
    let scopedStore = accountStores.get(`${account.storageAuthority}:${account.id}`);
    if (!scopedStore) { scopedStore = accountOutbox(store, account.id,account.storageAuthority!); accountStores.set(`${account.storageAuthority}:${account.id}`, scopedStore); }
    return scopedStore;
  }
  function authority(): PreferenceAccount {
    const account = preferenceAccount();
    if (!account?.storageAuthority) throw new Error(t('mail-offline-storage-unavailable'));
    return account;
  }

  /** Re-read the queue into the two signals the UI shows. */
  async function refresh(): Promise<QueuedItem[]> {
    const account = preferenceAccount();
    if (disposed||!account?.storageAuthority) return [];
    const items = (await scoped(account).all()) as QueuedItem[];
    if (disposed||!preferenceAccountIsCurrent(account)) return [];
    setPending(items.filter((i) => i.state === 'queued').length);
    setFailed(items.filter((i) => i.state === 'failed'));
    return items;
  }

  async function enqueueOffline(type: OutboundType, payload: unknown): Promise<string> {
    const owner=authority();
    const item = await enqueueOutbound(scoped(owner), { type, payload,context:{linkId:activeLinkedMailbox()?.linkId??null,serverBase:basePath(),cacheBase:offlineServerBase()} });
    await refresh();
    publishOfflineChange('queue');void requestBackgroundReplay();
    return item.id;
  }

  async function dequeueOffline(id: string): Promise<boolean> {
    const removed = await removeQueued(scoped(authority()), id);
    await refresh();publishOfflineChange('queue');
    return removed;
  }

  async function replayOffline(): Promise<DrainResult> {
    const account = preferenceAccount();
    if (disposed||!account?.storageAuthority) return { sent: 0, failed: 0 };
    const live=await client.me();
    if(disposed||live.passwordChangeRequired||(client.offlineReads===true&&isOffline())||!preferenceAccountIsCurrent(account)||live.accountId!==account.id||live.storageAuthority!==account.storageAuthority)return {sent:0,failed:0};
    const controller=new AbortController();
    activeReplays.add(controller);
    const stop=observePreferenceAccount(()=>{if(!preferenceAccountIsCurrent(account))controller.abort();});
    let result:DrainResult;
    try {result=await drainOutbox(scoped(account), client, undefined, () => !disposed&&preferenceAccountIsCurrent(account),controller.signal);}
    finally {stop();activeReplays.delete(controller);}
    if (disposed||!preferenceAccountIsCurrent(account)) return result;
    await refresh();publishOfflineChange('queue');
    if (result.sent > 0) showToast('success', t('mail-queue-sent', { count: result.sent }));
    if (result.failed > 0) showToast('error', t('mail-queue-failed', { count: result.failed }));
    return result;
  }

  async function retryOffline(id: string): Promise<void> {
    if (await retryOutbound(scoped(authority()), id)) await replayOffline();
    else await refresh();
  }

  async function discardOffline(id: string): Promise<void> {
    await discardOutbound(scoped(authority()), id);
    await refresh();
    showToast('info', t('mail-queue-discarded'));
  }

  function searchOffline(query: OfflineQuery): Email[] {
    return offlineSearch(cachedHeaders(), query);
  }

  function cacheHeaders(headers: Email[]): void {
    if (preferenceAccount()) setCachedHeaders(headers);
  }

  // Replay only on an offline→online transition (client.onNetwork fires `up` on
  // every successful request, so gate on the recovery edge).
  function wakePersistedContexts():void {
    // The worker verifies each stored mailbox separately. An empty active queue
    // does not mean an inactive linked mailbox has no work waiting in IndexedDB.
    if(idbAvailable()&&preferenceAccount()?.storageAuthority)void requestBackgroundReplay().catch(()=>undefined);
  }
  let wasOffline = false;
  const stopNetwork = client.onNetwork((up) => {
    if (!up) {
      wasOffline = true;
      return;
    }
    if (wasOffline) {
      wasOffline = false;
      void replayOffline().catch(()=>undefined);
      wakePersistedContexts();
    }
  });
  // The browser's own reconnect signal, independent of in-flight requests. It
  // usually fires together with the edge above; `drainOutbox` runs one drain at
  // a time, so the two do not replay the queue twice.
  const online = (): void => { void replayOffline().catch(()=>undefined);wakePersistedContexts(); };
  if (typeof window !== 'undefined') window.addEventListener('online', online);
  const stopAccount = observePreferenceAccount(account => {
    setPending(0); setFailed([]); setCachedHeaders([]);
    if (account?.storageAuthority) void refresh().then(items=>{
      // A tab can reopen already online, with no reconnect edge (notably when
      // Background Sync is unavailable). Resume only persisted owned work;
      // replayOffline still verifies live Me before any mutation is sent.
      if(disposed||!preferenceAccountIsCurrent(account)||wasOffline||isOffline()||ctx.online?.()===false||typeof navigator!=='undefined'&&navigator.onLine===false)return;
      wakePersistedContexts();
      if(items.some(item=>item.state==='queued'))void replayOffline().catch(()=>undefined);
    }).catch(()=>undefined);
  });
  if (getOwner()) onCleanup(() => {
    disposed=true;
    for(const replay of activeReplays)replay.abort();activeReplays.clear();
    stopAccount(); stopNetwork();
    if (typeof window !== 'undefined') window.removeEventListener('online', online);
  });

  const stopChanges=observeOfflineChanges(kind=>{if(kind==='queue')void refresh().catch(()=>undefined);});
  const background=(event:MessageEvent)=>{if(event.data?.type==='mw-outbox-updated')void refresh().catch(()=>undefined);};
  if(typeof navigator!=='undefined'&&'serviceWorker' in navigator)navigator.serviceWorker.addEventListener?.('message',background);
  if(getOwner())onCleanup(()=>{stopChanges();if(typeof navigator!=='undefined'&&'serviceWorker' in navigator)navigator.serviceWorker.removeEventListener?.('message',background);});
  // Hydrate the counts + register the SW; both no-op under jsdom tests.

  void registerServiceWorker().then(()=>requestBackgroundReplay());

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
