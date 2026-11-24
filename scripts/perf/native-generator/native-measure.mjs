// Node adapter for the native generator (mw-loadgen). It is a drop-in `run` for
// runCampaign(): the Node driver keeps preflights, diagnostics, stop rules and the
// report; each measured stage runs in one mw-loadgen process, and its raw counters
// are turned into the stage object with http-load.mjs's own Measurements/merge/report
// functions, so the stage schema is the one measure() returns. Private files only:
// the plan holds the fixture cookies and lives in a 0700 temporary directory that
// close() removes. Opt-in: MW_LOAD_GENERATOR=native (see docs/perf/native-generator.md).
import {spawn} from 'node:child_process';
import {mkdtempSync,writeFileSync,readFileSync,rmSync,existsSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {dirname,join} from 'node:path';
import {fileURLToPath} from 'node:url';
import {Measurements,mergeBytesSnapshots,bytesReport,mergeHttp503Reasons,mergeLatencyBreakdowns,SCHEDULING_LAG_LIMIT_MS,MAX_OUTSTANDING,MAX_STAGE_SECONDS,MAX_OFFERED_RPS} from '../http-load.mjs';
import {goalMixReport} from '../representative-workload.mjs';
import {DEFAULT_HEADER_PROPERTIES,MINIMAL_HEADER_PROPERTIES,READER_PROPERTIES,READER_FETCH_OPTIONS} from '../fixture-properties.mjs';
import {nativeReadContract} from '../native-read-intent.mjs';

const here=dirname(fileURLToPath(import.meta.url));
export const DEFAULT_RATE_LIMITS=Object.freeze({clientRequests:600,accountRequests:1800,rateWindowSecs:60});

export function nativeBinary(env=process.env){
 const path=env.MW_NATIVE_GENERATOR_BIN||join(here,'target','release',process.platform==='win32'?'mw-loadgen.exe':'mw-loadgen');
 if(!existsSync(path))throw new Error('Native generator binary missing: cargo build --release --manifest-path scripts/perf/native-generator/Cargo.toml (or set MW_NATIVE_GENERATOR_BIN)');
 return path;
}

// Group layout from prepareFixtureRequests: shape name and flags per slot, in slot order.
export function groupLayout(groups){
 return Object.fromEntries(Object.entries(groups).map(([name,list])=>[name,list.map(spec=>({name:spec.name,serialize:spec.serializePrincipal===true,stateful:spec.statefulMutation===true}))]));
}
export function rateLimitsOf(enforcedPolicy){
 const parsed=enforcedPolicy?.policy?.parsed;
 return parsed&&Number.isFinite(parsed.clientRequests)&&Number.isFinite(parsed.accountRequests)&&Number.isFinite(parsed.rateWindowSecs)
  ?{clientRequests:parsed.clientRequests,accountRequests:parsed.accountRequests,rateWindowSecs:parsed.rateWindowSecs,source:'handoff enforced policy'}
  :{...DEFAULT_RATE_LIMITS,source:'administrator defaults (no enforced policy in the handoff)'};
}
const bounded=(env,key,fallback,min,max)=>{
 if(env[key]===undefined||env[key]==='')return fallback;
 const value=Number(env[key]);if(!Number.isInteger(value)||value<min||value>max)throw new Error(`Invalid ${key}`);return value;
};
export function nativePlan({principals,clientSimulation,engineMode,representative,groups,enforcedPolicy,env=process.env}){
 return {schema:1,engineMode,representative,principals,...(clientSimulation===undefined?{}:{clientSimulation}),
  clientsPerUser:bounded(env,'MW_NATIVE_GENERATOR_CLIENTS_PER_USER',1,1,64),
  groups:groupLayout(groups),
  constants:{defaultHeaderProperties:[...DEFAULT_HEADER_PROPERTIES],minimalHeaderProperties:[...MINIMAL_HEADER_PROPERTIES],readerProperties:[...READER_PROPERTIES],readerFetchOptions:{...READER_FETCH_OPTIONS},
   readContract:{header:nativeReadContract.header,value:nativeReadContract.value,properties:[...nativeReadContract.properties],arguments:[...nativeReadContract.arguments],maxIds:nativeReadContract.maxIds,maxIdBytes:nativeReadContract.maxIdBytes,maxCallIdBytes:nativeReadContract.maxCallIdBytes,maxBodyBytes:nativeReadContract.maxBodyBytes}},
  rateLimits:rateLimitsOf(enforcedPolicy)};
}

export function runBinary(bin,args,{timeoutMs}){
 return new Promise((resolve,reject)=>{
  const child=spawn(bin,args,{stdio:['ignore','ignore','pipe'],windowsHide:true});
  let stderr='';child.stderr.on('data',chunk=>{if(stderr.length<4096)stderr+=chunk.toString('utf8');});
  const timer=setTimeout(()=>child.kill('SIGKILL'),timeoutMs);
  child.once('error',error=>{clearTimeout(timer);reject(error);});
  child.once('exit',code=>{clearTimeout(timer);code===0?resolve():reject(new Error((stderr.trim().split('\n')[0]||`native generator exited with ${code}`).slice(0,300)));});
 });
}

// The stage object of measure(), built from the native raw counters with Node's functions.
export function nativeStageResult(r,{requests,seconds,concurrency,arrivalRate,latePolicy='drop',includeRawMeasurements=false}){
 if(r?.schema!==1||r.generator!=='native'||!r.raw)throw new Error('Invalid native generator output');
 const names=[...new Set(requests.map(request=>request.name))];
 if(names.length!==r.names.length||names.some(name=>!r.names.includes(name)))throw new Error('Native generator shapes differ from the group');
 const elapsed=r.elapsedSeconds,lateDispatch=latePolicy==='dispatch';
 const measured=value=>new Measurements().merge(value);
 const aggregate=measured(r.raw.aggregate),byRequest=new Map(names.map(name=>[name,measured(r.raw.byRequest[name])]));
 const scheduling=measured(r.raw.scheduling),arrivalLatency=measured(r.raw.arrivalLatency);
 const profile=mergeBytesSnapshots([r.raw.bytesProfile]);
 const error503=mergeHttp503Reasons([r.raw.http503Reasons.aggregate],[aggregate.statuses[503]??0]);
 const latencyBreakdown=mergeLatencyBreakdowns([r.raw.latencyBreakdown.aggregate],[aggregate.count],[r.httpDispatchAttempts]);
 const cpuMs=r.cpu.userMs+r.cpu.systemMs;
 const hooks={...(r.window?{window:r.window}:{}),latencyBreakdown,http503Reasons:error503,...(includeRawMeasurements?{rawMeasurements:r.raw}:{})};
 const mix=goalMixReport(Object.fromEntries([...byRequest].map(([name,value])=>[name,value.report(elapsed)])),seconds,{users:r.uniquePrincipalsAcceptedDuringScheduledWindow,meanLatencyMs:aggregate.count?aggregate.totalMs/aggregate.count:undefined});
 const arrival=arrivalRate===undefined?{}:{arrivalRate,offered:r.offered,attempted:aggregate.count,attemptedRequestsPerSecond:aggregate.count/seconds,achievedRequestsPerSecond:aggregate.count/elapsed,droppedCapacity:r.droppedCapacity,droppedScheduler:r.droppedScheduler,peakOutstanding:r.peakOutstanding,latePolicy,lateSlotsDispatched:r.lateSlotsDispatched,droppedLateSlots:r.droppedLateSlots,unissuedAtWindowEnd:r.droppedScheduler-r.droppedLateSlots,latencyBasis:lateDispatch?'scheduled-slot':'send',generatorLimited:scheduling.percentile(.99)>SCHEDULING_LAG_LIMIT_MS,schedulingLagLimitMs:SCHEDULING_LAG_LIMIT_MS,schedulingLagMs:scheduling.report(elapsed).latencyMs,plannedArrivalLatencyMs:arrivalLatency.report(elapsed).latencyMs};
 return {...hooks,transfer:bytesReport(profile,{seconds,windowDecodedBytes:aggregate.acceptedDuringScheduledWindowDecodedBytes}),...(mix?{mix}:{}),protocol:r.http2?r.http2.protocol:'http/1.1',...(r.http2?{http2:r.http2}:{}),
  mutationStateTainted:r.mutationStateTainted,unknownMutationOutcomes:r.unknownMutationOutcomes,taintedPrincipals:r.taintedPrincipals,httpDispatchAttempts:r.httpDispatchAttempts,blockedAfterMutationTaint:r.blockedAfterMutationTaint,abortedUnscheduled:r.abortedUnscheduled,
  uniquePrincipalsAccepted:r.uniquePrincipalsAccepted,uniquePrincipalsAttempted:r.uniquePrincipalsAttempted,uniquePrincipalsAcceptedDuringScheduledWindow:r.uniquePrincipalsAcceptedDuringScheduledWindow,uniqueClientsAttempted:r.uniqueClientsAttempted,uniqueClientsAccepted:r.uniqueClientsAccepted,uniqueClientsAcceptedDuringScheduledWindow:r.uniqueClientsAcceptedDuringScheduledWindow,
  participantDistribution:r.participantDistribution,peakOutstanding:r.peakOutstanding,acceptedDuringScheduledWindow:r.acceptedDuringScheduledWindow,acceptedDuringScheduledWindowRps:r.acceptedDuringScheduledWindow/seconds,acceptedPerSecond:r.acceptedPerSecond,
  generator:{kind:'native',cpuUserMs:r.cpu.userMs,cpuSystemMs:r.cpu.systemMs,cpuLogicalCores:cpuMs*1000/(r.cpuElapsedSeconds*1000000),rssBeforeBytes:r.rssBeforeBytes,rssAfterBytes:r.rssAfterBytes,heapAfterBytes:null,cpuMicrosPerRequest:aggregate.count?cpuMs*1000/aggregate.count:null},
  concurrency,scheduledSeconds:seconds,elapsedSeconds:elapsed,...arrival,...aggregate.report(elapsed),
  byRequest:Object.fromEntries([...byRequest].map(([key,value])=>[key,{...value.report(elapsed),latencyBreakdown:mergeLatencyBreakdowns([r.raw.latencyBreakdown.byRequest[key]],[value.count]),http503Reasons:mergeHttp503Reasons([r.raw.http503Reasons.byRequest[key]],[value.statuses[503]??0])}])),
  generatorImplementation:'native',native:r.native};
}

// A `run` for runCampaign over the groups of one prepareFixtureRequests() result.
export function createNativeRun({principals,clientSimulation,engineMode,representative,groups,enforcedPolicy,env=process.env}){
 const bin=nativeBinary(env),directory=mkdtempSync(join(tmpdir(),'mw-native-generator-'));
 const plan=join(directory,'plan.private.json');
 writeFileSync(plan,JSON.stringify(nativePlan({principals,clientSimulation,engineMode,representative,groups,enforcedPolicy,env})),{mode:0o600});
 const endpoints=new Map(Object.entries(groups).map(([name,list])=>[list,name]));
 const threads=env.MW_NATIVE_GENERATOR_THREADS?bounded(env,'MW_NATIVE_GENERATOR_THREADS',undefined,1,64):undefined;
 const runtime=env.MW_NATIVE_GENERATOR_RUNTIME??'multi';
 if(!['multi','per-core'].includes(runtime))throw new Error('Invalid MW_NATIVE_GENERATOR_RUNTIME');
 const rateGuard=env.MW_NATIVE_GENERATOR_RATE_GUARD!=='off';
 const tickMicros=bounded(env,'MW_NATIVE_GENERATOR_TICK_US',400,50,1000);
 let draftPool=[],stage=0,closed=false;
 const close=()=>{if(!closed){closed=true;rmSync(directory,{recursive:true,force:true});}};
 process.once('exit',close);
 async function run({base,bases,requests,concurrency,seconds,timeoutMs=10000,arrivalRate,commonStartAtMs,includeRawMeasurements=false,latePolicy='drop',h2}){
  const endpoint=endpoints.get(requests);if(!endpoint)throw new Error('Native generator: request list is not a prepared group');
  if(!Number.isInteger(concurrency)||concurrency<1||concurrency>MAX_OUTSTANDING||!Number.isFinite(seconds)||seconds<=0||seconds>MAX_STAGE_SECONDS||arrivalRate!==undefined&&(!Number.isFinite(arrivalRate)||arrivalRate<=0||arrivalRate>MAX_OFFERED_RPS))throw new Error('Invalid bounded measurement parameters');
  const id=stage++,stageFile=join(directory,`stage-${id}.json`),out=join(directory,`result-${id}.json`),poolOut=join(directory,`pool-${id}.private.json`);
  writeFileSync(stageFile,JSON.stringify({base:String(base),...(bases?{bases}:{}),endpoint,concurrency,seconds,timeoutMs,...(arrivalRate===undefined?{}:{arrivalRate}),latePolicy,...(commonStartAtMs===undefined?{}:{commonStartAtMs}),...(h2?{h2}:{}),draftPool,draftPoolOut:poolOut,...(threads?{threads}:{}),runtime,rateGuard,tickMicros}),{mode:0o600});
  try{
   await runBinary(bin,['stage',`--plan=${plan}`,`--stage=${stageFile}`,`--out=${out}`],{timeoutMs:(seconds+timeoutMs/1000+60)*1000});
   const result=JSON.parse(readFileSync(out,'utf8'));
   draftPool=JSON.parse(readFileSync(poolOut,'utf8'));
   return nativeStageResult(result,{requests,seconds,concurrency,arrivalRate,latePolicy,includeRawMeasurements});
  }finally{for(const file of [stageFile,out,poolOut])rmSync(file,{force:true});}
 }
 return {run,close,directory};
}
