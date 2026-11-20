// Disposable synthetic server pool. Never point this at application data.
import {fork} from 'node:child_process';
import {createServer} from 'node:http';
import {randomBytes,timingSafeEqual} from 'node:crypto';
import {open} from 'node:fs/promises';
import {resolve} from 'node:path';
import {fileURLToPath,pathToFileURL} from 'node:url';
import {monitorEventLoopDelay} from 'node:perf_hooks';
import {JSON_BYTES,BINARY_BYTES} from './http-calibration-protocol.mjs';
import {calibrationChannel} from './http-cluster-calibration-protocol.mjs';

const permitted=(req,token)=>{const incoming=Buffer.from(String(req.headers['x-calibration-token']??'')),expected=Buffer.from(token);return incoming.length===expected.length&&timingSafeEqual(incoming,expected);};
export function aggregateServerMetrics(samples,primary){
 if(!Array.isArray(samples)||!samples.length||samples.length>16||new Set(samples.map(row=>row.index)).size!==samples.length)throw new Error('Invalid calibration server metrics');
 for(const [index,sample]of samples.entries()){
  if(sample.index!==index||['requests','bytes'].some(key=>!Number.isSafeInteger(sample[key])||sample[key]<0)||['elapsedSeconds','cpuUserMs','cpuSystemMs','rssBytes'].some(key=>!Number.isFinite(sample[key])||sample[key]<0))throw new Error('Invalid calibration server counters');
 }
 const sum=key=>samples.reduce((total,row)=>total+row[key],0);
 return {schema:1,workers:samples.length,requests:sum('requests'),bytes:sum('bytes'),cpuUserMs:sum('cpuUserMs'),cpuSystemMs:sum('cpuSystemMs'),rssBytes:sum('rssBytes'),perWorker:samples,primary,counterMeaning:'response handler attempts and offered payload bytes, not confirmed delivered bytes',resourceMethod:'per-process observation intervals begin at reset; include client start lead and drain; RSS sum is not synchronized peak'};
}

async function worker(){
 process.on('disconnect',()=>process.exit(1));
 process.once('message',async config=>{
  try{
   if(config?.type!=='init'||!Number.isInteger(config.index)||config.index<0||config.index>15||typeof config.token!=='string'||!/^[a-f0-9]{64}$/.test(config.token)||!Number.isInteger(config.seconds)||config.seconds<10||config.seconds>900)throw new Error();
   const loop=monitorEventLoopDelay({resolution:10});loop.enable();let started=performance.now(),cpu=process.cpuUsage(),requests=0,bytes=0;
   const reset=()=>{started=performance.now();cpu=process.cpuUsage();requests=0;bytes=0;loop.reset();};
   const snapshot=()=>{const used=process.cpuUsage(cpu);return {index:config.index,requests,bytes,elapsedSeconds:(performance.now()-started)/1000,cpuUserMs:used.user/1000,cpuSystemMs:used.system/1000,rssBytes:process.memoryUsage().rss,eventLoopDelayMs:{mean:Number.isFinite(loop.mean)?loop.mean/1e6:0,p99:loop.percentile(99)/1e6,max:loop.max/1e6}};};
   const server=createServer((req,res)=>{
    const payload=req.url==='/json'?JSON_BYTES:req.url==='/binary'?BINARY_BYTES:null;
    if(!permitted(req,config.token)||req.method!=='GET'||!payload){res.writeHead(404);res.end();return;}
    requests++;bytes+=payload.length;res.writeHead(200,{'content-type':req.url==='/json'?'application/json':'application/octet-stream','content-length':payload.length,'cache-control':'no-store'});res.end(payload);
   });
   server.requestTimeout=10000;server.headersTimeout=5000;server.maxHeadersCount=32;
   await new Promise((accept,reject)=>{server.once('error',reject);server.listen(0,'127.0.0.1',accept);});
   process.on('message',message=>{if(!Number.isSafeInteger(message?.id)||message.id<0)return;
    if(message.type==='reset'){reset();process.send({type:'reset-done',id:message.id,index:config.index});}
    else if(message.type==='snapshot')process.send({type:'metrics',id:message.id,...snapshot()});
   });
   setTimeout(()=>process.exit(1),(config.seconds+10)*1000).unref();
   process.send({type:'ready',index:config.index,port:server.address().port});
  }catch{process.send?.({type:'failed'});process.exitCode=1;process.disconnect();}
 });
}

