// External multi-process driver for the ignored real t32 Rust fixture.
// No token, signed URL, cookie, account ID, body, or private path is reported.
import {open,writeFile} from 'node:fs/promises';
import {fork} from 'node:child_process';
import {fileURLToPath,pathToFileURL} from 'node:url';
import {performance} from 'node:perf_hooks';
import {setTimeout as sleep} from 'node:timers/promises';
import {Measurements} from './http-load.mjs';
import {CASES,handoff,phase,counters,qualify,request,validImage,scheduledWindow,coordination} from './image-proxy-protocol.mjs';
const here=fileURLToPath(import.meta.url);
function bounded(value,min,max){const n=Number(value);if(!Number.isInteger(n)||n<min||n>max)throw Error('Invalid bounded image benchmark option');return n;}
async function control(proof,token,path,body){const r=await request(proof.control,path,{method:body===undefined?'GET':'POST',body,token,maxBytes:4*1048576,timeoutMs:120000});if(r.status!==200)throw Error('Private fixture control refused');return JSON.parse(r.bytes.toString('utf8'));}
async function worker(input){
 const {base,proof,seconds,concurrency,startAt,kind}=input;
 const measurements=new Measurements();let next=0,peakOutstanding=0,active=0,stopped=false,timeoutFailures=0;
 const capturedWall=Date.now(),capturedMono=performance.now();
 const window=scheduledWindow(startAt,seconds,capturedWall,capturedMono);
 await sleep(Math.max(0,window.startMono-performance.now()));
 const actualStart=performance.now(),actualStartUtc=Date.now(),deadline=window.deadlineMono;
 const beginCpu=process.cpuUsage(),beginWall=performance.now();
 const take=()=>{if(kind==='cold'||kind==='coalesced')return next<proof.paths.length?proof.paths[next++]:null;return proof.paths[next++%proof.paths.length];};
 await Promise.all(Array.from({length:concurrency},async()=>{
  while(!stopped&&performance.now()<deadline){
   const row=take();if(!row)break;let status='network',bytes=0,ok=false,timedOut=false;const began=performance.now();active++;peakOutstanding=Math.max(peakOutstanding,active);
   try{const result=await request(base,row.path,{principalIndex:row.principalIndex});status=String(result.status);bytes=result.bytes.length;ok=validImage(result,proof);}catch(error){timedOut=error.code==='ETIMEDOUT';if(timedOut)timeoutFailures++;}
   finally{active--;const ended=performance.now(),endedUtc=Date.now();measurements.add(ended-began,status,bytes,ok,timedOut,ok&&ended<=deadline&&endedUtc<=window.endAt);}
  }
 }));stopped=true;
 const cpu=process.cpuUsage(beginCpu),finishedMono=performance.now(),finishedWall=Date.now();return {raw:measurements.snapshot(),window:{startAt:window.startAt,endAt:window.endAt,readyBeforeStart:window.readyBeforeStart,actualStartUtc,actualFinishUtc:finishedWall,startLagMs:Math.max(0,actualStart-window.startMono),wallClockDriftMs:(finishedWall-capturedWall)-(finishedMono-capturedMono)},resources:{cpuMs:(cpu.user+cpu.system)/1000,elapsedMs:finishedMono-beginWall,rssBytes:process.memoryUsage().rss,peakOutstanding},corpusExhausted:['cold','coalesced'].includes(kind)&&next>=proof.paths.length,timeoutFailures};
}
export function runGenerator(input){return new Promise((resolve,reject)=>{
 const p=fork(here,['--worker'],{stdio:['ignore','ignore','ignore','ipc'],env:{PATH:process.env.PATH,SystemRoot:process.env.SystemRoot}});let answer,terminal=false,expired=false,observation;
 const timer=setTimeout(()=>{if(!terminal){expired=true;p.kill('SIGKILL');observation=setTimeout(()=>reject(Error('Image worker original handle still nonterminal')),5000);}},45000);
 p.once('error',()=>{clearTimeout(timer);reject(Error('Image worker failed'));});p.on('message',value=>{if(answer!==undefined){p.kill();return;}answer=value;});
 p.once('exit',code=>{terminal=true;clearTimeout(timer);clearTimeout(observation);if(!expired&&code===0&&answer?.ok)resolve(answer.result);else reject(Error('Image worker did not complete'));});p.send(input);
 });}
