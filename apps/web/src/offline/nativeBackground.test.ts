import {afterEach,describe,expect,it,vi} from 'vitest';
import {CAP_MAIL,type JmapSession} from '../api/jmap-types.ts';
import {memoryOutboxStore} from './outbox.ts';
import type {QueuedItem} from './outboxCore.ts';
import {nativeBackgroundStartup,replayNativeQueue} from './nativeBackground.ts';
import {observeOfflineChanges} from './events.ts';
const base='https://mail.example.test/mail';
const authority='verified_native_authority_123';
const session={accounts:{a:{}},primaryAccounts:{[CAP_MAIL]:'a'}} as unknown as JmapSession;
const row=(patch:Partial<QueuedItem>={}):QueuedItem=>({id:'pending',createdAt:1,state:'queued',type:'flag',cacheBase:`native:${base}`,linkId:'linked-a',storageAuthority:authority,payload:{accountId:'a',emailId:'email',keyword:'$flagged',value:true},...patch});
const identity={me:{accountId:'a',username:'alice',storageAuthority:authority},session};
afterEach(()=>{vi.unstubAllGlobals();});
describe('native background replay authority',()=>{
 it('boots only a shell explicitly launched for background replay',()=>{
  vi.stubGlobal('__MW_CONFIG__',{native:false,backgroundReplay:true});expect(nativeBackgroundStartup()).toBe(false);
  vi.stubGlobal('__MW_CONFIG__',{native:true,backgroundReplay:true});expect(nativeBackgroundStartup()).toBe(true);
  vi.stubGlobal('__MW_CONFIG__',{native:true});expect(nativeBackgroundStartup()).toBe(false);
 });
 it('verifies a linked principal before replaying only its configured-server rows',async()=>{
  const legacy=row({id:'legacy'});delete legacy.cacheBase;
  const store=memoryOutboxStore([row(),row({id:'other-server',cacheBase:'native:https://other.test'}),legacy]);
  const verify=vi.fn(async()=>identity),run=vi.fn(async(_row,_identity,selected)=>{const all=await selected.all();expect(all.map((item:QueuedItem)=>item.id)).toEqual(['pending']);await selected.delete('pending');});
  expect(await replayNativeQueue({base,store,verify,run})).toBe(true);
  expect(verify).toHaveBeenCalledExactlyOnceWith('linked-a');expect(run).toHaveBeenCalledTimes(1);
  expect((await store.all()).map(item=>item.id)).toEqual(['other-server','legacy']);
 });
 it.each(['principal','authority','discovery','password'] as const)('keeps pending data on a changed %s',async field=>{
  const store=memoryOutboxStore([row()]),run=vi.fn(async()=>undefined);
  const changed=structuredClone(identity);
  if(field==='principal')changed.me.accountId='b';
  if(field==='authority')changed.me.storageAuthority='other_verified_authority_123';
  if(field==='discovery')changed.session.primaryAccounts[CAP_MAIL]='b';
  if(field==='password')Object.assign(changed.me,{passwordChangeRequired:true});
  expect(await replayNativeQueue({base,store,verify:async()=>changed,run})).toBe(false);
  expect(run).not.toHaveBeenCalled();expect(await store.all()).toHaveLength(1);
 });
 it('propagates verification outages without deleting pending work',async()=>{
  const store=memoryOutboxStore([row()]),run=vi.fn(async()=>undefined);
  await expect(replayNativeQueue({base,store,verify:async()=>{throw new Error('offline');},run})).rejects.toThrow('offline');
  expect(run).not.toHaveBeenCalled();expect(await store.all()).toHaveLength(1);
 });
 it('leaves quarantined native rows on disk without keeping an unserviceable wake job alive',async()=>{
  const missingLink=row({id:'missing-link'});delete missingLink.linkId;
  const missingAuthority=row({id:'missing-authority'});delete missingAuthority.storageAuthority;
  const invalidAuthority=row({id:'invalid-authority',storageAuthority:'bad'});
  const mixed=row({id:'mixed',type:'pim',payload:{callId:'a',request:{using:[],methodCalls:[['Note/set',{accountId:'a',update:{one:{title:'A'}}},'a'],['Task/set',{accountId:'b',update:{two:{title:'B'}}},'b']]}}});
  const store=memoryOutboxStore([row(),missingLink,missingAuthority,invalidAuthority,mixed]),verify=vi.fn(async()=>identity);
  const run=vi.fn(async(_row,_identity,selected)=>{expect((await selected.all()).map((item:QueuedItem)=>item.id)).toEqual(['pending']);await selected.delete('pending');});
  expect(await replayNativeQueue({base,store,verify,run})).toBe(true);
  expect(verify).toHaveBeenCalledOnce();expect(run).toHaveBeenCalledOnce();expect((await store.all()).map(item=>item.id)).toEqual(['missing-link','missing-authority','invalid-authority','mixed']);
 });
 it('does not call completion successful when replay has no acknowledgement',async()=>{
  const store=memoryOutboxStore([row()]),run=vi.fn(async()=>undefined);
  expect(await replayNativeQueue({base,store,verify:async()=>identity,run})).toBe(false);
  expect(run).toHaveBeenCalledTimes(1);expect(await store.all()).toHaveLength(1);
 });
 it('invalidates foreground queue readers after a verified drain, including partial failures',async()=>{
  const changed=vi.fn(),off=observeOfflineChanges(changed),store=memoryOutboxStore([row()]);
  try{
   await expect(replayNativeQueue({base,store,verify:async()=>identity,run:async()=>{await store.delete('pending');throw new Error('later context unavailable');}})).rejects.toThrow('later context unavailable');
   expect(changed).toHaveBeenCalledExactlyOnceWith('queue');expect(await store.all()).toEqual([]);
  }finally{off();}
 });
});
