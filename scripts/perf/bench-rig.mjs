// Bench rig (t62-n12): one command for plan runs R0a and R1 (t62-100k section 8).
//
//   node scripts/perf/bench-rig.mjs plan    --run=r0a|r1 [options]            print the plan (cpusets, legs, env); no Docker
//   node scripts/perf/bench-rig.mjs run     --run=r0a|r1 --run-id=<id> --image=<fixture image> [--diagnostics-image=<image>]
//                                           --report-root=<dir> --private-root=<dir> --confirm-bench-slot [options]
//   node scripts/perf/bench-rig.mjs analyse --dir=<report dir of one run>      rebuild summary.json/summary.md from saved files
//
// Options: --backend=sqlite|postgres  --vcpus=<Docker VM vCPUs, default 26>  --app=4 --database=4 --generator=4 --rig=1
//          (CPUs per role; consecutive from CPU 1; or --cpuset-app=0-3 style explicit sets)  --app-memory=8g
//          --transport-concurrency=4,8  --principals=<n>  --smoke (5 s stages)  --host-sampler (Windows host processes)
//          --host-logical-cpus=<n>  --legs=<comma list of leg ids>
//          --generator-impl=node|native (default node)  --native-generator-bin=<host path of a Linux mw-loadgen binary>
//          (native: scripts/perf/native-generator, t62-n10; one generator process with one thread per generator CPU;
//          R1 only: mw-loadgen has no template for the transport group)
//
// Each leg: volumes + (PostgreSQL) an internal network and a postgres:16 container; the fixture app container; the
// generator container in the app's network namespace; the cgroup sampler sidecar (same image, rig-sampler.mjs). All
// pinned to disjoint cpusets, recorded through MW_LOAD_HOST_*. After the generator exits: reports and samples are
// copied out, containers, volumes and network are removed, and the leg is analysed. The images must be built from
// `git archive` of a commit that contains this rig (the goal-shape/transport groups and the commit counter).
import {execFile,spawn} from 'node:child_process';
import {mkdir,readFile,writeFile,readdir,rm,stat} from 'node:fs/promises';
import {join,resolve,dirname,basename} from 'node:path';
import {fileURLToPath,pathToFileURL} from 'node:url';
import {randomBytes} from 'node:crypto';
import os from 'node:os';
import {allocate,validatePlan,dockerCpuArgs,hostEnv,ROLES} from './rig-cpuset.mjs';
import {parseSamples} from './rig-cgroup.mjs';
import {DEFAULT_THRESHOLDS} from './rig-quiet.mjs';
import {runDefinition,legRows,weightedMix,markdownSummary} from './rig-shapes.mjs';

const HERE=dirname(fileURLToPath(import.meta.url));
export const SAMPLE_INTERVAL_MS=250,HOST_INTERVAL_MS=2000;

export function parseArgs(argv){
 const [command,...rest]=argv;
 if(!['plan','run','analyse'].includes(command))throw new Error('Use plan, run or analyse');
 const args={};
 for(const arg of rest){
  if(!arg.startsWith('--'))throw new Error('Use --name=value');
  const at=arg.indexOf('=');const key=at<0?arg.slice(2):arg.slice(2,at),value=at<0?true:arg.slice(at+1);
  if(!['run','run-id','image','diagnostics-image','report-root','private-root','confirm-bench-slot','backend','vcpus','app','database','generator','upstream','rig','cpuset-app','cpuset-database','cpuset-generator','cpuset-upstream','cpuset-rig','app-memory','transport-concurrency','principals','smoke','host-sampler','host-logical-cpus','legs','dir','generator-impl','native-generator-bin'].includes(key))throw new Error('Unknown option --'+key);
  args[key]=value;
 }
 return {command,args};
}
const int=(value,fallback,min,max,name)=>{if(value===undefined)return fallback;const n=Number(value);if(!Number.isInteger(n)||n<min||n>max)throw new Error(`Invalid --${name}`);return n;};

