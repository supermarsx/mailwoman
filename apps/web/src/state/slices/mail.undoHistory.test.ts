// The global undo history, driven through the real mail slice: each reversible
// action records one entry, the entry undoes and redoes on the fake server, and
// an undo whose message was moved or deleted elsewhere fails with a toast.
import {afterEach,beforeEach,describe,expect,it,vi} from 'vitest';
import {createRoot} from 'solid-js';
import {createMailSlice,type MailSlice} from './mail.ts';
import {clearSettings,loadSettings,type Fetcher} from '../../api/settings.ts';
import {clearUndoHistory,redoEntries,redoLast,undoEntries,undoLast} from '../undoHistory.ts';
import {toast as toastStore,toasts} from '../../components/toast/toast.ts';
import {loadCatalog} from '../../i18n/index.ts';
import type {Client,Me} from '../../api/client.ts';
import {CAP_MAIL,type Email,type JmapRequest,type JmapResponse,type JmapSession,type Mailbox} from '../../api/jmap-types.ts';

vi.mock('../../api/senderResolution.ts',()=>({resolveSender:vi.fn()}));
vi.mock('../../api/focusedSenders.ts',()=>({loadFocusedSenders:vi.fn(async()=>[]),saveFocusedSender:vi.fn(async()=>undefined),importFocusedSenders:vi.fn(async()=>[])}));

const BOXES:Mailbox[]=[
 {id:'inbox',name:'Inbox',parentId:null,role:'inbox',sortOrder:0,totalEmails:0,unreadEmails:0},
 {id:'archive',name:'Archive',parentId:null,role:'archive',sortOrder:1,totalEmails:0,unreadEmails:0},
 {id:'trash',name:'Trash',parentId:null,role:'trash',sortOrder:2,totalEmails:0,unreadEmails:0},
 {id:'work',name:'Work',parentId:null,role:null,sortOrder:4,totalEmails:0,unreadEmails:0},
];
const email=(id:string,over:Partial<Email>={}):Email=>({id,mailboxIds:{inbox:true},from:[{name:null,email:`${id}@example.org`}],to:[{name:null,email:'me@example.org'}],subject:`Subject ${id}`,receivedAt:'2026-01-01T00:00:00Z',preview:'',keywords:{},...over});
const SESSION:JmapSession={capabilities:{},accounts:{acct1:{name:'T',isPersonal:true,isReadOnly:false,accountCapabilities:{}}},primaryAccounts:{[CAP_MAIL]:'acct1'},username:'me@example.org',apiUrl:'/jmap/api',downloadUrl:'/d',uploadUrl:'/u',eventSourceUrl:'/e',state:'s0'};

interface Server {refuse:Set<string>;sets:Record<string,Record<string,unknown>>[]}
async function run(seed:Email[],body:(mail:MailSlice,server:Server)=>Promise<void>):Promise<void> {
 const refuse=new Set<string>(),sets:Record<string,Record<string,unknown>>[]=[];
 const jmap=vi.fn(async(request:JmapRequest):Promise<JmapResponse>=>{
  const names=request.methodCalls.map(call=>call[0]);
  if(names.includes('Mailbox/get')&&!names.includes('Email/set'))return {methodResponses:request.methodCalls.map(call=>call[0]==='Mailbox/get'?['Mailbox/get',{accountId:'acct1',state:'s',list:BOXES,notFound:[]},call[2]]:[call[0],{accountId:'acct1'},call[2]]),sessionState:'s'};
  if(names.includes('Email/get'))return {methodResponses:[['Email/query',{accountId:'acct1',ids:seed.map(item=>item.id)},'q'],['Email/get',{accountId:'acct1',state:'s',list:seed,notFound:[]},'g']],sessionState:'s'};
  return {methodResponses:request.methodCalls.map(call=>{
   if(call[0]==='Mailbox/get')return ['Mailbox/get',{accountId:'acct1',state:'s',list:BOXES,notFound:[]},call[2]] as JmapResponse['methodResponses'][number];
   const update=((call[1] as {update?:Record<string,Record<string,unknown>>}).update)??{};
   if(call[0]==='Email/set')sets.push(update);
   const notUpdated=Object.fromEntries(Object.keys(update).filter(id=>refuse.has(id)).map(id=>[id,{type:'notFound'}]));
   return [call[0],Object.keys(notUpdated).length>0?{notUpdated}:{},call[2]] as JmapResponse['methodResponses'][number];
  }),sessionState:'s'};
 });
 const client:Client={login:vi.fn(async():Promise<Me>=>({username:'me@example.org',accountId:'acct1'})),logout:vi.fn(async()=>undefined),me:vi.fn(async():Promise<Me>=>({username:'me@example.org',accountId:'acct1'})),session:vi.fn(async()=>SESSION),jmap,sanitize:vi.fn(async(html:string)=>html),onNetwork:vi.fn(()=>()=>undefined)};
 await createRoot(async dispose=>{
  const mail=createMailSlice({client,showToast:vi.fn()});
  await mail.login({username:'me@example.org',password:'p'});
  await body(mail,{refuse,sets});
  dispose();
 });
}
const setSize=(value:number)=>loadSettings({refresh:true,fetcher:(async()=>new Response(JSON.stringify({settings:{'ui.undoHistorySize':{value,source:'default',locked:false}}}),{status:200,headers:{'content-type':'application/json'}})) as Fetcher});

