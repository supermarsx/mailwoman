import {recoverRequest,recoveryRandom} from './recovery.ts';
import {afterEach,beforeEach,describe,expect,it,vi} from 'vitest';
import {apiFetch} from './http.ts';
import {activateAccountPreferences,suspendAccountPreferences} from './accountPreferenceLifecycle.ts';
import {createLinkedAccountsApi,type LinkedMailbox} from './linkedAccounts.ts';
vi.mock('./http.ts',()=>({apiFetch:vi.fn()}));
const row:LinkedMailbox={id:'link/1',accountId:'1',address:'child@example.org',displayName:'Child',storageAuthority:'msa1_child_authority',status:'ready',revision:'revision-1',expiresAt:42};
afterEach(()=>{suspendAccountPreferences();vi.resetAllMocks();});
describe('linked account management transport',()=>{
  it('unlinks by opaque link ID and exact quoted revision without a child selector',async()=>{
    activateAccountPreferences('1','msa1_parent_authority');
    vi.mocked(apiFetch).mockResolvedValue(new Response(null,{status:204}));
    const api=createLinkedAccountsApi();
    try {
      await api.unlink(row);
      const [url,init]=vi.mocked(apiFetch).mock.calls[0]!;
      expect(url).toBe('/api/accounts/link%2F1');
      expect(init?.method).toBe('DELETE');
      expect(new Headers(init?.headers).get('if-match')).toBe('"revision-1"');
      expect(new Headers(init?.headers).has('x-mailwoman-link')).toBe(false);
    } finally {api.dispose();}
  });
  it('aborts an in-flight management operation on same-ID realm change',async()=>{
    activateAccountPreferences('1','msa1_parent_authority');
    let release!:(response:Response)=>void;
    vi.mocked(apiFetch).mockImplementation(()=>new Promise(resolve=>{release=resolve;}));
    const api=createLinkedAccountsApi();
    try {
      const result=api.list();
      const rejected=expect(result).rejects.toMatchObject({name:'AbortError'});
      activateAccountPreferences('1','msa1_other_authority');
      expect(vi.mocked(apiFetch).mock.calls[0]?.[1]?.signal?.aborted).toBe(true);
      release(Response.json({accounts:[row]}));
      await rejected;
    } finally {api.dispose();}
  });
  it('preserves typed policy errors without exposing response diagnostic text',async()=>{
    activateAccountPreferences('1','msa1_parent_authority');
    vi.mocked(apiFetch).mockResolvedValue(Response.json({code:'linkingDisabled',error:'private diagnostic'},{status:403}));
    const api=createLinkedAccountsApi();
    try {await expect(api.list()).rejects.toMatchObject({code:'linkingDisabled',message:'Linked mailbox request failed'});} finally {api.dispose();}
  });
});

describe('bounded credential renewal recovery',()=>{
 beforeEach(()=>{vi.useFakeTimers();vi.stubGlobal('crypto',undefined);recoveryRandom.next=()=>1;activateAccountPreferences('1','msa1_parent_authority');});
 afterEach(()=>{vi.useRealTimers();vi.unstubAllGlobals();recoveryRandom.next=()=>Math.random();});
 function transport(send:(init:RequestInit|undefined)=>Promise<Response>):void {
  vi.mocked(apiFetch).mockImplementation((input,init)=>recoverRequest(String(input),init,send,init!.recovery!));
 }
 it('waits for the server 25s classified timeout and exposes it without another credential submission',async()=>{
  const send=vi.fn(()=>new Promise<Response>(resolve=>setTimeout(()=>resolve(Response.json({code:'linkedServerUnreachable'},{status:502})),25_000)));transport(send);
  const api=createLinkedAccountsApi();const done=api.renew('link',{password:'private'}).catch((error:unknown)=>error);
  await vi.advanceTimersByTimeAsync(20_000);expect(send).toHaveBeenCalledTimes(1);
  await vi.advanceTimersByTimeAsync(5_000);expect(await done).toMatchObject({code:'linkedServerUnreachable'});expect(send).toHaveBeenCalledTimes(1);api.dispose();
 });
 it.each([{code:'linkedPasswordRejected',status:401},{code:'linkedRenewLimit',status:429}])('does not resubmit a typed $code refusal',async({code,status})=>{
  const send=vi.fn(async()=>Response.json({code},{status}));transport(send);const api=createLinkedAccountsApi();
  const done=api.renew('link',{password:'private'}).catch((error:unknown)=>error);await vi.runAllTimersAsync();expect(await done).toMatchObject({code});expect(send).toHaveBeenCalledTimes(1);api.dispose();
 });
 it('retries one transport failure with the same idempotency key',async()=>{
  const send=vi.fn<(init:RequestInit|undefined)=>Promise<Response>>().mockRejectedValueOnce(new TypeError('Network failed')).mockResolvedValueOnce(Response.json({ok:true}));transport(send);
  const api=createLinkedAccountsApi();const done=api.renew('link',{password:'private'});await vi.runAllTimersAsync();await done;
  expect(send).toHaveBeenCalledTimes(2);const keys=send.mock.calls.map(([init])=>new Headers(init?.headers).get('Idempotency-Key'));expect(keys[0]).toBeTruthy();expect(keys[1]).toBe(keys[0]);api.dispose();
 });
 it('bounds never-answering renewal to two 30s attempts and one backoff',async()=>{
  const send=vi.fn((init:RequestInit|undefined)=>new Promise<Response>((_,reject)=>init?.signal?.addEventListener('abort',()=>reject(new DOMException('Aborted','AbortError')))));transport(send);
  const api=createLinkedAccountsApi();const start=Date.now();const done=api.renew('link',{password:'private'}).catch((error:unknown)=>error);await vi.runAllTimersAsync();
  expect(await done).toMatchObject({reason:'server'});expect(send).toHaveBeenCalledTimes(2);expect(Date.now()-start).toBe(60_400);api.dispose();
 });
});
