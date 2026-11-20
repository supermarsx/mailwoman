import {writeFile,mkdir} from 'node:fs/promises';
import {dirname,resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
import {monitorEventLoopDelay} from 'node:perf_hooks';
import {measure} from './http-load.mjs';
import {readPrivate} from './external-control.mjs';
import {calibrationHandoff,calibrationOptions,calibrationRequests,calibrationReport} from './http-calibration-protocol.mjs';

export async function runCalibration({handoff,options,run=measure}){
 const authority=calibrationHandoff(handoff),headers={'x-calibration-token':authority.token},stages=[];
 const control=async(path,method='GET')=>{
  const response=await fetch(new URL(path,authority.base),{method,headers,redirect:'error',signal:AbortSignal.timeout(5000)});
  if(!response.ok)throw new Error('Calibration server control failed');
  if(response.status===204){await response.body?.cancel();return null;}
  return response.json();
 };
 for(const spec of calibrationRequests())for(const arrivalRate of options.rates){
  if(Date.now()+(options.seconds+10)*1000>=authority.expiresAtMs)throw new Error('Calibration server lifetime cannot cover the next phase');
  await control('/__reset','POST');
  const loop=monitorEventLoopDelay({resolution:10});loop.enable();
  let result;
  try{result=await run({base:authority.base,headers,requests:[spec],concurrency:options.concurrency,seconds:options.seconds,timeoutMs:5000,arrivalRate});}
  finally{loop.disable();}
  const server=await control('/__metrics');
  stages.push({endpoint:spec.name,...result,generatorEventLoopDelayMs:{mean:Number.isFinite(loop.mean)?loop.mean/1e6:0,p99:loop.percentile(99)/1e6,max:loop.max/1e6},server,transportWindowSustained:result.errors===0&&result.throttled===0&&result.timeouts===0&&result.droppedCapacity===0&&result.droppedScheduler===0&&result.acceptedDuringScheduledWindow===result.offered});
 }
 return calibrationReport(options,stages);
}

async function main(){
 const args=Object.fromEntries(process.argv.slice(2).map(arg=>{const at=arg.indexOf('=');if(at<0)throw new Error();return [arg.slice(2,at),arg.slice(at+1)];}));
 const options=calibrationOptions(args),handoff=JSON.parse(await readPrivate(args.handoff));
 const report=await runCalibration({handoff,options});
 await mkdir(dirname(resolve(args.output)),{recursive:true});await writeFile(args.output,JSON.stringify(report,null,2)+'\n',{mode:0o600});
 console.log('Synthetic calibration complete; credential-free report written.');
}
if(process.argv[1]&&import.meta.url===pathToFileURL(resolve(process.argv[1])).href)main().catch(()=>{console.error('Synthetic calibration failed; no credentials logged.');process.exitCode=1;});
