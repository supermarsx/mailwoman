import {describe,it,expect,vi} from 'vitest';
import {CAP_MAIL,type JmapSession} from './jmap-types.ts';
import {createLinkedAccountSelection,validateLinkedAccountContext,type LinkedAccountContext} from './linkedAccountContext.ts';
import type {Me} from './client.ts';

const context=(linkId='child'):LinkedAccountContext=>({linkId,accountId:'1',storageAuthority:`msa1_verified_${linkId}`});
const me=(owner=context()):Me=>({username:'child@example.org',accountId:owner.accountId,storageAuthority:owner.storageAuthority});
const session=():JmapSession=>({capabilities:{},accounts:{'1':{name:'Child',isPersonal:true,isReadOnly:false,accountCapabilities:{}}},primaryAccounts:{[CAP_MAIL]:'1'},username:'child@example.org',apiUrl:'/jmap/api',downloadUrl:'/download',uploadUrl:'/upload',eventSourceUrl:'/events',state:'s'});
function harness(){
  const deps={authorize:vi.fn(async(id:string,_signal:AbortSignal)=>context(id)),me:vi.fn(async(owner:LinkedAccountContext,_signal:AbortSignal)=>me(owner)),session:vi.fn(async(_owner:LinkedAccountContext,_signal:AbortSignal)=>session()),publish:vi.fn(),isCurrent:vi.fn(()=>true)};
  return {deps,selection:createLinkedAccountSelection(deps)};
}
describe('linked mailbox selection',()=>{
  it('publishes only after effective identity and discovery both verify',async()=>{
    const {deps,selection}=harness();
    await selection.select('child');
    expect(deps.publish).toHaveBeenCalledExactlyOnceWith(context(),me(),session());
  });
  it('refuses the same raw ID with a different storage authority',async()=>{
    const {deps,selection}=harness();
    deps.me.mockResolvedValue(me(context('foreign')));
    await expect(selection.select('child')).rejects.toMatchObject({code:'linkedAccountMismatch'});
    expect(deps.publish).not.toHaveBeenCalled();
  });
  it('rejects a response that selects a different link',async()=>{
    const {deps,selection}=harness();
    deps.authorize.mockResolvedValue(context('foreign'));
    await expect(selection.select('child')).rejects.toMatchObject({code:'linkedAccountMismatch'});
    expect(deps.me).not.toHaveBeenCalled();
  });
  it('does not publish a late selection after a newer one completes',async()=>{
    const {deps,selection}=harness();
    let release!:(value:Me)=>void;
    deps.me.mockImplementationOnce(()=>new Promise(resolve=>{release=resolve;}));
    const old=selection.select('old');
    const oldResult=expect(old).rejects.toMatchObject({code:'accountChanged'});
    await vi.waitFor(()=>expect(deps.me).toHaveBeenCalledOnce());
    await selection.select('new');
    release(me(context('old')));
    await oldResult;
    expect(deps.publish).toHaveBeenCalledTimes(1);
    expect(deps.publish.mock.calls[0]?.[0]).toEqual(context('new'));
  });
  it('drops a delayed discovery after parent authority changes',async()=>{
    const {deps,selection}=harness();
    deps.session.mockImplementation(async()=>{deps.isCurrent.mockReturnValue(false);return session();});
    await expect(selection.select('child')).rejects.toMatchObject({code:'accountChanged'});
    expect(deps.publish).not.toHaveBeenCalled();
  });
  it('does not fall back or retry after an authorization failure',async()=>{
    const {deps,selection}=harness();
    deps.authorize.mockRejectedValue(new Error('expired link'));
    await expect(selection.select('child')).rejects.toThrow('expired link');
    expect(deps.authorize).toHaveBeenCalledOnce();
    expect(deps.me).not.toHaveBeenCalled();
    expect(deps.publish).not.toHaveBeenCalled();
  });
  it('refuses missing discovery ownership and held password changes',()=>{
    expect(()=>validateLinkedAccountContext(context(),me(),{...session(),accounts:{}})).toThrow();
    expect(()=>validateLinkedAccountContext(context(),me(),{...session(),primaryAccounts:{[CAP_MAIL]:'other'}})).toThrow();
    expect(()=>validateLinkedAccountContext(context(),{...me(),passwordChangeRequired:true},session())).toThrow();
  });
});
