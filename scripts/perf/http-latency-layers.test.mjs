import test from 'node:test';
import assert from 'node:assert/strict';
import {measure,Measurements,mergeLatencyBreakdowns,markdownReport} from './http-load.mjs';
import {aggregateWorkers} from './http-multi-protocol.mjs';
import {campaignOptions,runCampaign} from './http-campaign.mjs';
import {prepareFixtureRequests} from './fixture-requests.mjs';

test('real serialized sends separate queue, dispatch/body and nested validation under monotonic clock',async t=>{
 let now=0,fetches=0;t.mock.method(performance,'now',()=>now);
 t.mock.method(globalThis,'fetch',async()=>{
  fetches++;now+=5;let read=false;
  return {status:200,body:{getReader:()=>({read:async()=>{
   if(!read){read=true;return {done:false,value:Buffer.from('{}')};}now+=2;return {done:true};
  }})}};
 });
 const result=await measure({base:'http://127.0.0.1',requests:[{name:'Email/get',path:'/',serializePrincipal:true,principalIndex:0,validate:()=>{now+=3;return true;}}],concurrency:2,seconds:.01,includeRawMeasurements:true});
 assert.equal(fetches,2);assert.equal(result.requests,2);assert.equal(result.httpDispatchAttempts,2);
 assert.equal(result.latencyMs.mean,15,'old inclusive latency remains unchanged');
 assert.equal(result.latencyBreakdown.principalQueue.mean,5);
 assert.equal(result.latencyBreakdown.httpAndValidation.mean,10);
 assert.equal(result.latencyBreakdown.validation.mean,3);
 assert.equal(result.latencyBreakdown.principalQueue.samples,2);
 assert.equal(result.latencyBreakdown.undispatched,0);
 assert.deepEqual(result.byRequest['Email/get'].latencyBreakdown,result.latencyBreakdown);
 assert.equal(result.acceptedDuringScheduledWindow,1,'drained second completion is still excluded');
 assert.equal(result.acceptedDuringScheduledWindowDecodedBytes,2);
 assert.equal(mergeLatencyBreakdowns([result.rawMeasurements.latencyBreakdown.aggregate],[2],[2]).httpAndValidation.mean,10);
});

test('failed validation and body failure retain actual dispatch samples without inventing validation',async t=>{
 for(const kind of ['validator-throws','partial-body','unsent-configuration']){
  let now=0,read=0;t.mock.method(performance,'now',()=>now);
  t.mock.method(globalThis,'fetch',async()=>({status:200,body:{getReader:()=>({read:async()=>{
   now+=5;if(kind==='partial-body'){now=20;throw Error('partial');}if(read++===0)return {done:false,value:Buffer.from('{}')};return {done:true};
  }})}}));
  const request={name:'Email/get',path:'/',...(kind==='unsent-configuration'?{serializePrincipal:true}:{}),validate:()=>{now+=5;throw Error('validation');}};
  // Force only the failed unsent attempt to cross the outer loop deadline.
  if(kind==='unsent-configuration')Object.defineProperty(request,'principalIndex',{get(){now=20;return undefined;}});
  const r=await measure({base:'http://127.0.0.1',requests:[request],concurrency:1,seconds:.01,includeRawMeasurements:true});
  assert.equal(r.requests,1);assert.equal(r.accepted,0);assert.equal(r.errors,1);
  assert.equal(r.latencyBreakdown.httpAndValidation.samples,kind==='unsent-configuration'?0:1);
  assert.equal(r.latencyBreakdown.validation.samples,kind==='validator-throws'?1:0);
  assert.equal(r.latencyBreakdown.undispatched,kind==='unsent-configuration'?1:0);
  t.mock.restoreAll();
 }
});

function phase(ms,count=1){const m=new Measurements();for(let n=0;n<count;n++)m.add(ms,'scheduled',0,true);return m.snapshot();}
function breakdown(ms){return {principalQueue:phase(ms),httpAndValidation:phase(10),validation:phase(2)};}
test('weighted layer merge distinguishes missing legacy proof and rejects incoherent samples',()=>{
 const r=mergeLatencyBreakdowns([breakdown(1),breakdown(9)],[1,1],[1,1]);assert.equal(r.principalQueue.mean,5);assert.equal(r.validation.samples,2);
 assert.equal(mergeLatencyBreakdowns([undefined,undefined],[1,1],[1,1]),null);
 for(const bad of [undefined,{...breakdown(1),principalQueue:phase(1,2)},{...breakdown(1),httpAndValidation:phase(1)},{...breakdown(1),extra:phase(1)}])assert.throws(()=>mergeLatencyBreakdowns([breakdown(1),bad],[1,1],[1,1]));
 assert.throws(()=>mergeLatencyBreakdowns([breakdown(1)],[1],[0]));
});

