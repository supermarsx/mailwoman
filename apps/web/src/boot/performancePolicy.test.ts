import {afterEach,expect,it,vi} from 'vitest';
import {applyWebPerformance,DEFAULT_WEB_PERFORMANCE,parseWebPerformance,resetWebPerformance,webPerformancePolicy} from './performancePolicy.ts';
import {resetIdlePrefetch,scheduleIdlePrefetch} from './idlePrefetch.ts';
afterEach(()=>{resetWebPerformance();resetIdlePrefetch();});
it('rejects invalid bounds/types and keeps the app/four prewarm defaults; core reads as app',()=>{
 expect(parseWebPerformance({retainedLocales:1,concurrentLoads:0,idleDelayMs:Infinity,precacheMode:'https://bad.test/',precacheConcurrency:50,prefetchEnabled:'true',respectSaveData:null})).toEqual(DEFAULT_WEB_PERFORMANCE);
 expect(parseWebPerformance({precacheMode:'all'}).precacheMode).toBe('all');
 expect(parseWebPerformance({retainedLocales:8,concurrentLoads:1,idleDelayMs:30000,precacheMode:'core',precacheConcurrency:1,prefetchEnabled:false,respectSaveData:false})).toEqual({retainedLocales:8,concurrentLoads:1,idleDelayMs:30000,precacheMode:'app',precacheConcurrency:1,prefetchEnabled:false,prefetchModules:false,respectSaveData:false});
});
it('actually suppresses optional scheduling and stops later idle steps after an admin disable',async()=>{
 const idle:Array<()=>void>=[],run=vi.fn(async()=>undefined),host={requestIdleCallback:(callback:()=>void)=>{idle.push(callback);return 1;},setTimeout:vi.fn()};
 applyWebPerformance({prefetchEnabled:false});expect(scheduleIdlePrefetch([run],host)).toBe(false);
 applyWebPerformance({prefetchEnabled:true});expect(scheduleIdlePrefetch([run,run],host)).toBe(true);
 applyWebPerformance({prefetchEnabled:false});idle.shift()!();await Promise.resolve();expect(run).not.toHaveBeenCalled();expect(webPerformancePolicy().prefetchEnabled).toBe(false);
 applyWebPerformance({prefetchEnabled:true});expect(scheduleIdlePrefetch([run],host)).toBe(true);
});
it('does not speculate using unconfirmed startup defaults',()=>{
 const run=vi.fn(async()=>undefined),idle=vi.fn(),host={requestIdleCallback:idle,setTimeout:vi.fn()};
 resetWebPerformance();expect(scheduleIdlePrefetch([run],host)).toBe(false);expect(idle).not.toHaveBeenCalled();expect(run).not.toHaveBeenCalled();
});
