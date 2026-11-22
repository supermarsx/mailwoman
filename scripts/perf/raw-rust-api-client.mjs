import {fork,spawn} from 'node:child_process';
import {randomBytes,createHash} from 'node:crypto';
import {mkdtemp,chmod,lstat,readFile,writeFile,mkdir,rm} from 'node:fs/promises';
import {createReadStream} from 'node:fs';
import {tmpdir} from 'node:os';
import {join,resolve,dirname,basename,sep} from 'node:path';
import {fileURLToPath,pathToFileURL} from 'node:url';
import {setTimeout as sleep} from 'node:timers/promises';
import {measure} from './http-load.mjs';
import {readPrivate} from './external-control.mjs';
import {calibrationChannel} from './http-cluster-calibration-protocol.mjs';
import {rawAuthority,rawOptions,rawRequests,rawStage,rawMetrics,aggregateRaw,calibrationStageBudget,rawReport} from './raw-rust-api-protocol.mjs';
const runtimeEnv=()=>Object.fromEntries(['SystemRoot','WINDIR','TEMP','TMP','PATH','TZ'].filter(k=>process.env[k]!==undefined).map(k=>[k,process.env[k]]));
async function worker(){
 process.on('disconnect',()=>process.exit(1));
 process.once('message',config=>{
  try{
   if(config?.type!=='init'||!Number.isInteger(config.index)||config.index<0||config.index>15)throw Error();
   const authority=rawAuthority(config.authority);let phase=null,busy=false;
   setTimeout(()=>process.exit(1),Math.max(1,authority.expiresAtMs-Date.now()+1000)).unref();
   process.on('message',async message=>{
    try{
     if(message?.type==='prepare'){if(phase||busy)throw Error();phase=rawStage(message);process.send({type:'stage-ready',id:phase.id,index:config.index});return;}
     if(message?.type!=='start'||busy||!phase||message.id!==phase.id||!Number.isSafeInteger(message.startAtMs)||message.startAtMs<Date.now()-1000||message.startAtMs>Date.now()+10000)throw Error();
     busy=true;const current=phase,spec=rawRequests().find(s=>s.name===current.endpoint);
     const measurement=await measure({base:authority.dataBase,headers:{'x-calibration-token':authority.token},requests:[spec],concurrency:current.concurrency,seconds:current.seconds,timeoutMs:5000,arrivalRate:current.rate,commonStartAtMs:message.startAtMs,includeRawMeasurements:true});
     phase=null;busy=false;process.send({type:'result',schema:1,index:config.index,id:current.id,measurement});
    }catch{process.send({type:'failed'});}
   });
   process.send({type:'ready',index:config.index});
  }catch{process.send?.({type:'failed'});process.exitCode=1;}
 });
}
export async function runRawCalibration({handoff,options,startLeadMs=1000}){
 const authority=rawAuthority(handoff),validated=rawOptions({server:resolve('explicit-unused-binary'),output:'unused',...Object.fromEntries(Object.entries(options).map(([key,value])=>[key==='serverWorkers'?'server-workers':key,Array.isArray(value)?value.join(','):value]))});
 if(authority.runtimeWorkers!==validated.serverWorkers||!Number.isInteger(startLeadMs)||startLeadMs<500||startLeadMs>5000)throw Error('Raw runtime/coordination mismatch');
 const control=async(path,method='GET')=>{
  const response=await fetch(new URL(path,authority.controlBase),{method,headers:{'x-calibration-token':authority.token},redirect:'error',signal:AbortSignal.timeout(5000)});
  if(!response.ok)throw Error('Raw Rust control failed');
  if(response.status===204){await response.body?.cancel();return null;}
  return response.json();
 };
 // Exact status/framing before load; handler counters are reset per phase.
 for(const spec of rawRequests()){
  const response=await fetch(new URL(spec.path,authority.dataBase),{headers:{'x-calibration-token':authority.token},redirect:'error',signal:AbortSignal.timeout(5000)});
  if(response.status!==(spec.name==='raw-rust-empty-204'?204:200)||!spec.validateBytes(Buffer.from(await response.arrayBuffer())))throw Error('Raw Rust framing preflight failed');
 }
 const children=[],stages=[];let id=0;
 try{
  for(let index=0;index<validated.workers;index++){
   const child=calibrationChannel(fork(fileURLToPath(import.meta.url),['--worker'],{env:runtimeEnv(),stdio:['ignore','ignore','ignore','ipc'],execArgv:[]}));
   children.push(child);child.send({type:'init',index,authority});
  }
  const ready=await Promise.all(children.map(child=>child.wait('ready',undefined,10000)));
  if(ready.some((row,index)=>row.index!==index))throw Error('Raw generator allocation mismatch');
  for(const spec of rawRequests())for(const rate of validated.rates){
   if(Date.now()+startLeadMs+(validated.seconds+15)*1000>=authority.expiresAtMs)throw Error('Raw listener lifetime cannot cover phase');
   const current=id++,budget=calibrationStageBudget(validated,rate);
   for(const[index,child]of children.entries())child.send({type:'prepare',id:current,endpoint:spec.name,seconds:validated.seconds,concurrency:budget.concurrency[index],rate:budget.rates[index]});
   const acknowledgments=await Promise.all(children.map(child=>child.wait('stage-ready',current,5000)));
   if(acknowledgments.some((row,index)=>row.index!==index))throw Error('Raw generator readiness mismatch');
   await control('/__reset','POST');const cpu=process.cpuUsage(),startAtMs=Date.now()+startLeadMs;
   for(const child of children)child.send({type:'start',id:current,startAtMs});
   const results=await Promise.all(children.map(child=>child.wait('result',current,startLeadMs+(validated.seconds+15)*1000)));
   const stage=aggregateRaw(results,{workers:validated.workers,seconds:validated.seconds,concurrency:validated.concurrency,rate,endpoint:spec.name,startAtMs});
   const server=rawMetrics(await control('/__metrics'),authority.runtimeWorkers),used=process.cpuUsage(cpu);
   if(server.handlerInvocations<stage.accepted||server.handlerBytesOffered<stage.acceptedDecodedBytes)throw Error('Raw handler observations below accepted body proof');
   stages.push({...stage,server,controller:{cpuUserMs:used.user/1000,cpuSystemMs:used.system/1000,rssBytes:process.memoryUsage().rss}});
  }
  return rawReport(validated,stages);
 }finally{
  const stopped=await Promise.allSettled(children.map(child=>child.stop()));
  if(stopped.some(result=>result.status==='rejected'))throw Error('Raw generator exit unconfirmed');
 }
}
async function binaryHash(path){
 const stat=await lstat(path);if(!stat.isFile()||stat.isSymbolicLink()||stat.size>256*1024*1024)throw Error('Bounded regular example binary required');
 const hash=createHash('sha256');for await(const chunk of createReadStream(path))hash.update(chunk);
 return {sha256:hash.digest('hex'),bytes:stat.size};
}
export async function startOwnedRawServer({server,options,spawnServer=spawn}){
 const artifact=await binaryHash(server),directory=await mkdtemp(join(tmpdir(),'mw-raw-rust-api-'));
 await chmod(directory,0o700);const handoff=join(directory,'authority.private.json'),token=randomBytes(32).toString('hex');
 let child,exit=null,resolveExit,ready=false,readyText='',invalidReady=false;
 const terminal=new Promise(resolve=>{resolveExit=resolve;});
 const stop=async()=>{
  if(child&&exit===null){
   try{const authority=rawAuthority(JSON.parse(await readPrivate(handoff,4096)));await fetch(new URL('/__stop',authority.controlBase),{method:'POST',headers:{'x-calibration-token':token},redirect:'error',signal:AbortSignal.timeout(1000)});}catch{}
   if(exit===null)await Promise.race([terminal,sleep(2000)]);
   if(exit===null){child.kill('SIGKILL');await Promise.race([terminal,sleep(5000)]);}
   if(exit===null)throw Error('Original raw Rust process exit unconfirmed');
  }
  const final=await binaryHash(server);if(final.sha256!==artifact.sha256||final.bytes!==artifact.bytes)throw Error('Raw Rust binary changed during run');
 };
 try{
  child=spawnServer(server,[],{env:{...runtimeEnv(),MW_RUN_RAW_RUST_API_CALIBRATION:'1',MW_RAW_API_CALIBRATION_TOKEN:token,MW_RAW_API_CALIBRATION_HANDOFF:handoff,MW_RAW_API_RUNTIME_WORKERS:String(options.serverWorkers),MW_RAW_API_LIFETIME_SECONDS:String(options.lifetime)},stdio:['ignore','pipe','ignore']});
  child.stdout?.on('data',chunk=>{
   if(Buffer.byteLength(readyText)+chunk.length>256){invalidReady=true;return;}
   readyText+=chunk.toString('utf8');
   if(readyText.includes('\n')){ready=readyText.trim()==='{"ready":true,"rawRustAxum":true}';invalidReady=!ready;}
  });
  child.once('error',()=>{exit={error:true};resolveExit(exit);});child.once('exit',(code,signal)=>{exit={code,signal};resolveExit(exit);});
  const deadline=Date.now()+10000;let authority;
  while(Date.now()<deadline&&exit===null){
   if(invalidReady)throw Error('Raw readiness signal invalid');
   // The child emits readiness only AFTER write+fsync. Do not parse an
   // exclusively created file while the Rust producer is still writing it.
   if(ready){try{authority=rawAuthority(JSON.parse(await readPrivate(handoff,4096)));break;}catch{throw Error('Raw private handoff invalid');}}
   await sleep(20);
  }
  if(!authority||authority.token!==token||authority.runtimeWorkers!==options.serverWorkers)throw Error('Raw Rust startup/authority not observed');
  return {authority,artifact:{...artifact,name:basename(server)},async close(){await stop();const target=resolve(directory);if(!target.startsWith(resolve(tmpdir())+sep)||!basename(target).startsWith('mw-raw-rust-api-'))throw Error('Invalid owned cleanup path');await rm(target,{recursive:true,force:true});if(exit?.code!==0||exit?.signal)throw Error('Raw Rust child did not exit cleanly');}};
 }catch(error){await stop();const target=resolve(directory);if(target.startsWith(resolve(tmpdir())+sep)&&basename(target).startsWith('mw-raw-rust-api-'))await rm(target,{recursive:true,force:true});throw error;}
}
async function main(){
 if(process.argv[2]==='--worker'){await worker();return;}
 const args=Object.fromEntries(process.argv.slice(2).map(arg=>{const at=arg.indexOf('=');if(!arg.startsWith('--')||at<3)throw Error('Use explicit bounded raw calibration options');return[arg.slice(2,at),arg.slice(at+1)];}));
 const options=rawOptions(args),owned=await startOwnedRawServer({server:args.server,options});
 try{
  const report=await runRawCalibration({handoff:owned.authority,options});report.serverBinary=owned.artifact;
  await mkdir(dirname(resolve(args.output)),{recursive:true});await writeFile(args.output,JSON.stringify(report,null,2)+'\n',{mode:0o600});
  console.log('Raw Rust calibration written; no application capacity claim.');
 }finally{await owned.close();}
}
if(process.argv[1]&&import.meta.url===pathToFileURL(resolve(process.argv[1])).href)main().catch(()=>{console.error('Raw Rust calibration failed; no authority logged.');process.exitCode=1;});
