// Several generator hosts. A host agent runs on each additional generator
// machine and holds that machine's worker processes; the controller (on the
// first host) is a controller of controllers: it measures each host's clock
// offset, hands out a start time on its own clock, and merges the workers'
// raw histograms with the local ones. Nothing here averages independent runs.
//
// The agent is reachable only on a loopback or private rig address, takes one
// shared token, and carries per-worker private handoffs, so run it on the rig's
// private network only. Public bind addresses are refused.
import {createServer} from 'node:http';
import {fork} from 'node:child_process';
import {timingSafeEqual} from 'node:crypto';
import {mkdtemp,writeFile,unlink,rmdir} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import {dirname,join,resolve} from 'node:path';
import {fileURLToPath,pathToFileURL} from 'node:url';
import {childChannel} from './http-multi-channel.mjs';
import {classifyHost,parseTargetOrigin} from './http-targets.mjs';
import {readPrivate} from './external-control.mjs';
import {MAX_WORKERS_PER_HOST,MAX_WORKERS_TOTAL} from './http-load.mjs';

const BODY_LIMIT=256*1024*1024;
const equalToken=(a,b)=>typeof a==='string'&&typeof b==='string'&&a.length===b.length&&timingSafeEqual(Buffer.from(a),Buffer.from(b));

// NTP-style offset: host clock minus controller clock, from the lowest-round-trip
// sample. The uncertainty is half that round trip. Offsets are whole milliseconds.
export function estimateClockOffset(samples){
 if(!Array.isArray(samples)||!samples.length||samples.length>256)throw new Error('Invalid clock samples');
 let best=null;
 for(const {sentAtMs,hostNowMs,receivedAtMs} of samples){
  if(![sentAtMs,hostNowMs,receivedAtMs].every(Number.isFinite)||receivedAtMs<sentAtMs)throw new Error('Invalid clock sample');
  const rttMs=receivedAtMs-sentAtMs;
  if(!best||rttMs<best.rttMs)best={rttMs,offsetMs:Math.round(hostNowMs-(sentAtMs+receivedAtMs)/2)};
 }
 return {offsetMs:best.offsetMs,rttMs:best.rttMs,uncertaintyMs:Math.ceil(best.rttMs/2),samples:samples.length};
}

export async function startHostAgent({host,port=0,token,launch=fork,workerScript=join(dirname(fileURLToPath(import.meta.url)),'http-multi-worker.mjs'),now=Date.now}){
 if(classifyHost(host)==='refused')throw new Error('Host agent must bind a loopback or private rig address');
 if(typeof token!=='string'||token.length<16)throw new Error('Host agent needs a token of at least 16 characters');
 let state=null; // {directory,files,channels:Map(index->channel)}
 const reply=(res,status,body)=>{res.writeHead(status,{'content-type':'application/json'});res.end(JSON.stringify(body));};
 const readBody=req=>new Promise((resolveBody,reject)=>{
  const chunks=[];let size=0;
  req.on('data',chunk=>{size+=chunk.length;if(size>BODY_LIMIT){reject(new Error('Body too large'));req.destroy();}else chunks.push(chunk);});
  req.on('end',()=>{try{resolveBody(chunks.length?JSON.parse(Buffer.concat(chunks).toString('utf8')):{});}catch(error){reject(error);}});
  req.on('error',reject);
 });
 const teardown=async()=>{
  if(!state)return;
  const current=state;state=null;
  await Promise.allSettled([...current.channels.values()].map(channel=>channel.stop()));
  for(const path of current.files)await unlink(path).catch(()=>{});
  await rmdir(current.directory).catch(()=>{});
 };
 const server=createServer(async(req,res)=>{
  try{
   if(!equalToken(req.headers['x-host-token'],token))return reply(res,401,{error:'unauthorized'});
   if(req.method==='GET'&&req.url==='/clock')return reply(res,200,{nowMs:now()});
   if(req.method!=='POST')return reply(res,405,{error:'method'});
   const body=await readBody(req);
   if(req.url==='/init'){
    if(state||!Array.isArray(body.workers)||!body.workers.length||body.workers.length>MAX_WORKERS_PER_HOST)return reply(res,400,{error:'invalid init'});
    const directory=await mkdtemp(join(tmpdir(),'mw-host-agent-')),files=[],channels=new Map();
    state={directory,files,channels};
    const env=Object.fromEntries(['SystemRoot','WINDIR','TEMP','TMP','TZ'].filter(key=>process.env[key]!==undefined).map(key=>[key,process.env[key]]));
    for(const entry of body.workers){
     if(!Number.isInteger(entry?.workerIndex)||entry.workerIndex<0||entry.workerIndex>=MAX_WORKERS_TOTAL||channels.has(entry.workerIndex)||entry.data?.workerIndex!==entry.workerIndex)return reply(res,400,{error:'invalid worker'});
     const path=join(directory,'worker-'+entry.workerIndex+'.private.json');
     files.push(path);
     await writeFile(path,JSON.stringify(entry.data),{mode:0o600,flag:'wx'});
     channels.set(entry.workerIndex,childChannel(launch(workerScript,[path],{env,stdio:['ignore','ignore','ignore','ipc'],execArgv:[]}),entry.workerIndex));
    }
    return reply(res,200,{ok:true,workers:channels.size});
   }
   if(req.url==='/shutdown'){await teardown();return reply(res,200,{ok:true});}
   const channel=state?.channels.get(body.index);
   if(!channel)return reply(res,404,{error:'unknown worker'});
   if(req.url==='/send'){channel.send(body.message);return reply(res,200,{ok:true});}
   if(req.url==='/wait'){
    const timeoutMs=Number(body.timeoutMs);
    if(!Number.isFinite(timeoutMs)||timeoutMs<1||timeoutMs>3600000||typeof body.type!=='string')return reply(res,400,{error:'invalid wait'});
    return reply(res,200,{message:await channel.wait(body.type,body.stageId,timeoutMs)});
   }
   return reply(res,404,{error:'unknown route'});
  }catch{reply(res,500,{error:'failed'});}
 });
 await new Promise((done,fail)=>{server.once('error',fail);server.listen(port,host,done);});
 const bound=server.address().port;
 return {port:bound,origin:'http://'+(host.includes(':')?'['+host+']':host)+':'+bound,async close(){await teardown();server.closeAllConnections();await new Promise(done=>server.close(done));}};
}

