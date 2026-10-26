// Controlled read-only HTTP load. No credentials, response bodies or account IDs
// are written to the report. Run only against an isolated fixture you own.
import {writeFile,mkdir} from 'node:fs/promises';
import {dirname,resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
import {setTimeout as sleep} from 'node:timers/promises';

export class Measurements {
  count=0; bytes=0; errors=0; timeouts=0; totalMs=0; maximumMs=0;
  histogram=new Map(); statuses={};
  add(ms,status,bytes,ok,timeout=false){
    this.count++;this.bytes+=bytes;this.totalMs+=ms;this.maximumMs=Math.max(this.maximumMs,ms);
    if(!ok)this.errors++;if(timeout)this.timeouts++;
    this.statuses[status]=(this.statuses[status]??0)+1;
    const bucket=Math.min(120001,Math.ceil(ms));this.histogram.set(bucket,(this.histogram.get(bucket)??0)+1);
  }
  percentile(fraction){let n=0;const target=Math.ceil(this.count*fraction);for(const [ms,count]of [...this.histogram].sort((a,b)=>a[0]-b[0])){n+=count;if(n>=target)return ms;}return 0;}
  report(seconds){return {requests:this.count,requestsPerSecond:this.count/seconds,bytes:this.bytes,errors:this.errors,throttled:this.statuses[429]??0,timeouts:this.timeouts,statuses:this.statuses,latencyMs:{mean:this.count?this.totalMs/this.count:0,p50:this.percentile(.5),p95:this.percentile(.95),p99:this.percentile(.99),max:this.maximumMs},percentileResolutionMs:1};}
}

export async function measure({base,requests,concurrency,seconds,timeoutMs=10000,headers={},arrivalRate}){
  const aggregate=new Measurements(),byRequest=new Map(requests.map(request=>[request.name,new Measurements()]));
  const start=performance.now(),deadline=start+seconds*1000;let sequence=0;
  const send=async()=>{
      const spec=requests[sequence++%requests.length],began=performance.now();
      let status='network',bytes=0,ok=false,timedOut=false;
      const signal=AbortSignal.timeout(timeoutMs);
      try{
        const response=await fetch(new URL(spec.path,base),{method:spec.method??'GET',headers:{...headers,...spec.headers},...(spec.body!==undefined?{body:spec.body}:{}),signal,redirect:'error'});
        status=String(response.status);const chunks=[];const reader=response.body?.getReader();
        if(reader)for(;;){const part=await reader.read();if(part.done)break;bytes+=part.value.length;if(bytes>8*1024*1024){await reader.cancel();throw new Error('response too large');}if(spec.validate)chunks.push(part.value);}
        ok=(spec.statuses??[200]).includes(response.status);
        if(ok&&spec.validate&&response.status!==429)ok=spec.validate(JSON.parse(Buffer.concat(chunks).toString('utf8')));
      }catch{timedOut=signal.aborted;ok=false;}
      const elapsed=performance.now()-began;
      aggregate.add(elapsed,status,bytes,ok,timedOut);byRequest.get(spec.name).add(elapsed,status,bytes,ok,timedOut);
  };
  let offered=0,droppedCapacity=0,droppedScheduler=0,peakOutstanding=0;
  if(arrivalRate!==undefined){
    if(!Number.isFinite(arrivalRate)||arrivalRate<=0||arrivalRate>10000)throw new Error('Invalid arrival rate');
    const pending=new Set(),interval=1000/arrivalRate,total=Math.floor(seconds*arrivalRate);
    // Fixed arrivals are independent of response speed. Never replay an overdue
    // backlog as a burst: report generator misses separately from server refusal.
    for(let slot=0;slot<total;){
      const due=start+slot*interval,now=performance.now();
      if(now<due){await sleep(Math.max(1,due-now));continue;}
      const latest=Math.min(total-1,Math.floor((now-start)/interval));
      if(latest>slot){droppedScheduler+=latest-slot;offered+=latest-slot;slot=latest;}
      offered++;slot++;
      if(now>=deadline){droppedScheduler++;continue;}
      if(pending.size>=concurrency){droppedCapacity++;continue;}
      const operation=send();pending.add(operation);peakOutstanding=Math.max(peakOutstanding,pending.size);
      void operation.finally(()=>pending.delete(operation));
    }
    await Promise.all(pending);
    if(performance.now()<deadline)await sleep(deadline-performance.now());
  }else{
    await Promise.all(Array.from({length:concurrency},async()=>{while(performance.now()<deadline)await send();}));
  }
  const elapsed=(performance.now()-start)/1000;
  return {concurrency,scheduledSeconds:seconds,elapsedSeconds:elapsed,...(arrivalRate===undefined?{}:{arrivalRate,offered,droppedCapacity,droppedScheduler,peakOutstanding}),...aggregate.report(elapsed),byRequest:Object.fromEntries([...byRequest].map(([key,value])=>[key,value.report(elapsed)]))};
}

async function main(){
 const args=Object.fromEntries(process.argv.slice(2).map(arg=>{const split=arg.indexOf('=');if(split<0)throw new Error('Use --name=value options');return[arg.slice(2,split),arg.slice(split+1)];}));
 const allowed=new Set(['base','concurrency','seconds','warmup','timeout','scenario','output','allow-remote','max-p95','max-error-rate','allow-throttles','max-throttle-rate','arrival-rate','max-drop-rate']);for(const key of Object.keys(args))if(!allowed.has(key))throw new Error(`Unknown option: ${key}`);
 const base=new URL(args.base??'http://127.0.0.1:42214');
 if(!['http:','https:'].includes(base.protocol)||base.username||base.password||base.pathname!=='/'||base.search||base.hash)throw new Error('Base must be a plain HTTP(S) origin');
 if(!['127.0.0.1','localhost','[::1]'].includes(base.hostname)&&args['allow-remote']!=='true')throw new Error('Remote load requires explicit --allow-remote=true');
 const levels=(args.concurrency??'1,8,32,64').split(',').map(Number),seconds=Number(args.seconds??30),warmup=Number(args.warmup??3),timeoutMs=Number(args.timeout??10000);
 if(!levels.length||levels.length>10||levels.some(n=>!Number.isInteger(n)||n<1||n>256)||!Number.isFinite(seconds)||seconds<1||seconds>600||!Number.isFinite(warmup)||warmup<0||warmup>60||!Number.isFinite(timeoutMs)||timeoutMs<100||timeoutMs>120000)throw new Error('Invalid bounded load parameters');
 const arrivalRate=args['arrival-rate']===undefined?undefined:Number(args['arrival-rate']),dropRate=Number(args['max-drop-rate']??0);
 if((arrivalRate!==undefined&&(!Number.isFinite(arrivalRate)||arrivalRate<1||arrivalRate>10000))||!Number.isFinite(dropRate)||dropRate<0||dropRate>1)throw new Error('Invalid arrival or drop budget');
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
 const report={generatedAt:new Date().toISOString(),target:base.origin,scenario,model:arrivalRate===undefined?'closed-loop concurrent clients; not an open-loop capacity claim':'fixed arrival rate with bounded outstanding requests; dropped arrivals reported separately',runtime:process.version,stages:[]};
 try{
   if(warmup)await measure({base,requests,headers,concurrency:1,seconds:warmup,timeoutMs});
   for(const concurrency of levels){const result=await measure({base,requests,headers,concurrency,seconds,timeoutMs,arrivalRate});report.stages.push(result);console.log(`${scenario}: clients=${concurrency}, requests=${result.requests}, rps=${result.requestsPerSecond.toFixed(1)}, p95=${result.latencyMs.p95}ms, p99=${result.latencyMs.p99}ms, errors=${result.errors}, throttled=${result.throttled}, timeouts=${result.timeouts}, dropped=${(result.droppedCapacity??0)+(result.droppedScheduler??0)}`);}
   const health=await fetch(new URL('/healthz',base),{redirect:'error',signal:AbortSignal.timeout(timeoutMs)});report.healthyAfter=health.status===200;await health.body?.cancel();
 }finally{if(ownLogin){try{const response=await fetch(new URL('/api/logout',base),{method:'POST',headers,redirect:'error',signal:AbortSignal.timeout(timeoutMs)});await response.body?.cancel();}catch{/* Report measurements even if fixture-session cleanup fails. */}}}
 const output=resolve(args.output??'.orchestration/reports/http-load.json');await mkdir(dirname(output),{recursive:true});await writeFile(output,JSON.stringify(report,null,2)+'\n');

 if(!report.healthyAfter||report.stages.some(stage=>stage.latencyMs.p95>p95||stage.errors/Math.max(1,stage.requests)>errorRate||stage.throttled/Math.max(1,stage.requests)>throttleRate||((stage.droppedCapacity??0)+(stage.droppedScheduler??0))/Math.max(1,stage.offered??0)>dropRate))process.exitCode=1;
}
if(process.argv[1]&&import.meta.url===pathToFileURL(resolve(process.argv[1])).href)main().catch(error=>{console.error(error.message);process.exitCode=1;});
