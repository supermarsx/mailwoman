import {expect,it,vi} from 'vitest';
import type {Client} from '../api/client.ts';
import {drainOutbox,memoryOutboxStore,type QueuedItem} from './outbox.ts';
const queued=(type:'flag'|'move',payload:object):QueuedItem=>({id:'stable',storageAuthority:'verified_owner_123456',type,payload:{accountId:'a',emailId:'m',...payload},createdAt:1,state:'queued'});
const get=(keywords={},mailboxIds={inbox:true})=>({sessionState:'s',methodResponses:[['Email/get',{accountId:'a',state:'latest',list:[{id:'m',keywords,mailboxIds}]},'baseline']]});
it('keeps a server flag changed since the offline baseline without sending a set',async()=>{
 const store=memoryOutboxStore([queued('flag',{keyword:'$seen',value:false,previousValue:false})]);
 const jmap=vi.fn().mockResolvedValue(get({$seen:true}));
 expect(await drainOutbox(store,{jmap} as unknown as Client)).toEqual({sent:0,failed:1});
 expect(jmap).toHaveBeenCalledTimes(1);expect((await store.all())[0]).toMatchObject({state:'failed',lastError:expect.stringContaining('stateMismatch')});
});
it('does not repeat an already applied move',async()=>{
 const store=memoryOutboxStore([queued('move',{mailboxIds:{archive:true},previousMailboxIds:{inbox:true}})]);
 const jmap=vi.fn().mockResolvedValue(get({},{archive:true} as unknown as {inbox:boolean}));
 expect(await drainOutbox(store,{jmap} as unknown as Client)).toEqual({sent:1,failed:0});expect(jmap).toHaveBeenCalledTimes(1);
});
it('guards an unchanged object with the fresh server state before setting it',async()=>{
 const store=memoryOutboxStore([queued('flag',{keyword:'$seen',value:true,previousValue:false})]);
 const jmap=vi.fn().mockResolvedValueOnce(get()).mockResolvedValueOnce({sessionState:'s',methodResponses:[['Email/set',{updated:{m:null},notUpdated:null},'set']]});
 expect(await drainOutbox(store,{jmap} as unknown as Client)).toEqual({sent:1,failed:0});
 expect(jmap.mock.calls[1]?.[0].methodCalls[0][1]).toMatchObject({ifInState:'latest'});
});
it('reuses a queued send identity after a transport failure',async()=>{
 const store=memoryOutboxStore([{id:'stable',type:'send',state:'queued',createdAt:1,payload:{accountId:'a',draft:{from:{email:'a@example.test'},to:'b@example.test',subject:'Hello',htmlBody:'x',draftMailboxId:'drafts'}}}]);
 const jmap=vi.fn().mockRejectedValue(new TypeError('lost reply'));
 await drainOutbox(store,{jmap} as unknown as Client);await drainOutbox(store,{jmap} as unknown as Client);
 expect(jmap.mock.calls[0]?.[1].idempotencyKey).toBe(jmap.mock.calls[1]?.[1].idempotencyKey);
});

it('keeps queued work when a client mistakenly returns an offline acknowledgement',async()=>{
 const store=memoryOutboxStore([{id:'pim',type:'pim',createdAt:1,state:'queued',payload:{request:{using:[],methodCalls:[['Note/set',{accountId:'a',update:{note:{title:'Edit'}}},'set']]},callId:'set'}}]);
 const jmap=vi.fn().mockResolvedValue({sessionState:'offline-pending',methodResponses:[['Note/set',{updated:{note:null},mailwomanQueued:true},'set']]});
 expect(await drainOutbox(store,{jmap} as unknown as Client)).toEqual({sent:0,failed:0});expect((await store.all())[0]?.state).toBe('queued');
});