// Everything a run needs, without side effects.
export function buildPlan(args){
 const backend=args.backend??'sqlite';
 if(!['sqlite','postgres'].includes(backend))throw new Error('Invalid --backend');
 const vcpus=int(args.vcpus,26,2,1024,'vcpus');
 const explicit=ROLES.some(role=>args['cpuset-'+role]!==undefined);
 const cpus=explicit?validatePlan(Object.fromEntries(ROLES.map(role=>[role,args['cpuset-'+role]])),{vcpus})
  :allocate({app:int(args.app,4,1,512,'app'),database:backend==='postgres'?int(args.database,4,1,512,'database'):0,generator:int(args.generator,4,1,512,'generator'),upstream:int(args.upstream,0,0,512,'upstream'),rig:int(args.rig,1,1,16,'rig')},{vcpus});
 if(backend==='postgres'&&!cpus.roles.database)throw new Error('A PostgreSQL run needs a database cpuset');
 if(!cpus.roles.rig)throw new Error('The sampler needs a rig cpuset');
 const transportConcurrency=String(args['transport-concurrency']??'4,8');
 if(!/^[0-9]{1,5}(,[0-9]{1,5}){0,9}$/.test(transportConcurrency))throw new Error('Invalid --transport-concurrency');
 const run=runDefinition(String(args.run??''),{backend,appCpus:cpus.roles.app.cpus,transportConcurrency,principals:args.principals===undefined?undefined:int(args.principals,0,1,10000,'principals'),smoke:args.smoke===true});
 const wanted=args.legs===undefined?null:String(args.legs).split(',');
 const legs=wanted?run.legs.filter(leg=>wanted.includes(leg.id)):run.legs;
 if(!legs.length||wanted&&legs.length!==wanted.length)throw new Error('Invalid --legs');
 const minConcurrency=legs=>Math.min(...legs.flatMap(leg=>leg.env.MW_LOAD_FIXTURE_CONCURRENCY.split(',').map(Number)));
 const generatorImpl=String(args['generator-impl']??'node');
 if(!['node','native'].includes(generatorImpl))throw new Error('Invalid --generator-impl');
 if(generatorImpl==='native'&&run.name==='r0a')throw new Error('The native generator has no template for the transport group; run R0a with --generator-impl=node');
 if(generatorImpl==='native'&&typeof args['native-generator-bin']!=='string')throw new Error('--generator-impl=native needs --native-generator-bin (a Linux mw-loadgen built as docs/perf/native-generator.md says)');
 // Node: one worker process per generator CPU (never more than the smallest concurrency). Native: one process,
 // one runtime thread per generator CPU (the multi-process controller is not wired to it).
 const generatorWorkers=generatorImpl==='native'?1:Math.max(1,Math.min(cpus.roles.generator.cpus,64,minConcurrency(legs)));
 const generator={implementation:generatorImpl,...(generatorImpl==='native'?{nativeBin:resolve(String(args['native-generator-bin'])),threads:Math.min(cpus.roles.generator.cpus,64)}:{})};
 const appMemory=String(args['app-memory']??'8g');
 if(!/^[0-9]{1,4}[mg]$/.test(appMemory))throw new Error('Invalid --app-memory');
 return {schema:1,run:run.name,description:run.description,backend,cpus,legs,generatorWorkers,generator,appMemory,host:hostEnv(cpus),
  sampler:{intervalMs:SAMPLE_INTERVAL_MS,hostIntervalMs:HOST_INTERVAL_MS,hostSampler:args['host-sampler']===true},
  thresholds:DEFAULT_THRESHOLDS,hostLogicalCpus:int(args['host-logical-cpus'],os.availableParallelism?.()??os.cpus().length,1,4096,'host-logical-cpus')};
}

