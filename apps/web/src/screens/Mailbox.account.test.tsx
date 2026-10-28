// t38 §2.3: the signed-in address shows once, in the sidebar ("the webmail
// part"), never duplicated in the mail top bar; that one copy is the switcher.
import {beforeEach,afterEach,describe,expect,it,vi} from 'vitest';
import {render,fireEvent,screen,waitFor,within} from '@solidjs/testing-library';
import {loadSettings} from '../api/settings.ts';
import {WORKSPACE_STORAGE_KEY} from '../realtime/workspaceStorage.ts';
import {suspendAccountPreferences} from '../api/accountPreferenceLifecycle.ts';
import {setAccountUiPreferences} from '../api/accountPreferences.ts';
import {AppContext} from '../state/context.ts';
import {createAppState} from '../state/store.ts';
import {makeClient,mkEmail} from '../components/appHarness.tsx';
import {MailboxScreen} from './Mailbox.tsx';
import {RealtimeContext} from '../realtime/context.ts';
import {createRealtimeController} from '../realtime/controller.ts';
import {NARROW_QUERY} from '../components/narrowViewport.ts';
import {stripIsolates} from '../i18n/index.ts';

const AUTHORITY='test-storage-authority-acct1';
const linked={primary:{accountId:'acct1',address:'me@example.org',storageAuthority:AUTHORITY},
  accounts:[{id:'link-1',accountId:'acct2',address:'child@example.org',displayName:'Child',storageAuthority:'child-authority',status:'ready',revision:'1'}],
  policy:{enabled:true,maxAccounts:4,supportedAuthentication:['password']}};

beforeEach(async()=>{suspendAccountPreferences();localStorage.removeItem(WORKSPACE_STORAGE_KEY);history.replaceState(null,'','#/mail');await new Promise(resolve=>setTimeout(resolve,0));});
afterEach(()=>{vi.unstubAllGlobals();vi.restoreAllMocks();});

const calls:{method:string;path:string;body?:string}[]=[];
function stubAccounts(list:unknown|null){
  const real=globalThis.fetch;
  calls.length=0;
  vi.stubGlobal('fetch',vi.fn(async(input:RequestInfo|URL,init?:RequestInit)=>{
    const url=new URL(input instanceof Request?input.url:String(input),location.href);
    calls.push({method:init?.method??'GET',path:url.pathname,...(typeof init?.body==='string'?{body:init.body}:{})});
    if(url.pathname==='/api/accounts/primary')return new Response(init?.body as string,{status:200,headers:{'content-type':'application/json'}});
    if(url.pathname==='/api/accounts/layout'){const body=JSON.parse(init?.body as string) as Record<string,unknown>;return new Response(JSON.stringify({layout:{...body,revision:'r2'}}),{status:200,headers:{'content-type':'application/json'}});}
    if(/^\/api\/accounts\/[^/]+\/switch$/.test(url.pathname))return new Response('{}',{status:409});
    if(url.pathname==='/api/accounts')return list===null?new Response('{}',{status:503}):new Response(JSON.stringify(list),{status:200,headers:{'content-type':'application/json'}});
    if(url.pathname.startsWith('/api/settings'))return new Response(JSON.stringify({settings:{}}),{status:200,headers:{'content-type':'application/json'}});
    return real?real(input,init):new Response(null,{status:404});
  }));
}
function narrowViewport(){
  vi.stubGlobal('matchMedia',(query:string)=>({matches:query===NARROW_QUERY,media:query,addEventListener(){},removeEventListener(){},addListener(){},removeListener(){}}));
}
async function mount(){
  const client=makeClient({emails:[mkEmail('a')]});
  const app=createAppState(client);const realtime=createRealtimeController();
  const view=render(()=><AppContext.Provider value={app}><RealtimeContext.Provider value={realtime}><MailboxScreen/></RealtimeContext.Provider></AppContext.Provider>);
  await loadSettings({refresh:true,fetcher:async()=>new Response(JSON.stringify({settings:{}}),{headers:{'Content-Type':'application/json'}})});
  await app.login({username:'me@example.org',password:'p'});
  await waitFor(()=>expect(view.container.querySelector('.sidebar__account')).not.toBeNull());
  return {app,...view};
}
/** Indicators that are rendered and not inside a [hidden] subtree. */
function indicators(container:HTMLElement):Element[]{
  return [...container.querySelectorAll('.sidebar__account, [data-account-indicator]')]
    .filter(node=>!node.closest('[hidden]'))
    .filter((node,_,all)=>!all.some(other=>other!==node&&other.contains(node)));
}
const topBar=()=>screen.getByRole('toolbar',{name:'Workspace layout'});

