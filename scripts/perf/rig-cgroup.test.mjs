import test from 'node:test';
import assert from 'node:assert/strict';
import {parseCpuStat,parseProcStat,parseSamples,valueAt,deltaOver,stageWindow,stageCpuCost} from './rig-cgroup.mjs';
import {allocate} from './rig-cpuset.mjs';

test('cgroup v2 cpu.stat parses; missing throttle keys are zero; malformed input refused',()=>{
 const stat=parseCpuStat('usage_usec 1000\nuser_usec 600\nsystem_usec 400\nnr_periods 10\nnr_throttled 2\nthrottled_usec 50\nnr_bursts 0\nburst_usec 0\n');
 assert.deepEqual([stat.usage_usec,stat.user_usec,stat.system_usec,stat.nr_periods,stat.nr_throttled,stat.throttled_usec],[1000,600,400,10,2,50]);
 assert.deepEqual(parseCpuStat('usage_usec 5\nuser_usec 3\nsystem_usec 2\n').nr_throttled,0);
 for(const bad of ['','usage_usec x','user_usec 1\nsystem_usec 1','usage_usec -1\nuser_usec 1\nsystem_usec 1'])assert.throws(()=>parseCpuStat(bad));
});

test('/proc/stat gives busy and total jiffies per CPU',()=>{
 const p=parseProcStat('cpu  10 0 10 100 0 0 0 0 0 0\ncpu0 1 2 3 100 4 5 6 7 0 0\ncpu1 0 0 0 50 0 0 0 0 0 0\nintr 1\n');
 assert.deepEqual(p.cpus,{0:[24,128],1:[0,50]});
 assert.throws(()=>parseProcStat('intr 1\n'));
});

const sample=(tMs,usage,extra={})=>({schema:1,tMs,containers:{app:{id:'a',cpu:{usage_usec:usage,user_usec:usage*.75,system_usec:usage*.25,nr_periods:0,nr_throttled:0,throttled_usec:0}},generator:{id:'g',cpu:{usage_usec:usage/2,user_usec:usage/2,system_usec:0,nr_periods:0,nr_throttled:0,throttled_usec:0}}},others:{},procStat:null,...extra});

test('counters are interpolated at window edges; gaps, missing edges and resets give null',()=>{
 const samples=[sample(0,0),sample(1000,1_000_000),sample(2000,3_000_000)];
 const read=s=>s.containers.app.cpu.usage_usec;
 assert.equal(valueAt(samples,read,500,{maxGapMs:1000}),500_000);
 assert.equal(valueAt(samples,read,1000,{maxGapMs:1000}),1_000_000);
 assert.equal(valueAt(samples,read,2500,{maxGapMs:1000}),null);
 assert.equal(valueAt(samples,read,500,{maxGapMs:999}),null);
 assert.equal(deltaOver(samples,read,500,1500,{maxGapMs:1000}),1_500_000);
 assert.equal(valueAt([sample(0,5),sample(1000,1)],read,500,{maxGapMs:2000}),null,'counter went backwards');
 const {samples:parsed,malformed}=parseSamples(JSON.stringify(samples[1])+'\nnot json\n'+JSON.stringify({schema:2})+'\n'+JSON.stringify(samples[0])+'\n');
 assert.equal(malformed,2);assert.deepEqual(parsed.map(s=>s.tMs),[0,1000]);
});

test('stage cost: µs CPU per request per container and rps per logical CPU',()=>{
 const plan=allocate({app:4,generator:2,rig:1},{vcpus:26});
 // App uses 2 cores for 10 s (20 s CPU), generator 1 core; 10,000 requests accepted in the window.
 const samples=Array.from({length:21},(_,i)=>sample(1000+i*500,i*1_000_000));
 const stage={coordination:{scheduledStartAtMs:1000,scheduledEndAtMs:11000},acceptedDuringScheduledWindow:10000,acceptedDuringScheduledWindowRps:1000,scheduledSeconds:10};
 const cost=stageCpuCost(stage,samples,{plan,labels:{app:'app',generator:'generator'}});
 assert.equal(cost.available,true);
 assert.equal(cost.containers.app.cpuUsec,20_000_000);assert.equal(cost.containers.app.coresUsed,2);
 assert.equal(cost.containers.app.usecCpuPerRequest,2000);assert.equal(cost.containers.app.utilisationOfAllotted,0.5);
 assert.equal(cost.containers.app.userUsec,15_000_000);
 assert.equal(cost.containers.generator.usecCpuPerRequest,1000);
 assert.equal(cost.server.usecCpuPerRequest,2000);assert.equal(cost.server.allottedCpus,4);
 assert.equal(cost.rpsPerLogicalCpu.allottedServer,250);assert.equal(cost.rpsPerLogicalCpu.consumedServer,500);
 // A window the samples do not cover is unavailable, not zero.
 const late=stageCpuCost({...stage,coordination:{scheduledStartAtMs:50000,scheduledEndAtMs:60000}},samples,{plan,labels:{app:'app'}});
 assert.equal(late.available,false);assert.equal(late.server.usecCpuPerRequest,null);
 assert.equal(stageCpuCost({},samples,{plan,labels:{app:'app'}}).available,false);
 assert.deepEqual(stageWindow({window:{scheduledStartAtMs:1,scheduledEndAtMs:2}}),{startMs:1,endMs:2});
});