// Docker argument lists for one leg (secrets passed through the private env file only).
export function legCommands(plan,leg,{prefix,image,envFile,pgEnvFile}){
 const owner=['--label',`mw.perf.owner=${prefix}`],scratch=`${prefix}-scratch`,data=`${prefix}-data`,net=`${prefix}-net`;
 const app=`${prefix}-app`,generator=`${prefix}-generator`,sampler=`${prefix}-sampler`,database=`${prefix}-pg`;
 const pg=plan.backend==='postgres';
 const hostFlags=Object.entries(plan.host).flatMap(([key,value])=>['-e',`${key}=${value}`]);
 const native=plan.generator?.implementation==='native';
 const nativeFlags=native?['--mount',`type=bind,source=${dirname(plan.generator.nativeBin)},target=/rig-bin,readonly`,'-e','MW_LOAD_GENERATOR=native',
  '-e',`MW_NATIVE_GENERATOR_BIN=/rig-bin/${basename(plan.generator.nativeBin)}`,'-e',`MW_NATIVE_GENERATOR_THREADS=${plan.generator.threads}`]:[];
 return {names:{app,generator,sampler,database,scratch,data,net},
  volumes:[['volume','create',...owner,scratch],['volume','create',...owner,data]],
  network:pg?['network','create','--internal',...owner,net]:null,
  setup:['run','--rm','--name',`${prefix}-setup`,...owner,'--network','none','--mount',`type=volume,source=${scratch},target=/scratch`,'--entrypoint','node',image,'-e',
   "const fs=require('fs');for(const p of ['/scratch/control','/scratch/reports','/scratch/rig']){fs.mkdirSync(p,{recursive:true,mode:0o700});fs.chmodSync(p,0o700);}"],
  database:pg?['run','-d','--name',database,...owner,'--network',net,'--network-alias','mw-load-pg',...dockerCpuArgs(plan.cpus,'database'),'--memory','4g','--memory-swap','4g',
   '--env-file',pgEnvFile,'postgres:16','-c','max_connections=300']:null,
  app:['run','-d','--name',app,...owner,'--network',pg?net:'none',...dockerCpuArgs(plan.cpus,'app'),'--memory',plan.appMemory,'--memory-swap',plan.appMemory,'--ulimit','nofile=65536:65536',
   '--mount',`type=volume,source=${scratch},target=/scratch`,'--mount',`type=volume,source=${data},target=/tmp`,'--env-file',envFile,image],
  generator:['run','-d','--name',generator,...owner,'--network',`container:${app}`,...dockerCpuArgs(plan.cpus,'generator'),'--memory','2g','--memory-swap','2g','--ulimit','nofile=65536:65536',
   '--mount',`type=volume,source=${scratch},target=/scratch`,...hostFlags,...nativeFlags,'--entrypoint','node',image,
   '/src/scripts/perf/http-external-driver.mjs','--root=/scratch/control','--campaigns=1','--wait-seconds=3600',`--workers=${plan.generatorWorkers}`],
  sampler:(ids)=>['run','-d','--name',sampler,...owner,'--network','none',...dockerCpuArgs(plan.cpus,'rig'),'--memory','256m',
   '--mount','type=bind,source=/sys/fs/cgroup,target=/host-cgroup,readonly','--mount',`type=volume,source=${scratch},target=/scratch`,'--entrypoint','node',image,
   '/src/scripts/perf/rig-sampler.mjs','--root=/host-cgroup','--out=/scratch/rig/samples.jsonl',`--interval-ms=${plan.sampler.intervalMs}`,
   ...Object.entries(ids).map(([label,id])=>`--label=${label}=${id}`)],
  cleanup:[['rm','-f',generator,sampler,app,...(pg?[database]:[])],['volume','rm','-f',scratch,data],...(pg?[['network','rm',net]]:[])]};
}

function dockerBinary(){
 if(process.env.MW_RIG_DOCKER)return process.env.MW_RIG_DOCKER;
 return process.platform==='win32'?'C:/Program Files/Docker/Docker/resources/bin/docker.exe':'docker';
}
export function realDocker(binary=dockerBinary()){
 return (args,{timeoutMs=600000}={})=>new Promise((resolveRun,reject)=>execFile(binary,args,{timeout:timeoutMs,maxBuffer:64<<20,windowsHide:true},(error,stdout)=>{
  if(error)reject(new Error(`docker ${args[0]} ${args[1]??''} failed`));else resolveRun(String(stdout).trim());
 }));
}

const exists=async path=>{try{await stat(path);return true;}catch{return false;}};

async function clockOffsetMs(docker,sampler){
 const t0=Date.now();const vm=Number(await docker(['exec',sampler,'node','-e','process.stdout.write(String(Date.now()))'],{timeoutMs:30000}));const t1=Date.now();
 return Number.isFinite(vm)?{offsetMs:vm-(t0+t1)/2,uncertaintyMs:(t1-t0)/2}:null;
}

