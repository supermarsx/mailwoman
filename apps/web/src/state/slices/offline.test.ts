import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { activateAccountPreferences, suspendAccountPreferences } from '../../api/accountPreferenceLifecycle.ts';
beforeEach(() => activateAccountPreferences('acct1','test-storage-authority-acct1'));
afterEach(() => suspendAccountPreferences());
import { createRoot } from 'solid-js';
import { createOfflineSlice, type OfflineSlice } from './offline.ts';
import type { SliceContext } from './context.ts';
import type { Client } from '../../api/client.ts';
import {ApiError} from '../../api/client.ts';
import {reportOffline} from '../../offline/runtime.ts';
import * as offlineDb from '../../offline/idb.ts';
import * as serviceWorker from '../../sw/register.ts';
import {publishOfflineChange} from '../../offline/events.ts';
import type { JmapRequest, JmapResponse } from '../../api/jmap-types.ts';
import { memoryOutboxStore, type MovePayload, type OutboxStore, type SendPayload,type QueuedItem } from '../../offline/outbox.ts';
import { createAppState } from '../store.ts';
import { makeClient, mkEmail } from '../../components/appHarness.tsx';

const move: MovePayload = { accountId: 'acct1', emailId: 'm1', mailboxIds: { archive: true },previousMailboxIds:{inbox:true} };
const send: SendPayload = {
  accountId: 'acct1',
  draft: {
    from: { name: null, email: 'me@example.org' },
    to: 'you@example.org',
    subject: 'Hi',
    htmlBody: '<p>hi</p>',
    draftMailboxId: 'drafts1',
  },
};

const applied = (): JmapResponse => ({
  methodResponses: [['Email/set', { updated: { m1: null }, notUpdated: null }, 'set']],
  sessionState: 's',
});
const refused = (): JmapResponse => ({
  methodResponses: [
    ['Email/set', { updated: null, notUpdated: { m1: { type: 'notFound', description: 'no such message' } } }, 'set'],
  ],
  sessionState: 's',
});
const sentOk = (): JmapResponse => ({
  methodResponses: [
    ['Email/set', { created: { draft: { id: 'e1' } }, notCreated: null }, 'set'],
    ['EmailSubmission/set', { created: { send: { id: 's1' } }, notCreated: null }, 'submit'],
  ],
  sessionState: 's',
});

interface Rig {
  offline: OfflineSlice;
  store: OutboxStore;
  jmap: ReturnType<typeof vi.fn<Client['jmap']>>;
  toast: ReturnType<typeof vi.fn>;
  /** Report a network state change the way the client does. */
  network(up: boolean): void;
}

async function withSlice(run: (rig: Rig) => Promise<void>): Promise<void> {
  const listeners: ((up: boolean) => void)[] = [];
  const jmap = vi.fn<Client['jmap']>(async () => applied());
  const client = {
    me:async()=>({accountId:'acct1',storageAuthority:'test-storage-authority-acct1',username:'test'}),
    jmap:(request:JmapRequest,options:Parameters<Client['jmap']>[1])=>request.methodCalls[0]?.[2]==='baseline'?Promise.resolve({sessionState:'s',methodResponses:[['Email/get',{accountId:'acct1',state:'e1',list:[{id:'m1',mailboxIds:{inbox:true},keywords:{}}]},'baseline']]} as JmapResponse):jmap(request,options),
    onNetwork: (l: (up: boolean) => void) => {
      listeners.push(l);
      return () => undefined;
    },
  } as unknown as Client;
  const toast = vi.fn();
  const ctx: SliceContext = { client, showToast: toast };
  const store = memoryOutboxStore();
  await createRoot(async (dispose) => {
    const offline = createOfflineSlice(ctx, { store });
    await run({ offline, store, jmap, toast, network: (up) => listeners.forEach((l) => l(up)) });
    dispose();
  });
}

/** The compose+submit requests the client was handed. */
function submissions(jmap: Rig['jmap']): JmapRequest[] {
  return jmap.mock.calls.map((c) => c[0]).filter((r) => r.methodCalls.some((m) => m[0] === 'EmailSubmission/set'));
}

