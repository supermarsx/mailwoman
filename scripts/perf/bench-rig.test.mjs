import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtemp,mkdir,writeFile,readFile,readdir,rm} from 'node:fs/promises';
import {join} from 'node:path';
import os from 'node:os';
import {execFile} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {parseArgs,buildPlan,legCommands,runLeg,analyseLeg,writeSummary} from './bench-rig.mjs';

const ID=c=>c.repeat(64);

test('arguments are strict and the plan pins every role to disjoint CPUs recorded as MW_LOAD_HOST_*',()=>{
 assert.throws(()=>parseArgs(['go']));assert.throws(()=>parseArgs(['plan','--nope=1']));assert.throws(()=>parseArgs(['plan','x']));
 const plan=buildPlan(parseArgs(['plan','--run=r1','--vcpus=26']).args);
 assert.deepEqual(Object.fromEntries(Object.entries(plan.cpus.roles).map(([r,v])=>[r,v.cpuset])),{app:'1-4',generator:'5-8',rig:'9'});
 assert.deepEqual(plan.host,{MW_LOAD_HOST_DOCKER_VCPUS:'26',MW_LOAD_HOST_CPUSET_APP:'1-4',MW_LOAD_HOST_CPUSET_GENERATOR:'5-8',MW_LOAD_HOST_CPUSET_RIG:'9'});
 assert.equal(plan.generatorWorkers,4);assert.deepEqual(plan.legs.map(l=>l.id),['release-shapes','release-mix','diagnostics-shapes']);
 const pg=buildPlan(parseArgs(['plan','--run=r1','--backend=postgres','--app=8','--database=4','--generator=8','--legs=release-mix']).args);
 assert.equal(pg.cpus.roles.database.cpuset,'9-12');assert.equal(pg.host.MW_LOAD_HOST_CPUSET_DB,'9-12');assert.equal(pg.legs.length,1);
 assert.equal(pg.legs[0].env.MW_LOAD_FIXTURE_WORKERS,'8');
 const r0a=buildPlan(parseArgs(['plan','--run=r0a','--transport-concurrency=2,8']).args);
 assert.equal(r0a.generatorWorkers,2,'workers never exceed the smallest concurrency');
 const explicit=buildPlan(parseArgs(['plan','--run=r0a','--cpuset-app=2-5','--cpuset-generator=10-13','--cpuset-rig=20']).args);
 assert.equal(explicit.cpus.outsideCpuset,'0-1,6-9,14-19,21-25');
 assert.throws(()=>buildPlan(parseArgs(['plan','--run=r1','--legs=nope']).args));
 assert.throws(()=>buildPlan(parseArgs(['plan','--run=r1','--app=30']).args),/Not enough VM vCPUs/);
 assert.throws(()=>buildPlan(parseArgs(['plan','--run=r0a','--cpuset-app=1-4','--cpuset-generator=4-6','--cpuset-rig=9']).args),/both app and generator/);
});

test('docker commands: cpusets on every container, no network for SQLite, an internal network for PostgreSQL, secrets only in env files',()=>{
 const plan=buildPlan(parseArgs(['plan','--run=r0a']).args);
 const c=legCommands(plan,plan.legs[0],{prefix:'mw-rig-x-transport',image:'img',envFile:'/p/app.env',pgEnvFile:'/p/pg.env'});
 const flag=(args,name)=>args[args.indexOf(name)+1];
 assert.equal(flag(c.app,'--cpuset-cpus'),'1-4');assert.equal(flag(c.app,'--network'),'none');
 assert.equal(flag(c.generator,'--cpuset-cpus'),'5-8');assert.equal(flag(c.generator,'--network'),'container:mw-rig-x-transport-app');
 assert.ok(c.generator.includes('MW_LOAD_HOST_CPUSET_APP=1-4'));assert.ok(c.generator.includes('--workers=4'));
 const sampler=c.sampler({app:ID('a'),generator:ID('b')});
 assert.equal(flag(sampler,'--cpuset-cpus'),'9');assert.ok(sampler.includes(`--label=app=${ID('a')}`));
 assert.ok(sampler.includes('type=bind,source=/sys/fs/cgroup,target=/host-cgroup,readonly'));
 assert.equal(c.database,null);assert.equal(c.network,null);
 assert.ok(!JSON.stringify(c.app).includes('MW_DB_KEY'));
 const pg=buildPlan(parseArgs(['plan','--run=r1','--backend=postgres','--legs=release-mix']).args);
 const p=legCommands(pg,pg.legs[0],{prefix:'mw-rig-y-release-mix',image:'img',envFile:'/p/a',pgEnvFile:'/p/b'});
 assert.deepEqual(p.network.slice(0,3),['network','create','--internal']);
 assert.equal(flag(p.database,'--network-alias'),'mw-load-pg');assert.equal(flag(p.database,'--cpuset-cpus'),'5-8');
 assert.equal(flag(p.app,'--network'),'mw-rig-y-release-mix-net');
 assert.deepEqual(p.cleanup.at(-1),['network','rm','mw-rig-y-release-mix-net']);
});

