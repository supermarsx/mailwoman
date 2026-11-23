import {afterEach,beforeEach,expect,it,vi} from 'vitest';
import {createStore} from 'solid-js/store';
import {withSharedNetwork} from './networkProxy.ts';
import type {Client} from '../api/client.ts';
import type {JmapRequest} from '../api/jmap-types.ts';
import {clearKeptKeys} from '../api/recovery.ts';
import {activateAccountPreferences,suspendAccountPreferences} from '../api/accountPreferenceLifecycle.ts';
const fixture=vi.hoisted(()=>({call:vi.fn()}));
vi.mock('./networkChannel.ts',()=>({createChannelNetworkConnection:()=>({mode:'leader',ready:Promise.resolve(true),call:fixture.call,subscribe:()=>()=>{}})}));
beforeEach(()=>{vi.stubGlobal('SharedWorker',undefined);fixture.call.mockReset().mockResolvedValue({methodResponses:[],sessionState:'s'});activateAccountPreferences('a','owner-authority-a');});
afterEach(()=>{clearKeptKeys();suspendAccountPreferences();vi.unstubAllGlobals();});
const request=(conditions:unknown):JmapRequest=>({using:[],methodCalls:[['MailRule/set',{accountId:'a',create:{new:{conditions}}},'s']]});
const wrapped=()=>withSharedNetwork({jmap:vi.fn()} as unknown as Client);
it('rejects, naming the method, a payload that holds a Solid store proxy, and does not send it',async()=>{
 const [store]=createStore({conditions:[{type:'from',op:'contains',value:'x'}]});
 await expect(wrapped().jmap(request(store.conditions))).rejects.toThrow(/Network call "jmap".*cannot be structured-cloned/);
 expect(fixture.call).not.toHaveBeenCalled();
});
it('passes a plain payload through unchanged',async()=>{
 await wrapped().jmap(request([{type:'from',op:'contains',value:'x'}]));
 expect(fixture.call).toHaveBeenCalledTimes(1);
});
