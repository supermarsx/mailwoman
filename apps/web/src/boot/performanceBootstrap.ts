import {refreshPublicConfig} from '../api/publicConfig.ts';
import {applyWebPerformance,webPerformancePolicy,webPerformanceReady,invalidateWebPerformance,type WebPerformancePolicy} from './performancePolicy.ts';
let pending:Promise<Readonly<WebPerformancePolicy>|null>|undefined;
let generation=0;
/** Fresh public hints. No stale SWR result and no wait on the render path. */
export function refreshWebPerformance():Promise<Readonly<WebPerformancePolicy>|null>{
 if(pending)return pending;
 const startedGeneration=generation;
 const controller=new AbortController(),timeout=setTimeout(()=>controller.abort(),5000);
 pending=refreshPublicConfig(controller.signal).then(config=>{if(startedGeneration!==generation)return null;applyWebPerformance(config.webPerformance);return webPerformancePolicy();}).catch(()=>{if(startedGeneration===generation)invalidateWebPerformance();return null;}).finally(()=>{clearTimeout(timeout);pending=undefined;});
 return pending;
}
/** Never satisfy an acknowledged admin write with a pre-commit pending read.
 * At most one fresh follow-up per call; a newer commit supersedes older callers. */
export async function refreshWebPerformanceAfterCommit():Promise<Readonly<WebPerformancePolicy>|null>{
 const committedGeneration=++generation,previous=pending;invalidateWebPerformance();
 if(previous)await previous;
 if(committedGeneration!==generation)return null;
 return refreshWebPerformance();
}
/** Consume this document's freshly bootstrapped hints without another GET for
 * each speculative consumer. Foreground/admin refresh remain explicit above. */
export function ensureWebPerformance():Promise<Readonly<WebPerformancePolicy>|null>{
 return pending??(webPerformanceReady()?Promise.resolve(webPerformancePolicy()):refreshWebPerformance());
}
/** Install once per App owner; a fresh foreground read propagates admin edits. */
export function watchWebPerformance():()=>void{
 let last=Date.now();
 void refreshWebPerformance();
 const foreground=()=>{if(document.visibilityState==='visible'&&Date.now()-last>=30000){last=Date.now();void refreshWebPerformance();}};
 document.addEventListener('visibilitychange',foreground);
 return()=>document.removeEventListener('visibilitychange',foreground);
}