// Controller-side handle on one host agent.
export function remoteHost({origin,token,fetchImpl=fetch,now=Date.now}){
 const target=parseTargetOrigin(origin);
 const call=async(path,body,timeoutMs=30000)=>{
  const response=await fetchImpl(new URL(path,target.origin),{method:body===undefined?'GET':'POST',headers:{'x-host-token':token,...(body===undefined?{}:{'content-type':'application/json'})},...(body===undefined?{}:{body:JSON.stringify(body)}),redirect:'error',signal:AbortSignal.timeout(timeoutMs)});
  if(!response.ok){await response.body?.cancel();throw new Error('Host agent refused the request');}
  return response.json();
 };
 return {
  origin:target.origin,
  async measureClock(count=8){
   const samples=[];
   for(let i=0;i<count;i++){
    const sentAtMs=now(),{nowMs}=await call('/clock');
    samples.push({sentAtMs,hostNowMs:nowMs,receivedAtMs:now()});
   }
   return estimateClockOffset(samples);
  },
  async init(workers){return call('/init',{workers},120000);},
  // A channel with the controller's worker-channel shape. Start times are sent
  // on the host's clock; reported windows are brought back to the controller's.
  channel(index,offsetMs){
   let failed=false,chain=Promise.resolve();
   return {index,clockOffsetMs:offsetMs,
    send(message){
     if(failed)throw new Error('Worker is unavailable');
     chain=chain.then(()=>call('/send',{index,message})).catch(()=>{failed=true;});
    },
    async wait(type,stageId,timeoutMs){
     await chain;
     if(failed)throw new Error('Worker is unavailable');
     try{return (await call('/wait',{index,type,stageId,timeoutMs},timeoutMs+10000)).message;}catch(error){failed=true;throw error;}
    },
    async stop(){}};
  },
  async shutdown(){await call('/shutdown',{}).catch(()=>{});},
 };
}

async function main(){
 const args=Object.fromEntries(process.argv.slice(2).map(arg=>{const at=arg.indexOf('=');if(at<0)throw new Error('Use --name=value');return [arg.slice(2,at),arg.slice(at+1)];}));
 if(Object.keys(args).some(key=>!['listen','token-file'].includes(key))||!args.listen||!args['token-file'])throw new Error('--listen=<private-address>:<port> and --token-file=<private file> required');
 const at=args.listen.lastIndexOf(':'),host=args.listen.slice(0,at).replace(/^\[|\]$/g,''),port=Number(args.listen.slice(at+1));
 if(at<0||!Number.isInteger(port)||port<1||port>65535)throw new Error('Invalid listen address');
 const agent=await startHostAgent({host,port,token:(await readPrivate(resolve(args['token-file']),4096)).trim()});
 console.log(JSON.stringify({listening:agent.origin}));
}
if(process.argv[1]&&import.meta.url===pathToFileURL(resolve(process.argv[1])).href)main().catch(error=>{console.error(error.message);process.exitCode=1;});
