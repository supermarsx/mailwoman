import {afterEach,expect,it,vi} from 'vitest';
import {createRoot} from 'solid-js';
import {makeClient,mkEmail} from '../../components/appHarness.tsx';
import {createMailSlice,type MailSlice} from './mail.ts';
import {loadSettings,clearSettings} from '../../api/settings.ts';
import {pendingSmartArchive,finishSmartArchive} from '../smartArchiveRequest.ts';
import {toast} from '../../components/toast/toast.ts';
import {clearUndoHistory} from '../undoHistory.ts';
import type {Email,Mailbox,JmapRequest,JmapResponse} from '../../api/jmap-types.ts';
const disposals:(()=>void)[]=[];
afterEach(()=>{while(pendingSmartArchive())finishSmartArchive();for(const dispose of disposals.splice(0))dispose();toast.clear();clearUndoHistory();clearSettings();});
const box=(id:string,role:Mailbox['role']=null):Mailbox=>({id,name:id,role,parentId:null,sortOrder:0,totalEmails:0,unreadEmails:0});
async function fixture(policy:Record<string,unknown>={},history:Email[]=[mkEmail('history',{mailboxIds:{work:true}})]){
 const from=[{email:'sender@example.org',name:null}];history=history.map(email=>({...email,from}));
 const client=makeClient({emails:[mkEmail('a',{from,keywords:{}}),mkEmail('b',{from,keywords:{}})],mailboxes:[box('inbox','inbox'),box('archive','archive'),box('work'),box('other')]}),base=client.jmap;
 const writes:Record<string,Record<string,unknown>>[]=[],showToast=vi.fn();
 let historyFailure=false,release:((response:JmapResponse)=>void)|undefined,hold=false;
 client.jmap=vi.fn(async(request:JmapRequest):Promise<JmapResponse>=>{
  const smart=request.methodCalls.find(call=>call[2]==='sg');
  if(smart){if(historyFailure)throw new Error('history failed');if(hold)return new Promise(resolve=>{release=resolve;});return {sessionState:'s',methodResponses:[['Email/get',{accountId:'acct1',list:history,notFound:[]},'sg']]};}
  const set=request.methodCalls.find(call=>call[0]==='Email/set');
  if(set){const update=set[1]['update'] as Record<string,Record<string,unknown>>;writes.push(update);return {sessionState:'s',methodResponses:[['Email/set',{accountId:'acct1',updated:Object.fromEntries(Object.keys(update).map(id=>[id,null]))},set[2]]]};}
  return base(request);
 });
 let mail!:MailSlice;createRoot(dispose=>{disposals.push(dispose);mail=createMailSlice({client,showToast});});
 await mail.login({username:'me@example.org',password:'p'});
 const configure=async(values:Record<string,unknown>)=>loadSettings({refresh:true,fetcher:async()=>new Response(JSON.stringify({settings:Object.fromEntries(Object.entries(values).map(([key,value])=>[`mail.smartArchive.${key}`,{value,source:'user',locked:false}]))}))});
 const values={enabled:true,confirmation:'never',multipleFolders:'latest',...policy};await configure(values);
 return {mail,writes,showToast,configure,values,client,fail:()=>{historyFailure=true;},hold:()=>{hold=true;},release:()=>release?.({sessionState:'s',methodResponses:[['Email/get',{accountId:'acct1',list:history,notFound:[]},'sg']]})};
}
it('leaves normal Archive unchanged until replacement is explicitly enabled',async()=>{const s=await fixture({enabled:false,replaceArchive:true});await s.mail.smartArchiveMessage('a');expect(s.writes).toHaveLength(0);await s.mail.archiveMessage('a');expect(s.writes[0]?.['a']?.['mailboxIds']).toEqual({archive:true});});
it('still permits the explicitly configured ordinary Archive fallback when no custom folder matches',async()=>{const s=await fixture({},[]);await s.mail.smartArchiveMessage('a');expect(s.writes[0]?.['a']?.['mailboxIds']).toEqual({archive:true});});
it('routes replacement through Smart Archive and preserves an explicit ordinary Archive',async()=>{const s=await fixture({replaceArchive:true});await s.mail.archiveMessage('a');expect(s.writes[0]?.['a']?.['mailboxIds']).toEqual({work:true});await s.mail.archiveMessage('b',{standard:true});expect(s.writes[1]?.['b']?.['mailboxIds']).toEqual({archive:true});});
it('files to matching history without marking read and restores membership with Undo',async()=>{const s=await fixture();await s.mail.smartArchiveMessage('a');expect(s.writes).toHaveLength(1);expect(s.writes[0]?.['a']).toEqual({mailboxIds:{work:true}});await s.mail.undoNow();expect(s.writes[1]?.['a']?.['mailboxIds']).toEqual({inbox:true});});
it('asks for ambiguous folders even when extra confirmations are off, and cancellation moves nothing',async()=>{const s=await fixture({multipleFolders:'ask'},[mkEmail('one',{mailboxIds:{work:true}}),mkEmail('two',{mailboxIds:{other:true}})]);const pending=s.mail.smartArchiveMessage('a');await vi.waitFor(()=>expect(pendingSmartArchive()).toBeDefined());expect(pendingSmartArchive()?.groups[0]?.target).toBe('');expect(s.writes).toHaveLength(0);finishSmartArchive();await pending;expect(s.writes).toHaveLength(0);});
it('rechecks policy after confirmation and does not move after disablement',async()=>{const s=await fixture({confirmation:'always'});const pending=s.mail.smartArchiveMessage('a');await vi.waitFor(()=>expect(pendingSmartArchive()).toBeDefined());const groups=pendingSmartArchive()!.groups;await s.configure({...s.values,enabled:false});finishSmartArchive(groups);await pending;expect(s.writes).toHaveLength(0);});
it('cancels an open request when the account logs out',async()=>{const s=await fixture({confirmation:'always'});const pending=s.mail.smartArchiveMessage('a');await vi.waitFor(()=>expect(pendingSmartArchive()).toBeDefined());await s.mail.logout();await pending;expect(pendingSmartArchive()).toBeUndefined();expect(s.writes).toHaveLength(0);});
it('ignores a history lookup completing after logout and does not fall back on query errors',async()=>{const s=await fixture();s.hold();const pending=s.mail.smartArchiveMessage('a');await vi.waitFor(()=>expect(vi.mocked(s.client.jmap).mock.calls.some(([request])=>request.methodCalls.some(call=>call[2]==='sg'))).toBe(true));await s.mail.logout();s.release();await pending;expect(s.writes).toHaveLength(0);});
it('leaves mail alone when history lookup fails instead of silently using ordinary Archive',async()=>{const s=await fixture();s.fail();await s.mail.smartArchiveMessage('a');expect(s.writes).toHaveLength(0);expect(s.showToast).toHaveBeenCalledWith('error',expect.any(String));});
it('coalesces repeated clicks on the same pending smart action',async()=>{const s=await fixture();s.hold();const first=s.mail.smartArchiveMessage('a'),second=s.mail.smartArchiveMessage('a');expect(first).toBe(second);await vi.waitFor(()=>expect(vi.mocked(s.client.jmap).mock.calls.some(([request])=>request.methodCalls.some(call=>call[2]==='sg'))).toBe(true));s.release();await Promise.all([first,second]);expect(s.writes).toHaveLength(1);});