async function containerNames(docker){
 const text=await docker(['ps','-a','--no-trunc','--format','{{.ID}} {{.Names}}']);
 return Object.fromEntries(text.split('\n').filter(Boolean).map(line=>{const [id,name]=line.split(' ');return [id.slice(0,12),name];}));
}

// One leg end to end. `docker` is injectable for tests.
export async function runLeg(plan,leg,{runId,image,reportDir,privateDir,docker=realDocker(),startHostSampler=null,waitTimeoutMs=4*3600*1000}){
 const prefix=`mw-rig-${runId}-${leg.id}`,legDir=join(reportDir,leg.id);
 if(await exists(legDir))throw new Error('Existing evidence is retained; choose a new --run-id');
 await mkdir(legDir,{recursive:true});await mkdir(privateDir,{recursive:true,mode:0o700});
 const envFile=join(privateDir,`${leg.id}.app.env`),pgEnvFile=join(privateDir,`${leg.id}.pg.env`);
 const key=randomBytes(32).toString('hex'),pgPassword=randomBytes(24).toString('hex');
 const env={...leg.env,MW_DB_KEY:key,...(plan.backend==='postgres'?{MW_LOAD_FIXTURE_PG_DSN:`postgres://mailwoman:${pgPassword}@mw-load-pg:5432/mailwoman`}:{})};
 await writeFile(envFile,Object.entries(env).map(([k,v])=>`${k}=${v}`).join('\n')+'\n',{mode:0o600});
 if(plan.backend==='postgres')await writeFile(pgEnvFile,`POSTGRES_USER=mailwoman\nPOSTGRES_PASSWORD=${pgPassword}\nPOSTGRES_DB=mailwoman\n`,{mode:0o600});
 const commands=legCommands(plan,leg,{prefix,image,envFile,pgEnvFile});
 const infrastructure={schema:1,run:plan.run,leg:leg.id,prefix,image,imageId:null,backend:plan.backend,cpusets:Object.fromEntries(Object.entries(plan.cpus.roles).map(([role,v])=>[role,v.cpuset])),
  vmVcpus:plan.cpus.vcpus,outsideCpuset:plan.cpus.outsideCpuset,generatorWorkers:plan.generatorWorkers,appMemory:plan.appMemory,instrumented:leg.instrumented,
  environment:leg.env,privateEnvironment:'outside the report folder; deleted after the leg',startedAtUtc:new Date().toISOString()};
 let hostSampler=null;
 try{
  infrastructure.imageId=await docker(['image','inspect','--format','{{.Id}}',image]);
  for(const args of commands.volumes)await docker(args);
  if(commands.network)await docker(commands.network);
  await docker(commands.setup);
  const ids={};
  if(commands.database){ids.database=await docker(commands.database);
   for(let i=0;;i++){try{await docker(['exec',commands.names.database,'pg_isready','-U','mailwoman','-d','mailwoman'],{timeoutMs:10000});break;}catch{if(i>60)throw new Error('PostgreSQL did not become ready');await new Promise(r=>setTimeout(r,1000));}}}
  ids.app=await docker(commands.app);
  ids.generator=await docker(commands.generator);
  infrastructure.containerIds=Object.fromEntries(Object.entries(ids).map(([k,v])=>[k,v.slice(0,12)]));
  await docker(commands.sampler(ids));
  infrastructure.clock=await clockOffsetMs(docker,commands.names.sampler);
  infrastructure.containersAtStart=await containerNames(docker);
  if(startHostSampler)hostSampler=await startHostSampler(legDir);
  infrastructure.generatorExit=Number(await docker(['wait',commands.names.generator],{timeoutMs:waitTimeoutMs}));
  infrastructure.appExit=Number(await docker(['wait',commands.names.app],{timeoutMs:600000}));
  infrastructure.containersAtEnd=await containerNames(docker);
  await docker(['stop','-t','10',commands.names.sampler]);
  if(hostSampler)await hostSampler.stop();
  await docker(['cp',`${commands.names.app}:/scratch/reports/.`,join(legDir,'reports')]);
  await docker(['cp',`${commands.names.sampler}:/scratch/rig/samples.jsonl`,join(legDir,'samples.jsonl')]);
  await writeFile(join(legDir,'app.log'),await docker(['logs','--tail','400',commands.names.app]).catch(()=>'')+'\n');
 }finally{
  if(hostSampler)await hostSampler.stop().catch(()=>{});
  infrastructure.finishedAtUtc=new Date().toISOString();
  for(const args of commands.cleanup)await docker(args).catch(()=>{});
  await rm(envFile,{force:true});await rm(pgEnvFile,{force:true});
  await writeFile(join(legDir,'infrastructure.json'),JSON.stringify(infrastructure,null,2)+'\n');
 }
 return legDir;
}

