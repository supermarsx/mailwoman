import {afterEach,describe,expect,it,vi} from 'vitest';
import {cleanup,render,screen,waitFor} from '@solidjs/testing-library';
import {MessageList,listSpansFolders} from './MessageList.tsx';
import {makeClient,mkEmail,MAILBOXES} from './appHarness.tsx';
import {createAppState} from '../state/store.ts';
import {AppContext} from '../state/context.ts';
import * as focus from './messageFocus.ts';
import type {Email,Mailbox} from '../api/jmap-types.ts';

afterEach(()=>{cleanup();vi.restoreAllMocks();focus.clearFocusRequest();});
const box=(over:Partial<Mailbox>):Mailbox=>({id:'x',name:'X',parentId:null,role:null,sortOrder:9,totalEmails:0,unreadEmails:0,...over}) as Mailbox;

describe('listSpansFolders',()=>{
 const app=(selected:string|null,mailboxes:Partial<Mailbox>[],searching=false)=>({searchActive:()=>searching,selectedMailboxId:()=>selected,mailboxes:()=>mailboxes as never});
 it('is true for search, a saved-search view, an all-mail folder and the events view',()=>{
  expect(listSpansFolders(app('inbox',[box({id:'inbox',role:'inbox'})],true))).toBe(true);
  expect(listSpansFolders(app('saved',[box({id:'saved',mailwomanSearchQuery:'is:flagged'})]))).toBe(true);
  expect(listSpansFolders(app('all',[box({id:'all',role:'all'})]))).toBe(true);
  expect(listSpansFolders(app('mwViewEvents',[]))).toBe(true);
 });
 it('is false for an ordinary folder',()=>{
  expect(listSpansFolders(app('inbox',[box({id:'inbox',role:'inbox'})]))).toBe(false);
  expect(listSpansFolders(app(null,[]))).toBe(false);
 });
});

async function mount(emails:Email[],mailboxes:Mailbox[],selected:string,searching=false){
 const app=createAppState(makeClient({emails,mailboxes:[...MAILBOXES,...mailboxes]}));
 if(searching)vi.spyOn(app,'searchActive').mockReturnValue(true);
 render(()=><AppContext.Provider value={app}><MessageList/></AppContext.Provider>);
 await app.login({username:'me@example.org',password:'p'});
 if(app.selectedMailboxId()!==selected)await app.selectMailbox(selected);
 await waitFor(()=>expect(app.messages().length).toBe(emails.length));
 return app;
}

describe('folder chip beyond search',()=>{
 it('shows the folder on rows of a saved-search view and of an all-mail folder',async()=>{
  for(const [mailbox,id] of [[box({id:'saved',name:'Starred things',mailwomanSearchQuery:'is:flagged'}),'saved'],[box({id:'allmail',name:'All Mail',role:'all'}),'allmail']] as const){
   await mount([mkEmail('m1',{subject:'Quarterly',mailboxIds:{archive:true}})],[mailbox],id);
   expect(await screen.findByRole('button',{name:/Go to .?Archive/})).toBeInTheDocument();
   cleanup();
  }
 });
 it('shows it on a conversation head too (the latest message\'s folder)',async()=>{
  await mount([
   mkEmail('t1',{subject:'Plans',threadId:'T',mailboxIds:{archive:true},receivedAt:'2026-01-01T10:00:00Z'}),
   mkEmail('t2',{subject:'Re: Plans',threadId:'T',mailboxIds:{archive:true},receivedAt:'2026-01-02T10:00:00Z'}),
  ],[],'inbox',true);
  const head=await screen.findByTestId('thread-head');
  expect(head.parentElement!.querySelector('[data-testid="row-folder"]')).not.toBeNull();
 });
 it('is not shown in an ordinary folder',async()=>{
  await mount([mkEmail('m1',{subject:'Quarterly',mailboxIds:{archive:true}})],[],'inbox');
  expect(screen.queryByTestId('row-folder')).toBeNull();
 });
});

describe('a same-thread jump opens the conversation',()=>{
 it('expands a collapsed conversation so the reply row is there to flash; a plain jump leaves it collapsed',async()=>{
  const emails=[
   mkEmail('t1',{subject:'Plans',threadId:'T',receivedAt:'2026-01-01T10:00:00Z'}),
   mkEmail('t2',{subject:'Re: Plans',threadId:'T',receivedAt:'2026-01-02T10:00:00Z'}),
  ];
  const app=await mount(emails,[],'inbox');
  const head=await screen.findByTestId('thread-head');
  expect(head.getAttribute('aria-expanded')??'false').not.toBe('true');
  expect(document.querySelector('.list__row[data-email-id="t1"]')).toBeNull();
  const stub={accountId:()=>app.accountId(),selectedMailboxId:()=>app.selectedMailboxId(),searchActive:()=>false,selectMailbox:async()=>undefined,messages:()=>app.messages(),hasMore:()=>false,loadMore:async()=>undefined,mailboxes:()=>app.mailboxes(),showToast:vi.fn()};
  await focus.focusMessageInFolder(stub,'inbox','t1');
  await new Promise(r=>setTimeout(r,20));
  expect(document.querySelector('.list__row[data-email-id="t1"]:not([data-testid="thread-head"])')).toBeNull();
  focus.clearFocusRequest();
  await focus.focusMessageInFolder(stub,'inbox','t1',{thread:true});
  await waitFor(()=>expect(document.querySelector('.list__row[data-email-id="t1"]:not([data-testid="thread-head"])')).not.toBeNull());
 });
});