describe('one account indicator',()=>{
  it('expanded folders: the sidebar head shows the address; the top bar has none',async()=>{
    stubAccounts({...linked,accounts:[],policy:{...linked.policy,enabled:false}});
    const {container}=await mount();
    const head=container.querySelector('.sidebar__head')!;
    await waitFor(()=>expect(head.querySelector('[data-account-address]')?.textContent&&stripIsolates(head.querySelector('[data-account-address]')!.textContent!)).toBe('me@example.org'));
    expect(indicators(container)).toHaveLength(1);
    expect(topBar().textContent).not.toContain('me@example.org');
    expect(within(topBar()).queryByRole('combobox')).toBeNull();
    expect(container.querySelector('[data-bar="mailTopBar"]')).toHaveAttribute('data-mode','pinned');
  });
  it('the switcher works from the sidebar head', async()=>{
    stubAccounts(linked);
    const {app,container}=await mount();
    const switcher=await within(container.querySelector('.sidebar__head') as HTMLElement).findByRole('button',{name:/^Mailbox account/});
    expect(switcher).toHaveAttribute('aria-haspopup','menu');
    const sw=vi.spyOn(app,'switchLinkedMailbox').mockResolvedValue(undefined);
    fireEvent.click(switcher);
    const menu=await screen.findByRole('menu',{name:'Mailbox account'});
    // No management entry in the picker: linking is managed in Settings.
    expect(within(menu).queryByText(/Manage/)).toBeNull();
    expect(within(menu).getAllByRole('menuitemradio')).toHaveLength(2);
    fireEvent.click(within(menu).getAllByRole('menuitemradio')[1]!);
    await waitFor(()=>expect(sw).toHaveBeenCalledWith('link-1'));
    expect(within(topBar()).queryByRole('button',{name:/^Mailbox account/})).toBeNull();
    expect(indicators(container)).toHaveLength(1);
  });
  it('one mailbox: plain address with no menu in the sidebar, compact and hidden modes', async()=>{
    stubAccounts({...linked,accounts:[]});
    const {container}=await mount();
    const head=container.querySelector('.sidebar__head') as HTMLElement;
    await waitFor(()=>expect(head.querySelector('[data-account-indicator="plain"]')).not.toBeNull());
    expect(head.querySelector('[aria-haspopup]')).toBeNull();
    expect(head.querySelector('select')).toBeNull();
    setAccountUiPreferences({folderSidebarMode:'compact'});
    const icon=await waitFor(()=>{const node=head.querySelector('[data-account-indicator="icon"]');expect(node).not.toBeNull();return node as HTMLElement;});
    expect(icon.tagName).toBe('SPAN');
    expect(icon).not.toHaveAttribute('aria-haspopup');
    expect(stripIsolates(icon.getAttribute('aria-label')!)).toBe('Signed in as me@example.org');
    fireEvent.click(icon);
    expect(screen.queryByRole('menu')).toBeNull();
    setAccountUiPreferences({folderSidebarMode:'hidden'});
    const top=await waitFor(()=>{const node=topBar().querySelector('[data-account-indicator="icon"]');expect(node).not.toBeNull();return node as HTMLElement;});
    expect(top).not.toHaveAttribute('aria-haspopup');
  });
  it('the picker marks a mailbox as primary on the server', async()=>{
    stubAccounts(linked);
    const {container}=await mount();
    fireEvent.click(await within(container.querySelector('.sidebar__head') as HTMLElement).findByRole('button',{name:/^Mailbox account/}));
    const menu=await screen.findByRole('menu');
    fireEvent.click(within(menu).getByRole('menuitem',{name:/child@example\.org.*primary/}));
    await waitFor(()=>expect(calls).toContainEqual({method:'PUT',path:'/api/accounts/primary',body:JSON.stringify({linkId:'link-1'})}));
  });
  it('the saved order and nicknames drive the picker; Rename saves through the layout endpoint', async()=>{
    stubAccounts({...linked,layout:{revision:'r1',order:['link-1','main'],nicknames:{main:'Personal','link-1':'Work'}}});
    const {container}=await mount();
    const trigger=await within(container.querySelector('.sidebar__head') as HTMLElement).findByRole('button',{name:'Mailbox account: Personal (me@example.org)'});
    expect(stripIsolates(trigger.querySelector('[data-mailbox-label]')!.textContent!)).toBe('Personal');
    fireEvent.click(trigger);
    const menu=await screen.findByRole('menu');
    expect(within(menu).getAllByRole('menuitemradio').map(item=>stripIsolates(item.querySelector('[data-mailbox-label]')!.textContent!))).toEqual(['Work','Personal']);
    fireEvent.click(within(menu).getByRole('menuitem',{name:/Rename .*me@example\.org/}));
    const input=await within(menu).findByRole('textbox',{name:/Nickname for .*me@example\.org/});
    fireEvent.input(input,{target:{value:'Home'}});
    fireEvent.keyDown(input,{key:'Enter'});
    await waitFor(()=>expect(calls.find(call=>call.path==='/api/accounts/layout')?.body).toBe(JSON.stringify({revision:'r1',order:['link-1','main'],nicknames:{main:'Home','link-1':'Work'}})));
  });
  it('sign-in opens the primary mailbox first, through the normal switch', async()=>{
    stubAccounts({...linked,primaryMailbox:'link-1'});
    await mount();
    await waitFor(()=>expect(calls.some(call=>call.method==='POST'&&call.path==='/api/accounts/link-1/switch')).toBe(true));
  });
  it('a primary mailbox that is gone falls back to the sign-in account, says so once and clears the mark', async()=>{
    stubAccounts({...linked,accounts:[],primaryFallback:true});
    const {app}=await mount();
    await waitFor(()=>expect(calls).toContainEqual({method:'PUT',path:'/api/accounts/primary',body:JSON.stringify({linkId:null})}));
    expect(app.linkedMailboxId()).toBeNull();
    expect(calls.some(call=>call.path.endsWith('/switch'))).toBe(false);
  });
  it('compact folders: an icon button in the sidebar opens the switcher', async()=>{
    stubAccounts(linked);
    const {container}=await mount();
    setAccountUiPreferences({folderSidebarMode:'compact'});
    const icon=await waitFor(()=>{const node=container.querySelector('.sidebar__head [data-account-indicator="icon"]');expect(node).not.toBeNull();return node as HTMLElement;});
    expect(stripIsolates(icon.getAttribute('aria-label')!)).toBe('Signed in as me@example.org');
    expect(indicators(container)).toHaveLength(1);
    expect(topBar().querySelector('[data-account-indicator]')).toBeNull();
    expect(icon).toHaveAttribute('aria-haspopup','menu');
    fireEvent.click(icon);
    const menu=await screen.findByRole('menu',{name:'Mailbox account'});
    expect(within(menu).getAllByRole('menuitemradio')).toHaveLength(2);
  });
  it('hidden folders: the icon button moves to the top bar', async()=>{
    stubAccounts(linked);
    const {container}=await mount();
    setAccountUiPreferences({folderSidebarMode:'hidden'});
    await waitFor(()=>expect(topBar().querySelector('[data-account-indicator="icon"]')).not.toBeNull());
    expect(container.querySelector('aside.sidebar')).toHaveAttribute('hidden');
    expect(indicators(container)).toHaveLength(1);
    expect(topBar().textContent).not.toContain('me@example.org');
  });
  it('phone: the drawer head holds the switcher and the bar shows no account for the primary mailbox', async()=>{
    narrowViewport();stubAccounts(linked);
    const {container}=await mount();
    expect(container.querySelector('.shell__bar')).not.toBeNull();
    expect(container.querySelector('[data-account-indicator="chip"]')).toBeNull();
    expect(indicators(container)).toHaveLength(1);
    expect(topBar().querySelector('[data-account-indicator]')).toBeNull();
  });
  it('phone: a chip names the linked child mailbox while it is active', async()=>{
    narrowViewport();stubAccounts(linked);
    const {app,container}=await mount();
    vi.spyOn(app,'linkedMailboxId').mockReturnValue('link-1');
    vi.spyOn(app,'me').mockReturnValue({username:'child@example.org',accountId:'acct2',storageAuthority:'child-authority'});
    // Re-render reads the spies: remount the screen with the same app.
    container.remove();
    const realtime=createRealtimeController();
    const second=render(()=><AppContext.Provider value={app}><RealtimeContext.Provider value={realtime}><MailboxScreen/></RealtimeContext.Provider></AppContext.Provider>);
    const chip=await waitFor(()=>{const node=second.container.querySelector('.shell__bar [data-account-indicator="chip"]');expect(node).not.toBeNull();return node as HTMLElement;});
    expect(stripIsolates(chip.getAttribute('aria-label')!)).toBe('Viewing linked mailbox child@example.org');
    expect(stripIsolates(chip.textContent!)).toContain('child@example.org');
  });
});