// Windows host sampler (PowerShell loop), stopped through a stop file.
export function windowsHostSampler(){
 return async legDir=>{
  const out=join(legDir,'host-samples.jsonl'),stopFile=join(legDir,'host-sampler.stop');
  const child=spawn('powershell.exe',['-NoProfile','-ExecutionPolicy','Bypass','-File',join(HERE,'rig-host-sampler.ps1'),'-Out',out,'-StopFile',stopFile,'-IntervalMs',String(HOST_INTERVAL_MS)],{stdio:'ignore',windowsHide:true});
  const exited=new Promise(r=>child.once('exit',r));
  return {pid:child.pid,stop:async()=>{await writeFile(stopFile,'stop\n');await Promise.race([exited,new Promise(r=>setTimeout(r,15000))]);if(child.exitCode===null)child.kill();}};
 };
}

async function readHostSamples(path,offsetMs){
 let text;try{text=await readFile(path,'utf8');}catch{return {samples:[],excludePids:[]};}
 const samples=[],excludePids=[process.pid];
 for(const line of text.split('\n')){
  if(!line.trim())continue;
  try{const v=JSON.parse(line);if(Number.isInteger(v.samplerPid)){excludePids.push(v.samplerPid);continue;}if(Number.isFinite(v.tMs)&&v.processes)samples.push({tMs:v.tMs+(offsetMs??0),processes:v.processes});}catch{/* partial last line */}
 }
 return {samples,excludePids};
}

// Rows of one leg directory.
export async function analyseLeg(plan,leg,legDir){
 const infrastructure=JSON.parse(await readFile(join(legDir,'infrastructure.json'),'utf8'));
 const {samples,malformed}=parseSamples(await readFile(join(legDir,'samples.jsonl'),'utf8').catch(()=>''));
 const names=infrastructure.containersAtEnd??infrastructure.containersAtStart??{};
 for(const sample of samples)for(const [id,row] of Object.entries(sample.others??{}))if(names[id])row.name=names[id];
 const {samples:hostSamples,excludePids}=await readHostSamples(join(legDir,'host-samples.jsonl'),infrastructure.clock?.offsetMs);
 const reportsDir=join(legDir,'reports');
 const files=(await readdir(reportsDir).catch(()=>[])).filter(name=>/^native-[0-9]+\.json$/.test(name));
 if(files.length!==1)return {rows:[],notes:[`${leg.id}: expected one native-<users>.json report, found ${files.length}`]};
 const report=JSON.parse(await readFile(join(reportsDir,files[0]),'utf8'));
 const labels={app:'app',generator:'generator',...(plan.backend==='postgres'?{database:'database'}:{})};
 const runContainerIds=Object.values(infrastructure.containerIds??{}).concat(infrastructure.prefix?Object.entries(names).filter(([,n])=>n.startsWith(infrastructure.prefix)).map(([id])=>id):[]);
 const rows=legRows(report,{leg,samples,hostSamples,plan:plan.cpus,labels,runContainerIds,hostLogicalCpus:plan.hostLogicalCpus,excludePids,thresholds:plan.thresholds,intervalMs:plan.sampler.intervalMs,hostIntervalMs:plan.sampler.hostIntervalMs});
 const notes=[];
 if(malformed)notes.push(`${leg.id}: ${malformed} malformed sampler lines ignored`);
 if(report.kind!=='coordinated-native-http-fixture')notes.push(`${leg.id}: report kind ${report.kind??'unknown'} (a failure report has no stages)`);
 if(!hostSamples.length)notes.push(`${leg.id}: Windows host processes were not sampled (--host-sampler not given): work outside the Docker VM is not in the quiet verdict`);
 return {rows,notes,judgedAgainst:report.judgedAgainst??null};
}