function fakeDocker(log,{reportStages,samples}){
 return async args=>{
  log.push(args);
  const [verb]=args;
  if(verb==='image')return 'sha256:feed';
  if(verb==='run'&&args[1]==='-d'){const name=args[args.indexOf('--name')+1];return name.endsWith('-app')?ID('a'):name.endsWith('-generator')?ID('b'):name.endsWith('-pg')?ID('d'):ID('c');}
  if(verb==='exec')return String(Date.now());
  if(verb==='ps')return `${ID('a')} mw-rig-t1-transport-app\n${ID('e')} mw-shots-greenmail-1`;
  if(verb==='wait')return '0';
  if(verb==='cp'){
   const dest=args[2];
   if(args[1].endsWith('/scratch/reports/.')){await mkdir(dest,{recursive:true});await writeFile(join(dest,'native-100.json'),JSON.stringify({kind:'coordinated-native-http-fixture',stages:reportStages}));}
   else await writeFile(dest,samples);
   return '';
  }
  if(verb==='logs')return 'log line';
  return '';
 };
}

test('one leg end to end with a recorded Docker: order, cleanup, private files removed, evidence written, then analysed',async()=>{
 const root=await mkdtemp(join(os.tmpdir(),'bench-rig-'));
 try{
  const plan=buildPlan(parseArgs(['plan','--run=r0a','--vcpus=12']).args);
  const stage={endpoint:'transport-healthz',concurrency:8,phase:'concurrency-ramp',requests:2000,acceptedDuringScheduledWindow:2000,acceptedDuringScheduledWindowRps:400,scheduledSeconds:5,errors:0,throttled:0,timeouts:0,latencyMs:{p50:1,p99:2},
   coordination:{scheduledStartAtMs:1000,scheduledEndAtMs:6000},diagnostics:{before:{pool:{operations:{read:{calls:0},write:{calls:0},other:{calls:0},commits:{sqlite:{calls:0},postgres:{calls:0}},autocommitWrites:0}}},after:{pool:{operations:{read:{calls:2000},write:{calls:0},other:{calls:0},commits:{sqlite:{calls:0},postgres:{calls:0}},autocommitWrites:0}}}}};
  const lines=Array.from({length:40},(_,i)=>JSON.stringify({schema:1,tMs:i*250,containers:{app:{cpu:{usage_usec:i*250_000,user_usec:0,system_usec:0}},generator:{cpu:{usage_usec:i*100_000,user_usec:0,system_usec:0}}},others:{[ID('e').slice(0,12)]:{usage_usec:i*1000}},procStat:{cpus:Object.fromEntries(Array.from({length:12},(_,c)=>[c,[0,i*25]]))}})).join('\n')+'\n';
  const log=[];
  const legDir=await runLeg(plan,plan.legs[0],{runId:'t1',image:'img',reportDir:join(root,'out'),privateDir:join(root,'private'),docker:fakeDocker(log,{reportStages:[stage],samples:lines})});
  const verbs=log.map(a=>a[0]+(a[0]==='run'?(a[1]==='-d'?' -d '+a[a.indexOf('--name')+1].split('-').at(-1):' setup'):a[0]==='volume'||a[0]==='rm'?' '+a[1]:''));
  assert.deepEqual(verbs.slice(0,8),['image','volume create','volume create','run setup','run -d app','run -d generator','run -d sampler','exec']);
  assert.ok(verbs.indexOf('wait')<verbs.indexOf('cp'));
  assert.deepEqual(log.at(-2).slice(0,2),['rm','-f']);assert.deepEqual(log.at(-1).slice(0,2),['volume','rm']);
  assert.deepEqual(await readdir(join(root,'private')),[],'the private env file is removed');
  const infra=JSON.parse(await readFile(join(legDir,'infrastructure.json'),'utf8'));
  assert.equal(infra.cpusets.app,'1-4');assert.equal(infra.generatorExit,0);assert.equal(infra.imageId,'sha256:feed');
  assert.ok(!JSON.stringify(infra).includes('MW_DB_KEY'));
  const result=await analyseLeg(plan,plan.legs[0],legDir);
  assert.equal(result.rows.length,1);
  const row=result.rows[0];
  assert.equal(row.kind,'transport');assert.match(row.label,/transport control only/);
  assert.equal(row.usecCpuPerRequest.server,2500,'4 s of app CPU over 5 s / 2,000 requests ... per request');
  assert.equal(row.database.perRequest.reads,1);assert.equal(row.contaminated,false);
  assert.equal(row.quiet.vm.otherContainers.top[0].name,'mw-shots-greenmail-1');
  const summary=await writeSummary(plan,root,[result]);
  assert.equal(summary.rows.length,1);
  assert.match(await readFile(join(root,'summary.md'),'utf8'),/\| transport \| transport-healthz \| transport \|/);
  await assert.rejects(runLeg(plan,plan.legs[0],{runId:'t1',image:'img',reportDir:join(root,'out'),privateDir:join(root,'private'),docker:fakeDocker([],{reportStages:[],samples:''})}),/Existing evidence/);
 }finally{await rm(root,{recursive:true,force:true});}
});

