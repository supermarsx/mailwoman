// Synthetic transport only: this protocol cannot establish application capacity.
import {mergeMeasurements,MAX_OUTSTANDING,MAX_OFFERED_RPS} from './http-load.mjs';
import {splitBudget} from './http-multi-protocol.mjs';
import {calibrationRequests,JSON_BYTES,BINARY_BYTES} from './http-calibration-protocol.mjs';
export function loopbackOrigin(value){
 const url=new URL(value??'invalid:');
 if(url.protocol!=='http:'||url.hostname!=='127.0.0.1'||!url.port||url.pathname!=='/'||url.search||url.hash||url.username||url.password)throw new Error('Literal private loopback origin required');
 return url.origin;
}
export function clusterAuthority(value,now=Date.now()){
 if(value?.schema!==1||value.kind!=='synthetic-cluster-transport-calibration'||!Number.isInteger(value.serverWorkers)||value.serverWorkers<1||value.serverWorkers>16||!Array.isArray(value.dataBases)||value.dataBases.length!==value.serverWorkers||typeof value.token!=='string'||!/^[a-f0-9]{64}$/.test(value.token)||!Number.isSafeInteger(value.expiresAtMs)||value.expiresAtMs<=now||value.expiresAtMs>now+901000)throw new Error('Invalid private cluster calibration authority');
 const dataBases=value.dataBases.map(loopbackOrigin),controlBase=loopbackOrigin(value.controlBase);
 if(new Set([...dataBases,controlBase]).size!==dataBases.length+1)throw new Error('Cluster listener origins must be distinct');
 return {dataBases,controlBase,token:value.token,serverWorkers:value.serverWorkers,expiresAtMs:value.expiresAtMs};
}
export function clusterOptions(args){
 if(Object.keys(args).some(key=>!['handoff','output','seconds','rates','concurrency','workers'].includes(key))||!args.handoff||!args.output)throw new Error('Explicit private handoff and report required');
 const workers=Number(args.workers??4),seconds=Number(args.seconds??3),concurrency=Number(args.concurrency??256),rates=String(args.rates??'10000,20000,40000,60000,80000').split(',').map(Number);
 if(!Number.isInteger(workers)||workers<1||workers>64||!Number.isInteger(seconds)||seconds<1||seconds>3600||!Number.isInteger(concurrency)||concurrency<workers||concurrency>MAX_OUTSTANDING||!rates.length||rates.length>5||new Set(rates).size!==rates.length||rates.some(rate=>!Number.isInteger(rate)||rate<100||rate<workers||rate>MAX_OFFERED_RPS))throw new Error('Cluster calibration limits exceeded');
 return {workers,seconds,concurrency,rates};
}
export function calibrationStageBudget(options,rate){return {concurrency:splitBudget(options.concurrency,options.workers),rates:splitBudget(rate,options.workers)};}
export function clusterRequest(endpoint,dataBases,index){
 const spec=calibrationRequests().find(spec=>spec.name===endpoint);
 if(!spec||!Number.isInteger(index)||index<0||index>63||!Array.isArray(dataBases)||!dataBases.length||dataBases.length>16)throw new Error('Invalid synthetic request assignment');
 const origins=dataBases.map(loopbackOrigin);
 return {...spec,materialize:sequence=>({path:new URL(spec.path,origins[(sequence+index)%origins.length]).href})};
}
export function aggregateCalibrationWorkers(results,{workers,seconds,concurrency,rate,endpoint,startAtMs,maxStartLagMs=50,observedAtMs=Date.now()}){
 const budget=calibrationStageBudget({workers,concurrency},rate),ordered=new Map();
 if(!Array.isArray(results)||results.length!==workers||!Number.isInteger(seconds)||seconds<1||seconds>3600||!Number.isSafeInteger(observedAtMs))throw new Error('Incomplete calibration worker results');
 for(const result of results){if(!Number.isInteger(result?.index)||ordered.has(result.index))throw new Error('Duplicate calibration worker');ordered.set(result.index,result);}
 const samples=Array.from({length:workers},(_,index)=>{
  const result=ordered.get(index),sample=result?.measurement,window=sample?.window,raw=sample?.rawMeasurements;
  if(result?.schema!==1||!window||window.requestedStartAtMs!==startAtMs||window.scheduledStartAtMs!==startAtMs||window.scheduledEndAtMs!==startAtMs+seconds*1000||!Number.isFinite(window.startLagMs)||window.startLagMs<0||window.startLagMs>maxStartLagMs||!Number.isSafeInteger(window.actualStartAtMs)||Math.abs(window.actualStartAtMs-startAtMs-window.startLagMs)>2||!Number.isSafeInteger(window.completedAtMs)||window.completedAtMs<window.scheduledEndAtMs||!raw||raw.schema!==1||JSON.stringify(Object.keys(raw.byRequest))!==JSON.stringify([endpoint])||sample.mutationStateTainted!==false||sample.concurrency!==budget.concurrency[index]||sample.arrivalRate!==budget.rates[index]||sample.offered!==budget.rates[index]*seconds||sample.scheduledSeconds!==seconds)throw new Error('Invalid calibration window or budget');
  for(const key of ['droppedCapacity','droppedScheduler','acceptedDuringScheduledWindow','peakOutstanding'])if(!Number.isSafeInteger(sample[key])||sample[key]<0)throw new Error('Invalid calibration counter');
  if(window.completedAtMs>observedAtMs+maxStartLagMs||sample.offered!==raw.aggregate.count+sample.droppedCapacity+sample.droppedScheduler||sample.acceptedDuringScheduledWindow>raw.aggregate.accepted||sample.peakOutstanding>budget.concurrency[index]||raw.scheduling.count!==raw.aggregate.count||raw.arrivalLatency.count!==raw.aggregate.count)throw new Error('Inconsistent calibration counters');
  return sample;
 });
 const completedAtMs=Math.max(...samples.map(sample=>sample.window.completedAtMs)),elapsed=(completedAtMs-startAtMs)/1000;
 const merged=mergeMeasurements(samples.map(sample=>sample.rawMeasurements.aggregate),elapsed),byRequest=mergeMeasurements(samples.map(sample=>sample.rawMeasurements.byRequest[endpoint]),elapsed);
 for(const key of ['requests','accepted','bytes','errors','timeouts'])if(merged[key]!==byRequest[key])throw new Error('Calibration endpoint totals differ');
 const sum=key=>samples.reduce((total,sample)=>total+sample[key],0),resource=key=>samples.reduce((total,sample)=>{const value=sample.generator?.[key];if(!Number.isFinite(value)||value<0)throw new Error('Invalid generator resource counter');return total+value;},0);
 const acceptedDuringScheduledWindow=sum('acceptedDuringScheduledWindow'),droppedCapacity=sum('droppedCapacity'),droppedScheduler=sum('droppedScheduler');
 return {...merged,endpoint,workers,concurrency,scheduledSeconds:seconds,elapsedSeconds:elapsed,arrivalRate:rate,offered:rate*seconds,acceptedDuringScheduledWindow,acceptedDuringScheduledWindowRps:acceptedDuringScheduledWindow/seconds,droppedCapacity,droppedScheduler,peakOutstandingUpperBound:sum('peakOutstanding'),schedulingLagMs:mergeMeasurements(samples.map(sample=>sample.rawMeasurements.scheduling),elapsed).latencyMs,plannedArrivalLatencyMs:mergeMeasurements(samples.map(sample=>sample.rawMeasurements.arrivalLatency),elapsed).latencyMs,coordination:{valid:true,startAtMs,endAtMs:startAtMs+seconds*1000,completedAtMs,worstStartLagMs:Math.max(...samples.map(sample=>sample.window.startLagMs))},generator:{cpuUserMs:resource('cpuUserMs'),cpuSystemMs:resource('cpuSystemMs'),rssBeforeBytes:resource('rssBeforeBytes'),rssAfterBytes:resource('rssAfterBytes'),heapAfterBytes:resource('heapAfterBytes'),resourceMethod:'sum of process observations, not synchronized peak'},transportWindowSustained:merged.errors===0&&merged.timeouts===0&&merged.throttled===0&&droppedCapacity===0&&droppedScheduler===0&&acceptedDuringScheduledWindow===rate*seconds};
}
export function clusterCalibrationReport(options,stages){return {schema:1,kind:'synthetic-cluster-transport-calibration',applicationCapacityMeasured:false,securityProof:false,mailwomanCapacityClaim:false,distinctPhysicalGeneratorHostsMeasured:false,interpretation:'Constant-response generator/transport calibration only; no Mailwoman authentication, policies, mailbox operations or capacity proof.',options,payloadBytes:{json:JSON_BYTES.length,binary:BINARY_BYTES.length},stages};}

