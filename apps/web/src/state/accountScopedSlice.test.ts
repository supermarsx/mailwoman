import {createRoot,createSignal} from 'solid-js';
import {afterEach,describe,expect,it,vi} from 'vitest';
import {accountScopedSlice} from './accountScopedSlice.ts';
import {createOutboxSlice} from './slices/outbox.ts';
import {activateAccountPreferences,suspendAccountPreferences} from '../api/accountPreferenceLifecycle.ts';
import {CAP_MAIL,type JmapResponse,type JmapSession} from '../api/jmap-types.ts';
import type {Client} from '../api/client.ts';
afterEach(suspendAccountPreferences);
const session:JmapSession={capabilities:{},accounts:{same:{name:'Mailbox',isPersonal:true,isReadOnly:false,accountCapabilities:{}}},primaryAccounts:{[CAP_MAIL]:'same'},username:'mail@example.org',apiUrl:'/jmap/api',downloadUrl:'/d',uploadUrl:'/u',eventSourceUrl:'/e',state:'s'};
function client():Client{return {login:vi.fn(),logout:vi.fn(),me:vi.fn(),session:vi.fn(async()=>session),jmap:vi.fn(),sanitize:vi.fn(),onNetwork:vi.fn(()=>()=>undefined)};}
describe('private slice authority lifetime',()=>{
  it('clears cached identities and resolves discovery again for same-ID different realms',async()=>{
    activateAccountPreferences('same','msa1_parent_verified');
    const transport=client();
    vi.mocked(transport.jmap).mockResolvedValueOnce({methodResponses:[['Identity/get',{list:[{id:'parent-only',email:'parent@example.org',name:'Parent'}]},'i']],sessionState:'p'});
    await createRoot(async dispose=>{try{
      const outbox=accountScopedSlice({client:transport,showToast:vi.fn()},createOutboxSlice);
      await outbox.loadIdentities();
      expect(outbox.identities()[0]?.id).toBe('parent-only');
      activateAccountPreferences('same','msa1_child_verified');
      expect(outbox.identities()).toEqual([]);
      vi.mocked(transport.jmap).mockResolvedValueOnce({methodResponses:[['Identity/get',{list:[{id:'child-only',email:'child@example.org',name:'Child'}]},'i']],sessionState:'c'});
      await outbox.loadIdentities();
      expect(outbox.identities()[0]?.id).toBe('child-only');
      expect(transport.session).toHaveBeenCalledTimes(2);
    }finally{dispose();}});
  });
  it('prevents an old async action from dispatching after its authority is disposed',async()=>{
    activateAccountPreferences('same','msa1_parent_verified');
    const transport=client();
    let resume!:()=>void;
    const wait=new Promise<void>(resolve=>{resume=resolve;});
    await createRoot(async dispose=>{try{
      const state=accountScopedSlice({client:transport,showToast:vi.fn()},ctx=>{
        const [value,setValue]=createSignal('');
        return {value,async action(){await wait;const result=await ctx.client.jmap({using:[],methodCalls:[]});setValue(result.sessionState);}};
      });
      const action=state.action();
      const refused=expect(action).rejects.toMatchObject({name:'AbortError'});
      activateAccountPreferences('same','msa1_child_verified');resume();await refused;
      expect(transport.jmap).not.toHaveBeenCalled();expect(state.value()).toBe('');
    }finally{dispose();}});
  });
  it('drops a late response rather than publishing it into a fresh cache',async()=>{
    activateAccountPreferences('same','msa1_parent_verified');
    const transport=client();let release!:(result:JmapResponse)=>void;
    vi.mocked(transport.jmap).mockImplementation(()=>new Promise(resolve=>{release=resolve;}));
    await createRoot(async dispose=>{try{
      const outbox=accountScopedSlice({client:transport,showToast:vi.fn()},createOutboxSlice);
      const loading=outbox.loadIdentities();await vi.waitFor(()=>expect(transport.jmap).toHaveBeenCalledOnce());
      activateAccountPreferences('same','msa1_child_verified');
      release({methodResponses:[['Identity/get',{list:[{id:'foreign'}]},'i']],sessionState:'old'});
      await loading;expect(outbox.identities()).toEqual([]);
    }finally{dispose();}});
  });
});
