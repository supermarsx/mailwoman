// Run separately from the generator, with a separate CPU/memory quota.
import {createServer} from 'node:http';
import {randomBytes,timingSafeEqual} from 'node:crypto';
import {open} from 'node:fs/promises';
import {resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
import {monitorEventLoopDelay} from 'node:perf_hooks';
import {JSON_BYTES,BINARY_BYTES} from './http-calibration-protocol.mjs';

export function calibrationHandler({token,reset,snapshot,record}){
 const expected=Buffer.from(token);
 return (req,res)=>{
  const supplied=Buffer.from(String(req.headers['x-calibration-token']??''));
  if(supplied.length!==expected.length||!timingSafeEqual(supplied,expected)){res.writeHead(404);res.end();return;}
  if(req.method==='POST'&&req.url==='/__reset'){reset();res.writeHead(204);res.end();return;}
  if(req.method==='GET'&&req.url==='/__metrics'){res.writeHead(200,{'content-type':'application/json','cache-control':'no-store'});res.end(JSON.stringify(snapshot()));return;}
  const payload=req.url==='/json'?JSON_BYTES:req.url==='/binary'?BINARY_BYTES:null;
  if(req.method!=='GET'||!payload){res.writeHead(404);res.end();return;}
  record(payload.length);
  res.writeHead(200,{'content-type':req.url==='/json'?'application/json':'application/octet-stream','content-length':payload.length,'cache-control':'no-store'});res.end(payload);
 };
}

export async function startCalibrationServer({handoff,seconds=900}){
 if(!handoff||!Number.isInteger(seconds)||seconds<10||seconds>900)throw new Error('Explicit handoff and bounded lifetime required');
 const token=randomBytes(32).toString('hex'),loop=monitorEventLoopDelay({resolution:10});loop.enable();
 let started=performance.now(),cpu=process.cpuUsage(),requests=0,bytes=0;
 const reset=()=>{started=performance.now();cpu=process.cpuUsage();requests=0;bytes=0;loop.reset();};
 const snapshot=()=>{const elapsedSeconds=(performance.now()-started)/1000,used=process.cpuUsage(cpu);return {requests,bytes,elapsedSeconds,cpuUserMs:used.user/1000,cpuSystemMs:used.system/1000,cpuLogicalCores:(used.user+used.system)/(elapsedSeconds*1000000),rssBytes:process.memoryUsage().rss,eventLoopDelayMs:{mean:Number.isFinite(loop.mean)?loop.mean/1e6:0,p99:loop.percentile(99)/1e6,max:loop.max/1e6}};};
 const server=createServer(calibrationHandler({token,reset,snapshot,record:size=>{requests++;bytes+=size;}}));
 server.requestTimeout=10000;server.headersTimeout=5000;server.maxHeadersCount=32;
 await new Promise((accept,reject)=>{server.once('error',reject);server.listen(0,'127.0.0.1',accept);});
 const expiresAtMs=Date.now()+seconds*1000;
 const close=async()=>{clearTimeout(timer);loop.disable();server.closeAllConnections();await new Promise(done=>server.close(done));};
 const timer=setTimeout(()=>{void close();},seconds*1000);
 try{
  const file=await open(handoff,'wx',0o600);
  try{await file.writeFile(JSON.stringify({schema:1,kind:'synthetic-transport-calibration',base:`http://127.0.0.1:${server.address().port}`,token,expiresAtMs}));await file.sync();}finally{await file.close();}
 }catch(error){await close();throw error;}
 return {close};
}

async function main(){
 const args=Object.fromEntries(process.argv.slice(2).map(arg=>{const at=arg.indexOf('=');if(at<0)throw new Error();return [arg.slice(2,at),arg.slice(at+1)];}));
 if(Object.keys(args).some(key=>!['handoff','seconds'].includes(key)))throw new Error();
 // Exclusive creation avoids silently reusing a previous server's authority.
 await startCalibrationServer({handoff:args.handoff,seconds:Number(args.seconds??900)});
 console.log('Synthetic calibration server ready; private handoff written.');
}
if(process.argv[1]&&import.meta.url===pathToFileURL(resolve(process.argv[1])).href)main().catch(()=>{console.error('Synthetic calibration server failed; no credentials logged.');process.exitCode=1;});
