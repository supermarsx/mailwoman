import {beforeEach,afterEach,expect,it,vi} from 'vitest';
import {activateAccountPreferences,preferenceAccount,suspendAccountPreferences} from '../api/accountPreferenceLifecycle.ts';
import {configureOfflinePolicy,DEFAULT_OFFLINE_POLICY,reportOffline} from './runtime.ts';
import {synchronizeOffline} from './synchronize.ts';
import type {Client} from '../api/client.ts';
import type {JmapRequest,JmapResponse,JmapSession} from '../api/jmap-types.ts';
import type {SyncPolicy} from './syncPolicy.ts';
vi.mock('./vault.ts',()=>({offlineVault:async()=>({usage:async()=>({bytes:0})})}));
beforeEach(()=>{reportOffline(false);activateAccountPreferences('a','verified_owner_123456');configureOfflinePolicy({...DEFAULT_OFFLINE_POLICY,enabled:true},preferenceAccount()!);});
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
