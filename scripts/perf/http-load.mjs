// Controlled read-only HTTP load. No credentials, response bodies or account IDs
// are written to the report. Run only against an isolated fixture you own.
import {writeFile,mkdir} from 'node:fs/promises';
import {dirname,resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
import {setTimeout as sleep} from 'node:timers/promises';

export class Measurements {
  count=0; bytes=0; errors=0; accepted=0; timeouts=0; totalMs=0; maximumMs=0;
  histogram=new Map(); statuses={};
  add(ms,status,bytes,ok,timeout=false){
    this.count++;this.bytes+=bytes;this.totalMs+=ms;this.maximumMs=Math.max(this.maximumMs,ms);
    if(!ok)this.errors++;if(ok&&Number(status)>=200&&Number(status)<300)this.accepted++;if(timeout)this.timeouts++;
    this.statuses[status]=(this.statuses[status]??0)+1;
    const bucket=ms<=1000?Math.ceil(ms):ms<=10000?Math.ceil(ms/10)*10:Math.min(120100,Math.ceil(ms/100)*100);this.histogram.set(bucket,(this.histogram.get(bucket)??0)+1);
  }
  percentile(fraction){let n=0;const target=Math.ceil(this.count*fraction);for(const [ms,count]of [...this.histogram].sort((a,b)=>a[0]-b[0])){n+=count;if(n>=target)return ms;}return 0;}
  snapshot(){return {count:this.count,bytes:this.bytes,errors:this.errors,accepted:this.accepted,timeouts:this.timeouts,totalMs:this.totalMs,maximumMs:this.maximumMs,statuses:{...this.statuses},histogram:[...this.histogram].sort((a,b)=>a[0]-b[0])};}
  merge(value){
    const invalid=()=>{throw new Error('Invalid bounded raw measurements');};
    if(!value||['count','bytes','errors','accepted','timeouts'].some(key=>!Number.isSafeInteger(value[key])||value[key]<0)||!Number.isFinite(value.totalMs)||value.totalMs<0||!Number.isFinite(value.maximumMs)||value.maximumMs<0||value.maximumMs>value.totalMs||value.errors>value.count||value.accepted>value.count-value.errors||value.timeouts>value.errors||!Array.isArray(value.histogram)||value.histogram.length>3002||!value.statuses||Array.isArray(value.statuses)||typeof value.statuses!=='object')invalid();
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
    if(samples!==value.count||responses!==value.count||value.accepted>successResponses||value.count===0&&(value.totalMs!==0||value.maximumMs!==0||value.bytes!==0)||['count','bytes','errors','accepted','timeouts'].some(key=>!Number.isSafeInteger(this[key]+value[key]))||!Number.isFinite(this.totalMs+value.totalMs))invalid();
    for(const key of ['count','bytes','errors','accepted','timeouts','totalMs'])this[key]+=value[key];
    this.maximumMs=Math.max(this.maximumMs,value.maximumMs);
    for(const [bucket,count]of bins)this.histogram.set(bucket,(this.histogram.get(bucket)??0)+count);
    for(const [status,count]of statuses)this.statuses[status]=(this.statuses[status]??0)+count;
    return this;
  }
  report(seconds){return {requests:this.count,requestsPerSecond:this.count/seconds,bytes:this.bytes,accepted:this.accepted,acceptedRequestsPerSecond:this.accepted/seconds,errors:this.errors,throttled:this.statuses[429]??0,timeouts:this.timeouts,statuses:this.statuses,latencyMs:{mean:this.count?this.totalMs/this.count:0,p50:this.percentile(.5),p95:this.percentile(.95),p99:this.percentile(.99),max:this.maximumMs},percentileResolutionMs:'1 through 1s; 10 through 10s; 100 thereafter; overflow 120100'};}
}

export function mergeMeasurements(snapshots,seconds){
  if(!Array.isArray(snapshots)||!snapshots.length||snapshots.length>16||!Number.isFinite(seconds)||seconds<=0||seconds>600)throw new Error('Invalid bounded measurement merge');
  const merged=new Measurements();for(const snapshot of snapshots)merged.merge(snapshot);return merged.report(seconds);
}

export async function measure({base,requests,concurrency,seconds,timeoutMs=10000,headers={},arrivalRate,commonStartAtMs,includeRawMeasurements=false}){
  const aggregate=new Measurements(),byRequest=new Map(requests.map(request=>[request.name,new Measurements()]));
  if(!Number.isInteger(concurrency)||concurrency<1||concurrency>4096||!Number.isFinite(seconds)||seconds<=0||seconds>600||!requests.length||requests.length>100||!Number.isFinite(timeoutMs)||timeoutMs<1||timeoutMs>120000)throw new Error('Invalid bounded measurement parameters');
  if(typeof includeRawMeasurements!=='boolean')throw new Error('Invalid raw measurement option');
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
  const taintedPrincipals=new Set();let mutationStateTainted=false,unknownMutationOutcomes=0,httpDispatchAttempts=0,blockedAfterMutationTaint=0,abortedUnscheduled=0;
  const send=async(planned=performance.now())=>{
      const index=sequence++,template=requests[index%requests.length],spec=template.materialize?{...template,...template.materialize(index,{requestCount:requests.length}),name:template.name}:template,began=performance.now();
      increment(principals.attempted,spec.principalIndex);increment(clients.attempted,spec.clientIndex);
      peakOutstanding=Math.max(peakOutstanding,++activeCount);
      scheduling.add(Math.max(0,began-planned),'scheduled',0,true);
      let status='network',bytes=0,ok=false,timedOut=false;
      const signal=AbortSignal.timeout(timeoutMs);
      let unlock=null,dispatched=false;
      try{
        if(spec.serializePrincipal){
          if(!boundedIndex(spec.principalIndex))throw new Error('Serialized fixture requests require a bounded principal');
          const previous=principalQueues.get(spec.principalIndex)??Promise.resolve();
          let release;const current=new Promise(resolve=>{release=resolve;});
          principalQueues.set(spec.principalIndex,current);
          unlock=()=>{release();if(principalQueues.get(spec.principalIndex)===current)principalQueues.delete(spec.principalIndex);};
          await previous;
          if(mutationStateTainted){status='campaign-aborted';blockedAfterMutationTaint++;throw new Error('Mutation outcome is unknown; fixture state cannot be reused');}
          signal.throwIfAborted();
        }
        if(mutationStateTainted){status='campaign-aborted';blockedAfterMutationTaint++;throw new Error('Mutation outcome is unknown; fixture state cannot be reused');}
        dispatched=true;httpDispatchAttempts++;
        const response=await fetch(new URL(spec.path,base),{method:spec.method??'GET',headers:{...headers,...spec.headers},...(spec.body!==undefined?{body:spec.body}:{}),signal,redirect:'error'});
        status=String(response.status);const chunks=[];const reader=response.body?.getReader();
        if(reader)for(;;){const part=await reader.read();if(part.done)break;bytes+=part.value.length;if(bytes>8*1024*1024){await reader.cancel();throw new Error('response too large');}if(spec.validate||spec.validateBytes){if(bytes>(spec.validate?65536:1024*1024)){await reader.cancel();throw new Error('validation response too large');}chunks.push(part.value);}}
        ok=(spec.statuses??[200]).includes(response.status);
        if(ok&&spec.validate&&response.status!==429)ok=spec.validate(JSON.parse(Buffer.concat(chunks).toString('utf8')));
        if(ok&&spec.validateBytes&&response.status!==429)ok=spec.validateBytes(Buffer.concat(chunks));
      }catch{timedOut=status!=='campaign-aborted'&&signal.aborted;ok=false;}finally{
        // A client abort does not cancel a server mutation. Fence the queue
        // before releasing it and never reuse an uncertain fixture state.
        if(spec.statefulMutation&&dispatched&&!ok){mutationStateTainted=true;unknownMutationOutcomes++;if(boundedIndex(spec.principalIndex))taintedPrincipals.add(spec.principalIndex);}
        unlock?.();
      }
      const completed=performance.now(),elapsed=completed-began;
      if(ok&&Number(status)>=200&&Number(status)<300){
        increment(principals.accepted,spec.principalIndex);increment(clients.accepted,spec.clientIndex);
        if(completed<=deadline){acceptedWithinWindow++;if(boundedIndex(spec.principalIndex))principals.withinWindow.add(spec.principalIndex);if(boundedIndex(spec.clientIndex))clients.withinWindow.add(spec.clientIndex);}
      }
      aggregate.add(elapsed,status,bytes,ok,timedOut);arrivalLatency.add(performance.now()-planned,status,bytes,ok,timedOut);byRequest.get(spec.name).add(elapsed,status,bytes,ok,timedOut);activeCount--;
  };
  let offered=0,droppedCapacity=0,droppedScheduler=0,peakOutstanding=0;
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
      const oldest=Math.min(total,Math.max(slot,Math.ceil((now-start-100)/interval)));
      if(oldest>slot){droppedScheduler+=oldest-slot;offered+=oldest-slot;slot=oldest;}
      const end=Math.min(total,Math.floor((now-start)/interval)+1);
      const dispatch=Math.min(end-slot,concurrency-pending.size);
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
    ...(includeRawMeasurements?{rawMeasurements:{schema:1,aggregate:aggregate.snapshot(),byRequest:Object.fromEntries([...byRequest].map(([name,measurements])=>[name,measurements.snapshot()])),scheduling:scheduling.snapshot(),arrivalLatency:arrivalLatency.snapshot()}}:{}),
  };
  return {...hooks,mutationStateTainted,unknownMutationOutcomes,taintedPrincipals:taintedPrincipals.size,httpDispatchAttempts,blockedAfterMutationTaint,abortedUnscheduled,uniquePrincipalsAccepted:principals.accepted.size,uniquePrincipalsAttempted:principals.attempted.size,uniquePrincipalsAcceptedDuringScheduledWindow:principals.withinWindow.size,uniqueClientsAttempted:clients.attempted.size,uniqueClientsAccepted:clients.accepted.size,uniqueClientsAcceptedDuringScheduledWindow:clients.withinWindow.size,participantDistribution:{principals:{attempted:distribution(principals.attempted),accepted:distribution(principals.accepted)},simulatedClients:{attempted:distribution(clients.attempted),accepted:distribution(clients.accepted)}},peakOutstanding,acceptedDuringScheduledWindow:acceptedWithinWindow,acceptedDuringScheduledWindowRps:acceptedWithinWindow/seconds,generator:{cpuUserMs:cpu.user/1000,cpuSystemMs:cpu.system/1000,cpuLogicalCores:(cpu.user+cpu.system)/(cpuElapsed*1000000),rssBeforeBytes:memoryStart.rss,rssAfterBytes:memoryEnd.rss,heapAfterBytes:memoryEnd.heapUsed},concurrency,scheduledSeconds:seconds,elapsedSeconds:elapsed,...(arrivalRate===undefined?{}:{arrivalRate,offered,attempted:aggregate.count,attemptedRequestsPerSecond:aggregate.count/seconds,achievedRequestsPerSecond:aggregate.count/elapsed,droppedCapacity,droppedScheduler,peakOutstanding,schedulingLagMs:scheduling.report(elapsed).latencyMs,plannedArrivalLatencyMs:arrivalLatency.report(elapsed).latencyMs}),...aggregate.report(elapsed),byRequest:Object.fromEntries([...byRequest].map(([key,value])=>[key,value.report(elapsed)]))};
}