export async function startClusterCalibrationServer({handoff,workers=4,seconds=900,launch=fork}){
 if(!handoff||!Number.isInteger(workers)||workers<1||workers>16||!Number.isInteger(seconds)||seconds<10||seconds>900)throw new Error('Explicit bounded synthetic server configuration required');
 const token=randomBytes(32).toString('hex'),children=[];let id=0,closed=false,timer,server;
 let cpu=process.cpuUsage(),started=performance.now();
 const close=async()=>{if(closed)return;closed=true;clearTimeout(timer);if(server){server.closeAllConnections();await new Promise(done=>server.close(done));}const stopped=await Promise.allSettled(children.map(child=>child.stop()));if(stopped.some(result=>result.status==='rejected'))throw new Error('Calibration server worker exit unconfirmed');};
 try{
  for(let index=0;index<workers;index++){
   const env=Object.fromEntries(['SystemRoot','WINDIR','TEMP','TMP','TZ'].filter(key=>process.env[key]!==undefined).map(key=>[key,process.env[key]]));
   const child=calibrationChannel(launch(fileURLToPath(import.meta.url),['--worker'],{env,stdio:['ignore','ignore','ignore','ipc'],execArgv:[]}));children.push(child);child.send({type:'init',index,token,seconds});
  }
  const ready=await Promise.all(children.map(child=>child.wait('ready',undefined,10000)));
  if(ready.some((row,index)=>row.index!==index||!Number.isInteger(row.port)||row.port<1||row.port>65535)||new Set(ready.map(row=>row.port)).size!==workers)throw new Error('Calibration data listeners invalid');
  server=createServer(async(req,res)=>{
   try{
    if(!permitted(req,token)){res.writeHead(404);res.end();return;}
    if(req.method==='POST'&&req.url==='/__stop'){
     res.writeHead(204);res.end(()=>setImmediate(()=>{void close().catch(()=>{process.exitCode=1;});}));return;
    }
    const current=id++;
    if(req.method==='POST'&&req.url==='/__reset'){
     for(const child of children)child.send({type:'reset',id:current});await Promise.all(children.map(child=>child.wait('reset-done',current,5000)));
     cpu=process.cpuUsage();started=performance.now();res.writeHead(204);res.end();return;
    }
    if(req.method==='GET'&&req.url==='/__metrics'){
     for(const child of children)child.send({type:'snapshot',id:current});const samples=await Promise.all(children.map(child=>child.wait('metrics',current,5000))),used=process.cpuUsage(cpu);
     const metrics=aggregateServerMetrics(samples,{cpuUserMs:used.user/1000,cpuSystemMs:used.system/1000,elapsedSeconds:(performance.now()-started)/1000,rssBytes:process.memoryUsage().rss});
     res.writeHead(200,{'content-type':'application/json','cache-control':'no-store'});res.end(JSON.stringify(metrics));return;
    }
    res.writeHead(404);res.end();
   }catch{res.writeHead(503);res.end();}
  });
  server.requestTimeout=10000;server.headersTimeout=5000;server.maxHeadersCount=32;
  await new Promise((accept,reject)=>{server.once('error',reject);server.listen(0,'127.0.0.1',accept);});
  const authority={schema:1,kind:'synthetic-cluster-transport-calibration',token,controlBase:`http://127.0.0.1:${server.address().port}`,dataBases:ready.map(row=>`http://127.0.0.1:${row.port}`),serverWorkers:workers,expiresAtMs:Date.now()+seconds*1000};
  const file=await open(handoff,'wx',0o600);try{await file.writeFile(JSON.stringify(authority));await file.sync();}finally{await file.close();}
  timer=setTimeout(()=>{void close().catch(()=>{process.exitCode=1;});},seconds*1000);return {close};
 }catch(error){await close();throw error;}
}

async function main(){
 if(process.argv[2]==='--worker'){await worker();return;}
 const args=Object.fromEntries(process.argv.slice(2).map(arg=>{const at=arg.indexOf('=');if(at<0)throw new Error();return [arg.slice(2,at),arg.slice(at+1)];}));
 if(Object.keys(args).some(key=>!['handoff','workers','seconds'].includes(key)))throw new Error();
 await startClusterCalibrationServer({handoff:args.handoff,workers:Number(args.workers??4),seconds:Number(args.seconds??900)});console.log('Synthetic server pool ready; private authority written.');
}
if(process.argv[1]&&import.meta.url===pathToFileURL(resolve(process.argv[1])).href)main().catch(()=>{console.error('Synthetic server pool failed; no authority logged.');process.exitCode=1;});
