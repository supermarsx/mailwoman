// t109: a reload in a linked mailbox opens that mailbox again, through the same
// authorization as a switch; a link that is gone falls back to the sign-in
// account with a notice; nothing kept for one sign-in is used by another; the
// folder comes back when it still exists; sign-out forgets it all.
import {createRoot} from 'solid-js';
import {afterEach,beforeEach,describe,expect,it,vi} from 'vitest';
import {createMailSlice} from './mail.ts';
import type {Client,Me} from '../../api/client.ts';
import {CAP_MAIL,type JmapRequest,type JmapResponse,type JmapSession} from '../../api/jmap-types.ts';
import {activeLinkedMailbox,installLinkedMailbox,setPrimaryMailboxAuthority} from '../../api/linkedMailboxSession.ts';
import {suspendAccountPreferences} from '../../api/accountPreferenceLifecycle.ts';
import {SESSION_RESTORE_KEY,readSession,rememberView} from '../sessionRestore.ts';
const {authorize}=vi.hoisted(()=>({authorize:vi.fn(async(id:string)=>({context:{linkId:id,accountId:'same',storageAuthority:'msa1_child_verified'}}))}));
vi.mock('../../api/linkedAccounts.ts',()=>({createLinkedAccountsApi:()=>({authorize,list:async()=>({policy:{enabled:true},accounts:[],primaryMailbox:null,primaryFallback:false}),setPrimary:async()=>undefined,dispose:()=>undefined})}));
const PARENT='msa1_parent_verified';
beforeEach(()=>{sessionStorage.clear();history.replaceState(null,'','/#/mail');});
afterEach(()=>{installLinkedMailbox(null);setPrimaryMailboxAuthority(undefined);suspendAccountPreferences();vi.clearAllMocks();sessionStorage.clear();history.replaceState(null,'','/');});
const identity=(child:boolean,parent=PARENT):Me=>({accountId:'same',username:child?'child@example.org':'parent@example.org',storageAuthority:child?'msa1_child_verified':parent});
function discovery(child:boolean):JmapSession{return {accounts:{same:{name:'Mailbox',isPersonal:true,isReadOnly:false,accountCapabilities:{}}},capabilities:{},primaryAccounts:{[CAP_MAIL]:'same'},username:identity(child).username,apiUrl:'/jmap/api',downloadUrl:'/jmap/download/{accountId}/{blobId}/{name}',uploadUrl:'/jmap/upload/same',eventSourceUrl:'/events',state:'s'};}
function transport(parent=PARENT):Client{return {
  login:vi.fn(async()=>identity(false,parent)),logout:vi.fn(async()=>undefined),
  me:vi.fn(async(options)=>identity(!!(options?options.linkId:activeLinkedMailbox()?.linkId),parent)),
  session:vi.fn(async(options)=>discovery(!!(options?options.linkId:activeLinkedMailbox()?.linkId))),
  jmap:vi.fn(async(body:JmapRequest)=>({sessionState:'s',methodResponses:body.methodCalls.map((call):JmapResponse['methodResponses'][number]=>{
    const kind=activeLinkedMailbox()?'child':'parent';
    if(call[0]==='Mailbox/get')return [call[0],{list:[{id:`${kind}-inbox`,name:'Inbox',role:'inbox',sortOrder:0,totalEmails:1,unreadEmails:1},{id:`${kind}-work`,name:'Work',role:null,sortOrder:1,totalEmails:1,unreadEmails:0}]},call[2]];
    if(call[0]==='Email/query')return [call[0],{ids:[`${kind}-message`],total:1,position:0},call[2]];
    if(call[0]==='Email/get')return [call[0],{list:[{id:`${kind}-message`,mailboxIds:{[`${kind}-inbox`]:true},subject:kind,keywords:{},from:[],to:[],receivedAt:'2026-10-26T00:00:00Z',preview:kind}]},call[2]];
    return [call[0],{},call[2]];
  })})),sanitize:vi.fn(async value=>value),onNetwork:vi.fn(()=>()=>undefined),
};}
/** One page load: a fresh store over the same browser tab storage. */
async function page<T>(run:(mail:ReturnType<typeof createMailSlice>,toast:ReturnType<typeof vi.fn>)=>Promise<T>,client:Client=transport()):Promise<T>{
  installLinkedMailbox(null);setPrimaryMailboxAuthority(undefined);suspendAccountPreferences();
  return createRoot(async dispose=>{try{const toast=vi.fn();const mail=createMailSlice({client,showToast:toast});return await run(mail,toast);}finally{dispose();}});
}

