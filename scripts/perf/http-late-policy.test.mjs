import test from 'node:test';
import assert from 'node:assert/strict';
import {measure,Measurements} from './http-load.mjs';
import {aggregateWorkers} from './http-multi-protocol.mjs';
import {campaignOptions} from './http-campaign.mjs';
import {fixture} from './http-load-fixture.mjs';

const requests=[{name:'health',path:'/healthz'}];
async function blocked(policy){
 const host=await fixture();
 try{
  const run=measure({base:host.base,concurrency:50,seconds:1,arrivalRate:1000,requests,latePolicy:policy});
  const until=performance.now()+260;while(performance.now()<until){}
  return await run;
 }finally{await host.close();}
}
test('drop policy discards slots over 100 ms late, dispatch policy dispatches every slot in the window',async()=>{
 const drop=await blocked('drop'),dispatch=await blocked('dispatch');
 assert.equal(drop.latePolicy,'drop');
 assert.equal(dispatch.latePolicy,'dispatch');assert.equal(dispatch.droppedCapacity,0);
 // Only slots still unissued when the window closes (timer overshoot at the very end) can be missing; no late slot is discarded.
 assert.equal(dispatch.droppedLateSlots,0);assert.ok(drop.droppedLateSlots>100,`late drops ${drop.droppedLateSlots}`);
 assert.equal(dispatch.unissuedAtWindowEnd,dispatch.droppedScheduler);assert.equal(drop.droppedLateSlots+drop.unissuedAtWindowEnd,drop.droppedScheduler);assert.equal(dispatch.requests+dispatch.droppedScheduler,1000);assert.equal(dispatch.offered,1000);assert.ok(dispatch.lateSlotsDispatched>0);
 assert.equal(dispatch.latencyBasis,'scheduled-slot');assert.equal(drop.latencyBasis,'send');
 // The queue the drop policy hides shows up in latency from the slot and in the lag distribution.
 assert.ok(dispatch.latencyMs.max>=100,`max ${dispatch.latencyMs.max}`);assert.ok(dispatch.schedulingLagMs.max>=100);
 assert.equal(dispatch.generatorLimited,true);
});
test('dispatch policy holds a due slot at the outstanding cap instead of dropping it',async()=>{
 const host=await fixture({delayMs:30});
 try{
  const result=await measure({base:host.base,concurrency:2,seconds:.3,arrivalRate:200,timeoutMs:2000,requests:[{name:'slow',path:'/read',headers:{cookie:'session=fixture-only'}}],latePolicy:'dispatch'});
  assert.equal(result.droppedCapacity,0);assert.ok(result.peakOutstanding<=2);
  assert.equal(result.requests+result.droppedScheduler,60);
 }finally{await host.close();}
});
test('dispatch policy requires an arrival rate and a known policy name',async()=>{
 await assert.rejects(measure({base:'http://127.0.0.1:1',requests,concurrency:1,seconds:1,latePolicy:'dispatch'}),/Late policy/);
 await assert.rejects(measure({base:'http://127.0.0.1:1',requests,concurrency:1,seconds:1,arrivalRate:10,latePolicy:'wait'}),/Late policy/);
 assert.equal(campaignOptions({MW_LOAD_FIXTURE_LATE:'dispatch'}).latePolicy,'dispatch');
 assert.equal(campaignOptions({}).latePolicy,undefined);
 assert.throws(()=>campaignOptions({MW_LOAD_FIXTURE_LATE:'x'}),/LATE/);
});
test('an unloaded generator is not marked generator-limited and exposes per-second accepted series',async()=>{
 const host=await fixture();
 try{
  const result=await measure({base:host.base,concurrency:20,seconds:2,arrivalRate:200,requests,commonStartAtMs:Date.now()+50,includeRawMeasurements:true});
  assert.ok(Array.isArray(result.acceptedPerSecond)&&result.acceptedPerSecond.length===2);
  assert.equal(result.acceptedPerSecond.reduce((a,b)=>a+b,0),result.acceptedDuringScheduledWindow);
  assert.ok(result.acceptedPerSecond.every(n=>n>=150&&n<=250),JSON.stringify(result.acceptedPerSecond));
  assert.equal(result.schedulingLagLimitMs,1);
 }finally{await host.close();}
});