test('a failing leg still removes its containers and volumes',async()=>{
 const root=await mkdtemp(join(os.tmpdir(),'bench-rig-'));
 try{
  const plan=buildPlan(parseArgs(['plan','--run=r0a']).args),log=[];
  const docker=async args=>{log.push(args);if(args[0]==='run'&&args.includes('-d')&&args[args.indexOf('--name')+1].endsWith('-generator'))throw new Error('docker run failed');return args[0]==='run'?ID('a'):'';};
  await assert.rejects(runLeg(plan,plan.legs[0],{runId:'t2',image:'img',reportDir:root,privateDir:join(root,'p'),docker}),/docker run failed/);
  assert.ok(log.some(a=>a[0]==='rm'&&a.includes('mw-rig-t2-transport-app')));assert.ok(log.some(a=>a[0]==='volume'&&a[1]==='rm'));
 }finally{await rm(root,{recursive:true,force:true});}
});

test('the command line refuses a run without the bench slot confirmation and prints a plan without Docker',async()=>{
 const script=fileURLToPath(new URL('./bench-rig.mjs',import.meta.url));
 const exec=args=>new Promise(resolve=>execFile(process.execPath,[script,...args],(error,stdout,stderr)=>resolve({code:error?.code??0,stdout,stderr})));
 const refused=await exec(['run','--run=r0a','--run-id=x','--image=i','--report-root=r','--private-root=p']);
 assert.equal(refused.code,1);assert.match(refused.stderr,/--confirm-bench-slot/);
 const plan=await exec(['plan','--run=r0a']);
 assert.equal(plan.code,0);assert.equal(JSON.parse(plan.stdout).run,'r0a');
});

test('either generator: node workers by default, or the native mw-loadgen mounted read-only with one thread per generator CPU',()=>{
 const node=buildPlan(parseArgs(['plan','--run=r1']).args);
 assert.equal(node.generator.implementation,'node');
 const native=buildPlan(parseArgs(['plan','--run=r1','--generator=6','--generator-impl=native','--native-generator-bin=/opt/lg/mw-loadgen']).args);
 assert.equal(native.generatorWorkers,1);assert.equal(native.generator.threads,6);
 const c=legCommands(native,native.legs[0],{prefix:'p',image:'img',envFile:'e',pgEnvFile:'g'});
 assert.ok(c.generator.includes('MW_LOAD_GENERATOR=native'));assert.ok(c.generator.includes('MW_NATIVE_GENERATOR_BIN=/rig-bin/mw-loadgen'));
 assert.ok(c.generator.includes('MW_NATIVE_GENERATOR_THREADS=6'));assert.ok(c.generator.includes('--workers=1'));
 assert.ok(c.generator.some(arg=>/^type=bind,source=.*lg,target=\/rig-bin,readonly$/.test(arg)));
 assert.ok(!legCommands(node,node.legs[0],{prefix:'p',image:'img',envFile:'e',pgEnvFile:'g'}).generator.includes('MW_LOAD_GENERATOR=native'));
 assert.throws(()=>buildPlan(parseArgs(['plan','--run=r0a','--generator-impl=native','--native-generator-bin=/x']).args),/transport group/);
 assert.throws(()=>buildPlan(parseArgs(['plan','--run=r1','--generator-impl=native']).args),/native-generator-bin/);
 assert.throws(()=>buildPlan(parseArgs(['plan','--run=r1','--generator-impl=go']).args));
});

test('the default cpusets fit the 26-vCPU Docker VM for every run and backend (decision D3)',()=>{
 for(const args of [['--run=r0a'],['--run=r1'],['--run=r1','--backend=postgres']]){
  const plan=buildPlan(parseArgs(['plan',...args]).args);
  assert.equal(plan.cpus.vcpus,26);assert.ok(plan.cpus.usedCpus.at(-1)<26);
  assert.ok(plan.cpus.roles.app.cpus<=12&&plan.cpus.roles.generator.cpus<=8&&(plan.cpus.roles.database?.cpus??4)<=4);
 }
 // The largest split allowed for M0-M3 fits: app 12, database 4, generator 8, rig 1 = CPUs 1-25.
 assert.equal(buildPlan(parseArgs(['plan','--run=r1','--backend=postgres','--app=12','--database=4','--generator=8']).args).cpus.roles.rig.cpuset,'25');
});
