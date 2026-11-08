import {toast as toastStore,toasts} from '../../components/toast/toast.ts';
import {createRoot} from 'solid-js';
import {afterEach,describe,expect,it,vi} from 'vitest';
import {createMailSlice} from './mail.ts';
import type {Client,Me} from '../../api/client.ts';
import {CAP_MAIL,type JmapRequest,type JmapResponse,type JmapSession} from '../../api/jmap-types.ts';
import {activeLinkedMailbox,installLinkedMailbox,setPrimaryMailboxAuthority} from '../../api/linkedMailboxSession.ts';
import {suspendAccountPreferences} from '../../api/accountPreferenceLifecycle.ts';
const {authorize}=vi.hoisted(()=>({authorize:vi.fn(async(id:string)=>({context:{linkId:id,accountId:'same',storageAuthority:'msa1_child_verified'}}))}));
// `list` is what mail.ts asks at sign-in for the mailbox to open first (t40): no
// link is marked, so the sign-in mailbox opens and these tests switch by hand.
vi.mock('../../api/linkedAccounts.ts',()=>({createLinkedAccountsApi:()=>({authorize,list:async()=>({policy:{enabled:true},accounts:[],primaryMailbox:null,primaryFallback:false}),setPrimary:async()=>undefined,dispose:()=>undefined})}));
const state={noTrash:false,set:undefined as undefined|((call:JmapRequest['methodCalls'][number])=>JmapResponse['methodResponses'][number]),reports:[] as string[]};
vi.mock('../../observability/clientErrors.ts',()=>({reportBoundaryError:(e:Error,b:string)=>{state.reports.push(`${b}:${e.message}`);return null;}}));
afterEach(()=>{toastStore.clear();state.noTrash=false;state.set=undefined;state.reports=[];installLinkedMailbox(null);setPrimaryMailboxAuthority(undefined);suspendAccountPreferences();vi.clearAllMocks();});
const toastStoreTexts=():string[]=>toasts().map(i=>String((i as {message?:unknown}).message));
const identity=(child:boolean):Me=>({accountId:'same',username:child?'child@example.org':'parent@example.org',storageAuthority:child?'msa1_child_verified':'msa1_parent_verified'});
function discovery(child:boolean):JmapSession{return {accounts:{same:{name:'Mailbox',isPersonal:true,isReadOnly:false,accountCapabilities:{}}},capabilities:{},primaryAccounts:{[CAP_MAIL]:'same'},username:identity(child).username,apiUrl:'/jmap/api',downloadUrl:'/jmap/download/{accountId}/{blobId}/{name}',uploadUrl:'/jmap/upload/same',eventSourceUrl:'/events',state:'s'};}
function transport():Client{return {
  login:vi.fn(async()=>identity(false)),logout:vi.fn(async()=>undefined),
  me:vi.fn(async(options)=>identity(!!(options?options.linkId:activeLinkedMailbox()?.linkId))),
  session:vi.fn(async(options)=>discovery(!!(options?options.linkId:activeLinkedMailbox()?.linkId))),
  jmap:vi.fn(async(body:JmapRequest)=>({sessionState:'s',methodResponses:body.methodCalls.map((call):JmapResponse['methodResponses'][number]=>{
    const kind=activeLinkedMailbox()?'child':'parent';
    if(call[0]==='Mailbox/get')return [call[0],{list:[{id:`${kind}-inbox`,name:'Inbox',role:'inbox',sortOrder:0,totalEmails:1,unreadEmails:1},...(state.noTrash?[]:[{id:`${kind}-trash`,name:'Trash',role:'trash',sortOrder:5,totalEmails:0,unreadEmails:0}])]},call[2]];
    if(call[0]==='Email/set'&&state.set)return state.set(call);
    if(call[0]==='Email/query')return [call[0],{ids:[`${kind}-message`],total:1,position:0},call[2]];
    if(call[0]==='Email/get')return [call[0],{list:[{id:`${kind}-message`,mailboxIds:{[`${kind}-inbox`]:true},subject:kind,keywords:{},from:[],to:[],receivedAt:'2026-10-26T00:00:00Z',preview:kind}]},call[2]];
    return [call[0],{},call[2]];
  })})),sanitize:vi.fn(async value=>value),onNetwork:vi.fn(()=>()=>undefined),
};}