describe('offline slice — already-online cold startup',()=>{
 const identity={accountId:'acct1',storageAuthority:'test-storage-authority-acct1',username:'test'};
 const pending=():QueuedItem=>({id:'persisted-send',createdAt:1,state:'queued',type:'send',storageAuthority:identity.storageAuthority,payload:send});
 const clientFor=(me=vi.fn(async()=>identity))=>({me,jmap:vi.fn<Client['jmap']>(async()=>sentOk()),onNetwork:vi.fn(()=>()=>undefined)} as unknown as Client);
 afterEach(()=>{reportOffline(false);vi.restoreAllMocks();});
 it('replays a previously persisted owned send without any reconnect or browser online event',async()=>{
  reportOffline(false);const store=memoryOutboxStore([pending()]),client=clientFor();let dispose!:()=>void;
  try{
   createRoot(end=>{dispose=end;createOfflineSlice({client,online:()=>true,showToast:vi.fn()},{store});});
   await vi.waitFor(async()=>expect(await store.all()).toEqual([]));
   expect(client.me).toHaveBeenCalledOnce();expect(client.jmap).toHaveBeenCalledOnce();
   expect(vi.mocked(client.jmap).mock.calls[0]![0].methodCalls.some(call=>call[0]==='EmailSubmission/set')).toBe(true);
  }finally{dispose();}
 });
 it('does not make a fresh identity request for an empty owned queue',async()=>{
  const store=memoryOutboxStore(),client=clientFor();let dispose!:()=>void;
  try{createRoot(end=>{dispose=end;createOfflineSlice({client,online:()=>true,showToast:vi.fn()},{store});});await new Promise(resolve=>setTimeout(resolve,0));expect(client.me).not.toHaveBeenCalled();expect(client.jmap).not.toHaveBeenCalled();}
  finally{dispose();}
 });
 it.each(['cached','refused','held'] as const)('keeps the queue intact when fresh verification is %s',async kind=>{
  reportOffline(false);const store=memoryOutboxStore([pending()]),me=vi.fn(async()=>{if(kind==='refused')throw new ApiError(401,'Signed out');if(kind==='cached')reportOffline(true);return {...identity,...(kind==='held'?{passwordChangeRequired:true}:{})};}),client=clientFor(me);Object.assign(client,{offlineReads:true});let dispose!:()=>void;
  try{createRoot(end=>{dispose=end;createOfflineSlice({client,online:()=>true,showToast:vi.fn()},{store});});await vi.waitFor(()=>expect(me).toHaveBeenCalledOnce());await new Promise(resolve=>setTimeout(resolve,0));expect(client.jmap).not.toHaveBeenCalled();expect((await store.all()).map(item=>item.id)).toEqual(['persisted-send']);}
  finally{dispose();}
 });
 it.each(['core','runtime','browser'] as const)('does not initiate a cold replay while %s reports offline',async kind=>{
  reportOffline(kind==='runtime');if(kind==='browser')vi.spyOn(navigator,'onLine','get').mockReturnValue(false);
  const store=memoryOutboxStore([pending()]),client=clientFor();let dispose!:()=>void;
  try{createRoot(end=>{dispose=end;createOfflineSlice({client,online:()=>kind!=='core',showToast:vi.fn()},{store});});await new Promise(resolve=>setTimeout(resolve,0));expect(client.me).not.toHaveBeenCalled();expect(client.jmap).not.toHaveBeenCalled();}
  finally{dispose();}
 });
 it.each(['account','dispose'] as const)('does not resume after %s changes during the pending queue read',async change=>{
  const store=memoryOutboxStore([pending()]),all=store.all;let release!:(items:Awaited<ReturnType<OutboxStore['all']>>)=>void;
  store.all=vi.fn().mockImplementationOnce(()=>new Promise(resolve=>{release=resolve;})).mockImplementation(all);
  const client=clientFor();let dispose!:()=>void;
  try{createRoot(end=>{dispose=end;createOfflineSlice({client,online:()=>true,showToast:vi.fn()},{store});});if(change==='account')activateAccountPreferences('other','test-storage-authority-other');else dispose();release([pending()]);await new Promise(resolve=>setTimeout(resolve,0));expect(client.me).not.toHaveBeenCalled();expect(client.jmap).not.toHaveBeenCalled();}
  finally{dispose();}
 });
 it.each(['account','dispose'] as const)('does not send after %s changes during fresh identity verification',async change=>{
  const store=memoryOutboxStore([pending()]);let release!:(value:typeof identity)=>void;
  const me=vi.fn(()=>new Promise<typeof identity>(resolve=>{release=resolve;})),client=clientFor(me);let dispose!:()=>void;
  try{createRoot(end=>{dispose=end;createOfflineSlice({client,online:()=>true,showToast:vi.fn()},{store});});await vi.waitFor(()=>expect(me).toHaveBeenCalledOnce());if(change==='account')activateAccountPreferences('other','test-storage-authority-other');else dispose();release(identity);await new Promise(resolve=>setTimeout(resolve,0));expect(client.jmap).not.toHaveBeenCalled();expect(await store.all()).toHaveLength(1);}
  finally{dispose();}
 });
 it('does not verify or replay pending work belonging to an inactive account',async()=>{
  const store=memoryOutboxStore([{...pending(),storageAuthority:'test-storage-authority-other',payload:{...send,accountId:'other'}}]),client=clientFor();let dispose!:()=>void;
  try{createRoot(end=>{dispose=end;createOfflineSlice({client,online:()=>true,showToast:vi.fn()},{store});});await new Promise(resolve=>setTimeout(resolve,0));expect(client.me).not.toHaveBeenCalled();expect(client.jmap).not.toHaveBeenCalled();expect(await store.all()).toHaveLength(1);}
  finally{dispose();}
 });
 it('wakes persisted inactive contexts after verified activation and online events without a queue-change wake loop',async()=>{
  suspendAccountPreferences();vi.spyOn(offlineDb,'idbAvailable').mockReturnValue(true);vi.spyOn(serviceWorker,'registerServiceWorker').mockResolvedValue();
  const wake=vi.spyOn(serviceWorker,'requestBackgroundReplay').mockResolvedValue(false),store=memoryOutboxStore(),client=clientFor();let dispose!:()=>void;
  try{
   createRoot(end=>{dispose=end;createOfflineSlice({client,online:()=>true,showToast:vi.fn()},{store});});await new Promise(resolve=>setTimeout(resolve,0));wake.mockClear();
   activateAccountPreferences('acct1',identity.storageAuthority);await new Promise(resolve=>setTimeout(resolve,0));expect(wake).toHaveBeenCalledOnce();expect(client.me).not.toHaveBeenCalled();
   publishOfflineChange('queue');await new Promise(resolve=>setTimeout(resolve,0));expect(wake).toHaveBeenCalledOnce();
   window.dispatchEvent(new Event('online'));expect(wake).toHaveBeenCalledTimes(2);
  }finally{dispose();}
 });
});

