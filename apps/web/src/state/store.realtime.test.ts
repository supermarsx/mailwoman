import {toasts,resetToasts} from '../components/toast/index.ts';
import {createRoot} from 'solid-js';
import {afterEach,expect,it,vi} from 'vitest';
import {createAppState} from './store.ts';
import {makeClient} from '../components/appHarness.tsx';
import type {RealtimeSliceOptions} from './slices/realtime.ts';
const captured=vi.hoisted(()=>({options:undefined as RealtimeSliceOptions|undefined}));
vi.mock('./slices/realtime.ts',async importOriginal=>{
 const actual=await importOriginal<typeof import('./slices/realtime.ts')>();
 return {...actual,createRealtimeSlice:(ctx:Parameters<typeof actual.createRealtimeSlice>[0],options:RealtimeSliceOptions)=>{captured.options=options;return actual.createRealtimeSlice(ctx,options);}};
});
afterEach(()=>{vi.restoreAllMocks();localStorage.clear();});
it('settles interrupted background Outbox refreshes for primary and linked pushes, while explicit refresh still rejects',async()=>{
 const client=makeClient();
 await createRoot(async dispose=>{try{
  const app=createAppState(client);await app.login({username:'me@example.org',password:'p'});
  const failure=new TypeError('Failed to fetch');vi.mocked(client.session).mockRejectedValue(failure);
  captured.options!.onChanged!(app.accountId()!,['EmailSubmission']);
  captured.options!.onActiveLinkedChanged!(['EmailSubmission']);
  // An unhandled rejection would fail this test through Vitest's rejection listener.
  await new Promise(resolve=>setTimeout(resolve,0));
  expect(app.outbox()).toEqual([]);expect(app.outboxLoading()).toBe(false);
  await expect(app.refreshOutbox()).rejects.toBe(failure);
 }finally{dispose();}});
});

it.each(['user','expired','failed'] as const)('stops session monitoring before %s sign-out and only confirms successful user sign-out',async reason=>{
 const client=makeClient();let expired=()=>{};
 client.onUnauthenticated=vi.fn(listener=>{expired=listener;return()=>{};});
 await createRoot(async dispose=>{try{
  resetToasts();const app=createAppState(client);await app.login({username:'me@example.org',password:'p'});
  app.startRealtime();expect(app.connectionState()).not.toBe('idle');
  vi.mocked(client.logout).mockImplementation(async()=>{
   expect(app.connectionState()).toBe('idle');expired();expect(app.connectionState()).toBe('idle');
   if(reason==='failed')throw new Error('server unavailable');
  });
  if(reason==='failed')await expect(app.logout()).rejects.toThrow('server unavailable');
  else await app.logout(reason);
  expect(app.me()).toBeNull();expired();expect(app.connectionState()).toBe('idle');
  expect(toasts().some(row=>row.message==='Signed out successfully')).toBe(reason==='user');
 }finally{dispose();resetToasts();}});
});
