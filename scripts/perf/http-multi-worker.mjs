// Private child entrypoint. Its only argument is an owned 0600 handoff path.
import {resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
import {readPrivate,HANDOFF_LIMIT} from './external-control.mjs';
import {measure} from './http-load.mjs';
import {validateStage} from './http-multi-protocol.mjs';

export async function runWorker(path,{prepare,send=message=>process.send(message),onMessage=handler=>process.on('message',handler)}={}){
 const data=JSON.parse(await readPrivate(path,HANDOFF_LIMIT));
 if(data?.schema!==1||data.engineMode!=='native'||!Number.isInteger(data.workerIndex)||data.workerIndex<0||data.workerIndex>=16||!Array.isArray(data.principals)||!data.principals.length||data.principals.length>10000)throw new Error('Invalid worker assignment');
 const base=new URL(data.base);
 if(base.protocol!=='http:'||base.hostname!=='127.0.0.1'||!base.port||base.pathname!=='/'||base.search||base.hash||base.username||base.password)throw new Error('Invalid worker target');
 if(!prepare){const factory=await import('./fixture-requests.mjs');prepare=factory.prepareFixtureRequests;}
 const prepared=prepare({principals:data.principals,clientSimulation:data.clientSimulation,engineMode:'native',representative:data.representative,workloadProfile:data.workloadProfile});
 // The first owned identity must exercise every endpoint before measured load.
 for(const spec of prepared.firstRequests){
  const response=await fetch(new URL(spec.path,base),{method:spec.method??'GET',headers:{...prepared.headers,...spec.headers},...(spec.body===undefined?{}:{body:spec.body}),redirect:'error',signal:AbortSignal.timeout(10000)});
  if(response.status!==200){await response.body?.cancel();throw new Error('Worker preflight failed');}
  const chunks=[];let bytes=0;const reader=response.body?.getReader(),limit=spec.validateBytes?1024*1024:65536;
  if(reader)for(;;){const part=await reader.read();if(part.done)break;bytes+=part.value.length;if(bytes>limit){await reader.cancel();throw new Error('Worker preflight response too large');}chunks.push(part.value);}
  const content=Buffer.concat(chunks);
  if(spec.validateBytes){if(!spec.validateBytes(content))throw new Error('Worker preflight bytes failed');}
  else {const body=JSON.parse(content.toString('utf8'));if(spec.validate&&!spec.validate(body))throw new Error('Worker preflight validation failed');}
 }
 let phase=null,busy=false;
 onMessage(async message=>{
  try{
   if(message?.type==='stop'){process.exit(0);return;}
   if(message?.type==='prepare'){
    if(busy||phase||!Number.isSafeInteger(message.stageId)||message.stageId<0||!prepared.groups[message.endpoint])throw new Error('Invalid worker phase');
    validateStage({...message,workers:1});phase=message;
    send({type:'stage-ready',workerIndex:data.workerIndex,stageId:phase.stageId});return;
   }
   if(message?.type!=='start'||busy||!phase||message.stageId!==phase.stageId||!Number.isSafeInteger(message.startAtMs)||message.startAtMs<Date.now()-1000||message.startAtMs>Date.now()+10000)throw new Error('Invalid worker start');
   busy=true;
   const current=phase;
   const measurement=await measure({base:base.origin,headers:prepared.headers,requests:prepared.groups[current.endpoint],concurrency:current.concurrency,seconds:current.seconds,timeoutMs:current.timeoutMs,...(current.arrivalRate===undefined?{}:{arrivalRate:current.arrivalRate}),commonStartAtMs:message.startAtMs,includeRawMeasurements:true});
   phase=null;busy=false;
   send({type:'result',schema:1,workerIndex:data.workerIndex,partitionCount:data.principals.length,stageId:current.stageId,measurement});
  }catch{send({type:'failed',workerIndex:data.workerIndex,reason:'worker phase failed'});}
 });
 send({type:'ready',schema:1,workerIndex:data.workerIndex,partitionCount:data.principals.length});
}

if(process.argv[1]&&import.meta.url===pathToFileURL(resolve(process.argv[1])).href){
 if(process.argv.length!==3||!process.send){console.error('Private multiworker IPC entrypoint required.');process.exitCode=1;}
 else {process.on('disconnect',()=>process.exit(1));runWorker(process.argv[2]).catch(()=>{process.send?.({type:'failed',reason:'worker initialization failed'});process.exitCode=1;process.disconnect();});}
}
