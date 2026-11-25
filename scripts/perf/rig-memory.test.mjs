import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtemp,mkdir,writeFile,rm} from 'node:fs/promises';
import {join} from 'node:path';
import os from 'node:os';
import {containerMemory,serverSizes,stageMemory} from './rig-memory.mjs';
import {readMemory,parseKeyValues} from './rig-sampler.mjs';
import {compare,metricsOfRow} from './rig-compare.mjs';

const MiB=1048576;
const sample=(tMs,current,{oom=0,max=0,pids=40}={})=>({schema:1,tMs,containers:{app:{cpu:{usage_usec:0,user_usec:0,system_usec:0},mem:{current,peak:900*MiB,pids,events:{low:0,high:0,max,oom,oom_kill:0},stat:{anon:current*0.8,file:current*0.2,kernel:1,sock:2,shmem:3}}},
 pg:{cpu:{usage_usec:0,user_usec:0,system_usec:0},mem:{current:200*MiB,pids:31,events:{oom:0,max:0,high:0,oom_kill:0},stat:{}}}},others:{},procStat:null});

test('the sampler reads cgroup memory files and leaves out missing controllers',async()=>{
 const root=await mkdtemp(join(os.tmpdir(),'rig-mem-'));
 try{
  await writeFile(join(root,'memory.current'),'1048576\n');await writeFile(join(root,'memory.peak'),'2097152\n');await writeFile(join(root,'pids.current'),'17\n');
  await writeFile(join(root,'memory.events'),'low 0\nhigh 2\nmax 1\noom 0\noom_kill 0\noom_group_kill 0\n');
  await writeFile(join(root,'memory.stat'),'anon 700\nfile 300\nkernel 10\nsock 5\nshmem 1\nactive_anon 9\n');
  assert.deepEqual(await readMemory(root),{current:1048576,peak:2097152,pids:17,events:{low:0,high:2,max:1,oom:0,oom_kill:0},stat:{anon:700,file:300,kernel:10,sock:5,shmem:1}});
  const empty=join(root,'none');await mkdir(empty);
  assert.equal(await readMemory(empty),null);
  assert.deepEqual(parseKeyValues('a 1\r\nb x\nc 3',['a','c']),{a:1,c:3});
 }finally{await rm(root,{recursive:true,force:true});}
});

test('per container: memory at both edges, the largest sampled value inside, event increases, threads',()=>{
 const samples=[sample(0,500*MiB),sample(1000,600*MiB),sample(2000,800*MiB,{max:2}),sample(3000,650*MiB,{max:2,oom:1}),sample(4000,640*MiB,{max:2,oom:1})];
 const m=containerMemory(samples,'app',{startMs:1000,endMs:3500});
 assert.equal(m.currentStartBytes,600*MiB);assert.equal(m.currentEndBytes,650*MiB);assert.equal(m.maxSampledBytes,800*MiB);
 assert.deepEqual(m.events,{high:0,max:2,oom:1,oom_kill:0});assert.equal(m.pidsEnd,40);assert.equal(m.peakBytes,900*MiB);
 assert.equal(containerMemory(samples,'missing',{startMs:1000,endMs:3500}).available,false);
});

test('server sizes from the fixture diagnostics and bytes per connected user; PostgreSQL process count',()=>{
 const after={processMemory:{residentBytes:700*MiB,anonymousBytes:600*MiB,fileBackedBytes:100*MiB,sharedMemoryBytes:0,peakResidentBytes:720*MiB,threads:44},
  pool:{open:8,idle:7,maximum:8,statementCache:{cachedPerIdleConnection:[100,99]},sqlitePageCache:{plannedForegroundBytes:null,perConnectionKiB:null,idleSample:{sampledConnections:7}}},
  preparedMail:{chargedBytes:5},t34Modes:{requestProtection:{pending:3}},sessionTouchBuffer:{held:1000,pending:12,recorded:5},searchIndex:{writerConfiguration:{memoryBudgetBytes:50000000}},resources:{processResidentBytes:1}};
 const stage={uniquePrincipalsAcceptedDuringScheduledWindow:1000,diagnostics:{before:{processMemory:{residentBytes:650*MiB}},after}};
 const sizes=serverSizes(stage);
 assert.equal(sizes.process.end.anonymousBytes,600*MiB);assert.equal(sizes.process.start.residentBytes,650*MiB);assert.equal(sizes.process.end.threads,44);
 assert.deepEqual(sizes.pool.cachedStatementsPerIdleConnection,[100,99]);assert.deepEqual(sizes.sessionTouchBuffer,{held:1000,pending:12});assert.equal(sizes.writeBehindPending,3);
 assert.ok(sizes.notExposed.length>0);
 const samples=[sample(0,500*MiB),sample(5000,650*MiB)];
 const mem=stageMemory(stage,samples,{labels:{app:'app',pg:'database'},window:{startMs:1000,endMs:4000}});
 assert.equal(mem.derived.appBytesPerConnectedUser,Math.round(500*MiB/1000));
 assert.equal(mem.postgres.processes,31);assert.equal(mem.postgres.memoryCurrentEndBytes,200*MiB);
 assert.equal(serverSizes({}).available,false);
});

test('compare: app memory at the stage end regresses beyond 10% in both repeats; any OOM event regresses',()=>{
 const row=(bytes,oom=0)=>({shape:'changes',instrumented:false,errors:0,throttled:0,timeouts:0,http503:0,acceptedRps:100,latencyMs:{p50:3,p99:9},usecCpuPerRequest:{server:900},rpsPerLogicalCpu:{consumedServer:100},contaminated:false,
  database:{available:true,source:'process',commitsAvailable:true,perRequest:{reads:5,writes:0,other:1,databaseCommits:0,upstreamCommands:0}},
  memory:{containers:{app:{role:'app',available:true,currentEndBytes:bytes,events:{oom,oom_kill:0}}}}});
 const ref={sqlite:{changes:metricsOfRow(row(1000*MiB))}};
 const v=(r,name)=>r.rows[0].checks.find(c=>c.metric===name).verdict;
 assert.equal(v(compare([ref,ref],[{sqlite:{changes:metricsOfRow(row(1090*MiB))}},{sqlite:{changes:metricsOfRow(row(1095*MiB))}}]),'appMemoryEndBytes'),'same','9% is within the 10% memory threshold');
 assert.equal(v(compare([ref,ref],[{sqlite:{changes:metricsOfRow(row(1120*MiB))}},{sqlite:{changes:metricsOfRow(row(1150*MiB))}}]),'appMemoryEndBytes'),'regression');
 assert.equal(v(compare([ref],[{sqlite:{changes:metricsOfRow(row(1000*MiB,1))}}]),'oomEvents'),'regression');
});
