import {afterEach,beforeEach,expect,it,vi} from 'vitest';
vi.mock('../api/publicConfig.ts',()=>({refreshPublicConfig:vi.fn()}));
import {refreshPublicConfig} from '../api/publicConfig.ts';
import {refreshWebPerformance,watchWebPerformance,ensureWebPerformance,refreshWebPerformanceAfterCommit} from './performanceBootstrap.ts';
import {DEFAULT_WEB_PERFORMANCE,resetWebPerformance,webPerformancePolicy,webPerformanceReady} from './performancePolicy.ts';
beforeEach(()=>{resetWebPerformance();vi.mocked(refreshPublicConfig).mockReset();});
afterEach(()=>{resetWebPerformance();vi.restoreAllMocks();vi.useRealTimers();});
it('shares only pending fresh reads without delaying demand defaults, then applies a later fresh edit',async()=>{
 let finish!:(value:Awaited<ReturnType<typeof refreshPublicConfig>>)=>void;
 vi.mocked(refreshPublicConfig).mockImplementation(()=>new Promise(resolve=>{finish=resolve;}));
 const one=refreshWebPerformance(),two=refreshWebPerformance();expect(one).toBe(two);expect(webPerformancePolicy()).toEqual(DEFAULT_WEB_PERFORMANCE);
 finish({webPerformance:{retainedLocales:2,prefetchEnabled:false}} as Awaited<ReturnType<typeof refreshPublicConfig>>);await one;
 expect(webPerformancePolicy().retainedLocales).toBe(2);expect(webPerformancePolicy().prefetchEnabled).toBe(false);expect(refreshPublicConfig).toHaveBeenCalledTimes(1);
 await ensureWebPerformance();expect(refreshPublicConfig).toHaveBeenCalledTimes(1);
 vi.mocked(refreshPublicConfig).mockResolvedValue({webPerformance:{prefetchEnabled:true}} as Awaited<ReturnType<typeof refreshPublicConfig>>);await refreshWebPerformance();
 expect(refreshPublicConfig).toHaveBeenCalledTimes(2);expect(webPerformancePolicy().prefetchEnabled).toBe(true);
});
it('keeps failed bootstrap retryable and never fabricates a successful policy',async()=>{
 vi.mocked(refreshPublicConfig).mockRejectedValueOnce(Error('unavailable'));
 expect(await refreshWebPerformance()).toBeNull();expect(webPerformancePolicy()).toEqual(DEFAULT_WEB_PERFORMANCE);
 vi.mocked(refreshPublicConfig).mockResolvedValue({webPerformance:{concurrentLoads:1}} as Awaited<ReturnType<typeof refreshPublicConfig>>);
 expect((await refreshWebPerformance())?.concurrentLoads).toBe(1);
});
it('actually aborts the owned bootstrap read at its five-second bound',async()=>{
 vi.useFakeTimers();let signal:AbortSignal|undefined;
 vi.mocked(refreshPublicConfig).mockImplementation(current=>{signal=current;return new Promise((_,reject)=>current!.addEventListener('abort',()=>reject(current!.reason),{once:true}));});
 const pending=refreshWebPerformance();await vi.advanceTimersByTimeAsync(5000);
 expect(signal?.aborted).toBe(true);expect(await pending).toBeNull();
});
it('refreshes on a later foreground transition and removes the listener with its App owner',async()=>{
 vi.useFakeTimers();vi.spyOn(document,'visibilityState','get').mockReturnValue('visible');
 vi.mocked(refreshPublicConfig).mockResolvedValue({webPerformance:{}} as Awaited<ReturnType<typeof refreshPublicConfig>>);
 const stop=watchWebPerformance();await refreshWebPerformance();expect(refreshPublicConfig).toHaveBeenCalledTimes(1);
 document.dispatchEvent(new Event('visibilitychange'));expect(refreshPublicConfig).toHaveBeenCalledTimes(1);
 await vi.advanceTimersByTimeAsync(30000);document.dispatchEvent(new Event('visibilitychange'));await refreshWebPerformance();expect(refreshPublicConfig).toHaveBeenCalledTimes(2);
 stop();await vi.advanceTimersByTimeAsync(30000);document.dispatchEvent(new Event('visibilitychange'));expect(refreshPublicConfig).toHaveBeenCalledTimes(2);
});
it('an acknowledged admin commit cannot reuse or apply a pre-commit pending response',async()=>{
 let oldResponse!:(value:Awaited<ReturnType<typeof refreshPublicConfig>>)=>void;
 vi.mocked(refreshPublicConfig).mockImplementationOnce(()=>new Promise(resolve=>{oldResponse=resolve;})).mockResolvedValueOnce({webPerformance:{prefetchEnabled:false,retainedLocales:2}} as Awaited<ReturnType<typeof refreshPublicConfig>>);
 const beforeCommit=refreshWebPerformance(),afterCommit=refreshWebPerformanceAfterCommit();
 expect(refreshPublicConfig).toHaveBeenCalledTimes(1);expect(webPerformanceReady()).toBe(false);
 oldResponse({webPerformance:{prefetchEnabled:true,retainedLocales:8}} as Awaited<ReturnType<typeof refreshPublicConfig>>);
 expect(await beforeCommit).toBeNull();expect((await afterCommit)?.prefetchEnabled).toBe(false);
 expect(webPerformancePolicy().retainedLocales).toBe(2);expect(refreshPublicConfig).toHaveBeenCalledTimes(2);
});
