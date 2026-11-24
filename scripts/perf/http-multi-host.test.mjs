import test from 'node:test';
import assert from 'node:assert/strict';
import {fork} from 'node:child_process';
import {resolve} from 'node:path';
import {estimateClockOffset,startHostAgent,remoteHost} from './http-multi-host.mjs';
import {runMultiCampaign,validateAssignment,shiftWindow} from './http-multi-controller.mjs';
import {Measurements} from './http-load.mjs';
import {aggregateWorkers} from './http-multi-protocol.mjs';
import {handoffFor} from './test-fixtures/multi-handoff.mjs';

const TOKEN='host-agent-token-0123456789';
const stub=(_script,args,options)=>fork(resolve('scripts/perf/test-fixtures/http-multi-protocol-child.mjs'),args,options);
const options={mode:'smoke',concurrency:[4],rates:[100],seconds:1,timeoutMs:1000,endpoints:['query'],warmupSeconds:0,deadlineSeconds:30,maxErrorRate:0,maxThrottleRate:0,maxP99Ms:100,allowOverload:false,adaptive:true};
const isolationProbe=async()=>({verified:true,status:'verified',requests:2,directions:2});

test('clock offset comes from the lowest round-trip sample and states its uncertainty',()=>{
 const est=estimateClockOffset([
  {sentAtMs:1000,hostNowMs:6050,receivedAtMs:1100},
  {sentAtMs:2000,hostNowMs:7004,receivedAtMs:2008},
  {sentAtMs:3000,hostNowMs:8200,receivedAtMs:3300}]);
 assert.deepEqual(est,{offsetMs:5000,rttMs:8,uncertaintyMs:4,samples:3});
 assert.throws(()=>estimateClockOffset([]));
 assert.throws(()=>estimateClockOffset([{sentAtMs:5,hostNowMs:1,receivedAtMs:4}]));
});

test('windows from another host move onto the controller clock without touching the lag fields',()=>{
 const row={measurement:{window:{requestedStartAtMs:15000,scheduledStartAtMs:15000,scheduledEndAtMs:16000,actualStartAtMs:15001,completedAtMs:16010,startLagMs:1}}};
 const shifted=shiftWindow(row,5000).measurement.window;
 assert.deepEqual([shifted.requestedStartAtMs,shifted.scheduledEndAtMs,shifted.actualStartAtMs,shifted.completedAtMs,shifted.startLagMs],[10000,11000,10001,11010,1]);
 assert.equal(row.measurement.window.requestedStartAtMs,15000);
 assert.equal(shiftWindow(row,0),row);
});

test('the agent refuses public bind addresses, short tokens and unauthenticated calls',async()=>{
 await assert.rejects(startHostAgent({host:'0.0.0.0',token:TOKEN}),/private rig address/);
 await assert.rejects(startHostAgent({host:'8.8.8.8',token:TOKEN}),/private rig address/);
 await assert.rejects(startHostAgent({host:'127.0.0.1',token:'short'}),/token/);
 const agent=await startHostAgent({host:'127.0.0.1',token:TOKEN,launch:stub});
 try{
  assert.equal((await fetch(agent.origin+'/clock')).status,401);
  assert.equal((await fetch(agent.origin+'/clock',{headers:{'x-host-token':'x'.repeat(TOKEN.length)}})).status,401);
  assert.equal((await fetch(agent.origin+'/clock',{headers:{'x-host-token':TOKEN}})).status,200);
  assert.equal((await fetch(agent.origin+'/send',{method:'POST',headers:{'x-host-token':TOKEN},body:'{"index":0}'})).status,404);
 }finally{await agent.close();}
 assert.throws(()=>remoteHost({origin:'http://8.8.8.8:8080',token:TOKEN}),/private|loopback/);
});