export async function writeSummary(plan,dir,legResults){
 const rows=legResults.flatMap(result=>result.rows);
 const summary={schema:1,run:plan.run,description:plan.description,backend:plan.backend,generatedAt:new Date().toISOString(),
  host:{vmVcpus:plan.cpus.vcpus,hostLogicalCpus:plan.hostLogicalCpus,note:'VM vCPUs float over host cores; cpusets pin containers to vCPUs, not to physical cores'},
  cpusets:Object.fromEntries(Object.entries(plan.cpus.roles).map(([role,v])=>[role,v.cpuset])),thresholds:plan.thresholds,
  rows,...(plan.run==='r1'?{mix:weightedMix(rows)}:{}),notes:legResults.flatMap(result=>result.notes),
  reportFields:{usecCpuPerRequest:'per container and server total, from cgroup cpu.stat',rpsPerLogicalCpu:'accepted rps per allotted and per consumed server CPU',
   databaseCommitsPerRequest:'transaction commits + autocommit writes (mw-store pool telemetry commit counter)',contaminated:'quiet-host witness, plan section 8 rule 1'}};
 await writeFile(join(dir,'summary.json'),JSON.stringify(summary,null,2)+'\n');
 await writeFile(join(dir,'summary.md'),markdownSummary(summary));
 return summary;
}

async function main(){
 const {command,args}=parseArgs(process.argv.slice(2));
 if(command==='analyse'){
  if(!args.dir)throw new Error('--dir required');
  const dir=resolve(String(args.dir)),plan=JSON.parse(await readFile(join(dir,'plan.json'),'utf8'));
  const results=[];for(const leg of plan.legs)results.push(await analyseLeg(plan,leg,join(dir,leg.id)));
  const summary=await writeSummary(plan,dir,results);console.log(join(dir,'summary.md'));return summary;
 }
 const plan=buildPlan(args);
 if(command==='plan'){console.log(JSON.stringify(plan,null,2));return plan;}
 if(args['confirm-bench-slot']!==true)throw new Error('run needs --confirm-bench-slot: take the bench lock in .orchestration/state.md first (and the quiet lock for stages above 8 CPUs)');
 const runId=String(args['run-id']??'');if(!/^[a-z0-9][a-z0-9-]{0,31}$/.test(runId))throw new Error('Invalid --run-id');
 if(!args.image||!args['report-root']||!args['private-root'])throw new Error('--image, --report-root and --private-root required');
 if(plan.generator.implementation==='native')await stat(plan.generator.nativeBin).catch(()=>{throw new Error('--native-generator-bin does not exist');});
 if(plan.legs.some(leg=>leg.image==='diagnostics')&&!args['diagnostics-image'])throw new Error('R1 needs --diagnostics-image (fixture built with --build-arg MW_FIXTURE_DIAGNOSTICS=1), or --legs without diagnostics-shapes');
 const dir=join(resolve(String(args['report-root'])),runId),privateRoot=resolve(String(args['private-root']));
 if(await exists(dir))throw new Error('Existing evidence is retained; choose a new --run-id');
 await mkdir(dir,{recursive:true});await writeFile(join(dir,'plan.json'),JSON.stringify(plan,null,2)+'\n');
 const results=[];
 for(const leg of plan.legs){
  const image=leg.image==='diagnostics'?String(args['diagnostics-image']):String(args.image);
  const legDir=await runLeg(plan,leg,{runId,image,reportDir:dir,privateDir:join(privateRoot,`private-${runId}`),startHostSampler:plan.sampler.hostSampler?windowsHostSampler():null});
  results.push(await analyseLeg(plan,leg,legDir));
 }
 await rm(join(privateRoot,`private-${runId}`),{recursive:true,force:true});
 await writeSummary(plan,dir,results);console.log(join(dir,'summary.md'));
}
if(process.argv[1]&&import.meta.url===pathToFileURL(resolve(process.argv[1])).href)main().catch(error=>{console.error(error.message);process.exitCode=1;});