export async function run({handoffFile,token,output,seconds=5,workers=2,concurrency=16}){
 seconds=bounded(seconds,1,10);workers=bounded(workers,1,4);concurrency=bounded(concurrency,1,64);
 if(!/^[a-f0-9]{64}$/.test(token??''))throw Error('Private parent token required');
 const file=await open(handoffFile,'r');let bytes;
 try{const stat=await file.stat();if(!stat.isFile()||stat.size<1||stat.size>4096)throw Error('Invalid private handoff size');const buffer=Buffer.alloc(4097);const read=await file.read(buffer,0,buffer.length,0);if(read.bytesRead!==stat.size)throw Error('Private handoff changed');bytes=buffer.subarray(0,read.bytesRead);}finally{await file.close();}
 const source=handoff(JSON.parse(bytes));
 const results=[];let stopped=false;
 try{
  for(const size of [64,128])for(const kind of CASES){
   // Real UTC bucket separation, never resetting a limiter or raising a limit.
   // Valid display capabilities retain their ORIGINAL protection exemption;
   // signing/UA/source guards and every other live gate are unchanged.
   const waitMs=results.length?((Math.floor(Date.now()/60000)+1)*60000+50-Date.now()):0;
   if(waitMs)await sleep(waitMs);
   const prepared=phase(await control(source,token,'/prepare',{case:kind,size}),kind,size);
   // Queue completion belongs outside measurement. Disk outcomes also require
   // a real disk-hit delta, not just an empty queue or configured path.
   let before;const settle=Date.now()+10000;
   do{before=counters(await control(source,token,'/counters'));if(before.diskPendingBytes===0)break;await sleep(50);}while(Date.now()<settle);
   if(before.diskPendingBytes!==0)throw Error('Disk writes did not settle');
   const startAt=Date.now()+500;
   const work=Array.from({length:workers},(_,i)=>({...prepared,paths:kind==='coalesced'?prepared.paths.filter(row=>row.principalIndex%workers===i).flatMap(row=>Array(4).fill(row)):prepared.paths.filter((_,n)=>n%workers===i)}));
   const settled=await Promise.allSettled(work.map(proof=>runGenerator({base:source.base,proof,seconds,concurrency:kind==='coalesced'?proof.paths.length:concurrency,startAt,kind})));
   if(settled.some(x=>x.status!=='fulfilled'))throw Error('Image generator failed; original child outcomes retained by controller');
   const parts=settled.map(x=>x.value);
   const coordinated=coordination(parts,startAt,seconds);
   const after=counters(await control(source,token,'/counters'));
   const merged=new Measurements();for(const part of parts)merged.merge(part.raw);const measured=merged.report(seconds);
   results.push({case:kind,sourcePixels:[size,size],sourceAccounts:16,sourceSessions:16,responseBytes:prepared.expectedBytes,originDelayMs:20,actualQuotaIsolationWaitMs:waitMs,scheduledStartUtc:new Date(startAt).toISOString(),scheduledEndUtc:new Date(startAt+seconds*1000).toISOString(),...measured,
    inWindowAcceptedRps:measured.acceptedDuringScheduledWindow/seconds,
    outcome:coordinated.qualified?qualify(kind,before,after,measured):{...qualify(kind,before,after,measured),qualified:false,reason:'generator-coordination-not-qualified'},coordination:coordinated,workerWindows:parts.map(x=>x.window),cacheAfter:{ramUsedBytes:after.ramUsedBytes,ramBudgetBytes:after.ramBudgetBytes,diskUsedBytes:after.diskUsedBytes,diskBudgetBytes:after.diskBudgetBytes,diskPendingBytes:after.diskPendingBytes},
    generator:{workers,requestedConcurrencyPerWorker:concurrency,coalescedRepeatsPerAccount:kind==='coalesced'?4:null,cpuMs:parts.reduce((n,x)=>n+x.resources.cpuMs,0),rssBytesSumAtEnd:parts.reduce((n,x)=>n+x.resources.rssBytes,0),peakOutstandingSum:parts.reduce((n,x)=>n+x.resources.peakOutstanding,0),corpusLimited:kind==='cold'&&parts.some(x=>x.corpusExhausted)},
    serverCpuMs:null,serverRssBytes:null,serverSaturationProven:false});
  }
 }finally{
  const response=await request(source.control,'/stop',{method:'POST',token});if(response.status!==204)throw Error('Private fixture stop refused');stopped=true;
 }
 if(!stopped)throw Error('Fixture stop not confirmed');
 const report={schema:1,scope:'real-signed-image-route-loopback',security:'enforced-default-limits',proxy:'controlled-local-connect-standin',physicalExternalSourceDiversity:false,productionCapacityProven:false,wireBytes:null,fixtureStopAcknowledged:stopped,originalRustTestTerminalRequired:true,results};
 await writeFile(output,JSON.stringify(report,null,2),{flag:'wx',mode:0o600});return report;
}
if(process.argv[2]==='--worker')process.once('message',async input=>{try{process.send({ok:true,result:await worker(input)},()=>process.exit(0));}catch{process.send({ok:false},()=>process.exit(1));}});
else if(process.argv[1]&&import.meta.url===pathToFileURL(process.argv[1]).href){
 const args=Object.fromEntries(process.argv.slice(2).map(s=>{const i=s.indexOf('=');if(i<1)throw Error('Use named image benchmark options');return [s.slice(0,i),s.slice(i+1)];}));
 run({handoffFile:args.handoff,token:process.env.MW_IMAGE_BENCH_TOKEN,output:args.output,seconds:args.seconds??5,workers:args.workers??2,concurrency:args.concurrency??16}).then(()=>console.log('Image benchmark report written; require original Rust fixture terminal success.')).catch(()=>{console.error('Image benchmark failed; private details withheld.');process.exitCode=1;});
}
