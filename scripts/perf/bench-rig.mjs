// Bench rig (t62-n12): one command for plan runs R0a and R1 (t62-100k section 8).
//
//   node scripts/perf/bench-rig.mjs plan    --run=r0a|r1 [options]            print the plan (cpusets, legs, env); no Docker
//   node scripts/perf/bench-rig.mjs run     --run=r0a|r1 --run-id=<id> --image=<fixture image> [--diagnostics-image=<image>]
//                                           --report-root=<dir> --private-root=<dir> --confirm-bench-slot [options]
//   node scripts/perf/bench-rig.mjs ab      --run=r0a|r1 --run-id=<id> --parent-image=<A> --candidate-image=<B>
//                                           [--parent-diagnostics-image --candidate-diagnostics-image] --report-root --private-root
//                                           --confirm-bench-slot   runs A B B A, then compares (default flow for a new image)
//   node scripts/perf/bench-rig.mjs analyse --dir=<report dir of one run>      rebuild summary.json/summary.md from saved files
//   node scripts/perf/bench-rig.mjs compare --candidate=<run dir>[,<run dir>] [--reference=<run dir>[,<run dir>] | --baseline=<file>]
//                                           [--threshold=0.08] [--out=<path prefix>] [--bless --reason="..."]
//                                           exit 1 on a regression; the baseline changes only with --bless and a reason
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
import {containerMemory} from './rig-memory.mjs';
import {DEFAULT_THRESHOLDS} from './rig-quiet.mjs';
import {runDefinition,legRows,weightedMix,markdownSummary} from './rig-shapes.mjs';
import {compare,markdownComparison,metricsOfSummary,readBaseline,writeBaseline,bless,DEFAULT_THRESHOLD} from './rig-compare.mjs';

const HERE=dirname(fileURLToPath(import.meta.url));
export const SAMPLE_INTERVAL_MS=250,HOST_INTERVAL_MS=2000;

