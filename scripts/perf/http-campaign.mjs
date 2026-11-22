import {requestInputsExpected} from './request-inputs-experiment.mjs';
import {cacheExperimentFromEnv} from './sqlite-page-cache-witness.mjs';
// Bounded plans for an explicitly opted-in, isolated application fixture.
import {measure} from './http-load.mjs';

function number(env,key,fallback,min,max){
 const value=Number(env[key]??fallback);
 if(!Number.isFinite(value)||value<min||value>max)throw new Error(`Invalid ${key}`);
 return value;
}
function numbers(env,key,fallback,min,max,limit=10){
 const values=String(env[key]??fallback).split(',').map(Number);
 if(!values.length||values.length>limit||values.some(v=>!Number.isInteger(v)||v<min||v>max))throw new Error(`Invalid ${key}`);
 return values;
}
export function campaignOptions(env=process.env){
 const mode=env.MW_LOAD_FIXTURE_MODE??'smoke';
 if(!['smoke','ramp','capacity'].includes(mode))throw new Error('Invalid MW_LOAD_FIXTURE_MODE');
 if(mode!=='smoke'&&env.MW_LOAD_FIXTURE_LONG!=='1')throw new Error('Long campaigns require MW_LOAD_FIXTURE_LONG=1');
 const seconds=number(env,'MW_LOAD_FIXTURE_SECONDS',mode==='smoke'?1:30,.1,60);
 const warmupSeconds=number(env,'MW_LOAD_FIXTURE_WARMUP_SECONDS',mode==='smoke'?0:10,0,60);
 const deadlineSeconds=number(env,'MW_LOAD_FIXTURE_DEADLINE_SECONDS',mode==='smoke'?90:mode==='capacity'?600:300,10,1800);
 const concurrency=numbers(env,'MW_LOAD_FIXTURE_CONCURRENCY',mode==='smoke'?'32':'1,4,16,64,256',1,4096);
 const endpoints=String(env.MW_LOAD_FIXTURE_ENDPOINTS??'mixed').split(',');
 if(!endpoints.length||endpoints.length>12||endpoints.some(v=>!['mixed','identity','mailboxes','query','headers','refresh','representative','paged-headers','reader','attachment','flag-write','move-write'].includes(v)))throw new Error('Invalid MW_LOAD_FIXTURE_ENDPOINTS');
 const rates=mode==='ramp'&&!env.MW_LOAD_FIXTURE_RATES?[]:numbers(env,'MW_LOAD_FIXTURE_RATES',mode==='capacity'?'1000,5000,10000,20000,40000,42000':'100,500,1000',1,100000);
 if(mode==='smoke'&&seconds>5)throw new Error('Smoke phases are limited to five seconds; opt in to long mode');
 return {mode,seconds,warmupSeconds,deadlineSeconds,concurrency,endpoints,rates,
  ...(env.MW_LOAD_FIXTURE_REQUEST_INPUTS_EXPECT===undefined?{}:{requestInputsExpected:requestInputsExpected(env.MW_LOAD_FIXTURE_REQUEST_INPUTS_EXPECT)}),
  timeoutMs:number(env,'MW_LOAD_FIXTURE_TIMEOUT_MS',5000,100,30000),
  maxErrorRate:number(env,'MW_LOAD_FIXTURE_MAX_ERROR_RATE',.001,0,1),
  maxThrottleRate:number(env,'MW_LOAD_FIXTURE_MAX_THROTTLE_RATE',0,0,1),
  maxP99Ms:number(env,'MW_LOAD_FIXTURE_MAX_P99_MS',2000,1,120000),
  allowOverload:env.MW_LOAD_FIXTURE_ALLOW_OVERLOAD==='1',adaptive:env.MW_LOAD_FIXTURE_ADAPTIVE!=='0',...(env.MW_LOAD_FIXTURE_SQLITE_CACHE_EXPECT===undefined?{}:{sqliteCacheExperiment:cacheExperimentFromEnv(env.MW_LOAD_FIXTURE_SQLITE_CACHE_EXPECT)})};
}
export function stopReason(stage,options){
 if(stage.mutationStateTainted)return 'unknown-mutation-outcome';
 if(stage.timeouts>0)return 'timeout';
 if(Object.entries(stage.statuses).some(([status,count])=>Number(status)>=500&&Number(status)<600&&count>0))return 'server-5xx';
 if(stage.errors/Math.max(1,stage.requests)>options.maxErrorRate)return 'error-budget';
 if(stage.throttled/Math.max(1,stage.requests)>options.maxThrottleRate)return 'throttle-budget';
 if(stage.latencyMs.p99>options.maxP99Ms)return 'latency-budget';
 return null;
}
export async function runCampaign({base,headers,groups,options,run=measure,diagnostics=async()=>null}){
 const stages=[],warmups=[],decisions=[],started=performance.now();let deadlineReached=false,mutationStateTainted=false;
 const room=()=>options.deadlineSeconds-(performance.now()-started)/1000;
 campaign:for(const endpoint of options.endpoints){
  const requests=groups[endpoint];if(!requests?.length)throw new Error('Unknown endpoint group');
  if(options.warmupSeconds){
   if(room()<options.warmupSeconds+options.timeoutMs/1000+2){deadlineReached=true;break;}
   const result=await run({base,headers,requests,concurrency:1,seconds:options.warmupSeconds,timeoutMs:options.timeoutMs});
   warmups.push({endpoint,...result});
   if(result.mutationStateTainted){mutationStateTainted=true;decisions.push({endpoint,reason:'warmup-unknown-mutation-outcome'});break campaign;}
   const reason=stopReason(result,options);if(reason){decisions.push({endpoint,reason:'warmup-'+reason});continue;}
  }
  let peak=0,previous=0,plateaus=0,stopped=false;
  const stage=async(concurrency,arrivalRate)=>{
   if(room()<options.seconds+options.timeoutMs/1000+2){deadlineReached=true;return null;}
   const before=await diagnostics('before');
   const result=await run({base,headers,requests,concurrency,seconds:options.seconds,timeoutMs:options.timeoutMs,...(arrivalRate===undefined?{}:{arrivalRate})});
   const after=await diagnostics('after');
   stages.push({endpoint,phase:arrivalRate===undefined?'concurrency-ramp':'offered-rate',diagnostics:{before,after},...result,...(options.requestInputsExpected===undefined?{}:{requestInputsWindow:{schema:1,kind:'workload-and-drain-only',eligibleControlRequests:0}})});
   return result;
  };
  if(options.mode!=='smoke')for(const concurrency of options.concurrency){
   const result=await stage(concurrency);if(!result)break;
   if(result.mutationStateTainted){mutationStateTainted=true;decisions.push({endpoint,concurrency,reason:'unknown-mutation-outcome'});break campaign;}
   const reason=stopReason(result,options);peak=Math.max(peak,result.acceptedRequestsPerSecond);
   if(reason){decisions.push({endpoint,concurrency,reason});stopped=true;break;}
   plateaus=previous&&result.acceptedRequestsPerSecond<previous*1.05?plateaus+1:0;
   previous=result.acceptedRequestsPerSecond;
   if(options.adaptive&&plateaus>=2){decisions.push({endpoint,concurrency,reason:'accepted-throughput-plateau'});break;}
  }
  if(!stopped&&!deadlineReached)for(const rate of options.rates){
   if(options.mode!=='smoke'&&!options.allowOverload&&rate>peak*1.2){decisions.push({endpoint,rate,reason:'offered-rate-exceeds-observed-saturation'});break;}
   const result=await stage(options.concurrency.at(-1),rate);if(!result)break;
   if(result.mutationStateTainted){mutationStateTainted=true;decisions.push({endpoint,rate,reason:'unknown-mutation-outcome'});break campaign;}
   const reason=stopReason(result,options);
   if(options.mode!=='smoke'&&reason){decisions.push({endpoint,rate,reason});break;}
   if(options.mode!=='smoke'&&(result.droppedCapacity+result.droppedScheduler)>result.offered*.01){decisions.push({endpoint,rate,reason:'generator-or-outstanding-capacity'});break;}
  }
  if(deadlineReached)break;
 }
 return {options,stages,warmups,decisions,deadlineReached,mutationStateTainted,elapsedSeconds:(performance.now()-started)/1000};
}
