import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtemp,mkdir,writeFile,rm} from 'node:fs/promises';
import {join} from 'node:path';
import os from 'node:os';
import {parseArgs,buildPlan,m3Arms,m3Table,m3Order,M3_ARMS,analyseLeg} from './bench-rig.mjs';
import {runDefinition} from './rig-shapes.mjs';
import {campaignOptions} from './http-campaign.mjs';

test('M3 run: idle 60 s; identity, flag and the mix at C32 and C64 for 120 s; idle 300 s; open/close/reopen named as not built',()=>{
 const m3=runDefinition('m3');
 assert.equal(m3.legs.length,1);
 const env=m3.legs[0].env;
 assert.equal(env.MW_LOAD_FIXTURE_IDLE_BEFORE_SECONDS,'60');assert.equal(env.MW_LOAD_FIXTURE_IDLE_AFTER_SECONDS,'300');
 const options=campaignOptions(env);
 assert.deepEqual(options.endpoints,['goal-shape-authenticated-identity','goal-shape-flag-and-restore','goal-65-35']);assert.deepEqual(options.concurrency,[32,64]);assert.equal(options.seconds,120);
 assert.ok(env.MW_LOAD_FIXTURE_ENDPOINTS.length<=256);
 assert.ok(m3.notBuilt[0].includes('open / close / reopen'));
 assert.equal(runDefinition('m3',{smoke:true}).legs[0].env.MW_LOAD_FIXTURE_IDLE_AFTER_SECONDS,'5');
 assert.equal(buildPlan(parseArgs(['plan','--run=m3']).args).run,'m3');
});

test('M3 order: arms alternate direction per repeat (A B C, C B A), each run twice by default',()=>{
 const arms=[{id:'glibc'},{id:'glibc-arena'},{id:'musl'}];
 assert.deepEqual(m3Order(arms).map(a=>a.id+':'+a.repeat),['glibc:1','glibc-arena:1','musl:1','musl:2','glibc-arena:2','glibc:2']);
 assert.equal(m3Order(arms,3).length,9);assert.equal(m3Order(arms,3)[6].id,'glibc');
 assert.throws(()=>m3Order(arms,0));assert.throws(()=>m3Order(arms,7));
});

test('M3 arms: five defined; an arm runs only with its image; MALLOC_ARENA_MAX goes to the glibc-arena arm only',()=>{
 assert.deepEqual(M3_ARMS.map(a=>a.id),['glibc','glibc-arena','musl','jemalloc','mimalloc']);
 const {run,skipped}=m3Arms({'arm-image-glibc':'fixture:abc','arm-image-musl':'fixture-static:abc','arena-max':'4'});
 assert.deepEqual(run.map(a=>[a.id,a.image]),[['glibc','fixture:abc'],['glibc-arena','fixture:abc'],['musl','fixture-static:abc']]);
 assert.deepEqual(run[1].env,{MALLOC_ARENA_MAX:'4'});assert.deepEqual(run[0].env,{});assert.deepEqual(run[2].env,{});
 assert.deepEqual(skipped.map(s=>s.id),['jemalloc','mimalloc']);
 assert.equal(m3Arms({'arm-image-glibc':'g'}).run[1].env.MALLOC_ARENA_MAX,'2');
 assert.throws(()=>m3Arms({}),/at least one/);assert.throws(()=>m3Arms({'arm-image-glibc':'g','arena-max':'0'}),/arena-max/);
});

test('M3 table: anonymous and total memory at every boundary, +60 s inside the idle after, per arm repeat and stage',()=>{
 const MiB=1048576;
 const mem=(anon,current)=>({anonEndBytes:anon*MiB,currentEndBytes:current*MiB});
 const summary=(libc)=>({imageRuntime:{libc,allocator:libc==='glibc'?'glibc malloc':'musl'},
  idleWindows:[{phase:'before',app:mem(300,400)},{phase:'after',app:mem(320,650),at60:{anonBytes:340*MiB,currentBytes:660*MiB}}],
  rows:[{shape:'goal-65-35',concurrency:32,kind:'mix',usecCpuPerRequest:{server:30000},latencyMs:{p99:2400},contaminated:false,memory:{containers:{app:{role:'app',available:true,...mem(500,700),anonMaxSampledBytes:550*MiB,maxSampledBytes:750*MiB}}}},
   {shape:'authenticated-identity',concurrency:64,kind:'read',usecCpuPerRequest:{server:2000},latencyMs:{p99:50},contaminated:true,memory:{containers:{}}}]});
 const table=m3Table({'glibc r1':summary('glibc'),'musl r1':summary('musl')},[{id:'jemalloc',reason:'no --arm-image-jemalloc'}]);
 assert.ok(table.includes('anonymous / total'));
 assert.ok(table.includes('| glibc r1 | glibc / glibc malloc | 300.0 / 400.0 | goal-65-35 C32 | 500.0 / 700.0 (550.0 / 750.0) | 30000 | 2400 | 340.0 / 660.0 | 320.0 / 650.0 | quiet |'),table);
 assert.ok(table.includes('| authenticated-identity C64 | – / – (– / –) | 2000 | 50 |')&&table.includes('| CONTAMINATED |'),table);
 assert.ok(table.includes('| jemalloc | – | not run: no --arm-image-jemalloc'));assert.ok(table.includes('Not built: 1,000 users open'));
});

test('idle windows written by the fixture are read back with app memory at their edges',async()=>{
 const root=await mkdtemp(join(os.tmpdir(),'rig-m3-'));
 try{
  const plan=buildPlan(parseArgs(['plan','--run=m3','--vcpus=12']).args),leg=plan.legs[0],dir=join(root,'m3');
  await mkdir(join(dir,'reports'),{recursive:true});
  await writeFile(join(dir,'infrastructure.json'),JSON.stringify({containerIds:{},prefix:'p'}));
  await writeFile(join(dir,'reports','idle-windows.jsonl'),JSON.stringify({phase:'before',startMs:1000,endMs:3000,seconds:2})+'\n');
  await writeFile(join(dir,'reports','native-1000.json'),JSON.stringify({kind:'coordinated-native-http-fixture',stages:[]}));
  await writeFile(join(dir,'samples.jsonl'),[0,1000,2000,3000,4000].map(t=>JSON.stringify({schema:1,tMs:t,containers:{app:{cpu:{usage_usec:0,user_usec:0,system_usec:0},mem:{current:t*1000}}},others:{},procStat:null})).join('\n')+'\n');
  const result=await analyseLeg(plan,leg,dir);
  assert.equal(result.idleWindows.length,1);assert.equal(result.idleWindows[0].at60,null,'a window shorter than 60 s has no +60 s reading');assert.equal(result.idleWindows[0].app.currentStartBytes,1_000_000);assert.equal(result.idleWindows[0].app.currentEndBytes,3_000_000);
 }finally{await rm(root,{recursive:true,force:true});}
});