export function parseArgs(argv){
 const [command,...rest]=argv;
 if(!['plan','run','ab','m3','analyse','compare'].includes(command))throw new Error('Use plan, run, ab, m3, analyse or compare');
 const args={};
 for(const arg of rest){
  if(!arg.startsWith('--'))throw new Error('Use --name=value');
  const at=arg.indexOf('=');const key=at<0?arg.slice(2):arg.slice(2,at),value=at<0?true:arg.slice(at+1);
  if(!['run','run-id','image','diagnostics-image','report-root','private-root','confirm-bench-slot','backend','vcpus','app','database','generator','upstream','rig','cpuset-app','cpuset-database','cpuset-generator','cpuset-upstream','cpuset-rig','app-memory','transport-concurrency','principals','smoke','host-sampler','host-logical-cpus','legs','dir','generator-impl','native-generator-bin','parent-image','candidate-image','parent-diagnostics-image','candidate-diagnostics-image','candidate','reference','baseline','threshold','bless','reason','out','arm-image-glibc','arm-image-musl','arm-image-jemalloc','arm-image-mimalloc','arena-max'].includes(key))throw new Error('Unknown option --'+key);
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

// The fixture image's C library and allocator, from ldd inside the image (the t63 audit: shipped installs are a musl
// static image, the fixture image was never checked). A Rust #[global_allocator] would not show in ldd output.
export function imageRuntime(text){
 const out=String(text??'');
 const libc=/GLIBC|GNU libc/i.test(out)?'glibc':/musl/i.test(out)?'musl':/not a dynamic executable|statically linked/i.test(out)?'static (libc not visible to ldd)':'unknown';
 const allocator=/jemalloc/i.test(out)?'jemalloc (shared library)':/mimalloc/i.test(out)?'mimalloc (shared library)':libc==='glibc'?'glibc malloc unless the binary sets a Rust #[global_allocator] (none in the workspace at t62-n12)':'not determined';
 return {libc,allocator,ldd:out.split(/\r?\n/).map(line=>line.trim()).filter(Boolean).slice(0,12)};
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
  infrastructure.imageRuntime=imageRuntime(await docker(['run','--rm','--network','none','--cpus','0.5','--entrypoint','sh',image,'-c','ldd --version 2>&1 | head -1; ldd /out/request-load-fixture 2>&1; grep PRETTY_NAME /etc/os-release'],{timeoutMs:60000}).catch(()=>''));
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
 const idleText=await readFile(join(legDir,'reports','idle-windows.jsonl'),'utf8').catch(()=>'');
 const idleWindows=idleText.split(/\r?\n/).filter(Boolean).flatMap(line=>{try{const w=JSON.parse(line);return Number.isFinite(w.startMs)&&Number.isFinite(w.endMs)?[{phase:String(w.phase),startMs:w.startMs,endMs:w.endMs,app:containerMemory(samples,'app',{startMs:w.startMs,endMs:w.endMs})}]:[];}catch{return [];}});
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
 return {rows,notes,judgedAgainst:report.judgedAgainst??null,imageRuntime:infrastructure.imageRuntime??null,idleWindows};
}

export async function writeSummary(plan,dir,legResults){
 const rows=legResults.flatMap(result=>result.rows);
 const summary={schema:1,run:plan.run,description:plan.description,backend:plan.backend,generatedAt:new Date().toISOString(),
  host:{vmVcpus:plan.cpus.vcpus,hostLogicalCpus:plan.hostLogicalCpus,note:'VM vCPUs float over host cores; cpusets pin containers to vCPUs, not to physical cores'},
  cpusets:Object.fromEntries(Object.entries(plan.cpus.roles).map(([role,v])=>[role,v.cpuset])),thresholds:plan.thresholds,
  imageRuntime:legResults.find(result=>result.imageRuntime)?.imageRuntime??null,
  idleWindows:legResults.flatMap(result=>result.idleWindows??[]),
  ...(plan.arm?{arm:plan.arm}:{}),
  rows,...(plan.run==='r1'?{mix:weightedMix(rows)}:{}),notes:legResults.flatMap(result=>result.notes),
  reportFields:{usecCpuPerRequest:'per container and server total, from cgroup cpu.stat',rpsPerLogicalCpu:'accepted rps per allotted and per consumed server CPU',
   databaseCommitsPerRequest:'transaction commits + autocommit writes (mw-store pool telemetry commit counter)',contaminated:'quiet-host witness, plan section 8 rule 1'}};
 await writeFile(join(dir,'summary.json'),JSON.stringify(summary,null,2)+'\n');
 await writeFile(join(dir,'summary.md'),markdownSummary(summary));
 return summary;
}

// One run (all legs of the plan) with one fixture image (and its diagnostics image). Returns the summary.
export async function runOnce(plan,{runId,image,diagnosticsImage,reportRoot,privateRoot,runLegImpl=runLeg,analyse=analyseLeg}){
 if(!/^[a-z0-9][a-z0-9-]{0,31}$/.test(runId))throw new Error('Invalid --run-id');
 if(plan.legs.some(leg=>leg.image==='diagnostics')&&!diagnosticsImage)throw new Error('R1 needs a diagnostics image (fixture built with --build-arg MW_FIXTURE_DIAGNOSTICS=1), or --legs without diagnostics-shapes');
 const dir=join(resolve(reportRoot),runId),privateDir=join(resolve(privateRoot),`private-${runId}`);
 if(await exists(dir))throw new Error('Existing evidence is retained; choose a new --run-id');
 await mkdir(dir,{recursive:true});await writeFile(join(dir,'plan.json'),JSON.stringify({...plan,image,diagnosticsImage:diagnosticsImage??null},null,2)+'\n');
 const results=[];
 for(const leg of plan.legs){
  const legDir=await runLegImpl(plan,leg,{runId,image:leg.image==='diagnostics'?diagnosticsImage:image,reportDir:dir,privateDir,startHostSampler:plan.sampler.hostSampler?windowsHostSampler():null});
  results.push(await analyse(plan,leg,legDir));
 }
 await rm(privateDir,{recursive:true,force:true});
 return {dir,summary:await writeSummary(plan,dir,results)};
}

// The A B B A order of a parent/candidate comparison: run ids and images.
export const M3_ARMS=Object.freeze([
 {id:'glibc',image:'arm-image-glibc',env:{},describe:'glibc malloc, default settings (the fixture image today)'},
 {id:'glibc-arena',image:'arm-image-glibc',env:{MALLOC_ARENA_MAX:'arena-max'},describe:'glibc malloc with MALLOC_ARENA_MAX (value from --arena-max, default 2: a commonly used setting, not measured here)'},
 {id:'musl',image:'arm-image-musl',env:{},describe:'musl static, as shipped (fixture-static target)'},
 {id:'jemalloc',image:'arm-image-jemalloc',env:{},describe:'jemalloc (needs an image built with the alloc-jemalloc feature; not available yet)'},
 {id:'mimalloc',image:'arm-image-mimalloc',env:{},describe:'mimalloc (needs an image built with the alloc-mimalloc feature; not available yet)'},
]);
// The arms that can run with the images given; the others are listed with the reason.
export function m3Arms(args){
 const arena=String(args['arena-max']??'2');if(!/^[1-9][0-9]{0,2}$/.test(arena))throw new Error('Invalid --arena-max');
 const run=[],skipped=[];
 for(const arm of M3_ARMS){
  const image=args[arm.image];
  if(typeof image!=='string'||!image){skipped.push({id:arm.id,reason:`no --${arm.image}`});continue;}
  run.push({id:arm.id,image,describe:arm.describe,env:Object.fromEntries(Object.entries(arm.env).map(([k,v])=>[k,v==='arena-max'?arena:v]))});
 }
 if(!run.length)throw new Error('m3 needs at least one --arm-image-<arm>');
 return {run,skipped};
}

export function abOrder(runId,{parent,candidate}){
 return [['a1',parent],['b1',candidate],['b2',candidate],['a2',parent]].map(([tag,images])=>({runId:`${runId}-${tag}`,tag,...images}));
}
// Run A B B A with `runOne(step)` and compare repeat 1 (a1 vs b1) and repeat 2 (a2 vs b2).
export async function runAb(order,runOne,{threshold=DEFAULT_THRESHOLD}={}){
 const done={};
 for(const step of order)done[step.tag]=await runOne(step);
 return compare([done.a1,done.a2].map(metricsOfSummary),[done.b1,done.b2].map(metricsOfSummary),{threshold});
}

// M3 table: per arm, app memory at each boundary, µs CPU per request and p99 of the mix.
export function m3Table(byArm,skipped=[]){
 const mib=v=>Number.isFinite(v)?(v/1048576).toFixed(1):'–';
 const lines=['| Arm | libc / allocator | App MiB end of idle before | App MiB end of mix (max inside) | App MiB end of idle after | µs CPU/req (server) | p99 ms | Quiet |','| --- | --- | ---: | ---: | ---: | ---: | ---: | --- |'];
 for(const [arm,summary] of Object.entries(byArm)){
  const row=summary.rows.find(r=>r.kind==='mix')??summary.rows[0];
  const app=Object.values(row?.memory?.containers??{}).find(c=>c.role==='app');
  const idle=phase=>summary.idleWindows?.find(w=>w.phase===phase)?.app?.currentEndBytes;
  lines.push(`| ${arm} | ${summary.imageRuntime?summary.imageRuntime.libc+' / '+summary.imageRuntime.allocator:'not recorded'} | ${mib(idle('before'))} | ${mib(app?.currentEndBytes)} (${mib(app?.maxSampledBytes)}) | ${mib(idle('after'))} | ${row?.usecCpuPerRequest?.server??'–'} | ${row?.latencyMs?.p99??'–'} | ${row?.contaminated?'CONTAMINATED':'quiet'} |`);
 }
 for(const s of skipped)lines.push(`| ${s.id} | – | not run: ${s.reason} | | | | | |`);
 lines.push('','Not built: 1,000 users open / close / reopen (the fixture seeds sessions once; the generator has no close/reopen phase).');
 return lines.join('\n')+'\n';
}

async function summaries(list){return Promise.all(String(list).split(',').map(async dir=>JSON.parse(await readFile(join(resolve(dir),'summary.json'),'utf8'))));}

export async function compareCommand(args,{baselinePath=resolve(String(args.baseline??join(HERE,'rig-baseline.json'))),write=process.stdout.write.bind(process.stdout)}={}){
 if(!args.candidate)throw new Error('compare needs --candidate=<run dir>[,<run dir>]');
 const candidates=(await summaries(args.candidate)).map(metricsOfSummary);
 const threshold=args.threshold===undefined?DEFAULT_THRESHOLD:Number(args.threshold);
 if(args.bless===true&&args.reference)throw new Error('--bless applies to the baseline file, not to --reference runs');
 let references;
 if(args.reference)references=(await summaries(args.reference)).map(metricsOfSummary);
 else{
  const baseline=await readBaseline(baselinePath);
  if(args.bless===true){
   if(candidates.length!==1)throw new Error('--bless takes exactly one candidate run');
   await writeBaseline(baselinePath,bless(baseline,candidates[0],{reason:args.reason,from:String(args.candidate)}));
   write(`baseline updated: ${baselinePath}\n`);return {blessed:true};
  }
  references=[baseline.rows];
 }
 const result=compare(references,candidates,{threshold});
 const markdown=markdownComparison(result);
 if(args.out){await writeFile(resolve(String(args.out))+'.json',JSON.stringify(result,null,2)+'\n');await writeFile(resolve(String(args.out))+'.md',markdown);}
 write(markdown);
 return result;
}

async function main(){
 const {command,args}=parseArgs(process.argv.slice(2));
 if(command==='analyse'){
  if(!args.dir)throw new Error('--dir required');
  const dir=resolve(String(args.dir)),plan=JSON.parse(await readFile(join(dir,'plan.json'),'utf8'));
  const results=[];for(const leg of plan.legs)results.push(await analyseLeg(plan,leg,join(dir,leg.id)));
  await writeSummary(plan,dir,results);console.log(join(dir,'summary.md'));return;
 }
 if(command==='compare'){const result=await compareCommand(args);if(result.regressions)process.exitCode=1;return;}
 const plan=buildPlan(args);
 if(command==='plan'){console.log(JSON.stringify(plan,null,2));return;}
 if(args['confirm-bench-slot']!==true)throw new Error(`${command} needs --confirm-bench-slot: take the bench lock in .orchestration/state.md first (and the quiet lock for stages above 8 CPUs)`);
 if(!args['report-root']||!args['private-root'])throw new Error('--report-root and --private-root required');
 if(plan.generator.implementation==='native')await stat(plan.generator.nativeBin).catch(()=>{throw new Error('--native-generator-bin does not exist');});
 const runId=String(args['run-id']??''),roots={reportRoot:String(args['report-root']),privateRoot:String(args['private-root'])};
 if(command==='run'){
  if(!args.image)throw new Error('--image required');
  const {dir}=await runOnce(plan,{runId,image:String(args.image),diagnosticsImage:args['diagnostics-image'],...roots});
  console.log(join(dir,'summary.md'));return;
 }
 if(command==='m3'){
  if(plan.run!=='m3')throw new Error('m3 needs --run=m3');
  const {run,skipped}=m3Arms(args),summaries={};
  for(const arm of run){
   const armPlan={...plan,legs:plan.legs.map(leg=>({...leg,env:{...leg.env,...arm.env}})),arm:{id:arm.id,describe:arm.describe,env:arm.env}};
   summaries[arm.id]=(await runOnce(armPlan,{runId:`${runId}-${arm.id}`,image:arm.image,...roots})).summary;
  }
  const table=m3Table(summaries,skipped);
  await writeFile(join(resolve(roots.reportRoot),`${runId}-m3.md`),table);process.stdout.write(table);return;
 }
 // ab: the default flow for a new image. Parent vs candidate, order A B B A, then compare.
 for(const key of ['parent-image','candidate-image'])if(!args[key])throw new Error(`ab needs --${key}`);
 const order=abOrder(runId,{parent:{image:String(args['parent-image']),diagnosticsImage:args['parent-diagnostics-image']},candidate:{image:String(args['candidate-image']),diagnosticsImage:args['candidate-diagnostics-image']}});
 const result=await runAb(order,async step=>(await runOnce(plan,{runId:step.runId,image:step.image,diagnosticsImage:step.diagnosticsImage,...roots})).summary,{threshold:args.threshold===undefined?DEFAULT_THRESHOLD:Number(args.threshold)});
 const out=join(resolve(roots.reportRoot),`${runId}-compare`);
 await writeFile(out+'.json',JSON.stringify(result,null,2)+'\n');await writeFile(out+'.md',markdownComparison(result));
 process.stdout.write(markdownComparison(result));
 if(result.regressions)process.exitCode=1;
}
if(process.argv[1]&&import.meta.url===pathToFileURL(resolve(process.argv[1])).href)main().catch(error=>{console.error(error.message);process.exitCode=1;});
