import { afterEach, expect, it, vi } from 'vitest';
import { createRoot, createSignal } from 'solid-js';
import { makeClient, mkEmail } from '../../components/appHarness.tsx';
import { createMailSlice, type MailSlice } from './mail.ts';
import type { Email, JmapRequest, JmapResponse } from '../../api/jmap-types.ts';
import { toast } from '../../components/toast/toast.ts';
import { clearUndoHistory } from '../undoHistory.ts';

const DONE='mw-followup-completed',OLD='2026-11-20T12:00:00Z',NEXT='2026-12-01T09:00:00Z';
const disposals:(()=>void)[]=[];
afterEach(()=>{for(const dispose of disposals.splice(0))dispose();toast.clear();clearUndoHistory();});
async function fixture(emails:Email[],options:{held?:boolean;failId?:string;queueFailure?:boolean}={}){
  const client=makeClient({emails}),base=client.jmap,writes:{update:Record<string,Record<string,unknown>>;settle:(fail?:boolean)=>void}[]=[];
  client.jmap=vi.fn(async(request:JmapRequest):Promise<JmapResponse>=>{
    const set=request.methodCalls.find(([name])=>name==='Email/set');if(!set)return base(request);
    const update=(set[1]as{update:Record<string,Record<string,unknown>>}).update;
    return new Promise(resolve=>{const settle=(fail=false)=>resolve({methodResponses:[['Email/set',{accountId:'acct1',...(fail?{notUpdated:Object.fromEntries(Object.keys(update).map(id=>[id,{type:'forbidden'}]))}:{updated:Object.fromEntries(Object.keys(update).map(id=>[id,null]))})},set[2]]],sessionState:'s'});
      writes.push({update,settle});if(!options.held)settle(options.failId!==undefined&&options.failId in update);
    });
  });
  const [online,setOnline]=createSignal(true),showToast=vi.fn();
  const enqueueOffline=vi.fn(async()=>{if(options.queueFailure)throw new Error('queue failed');return 'queued';});
  let mail!:MailSlice;createRoot(dispose=>{disposals.push(dispose);mail=createMailSlice({client,online,showToast,enqueueOffline});});
  await mail.login({username:'me@example.org',password:'p'});
  return{mail,writes,setOnline,enqueueOffline,showToast};
}
it('does not roll back a newer reminder when an older completion request fails',async()=>{
  const {mail,writes}=await fixture([mkEmail('a',{followUpAt:OLD,keywords:{$flagged:true}})],{held:true});
  const old=mail.completeFollowUp('a').then(()=>null,error=>error);expect(writes).toHaveLength(1);
  const newer=mail.setFollowUp('a',NEXT);expect(writes).toHaveLength(2);writes[1]!.settle();await newer;
  writes[0]!.settle(true);expect(await old).toBeInstanceOf(Error);
  expect(mail.messages()[0]!.followUpAt).toBe(NEXT);expect(mail.messages()[0]!.keywords?.[DONE]).not.toBe(true);expect(mail.messages()[0]!.keywords?.['$flagged']).toBe(true);
});
it('does not continue a bulk restart across logout/login when message IDs are reused',async()=>{
  const {mail,writes}=await fixture([mkEmail('a',{keywords:{[DONE]:true}}),mkEmail('b',{keywords:{[DONE]:true}})],{held:true});
  const pending=mail.flagMessage(['a','b'],true);expect(writes).toHaveLength(1);
  await mail.logout();await mail.login({username:'me@example.org',password:'p'});writes[0]!.settle();await pending;
  expect(writes).toHaveLength(1);expect(mail.messages().every(email=>email.keywords?.[DONE]===true)).toBe(true);expect(mail.pendingUndo()).toBeNull();
});
it('queues both durable keyword changes for offline restart and keeps completion refusal unchanged',async()=>{
  const {mail,writes,setOnline,enqueueOffline}=await fixture([mkEmail('a',{keywords:{[DONE]:true}})]);setOnline(false);
  await mail.flagMessage('a',true);expect(writes).toHaveLength(0);
  expect(enqueueOffline).toHaveBeenNthCalledWith(1,'flag',{accountId:'acct1',emailId:'a',keyword:DONE,value:false});
  expect(enqueueOffline).toHaveBeenNthCalledWith(2,'flag',{accountId:'acct1',emailId:'a',keyword:'$flagged',value:true});
  await expect(mail.completeFollowUp('a')).rejects.toThrow();expect(mail.messages()[0]!.keywords?.[DONE]).not.toBe(true);expect(mail.messages()[0]!.keywords?.['$flagged']).toBe(true);
});
it('retains Undo for successful bulk restarts when another message refuses the update',async()=>{
  const {mail,writes,showToast}=await fixture([mkEmail('a',{keywords:{[DONE]:true}}),mkEmail('b',{keywords:{[DONE]:true}})],{failId:'b'});
  await mail.flagMessage(['a','b'],true);expect(writes).toHaveLength(2);expect(showToast).toHaveBeenCalledWith('error',expect.any(String));
  expect(mail.messages()[0]!.keywords?.[DONE]).not.toBe(true);expect(mail.messages()[1]!.keywords?.[DONE]).toBe(true);expect(mail.pendingUndo()).not.toBeNull();
  await mail.undoNow();expect(mail.messages().every(email=>email.keywords?.[DONE]===true)).toBe(true);expect(writes).toHaveLength(3);
});
it('restores optimistic state when an offline queue write fails',async()=>{
  const {mail,setOnline}=await fixture([mkEmail('a',{keywords:{[DONE]:true},followUpAt:OLD})],{queueFailure:true});setOnline(false);
  await expect(mail.setFollowUp('a',NEXT)).rejects.toThrow('queue failed');
  expect(mail.messages()[0]!.keywords?.[DONE]).toBe(true);expect(mail.messages()[0]!.followUpAt).toBe(OLD);
});
