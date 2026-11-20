// This exercises EXISTING transport behavior; no production retry policy changes.
import {afterEach,beforeEach,expect,it,vi} from 'vitest';
import {apiFetch,setNativeTransport} from './http.ts';
import {rateRuntime,resetRateState} from './rateAware.ts';
import {activateAccountPreferences,suspendAccountPreferences} from './accountPreferenceLifecycle.ts';
import {installLinkedMailbox} from './linkedMailboxSession.ts';
import {errorDetails} from './client.ts';

const runtime={...rateRuntime};let clock=1_000_000;let waits:number[]=[];
const CODE='authorizationNotConfirmedBeforeDispatch';
const read=JSON.stringify({using:[],methodCalls:[['Email/get',{accountId:'account-one',ids:['message']},'get']]});
const mutation=JSON.stringify({using:[],methodCalls:[['Email/set',{accountId:'account-one',update:{message:{'keywords/$seen':true}}},'set']]});
function refusal():Response {return Response.json({type:'urn:mailwoman:problem:authorization-not-confirmed',status:503,code:CODE,notDispatched:true,retryAfterSecs:1},{status:503,headers:{'content-type':'application/problem+json','retry-after':'1','cache-control':'no-store'}});}
const ok=()=>Response.json({methodResponses:[],sessionState:'s'});
const options=(body:string,signal?:AbortSignal):RequestInit=>({method:'POST',headers:{'content-type':'application/json','X-Mailwoman-Account':'account-one','X-Mailwoman-Storage-Authority':'msa1_test_verified'},body,...(signal?{signal}:{})});
beforeEach(()=>{clock=1_000_000;waits=[];resetRateState();setNativeTransport(null);installLinkedMailbox(null);activateAccountPreferences('account-one','msa1_test_verified');document.cookie='mw_csrf=; Max-Age=0; Path=/';rateRuntime.now=()=>clock;rateRuntime.random=()=>0;rateRuntime.sleep=async(ms)=>{waits.push(ms);clock+=ms;};});
afterEach(()=>{Object.assign(rateRuntime,runtime);resetRateState();installLinkedMailbox(null);suspendAccountPreferences();setNativeTransport(null);vi.unstubAllGlobals();});
it('retains the precise fence code and Retry-After while leaving the problem body readable',async()=>{
 const response=refusal();expect(await errorDetails(response)).toMatchObject({code:CODE,retryAfterSecs:1});expect((await response.json()).notDispatched).toBe(true);
});
it('uses existing read-only pacing and preserves the original URL, body and authority',async()=>{
 const send=vi.fn(async(_input:string,_init?:RequestInit)=>ok()).mockResolvedValueOnce(refusal());vi.stubGlobal('fetch',send);
 expect((await apiFetch('/jmap/api',options(read))).status).toBe(200);expect(send).toHaveBeenCalledTimes(2);expect(waits).toHaveLength(1);expect(waits[0]).toBeGreaterThanOrEqual(1000);
 const [first,next]=send.mock.calls;expect(next?.[0]).toBe(first?.[0]);expect(next?.[1]?.body).toBe(first?.[1]?.body);
 expect([...new Headers(next?.[1]?.headers)]).toEqual([...new Headers(first?.[1]?.headers)]);
});
it('keeps retry headers bound to the original mailbox and discards a late answer after context changes',async()=>{
 const send=vi.fn(async(_input:string,_init?:RequestInit)=>ok()).mockResolvedValueOnce(refusal());vi.stubGlobal('fetch',send);
 rateRuntime.sleep=async(ms)=>{clock+=ms;installLinkedMailbox({linkId:'new-link',accountId:'other',storageAuthority:'msa1_other_verified'});};
 await expect(apiFetch('/jmap/api',options(read))).rejects.toMatchObject({name:'AbortError'});
 expect(send).toHaveBeenCalledTimes(2);
 for(const [url,init] of send.mock.calls){expect(url).toBe('/jmap/api');const headers=new Headers(init?.headers);expect(headers.get('x-mailwoman-link')).toBeNull();expect(headers.get('x-mailwoman-account')).toBe('account-one');expect(headers.get('x-mailwoman-storage-authority')).toBe('msa1_test_verified');}
});
it('honors abort during the advised wait without a second dispatch',async()=>{
 const controller=new AbortController();const send=vi.fn(async(_input:string,_init?:RequestInit)=>ok()).mockResolvedValueOnce(refusal());vi.stubGlobal('fetch',send);
 rateRuntime.sleep=(ms,signal)=>{const waiting=runtime.sleep(ms,signal);queueMicrotask(()=>controller.abort(new DOMException('stopped','AbortError')));return waiting;};
 await expect(apiFetch('/jmap/api',options(read,controller.signal))).rejects.toMatchObject({name:'AbortError'});expect(send).toHaveBeenCalledTimes(1);
});
it('does not replay an unkeyed mutation even for the exact typed pre-dispatch refusal',async()=>{
 const send=vi.fn(async(_input:string,_init?:RequestInit)=>refusal());vi.stubGlobal('fetch',send);
 const response=await apiFetch('/jmap/api',options(mutation));expect(response.status).toBe(503);expect((await errorDetails(response)).code).toBe(CODE);expect(send).toHaveBeenCalledTimes(1);expect(waits).toEqual([]);
});
it('does not broaden replay or claim non-dispatch for generic503 or uncertain SMTP outcomes',async()=>{
 for(const response of [new Response(null,{status:503}),Response.json({code:'outcomeUnknown'},{status:503,headers:{'retry-after':'1'}})]){
  resetRateState();const send=vi.fn(async(_input:string,_init?:RequestInit)=>response);vi.stubGlobal('fetch',send);
  const answer=await apiFetch('/jmap/api',options(mutation));expect(answer.status).toBe(503);expect((await errorDetails(answer)).code).not.toBe(CODE);expect(send).toHaveBeenCalledTimes(1);
 }
});