beforeEach(async()=>{await loadCatalog('undo');await loadCatalog('mail');clearUndoHistory();toastStore.clear();});
afterEach(()=>{clearSettings();clearUndoHistory();toastStore.clear();});

describe('undo history through the mail slice',()=>{
 it('records mark read, flag, archive, trash and move as one entry each and walks them back and forth',async()=>{
  await run([email('a'),email('b'),email('c'),email('d'),email('e')],async mail=>{
   await mail.markRead('a',true);
   await mail.flagMessage('b',true);
   await mail.archiveMessage('c');
   await mail.trashMessage(['d']);
   await mail.moveMessage('e','work');
   expect(undoEntries()).toHaveLength(5);
   expect(undoEntries()[0]!.accountId).toBe('acct1');
   expect(mail.messages().map(item=>item.id)).toEqual(['a','b']);
   expect(await undoLast()).toBe(1);
   expect(mail.messages().map(item=>item.id)).toContain('e');
   expect(redoEntries()).toHaveLength(1);
   expect(await redoLast()).toBe(1);
   expect(mail.messages().map(item=>item.id)).not.toContain('e');
   expect(undoEntries()).toHaveLength(5);
   expect(redoEntries()).toHaveLength(0);
   for(let i=0;i<5;i++)await undoLast();
   expect(undoEntries()).toHaveLength(0);
   expect(redoEntries()).toHaveLength(5);
   expect(mail.messages().map(item=>item.id).sort()).toEqual(['a','b','c','d','e']);
   expect(mail.messages().find(item=>item.id==='a')!.keywords?.$seen).toBeFalsy();
   expect(mail.messages().find(item=>item.id==='b')!.keywords?.$flagged).toBeFalsy();
  });
 });

 it('a bulk gesture is one entry with a count',async()=>{
  await run([email('a'),email('b'),email('c')],async mail=>{
   await mail.archiveMessage(['a','b','c']);
   expect(undoEntries()).toHaveLength(1);
   expect(undoEntries()[0]!.label).toContain('3');
  });
 });

 it('a new action empties the redo list',async()=>{
  await run([email('a'),email('b')],async mail=>{
   await mail.markRead('a',true);
   await undoLast();
   expect(redoEntries()).toHaveLength(1);
   await mail.flagMessage('b',true);
   expect(redoEntries()).toHaveLength(0);
  });
 });

 it('the toast Undo goes through the history, so the entry can be redone',async()=>{
  await run([email('a')],async mail=>{
   await mail.archiveMessage('a');
   expect(undoEntries()).toHaveLength(1);
   await mail.undoNow();
   expect(undoEntries()).toHaveLength(0);
   expect(redoEntries()).toHaveLength(1);
   expect(mail.messages().map(item=>item.id)).toEqual(['a']);
  });
 });

 it('an undo whose message was moved or deleted elsewhere fails with a toast and drops the entry',async()=>{
  await run([email('a')],async(mail,server)=>{
   await mail.archiveMessage('a');
   server.refuse.add('a');
   expect(await undoLast()).toBe(0);
   expect(undoEntries()).toHaveLength(0);
   expect(redoEntries()).toHaveLength(0);
   expect(toasts().some(item=>item.tone==='error'&&item.message.includes("Can't undo"))).toBe(true);
  });
 });

 it('undoes a flag on a message that is no longer in the list by talking to the server',async()=>{
  await run([email('a'),email('b')],async(mail,server)=>{
   await mail.flagMessage('a',true);
   await mail.archiveMessage('a');
   server.sets.length=0;
   await undoLast();await undoLast();
   expect(server.sets.some(update=>update.a&&'keywords/$flagged' in update.a)).toBe(true);
  });
 });

 it('records nothing when the history size is 0',async()=>{
  await setSize(0);
  await run([email('a')],async mail=>{
   await mail.markRead('a',true);
   await mail.archiveMessage('a');
   expect(undoEntries()).toHaveLength(0);
   expect(mail.pendingUndo()).not.toBeNull();
   await mail.undoNow();
   expect(mail.messages().map(item=>item.id)).toEqual(['a']);
  });
 });

 it('keeps at most the configured number of actions',async()=>{
  await setSize(2);
  await run([email('a'),email('b'),email('c')],async mail=>{
   await mail.flagMessage('a',true);await mail.flagMessage('b',true);await mail.flagMessage('c',true);
   expect(undoEntries()).toHaveLength(2);
  });
 });
});