test('workers on two more hosts join one coordinated stage; offsets are recorded and histograms merged',async()=>{
 const skewMs=3000;
 const a=await startHostAgent({host:'127.0.0.1',token:TOKEN,launch:stub,now:()=>Date.now()+skewMs});
 const b=await startHostAgent({host:'127.0.0.1',token:TOKEN,launch:stub});
 try{
  const report=await runMultiCampaign({handoff:handoffFor(4),workers:1,hosts:[{origin:a.origin,token:TOKEN,workers:1},{origin:b.origin,token:TOKEN,workers:2}],options,launch:stub,startLeadMs:500,scannerProbe:async()=>403,isolationProbe});
  const stage=report.stages[0];
  assert.equal(stage.coordination.valid,true);
  assert.equal(stage.coordination.workers,4);
  assert.deepEqual(stage.coordination.partitionSizes,[1,1,1,1]);
  assert.equal(stage.offered,100);
  assert.equal(stage.acceptedDuringScheduledWindow,100);
  assert.equal(stage.uniquePrincipalsAccepted,4);
  const [hostA,hostB]=stage.coordination.hosts;
  assert.ok(Math.abs(hostA.offsetMs-skewMs)<=hostA.uncertaintyMs+2,'offset '+hostA.offsetMs);
  assert.ok(Math.abs(hostB.offsetMs)<=hostB.uncertaintyMs+2);
  assert.deepEqual([hostA.workerIndexes,hostB.workerIndexes],[[1],[2,3]]);
  assert.ok(stage.coordination.clockToleranceMs>=1);
  assert.equal(report.distinctPhysicalGeneratorHostsMeasured,true);
  assert.equal(report.generatorHosts.length,3);
  assert.ok(!JSON.stringify(report).includes(TOKEN)&&!JSON.stringify(report).includes('private-'));
 }finally{await a.close();await b.close();}
});

test('host and worker counts are bounded and a wrong host token stops the run before traffic',async()=>{
 assert.throws(()=>validateAssignment(handoffFor(4),1,[],[65]),/Each additional host/);
 assert.throws(()=>validateAssignment(handoffFor(4),64,[],[64,64,64,64,1]),/workers per host/);
 assert.equal(validateAssignment(handoffFor(4),0,[],[2,2]).totalWorkers,4);
 assert.throws(()=>validateAssignment(handoffFor(4),0,[],[]),/workers per host/);
 const agent=await startHostAgent({host:'127.0.0.1',token:TOKEN,launch:stub});
 try{
  await assert.rejects(runMultiCampaign({handoff:handoffFor(4),workers:1,hosts:[{origin:agent.origin,token:'y'.repeat(TOKEN.length),workers:1}],options,launch:stub,startLeadMs:500,isolationProbe}),/Host agent refused/);
 }finally{await agent.close();}
});

test('windows from a host with a different clock are rejected unless shifted by the measured offset',()=>{
 const startAtMs=100000;
 const make=(index,shift)=>{
  const aggregate=new Measurements(),scheduling=new Measurements();
  for(let n=0;n<10;n++){aggregate.add(1,'200',4,true,false,true);scheduling.add(0,'scheduled',0,true);}
  return {schema:1,workerIndex:index,partitionCount:1,measurement:{window:{requestedStartAtMs:startAtMs+shift,scheduledStartAtMs:startAtMs+shift,scheduledEndAtMs:startAtMs+shift+1000,actualStartAtMs:startAtMs+shift,completedAtMs:startAtMs+shift+1005,startLagMs:0},rawMeasurements:{schema:1,aggregate:aggregate.snapshot(),byRequest:{'Email/query':aggregate.snapshot()},scheduling:scheduling.snapshot(),arrivalLatency:aggregate.snapshot()},concurrency:1,scheduledSeconds:1,arrivalRate:10,offered:10,droppedCapacity:0,droppedScheduler:0,acceptedDuringScheduledWindow:10,uniquePrincipalsAttempted:1,uniquePrincipalsAccepted:1,uniquePrincipalsAcceptedDuringScheduledWindow:1,uniqueClientsAttempted:1,uniqueClientsAccepted:1,uniqueClientsAcceptedDuringScheduledWindow:1,peakOutstanding:1,mutationStateTainted:false,unknownMutationOutcomes:0,taintedPrincipals:0,httpDispatchAttempts:10,blockedAfterMutationTaint:0,abortedUnscheduled:0,generator:{cpuUserMs:1,cpuSystemMs:1,rssBeforeBytes:1,rssAfterBytes:1,heapAfterBytes:1}}};
 };
 const input={partitions:[{index:0,count:1},{index:1,count:1}],stage:{concurrency:2,seconds:1,timeoutMs:1000,arrivalRate:20},startAtMs};
 assert.throws(()=>aggregateWorkers({...input,results:[make(0,0),make(1,5000)]}),/skewed/);
 assert.equal(aggregateWorkers({...input,results:[make(0,0),shiftWindow(make(1,5000),5000)]}).requests,20);
 assert.throws(()=>aggregateWorkers({...input,clockToleranceMs:3,results:[make(0,0),make(1,-10)]}),/skewed/);
});
