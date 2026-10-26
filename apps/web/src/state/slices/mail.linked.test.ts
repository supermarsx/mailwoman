import {createRoot} from 'solid-js';
import {afterEach,describe,expect,it,vi} from 'vitest';
import {createMailSlice} from './mail.ts';
import type {Client,Me} from '../../api/client.ts';
import {CAP_MAIL,type JmapRequest,type JmapResponse,type JmapSession} from '../../api/jmap-types.ts';
import {activeLinkedMailbox,installLinkedMailbox,setPrimaryMailboxAuthority} from '../../api/linkedMailboxSession.ts';
import {preferenceAccount,suspendAccountPreferences} from '../../api/accountPreferenceLifecycle.ts';
const {authorize}=vi.hoisted(()=>({authorize:vi.fn(async(id:string)=>({context:{linkId:id,accountId:'same',storageAuthority:'msa1_child_verified'}}))}));
vi.mock('../../api/linkedAccounts.ts',()=>({createLinkedAccountsApi:()=>({authorize,dispose:()=>undefined})}));
afterEach(()=>{installLinkedMailbox(null);setPrimaryMailboxAuthority(undefined);suspendAccountPreferences();vi.clearAllMocks();});
const identity=(child:boolean):Me=>({accountId:'same',username:child?'child@example.org':'parent@example.org',storageAuthority:child?'msa1_child_verified':'msa1_parent_verified'});
function discovery(child:boolean):JmapSession{return {accounts:{same:{name:'Mailbox',isPersonal:true,isReadOnly:false,accountCapabilities:{}}},capabilities:{},primaryAccounts:{[CAP_MAIL]:'same'},username:identity(child).username,apiUrl:'/jmap/api',downloadUrl:'/jmap/download/{accountId}/{blobId}/{name}',uploadUrl:'/jmap/upload/same',eventSourceUrl:'/events',state:'s'};}
function transport():Client{return {
  login:vi.fn(async()=>identity(false)),logout:vi.fn(async()=>undefined),
  me:vi.fn(async(options)=>identity(!!(options?options.linkId:activeLinkedMailbox()?.linkId))),
  session:vi.fn(async(options)=>discovery(!!(options?options.linkId:activeLinkedMailbox()?.linkId))),
  jmap:vi.fn(async(body:JmapRequest)=>({sessionState:'s',methodResponses:body.methodCalls.map((call):JmapResponse['methodResponses'][number]=>{
    const kind=activeLinkedMailbox()?'child':'parent';
    if(call[0]==='Mailbox/get')return [call[0],{list:[{id:`${kind}-inbox`,name:'Inbox',role:'inbox',sortOrder:0,totalEmails:1,unreadEmails:1}]},call[2]];
    if(call[0]==='Email/query')return [call[0],{ids:[`${kind}-message`],total:1,position:0},call[2]];
    if(call[0]==='Email/get')return [call[0],{list:[{id:`${kind}-message`,mailboxIds:{[`${kind}-inbox`]:true},subject:kind,keywords:{},from:[],to:[],receivedAt:'2026-10-26T00:00:00Z',preview:kind}]},call[2]];
    return [call[0],{},call[2]];
  })})),sanitize:vi.fn(async value=>value),onNetwork:vi.fn(()=>()=>undefined),
};}
describe('mailbox switching publication',()=>{
  it('installs verified child state and restores primary without replacing its login',async()=>{
    const client=transport();
    await createRoot(async dispose=>{try{
      const mail=createMailSlice({client,showToast:vi.fn()});
      await mail.login({username:'parent',password:'parent-pass'});
      expect(mail.messages()[0]?.subject).toBe('parent');
      await mail.switchLinkedMailbox('child-link');
      expect(mail.me()?.username).toBe('child@example.org');expect(mail.messages()[0]?.subject).toBe('child');
      expect(preferenceAccount()?.storageAuthority).toBe('msa1_child_verified');
      await mail.switchLinkedMailbox(null);
      expect(mail.me()?.username).toBe('parent@example.org');expect(mail.messages()[0]?.subject).toBe('parent');
      expect(client.login).toHaveBeenCalledOnce();expect(client.logout).not.toHaveBeenCalled();
    }finally{dispose();}});
  });
  it('does not replace the current mailbox when effective child identity mismatches',async()=>{
    const client=transport();
    await createRoot(async dispose=>{try{
      const mail=createMailSlice({client,showToast:vi.fn()});await mail.login({username:'parent',password:'p'});
      vi.mocked(client.me).mockResolvedValue({...identity(true),storageAuthority:'msa1_foreign_verified'});
      await expect(mail.switchLinkedMailbox('child-link')).rejects.toThrow();
      expect(mail.me()?.username).toBe('parent@example.org');expect(mail.messages()[0]?.subject).toBe('parent');expect(activeLinkedMailbox()).toBeNull();
    }finally{dispose();}});
  });
  it('cannot publish delayed child discovery after logout',async()=>{
    const client=transport();let release!:(value:JmapSession)=>void;
    await createRoot(async dispose=>{try{
      const mail=createMailSlice({client,showToast:vi.fn()});await mail.login({username:'parent',password:'p'});
      vi.mocked(client.session).mockImplementationOnce(()=>new Promise(resolve=>{release=resolve;}));
      const selecting=mail.switchLinkedMailbox('child-link');const rejected=expect(selecting).rejects.toMatchObject({name:'AbortError'});
      await vi.waitFor(()=>expect(client.session).toHaveBeenCalledTimes(2));await mail.logout();release(discovery(true));await rejected;
      expect(mail.me()).toBeNull();expect(mail.messages()).toEqual([]);expect(activeLinkedMailbox()).toBeNull();
    }finally{dispose();}});
  });
});
