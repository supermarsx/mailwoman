// Bench rig (t62-n12): cgroup sampler sidecar. Runs in its own container on the rig cpuset with the VM's cgroup v2
// tree mounted read-only, and appends one JSON line per interval:
//   {schema:1,tMs,containers:{label:{id,cpu:{usage_usec,...}}|{id,error}},others:{id:{usage_usec}},procStat:{cpus}}
// `containers` are this run's (given as --label=<label>=<full container id>); `others` are every other container
// cgroup found under the tree (for the quiet-host witness). /proc/stat is not namespaced, so it shows every VM vCPU.
// Reads files only; no Docker API, no network. Stops on SIGTERM/SIGINT or after --max-seconds.
import {readFile,readdir,appendFile} from 'node:fs/promises';
import {join,resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
import {setTimeout as sleep} from 'node:timers/promises';
import {parseCpuStat,parseProcStat,SAMPLE_SCHEMA} from './rig-cgroup.mjs';

const ID=/(?:^|[-_])([0-9a-f]{64})(?:\.scope)?$/;
// Directories under the cgroup root whose name carries a 64-hex container id (cgroupfs: docker/<id>; systemd:
// system.slice/docker-<id>.scope), to depth 4. Returns Map(id -> absolute directory).
export async function discover(root,{maxDepth=4,list=readdir}={}){
 const found=new Map();
 const walk=async(dir,depth)=>{
  let entries;try{entries=await list(dir,{withFileTypes:true});}catch{return;}
  for(const entry of entries){
   if(!entry.isDirectory())continue;
   const path=join(dir,entry.name),match=ID.exec(entry.name);
   if(match){if(!found.has(match[1]))found.set(match[1],path);continue;}
   if(depth<maxDepth)await walk(path,depth+1);
  }
 };
 await walk(root,1);
 return found;
}

// cgroup v2 memory of one container: memory.current/peak, memory.events, the main memory.stat lines, pids.current
// (tasks = threads of every process in the cgroup). Missing files (controller not enabled) are left out.
export async function readMemory(dir,read=path=>readFile(path,'utf8')){
 const out={};
 const int=async name=>{try{const v=Number((await read(join(dir,name))).trim());return Number.isSafeInteger(v)?v:null;}catch{return null;}};
 const current=await int('memory.current'),peak=await int('memory.peak'),pids=await int('pids.current');
 if(current!==null)out.current=current;if(peak!==null)out.peak=peak;if(pids!==null)out.pids=pids;
 try{out.events=parseKeyValues(await read(join(dir,'memory.events')),['low','high','max','oom','oom_kill']);}catch{}
 try{out.stat=parseKeyValues(await read(join(dir,'memory.stat')),['anon','file','kernel','kernel_stack','sock','shmem','slab']);}catch{}
 return Object.keys(out).length?out:null;
}
export function parseKeyValues(text,keys){
 const out={};
 for(const line of String(text).split(/\r?\n/)){const m=/^([a-z_]+) ([0-9]+)$/.exec(line.trim());if(m&&keys.includes(m[1])){const v=Number(m[2]);if(Number.isSafeInteger(v))out[m[1]]=v;}}
 return out;
}
export async function sampleOnce({labels,discovered,procStatPath='/proc/stat',read=path=>readFile(path,'utf8'),now=Date.now}){
 const tMs=now(),containers={},others={};
 for(const [label,id] of Object.entries(labels)){
  const dir=discovered.get(id);
  if(!dir){containers[label]={id:id.slice(0,12),error:'cgroup not found'};continue;}
  try{containers[label]={id:id.slice(0,12),cpu:parseCpuStat(await read(join(dir,'cpu.stat')))};}
  catch{containers[label]={id:id.slice(0,12),error:'cpu.stat unreadable'};continue;}
  const mem=await readMemory(dir,read);if(mem)containers[label].mem=mem;
 }
 const own=new Set(Object.values(labels));
 for(const [id,dir] of discovered){
  if(own.has(id))continue;
  try{others[id.slice(0,12)]={usage_usec:parseCpuStat(await read(join(dir,'cpu.stat'))).usage_usec};}catch{/* exited */}
 }
 let procStat=null;try{procStat=parseProcStat(await read(procStatPath));}catch{procStat=null;}
 return {schema:SAMPLE_SCHEMA,tMs,containers,others,procStat};
}

export function parseArgs(argv){
 const labels={},args={};
 for(const arg of argv){
  const at=arg.indexOf('=');if(!arg.startsWith('--')||at<0)throw new Error('Use --name=value');
  const key=arg.slice(2,at),value=arg.slice(at+1);
  if(key==='label'){const [label,id]=value.split('=');if(!/^[a-z][a-z0-9-]{0,31}$/.test(label??'')||!/^[0-9a-f]{64}$/.test(id??''))throw new Error('Use --label=<label>=<64-hex container id>');labels[label]=id;continue;}
  if(!['root','out','interval-ms','max-seconds','proc-stat','rediscover-ms'].includes(key))throw new Error('Unknown option '+key);
  args[key]=value;
 }
 const intervalMs=Number(args['interval-ms']??250),maxSeconds=Number(args['max-seconds']??14400),rediscoverMs=Number(args['rediscover-ms']??5000);
 if(!args.root||!args.out||!Object.keys(labels).length)throw new Error('Explicit --root, --out and at least one --label required');
 if(!Number.isInteger(intervalMs)||intervalMs<50||intervalMs>10000||!Number.isFinite(maxSeconds)||maxSeconds<1||maxSeconds>86400||!Number.isInteger(rediscoverMs)||rediscoverMs<intervalMs)throw new Error('Invalid sampler bounds');
 return {root:resolve(args.root),out:resolve(args.out),procStatPath:args['proc-stat']??'/proc/stat',intervalMs,maxSeconds,rediscoverMs,labels};
}

export async function run(options,{stopped=()=>false}={}){
 const started=Date.now();let discovered=await discover(options.root),lastDiscovery=started,lines=0;
 while(!stopped()&&Date.now()-started<options.maxSeconds*1000){
  const due=Date.now()+options.intervalMs;
  if(Date.now()-lastDiscovery>=options.rediscoverMs){discovered=await discover(options.root);lastDiscovery=Date.now();}
  await appendFile(options.out,JSON.stringify(await sampleOnce({labels:options.labels,discovered,procStatPath:options.procStatPath}))+'\n');lines++;
  await sleep(Math.max(1,due-Date.now()));
 }
 // One last sample so the final stage edge is bracketed.
 await appendFile(options.out,JSON.stringify(await sampleOnce({labels:options.labels,discovered,procStatPath:options.procStatPath}))+'\n');
 return lines+1;
}

if(process.argv[1]&&import.meta.url===pathToFileURL(resolve(process.argv[1])).href){
 let stop=false;for(const signal of ['SIGTERM','SIGINT'])process.on(signal,()=>{stop=true;});
 try{await run(parseArgs(process.argv.slice(2)),{stopped:()=>stop});}catch(error){console.error(error.message);process.exitCode=1;}
}
