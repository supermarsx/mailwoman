import {afterEach,describe,expect,it,vi} from 'vitest';
import {cleanup,fireEvent,render,screen,waitFor,within} from '@solidjs/testing-library';
import {MessageList} from './MessageList.tsx';
import {makeClient,mkEmail,MAILBOXES} from './appHarness.tsx';
import {createAppState} from '../state/store.ts';
import {AppContext} from '../state/context.ts';
import * as relations from './messageRelations.ts';
import * as focus from './messageFocus.ts';
import * as download from './messageAttachmentDownload.ts';
import * as bulk from './messageAttachmentsBulk.ts';
import type {Email} from '../api/jmap-types.ts';

// Menu entries added after the single-message menu: attachments of a selection,
// a conversation head acting on its messages, and the hand-set answered /
// forwarded marks.

afterEach(()=>{cleanup();vi.restoreAllMocks();});
const part=(name:string,size=10)=>({blobId:`b-${name}`,type:'application/pdf',name,size,partId:name,cid:null,disposition:'attachment'}) as never;
const SENT={id:'sent',name:'Sent',parentId:null,role:'sent',sortOrder:3,totalEmails:0,unreadEmails:0};
async function mount(emails:Email[]){
 const app=createAppState(makeClient({emails,mailboxes:[...MAILBOXES,SENT]}));
 render(()=><AppContext.Provider value={app}><MessageList/></AppContext.Provider>);
 await app.login({username:'me@example.org',password:'p'});
 await waitFor(()=>expect(app.messages().length).toBe(emails.length));
 return app;
}
const rowOf=(subject:string)=>screen.getByText(subject).closest('button')!;

describe('message menu: attachments of the selection',()=>{
 it('offers one ZIP for the selected messages that have attachments, and passes exactly those messages',async()=>{
  const zip=vi.spyOn(bulk,'downloadAttachmentsOf').mockResolvedValue();
  const app=await mount([mkEmail('m1',{subject:'First',hasAttachment:true}),mkEmail('m2',{subject:'Second',hasAttachment:true}),mkEmail('m3',{subject:'Third'})]);
  app.setSelection(['m1','m2','m3']);
  fireEvent.contextMenu(rowOf('First'));
  fireEvent.click(await screen.findByRole('menuitem',{name:'Download attachments of 2 selected messages as ZIP'}));
  expect(zip).toHaveBeenCalledTimes(1);
  expect(zip.mock.calls[0]![2].map(message=>message.id)).toEqual(['m1','m2']);
 });
 it('is absent for a single, unselected message',async()=>{
  const app=await mount([mkEmail('m1',{subject:'First',hasAttachment:true}),mkEmail('m2',{subject:'Second',hasAttachment:true})]);
  app.setSelection(['m2']);
  fireEvent.contextMenu(rowOf('First'));
  await screen.findAllByRole('menuitem');
  expect(screen.queryByRole('menuitem',{name:/selected message/})).toBeNull();
 });
});

describe('message menu: answered and forwarded marks',()=>{
 it('marks a message answered',async()=>{
  const app=await mount([mkEmail('m1',{subject:'First'}),mkEmail('m2',{subject:'Second',keywords:{$seen:true,$answered:true}})]);
  const mark=vi.spyOn(app,'markReplyState').mockResolvedValue();
  fireEvent.contextMenu(rowOf('First'));
  fireEvent.click(await screen.findByRole('menuitem',{name:'Mark as'}));
  fireEvent.click(await screen.findByRole('menuitem',{name:'Mark as answered'}));
  expect(mark).toHaveBeenCalledWith(['m1'],'$answered',true);
 });
 it('offers Mark as not forwarded on a forwarded message and clears the keyword',async()=>{
  const app=await mount([mkEmail('m1',{subject:'First',keywords:{$seen:true,$forwarded:true}})]);
  const mark=vi.spyOn(app,'markReplyState').mockResolvedValue();
  fireEvent.contextMenu(rowOf('First'));
  fireEvent.click(await screen.findByRole('menuitem',{name:'Mark as'}));
  fireEvent.click(await screen.findByRole('menuitem',{name:'Mark as not forwarded'}));
  expect(mark).toHaveBeenCalledWith(['m1'],'$forwarded',false);
 });
 it('with a selection the mark applies to every selected message; mixed state sets it on',async()=>{
  const app=await mount([mkEmail('m1',{subject:'First',keywords:{$seen:true,$answered:true}}),mkEmail('m2',{subject:'Second'})]);
  const mark=vi.spyOn(app,'markReplyState').mockResolvedValue();
  app.setSelection(['m1','m2']);
  fireEvent.contextMenu(rowOf('First'));
  fireEvent.click(await screen.findByRole('menuitem',{name:'Mark as'}));
  fireEvent.click(await screen.findByRole('menuitem',{name:'Mark as answered'}));
  expect(mark).toHaveBeenCalledWith(expect.arrayContaining(['m1','m2']),'$answered',true);
 });
});

