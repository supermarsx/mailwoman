import {beforeEach,expect,it,vi} from 'vitest';
import {pendingId,pendingResponse,projectPending,recordPendingIds,resolvePendingIds} from './pendingObjects.ts';
import type {QueuedItem} from './outboxCore.ts';
import type {JmapRequest} from '../api/jmap-types.ts';
const records=vi.hoisted(()=>new Map<string,unknown>());
vi.mock('./vault.ts',()=>({offlineVault:async()=>({get:async(namespace:string,key:string)=>records.get(namespace+key),put:async(namespace:string,key:string,value:unknown)=>{records.set(namespace+key,value);return true;}})}));
beforeEach(()=>records.clear());
const create:QueuedItem={id:'queue',type:'pim',storageAuthority:'verified_owner_123456',cacheBase:'https://mail.example',createdAt:1,state:'queued',payload:{request:{using:[],methodCalls:[['Note/set',{accountId:'a',create:{new:{title:'Offline note'}}},'set']]},callId:'set'}};
it('creates, edits and deletes provisional objects without modifying the last server snapshot',()=>{
 const id=pendingId('queue','new','Note');
 const update:QueuedItem={...create,id:'update',payload:{request:{using:[],methodCalls:[['Note/set',{accountId:'a',update:{[id]:{title:'Edited offline'}}},'set']]},callId:'set'}};
 const response=pendingResponse(create);expect(response.methodResponses[0]?.[1]['mailwomanQueued']).toBe(true);
 const rows=projectPending([],'Note','a',[create,update]);expect(rows).toHaveLength(1);expect(rows[0]).toMatchObject({id,title:'Edited offline',mailwomanQueued:true,bodyHtml:'',tags:[]});
 const destroy:QueuedItem={...create,id:'delete',payload:{request:{using:[],methodCalls:[['Note/set',{accountId:'a',destroy:[id]},'set']]},callId:'set'}};
 expect(projectPending([],'Note','a',[create,update,destroy])).toEqual([]);
 expect(projectPending([],'Note','other',[create])).toEqual([]);
});
it('translates temporary identifiers in both object keys and relationship values after creation',async()=>{
 const id=pendingId('queue','new','Note');
 await recordPendingIds(create,{sessionState:'s',methodResponses:[['Note/set',{created:{new:{id:'server-note'}}},'set']]});
 const request={using:[],methodCalls:[['Note/set',{accountId:'a',update:{[id]:{title:'Next edit',links:[{type:'note',id}]}}},'set']]} as JmapRequest;
 const resolved=await resolvePendingIds(create,request);
 expect(resolved.methodCalls[0]?.[1]['update']).toEqual({'server-note':{title:'Next edit',links:[{type:'note',id:'server-note'}]}});
 await expect(resolvePendingIds({...create,storageAuthority:'different_owner_123456'},request)).rejects.toThrow('earlier queued item');
});
it('projects pending moves before folder filtering and drops failed flag projections',()=>{
 const source={id:'m',mailboxIds:{inbox:true},keywords:{}};
 const move:QueuedItem={id:'move',type:'move',state:'queued',createdAt:1,payload:{accountId:'a',emailId:'m',mailboxIds:{archive:true}}};
 expect(projectPending([source],'Email','a',[move])[0]?.['mailboxIds']).toEqual({archive:true});expect(source.mailboxIds).toEqual({inbox:true});
 expect(projectPending([source],'Email','a',[{...move,state:'failed'}])[0]).toBe(source);
});
