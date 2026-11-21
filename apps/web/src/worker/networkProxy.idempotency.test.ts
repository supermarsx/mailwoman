import {afterEach,beforeEach,expect,it,vi} from 'vitest';
import {withSharedNetwork} from './networkProxy.ts';
import {ApiError,type Client} from '../api/client.ts';
import type {JmapRequest} from '../api/jmap-types.ts';
import {clearKeptKeys} from '../api/recovery.ts';
import {activateAccountPreferences,suspendAccountPreferences} from '../api/accountPreferenceLifecycle.ts';
const fixture=vi.hoisted(()=>({call:vi.fn()}));
vi.mock('./networkChannel.ts',()=>({createChannelNetworkConnection:()=>({mode:'leader',ready:Promise.resolve(true),call:fixture.call,subscribe:()=>()=>{}})}));
beforeEach(()=>{vi.stubGlobal('SharedWorker',undefined);fixture.call.mockReset().mockResolvedValue({methodResponses:[],sessionState:'s'});activateAccountPreferences('a','owner-authority-a');});
afterEach(()=>{clearKeptKeys();suspendAccountPreferences();vi.unstubAllGlobals();});
const envelope=():JmapRequest=>({using:[],methodCalls:[['Task/set',{accountId:'a',create:{task:{title:'Only once'}}},'create']]});
const wrapped=()=>withSharedNetwork({jmap:vi.fn()} as unknown as Client);
it('keeps the same create key across a lost acknowledgement and creates a fresh key only after confirmation',async()=>{
 const client=wrapped();fixture.call.mockRejectedValueOnce(new ApiError(409,'Acknowledgement lost',{code:'outcomeUnknown'}));
 await expect(client.jmap(envelope())).rejects.toMatchObject({code:'outcomeUnknown'});
 await client.jmap(envelope());const first=fixture.call.mock.calls[0]![2].idempotencyKey;
 expect(first).toBeTruthy();expect(fixture.call.mock.calls[1]![2].idempotencyKey).toBe(first);
 await client.jmap(envelope());expect(fixture.call.mock.calls[2]![2].idempotencyKey).not.toBe(first);
});
it('respects caller supplied keys and opt-out, and isolates uncertain creates by account authority',async()=>{
 const client=wrapped();await client.jmap(envelope(),{idempotencyKey:'explicit'});expect(fixture.call.mock.calls.at(-1)![2].idempotencyKey).toBe('explicit');
 await client.jmap(envelope(),{idempotencyKey:false});expect(fixture.call.mock.calls.at(-1)![2].idempotencyKey).toBe(false);
 fixture.call.mockRejectedValueOnce(new ApiError(503,'unknown',{code:'outcomeUnknown'}));await expect(client.jmap(envelope())).rejects.toThrow();
 const first=fixture.call.mock.calls.at(-1)![2].idempotencyKey;activateAccountPreferences('a','different-owner-authority');
 await client.jmap(envelope());expect(fixture.call.mock.calls.at(-1)![2].idempotencyKey).not.toBe(first);
});
