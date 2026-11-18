import {afterEach,beforeEach,expect,it,vi} from 'vitest';
import {accountDraftApi} from './accountDrafts.ts';
import {activateAccountPreferences,suspendAccountPreferences,preferenceAccount} from './accountPreferenceLifecycle.ts';
import {apiFetch} from './http.ts';
vi.mock('./http.ts',()=>({apiFetch:vi.fn()}));
const authority='test-draft-list-authority';
const result=()=>new Response(JSON.stringify({accountId:'a',storageAuthority:authority,supported:true,items:[]}));
beforeEach(()=>{vi.useFakeTimers();activateAccountPreferences('a',authority);vi.mocked(apiFetch).mockReset();});
afterEach(()=>{suspendAccountPreferences();vi.useRealTimers();});
it('shares concurrent and recently completed list reads across readers and composers',async()=>{
 let finish!:(value:Response)=>void;
 vi.mocked(apiFetch).mockImplementation(()=>new Promise(resolve=>{finish=resolve;}));
 const owner=preferenceAccount()!,calls=Array.from({length:20},()=>accountDraftApi(owner).list());
 expect(apiFetch).toHaveBeenCalledOnce();finish(result());await Promise.all(calls);
 await accountDraftApi(owner).list();expect(apiFetch).toHaveBeenCalledOnce();
});
it('cancels one consumer without aborting another readers shared request',async()=>{
 let finish!:(value:Response)=>void;
 vi.mocked(apiFetch).mockImplementation(()=>new Promise(resolve=>{finish=resolve;}));
 const owner=preferenceAccount()!,abort=new AbortController();
 const first=accountDraftApi(owner,abort.signal).list(),second=accountDraftApi(owner).list();
 abort.abort();await expect(first).rejects.toMatchObject({name:'AbortError'});
 expect(vi.mocked(apiFetch).mock.calls[0]![1]?.signal?.aborted).toBe(false);
 finish(result());await second;expect(apiFetch).toHaveBeenCalledOnce();
});
it('honors Retry-After without retrying on every newly opened message',async()=>{
 vi.mocked(apiFetch).mockResolvedValue(new Response(JSON.stringify({code:'rateLimited'}),{status:429,headers:{'Retry-After':'60'}}));
 const owner=preferenceAccount()!;
 await expect(accountDraftApi(owner).list()).rejects.toThrow('rateLimited');
 await vi.advanceTimersByTimeAsync(30_000);
 for(let i=0;i<10;i++)await expect(accountDraftApi(owner).list()).rejects.toThrow('rateLimited');
 expect(apiFetch).toHaveBeenCalledOnce();
 vi.mocked(apiFetch).mockResolvedValue(result());await vi.advanceTimersByTimeAsync(30_001);
 await accountDraftApi(owner).list();expect(apiFetch).toHaveBeenCalledTimes(2);
});
it('drops cached summaries on a successful draft mutation and account replacement',async()=>{
 vi.mocked(apiFetch).mockImplementation(async()=>result());const owner=preferenceAccount()!;
 await accountDraftApi(owner).list();await accountDraftApi(owner).discard('draft','r1');await accountDraftApi(owner).list();
 expect(apiFetch).toHaveBeenCalledTimes(3);
 activateAccountPreferences('a','replacement-draft-authority');
 await expect(accountDraftApi(owner).list()).rejects.toThrow('accountChanged');
});