describe('offline slice — queue and undo', () => {
  it('clears visible data and cannot replay or mutate another account queue', async () => {
    await withSlice(async ({ offline, store, jmap }) => {
      const id = await offline.enqueueOffline('move', move);
      offline.cacheHeaders([mkEmail('private')]);
      suspendAccountPreferences();
      expect(offline.offlineQueuePending()).toBe(0);
      expect(await offline.replayOffline()).toEqual({ sent: 0, failed: 0 });
      activateAccountPreferences('other','test-storage-authority-other');
      expect(await offline.dequeueOffline(id)).toBe(false);
      await offline.replayOffline();
      expect(jmap).not.toHaveBeenCalled();
      expect(await store.all()).toHaveLength(1);
      await expect(offline.enqueueOffline('move', move)).rejects.toThrow('active account');
      activateAccountPreferences('acct1','test-storage-authority-acct1');
      // Once its verified owner returns online, the persisted change resumes
      // automatically; the other account above never got to dispatch it.
      await vi.waitFor(async() => expect(await store.all()).toEqual([]));
      expect(offline.offlineQueuePending()).toBe(0);expect(jmap).toHaveBeenCalledOnce();
    });
  });

  it('aborts a pending replay when the same raw account moves to another verified realm',async()=>{
    await withSlice(async({offline,store,jmap})=>{
      await offline.enqueueOffline('move',move);
      jmap.mockImplementationOnce((_request,options)=>new Promise((_resolve,reject)=>{
        options?.signal?.addEventListener('abort',()=>reject(new DOMException('Authority changed','AbortError')),{once:true});
      }));
      const draining=offline.replayOffline();
      await vi.waitFor(()=>expect(jmap).toHaveBeenCalledTimes(1));
      const signal=jmap.mock.calls[0]?.[1]?.signal;
      expect(signal?.aborted).toBe(false);
      activateAccountPreferences('acct1','test-storage-authority-different-realm');
      expect(signal?.aborted).toBe(true);
      expect(await draining).toEqual({sent:0,failed:0});
      expect(await store.all()).toHaveLength(1);
      expect(await offline.replayOffline()).toEqual({sent:0,failed:0});
      expect(jmap).toHaveBeenCalledTimes(1);
    });
  });

  it('enqueue resolves to the item id and counts it; dequeue removes it before it is sent', async () => {
    await withSlice(async ({ offline, store, jmap }) => {
      const id = await offline.enqueueOffline('move', move);
      expect((await store.all()).map((i) => i.id)).toEqual([id]);
      expect(offline.offlineQueuePending()).toBe(1);

      expect(await offline.dequeueOffline(id)).toBe(true);
      expect(offline.offlineQueuePending()).toBe(0);
      expect(await offline.replayOffline()).toEqual({ sent: 0, failed: 0 });
      expect(jmap).not.toHaveBeenCalled();
    });
  });

  it('dequeue says false once the item has been replayed', async () => {
    await withSlice(async ({ offline, jmap }) => {
      const id = await offline.enqueueOffline('move', move);
      await offline.replayOffline();
      expect(jmap).toHaveBeenCalledTimes(1);
      expect(await offline.dequeueOffline(id)).toBe(false);
    });
  });
});