export function markdownReport(report){
 const perUserRange=stage=>{const accepted=stage.participantDistribution?.principals?.accepted;return accepted&&Number.isFinite(accepted.min)&&Number.isFinite(accepted.max)?`${accepted.min} / ${accepted.max}`:'not measured';};
 const readiness=report.startupReadiness;
 const startup=readiness?.verified===true?`Startup index readiness verified: ${readiness.indexDocuments}/${readiness.expectedIndexDocuments} documents; ${Number(readiness.startupSeconds).toFixed(1)} s startup, ${Number(readiness.indexWaitSeconds).toFixed(1)} s index wait; ${readiness.zeroAccessExcludedMessages} policy-excluded messages.`:'Startup index readiness was not measured in this report.';
 return `# HTTP load report\n\nTarget: ${report.target}\n\nScenario: ${report.scenario}; runtime: ${report.runtime}. ${report.model}.${report.protection?' Protection: '+report.protection+'.':''}\n\n${startup}\n\n| Phase / endpoint / outstanding requests | Fixture protection | Offered RPS | Attempted | Accepted | Completed RPS | Accepted RPS | p50/p95/p99 ms | Errors | 429 | Timeouts | Capacity/scheduler drops | Lag p99 ms |\n| --- | --- | ---: | ---: | ---: | ---: | ---: | --- | ---: | ---: | ---: | --- | ---: |\n`+report.stages.map(s=>`| ${s.phase??'smoke'} / ${s.endpoint??'mixed'} / ${s.concurrency} | ${s.fixtureProtection===undefined?'see scenario':s.fixtureProtection?'on (simulated)':'off (simulated)'} | ${s.arrivalRate??'closed-loop'} | ${s.requests} | ${s.accepted} | ${s.requestsPerSecond.toFixed(1)} | ${s.acceptedRequestsPerSecond.toFixed(1)} | ${s.latencyMs.p50}/${s.latencyMs.p95}/${s.latencyMs.p99} | ${s.errors} | ${s.throttled} | ${s.timeouts} | ${s.droppedCapacity??0}/${s.droppedScheduler??0} | ${s.schedulingLagMs?.p99??0} |`).join('\n')+'\n\n| Phase / endpoint | Distinct users attempted / accepted | Simulated clients attempted / accepted | Accepted requests per user min / max |\n| --- | ---: | ---: | ---: |\n'+report.stages.map(s=>`| ${s.phase??'smoke'} / ${s.endpoint??'mixed'} | ${s.uniquePrincipalsAttempted??0} / ${s.uniquePrincipalsAccepted??0} | ${s.uniqueClientsAttempted??0} / ${s.uniqueClientsAccepted??0} | ${perUserRange(s)} |`).join('\n')+'\n\nSimulated client addresses are a fixture proxy model, not separate physical generator hosts. Zero tracked identities is not evidence of multi-user load. Dropped arrivals have no response latency and must be evaluated alongside percentiles. This report alone is not a production capacity claim.\n';
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
   for(const concurrency of levels)for(const stage of stages){const result=await measure({base,requests,headers,concurrency,seconds:stage.duration,timeoutMs,arrivalRate:stage.rate});report.stages.push(result);console.log(`${scenario}: outstanding=${concurrency}, requests=${result.requests}, rps=${result.requestsPerSecond.toFixed(1)}, p95=${result.latencyMs.p95}ms, p99=${result.latencyMs.p99}ms, errors=${result.errors}, throttled=${result.throttled}, timeouts=${result.timeouts}, dropped=${(result.droppedCapacity??0)+(result.droppedScheduler??0)}`);}
   const health=await fetch(new URL('/healthz',base),{redirect:'error',signal:AbortSignal.timeout(timeoutMs)});report.healthyAfter=health.status===200;await health.body?.cancel();
 }finally{if(ownLogin){try{const response=await fetch(new URL('/api/logout',base),{method:'POST',headers,redirect:'error',signal:AbortSignal.timeout(timeoutMs)});await response.body?.cancel();}catch{/* Report measurements even if fixture-session cleanup fails. */}}}
 const output=resolve(args.output??'.orchestration/reports/http-load.json');await mkdir(dirname(output),{recursive:true});await writeFile(output,JSON.stringify(report,null,2)+'\n');await writeFile(output.replace(/\.json$/,'')+'.md',markdownReport(report));

 if(!report.healthyAfter||report.stages.some(stage=>stage.latencyMs.p95>p95||stage.errors/Math.max(1,stage.requests)>errorRate||stage.throttled/Math.max(1,stage.requests)>throttleRate||((stage.droppedCapacity??0)+(stage.droppedScheduler??0))/Math.max(1,stage.offered??0)>dropRate))process.exitCode=1;
}
if(process.argv[1]&&import.meta.url===pathToFileURL(resolve(process.argv[1])).href)main().catch(error=>{console.error(error.message);process.exitCode=1;});
