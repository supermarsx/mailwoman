import {afterEach,describe,expect,it,vi} from 'vitest';
import {createRoot,createSignal} from 'solid-js';
import type {Client} from '../../api/client.ts';
import type {JmapRequest,JmapResponse} from '../../api/jmap-types.ts';
import {activateAccountPreferences,suspendAccountPreferences} from '../../api/accountPreferenceLifecycle.ts';
import {createRulesSlice,emptyRuleDraft,RuleRefusedError} from './rules.ts';
import type {SliceContext} from './context.ts';
const rule=(account:string)=>({id:`rule-${account}`,name:account,enabled:true,matchAll:true,conditions:[],actions:[],runsAt:'engine'});
const get=(account:string):JmapResponse=>({sessionState:'s',methodResponses:[['MailRule/get',{accountId:account,state:`state-${account}`,list:[rule(account)],notFound:[]},'g']]});
const set=(account:string):JmapResponse=>({sessionState:'s',methodResponses:[['MailRule/set',{accountId:account,created:{new:{id:'new'}},updated:{},destroyed:[]},'s']]});
function context(jmap:(body:JmapRequest)=>Promise<JmapResponse>):SliceContext {
 return {client:{jmap,session:vi.fn(()=>{throw Error('selected mailbox must not fall back to primary discovery');})} as unknown as Client,showToast:vi.fn(),broadcastChange:vi.fn()} as unknown as SliceContext;
}
afterEach(()=>suspendAccountPreferences());
describe('rules mailbox generations',()=>{
 it('does not send a selected-account request under another verified preference owner',async()=>{
  activateAccountPreferences('b','test_storage_authority_b');
  await createRoot(async dispose=>{
   try{
    const jmap=vi.fn(async()=>get('a')),slice=createRulesSlice(context(jmap),()=> 'a');
    await slice.loadRules();await expect(slice.saveRule({...emptyRuleDraft(),name:'Wrong owner'})).rejects.toMatchObject({name:'AbortError'});
    expect(jmap).not.toHaveBeenCalled();expect(slice.rules()).toEqual([]);
   }finally{dispose();}
  });
 });
 it('drops old loads and clears rows immediately on A to B and same-ID realm changes',async()=>{
  activateAccountPreferences('a','test_storage_authority_a');
  await createRoot(async dispose=>{
   try{
    const [account,select]=createSignal('a');let finish!:(value:JmapResponse)=>void;
    const jmap=vi.fn(async(body:JmapRequest)=>body.methodCalls[0]?.[1]['accountId']==='a'?await new Promise<JmapResponse>(resolve=>{finish=resolve;}):get('b'));
    const slice=createRulesSlice(context(jmap),account);
    const old=slice.loadRules();await vi.waitFor(()=>expect(jmap).toHaveBeenCalledOnce());
    select('b');activateAccountPreferences('b','test_storage_authority_b');
    expect(slice.rules()).toEqual([]);await slice.loadRules();expect(slice.rules()[0]?.name).toBe('b');
    finish(get('a'));await old;expect(slice.rules()[0]?.name).toBe('b');
    activateAccountPreferences('b','test_storage_authority_replaced_b');expect(slice.rules()).toEqual([]);
   }finally{dispose();}
  });
 });
 it('does not reload or announce an A mutation after switching to B',async()=>{
  activateAccountPreferences('a','test_storage_authority_a');
  await createRoot(async dispose=>{
   try{
    const [account,select]=createSignal('a');let finish!:(value:JmapResponse)=>void;
    const jmap=vi.fn(async(body:JmapRequest)=>{
     const [method,args]=body.methodCalls[0]!;
     return method==='MailRule/set'?await new Promise<JmapResponse>(resolve=>{finish=resolve;}):get(String(args['accountId']));
    });
    const ctx=context(jmap),slice=createRulesSlice(ctx,account);await slice.loadRules();
    const save=slice.saveRule({...emptyRuleDraft(),name:'A draft'});
    await vi.waitFor(()=>expect(jmap.mock.calls.some(([body])=>body.methodCalls[0]?.[0]==='MailRule/set')).toBe(true));
    select('b');activateAccountPreferences('b','test_storage_authority_b');await slice.loadRules();
    const calls=jmap.mock.calls.length;finish(set('a'));await save;
    expect(jmap.mock.calls.length).toBe(calls);expect(slice.rules()[0]?.name).toBe('b');
    expect(ctx.showToast).not.toHaveBeenCalled();expect(ctx.broadcastChange).not.toHaveBeenCalled();
   }finally{dispose();}
  });
 });
 it('rejects mismatched reads and refused deletes without reporting success',async()=>{
  activateAccountPreferences('a','test_storage_authority_a');
  await createRoot(async dispose=>{
   try{
    let foreign=false;
    const jmap=vi.fn(async(body:JmapRequest)=>body.methodCalls[0]?.[0]==='MailRule/get'?get(foreign?'b':'a'):
     {sessionState:'s',methodResponses:[['MailRule/set',{accountId:'a',notDestroyed:{'rule-a':{type:'forbidden'}}},'s']]} as JmapResponse);
    const ctx=context(jmap),slice=createRulesSlice(ctx,()=> 'a');await slice.loadRules();
    await expect(slice.deleteRule('rule-a')).rejects.toBeInstanceOf(RuleRefusedError);
    expect(slice.rules()[0]?.name).toBe('a');expect(ctx.showToast).not.toHaveBeenCalledWith('success',expect.anything());
    foreign=true;await expect(slice.loadRules()).rejects.toThrow('another mailbox');expect(slice.rules()).toEqual([]);
   }finally{dispose();}
  });
 });
});