const startAtMs=100000,stage={concurrency:4,seconds:2,timeoutMs:1000,arrivalRate:100};
function worker(index,lagMs,perSecond=[25,25]){
 const aggregate=new Measurements(),scheduling=new Measurements();
 for(let n=0;n<50;n++){aggregate.add(1,'200',4,true,false,true);scheduling.add(lagMs,'scheduled',0,true);}
 return {schema:1,workerIndex:index,partitionCount:2,measurement:{window:{requestedStartAtMs:startAtMs,scheduledStartAtMs:startAtMs,scheduledEndAtMs:startAtMs+2000,actualStartAtMs:startAtMs+1,completedAtMs:startAtMs+2010,startLagMs:1},rawMeasurements:{schema:1,aggregate:aggregate.snapshot(),byRequest:{'Email/query':aggregate.snapshot()},scheduling:scheduling.snapshot(),arrivalLatency:aggregate.snapshot()},acceptedPerSecond:perSecond,concurrency:2,scheduledSeconds:2,arrivalRate:50,offered:100,droppedCapacity:0,droppedScheduler:50,acceptedDuringScheduledWindow:50,uniquePrincipalsAttempted:2,uniquePrincipalsAccepted:2,uniquePrincipalsAcceptedDuringScheduledWindow:2,uniqueClientsAttempted:2,uniqueClientsAccepted:2,uniqueClientsAcceptedDuringScheduledWindow:2,peakOutstanding:2,mutationStateTainted:false,unknownMutationOutcomes:0,taintedPrincipals:0,httpDispatchAttempts:50,blockedAfterMutationTaint:0,abortedUnscheduled:0,generator:{cpuUserMs:1,cpuSystemMs:1,rssBeforeBytes:1,rssAfterBytes:1,heapAfterBytes:1}}};
}
const parts=[{index:0,count:2},{index:1,count:2}];
test('multi-worker merge keeps per-worker scheduling lag, flags a lagging worker and sums per-second series',()=>{
 const merged=aggregateWorkers({results:[worker(1,5,[10,40]),worker(0,0,[30,20])],partitions:parts,stage:{...stage,latePolicy:'dispatch'},startAtMs});
 assert.deepEqual(merged.acceptedPerSecond,[40,60]);assert.equal(merged.acceptedPerSecondMin,40);assert.equal(merged.acceptedPerSecondMax,60);
 assert.equal(merged.latePolicy,'dispatch');
 assert.deepEqual(merged.perWorker.map(row=>[row.workerIndex,row.generatorLimited]),[[0,false],[1,true]]);
 assert.equal(merged.perWorker[1].schedulingLagMs.p99,5);assert.ok('p50' in merged.perWorker[1].schedulingLagMs&&'max' in merged.perWorker[1].schedulingLagMs);
 assert.equal(merged.generatorLimited,true);
 const clean=aggregateWorkers({results:[worker(1,0),worker(0,1)],partitions:parts,stage,startAtMs});
 assert.equal(clean.generatorLimited,false);assert.equal(clean.latePolicy,'drop');
});
test('a series that disagrees with the window count or has the wrong length is rejected',()=>{
 assert.throws(()=>aggregateWorkers({results:[worker(0,0,[25,26]),worker(1,0)],partitions:parts,stage,startAtMs}),/series/);
 assert.throws(()=>aggregateWorkers({results:[worker(0,0,[50]),worker(1,0)],partitions:parts,stage,startAtMs}),/series/);
});

test('the Markdown report names generator-limited stages and stays unchanged otherwise',async()=>{
 const {markdownReport}=await import('./http-load.mjs');
 const stage={phase:'offered-rate',endpoint:'mixed',concurrency:4,arrivalRate:40000,requests:1,accepted:1,requestsPerSecond:1,acceptedRequestsPerSecond:1,latencyMs:{p50:1,p95:1,p99:1},errors:0,throttled:0,timeouts:0};
 const base={target:'t',scenario:'s',runtime:'r',model:'m',stages:[stage]};
 assert.ok(!markdownReport(base).includes('Generator-limited'));
 assert.match(markdownReport({...base,stages:[{...stage,generatorLimited:true}]}),/Generator-limited stages .*offered-rate \/ mixed \/ 40000/);
});