// Small, private IPC channel. Failures never copy child messages or secrets.
export function calibrationChannel(child){
 const inbox=[],waiters=[];let failed=false,exited=false;
 const fail=()=>{failed=true;for(const waiter of waiters.splice(0))waiter.reject(new Error('Calibration child failed'));};
 child.on('message',message=>{
  if(message?.type==='failed'){fail();return;}
  const at=waiters.findIndex(waiter=>waiter.type===message?.type&&waiter.id===message.id);
  if(at>=0)waiters.splice(at,1)[0].resolve(message);else if(inbox.length<8)inbox.push(message);else fail();
 });
 child.once('error',fail);child.once('exit',()=>{exited=true;fail();});
 return {send(message){if(failed||!child.connected)throw new Error('Calibration child unavailable');child.send(message);},wait(type,id,ms=5000){
  if(failed)return Promise.reject(new Error('Calibration child unavailable'));
  const at=inbox.findIndex(message=>message?.type===type&&message.id===id);if(at>=0)return Promise.resolve(inbox.splice(at,1)[0]);
  return new Promise((resolve,reject)=>{let timer;const waiter={type,id,resolve:value=>{clearTimeout(timer);resolve(value);},reject:error=>{clearTimeout(timer);reject(error);}};waiters.push(waiter);timer=setTimeout(()=>{const at=waiters.indexOf(waiter);if(at>=0)waiters.splice(at,1);reject(new Error('Calibration child deadline exceeded'));},ms);});
 },async stop(){if(exited)return;child.kill('SIGKILL');await new Promise((resolve,reject)=>{const timer=setTimeout(()=>reject(new Error('Calibration child exit unconfirmed')),5000);child.once('exit',()=>{clearTimeout(timer);resolve();});if(exited){clearTimeout(timer);resolve();}});}};
}
