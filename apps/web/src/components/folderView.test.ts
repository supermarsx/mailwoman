import {createRoot} from 'solid-js';
import {afterEach,expect,it,vi} from 'vitest';
import {activateAccountPreferences,suspendAccountPreferences} from '../api/accountPreferenceLifecycle.ts';
import {createFolderView} from './folderView.ts';
import type {Client} from '../api/client.ts';
import type {JmapRequest,JmapResponse} from '../api/jmap-types.ts';
afterEach(suspendAccountPreferences);
it('queries saved folder IDs through the same server-owned inMailbox expansion as the main list',async()=>{
 activateAccountPreferences('same','msa1_verified_parent');
 const requests:JmapRequest[]=[];const jmap=vi.fn(async(request:JmapRequest)=>{requests.push(request);return result('saved-');});
 await createRoot(async dispose=>{try{
  const view=createFolderView({jmap} as unknown as Client,'saved-query-id',()=>true);await view.load();
  expect(requests[0]?.methodCalls[0]?.[1]).toMatchObject({accountId:'same',filter:{inMailbox:'saved-query-id'}});
  expect(view.messages()).toHaveLength(3);
 }finally{dispose();}});
});
function result(prefix:string,position=0,total:number|undefined=240):JmapResponse{return {sessionState:'s',methodResponses:[['Email/query',{accountId:'same',ids:[prefix+'1',prefix+'2',prefix+'3'],position,...(total===undefined?{}:{total})},'q'],['Email/get',{accountId:'same',list:[1,2,3].map(i=>({id:prefix+i,subject:prefix+i,mailboxIds:{[prefix]:true},keywords:{}})),notFound:[],state:'s'},'g']]};}
it('retains independent folder page, selection and scroll without changing another view',async()=>{
 activateAccountPreferences('same','msa1_verified_parent');
 const jmap=vi.fn(async()=>result('a'));
 await createRoot(async dispose=>{try{
  const a=createFolderView({jmap} as unknown as Client,'a',()=>true),b=createFolderView({jmap} as unknown as Client,'b',()=>true);
  await a.load();a.select('a1');a.select('a3',true);a.setScroll(320);
  jmap.mockResolvedValueOnce(result('b'));await b.load();b.select('b2');b.setScroll(40);
  expect([...a.selection()]).toEqual(['a1','a2','a3']);expect([...b.selection()]).toEqual(['b2']);expect(a.scroll()).toBe(320);
  jmap.mockResolvedValueOnce(result('b',100));await b.load(100);expect(b.position()).toBe(100);expect(b.scroll()).toBe(0);expect(b.selection().size).toBe(0);expect(a.position()).toBe(0);
  expect(jmap.mock.calls).toHaveLength(3);
 }finally{dispose();}});
});
it('aborts and discards late data after same-ID realm switching',async()=>{
 activateAccountPreferences('same','msa1_verified_parent');let release!:(response:JmapResponse)=>void;
 const jmap=vi.fn(()=>new Promise<JmapResponse>(resolve=>{release=resolve;}));
 await createRoot(async dispose=>{try{
  const view=createFolderView({jmap} as unknown as Client,'a',()=>true);const loading=view.load();
  activateAccountPreferences('same','msa1_verified_child');release(result('private'));await loading;
  expect(view.messages()).toEqual([]);expect(view.current()).toBe(false);await view.load();expect(jmap).toHaveBeenCalledOnce();
 }finally{dispose();}});
});
it('refuses a removed mailbox and preserves unknown totals honestly',async()=>{
 activateAccountPreferences('same','msa1_verified_parent');let allowed=true;
 const response=result('a');delete (response.methodResponses[0]![1] as {total?:number}).total;
 const jmap=vi.fn(async()=>response);
 await createRoot(async dispose=>{try{
  const view=createFolderView({jmap} as unknown as Client,'a',()=>allowed);await view.load();expect(view.total()).toBeNull();
  allowed=false;await view.load();expect(view.messages()).toEqual([]);expect(jmap).toHaveBeenCalledOnce();
 }finally{dispose();}});
});