test('multiworker timing is weighted, finite per route and rejects missing/misassigned proof',()=>{
 const sample=index=>{
  const m=new Measurements();m.add(20,'200',2,true,false,true);
  return {schema:1,workerIndex:index,partitionCount:1,measurement:{window:{requestedStartAtMs:1000,scheduledStartAtMs:1000,scheduledEndAtMs:2000,actualStartAtMs:1000,completedAtMs:2001,startLagMs:0},rawMeasurements:{schema:1,aggregate:m.snapshot(),byRequest:{'Email/get':m.snapshot()},scheduling:phase(0),arrivalLatency:m.snapshot(),latencyBreakdown:{schema:1,aggregate:breakdown(index?9:1),byRequest:{'Email/get':breakdown(index?9:1)}}},concurrency:1,scheduledSeconds:1,acceptedDuringScheduledWindow:1,uniquePrincipalsAttempted:1,uniquePrincipalsAccepted:1,uniquePrincipalsAcceptedDuringScheduledWindow:1,uniqueClientsAttempted:1,uniqueClientsAccepted:1,uniqueClientsAcceptedDuringScheduledWindow:1,peakOutstanding:1,mutationStateTainted:false,unknownMutationOutcomes:0,taintedPrincipals:0,httpDispatchAttempts:1,blockedAfterMutationTaint:0,abortedUnscheduled:0,generator:{cpuUserMs:0,cpuSystemMs:0,rssBeforeBytes:0,rssAfterBytes:0,heapAfterBytes:0}}};
 };
 const merge=results=>aggregateWorkers({results,partitions:[{index:0,count:1},{index:1,count:1}],stage:{concurrency:2,seconds:1,timeoutMs:1000},startAtMs:1000});
 const rows=[sample(0),sample(1)],r=merge(rows);assert.equal(r.latencyBreakdown.principalQueue.mean,5);assert.equal(r.byRequest['Email/get'].latencyBreakdown.httpAndValidation.samples,2);
 for(const mutate of [x=>{delete x[0].measurement.rawMeasurements.latencyBreakdown;},x=>{delete x[0].measurement.rawMeasurements.latencyBreakdown.aggregate;},x=>{x[0].measurement.rawMeasurements.latencyBreakdown.byRequest['Email/get'].principalQueue=phase(900);},x=>{x[0].measurement.httpDispatchAttempts=0;}]){const copy=structuredClone(rows);mutate(copy);assert.throws(()=>merge(copy));}
 for(const row of rows)delete row.measurement.rawMeasurements.latencyBreakdown;
 assert.equal(merge(rows).latencyBreakdown,null,'historical receipts are unreported, never fabricated zero phase times');
});

test('rendered report distinguishes observed intervals from missing historical data',()=>{
 const stage={requests:1,accepted:1,requestsPerSecond:1,acceptedRequestsPerSecond:1,latencyMs:{p50:20,p95:20,p99:20},errors:0,throttled:0,timeouts:0};
 const text=markdownReport({stages:[stage,{...stage,latencyBreakdown:mergeLatencyBreakdowns([breakdown(1)],[1],[1])}]});
 assert.match(text,/not measured/);assert.match(text,/client wall time, not isolated Rust CPU or DB time/);assert.match(text,/1.00 \/ 1/);assert.match(text,/10.00 \/ 10/);
});

test('isolated area groups reuse the exact representative templates, mutations and validators',async()=>{
 const principal={accountId:'owned',messageIds:['m1','m2'],inboxId:'inbox',archiveId:'archive',attachmentBlobId:'m1.2',cookie:'mw_session=owned; mw_csrf=owned',csrfToken:'owned',clientAddress:'198.18.0.1'};
 const {requests,groups}=prepareFixtureRequests({principals:[principal],engineMode:'native',representative:true,clientSimulation:{mode:'fixture-loopback-trusted-proxy',clientCount:1}});
 const expected={'paged-headers':5,reader:6,attachment:7,'flag-write':8,'move-write':9};
 const endpoints=['identity','mailboxes','query','headers','refresh',...Object.keys(expected)];
 const options=campaignOptions({MW_LOAD_FIXTURE_ENDPOINTS:endpoints.join(','),MW_LOAD_FIXTURE_SECONDS:'.1',MW_LOAD_FIXTURE_WARMUP_SECONDS:'0',MW_LOAD_FIXTURE_CONCURRENCY:'1',MW_LOAD_FIXTURE_RATES:'1'});
 for(const [name,index]of Object.entries(expected)){assert.equal(groups[name][0],requests[index]);assert.equal(groups[name].length,1);assert.equal(groups[name][0].serializePrincipal,true);}
 assert.equal(groups['flag-write'][0].statefulMutation,true);assert.equal(groups['move-write'][0].statefulMutation,true);assert.equal(requests.length,10);assert.equal(groups.representative.length,10);
 const seen=[];const r=await runCampaign({groups,options,run:async spec=>{seen.push(spec.requests);return {requests:1,errors:0,throttled:0,timeouts:0,statuses:{200:1},latencyMs:{p99:1}};}});
 assert.deepEqual(r.stages.map(stage=>stage.endpoint),endpoints);assert.deepEqual(seen,endpoints.map(endpoint=>groups[endpoint]));
 assert.throws(()=>campaignOptions({MW_LOAD_FIXTURE_ENDPOINTS:'reader,unknown'}));
});