describe('offline slice — reconnect', () => {
  it('the network edge and the browser online event together submit a queued send exactly once', async () => {
    await withSlice(async ({ offline, store, jmap, toast, network }) => {
      jmap.mockImplementation(async () => {
        await new Promise((r) => setTimeout(r, 5));
        return sentOk();
      });
      await offline.enqueueOffline('send', send);
      network(false);
      // Precondition: nothing has been sent while offline.
      expect(jmap).not.toHaveBeenCalled();

      // Both triggers, in the same tick, as a real reconnect produces them.
      network(true);
      window.dispatchEvent(new Event('online'));

      await vi.waitFor(async () => expect(await store.all()).toEqual([]));
      await vi.waitFor(() => expect(offline.offlineQueuePending()).toBe(0));
      expect(submissions(jmap)).toHaveLength(1);
      expect(jmap).toHaveBeenCalledTimes(1);
      expect(toast).toHaveBeenCalledWith('success', 'Sent 1 queued change');
    });
  });
});

describe('offline slice — failed items', () => {
  it('a refused item leaves the pending count and is listed with its reason', async () => {
    await withSlice(async ({ offline, jmap, toast }) => {
      jmap.mockResolvedValueOnce(refused());
      const id = await offline.enqueueOffline('move', move);
      // Precondition: it is pending, not failed, before the replay.
      expect(offline.offlineQueuePending()).toBe(1);
      expect(offline.offlineFailed()).toEqual([]);

      expect(await offline.replayOffline()).toEqual({ sent: 0, failed: 1 });

      expect(offline.offlineQueuePending()).toBe(0);
      expect(offline.offlineFailed().map((i) => [i.id, i.type, i.state, i.lastError])).toEqual([
        [id, 'move', 'failed', 'no such message'],
      ]);
      expect(toast).toHaveBeenCalledWith('error', '1 queued change failed');
    });
  });

  it('retry replays it at once; when the server now applies it, it leaves the list', async () => {
    await withSlice(async ({ offline, store, jmap, toast }) => {
      jmap.mockResolvedValueOnce(refused());
      const id = await offline.enqueueOffline('move', move);
      await offline.replayOffline();
      expect(offline.offlineFailed()).toHaveLength(1);
      // A plain replay leaves a failed item alone.
      await offline.replayOffline();
      expect(jmap).toHaveBeenCalledTimes(1);

      await offline.retryOffline(id);

      expect(jmap).toHaveBeenCalledTimes(2);
      expect(offline.offlineFailed()).toEqual([]);
      expect(await store.all()).toEqual([]);
      expect(toast).toHaveBeenLastCalledWith('success', 'Sent 1 queued change');
    });
  });

  it('discard drops it without sending it again', async () => {
    await withSlice(async ({ offline, store, jmap, toast }) => {
      jmap.mockResolvedValueOnce(refused());
      const id = await offline.enqueueOffline('move', move);
      await offline.replayOffline();
      expect(offline.offlineFailed()).toHaveLength(1);

      await offline.discardOffline(id);

      expect(offline.offlineFailed()).toEqual([]);
      expect(await store.all()).toEqual([]);
      expect(jmap).toHaveBeenCalledTimes(1);
      expect(toast).toHaveBeenLastCalledWith('info', 'Discarded. The change was not applied.');
    });
  });
});

describe('the composed store — an offline move and its undo', () => {
  it('undo takes the queued move out again: nothing pending, nothing sent', async () => {
    const client = makeClient({ emails: [mkEmail('a'), mkEmail('b')] });
    await createRoot(async (dispose) => {
      const app = createAppState(client);
      await app.login({ username: 'me@example.org', password: 'p' });
      // Go offline the way the client reports it.
      for (const call of vi.mocked(client.onNetwork).mock.calls) call[0](false);
      expect(app.online()).toBe(false);
      vi.mocked(client.jmap).mockClear();

      await app.archiveMessage('a');
      // Precondition: the move went into the offline queue, not to the server.
      expect(app.offlineQueuePending()).toBe(1);
      expect(app.messages().map((m) => m.id)).toEqual(['b']);

      await app.undoNow();

      expect(app.offlineQueuePending()).toBe(0);
      expect(app.messages().map((m) => m.id)).toEqual(['a', 'b']);
      expect(await app.replayOffline()).toEqual({ sent: 0, failed: 0 });
      expect(client.jmap).not.toHaveBeenCalled();
      dispose();
    });
  });
});
