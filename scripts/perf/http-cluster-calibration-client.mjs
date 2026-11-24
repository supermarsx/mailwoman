// Coordinated constant-response calibration, never a Mailwoman capacity test.
import {fork} from 'node:child_process';
import {writeFile,mkdir} from 'node:fs/promises';
import {dirname,resolve} from 'node:path';
import {fileURLToPath,pathToFileURL} from 'node:url';
import {measure,MAX_OUTSTANDING,MAX_OFFERED_RPS} from './http-load.mjs';
import {readPrivate} from './external-control.mjs';
import {calibrationRequests} from './http-calibration-protocol.mjs';
import {clusterAuthority,clusterOptions,clusterRequest,calibrationStageBudget,aggregateCalibrationWorkers,clusterCalibrationReport,calibrationChannel} from './http-cluster-calibration-protocol.mjs';

export function serverMetricsReport(value,workers){
 const fields=['requests','bytes','elapsedSeconds','cpuUserMs','cpuSystemMs','rssBytes'];
 const project=(row,keys)=>{if(!row||keys.some(key=>!Number.isFinite(row[key])||row[key]<0))throw new Error('Invalid synthetic server resource observations');return Object.fromEntries(keys.map(key=>[key,row[key]]));};
 if(value?.schema!==1||value.workers!==workers||!Array.isArray(value.perWorker)||value.perWorker.length!==workers)throw new Error('Missing synthetic server observations');
 const perWorker=value.perWorker.map((row,index)=>{if(row.index!==index||!Number.isSafeInteger(row.requests)||!Number.isSafeInteger(row.bytes))throw new Error('Invalid synthetic server counters');return {index,...project(row,fields),...(row.eventLoopDelayMs===undefined?{}:{eventLoopDelayMs:project(row.eventLoopDelayMs,['mean','p99','max'])})};});
 const sum=key=>perWorker.reduce((total,row)=>total+row[key],0);
 for(const key of ['requests','bytes','cpuUserMs','cpuSystemMs','rssBytes'])if(value[key]!==sum(key))throw new Error('Synthetic server totals differ');
 return {workers,perWorker,requests:sum('requests'),bytes:sum('bytes'),cpuUserMs:sum('cpuUserMs'),cpuSystemMs:sum('cpuSystemMs'),rssBytes:sum('rssBytes'),primary:project(value.primary,['cpuUserMs','cpuSystemMs','elapsedSeconds','rssBytes']),observationMeaning:'reset through post-drain snapshot, including common-start lead; response counts are handler attempts, not confirmed delivery; RSS is a sum, not synchronized peak'};
}

async function worker(){
 process.on('disconnect',()=>process.exit(1));
 process.once('message',config=>{
  try{
   if(config?.type!=='init'||!Number.isInteger(config.index)||config.index<0||config.index>15)throw new Error();
   const authority=clusterAuthority(config.authority);let phase=null,busy=false;
   setTimeout(()=>process.exit(1),Math.max(1,authority.expiresAtMs-Date.now()+1000)).unref();
   process.on('message',async message=>{
    try{
     if(message?.type==='prepare'){
      if(phase||busy||!Number.isSafeInteger(message.id)||message.id<0||!Number.isInteger(message.seconds)||message.seconds<1||message.seconds>3600||!Number.isInteger(message.concurrency)||message.concurrency<1||message.concurrency>MAX_OUTSTANDING||!Number.isInteger(message.rate)||message.rate<1||message.rate>MAX_OFFERED_RPS)throw new Error();
      phase={...message,spec:clusterRequest(message.endpoint,authority.dataBases,config.index)};process.send({type:'stage-ready',id:message.id,index:config.index});return;
     }
     if(message?.type!=='start'||busy||!phase||message.id!==phase.id||!Number.isSafeInteger(message.startAtMs)||message.startAtMs<Date.now()-1000||message.startAtMs>Date.now()+10000)throw new Error();
     busy=true;const current=phase;
     const measurement=await measure({base:authority.dataBases[0],headers:{'x-calibration-token':authority.token},requests:[current.spec],concurrency:current.concurrency,seconds:current.seconds,arrivalRate:current.rate,timeoutMs:5000,commonStartAtMs:message.startAtMs,includeRawMeasurements:true});
     phase=null;busy=false;process.send({type:'result',schema:1,id:current.id,index:config.index,measurement});
    }catch{process.send({type:'failed'});}
   });
   process.send({type:'ready',index:config.index});
  }catch{process.send?.({type:'failed'});process.exitCode=1;process.disconnect();}
 });
}

