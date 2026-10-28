import {afterEach,beforeEach,describe,expect,it,vi} from 'vitest';
import {cleanup,fireEvent,render,screen,waitFor} from '@solidjs/testing-library';
import {MessageList} from './MessageList.tsx';
import {makeClient,mkEmail} from './appHarness.tsx';
import {createAppState} from '../state/store.ts';
import {AppContext} from '../state/context.ts';
import {suspendAccountPreferences} from '../api/accountPreferenceLifecycle.ts';
const sync=vi.hoisted(()=>({status:'syncing',synced:false,calls:[] as Record<string,unknown>[]}));
vi.mock('../api/transport.ts',async original=>{
 const actual=await original<typeof import('../api/transport.ts')>();
 return {...actual,createConfiguredClient:()=>{const real=actual.createConfiguredClient();return new Proxy(real,{get(target,key,receiver){
  if(key!=='jmap')return Reflect.get(target,key,receiver);
  return async(request:{methodCalls:[string,Record<string,unknown>,string][]},options?:unknown)=>{
   const call=request.methodCalls[0]!;
   if(call[0]!=='Mailbox/get'||call[2]!=='sync')return (target as {jmap:(r:unknown,o?:unknown)=>Promise<unknown>}).jmap(request,options);
   sync.calls.push(call[1]);if(call[1].mailwomanRetryInitialSync)sync.status='retrying';
   return {methodResponses:[['Mailbox/get',{accountId:call[1].accountId,mailwomanInitialSync:sync.status,list:(call[1].ids as string[]).map(id=>({id,mailwomanSynced:sync.synced}))},'sync']]};
  };
 }});}};
});
beforeEach(()=>{sync.status='syncing';sync.synced=false;sync.calls=[];});
afterEach(()=>{cleanup();suspendAccountPreferences();localStorage.clear();vi.useRealTimers();});
async function mount(emails=[] as ReturnType<typeof mkEmail>[]){
 const app=createAppState(makeClient({emails}));
 render(()=><AppContext.Provider value={app}><MessageList/></AppContext.Provider>);
 await app.login({username:'me@example.org',password:'p'});
 return app;
}
const list=()=>document.querySelector('section.list')!;
describe('MessageList during the first sync',()=>{
 it('shows density-sized skeleton rows, not "No messages", while the folder is not indexed',async()=>{
  const app=await mount();
  await waitFor(()=>expect(sync.calls.length).toBeGreaterThan(0));
  await waitFor(()=>expect(app.listLoading()).toBe(false));
  const pending=screen.getByTestId('async-pending');
  expect(pending.querySelector('[data-skeleton="messages"]')).not.toBeNull();
  expect(pending).toHaveAttribute('role','status');expect(pending).toHaveTextContent('Loading messages');
  expect(list()).toHaveAttribute('aria-busy','true');
  expect(document.querySelector('.list__empty')).toBeNull();
  expect(screen.queryByText(/Synchronizing your messages/)).toBeNull();
  const row=pending.querySelector<HTMLElement>('[data-skeleton="messages"] > div')!;expect(row.style.blockSize).toBe('72px');
 });
 it('shows "No messages" only once the server reports the folder indexed, and reloads it',async()=>{
  vi.useFakeTimers({shouldAdvanceTime:true});
  const app=await mount();await waitFor(()=>expect(screen.queryByTestId('async-pending')).not.toBeNull());
  const refresh=vi.spyOn(app,'refreshCurrentMailbox');
  sync.synced=true;sync.status='ready';await vi.advanceTimersByTimeAsync(2100);
  await waitFor(()=>expect(document.querySelector('.list__empty')).not.toBeNull());
  expect(refresh).toHaveBeenCalled();expect(screen.queryByTestId('async-pending')).toBeNull();expect(list()).toHaveAttribute('aria-busy','false');
 });
 it('offers a retry when indexing failed instead of pending forever',async()=>{
  sync.status='failed';await mount();
  const alert=await screen.findByRole('alert');expect(alert).toHaveTextContent(/could not be synchronized/i);
  expect(document.querySelector('.list__empty')).toBeNull();expect(screen.queryByTestId('async-pending')).toBeNull();
  fireEvent.click(screen.getByTestId('async-retry'));
  await waitFor(()=>expect(sync.calls.some(call=>call.mailwomanRetryInitialSync===true)).toBe(true));
  await waitFor(()=>expect(screen.getByTestId('async-pending').querySelector('[data-skeleton="messages"]')).not.toBeNull());
 });
 it('keeps listed rows and marks background indexing with a thin decorative line only',async()=>{
  await mount([mkEmail('a',{subject:'Already here'})]);
  await screen.findByText('Already here');
  await waitFor(()=>expect(screen.getByTestId('sync-progress')).toHaveAttribute('aria-hidden','true'));
  expect(screen.queryByTestId('async-pending')).toBeNull();expect(screen.queryByText(/Synchronizing/)).toBeNull();
 });
});
