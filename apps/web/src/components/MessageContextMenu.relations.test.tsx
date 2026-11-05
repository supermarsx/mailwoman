import {afterEach,describe,expect,it,vi} from 'vitest';
import {cleanup,fireEvent,render,screen,waitFor} from '@solidjs/testing-library';
import {MessageList} from './MessageList.tsx';
import {makeClient,mkEmail,MAILBOXES} from './appHarness.tsx';
import {createAppState} from '../state/store.ts';
import {AppContext} from '../state/context.ts';
import * as relations from './messageRelations.ts';
import * as focus from './messageFocus.ts';
import * as download from './messageAttachmentDownload.ts';
import {classifyRelated} from './messageRelations.ts';
import {buildZip,crc32} from '../viewers/zipStore.ts';
import type {Email} from '../api/jmap-types.ts';

afterEach(()=>{cleanup();vi.restoreAllMocks();});
const part=(name:string,size=10)=>({blobId:`b-${name}`,type:'application/pdf',name,size,partId:name,cid:null,disposition:'attachment'}) as never;
async function open(email:Partial<Email>,extras:relations.MessageExtras|Promise<relations.MessageExtras>){
 vi.spyOn(relations,'fetchMessageExtras').mockImplementation(async()=>extras);
 const app=createAppState(makeClient({emails:[mkEmail('m1',{subject:'Quarterly',...email})],mailboxes:[...MAILBOXES,{id:'sent',name:'Sent',parentId:null,role:'sent',sortOrder:3,totalEmails:0,unreadEmails:0}]}));
 render(()=><AppContext.Provider value={app}><MessageList/></AppContext.Provider>);
 await app.login({username:'me@example.org',password:'p'});await waitFor(()=>expect(app.messages()).toHaveLength(1));
 fireEvent.contextMenu(screen.getByText('Quarterly').closest('button')!);
 return app;
}
const reply=(id:string,mailbox='sent')=>mkEmail(id,{mailboxIds:{[mailbox]:true},to:[{name:'Ann',email:'ann@example.org'}],sentAt:'2026-02-01T10:00:00Z'});
const answered={$seen:true,$answered:true};
const forwarded={$seen:true,$forwarded:true};
describe('message menu: attachments, go to reply, go to forward',()=>{
 it('hides all three entries for a plain message',async()=>{
  await open({},{attachments:[],replies:[],forwards:[]});
  const labels=screen.getAllByRole('menuitem').map(i=>i.textContent);
  expect(labels.some(l=>/Download attachment|Go to/.test(l??''))).toBe(false);
 });
 it('one attachment shows Download <name> and uses the shared helper',async()=>{
  const spy=vi.spyOn(download,'downloadOne').mockResolvedValue();
  await open({hasAttachment:true},{attachments:[part('a.pdf')],replies:[],forwards:[]});
  fireEvent.click(await screen.findByRole('menuitem',{name:'Download a.pdf'}));
  expect(spy).toHaveBeenCalledWith(expect.anything(),'m1',expect.objectContaining({name:'a.pdf'}));
 });
 it('shows a disabled loading entry until the attachments arrive',async()=>{
  await open({hasAttachment:true},new Promise(()=>undefined));
  expect(screen.getByRole('menuitem',{name:'Download attachments'})).toHaveAttribute('aria-disabled','true');
 });
 it('several attachments open a submenu with Download all as ZIP',async()=>{
  const zip=vi.spyOn(download,'downloadZip').mockResolvedValue();
  await open({hasAttachment:true},{attachments:[part('a.pdf'),part('b.pdf')],replies:[],forwards:[]});
  fireEvent.click(await screen.findByRole('menuitem',{name:'Download attachments'}));
  expect(await screen.findByRole('menuitem',{name:/a\.pdf/})).toBeInTheDocument();
  fireEvent.click(screen.getByRole('menuitem',{name:'Download all as ZIP'}));
  expect(zip).toHaveBeenCalled();
 });
 it('Go to reply is disabled with a tooltip when none is found, and Go to forward is hidden when not forwarded',async()=>{
  await open({keywords:answered},{attachments:[],replies:[],forwards:[]});
  const item=await screen.findByRole('menuitem',{name:'Go to reply'});
  await waitFor(()=>expect(item).toHaveAttribute('title','Reply not found'));
  expect(item).toHaveAttribute('aria-disabled','true');
  expect(screen.queryByRole('menuitem',{name:'Go to forward'})).toBeNull();
 });
 it('Go to reply navigates to the reply in its folder',async()=>{
  const go=vi.spyOn(focus,'focusMessageInFolder').mockResolvedValue(true);
  await open({keywords:answered},{attachments:[],replies:[reply('r1')],forwards:[]});
  await waitFor(()=>expect(screen.getByRole('menuitem',{name:'Go to reply'})).not.toHaveAttribute('aria-disabled','true'));
  fireEvent.click(screen.getByRole('menuitem',{name:'Go to reply'}));
  expect(go).toHaveBeenCalledWith(expect.anything(),'sent','r1',{open:true});
 });
 it('several forwards give a submenu with date and recipients',async()=>{
  await open({keywords:forwarded},{attachments:[],replies:[],forwards:[reply('f1'),reply('f2')]});
  await waitFor(()=>expect(screen.getByRole('menuitem',{name:'Go to forward'})).not.toHaveAttribute('aria-disabled','true'));
  fireEvent.click(screen.getByRole('menuitem',{name:'Go to forward'}));
  expect((await screen.findAllByRole('menuitem',{name:/Ann/})).length).toBe(2);
 });
});
describe('classifyRelated',()=>{
 it('splits replies from forwards by citation and subject prefix, ignoring drafts and non-sent replies',()=>{
  const orig={id:'o',messageId:['A@x']};
  const r=mkEmail('r',{mailboxIds:{sent:true},inReplyTo:['a@x'],subject:'Re: hi'});
  const f=mkEmail('f',{mailboxIds:{sent:true},references:['A@x'],subject:'Fwd: hi'});
  const f2=mkEmail('f2',{mailboxIds:{sent:true},subject:'Fw: hi'});
  const draft=mkEmail('d',{mailboxIds:{sent:true},inReplyTo:['A@x'],keywords:{$draft:true}});
  const theirs=mkEmail('t',{mailboxIds:{inbox:true},inReplyTo:['A@x']});
  const out=classifyRelated(orig,[r,f,f2,draft,theirs],new Set(['sent']));
  expect(out.replies.map(e=>e.id)).toEqual(['r']);
  expect(out.forwards.map(e=>e.id).sort()).toEqual(['f','f2']);
 });
});
describe('zipStore',()=>{
 it('writes a valid stored archive',async()=>{
  expect(crc32(new TextEncoder().encode('123456789'))).toBe(0xcbf43926);
  const blob=buildZip([{name:'m/a.txt',data:new TextEncoder().encode('hi')}]);
  const bytes=new Uint8Array(await new Promise<ArrayBuffer>(resolve=>{const r=new FileReader();r.onload=()=>resolve(r.result as ArrayBuffer);r.readAsArrayBuffer(blob);}));
  expect(Array.from(bytes.slice(0,4))).toEqual([0x50,0x4b,3,4]);
  expect(Array.from(bytes.slice(-22,-18))).toEqual([0x50,0x4b,5,6]);
 });
});
describe('focusMessageInFolder',()=>{
 const mk=(over:Partial<Parameters<typeof focus.focusMessageInFolder>[0]>={})=>{
  let list:Email[]=[mkEmail('a')];
  const toast=vi.fn();
  let more=2;
  const app={accountId:()=>'acc',selectedMailboxId:()=>'sent',searchActive:()=>false,selectMailbox:vi.fn(async()=>undefined),messages:()=>list,hasMore:()=>more>0,
   loadMore:vi.fn(async()=>{more--;list=[...list,mkEmail(`p${more}`)];}),mailboxes:()=>[{id:'sent',name:'Sent'}],showToast:toast,...over};
  return {app,toast};
 };
 it('pages forward until the message is loaded and requests the flash',async()=>{
  const {app}=mk();
  focus.clearFocusRequest();
  expect(await focus.focusMessageInFolder(app,'sent','p0')).toBe(true);
  expect(app.loadMore).toHaveBeenCalledTimes(2);
  expect(focus.focusRequest()?.emailId).toBe('p0');
 });
 it('switches folder first when elsewhere',async()=>{
  const {app}=mk({selectedMailboxId:()=>'inbox'});
  await focus.focusMessageInFolder(app,'sent','a');
  expect(app.selectMailbox).toHaveBeenCalledWith('sent');
 });
 it('toasts when the message is not in the folder',async()=>{
  const {app,toast}=mk();
  focus.clearFocusRequest();
  expect(await focus.focusMessageInFolder(app,'sent','nope')).toBe(false);
  expect(toast).toHaveBeenCalledWith('error','Message not found in Sent');
  expect(focus.focusRequest()).toBeNull();
 });
});