describe('message menu: a conversation head acts on the conversation',()=>{
 const members=():Email[]=>[
  mkEmail('t1',{subject:'Plans',threadId:'T',hasAttachment:true,keywords:{$seen:true,$answered:true},receivedAt:'2026-01-01T10:00:00Z',sentAt:'2026-01-01T10:00:00Z'}),
  mkEmail('t2',{subject:'Re: Plans',threadId:'T',hasAttachment:true,keywords:{$seen:true,$forwarded:true},receivedAt:'2026-01-02T10:00:00Z',sentAt:'2026-01-02T10:00:00Z'}),
 ];
 const reply=(id:string)=>mkEmail(id,{mailboxIds:{sent:true},threadId:'T',to:[{name:'Ann',email:'ann@example.org'}],sentAt:'2026-02-01T10:00:00Z'});
 async function openHead(){
  vi.spyOn(relations,'fetchMessageExtras').mockImplementation(async(_client,_account,email)=>email.id==='t1'
   ?{attachments:[part('a.pdf')],replies:[reply('r1')],forwards:[]}
   :{attachments:[part('b.pdf'),part('c.pdf')],replies:[],forwards:[reply('f1')]});
  const app=await mount(members());
  const head=await screen.findByTestId('thread-head');
  fireEvent.contextMenu(head);
  return {app,head};
 }
 it('groups attachments by message, each with its files and its own ZIP, plus one ZIP for the conversation',async()=>{
  const zip=vi.spyOn(download,'downloadZip').mockResolvedValue();
  await openHead();
  fireEvent.click(await screen.findByRole('menuitem',{name:'Download attachments'}));
  const groups=await screen.findAllByRole('menuitem',{name:/\(\d\)/});
  expect(groups).toHaveLength(2);
  expect(screen.getByRole('menuitem',{name:'Download all 3 attachments of the conversation as ZIP'})).toBeInTheDocument();
  fireEvent.click(screen.getByRole('menuitem',{name:'Download all 3 attachments of the conversation as ZIP'}));
  expect(zip.mock.calls[0]![1].map(group=>group.emailId)).toEqual(['t1','t2']);
 });
 it('Go to reply and Go to forward jump to the target of the message that was answered or forwarded, and open the conversation',async()=>{
  const go=vi.spyOn(focus,'focusMessageInFolder').mockResolvedValue(true);
  await openHead();
  await waitFor(()=>expect(screen.getByRole('menuitem',{name:'Go to reply'})).not.toHaveAttribute('aria-disabled','true'));
  fireEvent.click(screen.getByRole('menuitem',{name:'Go to reply'}));
  expect(go).toHaveBeenCalledWith(expect.anything(),'sent','r1',{open:true,thread:true});
 });
 it('with several targets the submenu names the message each one answers',async()=>{
  vi.spyOn(relations,'fetchMessageExtras').mockImplementation(async(_client,_account,email)=>email.id==='t1'
   ?{attachments:[],replies:[reply('r1')],forwards:[]}:{attachments:[],replies:[reply('r2')],forwards:[]});
  const emails=members().map(email=>({...email,keywords:{$seen:true,$answered:true}}));
  const app=createAppState(makeClient({emails,mailboxes:[...MAILBOXES,SENT]}));
  render(()=><AppContext.Provider value={app}><MessageList/></AppContext.Provider>);
  await app.login({username:'me@example.org',password:'p'});
  fireEvent.contextMenu(await screen.findByTestId('thread-head'));
  await waitFor(()=>expect(screen.getByRole('menuitem',{name:'Go to reply'})).not.toHaveAttribute('aria-disabled','true'));
  fireEvent.click(screen.getByRole('menuitem',{name:'Go to reply'}));
  const items=await screen.findAllByRole('menuitem',{name:/→/});
  expect(items).toHaveLength(2);
  expect(within(items[0]!.parentElement!).getAllByRole('menuitem').length).toBeGreaterThanOrEqual(2);
 });
});