async function linked(run:(mail:ReturnType<typeof createMailSlice>,client:Client,toast:ReturnType<typeof vi.fn>)=>Promise<void>){
  const client=transport();
  await createRoot(async dispose=>{try{
    const toast=vi.fn();const mail=createMailSlice({client,showToast:toast});
    await mail.login({username:'parent',password:'p'});await mail.switchLinkedMailbox('child-link');
    expect(mail.messages()[0]?.id).toBe('child-message');
    await run(mail,client,toast);
  }finally{dispose();}});
}
const setCalls=(client:Client)=>vi.mocked(client.jmap).mock.calls.flatMap(c=>c[0].methodCalls).filter(c=>c[0]==='Email/set');
describe('Move to Trash in a linked mailbox',()=>{
  it('sends one Email/set naming the linked account own Trash folder',async()=>{
    await linked(async(mail,client)=>{
      await mail.trashMessage('child-message');
      const [call]=setCalls(client);
      expect(call![1]['update']).toEqual({'child-message':{mailboxIds:{'child-trash':true}}});
      expect(mail.messages()).toEqual([]);
    });
  });
  it.each([
    ['notUpdated forbidden',{notUpdated:{'child-message':{type:'forbidden',description:'legal hold'}}},'forbidden: legal hold'],
    ['notUpdated invalidProperties',{notUpdated:{'child-message':{type:'invalidProperties',description:'mailbox child-trash not found'}}},'invalidProperties: mailbox child-trash not found'],
    ['notUpdated notFound',{notUpdated:{'child-message':{type:'notFound'}}},'notFound'],
  ])('keeps the message and shows the reason when the server answers %s',async(_n,answer,reason)=>{
    await linked(async(mail,_c,toast)=>{
      state.set=call=>[call[0],{accountId:'same',...answer},call[2]];
      await mail.trashMessage('child-message');
      await vi.waitFor(()=>expect(toastStoreTexts().join(' ')).toContain(reason));
      expect(mail.messages().map(m=>m.id)).toEqual(['child-message']);
      expect(JSON.stringify(vi.mocked(toast).mock.calls)+JSON.stringify(toastStoreTexts())).toContain(reason);
      expect(state.reports.some(r=>r.startsWith('mail/trash:'))).toBe(true);
    });
  });
  it('keeps the message and shows a reason when the whole Email/set is refused (stateMismatch)',async()=>{
    await linked(async(mail)=>{
      state.set=call=>['error',{type:'stateMismatch'},call[2]] as never;
      await mail.trashMessage('child-message');
      await vi.waitFor(()=>expect(toastStoreTexts().join(' ')).toContain('stateMismatch'));
      expect(mail.messages().map(m=>m.id)).toEqual(['child-message']);
    });
  });
  it('keeps the message and names the failure when the request itself throws (an unsupported linked route)',async()=>{
    await linked(async(mail,client)=>{
      const original=vi.mocked(client.jmap).getMockImplementation()!;
      vi.mocked(client.jmap).mockImplementation(async(body,o)=>{if(body.methodCalls.some(c=>c[0]==='Email/set'))throw Object.assign(new Error('This operation is unavailable in a linked mailbox.'),{name:'LinkedContextUnsupportedError'});return original(body,o);});
      await mail.trashMessage('child-message');
      await vi.waitFor(()=>expect(toastStoreTexts().join(' ')).toContain('LinkedContextUnsupportedError'));
      expect(mail.messages().map(m=>m.id)).toEqual(['child-message']);
      expect(state.reports.some(r=>r.startsWith('mail/trash:'))).toBe(true);
    });
  });
  it('says no Trash folder is configured when the linked account has none, and sends nothing',async()=>{
    state.noTrash=true;
    await linked(async(mail,client)=>{
      await mail.trashMessage('child-message');
      expect(setCalls(client)).toHaveLength(0);expect(mail.messages().map(m=>m.id)).toEqual(['child-message']);
    });
  });
});
