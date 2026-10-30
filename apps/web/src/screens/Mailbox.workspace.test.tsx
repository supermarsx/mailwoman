import {createSignal} from 'solid-js';
import {setAdminLeetspeak} from '../i18n/presentation.ts';
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
import {accountUiPreferences,setAccountUiPreferences} from '../api/accountPreferences.ts';

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
  it('attempts a separate composer window when configured and retains its tab if popups are blocked',async()=>{
    const {app,show}=setup();await app.login({username:'me@example.org',password:'p'});
    await loadSettings({refresh:true,fetcher:async()=>new Response(JSON.stringify({settings:{'mail.composeWindow':{value:'window',source:'user',locked:false}}}),{headers:{'content-type':'application/json'}})});
    const popup=vi.spyOn(window,'open').mockReturnValue(null);
    try{
      show();fireEvent.click(await screen.findByRole('button',{name:/^Compose$/}));
      await waitFor(()=>expect(popup).toHaveBeenCalledTimes(1));
      expect(popup.mock.calls[0]?.[0]).toContain('/workspace-popup.html');
      expect(await screen.findByRole('tab',{name:/^Compose$/})).toBeInTheDocument();
    }finally{popup.mockRestore();}
  });

  it.each([false,true])('keeps one folder region and placeholders for visible rows while custom folders collapsed=%s',async collapsed=>{
    const {app,show}=setup();await app.login({username:'me@example.org',password:'p'});
    const custom=Array.from({length:15},(_,index)=>({id:`custom-${index}`,name:`Project ${index}`,parentId:null,role:null,sortOrder:10+index,totalEmails:0,unreadEmails:0}));
    const initial=[...app.mailboxes(),...custom];
    const [boxes,setBoxes]=createSignal(initial);
    vi.spyOn(app,'mailboxes').mockImplementation(boxes);
    setAccountUiPreferences({folderHidden:['custom-0'],userFoldersCollapsed:collapsed});
    const {container}=show();
    await screen.findByRole('button',{name:'User folders'});
    const region=container.querySelector('[data-folder-region]')!;
    const nav=region.parentElement!;
    const children=nav.children.length;
    const expected=initial.filter(box=>box.role).length+1+(collapsed?0:14);
    expect(region.querySelectorAll('button')).toHaveLength(expected);
    expect(region.querySelector('#sidebar-user-folders')).toHaveProperty('hidden',collapsed);
    setBoxes([]);
    await waitFor(()=>expect(region.querySelectorAll('[data-folder-skeleton]')).toHaveLength(expected));
    expect(container.querySelector('[data-folder-region]')).toBe(region);
    expect(nav.children).toHaveLength(children);
    expect(region.querySelectorAll('.sidebar__folder-rows:not([hidden])')).toHaveLength(0);
    setBoxes(initial);
    await waitFor(()=>expect(region.querySelector('[data-folder-skeleton]')).toBeNull());
    expect(region.querySelectorAll('button')).toHaveLength(expected);
    expect(nav.children).toHaveLength(children);
  });

  it('displays folder and workspace labels in leetspeak without persisting transformed titles',async()=>{
    const {app,realtime,show}=setup();await app.login({username:'me@example.org',password:'p'});
    setAdminLeetspeak(true);
    try{
      show();
      const inbox=await screen.findByRole('button',{name:'1n80x'});
      fireEvent.click(inbox);
      expect(app.selectedMailboxId()).toBe('inbox');
      expect(app.mailboxes().find(box=>box.id==='inbox')?.name).toBe('Inbox');
      expect(realtime.subTabs.tabs().find(tab=>tab.id==='mail')?.title).toBe('Mail');
      setAdminLeetspeak(false);
      expect(screen.getByRole('tab',{name:'Mail'})).toBeInTheDocument();
      expect(screen.getByRole('button',{name:'Inbox'})).toBeInTheDocument();
    }finally{setAdminLeetspeak(null);}
  });

  it('collapses user folders independently of system folders and hiding the selected folder returns to Inbox',async()=>{
    const {app,show}=setup();await app.login({username:'me@example.org',password:'p'});
    const boxes=[...app.mailboxes(),{id:'projects',name:'Projects',parentId:null,role:null,sortOrder:5,totalEmails:0,unreadEmails:0}];
    vi.spyOn(app,'mailboxes').mockReturnValue(boxes);
    show();
    const disclosure=await screen.findByRole('button',{name:'User folders'});
    expect(disclosure).toHaveAttribute('aria-expanded','true');
    fireEvent.click(disclosure);
    expect(accountUiPreferences().userFoldersCollapsed).toBe(true);
    expect(screen.queryByRole('button',{name:'Projects'})).toBeNull();
    expect(screen.getByRole('button',{name:'Inbox'})).toBeVisible();
    fireEvent.click(disclosure);
    const projects=screen.getByRole('button',{name:'Projects'});
    fireEvent.click(projects);await waitFor(()=>expect(app.selectedMailboxId()).toBe('projects'));
    fireEvent.keyDown(projects,{key:'F10',shiftKey:true});
    fireEvent.click(await screen.findByRole('menuitem',{name:'Hide from sidebar'}));
    await waitFor(()=>expect(app.selectedMailboxId()).toBe('inbox'));
    expect(screen.queryByRole('button',{name:'Projects'})).toBeNull();
    expect(app.mailboxes().some(box=>box.id==='projects')).toBe(true);
    setAccountUiPreferences({folderHidden:[]});
    expect(screen.getByRole('button',{name:'Projects'})).toBeVisible();
  });
  it('shows the message-list toggle only for the active specific message tab',async()=>{
    const {app}=mount();await app.login({username:'me@example.org',password:'p'});
    const toggle=()=>screen.queryByRole('button',{name:'Show or hide message list'});
    expect(toggle()).not.toBeInTheDocument();
    fireEvent.dblClick(await screen.findByText('Subject a',{selector:'.list__subject'}));
    await waitFor(()=>expect(toggle()).toBeInTheDocument());
    fireEvent.click(screen.getByRole('tab',{name:'Mail'}));
    expect(toggle()).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole('tab',{name:'Subject a'}));
    expect(toggle()).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button',{name:'Compose'}));
    expect(toggle()).not.toBeInTheDocument();
  });
  it('keeps an explicit module deep link and its parameter over the saved active mail tab',async()=>{
    history.replaceState(null,'','#/contacts/person-7');
    const {app,realtime}=mount();
    localStorage.setItem(WORKSPACE_STORAGE_KEY,JSON.stringify({version:2,accountId:'acct1',storageAuthority:'test-storage-authority-acct1',tabs:[{id:'mail',kind:'messages',pinned:true},{id:'settings:acct1',kind:'settings',pinned:false}],activeId:'settings:acct1',layout:'tabs'}));
    await loadSettings({refresh:true,fetcher:async()=>new Response(JSON.stringify({settings:{}}),{headers:{'Content-Type':'application/json'}})});
    await app.login({username:'me@example.org',password:'p'});
    await waitFor(()=>expect(screen.getByRole('tab',{name:'Contacts'})).toHaveAttribute('aria-selected','true'));
    expect(location.hash).toBe('#/contacts/person-7');
    expect(realtime.subTabs.tabs().some(tab=>tab.id==='settings:acct1')).toBe(true);
  });
  it.each([true,false])('opens an initial module link with no saved tabs when resume is %s',async(resume)=>{
    history.replaceState(null,'','#/contacts/person-7');
    const {app}=mount();
    await loadSettings({refresh:true,fetcher:async()=>new Response(JSON.stringify({settings:{'mail.resumeTabs':{value:resume,source:'global',locked:true}}}),{headers:{'Content-Type':'application/json'}})});
    await app.login({username:'me@example.org',password:'p'});
    await waitFor(()=>expect(screen.getByRole('tab',{name:'Contacts'})).toHaveAttribute('aria-selected','true'));
    expect(location.hash).toBe('#/contacts/person-7');
  });
  it.each([true,false])('keeps explicit Mail over a saved Contacts tab when resume is %s',async(resume)=>{
    const {app,realtime}=mount();
    localStorage.setItem(WORKSPACE_STORAGE_KEY,JSON.stringify({version:2,accountId:'acct1',storageAuthority:'test-storage-authority-acct1',tabs:[{id:'mail',kind:'messages',pinned:true},{id:'surface:acct1:contacts',kind:'module',reference:'contacts',pinned:false}],activeId:'surface:acct1:contacts',layout:'tabs'}));
    await loadSettings({refresh:true,fetcher:async()=>new Response(JSON.stringify({settings:{'mail.resumeTabs':{value:resume,source:'global',locked:true}}}),{headers:{'Content-Type':'application/json'}})});
    await app.login({username:'me@example.org',password:'p'});
    await waitFor(()=>expect(realtime.subTabs.activeId()).toBe('mail'));
    await new Promise(resolve=>setTimeout(resolve,0));
    expect(realtime.subTabs.activeId()).toBe('mail');expect(location.hash).toBe('#/mail');
    expect(realtime.subTabs.tabs().some(tab=>tab.title==='Contacts')).toBe(resume);
  });
  it('restores the saved active Contacts tab when there is no explicit initial route',async()=>{
    history.replaceState(null,'',location.pathname);
    const {app,realtime}=mount();
    localStorage.setItem(WORKSPACE_STORAGE_KEY,JSON.stringify({version:2,accountId:'acct1',storageAuthority:'test-storage-authority-acct1',tabs:[{id:'mail',kind:'messages',pinned:true},{id:'surface:acct1:contacts',kind:'module',reference:'contacts',pinned:false}],activeId:'surface:acct1:contacts',layout:'tabs'}));
    await loadSettings({refresh:true,fetcher:async()=>new Response(JSON.stringify({settings:{}}),{headers:{'Content-Type':'application/json'}})});
    await app.login({username:'me@example.org',password:'p'});
    await waitFor(()=>expect(realtime.subTabs.activeId()).toBe('surface:acct1:contacts'));
  });
  it.each([true,false])('preserves navigation before settings finish loading when resume is %s',async(resume)=>{
    history.replaceState(null,'','#/contacts/person-7');
    const {app,realtime,show}=setup();await app.login({username:'me@example.org',password:'p'});
    let resolve!:(response:Response)=>void;
    const loading=loadSettings({refresh:true,fetcher:()=>new Promise<Response>(done=>{resolve=done;})});
    show();
    fireEvent.click(screen.getByRole('button',{name:'Tasks'}));
    await waitFor(()=>expect(realtime.subTabs.activeId()).toBe('surface:acct1:tasks'));
    resolve(new Response(JSON.stringify({settings:{'mail.resumeTabs':{value:resume,source:'global',locked:true}}}),{headers:{'Content-Type':'application/json'}}));
    await loading;await new Promise(done=>setTimeout(done,0));
    expect(realtime.subTabs.activeId()).toBe('surface:acct1:tasks');expect(location.hash).toBe('#/tasks');
  });
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
    // The pending policy strip is deliberately inert; wait for the real controls.
    await waitFor(()=>expect(document.querySelector('[data-policy="ready"]')).toBeInTheDocument());
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
    // A message open in its tab: a single click from the Mail tab goes to that tab, with no second load.
    fireEvent.click(screen.getByRole('tab',{name:'Mail'}));
    fireEvent.click(screen.getByText('Subject b',{selector:'.list__subject'}),{detail:1});
    await waitFor(()=>expect(realtime.subTabs.activeId()).toBe('message:acct1:b'));
    expect(gets('b')).toBe(loads);expect(paneSubject()).toBeUndefined();
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
