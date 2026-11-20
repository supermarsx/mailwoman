import {beforeEach,afterEach,expect,it,vi} from 'vitest';
import {activateAccountPreferences,preferenceAccount,suspendAccountPreferences} from '../api/accountPreferenceLifecycle.ts';
import {configureOfflinePolicy,DEFAULT_OFFLINE_POLICY,reportOffline} from './runtime.ts';
import {synchronizeOffline} from './synchronize.ts';
import type {Client} from '../api/client.ts';
import type {JmapRequest,JmapResponse,JmapSession} from '../api/jmap-types.ts';
import type {SyncPolicy} from './syncPolicy.ts';
import {apiFetch} from '../api/http.ts';
vi.mock('../api/http.ts',()=>({apiFetch:vi.fn()}));
vi.mock('./vault.ts',()=>({offlineVault:async()=>({usage:async()=>({bytes:0})})}));
beforeEach(()=>{vi.mocked(apiFetch).mockReset();reportOffline(false);activateAccountPreferences('a','verified_owner_123456');configureOfflinePolicy({...DEFAULT_OFFLINE_POLICY,enabled:true},preferenceAccount()!);});
afterEach(()=>{suspendAccountPreferences();reportOffline(false);});

it.each([
 {name:'body prefetch',bodies:'prefetch',attachments:'on-demand',pinned:false,fetchBodies:true},
 {name:'pinned body read-ahead',bodies:'on-demand',attachments:'on-demand',pinned:true,fetchBodies:true},
 {name:'attachment metadata read-ahead',bodies:'on-demand',attachments:'all',pinned:false,fetchBodies:false},
] as const)('marks actual offline $name requests as background without changing body fetch options',async scenario=>{
 const owner=preferenceAccount()!,signal=new AbortController().signal;
 const jmap=vi.fn(async(request:JmapRequest):Promise<JmapResponse>=>{
  if(request.methodCalls.some(call=>call[2]==='sync-query'))return {sessionState:'s',methodResponses:[
   ['Email/query',{accountId:owner.id,ids:['m1'],position:0,total:1},'sync-query'],
   ['Email/get',{accountId:owner.id,list:[{id:'m1',pinned:scenario.pinned,keywords:{}}],notFound:[]},'sync-headers'],
  ]};
  return {sessionState:'s',methodResponses:request.methodCalls.map(([method,args,tag])=>[method,{accountId:args['accountId'],list:[],notFound:[]},tag])};
 });
 const session={capabilities:{},accounts:{[owner.id]:{accountCapabilities:{}}},downloadUrl:'/jmap/download/{accountId}/{blobId}/{name}'} as unknown as JmapSession;
 const client={session:async()=>session,jmap} as unknown as Client;
 const policy:SyncPolicy={headersWindow:'all',bodies:scenario.bodies,attachments:scenario.attachments};
 await synchronizeOffline(client,owner,policy,signal);
 const calls=jmap.mock.calls.flatMap(([request])=>request.methodCalls);
 const headers=calls.find(call=>call[2]==='sync-headers')![1],body=calls.find(call=>call[2]==='sync-bodies')![1];
 expect(headers['mailwomanBodyUse']).toBe(false);expect(body['mailwomanBodyUse']).toBe(false);
 expect(body['ids']).toEqual(['m1']);expect(body['accountId']).toBe(owner.id);
 if(scenario.fetchBodies){expect(body['fetchHTMLBodyValues']).toBe(true);expect(body['fetchTextBodyValues']).toBe(true);expect(body['maxBodyValueBytes']).toBe(1000000);expect(body['properties']).toContain('bodyValues');}
 else {expect(body['fetchHTMLBodyValues']).toBeUndefined();expect(body['fetchTextBodyValues']).toBeUndefined();expect(body['properties']).toEqual(['id','attachments','bodyStructure']);}
 expect(jmap).toHaveBeenCalledWith(expect.anything(),{signal,pace:'background'});
});

it('downloads an offline attachment as a preview without renewing the message body lease',async()=>{
 const owner=preferenceAccount()!,signal=new AbortController().signal;
 const response=new Response(new Uint8Array([1,2,3]),{headers:{'content-type':'application/pdf'}});
 const cancel=vi.spyOn(response.body!,'cancel');
 vi.mocked(apiFetch).mockResolvedValue(response);
 const jmap=vi.fn(async(request:JmapRequest):Promise<JmapResponse>=>({sessionState:'s',methodResponses:
  request.methodCalls.some(call=>call[2]==='sync-query')?[
   ['Email/query',{accountId:owner.id,ids:['m1'],position:0,total:1},'sync-query'],
   ['Email/get',{accountId:owner.id,list:[{id:'m1',keywords:{}}],notFound:[]},'sync-headers'],
  ]:[['Email/get',{accountId:owner.id,list:[{id:'m1',attachments:[{blobId:'m1.2',name:'statement.pdf',type:'application/pdf',size:3}]}],notFound:[]},'sync-bodies']],
 }));
 const session={capabilities:{},accounts:{[owner.id]:{accountCapabilities:{}}},downloadUrl:'/jmap/download/{accountId}/{blobId}/{name}'} as unknown as JmapSession;
 await synchronizeOffline({session:async()=>session,jmap} as unknown as Client,owner,{headersWindow:'all',bodies:'prefetch',attachments:'all'},signal);
 expect(apiFetch).toHaveBeenCalledTimes(1);
 const [target,options]=vi.mocked(apiFetch).mock.calls[0]!;
 const url=new URL(target);
 expect(url.origin).toBe(location.origin);expect(url.pathname).toBe('/jmap/download/a/m1.2/statement.pdf');
 expect([...url.searchParams.entries()]).toEqual([['emailId','m1'],['preview','true']]);
 expect(options).toEqual({credentials:'same-origin',headers:{'x-mailwoman-body-use':'false'},signal,pace:'background'});
 expect(options?.method).toBeUndefined(); // Preserve the existing GET and preview permission checks.
 const body=jmap.mock.calls.flatMap(([request])=>request.methodCalls).find(call=>call[2]==='sync-bodies')![1];
 expect(body).toMatchObject({mailwomanBodyUse:false,fetchHTMLBodyValues:true,fetchTextBodyValues:true,maxBodyValueBytes:1000000});
 expect(cancel).toHaveBeenCalledOnce();
});
