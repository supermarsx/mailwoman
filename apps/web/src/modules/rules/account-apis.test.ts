import {afterEach,expect,it,vi} from 'vitest';
import {createClient} from '../../api/client.ts';
import {installLinkedMailbox} from '../../api/linkedMailboxSession.ts';
import {activateAccountPreferences,suspendAccountPreferences} from '../../api/accountPreferenceLifecycle.ts';
import {queueRuleDraft,takeRuleDraft} from '../../state/ruleDraft.ts';
import {emptyRuleDraft} from '../../state/slices/rules.ts';
import {sieveApi,CAP_SIEVE} from './sieve-api.ts';
import {vacationApi,CAP_VACATION} from './vacation-api.ts';
const activate=(id:string)=>{installLinkedMailbox({linkId:`link-${id}`,accountId:id,storageAuthority:`test_storage_authority_${id}`});activateAccountPreferences(id,`test_storage_authority_${id}`);};
const json=(value:unknown,status=200)=>new Response(JSON.stringify(value),{status,headers:{'content-type':'application/json'}});
const session=(account:string)=>({capabilities:{[CAP_SIEVE]:{},[CAP_VACATION]:{}},accounts:{[account]:{name:account,isPersonal:true,isReadOnly:false,accountCapabilities:{[CAP_SIEVE]:{maxSizeScript:65536,sieveExtensions:['fileinto','vacation']},[CAP_VACATION]:{}}}},primaryAccounts:{},username:'fixture',state:'s',apiUrl:'/jmap/api',uploadUrl:'/jmap/upload/{accountId}',downloadUrl:'/jmap/download/{accountId}/{blobId}/{name}'});
afterEach(()=>{installLinkedMailbox(null);suspendAccountPreferences();takeRuleDraft();vi.unstubAllGlobals();});
it('real client wire keeps Sieve and vacation CRUD on the selected link and never falls back after refusal',async()=>{
 activate('linked');const calls:{path:string;link:string|null;body:unknown}[]=[];let refuse=false;
 vi.stubGlobal('fetch',vi.fn(async(input:RequestInfo|URL,init?:RequestInit)=>{
  const path=new URL(String(input),location.origin).pathname,link=new Headers(init?.headers).get('x-mailwoman-link');
  const body=typeof init?.body==='string'?JSON.parse(init.body):null;calls.push({path,link,body});
  if(refuse)return json({error:'denied'},403);
  if(path==='/jmap/session')return json(session('linked'));
  if(path==='/jmap/upload/linked')return json({accountId:'linked',blobId:'blob',type:'application/sieve',size:5});
  const [name,args,tag]=body.methodCalls[0];
  const value=name==='SieveScript/get'?{accountId:args.accountId,state:'s',list:[],notFound:[]}:
   name==='VacationResponse/get'?{accountId:args.accountId,state:'s',list:[{id:'singleton',isEnabled:false,fromDate:null,toDate:null,subject:null,textBody:'Away',htmlBody:null}]}:
    {accountId:args.accountId,newState:'s2',created:{new:{id:'script'}},updated:{singleton:null},destroyed:[]};
  return json({sessionState:'s',methodResponses:[[name,value,tag]]});
 }));
 const client=createClient(),scripts=sieveApi(client,'linked'),vacation=vacationApi(client,'linked');
 await scripts.connect();await scripts.get();await scripts.save(null,'Rules','keep;', 's',false);await scripts.activate('script','s2');await scripts.remove('script','s2');
 const away=await vacation.get();await vacation.save({...away.value,isEnabled:true},away.state);
 expect(calls.every(call=>call.link==='link-linked')).toBe(true);
 const methods=calls.filter(call=>call.path==='/jmap/api').map(call=>(call.body as {methodCalls:[string,{accountId:string},string][]}).methodCalls[0]!);
 expect(methods.map(call=>call[0])).toEqual(['SieveScript/get','SieveScript/set','SieveScript/set','SieveScript/set','VacationResponse/get','VacationResponse/set']);
 expect(methods.every(call=>call[1].accountId==='linked')).toBe(true);
 const before=calls.length;refuse=true;await expect(scripts.get()).rejects.toMatchObject({status:403});expect(calls.length).toBe(before+1);
 activate('other');const after=calls.length;await expect(scripts.get()).rejects.toMatchObject({name:'AbortError'});expect(calls.length).toBe(after);
});
it('a late old upload cannot issue its Sieve set after changing linked accounts',async()=>{
 activate('a');let finish!:(response:Response)=>void;const paths:string[]=[];
 vi.stubGlobal('fetch',vi.fn(async(input:RequestInfo|URL)=>{
  const path=new URL(String(input),location.origin).pathname;paths.push(path);
  if(path==='/jmap/session')return json(session('a'));
  if(path==='/jmap/upload/a')return await new Promise<Response>(resolve=>{finish=resolve;});
  throw Error('stale upload must not reach a set or another account');
 }));
 const api=sieveApi(createClient(),'a');await api.connect();const save=api.save(null,'A rules','keep;','s',false);
 await vi.waitFor(()=>expect(paths).toContain('/jmap/upload/a'));
 activate('b');finish(json({accountId:'a',blobId:'old-upload',type:'application/sieve',size:5}));
 await expect(save).rejects.toMatchObject({name:'AbortError'});expect(paths).not.toContain('/jmap/api');expect(paths).not.toContain('/jmap/upload/b');
});
it('queued message/list rules cannot cross mailbox or same-ID realm generations',()=>{
 activate('a');queueRuleDraft({...emptyRuleDraft(),name:'A sender'});activate('b');expect(takeRuleDraft()).toBeNull();
 queueRuleDraft({...emptyRuleDraft(),name:'B sender'});activateAccountPreferences('b','different_storage_authority_b');expect(takeRuleDraft()).toBeNull();
 queueRuleDraft({...emptyRuleDraft(),name:'Current B'});expect(takeRuleDraft()?.name).toBe('Current B');expect(takeRuleDraft()).toBeNull();
});
it('conflicting active-link and preference owners refuse before sending a rules request',async()=>{
 activate('a');installLinkedMailbox({linkId:'link-b',accountId:'b',storageAuthority:'test_storage_authority_b'});
 const fetch=vi.fn();vi.stubGlobal('fetch',fetch);
 await expect(sieveApi(createClient(),'a').get()).rejects.toMatchObject({name:'AbortError'});
 await expect(vacationApi(createClient(),'a').get()).rejects.toMatchObject({name:'AbortError'});
 expect(fetch).not.toHaveBeenCalled();
});
