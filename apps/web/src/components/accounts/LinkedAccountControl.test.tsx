// t71: switching mailboxes must not take the mailbox shell down. The mailbox
// renders under the route's <Suspense> (LazyRoute); the account control's
// linked-mailbox list is re-fetched on every switch (the control is keyed on
// the mailbox's preference account), and reading it while it loads used to
// suspend that boundary, swapping the whole shell for a "Loading…" card.
import {afterEach,beforeEach,describe,expect,it,vi} from 'vitest';
import {render,waitFor} from '@solidjs/testing-library';
import {Suspense} from 'solid-js';
import {loadSettings} from '../../api/settings.ts';
import {WORKSPACE_STORAGE_KEY} from '../../realtime/workspaceStorage.ts';
import {activateAccountPreferences,suspendAccountPreferences} from '../../api/accountPreferenceLifecycle.ts';
import {invalidateLinkedAccounts} from '../../api/linkedMailboxSession.ts';
import {AppContext} from '../../state/context.ts';
import {createAppState} from '../../state/store.ts';
import {makeClient,mkEmail} from '../appHarness.tsx';
import {MailboxScreen} from '../../screens/Mailbox.tsx';
import {RealtimeContext} from '../../realtime/context.ts';
import {createRealtimeController} from '../../realtime/controller.ts';

const AUTHORITY='test-storage-authority-acct1';
const CHILD_AUTHORITY='child-storage-authority-acct2';
const linked={primary:{accountId:'acct1',address:'me@example.org',storageAuthority:AUTHORITY},
  accounts:[{id:'link-1',accountId:'acct2',address:'child@example.org',displayName:'Child',storageAuthority:CHILD_AUTHORITY,status:'ready',revision:'1'}],
  policy:{enabled:true,maxAccounts:4,supportedAuthentication:['password']}};

beforeEach(async()=>{suspendAccountPreferences();localStorage.removeItem(WORKSPACE_STORAGE_KEY);history.replaceState(null,'','#/mail');await new Promise(resolve=>setTimeout(resolve,0));});
afterEach(()=>{vi.unstubAllGlobals();vi.restoreAllMocks();});

/** /api/accounts answers only when released, so the in-flight state is observable. */
function stubAccounts(){
  const real=globalThis.fetch;
  const waiting:(()=>void)[]=[];
  let hold=false;
  vi.stubGlobal('fetch',vi.fn(async(input:RequestInfo|URL,init?:RequestInit)=>{
    const url=new URL(input instanceof Request?input.url:String(input),location.href);
    if(url.pathname==='/api/accounts'){
      if(hold)await new Promise<void>(resolve=>waiting.push(resolve));
      return new Response(JSON.stringify(linked),{status:200,headers:{'content-type':'application/json'}});
    }
    if(/^\/api\/accounts\/[^/]+\/switch$/.test(url.pathname))return new Response('{}',{status:409});
    if(url.pathname.startsWith('/api/settings'))return new Response(JSON.stringify({settings:{}}),{status:200,headers:{'content-type':'application/json'}});
    return real?real(input,init):new Response(null,{status:404});
  }));
  return {hold(on:boolean){hold=on;},release(){for(const resolve of waiting.splice(0))resolve();}};
}
async function mount(){
  const client=makeClient({emails:[mkEmail('a')]});
  const app=createAppState(client);const realtime=createRealtimeController();
  let fallbacks=0;
  const Fallback=()=>{fallbacks++;return <p data-testid="route-fallback">Loading…</p>;};
  const view=render(()=><AppContext.Provider value={app}><RealtimeContext.Provider value={realtime}>
    <Suspense fallback={<Fallback/>}><MailboxScreen/></Suspense>
  </RealtimeContext.Provider></AppContext.Provider>);
  await loadSettings({refresh:true,fetcher:async()=>new Response(JSON.stringify({settings:{}}),{headers:{'Content-Type':'application/json'}})});
  await app.login({username:'me@example.org',password:'p'});
  await waitFor(()=>expect(view.container.querySelector('button[data-account-indicator="picker"]')).not.toBeNull());
  return {app,...view,fallbacks:()=>fallbacks};
}

describe('the account control while its list reloads',()=>{
  it('a refetch of the linked-mailbox list keeps the shell and the picker',async()=>{
    const accounts=stubAccounts();
    const {container,fallbacks}=await mount();
    const shell=container.querySelector('.workspace-shell')!;
    const before=fallbacks();
    accounts.hold(true);
    invalidateLinkedAccounts();
    await new Promise(resolve=>setTimeout(resolve,20));
    expect(container.querySelector('[data-testid="route-fallback"]')).toBeNull();
    expect(container.querySelector('.workspace-shell')).toBe(shell);
    expect(container.querySelector('button[data-account-indicator="picker"]')).not.toBeNull();
    accounts.release();
    await new Promise(resolve=>setTimeout(resolve,0));
    expect(fallbacks()).toBe(before);
  });

  it('a mailbox switch (a new preference account) keeps the shell, and the picker stands in with the last list',async()=>{
    const accounts=stubAccounts();
    const {container,fallbacks}=await mount();
    const shell=container.querySelector('.workspace-shell')!;
    const head=container.querySelector('.sidebar__account')!;
    const before=fallbacks();
    accounts.hold(true);
    // What switchLinkedMailbox does to the preference account: the control
    // remounts under the child mailbox and fetches its list again.
    activateAccountPreferences('acct2',CHILD_AUTHORITY);
    await new Promise(resolve=>setTimeout(resolve,20));
    expect(container.querySelector('[data-testid="route-fallback"]')).toBeNull();
    expect(container.querySelector('.workspace-shell')).toBe(shell);
    expect(container.querySelector('.sidebar__account')).toBe(head);
    // The list verified for the same sign-in mailbox, not the plain address.
    expect(container.querySelector('button[data-account-indicator="picker"]')).not.toBeNull();
    accounts.release();
    await waitFor(()=>expect(container.querySelector('button[data-account-indicator="picker"]')).not.toBeNull());
    expect(fallbacks()).toBe(before);
  });
});
