import * as workspaceAttachments from '../realtime/workspaceAttachment.ts';
import {loadSettings} from '../api/settings.ts';
import {WORKSPACE_STORAGE_KEY} from '../realtime/workspaceStorage.ts';
import {suspendAccountPreferences} from '../api/accountPreferenceLifecycle.ts';
import { beforeEach, afterEach, describe, expect, it, vi } from 'vitest';
import { render, fireEvent, screen, within, waitFor } from '@solidjs/testing-library';
import { AppContext } from '../state/context.ts';
import { createAppState } from '../state/store.ts';
import { makeClient, mkEmail } from '../components/appHarness.tsx';
import { MailboxScreen } from './Mailbox.tsx';
import { RealtimeContext } from '../realtime/context.ts';
import { createRealtimeController } from '../realtime/controller.ts';
import type { EmailGetResponse, JmapRequest } from '../api/jmap-types.ts';
import { setReadingPane } from '../components/readingPane.ts';

beforeEach(async()=>{suspendAccountPreferences();localStorage.removeItem(WORKSPACE_STORAGE_KEY);history.replaceState(null,'','#/mail');await new Promise(resolve=>setTimeout(resolve,0));});

afterEach(()=>vi.unstubAllGlobals());

function setup(attachment=false) {
  const first=mkEmail('a');
  if(attachment)Object.assign(first,{attachments:[{partId:'2',blobId:'file-a',name:'report.txt',type:'text/plain',size:4}]});
  const emails=[first,mkEmail('b')];const client=makeClient({emails});
  const original=client.jmap;
  client.jmap=vi.fn(async (request:JmapRequest)=>{const response=await original(request);for(const [method,args,id] of request.methodCalls){if(/^(AddressBook|ContactCard|ContactGroup)\//.test(method)){const index=response.methodResponses.findIndex(call=>call[2]===id);if(index>=0)response.methodResponses[index]=[method,{accountId:args['accountId'],state:'s',list:[],ids:[],notFound:[]},id];}}const ids=request.methodCalls.find(call=>call[0]==='Email/get')?.[1]['ids'];if(Array.isArray(ids)){for(const call of response.methodResponses)if(call[0]==='Email/get')(call[1] as unknown as EmailGetResponse).list=emails.filter(email=>ids.includes(email.id));}return response;});
  const app=createAppState(client);const realtime=createRealtimeController();
  const show=()=>render(()=><AppContext.Provider value={app}><RealtimeContext.Provider value={realtime}><MailboxScreen/></RealtimeContext.Provider></AppContext.Provider>);
  return {app,realtime,show,client};
}
function mount(attachment=false) {
  const {app,realtime,show}=setup(attachment);
  return {app,realtime,...show()};
}
describe('retained mailbox workspace',()=>{
  it('does not overwrite newer user intent when attachment restoration finishes late',async()=>{
    let finish!:(value:Awaited<ReturnType<typeof workspaceAttachments.resolveWorkspaceAttachment>>)=>void;
    const pending=new Promise<Awaited<ReturnType<typeof workspaceAttachments.resolveWorkspaceAttachment>>>(resolve=>{finish=resolve;});
    const resolve=vi.spyOn(workspaceAttachments,'resolveWorkspaceAttachment').mockReturnValue(pending);
    const {app,realtime}=mount();
    localStorage.setItem(WORKSPACE_STORAGE_KEY,JSON.stringify({version:2,accountId:'acct1',storageAuthority:'test-storage-authority-acct1',tabs:[{id:'mail',kind:'messages',pinned:true},{id:'attachment:acct1:file-a',kind:'attachment',reference:'file-a',messageId:'a',pinned:false}],activeId:'attachment:acct1:file-a',layout:'tabs'}));
    await loadSettings({refresh:true,fetcher:async()=>new Response(JSON.stringify({settings:{}}),{headers:{'Content-Type':'application/json'}})});
    await app.login({username:'me@example.org',password:'p'});
    await waitFor(()=>expect(resolve).toHaveBeenCalledOnce());
    fireEvent.click(screen.getByRole('button',{name:/^Compose$/}));const active=realtime.subTabs.activeId();
    finish({emailId:'a',blobId:'file-a',name:'late.txt',mime:'text/plain',size:4,subject:'old',from:'',receivedAt:'2026-01-01T00:00:00Z'});
    await waitFor(()=>expect(localStorage.getItem(WORKSPACE_STORAGE_KEY)).toContain(active!));
    expect(realtime.subTabs.activeId()).toBe(active);expect(realtime.subTabs.tabs().some(tab=>tab.kind==='attachment')).toBe(false);resolve.mockRestore();
  });
  it('restores only account-owned references and clears retained panes on authority suspension',async()=>{
    const {app,realtime,container}=mount();
    localStorage.setItem(WORKSPACE_STORAGE_KEY,JSON.stringify({version:2,accountId:'acct1',storageAuthority:'test-storage-authority-acct1',tabs:[{id:'mail',kind:'messages',pinned:true},{id:'surface:acct1:contacts',kind:'module',reference:'contacts',pinned:true}],activeId:'surface:acct1:contacts',layout:'tabs'}));
    await loadSettings({refresh:true,fetcher:async()=>new Response(JSON.stringify({settings:{}}),{headers:{'Content-Type':'application/json'}})});
    await app.login({username:'me@example.org',password:'p'});
    await screen.findByRole('tab',{name:'Contacts'});
    expect(realtime.subTabs.tabs().find(tab=>tab.title==='Contacts')?.pinned).toBe(true);
    const stored=localStorage.getItem(WORKSPACE_STORAGE_KEY)!;expect(stored).not.toContain('Subject');
    suspendAccountPreferences();
    await waitFor(()=>expect(container.querySelector('[data-surface="contacts"]')).toBeNull());
    expect(realtime.subTabs.tabs().filter(tab=>tab.id!=='mail')).toHaveLength(0);
  });
  it('honors disabled resume policy and removes the saved workspace record',async()=>{
    const {app,realtime}=mount();await app.login({username:'me@example.org',password:'p'});
    localStorage.setItem(WORKSPACE_STORAGE_KEY,JSON.stringify({version:2,accountId:'acct1',storageAuthority:'test-storage-authority-acct1',tabs:[{id:'surface:acct1:contacts',kind:'module',reference:'contacts',pinned:false}],activeId:'surface:acct1:contacts',layout:'tabs'}));
    await loadSettings({refresh:true,fetcher:async()=>new Response(JSON.stringify({settings:{'mail.resumeTabs':{value:false,source:'global',locked:true}}}),{headers:{'Content-Type':'application/json'}})});
    await waitFor(()=>expect(localStorage.getItem(WORKSPACE_STORAGE_KEY)).toBeNull());
    expect(realtime.subTabs.tabs().some(tab=>tab.title==='Contacts')).toBe(false);
  });
  it('retains module panes while routing and closes only the requested tab',async()=>{
    const {app,realtime,container}=mount();await app.login({username:'me@example.org',password:'p'});
    fireEvent.click(screen.getByRole('button',{name:'Contacts'}));
    await screen.findByRole('tab',{name:'Contacts'});
    const contacts=container.querySelector('[data-surface="contacts"]');expect(contacts).not.toBeNull();
    fireEvent.click(screen.getByRole('button',{name:'Tasks'}));
    await screen.findByRole('tab',{name:'Tasks'});
    expect(container.querySelector('[data-surface="contacts"]')).toBe(contacts);
    fireEvent.click(screen.getByRole('tab',{name:'Contacts'}));
    expect(screen.getByRole('tab',{name:'Contacts'})).toHaveAttribute('aria-selected','true');
    expect(screen.queryByRole('button',{name:'Close Tasks'})).toBeNull();
    fireEvent.contextMenu(screen.getByRole('tab',{name:'Tasks'}));
    fireEvent.click(await screen.findByRole('menuitem',{name:'Close Tasks'}));
    await waitFor(()=>expect(realtime.subTabs.tabs().some(tab=>tab.title==='Tasks')).toBe(false));
    expect(container.querySelector('[data-surface="contacts"]')).toBe(contacts);
  });
  it('opens the selected message export editor in Settings and retains its reader',async()=>{
    const {app}=mount();await app.login({username:'me@example.org',password:'p'});
    fireEvent.dblClick(await screen.findByText('Subject a',{selector:'.list__subject'}));
    fireEvent.click(await screen.findByTestId('reader-export'));
    expect(await screen.findByRole('heading',{name:'Export message'})).toBeInTheDocument();
    expect(screen.getByRole('tab',{name:'Subject a'})).toBeInTheDocument();
    expect(screen.getByRole('tab',{name:'Settings'})).toHaveAttribute('aria-selected','true');
  });
  it('opens attachments in their own reusable tab without replacing the message reader',async()=>{
    const policy={previewAllowed:true,downloadAllowed:true,maxBytes:1024,allowedExtensions:[],blockedExtensions:[],warnExtensions:[],allowedMimeTypes:[],blockedMimeTypes:[],warnMimeTypes:[]};
    vi.stubGlobal('fetch',vi.fn(async(input:RequestInfo|URL)=>new URL(input instanceof Request?input.url:String(input),location.href).pathname==='/api/attachments/policy'?new Response(JSON.stringify(policy),{status:200,headers:{'content-type':'application/json'}}):new Response(null,{status:404})));
    const {app,realtime}=mount(true);await app.login({username:'me@example.org',password:'p'});
    fireEvent.dblClick(await screen.findByText('Subject a',{selector:'.list__subject'}));
    const item=await screen.findByRole('option',{name:/report.txt/});fireEvent.click(item);
    expect(screen.getByRole('tab',{name:'report.txt'})).toHaveAttribute('aria-selected','true');
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole('tab',{name:'Subject a'}));
    fireEvent.click(await screen.findByRole('option',{name:/report.txt/}));
    expect(realtime.subTabs.tabs().filter(tab=>tab.kind==='attachment')).toHaveLength(1);
    expect(screen.getByRole('tab',{name:'Subject a'})).toBeInTheDocument();
  });
  it('opens independent message tabs and restores each reader without another load',async()=>{
    const {app,realtime,container}=mount();await app.login({username:'me@example.org',password:'p'});
    fireEvent.dblClick(await screen.findByText('Subject a',{selector:'.list__subject'}));
    await waitFor(()=>expect(app.openEmail()?.id).toBe('a'));
    fireEvent.dblClick(screen.getByText('Subject b',{selector:'.list__subject'}));
    await waitFor(()=>expect(app.openEmail()?.id).toBe('b'));
    expect(container.querySelectorAll('.workspace-pane .reader__subject')).toHaveLength(2);
    const first=realtime.subTabs.tabs().find(tab=>tab.title==='Subject a')!;
    fireEvent.click(document.getElementById(`subtab-tab-${first.id}`)!);
    expect(app.openEmail()?.id).toBe('a');
    expect(screen.getByRole('tab',{name:'Subject a'})).toHaveAttribute('aria-selected','true');
  });
  // t74: the reading pane on (default Right): a click shows the message in the
  // one reading pane and adds no tab; a double-click opens its tab, which takes
  // the pane's view (no second Email/get). Pane Off: a click opens the tab.
  it('reading pane on: clicks reuse the reading pane and add no tab; a double-click opens one tab without a second load',async()=>{
    const {app,realtime,show,client}=setup();const {container}=show();await app.login({username:'me@example.org',password:'p'});
    const gets=(id:string)=>vi.mocked(client.jmap).mock.calls.filter(([request])=>request.methodCalls.some(call=>{const ids=call[1]['ids'];return call[0]==='Email/get'&&Array.isArray(ids)&&ids.length===1&&ids[0]===id;})).length;
    const messageTabs=()=>realtime.subTabs.tabs().filter(tab=>tab.kind==='messages'&&tab.id!=='mail');
    const paneSubject=()=>screen.getByTestId('reading-pane').querySelector('.reader__subject')?.textContent;
    fireEvent.click(await screen.findByText('Subject a',{selector:'.list__subject'}),{detail:1});
    await waitFor(()=>expect(paneSubject()).toBe('Subject a'));
    expect(realtime.subTabs.activeId()).toBe('mail');expect(messageTabs()).toHaveLength(0);
    fireEvent.click(screen.getByText('Subject b',{selector:'.list__subject'}),{detail:1});
    await waitFor(()=>expect(paneSubject()).toBe('Subject b'));
    expect(messageTabs()).toHaveLength(0);expect(container.querySelectorAll('[data-testid="reading-pane"]')).toHaveLength(1);
    const loads=gets('b');expect(loads).toBeGreaterThan(0);
    const row=screen.getByText('Subject b',{selector:'.list__subject'});
    fireEvent.click(row,{detail:2});fireEvent.dblClick(row,{detail:2});
    await waitFor(()=>expect(realtime.subTabs.activeId()).toBe('message:acct1:b'));
    expect(messageTabs()).toHaveLength(1);expect(gets('b')).toBe(loads);
    expect(within(screen.getByRole('tabpanel',{name:'Subject b'})).getByRole('heading',{name:'Subject b'})).toBeInTheDocument();
    expect(app.openEmail()?.id).toBe('b');
    // Again: the tab is reused, not duplicated.
    fireEvent.dblClick(screen.getByText('Subject b',{selector:'.list__subject'}));
    expect(messageTabs()).toHaveLength(1);
    // A click from beside the tab goes back to the reading pane on the Mail tab.
    fireEvent.click(screen.getByText('Subject a',{selector:'.list__subject'}),{detail:1});
    await waitFor(()=>expect(realtime.subTabs.activeId()).toBe('mail'));
    await waitFor(()=>expect(paneSubject()).toBe('Subject a'));
    expect(messageTabs()).toHaveLength(1);
  });
  it('reading pane off: a click opens the message tab and a second click on it reuses that tab',async()=>{
    try{
      const {app,realtime}=mount();await app.login({username:'me@example.org',password:'p'});setReadingPane('off');
      fireEvent.click(await screen.findByText('Subject a',{selector:'.list__subject'}),{detail:1});
      await waitFor(()=>expect(realtime.subTabs.activeId()).toBe('message:acct1:a'));
      expect(screen.getByTestId('reading-pane')).not.toBeVisible();
      fireEvent.click(screen.getByRole('tab',{name:'Mail'}));
      fireEvent.click(screen.getByText('Subject a',{selector:'.list__subject'}),{detail:1});
      await waitFor(()=>expect(realtime.subTabs.activeId()).toBe('message:acct1:a'));
      expect(realtime.subTabs.tabs().filter(tab=>tab.kind==='messages'&&tab.id!=='mail')).toHaveLength(1);
    }finally{setReadingPane('right');}
  });
  it('keeps each composer mounted and opens sectioned Settings without a modal',async()=>{
    const {app,realtime}=mount();await app.login({username:'me@example.org',password:'p'});
    fireEvent.click(screen.getByRole('button',{name:/^Compose$/}));
    const first=realtime.subTabs.activeId()!;fireEvent.input(within(screen.getByRole('tabpanel',{name:'Compose'})).getByLabelText('Subject'),{target:{value:'Draft alpha'}});
    fireEvent.click(screen.getByRole('button',{name:/^Compose$/}));
    fireEvent.input(within(screen.getByRole('tabpanel',{name:'Compose'})).getByLabelText('Subject'),{target:{value:'Draft beta'}});
    fireEvent.click(document.getElementById(`subtab-tab-${first}`)!);
    expect(within(screen.getByRole('tabpanel',{name:'Draft alpha'})).getByLabelText('Subject')).toHaveValue('Draft alpha');
    fireEvent.click(screen.getByRole('button',{name:/^Settings$/}));
    const settings=await screen.findByRole('region',{name:'Settings'});
    expect(within(settings).getByRole('navigation',{name:'Settings sections'})).toBeInTheDocument();
    expect(screen.queryByRole('dialog',{name:'Settings'})).not.toBeInTheDocument();
    fireEvent.click(document.getElementById(`subtab-tab-${first}`)!);
    expect(within(screen.getByRole('tabpanel',{name:'Draft alpha'})).getByLabelText('Subject')).toHaveValue('Draft alpha');
  });
  it('a bulk close lists the composers that would lose their message in one dialog: keep, then discard (t50)',async()=>{
    // No storage authority: the composer cannot keep an account draft, so closing it loses the message.
    const {app,realtime,show,client}=setup();const unstored=async()=>({username:'me@example.org',accountId:'acct1'});
    client.login=vi.fn(unstored) as typeof client.login;client.me=vi.fn(unstored) as typeof client.me;show();
    await app.login({username:'me@example.org',password:'p'});
    fireEvent.click(screen.getByRole('button',{name:'Contacts'}));await screen.findByRole('tab',{name:'Contacts'});
    fireEvent.click(screen.getByRole('button',{name:/^Compose$/}));
    fireEvent.input(within(screen.getByRole('tabpanel',{name:'Compose'})).getByLabelText('Subject'),{target:{value:'Unstored draft'}});
    const composer=realtime.subTabs.activeId()!;
    fireEvent.click(screen.getByRole('button',{name:'Tasks'}));await screen.findByRole('tab',{name:'Tasks'});
    fireEvent.contextMenu(screen.getByRole('tab',{name:'Mail'}));
    expect(screen.getByRole('menuitem',{name:'Close tabs to the left'})).toHaveAttribute('aria-disabled','true');
    fireEvent.click(screen.getByRole('menuitem',{name:'Close all tabs'}));
    const dialog=await screen.findByTestId('workspace-tabs-close-dirty');
    expect(within(dialog).getAllByRole('listitem').map(item=>item.textContent)).toEqual(['Unstored draft']);
    expect(screen.queryByTestId('compose-close-confirm')).not.toBeInTheDocument();
    fireEvent.click(within(dialog).getByRole('button',{name:'Keep them open'}));
    await waitFor(()=>expect(realtime.subTabs.tabs().map(tab=>tab.id)).toEqual(['mail',composer]));
    fireEvent.contextMenu(screen.getByRole('tab',{name:'Unstored draft'}));fireEvent.click(screen.getByRole('menuitem',{name:'Close all tabs'}));
    fireEvent.click(within(await screen.findByTestId('workspace-tabs-close-dirty')).getByRole('button',{name:'Discard and close'}));
    await waitFor(()=>expect(realtime.subTabs.tabs().map(tab=>tab.id)).toEqual(['mail']));
    expect(screen.queryByTestId('compose-close-confirm')).not.toBeInTheDocument();
  });
  it('restores a message tab when the mailbox mounts signed in (a reload): Mail stays first and the reader loads',async()=>{
    const {app,realtime,show}=setup();
    localStorage.setItem(WORKSPACE_STORAGE_KEY,JSON.stringify({version:2,accountId:'acct1',storageAuthority:'test-storage-authority-acct1',tabs:[{id:'mail',kind:'messages',pinned:true},{id:'message:acct1:a',kind:'messages',reference:'a',pinned:false}],activeId:'message:acct1:a',layout:'tabs'}));
    await loadSettings({refresh:true,fetcher:async()=>new Response(JSON.stringify({settings:{}}),{headers:{'Content-Type':'application/json'}})});
    await app.login({username:'me@example.org',password:'p'});
    const {container}=show();
    await waitFor(()=>expect(realtime.subTabs.tabs().some(tab=>tab.id==='message:acct1:a')).toBe(true));
    expect(realtime.subTabs.tabs()[0]?.id).toBe('mail');
    await waitFor(()=>expect(container.querySelector('.workspace-pane .reader__subject')?.textContent).toBe('Subject a'));
  });
});
