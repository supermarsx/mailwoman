// Controlled read-only HTTP load. No credentials, response bodies or account IDs
// are written to the report. Run only against an isolated fixture you own.
import {writeFile,mkdir} from 'node:fs/promises';
import {dirname,resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
import {setTimeout as sleep} from 'node:timers/promises';
import {MAX_TARGETS} from './http-targets.mjs';

// Fetch exposes decoded body chunks, never TLS/header/on-wire byte counts.
// Generator scale limits (t53s). Defaults of every caller are unchanged; only the ceilings moved.
export const SCHEDULING_LAG_LIMIT_MS=1;
export const MAX_WORKERS_PER_HOST=64,MAX_WORKERS_TOTAL=256,MAX_OUTSTANDING=20000,MAX_STAGE_SECONDS=3600;
export function acceptedInScheduledWindow(status,ok,completed,deadline){
 return Boolean(ok&&Number(status)>=200&&Number(status)<300&&completed<=deadline);
}
export class Measurements {
  count=0; bytes=0; acceptedDecodedBytes=0; acceptedDuringScheduledWindow=0; acceptedDuringScheduledWindowDecodedBytes=0; errors=0; accepted=0; timeouts=0; totalMs=0; maximumMs=0;
  histogram=new Map(); statuses={};
  add(ms,status,bytes,ok,timeout=false,inWindow=false){
    const accepted=ok&&Number(status)>=200&&Number(status)<300;
    if(!Number.isSafeInteger(bytes)||bytes<0||typeof inWindow!=='boolean'||!Number.isSafeInteger(this.bytes+bytes)||!Number.isSafeInteger(this.acceptedDecodedBytes+bytes)||!Number.isSafeInteger(this.acceptedDuringScheduledWindowDecodedBytes+bytes))throw new Error('Response decoded-byte counter overflow');
    if(accepted){this.acceptedDecodedBytes+=bytes;if(inWindow){this.acceptedDuringScheduledWindow++;this.acceptedDuringScheduledWindowDecodedBytes+=bytes;}}
    this.count++;this.bytes+=bytes;this.totalMs+=ms;this.maximumMs=Math.max(this.maximumMs,ms);
    if(!ok)this.errors++;if(ok&&Number(status)>=200&&Number(status)<300)this.accepted++;if(timeout)this.timeouts++;
    this.statuses[status]=(this.statuses[status]??0)+1;
    const bucket=ms<=1000?Math.ceil(ms):ms<=10000?Math.ceil(ms/10)*10:Math.min(120100,Math.ceil(ms/100)*100);this.histogram.set(bucket,(this.histogram.get(bucket)??0)+1);
  }
  percentile(fraction){let n=0;const target=Math.ceil(this.count*fraction);for(const [ms,count]of [...this.histogram].sort((a,b)=>a[0]-b[0])){n+=count;if(n>=target)return ms;}return 0;}
  snapshot(){return {count:this.count,bytes:this.bytes,acceptedDecodedBytes:this.acceptedDecodedBytes,acceptedDuringScheduledWindow:this.acceptedDuringScheduledWindow,acceptedDuringScheduledWindowDecodedBytes:this.acceptedDuringScheduledWindowDecodedBytes,errors:this.errors,accepted:this.accepted,timeouts:this.timeouts,totalMs:this.totalMs,maximumMs:this.maximumMs,statuses:{...this.statuses},histogram:[...this.histogram].sort((a,b)=>a[0]-b[0])};}
  merge(value){
    const invalid=()=>{throw new Error('Invalid bounded raw measurements');};
    if(!value||['count','bytes','errors','accepted','timeouts','acceptedDecodedBytes','acceptedDuringScheduledWindow','acceptedDuringScheduledWindowDecodedBytes'].some(key=>!Number.isSafeInteger(value[key])||value[key]<0)||!Number.isFinite(value.totalMs)||value.totalMs<0||!Number.isFinite(value.maximumMs)||value.maximumMs<0||value.maximumMs>value.totalMs||value.acceptedDuringScheduledWindow>value.accepted||(value.acceptedDuringScheduledWindow===value.accepted&&value.acceptedDuringScheduledWindowDecodedBytes!==value.acceptedDecodedBytes)||value.acceptedDecodedBytes>value.bytes||value.acceptedDuringScheduledWindowDecodedBytes>value.acceptedDecodedBytes||(value.accepted===0&&value.acceptedDecodedBytes!==0)||(value.acceptedDuringScheduledWindow===0&&value.acceptedDuringScheduledWindowDecodedBytes!==0)||value.errors>value.count||value.accepted>value.count-value.errors||value.timeouts>value.errors||!Array.isArray(value.histogram)||value.histogram.length>3002||!value.statuses||Array.isArray(value.statuses)||typeof value.statuses!=='object')invalid();
    const bins=new Map();let samples=0;
    for(const entry of value.histogram){
      if(!Array.isArray(entry)||entry.length!==2)invalid();
      const [bucket,count]=entry;
      if(!Number.isInteger(bucket)||bucket<0||bucket>120100||(bucket>1000&&bucket<=10000&&bucket%10!==0)||(bucket>10000&&bucket%100!==0)||!Number.isSafeInteger(count)||count<1||bins.has(bucket))invalid();
      bins.set(bucket,count);samples+=count;
    }
    const statuses=Object.entries(value.statuses);let responses=0,successResponses=0;
    if(statuses.length>504)invalid();
    for(const [status,count]of statuses){
      if(!/^(?:[1-5][0-9]{2}|network|scheduled|campaign-aborted)$/.test(status)||!Number.isSafeInteger(count)||count<1)invalid();
      responses+=count;if(Number(status)>=200&&Number(status)<300)successResponses+=count;
    }
    if(samples!==value.count||responses!==value.count||value.accepted>successResponses||value.count===0&&(value.totalMs!==0||value.maximumMs!==0||value.bytes!==0)||['count','bytes','errors','accepted','timeouts','acceptedDecodedBytes','acceptedDuringScheduledWindow','acceptedDuringScheduledWindowDecodedBytes'].some(key=>!Number.isSafeInteger(this[key]+value[key]))||!Number.isFinite(this.totalMs+value.totalMs))invalid();
    for(const key of ['count','bytes','errors','accepted','timeouts','totalMs','acceptedDecodedBytes','acceptedDuringScheduledWindow','acceptedDuringScheduledWindowDecodedBytes'])this[key]+=value[key];
    this.maximumMs=Math.max(this.maximumMs,value.maximumMs);
    for(const [bucket,count]of bins)this.histogram.set(bucket,(this.histogram.get(bucket)??0)+count);
    for(const [status,count]of statuses)this.statuses[status]=(this.statuses[status]??0)+count;
    return this;
  }
  report(seconds){return {requests:this.count,requestsPerSecond:this.count/seconds,bytes:this.bytes,acceptedDecodedBytes:this.acceptedDecodedBytes,acceptedDuringScheduledWindow:this.acceptedDuringScheduledWindow,acceptedDuringScheduledWindowDecodedBytes:this.acceptedDuringScheduledWindowDecodedBytes,responseByteMeasurement:'fetch-decoded-body',wireResponseBytes:null,accepted:this.accepted,acceptedRequestsPerSecond:this.accepted/seconds,errors:this.errors,throttled:this.statuses[429]??0,timeouts:this.timeouts,statuses:this.statuses,latencyMs:{mean:this.count?this.totalMs/this.count:0,p50:this.percentile(.5),p95:this.percentile(.95),p99:this.percentile(.99),max:this.maximumMs},percentileResolutionMs:'1 through 1s; 10 through 10s; 100 thereafter; overflow 120100'};}
}

export function mergeMeasurements(snapshots,seconds){
  if(!Array.isArray(snapshots)||!snapshots.length||snapshots.length>MAX_WORKERS_TOTAL||!Number.isFinite(seconds)||seconds<=0||seconds>MAX_STAGE_SECONDS)throw new Error('Invalid bounded measurement merge');
  const merged=new Measurements();for(const snapshot of snapshots)merged.merge(snapshot);return merged.report(seconds);
}

const reason503Codes=Object.freeze(['publishedGuardFenceRefusal','authorizationNotConfirmedBeforeDispatch','requestProtectionUnavailable','empty503','other503']);
function reasons503(){return {schema:1,total:0,counts:Object.fromEntries(reason503Codes.map(code=>[code,0]))};}
// Never export arbitrary body fields or strings. An incomplete/large response
// cannot establish either an empty body or a known error code.
export function classifyHttp503(body,complete){
 if(!complete||!Buffer.isBuffer(body)||body.length>4096)return 'other503';
 if(body.length===0)return 'empty503';
 try{
  const value=JSON.parse(body.toString('utf8'));
  return value&&!Array.isArray(value)&&typeof value==='object'&&reason503Codes.slice(0,3).includes(value.code)?value.code:'other503';
 }catch{return 'other503';}
}
export function mergeHttp503Reasons(values,statusCounts){
 if(!Array.isArray(values)||!values.length||values.length>MAX_WORKERS_TOTAL||!Array.isArray(statusCounts)||statusCounts.length!==values.length)throw Error('Invalid HTTP503 reason proof');
 if(values.every(value=>value===undefined))return null;
 const result=reasons503();
 for(let index=0;index<values.length;index++){
  const value=values[index],count=statusCounts[index];
  if(!value||Object.keys(value).sort().join(',')!=='counts,schema,total'||value.schema!==1||!Number.isSafeInteger(count)||count<0||value.total!==count||!value.counts||Object.keys(value.counts).sort().join(',')!==[...reason503Codes].sort().join(','))throw Error('Invalid HTTP503 reason proof');
  let sum=0;
  for(const code of reason503Codes){const n=value.counts[code];if(!Number.isSafeInteger(n)||n<0||!Number.isSafeInteger(result.counts[code]+n))throw Error('Invalid HTTP503 reason proof');sum+=n;}
  if(sum!==count||!Number.isSafeInteger(result.total+count))throw Error('Inconsistent HTTP503 reason proof');
  for(const code of reason503Codes)result.counts[code]+=value.counts[code];result.total+=count;
 }
 return result;
}

// Separate histograms: legacy latencyMs continues to include principal queue
// time. These are observed client intervals, never isolated server CPU/DB time.
const timingNames=['principalQueue','httpAndValidation','validation'];
function timings(){return Object.fromEntries(timingNames.map(name=>[name,new Measurements()]));}
function timingSnapshot(value){return Object.fromEntries(timingNames.map(name=>[name,value[name].snapshot()]));}
function timingReport(value,count){return {schema:1,...Object.fromEntries(timingNames.map(name=>{
 const report=value[name].report(1);return [name,{samples:report.requests,...report.latencyMs}];
})),undispatched:count-value.httpAndValidation.count,scope:'client monotonic wall time; HTTP includes Fetch, decoded body read and validation; validation is a nested subset; queue includes zero-wait samples; not server CPU or isolated DB time'};}
export function mergeLatencyBreakdowns(snapshots,counts,dispatches){
 if(!Array.isArray(snapshots)||!snapshots.length||snapshots.length>MAX_WORKERS_TOTAL||snapshots.length!==counts.length||dispatches&&dispatches.length!==counts.length)throw Error('Invalid latency breakdown');
 if(snapshots.every(value=>value===undefined))return null;
 const merged=timings();let total=0;
 for(let index=0;index<snapshots.length;index++){
  const value=snapshots[index],count=counts[index];
  if(!value||Object.keys(value).sort().join(',')!==[...timingNames].sort().join(',')||!Number.isSafeInteger(count)||count<0)throw Error('Invalid latency breakdown');
  const checked=timings();
  for(const name of timingNames){checked[name].merge(value[name]);const p=checked[name];if(p.bytes!==0||p.errors!==0||p.accepted!==0||p.timeouts!==0||Object.keys(p.statuses).some(status=>status!=='scheduled'))throw Error('Invalid latency breakdown');}
  if(checked.principalQueue.count!==count||checked.httpAndValidation.count>count||checked.validation.count>checked.httpAndValidation.count||checked.validation.totalMs>checked.httpAndValidation.totalMs+1e-6||checked.validation.maximumMs>checked.httpAndValidation.maximumMs+1e-6||dispatches&&(checked.httpAndValidation.count!==dispatches[index]))throw Error('Inconsistent latency breakdown');
  for(const name of timingNames)merged[name].merge(value[name]);
  total+=count;
 }
 return timingReport(merged,total);
}

export async function measure({base,requests,concurrency,seconds,timeoutMs=10000,headers={},arrivalRate,commonStartAtMs,includeRawMeasurements=false,latePolicy='drop',bases}){
  const aggregate=new Measurements(),byRequest=new Map(requests.map(request=>[request.name,new Measurements()]));
  if(!Number.isInteger(concurrency)||concurrency<1||concurrency>MAX_OUTSTANDING||!Number.isFinite(seconds)||seconds<=0||seconds>MAX_STAGE_SECONDS||!requests.length||requests.length>100||!Number.isFinite(timeoutMs)||timeoutMs<1||timeoutMs>120000)throw new Error('Invalid bounded measurement parameters');
  if(typeof includeRawMeasurements!=='boolean')throw new Error('Invalid raw measurement option');
  if(bases!==undefined&&(!Array.isArray(bases)||!bases.length||bases.length>MAX_TARGETS||bases.some(value=>typeof value!=='string')))throw new Error('Invalid target rotation list');
  if(!['drop','dispatch'].includes(latePolicy)||latePolicy==='dispatch'&&arrivalRate===undefined)throw new Error('Late policy must be drop or dispatch, and dispatch needs an arrival rate');
  const lateDispatch=latePolicy==='dispatch',acceptedPerSecond=new Array(Math.ceil(seconds)).fill(0);let lateSlotsDispatched=0;
  const wallOrigin=Date.now(),monotonicOrigin=performance.now();
  if(commonStartAtMs!==undefined&&(!Number.isSafeInteger(commonStartAtMs)||Math.abs(commonStartAtMs-wallOrigin)>60000))throw new Error('Common fixture start must be within sixty seconds');
  const start=commonStartAtMs===undefined?monotonicOrigin:monotonicOrigin+(commonStartAtMs-wallOrigin),deadline=start+seconds*1000;
  while(performance.now()<start)await sleep(Math.min(1000,start-performance.now()));
  const actualStart=performance.now();let sequence=0,acceptedWithinWindow=0,activeCount=0;
  const cpuStart=process.cpuUsage(),memoryStart=process.memoryUsage();
  const participant=()=>({attempted:new Map(),accepted:new Map(),withinWindow:new Set()});
  const principals=participant(),clients=participant();
  const boundedIndex=value=>Number.isInteger(value)&&value>=0&&value<10000;
  const increment=(counts,index)=>{if(boundedIndex(index))counts.set(index,(counts.get(index)??0)+1);};
  const distribution=counts=>{let total=0,min=Infinity,max=0;for(const count of counts.values()){total+=count;min=Math.min(min,count);max=Math.max(max,count);}return {participants:counts.size,requests:total,min:counts.size?min:0,max,mean:counts.size?total/counts.size:0};};
  const scheduling=new Measurements(),arrivalLatency=new Measurements();
  const principalQueues=new Map();
  const error503=reasons503(),error503ByRequest=new Map(requests.map(request=>[request.name,reasons503()]));
  const latency=timings(),latencyByRequest=new Map(requests.map(request=>[request.name,timings()]));
  const taintedPrincipals=new Set();let mutationStateTainted=false,unknownMutationOutcomes=0,httpDispatchAttempts=0,blockedAfterMutationTaint=0,abortedUnscheduled=0;
  const send=async(planned=performance.now())=>{
      const index=sequence++,template=requests[index%requests.length],spec=template.materialize?{...template,...template.materialize(index,{requestCount:requests.length}),name:template.name}:template,began=performance.now();
      increment(principals.attempted,spec.principalIndex);increment(clients.attempted,spec.clientIndex);
      peakOutstanding=Math.max(peakOutstanding,++activeCount);
      scheduling.add(Math.max(0,began-planned),'scheduled',0,true);if(lateDispatch&&began-planned>100)lateSlotsDispatched++;
      let status='network',bytes=0,ok=false,timedOut=false;
      const signal=AbortSignal.timeout(timeoutMs);
      let unlock=null,dispatched=false,dispatchAt=null,queueMs=0,validationMs=null,reason503='other503';
      try{
        if(spec.serializePrincipal){
          if(!boundedIndex(spec.principalIndex))throw new Error('Serialized fixture requests require a bounded principal');
          const previous=principalQueues.get(spec.principalIndex)??Promise.resolve();
          let release;const current=new Promise(resolve=>{release=resolve;});
          principalQueues.set(spec.principalIndex,current);
          unlock=()=>{release();if(principalQueues.get(spec.principalIndex)===current)principalQueues.delete(spec.principalIndex);};
          const queuedAt=performance.now();
          try{await previous;}finally{queueMs=performance.now()-queuedAt;}
          if(mutationStateTainted){status='campaign-aborted';blockedAfterMutationTaint++;throw new Error('Mutation outcome is unknown; fixture state cannot be reused');}
          signal.throwIfAborted();
        }
        if(mutationStateTainted){status='campaign-aborted';blockedAfterMutationTaint++;throw new Error('Mutation outcome is unknown; fixture state cannot be reused');}
        dispatched=true;httpDispatchAttempts++;dispatchAt=performance.now();
        const response=await fetch(new URL(spec.path,bases?bases[index%bases.length]:base),{method:spec.method??'GET',headers:{...headers,...spec.headers},...(spec.body!==undefined?{body:spec.body}:{}),signal,redirect:'error'});
        status=String(response.status);const chunks=[];let errorBody=status==='503'?Buffer.allocUnsafe(4096):null;const reader=response.body?.getReader();
        if(reader)for(;;){const part=await reader.read();if(part.done)break;bytes+=part.value.length;if(errorBody){if(bytes<=4096)errorBody.set(part.value,bytes-part.value.length);else errorBody=null;}if(bytes>8*1024*1024){await reader.cancel();throw new Error('response too large');}if(spec.validate||spec.validateBytes){if(bytes>(spec.validate?65536:1024*1024)){await reader.cancel();throw new Error('validation response too large');}chunks.push(part.value);}}
        if(status==='503')reason503=classifyHttp503(errorBody?errorBody.subarray(0,bytes):null,true);
        ok=(spec.statuses??[200]).includes(response.status);
        if(ok&&response.status!==429&&(spec.validate||spec.validateBytes)){
          const validationAt=performance.now();
          try{
            if(spec.validate)ok=spec.validate(JSON.parse(Buffer.concat(chunks).toString('utf8')));
            if(ok&&spec.validateBytes)ok=spec.validateBytes(Buffer.concat(chunks));
          }finally{validationMs=performance.now()-validationAt;}
        }
      }catch{timedOut=status!=='campaign-aborted'&&signal.aborted;ok=false;}finally{
        // A client abort does not cancel a server mutation. Fence the queue
        // before releasing it and never reuse an uncertain fixture state.
        if(spec.statefulMutation&&dispatched&&!ok){mutationStateTainted=true;unknownMutationOutcomes++;if(boundedIndex(spec.principalIndex))taintedPrincipals.add(spec.principalIndex);}
        unlock?.();
      }
      const completed=performance.now(),elapsed=lateDispatch?completed-planned:completed-began,inWindow=acceptedInScheduledWindow(status,ok,completed,deadline);
      if(status==='503')for(const value of [error503,error503ByRequest.get(spec.name)]){value.total++;value.counts[reason503]++;}
      for(const value of [latency,latencyByRequest.get(spec.name)]){
        value.principalQueue.add(queueMs,'scheduled',0,true);
        if(dispatchAt!==null)value.httpAndValidation.add(completed-dispatchAt,'scheduled',0,true);
        if(validationMs!==null)value.validation.add(validationMs,'scheduled',0,true);
      }
      if(ok&&Number(status)>=200&&Number(status)<300){
        increment(principals.accepted,spec.principalIndex);increment(clients.accepted,spec.clientIndex);
        if(inWindow){acceptedWithinWindow++;acceptedPerSecond[Math.min(acceptedPerSecond.length-1,Math.max(0,Math.floor((completed-start)/1000)))]++;if(boundedIndex(spec.principalIndex))principals.withinWindow.add(spec.principalIndex);if(boundedIndex(spec.clientIndex))clients.withinWindow.add(spec.clientIndex);}
      }
      aggregate.add(elapsed,status,bytes,ok,timedOut,inWindow);arrivalLatency.add(performance.now()-planned,status,bytes,ok,timedOut,inWindow);byRequest.get(spec.name).add(elapsed,status,bytes,ok,timedOut,inWindow);activeCount--;
  };
  let offered=0,droppedCapacity=0,droppedScheduler=0,droppedLateSlots=0,peakOutstanding=0;
  if(arrivalRate!==undefined){
    if(!Number.isFinite(arrivalRate)||arrivalRate<=0||arrivalRate>100000)throw new Error('Invalid arrival rate');
    const pending=new Set(),interval=1000/arrivalRate,total=Math.floor(seconds*arrivalRate);
    // Batch due arrivals because JS timers cannot represent 10us slots. No
    // unbounded work queue: discard slots over 100ms late and cap outstanding.
    for(let slot=0;slot<total;){
      if(mutationStateTainted){abortedUnscheduled=total-slot;break;}
      const now=performance.now(),due=start+slot*interval;
      if(now<due){await sleep(Math.max(1,due-now));continue;}
      if(now>=deadline){const missed=total-slot;droppedScheduler+=missed;offered+=missed;break;}
      const oldest=lateDispatch?slot:Math.min(total,Math.max(slot,Math.ceil((now-start-100)/interval)));
      if(oldest>slot){droppedLateSlots+=oldest-slot;droppedScheduler+=oldest-slot;offered+=oldest-slot;slot=oldest;}
      // late=dispatch: a due slot is never discarded; at the outstanding cap it waits and its lag is measured.
      const dueEnd=Math.min(total,Math.floor((now-start)/interval)+1);
      const dispatch=Math.min(dueEnd-slot,concurrency-pending.size),end=lateDispatch?slot+dispatch:dueEnd;
      const skipped=end-slot-dispatch;
      droppedCapacity+=skipped;offered+=end-slot;
      for(let i=0;i<dispatch;i++){
        const planned=start+(slot+i)*interval,operation=send(planned);
        pending.add(operation);peakOutstanding=Math.max(peakOutstanding,pending.size);
        void operation.finally(()=>pending.delete(operation));
      }
      slot=end;
      // Yield even when arrivals remain due, allowing completions and timers.
      await sleep(1);
    }
    await Promise.all(pending);
    while(!mutationStateTainted&&performance.now()<deadline)await sleep(Math.max(1,deadline-performance.now()));
  }else{
    await Promise.all(Array.from({length:concurrency},async()=>{while(!mutationStateTainted&&performance.now()<deadline)await send();}));
  }
  const completedAt=performance.now(),elapsed=(completedAt-start)/1000;
  const cpu=process.cpuUsage(cpuStart),memoryEnd=process.memoryUsage();
  const cpuElapsed=commonStartAtMs===undefined?elapsed:Math.max(.000001,(completedAt-actualStart)/1000);
  const hooks={
    ...(commonStartAtMs===undefined?{}:{window:{requestedStartAtMs:commonStartAtMs,scheduledStartAtMs:commonStartAtMs,scheduledEndAtMs:commonStartAtMs+seconds*1000,actualStartAtMs:Math.round(wallOrigin+actualStart-monotonicOrigin),completedAtMs:Math.round(wallOrigin+completedAt-monotonicOrigin),startLagMs:Math.max(0,actualStart-start)}}),
    latencyBreakdown:timingReport(latency,aggregate.count),http503Reasons:error503,
    ...(includeRawMeasurements?{rawMeasurements:{schema:1,aggregate:aggregate.snapshot(),byRequest:Object.fromEntries([...byRequest].map(([name,measurements])=>[name,measurements.snapshot()])),scheduling:scheduling.snapshot(),arrivalLatency:arrivalLatency.snapshot(),http503Reasons:{schema:1,aggregate:error503,byRequest:Object.fromEntries(error503ByRequest)},latencyBreakdown:{schema:1,aggregate:timingSnapshot(latency),byRequest:Object.fromEntries([...latencyByRequest].map(([name,value])=>[name,timingSnapshot(value)]))}}}:{}),
  };
  return {...hooks,mutationStateTainted,unknownMutationOutcomes,taintedPrincipals:taintedPrincipals.size,httpDispatchAttempts,blockedAfterMutationTaint,abortedUnscheduled,uniquePrincipalsAccepted:principals.accepted.size,uniquePrincipalsAttempted:principals.attempted.size,uniquePrincipalsAcceptedDuringScheduledWindow:principals.withinWindow.size,uniqueClientsAttempted:clients.attempted.size,uniqueClientsAccepted:clients.accepted.size,uniqueClientsAcceptedDuringScheduledWindow:clients.withinWindow.size,participantDistribution:{principals:{attempted:distribution(principals.attempted),accepted:distribution(principals.accepted)},simulatedClients:{attempted:distribution(clients.attempted),accepted:distribution(clients.accepted)}},peakOutstanding,acceptedDuringScheduledWindow:acceptedWithinWindow,acceptedDuringScheduledWindowRps:acceptedWithinWindow/seconds,acceptedPerSecond,generator:{cpuUserMs:cpu.user/1000,cpuSystemMs:cpu.system/1000,cpuLogicalCores:(cpu.user+cpu.system)/(cpuElapsed*1000000),rssBeforeBytes:memoryStart.rss,rssAfterBytes:memoryEnd.rss,heapAfterBytes:memoryEnd.heapUsed},concurrency,scheduledSeconds:seconds,elapsedSeconds:elapsed,...(arrivalRate===undefined?{}:{arrivalRate,offered,attempted:aggregate.count,attemptedRequestsPerSecond:aggregate.count/seconds,achievedRequestsPerSecond:aggregate.count/elapsed,droppedCapacity,droppedScheduler,peakOutstanding,latePolicy,lateSlotsDispatched,droppedLateSlots,unissuedAtWindowEnd:droppedScheduler-droppedLateSlots,latencyBasis:lateDispatch?'scheduled-slot':'send',generatorLimited:scheduling.percentile(.99)>SCHEDULING_LAG_LIMIT_MS,schedulingLagLimitMs:SCHEDULING_LAG_LIMIT_MS,schedulingLagMs:scheduling.report(elapsed).latencyMs,plannedArrivalLatencyMs:arrivalLatency.report(elapsed).latencyMs}),...aggregate.report(elapsed),byRequest:Object.fromEntries([...byRequest].map(([key,value])=>[key,{...value.report(elapsed),latencyBreakdown:timingReport(latencyByRequest.get(key),value.count),http503Reasons:error503ByRequest.get(key)}]))};
}

function limitedNote(report){
 const rows=report.stages.filter(stage=>stage.generatorLimited===true);
 return rows.length?`\nGenerator-limited stages (scheduling lag p99 above ${SCHEDULING_LAG_LIMIT_MS} ms, so the generator rather than the target may explain a shortfall): ${rows.map(stage=>`${stage.phase??'smoke'} / ${stage.endpoint??'mixed'} / ${stage.arrivalRate}`).join('; ')}.\n`:'';
}
export function markdownReport(report){
 const perUserRange=stage=>{const accepted=stage.participantDistribution?.principals?.accepted;return accepted&&Number.isFinite(accepted.min)&&Number.isFinite(accepted.max)?`${accepted.min} / ${accepted.max}`:'not measured';};
 const readiness=report.startupReadiness;
 const startup=readiness?.verified===true?`Startup index readiness verified: ${readiness.indexDocuments}/${readiness.expectedIndexDocuments} documents; ${Number(readiness.startupSeconds).toFixed(1)} s startup, ${Number(readiness.indexWaitSeconds).toFixed(1)} s index wait; ${readiness.zeroAccessExcludedMessages} policy-excluded messages.`:'Startup index readiness was not measured in this report.';
 const layerTable='\n\nClient latency layers (all completions, including failures and drain):\n\n| Phase / endpoint | Queue mean / p50 / p95 / p99 ms | Dispatched HTTP + validation mean / p50 / p95 / p99 ms | Nested validation mean / p50 / p95 / p99 ms | Dispatched / validation samples |\n| --- | ---: | ---: | ---: | ---: |\n'+report.stages.map(stage=>{
  const value=stage.latencyBreakdown,cell=part=>part.samples===0?'not measured':`${part.mean.toFixed(2)} / ${part.p50} / ${part.p95} / ${part.p99}`;
  return `| ${stage.phase??'smoke'} / ${stage.endpoint??'mixed'} | ${value?cell(value.principalQueue):'not measured'} | ${value?cell(value.httpAndValidation):'not measured'} | ${value?cell(value.validation):'not measured'} | ${value?`${value.httpAndValidation.samples} / ${value.validation.samples}`:'not measured'} |`;
 }).join('\n')+'\n\nLegacy latencyMs above includes principal serialization wait. HTTP + validation starts immediately before Fetch and includes decoded body reads; validation is nested inside it. These intervals are client wall time, not isolated Rust CPU or DB time. Percentiles cannot be subtracted or added to attribute server layers. Historical reports without phase proof remain unreported.\n';
 return `# HTTP load report\n\nTarget: ${report.target}\n\nScenario: ${report.scenario}; runtime: ${report.runtime}. ${report.model}.${report.protection?' Protection: '+report.protection+'.':''}\n\n${startup}\n\n| Phase / endpoint / outstanding requests | Fixture protection | Offered RPS | Attempted | Accepted | Completed RPS | Accepted RPS | p50/p95/p99 ms | Errors | 429 | Timeouts | Capacity/scheduler drops | Lag p99 ms |\n| --- | --- | ---: | ---: | ---: | ---: | ---: | --- | ---: | ---: | ---: | --- | ---: |\n`+report.stages.map(s=>`| ${s.phase??'smoke'} / ${s.endpoint??'mixed'} / ${s.concurrency} | ${s.fixtureProtection===undefined?'see scenario':s.fixtureProtection?'on (simulated)':'off (simulated)'} | ${s.arrivalRate??'closed-loop'} | ${s.requests} | ${s.accepted} | ${s.requestsPerSecond.toFixed(1)} | ${s.acceptedRequestsPerSecond.toFixed(1)} | ${s.latencyMs.p50}/${s.latencyMs.p95}/${s.latencyMs.p99} | ${s.errors} | ${s.throttled} | ${s.timeouts} | ${s.droppedCapacity??0}/${s.droppedScheduler??0} | ${s.schedulingLagMs?.p99??0} |`).join('\n')+'\n\n| Phase / endpoint | Distinct users attempted / accepted | Simulated clients attempted / accepted | Accepted requests per user min / max |\n| --- | ---: | ---: | ---: |\n'+report.stages.map(s=>`| ${s.phase??'smoke'} / ${s.endpoint??'mixed'} | ${s.uniquePrincipalsAttempted??0} / ${s.uniquePrincipalsAccepted??0} | ${s.uniqueClientsAttempted??0} / ${s.uniqueClientsAccepted??0} | ${perUserRange(s)} |`).join('\n')+'\n\nSimulated client addresses are a fixture proxy model, not separate physical generator hosts. Zero tracked identities is not evidence of multi-user load. Dropped arrivals have no response latency and must be evaluated alongside percentiles. This report alone is not a production capacity claim.\n'+limitedNote(report)+layerTable;
}

async function main(){
 const args=Object.fromEntries(process.argv.slice(2).map(arg=>{const split=arg.indexOf('=');if(split<0)throw new Error('Use --name=value options');return[arg.slice(2,split),arg.slice(split+1)];}));
 const allowed=new Set(['base','concurrency','seconds','warmup','timeout','scenario','output','allow-remote','max-p95','max-error-rate','allow-throttles','max-throttle-rate','arrival-rate','max-drop-rate','profile','rates','acknowledge-staging']);for(const key of Object.keys(args))if(!allowed.has(key))throw new Error(`Unknown option: ${key}`);
 if(!args.base)throw new Error('An explicit --base= origin is required');
 const base=new URL(args.base);
 if(!['http:','https:'].includes(base.protocol)||base.username||base.password||base.pathname!=='/'||base.search||base.hash)throw new Error('Base must be a plain HTTP(S) origin');
 if(!['127.0.0.1','localhost','[::1]'].includes(base.hostname)&&(args['allow-remote']!=='true'||args['acknowledge-staging']!==base.origin))throw new Error('Remote load requires --allow-remote=true and --acknowledge-staging matching the explicit origin');
 const levels=(args.concurrency??'1,8,32,64').split(',').map(Number),seconds=Number(args.seconds??30),warmup=Number(args.warmup??3),timeoutMs=Number(args.timeout??10000);
 if(!levels.length||levels.length>10||levels.some(n=>!Number.isInteger(n)||n<1||n>4096)||!Number.isFinite(seconds)||seconds<1||seconds>600||!Number.isFinite(warmup)||warmup<0||warmup>60||!Number.isFinite(timeoutMs)||timeoutMs<100||timeoutMs>120000)throw new Error('Invalid bounded load parameters');
 const arrivalRate=args['arrival-rate']===undefined?undefined:Number(args['arrival-rate']),dropRate=Number(args['max-drop-rate']??0);
 if((arrivalRate!==undefined&&(!Number.isFinite(arrivalRate)||arrivalRate<1||arrivalRate>100000))||!Number.isFinite(dropRate)||dropRate<0||dropRate>1)throw new Error('Invalid arrival or drop budget');
 const p95=Number(args['max-p95']??Infinity),errorRate=Number(args['max-error-rate']??0),throttleRate=Number(args['max-throttle-rate']??(args['allow-throttles']==='true'?1:0));
 if(!(p95>0)||!Number.isFinite(errorRate)||errorRate<0||errorRate>1||!Number.isFinite(throttleRate)||throttleRate<0||throttleRate>1)throw new Error('Invalid report budgets');
 const scenario=args.scenario??'public';if(!['public','mail','malformed'].includes(scenario))throw new Error('Scenario must be public, mail or malformed');
 const headers={},jar=new Map();let ownLogin=false;
 const remember=response=>{for(const cookie of response.headers.getSetCookie()){const pair=cookie.split(';',1)[0],at=pair.indexOf('=');jar.set(pair.slice(0,at),pair.slice(at+1));}headers.cookie=[...jar].map(([key,value])=>`${key}=${value}`).join('; ');if(jar.has('mw_csrf'))headers['x-csrf-token']=decodeURIComponent(jar.get('mw_csrf'));};
 if(process.env.MAILWOMAN_LOAD_BEARER)headers.authorization=`Bearer ${process.env.MAILWOMAN_LOAD_BEARER}`;
 if(scenario!=='public'&&!headers.authorization){
   if(!process.env.MAILWOMAN_LOAD_USERNAME||!process.env.MAILWOMAN_LOAD_PASSWORD)throw new Error('Mail/malformed scenarios require fixture credentials or MAILWOMAN_LOAD_BEARER');
   const bootstrap=await fetch(base,{redirect:'error',signal:AbortSignal.timeout(timeoutMs)});remember(bootstrap);await bootstrap.arrayBuffer();
   const login=await fetch(new URL('/api/login',base),{method:'POST',headers:{...headers,'content-type':'application/json'},body:JSON.stringify({username:process.env.MAILWOMAN_LOAD_USERNAME,password:process.env.MAILWOMAN_LOAD_PASSWORD}),redirect:'error',signal:AbortSignal.timeout(30000)});
   if(login.status!==200){await login.body?.cancel();throw new Error(`Fixture login refused (${login.status}); no automatic credential retry`);}remember(login);await login.arrayBuffer();ownLogin=true;
 }
 let requests=[{name:'health',path:'/healthz'},{name:'public-config',path:'/api/public-config'},{name:'application-shell',path:'/'}];
 if(scenario!=='public'){
   const session=await fetch(new URL('/jmap/session',base),{headers,redirect:'error',signal:AbortSignal.timeout(timeoutMs)});if(!session.ok)throw new Error('Fixture session unavailable');const payload=await session.json(),accountId=payload.primaryAccounts?.['urn:ietf:params:jmap:mail'];if(!accountId)throw new Error('No primary mail account');
   const jmap=(name,args)=>({name,path:'/jmap/api',method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({using:['urn:ietf:params:jmap:core','urn:ietf:params:jmap:mail'],methodCalls:[[name,{accountId,...args},'load']]}),validate:value=>value.methodResponses?.length===1&&value.methodResponses[0][0]===name&&value.methodResponses[0][1].accountId===accountId});
   requests=[{name:'health',path:'/healthz'},{name:'current-user',path:'/api/me'},{name:'mail-session',path:'/jmap/session'},jmap('Mailbox/get',{properties:['id','name','totalEmails','unreadEmails']}),jmap('Email/query',{limit:25,sort:[{property:'receivedAt',isAscending:false}]})];
   if(scenario==='malformed')requests=[{name:'health',path:'/healthz'},...['{','{"methodCalls":false}','null','{"using":[],"methodCalls":[42]}'].map((body,i)=>({name:`invalid-json-envelope-${i}`,path:'/jmap/api',method:'POST',headers:{'content-type':'application/json'},body,statuses:[400,422]}))];
 }
 if(args['allow-throttles']==='true')requests=requests.map(request=>request.name==='health'?request:{...request,statuses:[...(request.statuses??[200]),429]});
 const profile=args.profile??'sustained',rates=(args.rates??'20000,50000,100000').split(',').map(Number);
 if(!['sustained','burst','peak'].includes(profile)||rates.length>10||!rates.length||rates.some(n=>!Number.isFinite(n)||n<1||n>100000))throw new Error('Invalid bounded profile/rates');
 const stages=args.profile?(profile==='sustained'?rates.map(rate=>({rate,duration:seconds})):profile==='burst'?[{rate:rates[0],duration:seconds},{rate:rates.at(-1),duration:Math.min(5,seconds)},{rate:rates[0],duration:seconds}]:[...rates,...rates.slice(0,-1).reverse()].map(rate=>({rate,duration:seconds}))):[{rate:arrivalRate,duration:seconds}];
 const report={profile:args.profile??'custom',generatedAt:new Date().toISOString(),target:base.origin,scenario,model:!args.profile&&arrivalRate===undefined?'single-session diagnostic; closed-loop concurrent requests; not multi-user capacity':'single-session diagnostic; fixed arrival rate with bounded outstanding requests; dropped arrivals reported separately',runtime:process.version,workload:'single-session-diagnostic',stages:[]};
 try{
   if(warmup)await measure({base,requests,headers,concurrency:1,seconds:warmup,timeoutMs});
   for(const concurrency of levels)for(const stage of stages){const result=await measure({base,requests,headers,concurrency,seconds:stage.duration,timeoutMs,arrivalRate:stage.rate});report.stages.push(result);console.log(`${scenario}: outstanding=${concurrency}, requests=${result.requests}, rps=${result.requestsPerSecond.toFixed(1)}, p50=${result.latencyMs.p50}ms, p95=${result.latencyMs.p95}ms, p99=${result.latencyMs.p99}ms, errors=${result.errors}, throttled=${result.throttled}, timeouts=${result.timeouts}, dropped=${(result.droppedCapacity??0)+(result.droppedScheduler??0)}`);}
   const health=await fetch(new URL('/healthz',base),{redirect:'error',signal:AbortSignal.timeout(timeoutMs)});report.healthyAfter=health.status===200;await health.body?.cancel();
 }finally{if(ownLogin){try{const response=await fetch(new URL('/api/logout',base),{method:'POST',headers,redirect:'error',signal:AbortSignal.timeout(timeoutMs)});await response.body?.cancel();}catch{/* Report measurements even if fixture-session cleanup fails. */}}}
 const output=resolve(args.output??'.orchestration/reports/http-load.json');await mkdir(dirname(output),{recursive:true});await writeFile(output,JSON.stringify(report,null,2)+'\n');await writeFile(output.replace(/\.json$/,'')+'.md',markdownReport(report));

 if(!report.healthyAfter||report.stages.some(stage=>stage.latencyMs.p95>p95||stage.errors/Math.max(1,stage.requests)>errorRate||stage.throttled/Math.max(1,stage.requests)>throttleRate||((stage.droppedCapacity??0)+(stage.droppedScheduler??0))/Math.max(1,stage.offered??0)>dropRate))process.exitCode=1;
}
if(process.argv[1]&&import.meta.url===pathToFileURL(resolve(process.argv[1])).href)main().catch(error=>{console.error(error.message);process.exitCode=1;});
