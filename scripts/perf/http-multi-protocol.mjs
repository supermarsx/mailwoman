// Aggregate only: credentials and principal identifiers never enter this format.
import {mergeMeasurements,mergeLatencyBreakdowns,mergeHttp503Reasons,MAX_WORKERS_TOTAL,MAX_OUTSTANDING,MAX_STAGE_SECONDS} from './http-load.mjs';
export const MULTI_SCHEMA=1;
export const ENDPOINT_LABELS=['authenticated-identity','Mailbox/get','Email/query','Email/get','concurrent-refresh-handler','paged-query-and-headers','body-and-attachment-metadata','attachment-download','mark-read-and-restore','move-and-restore','jmap-session','list-batch','changes','account-preferences','appearance-customization','session-activity'];
const number=(value)=>typeof value==='number'&&Number.isFinite(value)&&value>=0;
const integer=value=>Number.isSafeInteger(value)&&value>=0;
export function splitBudget(total,count){
 if(!Number.isInteger(count)||count<1||count>MAX_WORKERS_TOTAL||!Number.isInteger(total)||total<count||total>100000)throw new Error('Invalid exact worker budget');
 return Array.from({length:count},(_,index)=>Math.floor(total/count)+(index<total%count?1:0));
}
export function partitionPrincipals(principals,workers){
 if(!Array.isArray(principals)||principals.length>10000)throw new Error('Invalid principal allocation');
 let offset=0;
 return splitBudget(principals.length,workers).map((count,index)=>{
  const result={index,offset,count,principals:principals.slice(offset,offset+count)};offset+=count;return result;
 });
}
export function validateStage({workers,concurrency,seconds,arrivalRate,timeoutMs}){
 if(!Number.isInteger(workers)||workers<1||workers>MAX_WORKERS_TOTAL||!Number.isInteger(concurrency)||concurrency<workers||concurrency>MAX_OUTSTANDING||!Number.isInteger(seconds)||seconds<1||seconds>MAX_STAGE_SECONDS||!Number.isInteger(timeoutMs)||timeoutMs<100||timeoutMs>30000||arrivalRate!==undefined&&(!Number.isInteger(arrivalRate)||arrivalRate<workers||arrivalRate>100000))throw new Error('Invalid coordinated stage');
 return {concurrency:splitBudget(concurrency,workers),...(arrivalRate===undefined?{}:{rates:splitBudget(arrivalRate,workers)})};
}
// Reuse the generator's exact bounded histogram/counter validator.
export function mergeSnapshots(snapshots,seconds){
 return mergeMeasurements(snapshots,seconds);
}
export function aggregateWorkers({results,partitions,stage,startAtMs,maxStartLagMs=50}){
 const budget=validateStage({...stage,workers:partitions.length});
 if(results.length!==partitions.length||!number(startAtMs)||!number(maxStartLagMs))throw new Error('Incomplete worker phase');
 const ordered=new Map();
 for(const result of results){
  if(!Number.isInteger(result?.workerIndex)||ordered.has(result.workerIndex))throw new Error('Duplicate worker phase');
  ordered.set(result.workerIndex,result);
 }
 let completedAtMs=startAtMs,worstStartLagMs=0;
 for(const part of partitions){
  const value=ordered.get(part.index),window=value?.measurement?.window,raw=value?.measurement?.rawMeasurements;
  if(value?.schema!==1||value.partitionCount!==part.count||!window||!raw||raw.schema!==1||window.requestedStartAtMs!==startAtMs||window.scheduledStartAtMs!==startAtMs||window.scheduledEndAtMs!==startAtMs+stage.seconds*1000||!number(window.actualStartAtMs)||window.actualStartAtMs<startAtMs-1||!number(window.completedAtMs)||window.completedAtMs<window.actualStartAtMs||(!value.measurement.mutationStateTainted&&window.completedAtMs<window.scheduledEndAtMs)||!number(window.startLagMs)||window.startLagMs>maxStartLagMs||Math.abs(window.actualStartAtMs-startAtMs-window.startLagMs)>2)throw new Error('Mismatched or skewed worker window');
  completedAtMs=Math.max(completedAtMs,window.completedAtMs);worstStartLagMs=Math.max(worstStartLagMs,window.startLagMs);
  const sample=value.measurement;
  if(typeof sample.mutationStateTainted!=='boolean'||!integer(sample.peakOutstanding)||sample.peakOutstanding>budget.concurrency[part.index])throw new Error('Invalid worker completion state');
  for(const key of ['unknownMutationOutcomes','taintedPrincipals','httpDispatchAttempts','blockedAfterMutationTaint','abortedUnscheduled'])if(!integer(sample[key]))throw new Error('Invalid mutation outcome counters');
  if(sample.mutationStateTainted!==(sample.unknownMutationOutcomes>0)||sample.unknownMutationOutcomes>raw.aggregate.errors||sample.taintedPrincipals>part.count||sample.taintedPrincipals>sample.unknownMutationOutcomes||sample.httpDispatchAttempts>raw.aggregate.count||sample.blockedAfterMutationTaint>raw.aggregate.errors||sample.abortedUnscheduled>0&&!sample.mutationStateTainted)throw new Error('Inconsistent mutation outcome counters');
  for(const key of ['uniquePrincipalsAttempted','uniquePrincipalsAccepted','uniquePrincipalsAcceptedDuringScheduledWindow','uniqueClientsAttempted','uniqueClientsAccepted','uniqueClientsAcceptedDuringScheduledWindow'])if(!integer(sample[key])||sample[key]>part.count)throw new Error('Invalid worker coverage');
  if(sample.concurrency!==budget.concurrency[part.index]||sample.scheduledSeconds!==stage.seconds||!integer(sample.acceptedDuringScheduledWindow)||sample.acceptedDuringScheduledWindow>raw.aggregate.accepted||sample.acceptedDuringScheduledWindow!==raw.aggregate.acceptedDuringScheduledWindow)throw new Error('Invalid worker phase counters');
  if(sample.uniquePrincipalsAccepted>sample.uniquePrincipalsAttempted||sample.uniquePrincipalsAttempted>raw.aggregate.count||sample.uniquePrincipalsAccepted>raw.aggregate.accepted||sample.uniquePrincipalsAcceptedDuringScheduledWindow>sample.uniquePrincipalsAccepted||sample.uniqueClientsAccepted>sample.uniqueClientsAttempted||sample.uniqueClientsAttempted>raw.aggregate.count||sample.uniqueClientsAccepted>raw.aggregate.accepted||sample.uniqueClientsAcceptedDuringScheduledWindow>sample.uniqueClientsAccepted||sample.uniquePrincipalsAcceptedDuringScheduledWindow>sample.acceptedDuringScheduledWindow||sample.uniqueClientsAcceptedDuringScheduledWindow>sample.acceptedDuringScheduledWindow||raw.scheduling?.count!==raw.aggregate.count||raw.arrivalLatency?.count!==raw.aggregate.count)throw new Error('Inconsistent worker coverage or latency counts');
  if(stage.arrivalRate!==undefined&&(sample.arrivalRate!==budget.rates[part.index]||!integer(sample.offered)||sample.offered+sample.abortedUnscheduled!==budget.rates[part.index]*stage.seconds||!integer(sample.droppedCapacity)||!integer(sample.droppedScheduler)||sample.offered!==raw.aggregate.count+sample.droppedCapacity+sample.droppedScheduler))throw new Error('Invalid worker arrival allocation');
 }
 const samples=partitions.map(part=>ordered.get(part.index).measurement),elapsedSeconds=(completedAtMs-startAtMs)/1000;
 const aggregate=mergeSnapshots(samples.map(value=>value.rawMeasurements.aggregate),elapsedSeconds);
 const names=Object.keys(samples[0].rawMeasurements.byRequest).sort();
 if(!names.length||names.some(name=>!ENDPOINT_LABELS.includes(name))||samples.some(value=>JSON.stringify(Object.keys(value.rawMeasurements.byRequest).sort())!==JSON.stringify(names)))throw new Error('Invalid worker endpoint labels');
 const byRequest=Object.fromEntries(names.map(name=>[name,mergeSnapshots(samples.map(value=>value.rawMeasurements.byRequest[name]),elapsedSeconds)]));
 const reasons=samples.map(value=>value.rawMeasurements.http503Reasons);
 if(reasons.some(value=>value!==undefined)&&reasons.some(value=>!value||value.schema!==1||!value.aggregate||JSON.stringify(Object.keys(value.byRequest??{}).sort())!==JSON.stringify(names)))throw Error('Invalid worker HTTP503 reason labels');
 const http503Reasons=mergeHttp503Reasons(reasons.map(value=>value?.aggregate),samples.map(value=>value.rawMeasurements.aggregate.statuses['503']??0));
 for(const name of names)byRequest[name].http503Reasons=mergeHttp503Reasons(reasons.map(value=>value?.byRequest[name]),samples.map(value=>value.rawMeasurements.byRequest[name].statuses['503']??0));
 if(http503Reasons)for(const code of Object.keys(http503Reasons.counts))if(Object.values(byRequest).reduce((sum,row)=>sum+row.http503Reasons.counts[code],0)!==http503Reasons.counts[code])throw Error('Worker endpoint HTTP503 reasons differ');
 const breakdowns=samples.map(value=>value.rawMeasurements.latencyBreakdown);
 if(breakdowns.some(value=>value!==undefined)&&breakdowns.some(value=>!value||value.schema!==1||!value.aggregate||JSON.stringify(Object.keys(value.byRequest??{}).sort())!==JSON.stringify(names)))throw Error('Invalid worker latency breakdown labels');
 const latencyBreakdown=mergeLatencyBreakdowns(breakdowns.map(value=>value?.aggregate),samples.map(value=>value.rawMeasurements.aggregate.count),samples.map(value=>value.httpDispatchAttempts));
 if(latencyBreakdown)for(let index=0;index<samples.length;index++){
  const value=breakdowns[index].aggregate;
  if(value.principalQueue.totalMs+value.httpAndValidation.totalMs>samples[index].rawMeasurements.aggregate.totalMs+1e-6)throw Error('Worker latency intervals exceed inclusive latency');
 }
 for(const name of names)byRequest[name].latencyBreakdown=mergeLatencyBreakdowns(breakdowns.map(value=>value?.byRequest[name]),samples.map(value=>value.rawMeasurements.byRequest[name].count));
 if(latencyBreakdown)for(const key of ['principalQueue','httpAndValidation','validation']){
  const expected=latencyBreakdown[key],rows=Object.values(byRequest).map(row=>row.latencyBreakdown[key]);
  if(rows.reduce((sum,row)=>sum+row.samples,0)!==expected.samples||Math.abs(rows.reduce((sum,row)=>sum+row.mean*row.samples,0)-expected.mean*expected.samples)>1e-5)throw Error('Worker endpoint latency counts differ');
 }
 for(const key of ['requests','bytes','accepted','errors','timeouts','acceptedDecodedBytes','acceptedDuringScheduledWindow','acceptedDuringScheduledWindowDecodedBytes'])if(Object.values(byRequest).reduce((sum,row)=>sum+row[key],0)!==aggregate[key])throw new Error('Worker endpoint counts differ');
 const sum=key=>samples.reduce((total,row)=>total+row[key],0);
 const resource=key=>{if(samples.some(row=>!number(row.generator?.[key])))throw new Error('Invalid worker resource counter');return samples.reduce((n,row)=>n+row.generator[key],0);};
 const cpuUserMs=resource('cpuUserMs'),cpuSystemMs=resource('cpuSystemMs');
 return {...aggregate,byRequest,latencyBreakdown,http503Reasons,mutationStateTainted:samples.some(row=>row.mutationStateTainted),unknownMutationOutcomes:sum('unknownMutationOutcomes'),taintedPrincipals:sum('taintedPrincipals'),httpDispatchAttempts:sum('httpDispatchAttempts'),blockedAfterMutationTaint:sum('blockedAfterMutationTaint'),abortedUnscheduled:sum('abortedUnscheduled'),concurrency:stage.concurrency,scheduledSeconds:stage.seconds,elapsedSeconds,acceptedDuringScheduledWindow:sum('acceptedDuringScheduledWindow'),acceptedDuringScheduledWindowRps:sum('acceptedDuringScheduledWindow')/stage.seconds,uniquePrincipalsAttempted:sum('uniquePrincipalsAttempted'),uniquePrincipalsAccepted:sum('uniquePrincipalsAccepted'),uniquePrincipalsAcceptedDuringScheduledWindow:sum('uniquePrincipalsAcceptedDuringScheduledWindow'),uniqueClientsAttempted:sum('uniqueClientsAttempted'),uniqueClientsAccepted:sum('uniqueClientsAccepted'),uniqueClientsAcceptedDuringScheduledWindow:sum('uniqueClientsAcceptedDuringScheduledWindow'),peakOutstanding:samples.reduce((n,row)=>n+row.peakOutstanding,0),peakOutstandingMethod:'sum of per-worker peaks; upper bound, not simultaneous observation',generator:{cpuUserMs,cpuSystemMs,cpuLogicalCores:(cpuUserMs+cpuSystemMs)/(elapsedSeconds*1000),rssBeforeBytes:resource('rssBeforeBytes'),rssAfterBytes:resource('rssAfterBytes'),heapAfterBytes:resource('heapAfterBytes'),rssMethod:'sum of worker before/after observations; not synchronized peak'},coordination:{schema:1,valid:true,workers:partitions.length,partitionSizes:partitions.map(part=>part.count),scheduledStartAtMs:startAtMs,scheduledEndAtMs:startAtMs+stage.seconds*1000,completedAtMs,worstStartLagMs,maxStartLagMs},...(stage.arrivalRate===undefined?{}:{arrivalRate:stage.arrivalRate,plannedArrivals:stage.arrivalRate*stage.seconds,offered:sum('offered'),attempted:aggregate.requests,droppedCapacity:sum('droppedCapacity'),droppedScheduler:sum('droppedScheduler'),schedulingLagMs:mergeSnapshots(samples.map(row=>row.rawMeasurements.scheduling),elapsedSeconds,{auxiliary:true}).latencyMs,plannedArrivalLatencyMs:mergeSnapshots(samples.map(row=>row.rawMeasurements.arrivalLatency),elapsedSeconds,{auxiliary:true}).latencyMs})};
}