export async function runClusterCalibration({handoff,options,launch=fork,startLeadMs=2000}){
 const authority=clusterAuthority(handoff),validated=clusterOptions({handoff:'private',output:'report',...options,rates:options.rates.join(',')});
 if(!Number.isInteger(startLeadMs)||startLeadMs<500||startLeadMs>5000)throw new Error('Invalid calibration start lead');
 const children=[],stages=[];let id=0;
 const control=async(path,method='GET')=>{const response=await fetch(new URL(path,authority.controlBase),{method,headers:{'x-calibration-token':authority.token},redirect:'error',signal:AbortSignal.timeout(5000)});if(!response.ok)throw new Error('Synthetic server control failed');if(response.status===204){await response.body?.cancel();return null;}return response.json();};
 try{
  for(let index=0;index<validated.workers;index++){
   const env=Object.fromEntries(['SystemRoot','WINDIR','TEMP','TMP','TZ'].filter(key=>process.env[key]!==undefined).map(key=>[key,process.env[key]]));
   const child=calibrationChannel(launch(fileURLToPath(import.meta.url),['--worker'],{env,stdio:['ignore','ignore','ignore','ipc'],execArgv:[]}));children.push(child);child.send({type:'init',index,authority:{...handoff}});
  }
  const ready=await Promise.all(children.map(child=>child.wait('ready',undefined,10000)));if(ready.some((row,index)=>row.index!==index))throw new Error('Calibration worker identity mismatch');
  for(const spec of calibrationRequests())for(const rate of validated.rates){
   if(Date.now()+startLeadMs+(validated.seconds+15)*1000>=authority.expiresAtMs)throw new Error('Synthetic server lifetime too short');
   const current=id++,budget=calibrationStageBudget(validated,rate);
   for(const [index,child]of children.entries())child.send({type:'prepare',id:current,endpoint:spec.name,seconds:validated.seconds,concurrency:budget.concurrency[index],rate:budget.rates[index]});
   const acknowledgments=await Promise.all(children.map(child=>child.wait('stage-ready',current,5000)));if(acknowledgments.some((row,index)=>row.index!==index))throw new Error('Calibration readiness mismatch');
   await control('/__reset','POST');const controllerCpu=process.cpuUsage(),startAtMs=Date.now()+startLeadMs;
   for(const child of children)child.send({type:'start',id:current,startAtMs});
   const results=await Promise.all(children.map(child=>child.wait('result',current,startLeadMs+(validated.seconds+15)*1000)));
   const stage=aggregateCalibrationWorkers(results,{workers:validated.workers,seconds:validated.seconds,concurrency:validated.concurrency,rate,endpoint:spec.name,startAtMs});
   const server=serverMetricsReport(await control('/__metrics'),authority.serverWorkers),cpu=process.cpuUsage(controllerCpu);
   stages.push({...stage,server,serverCpuOverScheduledSecondsUpperBound:(server.cpuUserMs+server.cpuSystemMs+server.primary.cpuUserMs+server.primary.cpuSystemMs)/(validated.seconds*1000),controller:{cpuUserMs:cpu.user/1000,cpuSystemMs:cpu.system/1000,rssBytes:process.memoryUsage().rss},serverDataWorkers:authority.serverWorkers,transportWindowSustained:stage.transportWindowSustained&&server.requests>=stage.accepted});
  }
  return clusterCalibrationReport(validated,stages);
 }finally{const stopped=await Promise.allSettled(children.map(child=>child.stop()));if(stopped.some(result=>result.status==='rejected'))throw new Error('Calibration worker shutdown unconfirmed');}
}

async function main(){
 if(process.argv[2]==='--worker'){await worker();return;}
 const args=Object.fromEntries(process.argv.slice(2).map(arg=>{const at=arg.indexOf('=');if(at<0)throw new Error();return [arg.slice(2,at),arg.slice(at+1)];}));
 const options=clusterOptions(args);await mkdir(dirname(resolve(args.output)),{recursive:true});
 try{const report=await runClusterCalibration({handoff:JSON.parse(await readPrivate(args.handoff)),options});await writeFile(args.output,JSON.stringify(report,null,2)+'\n',{mode:0o600});console.log('Synthetic pool calibration complete; aggregate report written.');}
 catch{await writeFile(args.output,JSON.stringify({...clusterCalibrationReport(options,[]),coordinationValid:false,reason:'Synthetic pool calibration failed; no headroom or application capacity proof.'})+'\n',{mode:0o600});throw new Error();}
}
if(process.argv[1]&&import.meta.url===pathToFileURL(resolve(process.argv[1])).href)main().catch(()=>{console.error('Synthetic pool calibration failed; no authority logged.');process.exitCode=1;});