describe('t109 reload restore',()=>{
  it('a reload in a linked mailbox opens that mailbox again, never the sign-in account first',async()=>{
    await page(async mail=>{await mail.login({username:'parent',password:'p'});await mail.switchLinkedMailbox('child-link');});
    expect(readSession(PARENT)?.active).toBe('child-link');
    const client=transport();
    await page(async mail=>{
      await mail.init();
      expect(activeLinkedMailbox()?.linkId).toBe('child-link');
      expect(mail.me()?.username).toBe('child@example.org');expect(mail.messages()[0]?.subject).toBe('child');
      // The sign-in account's list was never asked for.
      const lists=vi.mocked(client.jmap).mock.calls.filter(([body])=>body.methodCalls.some(call=>call[0]==='Email/query'));
      expect(lists).toHaveLength(1);
    },client);
    expect(authorize).toHaveBeenCalledWith('child-link');
  });
  it('a reload after switching back opens the sign-in account',async()=>{
    await page(async mail=>{await mail.login({username:'parent',password:'p'});await mail.switchLinkedMailbox('child-link');await mail.switchLinkedMailbox(null);});
    expect(readSession(PARENT)?.active).toBeNull();
    authorize.mockClear();
    await page(async mail=>{await mail.init();expect(activeLinkedMailbox()).toBeNull();expect(mail.messages()[0]?.subject).toBe('parent');});
    expect(authorize).not.toHaveBeenCalled();
  });
  it('a link that is gone or off falls back to the sign-in account with a notice and is forgotten',async()=>{
    await page(async mail=>{await mail.login({username:'parent',password:'p'});await mail.switchLinkedMailbox('child-link');});
    authorize.mockRejectedValueOnce(new Error('not linked'));
    await page(async(mail,toast)=>{
      await mail.init();
      expect(activeLinkedMailbox()).toBeNull();expect(mail.me()?.username).toBe('parent@example.org');expect(mail.messages()[0]?.subject).toBe('parent');
      await vi.waitFor(()=>expect(toast).toHaveBeenCalledWith('info',expect.any(String)));
    });
    expect(readSession(PARENT)?.active).toBeNull();
  });
  it('nothing kept for one sign-in is used by another',async()=>{
    await page(async mail=>{await mail.login({username:'parent',password:'p'});await mail.switchLinkedMailbox('child-link');});
    authorize.mockClear();
    await page(async mail=>{await mail.init();expect(activeLinkedMailbox()).toBeNull();expect(mail.me()?.username).toBe('parent@example.org');},transport('msa1_someone_else'));
    expect(authorize).not.toHaveBeenCalled();
    // The other person's record is replaced by this sign-in's own, not kept for later.
    expect(sessionStorage.getItem(SESSION_RESTORE_KEY)).not.toContain('child-link');
    expect(readSession('msa1_someone_else')?.active).toBeNull();
    expect(readSession(PARENT)).toBeNull();
  });
  it('restores the folder when it still exists and opens the inbox when it does not',async()=>{
    rememberView(PARENT,{id:'same',storageAuthority:PARENT},{folder:'parent-work'});
    await page(async mail=>{await mail.login({username:'parent',password:'p'});expect(mail.selectedMailboxId()).toBe('parent-work');});
    rememberView(PARENT,{id:'same',storageAuthority:PARENT},{folder:'deleted-folder'});
    await page(async mail=>{await mail.login({username:'parent',password:'p'});expect(mail.selectedMailboxId()).toBe('parent-inbox');});
    // A view kept under another mailbox's authority (same account id) is not this one's.
    rememberView(PARENT,{id:'same',storageAuthority:'msa1_child_verified'},{folder:'parent-work'});
    await page(async mail=>{await mail.login({username:'parent',password:'p'});expect(mail.selectedMailboxId()).toBe('parent-inbox');});
  });
  it('restores a search in the restored folder',async()=>{
    rememberView(PARENT,{id:'same',storageAuthority:PARENT},{folder:'parent-work',search:'from:alice'});
    await page(async mail=>{await mail.login({username:'parent',password:'p'});expect(mail.selectedMailboxId()).toBe('parent-work');expect(mail.searchActive()).toBe(true);expect(mail.search()).toBe('from:alice');});
  });
  it('a link in the address bar names the mailbox and folder',async()=>{
    history.replaceState(null,'','/?mailbox=child-link&folder=child-work#/mail');
    await page(async mail=>{await mail.login({username:'parent',password:'p'});expect(activeLinkedMailbox()?.linkId).toBe('child-link');expect(mail.selectedMailboxId()).toBe('child-work');});
  });
  it('sign-out forgets the tab state and the address-bar target',async()=>{
    history.replaceState(null,'','/?mailbox=child-link&folder=child-work#/mail');
    await page(async mail=>{await mail.login({username:'parent',password:'p'});await mail.logout();});
    expect(sessionStorage.getItem(SESSION_RESTORE_KEY)).toBeNull();
    expect(location.search).toBe('');expect(location.hash).toBe('#/mail');
  });
});
