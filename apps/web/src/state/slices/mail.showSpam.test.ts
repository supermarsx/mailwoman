// mail.showSpam off: a folder list never shows a `$junk` message, even when the
// rows come from a cached or offline read that predates the keyword.
import {createRoot} from 'solid-js';
import {afterEach,describe,expect,it,vi} from 'vitest';
import {createMailSlice} from './mail.ts';
import type {Client,Me} from '../../api/client.ts';
import {CAP_MAIL,type JmapRequest,type JmapResponse,type JmapSession} from '../../api/jmap-types.ts';
import {suspendAccountPreferences} from '../../api/accountPreferenceLifecycle.ts';
const state=vi.hoisted(()=>({values:{} as Record<string,{value:unknown}>}));
vi.mock('../../api/settings.ts',async()=>({...(await vi.importActual<typeof import('../../api/settings.ts')>('../../api/settings.ts')),settings:()=>state.values}));
afterEach(()=>{suspendAccountPreferences();state.values={};vi.clearAllMocks();});
const me:Me={accountId:'a',username:'u@example.org',storageAuthority:'msa1_x'};
const session:JmapSession={accounts:{a:{name:'Mailbox',isPersonal:true,isReadOnly:false,accountCapabilities:{}}},capabilities:{},primaryAccounts:{[CAP_MAIL]:'a'},username:me.username,apiUrl:'/jmap/api',downloadUrl:'/jmap/download/{accountId}/{blobId}/{name}',uploadUrl:'/jmap/upload/a',eventSourceUrl:'/events',state:'s'};
const row=(id:string,keywords:Record<string,boolean>)=>({id,mailboxIds:{inbox:true},subject:id,keywords,from:[],to:[],receivedAt:'2026-10-26T00:00:00Z',preview:id});
function client():Client{return {
  login:vi.fn(async()=>me),logout:vi.fn(async()=>undefined),me:vi.fn(async()=>me),session:vi.fn(async()=>session),
  jmap:vi.fn(async(body:JmapRequest)=>({sessionState:'s',methodResponses:body.methodCalls.map((call):JmapResponse['methodResponses'][number]=>{
    if(call[0]==='Mailbox/get')return [call[0],{list:[{id:'inbox',name:'Inbox',role:'inbox',sortOrder:0,totalEmails:2,unreadEmails:0}]},call[2]];
    if(call[0]==='Email/query')return [call[0],{ids:['ham','spam'],total:2,position:0},call[2]];
    if(call[0]==='Email/get')return [call[0],{list:[row('ham',{}),row('spam',{$junk:true})]},call[2]];
    return [call[0],{},call[2]];
  })})),sanitize:vi.fn(async value=>value),onNetwork:vi.fn(()=>()=>undefined),
};}
async function subjects(values:Record<string,{value:unknown}>):Promise<string[]>{
  state.values=values;
  return createRoot(async dispose=>{try{const mail=createMailSlice({client:client(),showToast:vi.fn()});await mail.init();return mail.messages().map(m=>m.subject??'');}finally{dispose();}});
}
describe('folder list and mail.showSpam',()=>{
  it('keeps a $junk row that the response carries when showSpam is on',async()=>{
    expect(await subjects({})).toEqual(['ham','spam']);
  });
  it('drops a $junk row from a response that predates the filter when showSpam is off',async()=>{
    expect(await subjects({'mail.showSpam':{value:false}})).toEqual(['ham']);
  });
});
