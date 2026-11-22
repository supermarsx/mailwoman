import test from 'node:test';
import assert from 'node:assert/strict';
import {classifyHttp503,mergeHttp503Reasons,measure,Measurements} from './http-load.mjs';
import {aggregateWorkers} from './http-multi-protocol.mjs';

const codes=['publishedGuardFenceRefusal','authorizationNotConfirmedBeforeDispatch','requestProtectionUnavailable','empty503','other503'];
function proof(code='other503',count=1){return {schema:1,total:count,counts:Object.fromEntries(codes.map(key=>[key,key===code?count:0]))};}

test('only exact fixed codes from complete bounded JSON enter exported reason buckets',()=>{
 for(const code of codes.slice(0,3))assert.equal(classifyHttp503(Buffer.from(JSON.stringify({code,error:'private-body',accountId:'private-identity'})),true),code);
 assert.equal(classifyHttp503(Buffer.alloc(0),true),'empty503');
 for(const body of ['private-body',' ',JSON.stringify({code:'private-token'}),JSON.stringify({code:['publishedGuardFenceRefusal']}),'null','[]','{"code":"publishedGuardFenceRefusal"','x'.repeat(4097)])assert.equal(classifyHttp503(Buffer.from(body),true),'other503');
 for(const body of [Buffer.alloc(0),Buffer.from('{"code":"publishedGuardFenceRefusal"}'),null])assert.equal(classifyHttp503(body,false),'other503');
 assert.equal(classifyHttp503(null,true),'other503');
});

test('actual response reader counts one reason per503 without exporting body, query or headers',async t=>{
 for(const kind of ['known','fragmented','empty','unknown','partial','oversized','not503']){
  let now=0,read=0;t.mock.method(performance,'now',()=>now);
  const body=kind==='empty'?Buffer.alloc(0):Buffer.from(kind==='oversized'?'x'.repeat(4097):JSON.stringify({code:kind==='unknown'?'private-token':'publishedGuardFenceRefusal',error:'private-body',accountId:'private-identity'}));
  const fragments=kind==='fragmented'?[body.subarray(0,5),body.subarray(5,11),body.subarray(11)]:[body];
  t.mock.method(globalThis,'fetch',async()=>({status:kind==='not503'?500:503,body:{getReader:()=>({read:async()=>{
   if(read<fragments.length)return {done:false,value:fragments[read++]};now=20;if(kind==='partial')throw Error('private-stream-error');return {done:true};
  }})}}));
  const r=await measure({base:'http://127.0.0.1',headers:{cookie:'private-cookie'},requests:[{name:'Email/get',path:'/?private-query',validate:()=>true}],concurrency:1,seconds:.01,includeRawMeasurements:true});
  assert.equal(r.requests,1);assert.equal(r.errors,1);assert.equal(r.accepted,0);
  const code=['known','fragmented'].includes(kind)?'publishedGuardFenceRefusal':kind==='empty'?'empty503':'other503';
  assert.equal(r.http503Reasons.total,kind==='not503'?0:1);
  assert.equal(r.http503Reasons.counts[code],kind==='not503'?0:1);
  assert.equal(r.bytes,body.length,'bounded diagnostics do not alter actual decoded-byte accounting');
  assert.deepEqual(r.http503Reasons,r.byRequest['Email/get'].http503Reasons);
  assert.deepEqual(r.http503Reasons,r.rawMeasurements.http503Reasons.aggregate);
  for(const secret of ['private-token','private-body','private-identity','private-cookie','private-query','private-stream-error'])assert.ok(!JSON.stringify(r).includes(secret));
  t.mock.restoreAll();
 }
});

test('weighted reason merge binds fixed schema and all counts to actual503 statuses',()=>{
 const r=mergeHttp503Reasons([proof('empty503',2),proof('publishedGuardFenceRefusal',3)],[2,3]);
 assert.equal(r.total,5);assert.equal(r.counts.empty503,2);assert.equal(r.counts.publishedGuardFenceRefusal,3);
 assert.equal(mergeHttp503Reasons([undefined,undefined],[3,4]),null);
 for(const value of [undefined,null,{...proof(),total:2},{...proof(),private:'secret'},{...proof(),counts:{...proof().counts,other503:-1}},{...proof(),counts:{...proof().counts,private:1}},{...proof(),counts:{...proof().counts,other503:0}},{...proof(),counts:{...proof().counts,other503:1.5}}])assert.throws(()=>mergeHttp503Reasons([proof(),value],[1,1]));
 assert.throws(()=>mergeHttp503Reasons([proof()],[0]));
});

function worker(index){
 const m=new Measurements(),s=new Measurements();m.add(10,'503',10,false);s.add(0,'scheduled',0,true);
 return {schema:1,workerIndex:index,partitionCount:1,measurement:{window:{requestedStartAtMs:1000,scheduledStartAtMs:1000,scheduledEndAtMs:2000,actualStartAtMs:1000,completedAtMs:2001,startLagMs:0},rawMeasurements:{schema:1,aggregate:m.snapshot(),byRequest:{'Email/get':m.snapshot()},scheduling:s.snapshot(),arrivalLatency:m.snapshot(),http503Reasons:{schema:1,aggregate:proof(index?'empty503':'publishedGuardFenceRefusal'),byRequest:{'Email/get':proof(index?'empty503':'publishedGuardFenceRefusal')}}},concurrency:1,scheduledSeconds:1,acceptedDuringScheduledWindow:0,uniquePrincipalsAttempted:1,uniquePrincipalsAccepted:0,uniquePrincipalsAcceptedDuringScheduledWindow:0,uniqueClientsAttempted:1,uniqueClientsAccepted:0,uniqueClientsAcceptedDuringScheduledWindow:0,peakOutstanding:1,mutationStateTainted:false,unknownMutationOutcomes:0,taintedPrincipals:0,httpDispatchAttempts:1,blockedAfterMutationTaint:0,abortedUnscheduled:0,generator:{cpuUserMs:0,cpuSystemMs:0,rssBeforeBytes:0,rssAfterBytes:0,heapAfterBytes:0}}};
}
test('actual worker aggregator preserves fixed routes and rejects partial/misassigned reason proofs',()=>{
 const merge=results=>aggregateWorkers({results,partitions:[{index:0,count:1},{index:1,count:1}],stage:{concurrency:2,seconds:1,timeoutMs:1000},startAtMs:1000});
 const rows=[worker(0),worker(1)],r=merge(rows);assert.equal(r.http503Reasons.total,2);assert.equal(r.http503Reasons.counts.empty503,1);assert.deepEqual(r.http503Reasons,r.byRequest['Email/get'].http503Reasons);
 for(const mutate of [x=>{delete x[0].measurement.rawMeasurements.http503Reasons;},x=>{delete x[0].measurement.rawMeasurements.http503Reasons.aggregate;},x=>{x[0].measurement.rawMeasurements.http503Reasons.byRequest['Email/get']=proof('other503');},x=>{x[0].measurement.rawMeasurements.aggregate.statuses={'200':1};}]){const copy=structuredClone(rows);mutate(copy);assert.throws(()=>merge(copy));}
 for(const row of rows)delete row.measurement.rawMeasurements.http503Reasons;
 assert.equal(merge(rows).http503Reasons,null,'historical absent evidence is unreported, not a manufactured zero503 tally');
});
