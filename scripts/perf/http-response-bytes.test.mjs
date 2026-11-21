import test from 'node:test';
import assert from 'node:assert/strict';
import {createServer} from 'node:http';
import {gzipSync} from 'node:zlib';
import {Measurements,measure,acceptedInScheduledWindow} from './http-load.mjs';
import {aggregateWorkers} from './http-multi-protocol.mjs';
test('successful decoded bytes use exact inclusive completion deadline',()=>{
 const m=new Measurements();for(const [status,ok,end,bytes]of [['200',true,99,3],['200',true,100,7],['200',true,101,11],['503',false,90,13],['200',false,90,17],['network',false,90,19]])m.add(1,status,bytes,ok,false,acceptedInScheduledWindow(status,ok,end,100));
 const r=m.report(1);assert.equal(r.bytes,70);assert.equal(r.accepted,3);assert.equal(r.acceptedDecodedBytes,21);assert.equal(r.acceptedDuringScheduledWindow,2);assert.equal(r.acceptedDuringScheduledWindowDecodedBytes,10);assert.equal(r.wireResponseBytes,null);
});
test('real measure streams exclude validator/partial failures and drain under controlled clock',async t=>{
 let now=0;t.mock.method(performance,'now',()=>now);
 for(const mode of ['boundary','drain','bad-status','bad-validation','partial']){
  now=0;let reads=0;t.mock.method(globalThis,'fetch',async()=>({status:mode==='bad-status'?503:200,body:{getReader:()=>({read:async()=>{if(reads++===0)return {done:false,value:Buffer.from('{}')};now=mode==='drain'?11:10;if(mode==='partial')throw Error('partial stream');return {done:true};}})}}));
  const r=await measure({base:'http://127.0.0.1',requests:[{name:'Email/get',path:'/',validate:()=>mode==='bad-validation'?undefined:true}],concurrency:1,seconds:.01});assert.equal(r.requests,1);assert.equal(r.bytes,2);assert.equal(r.acceptedDecodedBytes,['boundary','drain'].includes(mode)?2:0);assert.equal(r.acceptedDuringScheduledWindowDecodedBytes,mode==='boundary'?2:0);assert.equal(r.acceptedDuringScheduledWindow,mode==='boundary'?1:0);assert.equal(r.byRequest['Email/get'].acceptedDuringScheduledWindowDecodedBytes,r.acceptedDuringScheduledWindowDecodedBytes);
  t.mock.restoreAll();t.mock.method(performance,'now',()=>now);
 }
});
test('actual Fetch bytes are decompressed body not compressed Content-Length',async()=>{
 const raw=Buffer.alloc(2048,'a'),coded=gzipSync(raw);const server=createServer((_q,r)=>{r.writeHead(200,{'content-encoding':'gzip','content-length':coded.length});r.end(coded);});await new Promise(r=>server.listen(0,'127.0.0.1',r));
 try{const r=await measure({base:`http://127.0.0.1:${server.address().port}`,requests:[{name:'attachment-download',path:'/',validateBytes:b=>b.equals(raw)}],concurrency:1,seconds:.04});assert.ok(r.accepted>0);assert.equal(r.acceptedDecodedBytes,r.accepted*raw.length);assert.equal(r.acceptedDuringScheduledWindowDecodedBytes,r.acceptedDuringScheduledWindow*raw.length);assert.notEqual(raw.length,coded.length);assert.equal(r.wireResponseBytes,null);}finally{server.closeAllConnections();await new Promise(r=>server.close(r));}
});
function worker(index){
 const a=new Measurements(),s=new Measurements(),first=new Measurements(),second=new Measurements();first.add(1,'200',index?19:3,true,false,true);second.add(1,'200',index?23:7,true,false,false);a.merge(first.snapshot()).merge(second.snapshot());s.add(1,'scheduled',0,true);s.add(1,'scheduled',0,true);
 return {schema:1,workerIndex:index,partitionCount:1,measurement:{window:{requestedStartAtMs:1000,scheduledStartAtMs:1000,scheduledEndAtMs:2000,actualStartAtMs:1000,completedAtMs:2010,startLagMs:0},rawMeasurements:{schema:1,aggregate:a.snapshot(),byRequest:{'Email/get':first.snapshot(),'mark-read-and-restore':second.snapshot()},scheduling:s.snapshot(),arrivalLatency:a.snapshot()},concurrency:1,scheduledSeconds:1,acceptedDuringScheduledWindow:1,uniquePrincipalsAttempted:1,uniquePrincipalsAccepted:1,uniquePrincipalsAcceptedDuringScheduledWindow:1,uniqueClientsAttempted:1,uniqueClientsAccepted:1,uniqueClientsAcceptedDuringScheduledWindow:1,peakOutstanding:1,mutationStateTainted:false,unknownMutationOutcomes:0,taintedPrincipals:0,httpDispatchAttempts:2,blockedAfterMutationTaint:0,abortedUnscheduled:0,generator:{cpuUserMs:0,cpuSystemMs:0,rssBeforeBytes:0,rssAfterBytes:0,heapAfterBytes:0}}};
}
const merge=results=>aggregateWorkers({results,partitions:[{index:0,count:1},{index:1,count:1}],stage:{concurrency:2,seconds:1,timeoutMs:1000},startAtMs:1000});
test('worker finite shapes sum exactly and missing/inconsistent byte proof fails closed',()=>{
 const rows=[worker(0),worker(1)],r=merge(rows);assert.equal(r.acceptedDecodedBytes,52);assert.equal(r.acceptedDuringScheduledWindowDecodedBytes,22);assert.equal(r.byRequest['Email/get'].acceptedDuringScheduledWindowDecodedBytes,22);assert.equal(r.byRequest['mark-read-and-restore'].requests,2);assert.equal(r.byRequest['mark-read-and-restore'].acceptedDuringScheduledWindowDecodedBytes,0);
 for(const change of [r=>{delete r[0].measurement.rawMeasurements.aggregate.acceptedDecodedBytes;},r=>{r[0].measurement.rawMeasurements.byRequest['Email/get'].acceptedDuringScheduledWindowDecodedBytes=2;},r=>{r[0].measurement.acceptedDuringScheduledWindow=2;},r=>{r[0].measurement.rawMeasurements.byRequest['Email/get'].acceptedDuringScheduledWindowDecodedBytes=2; r[0].measurement.rawMeasurements.aggregate.acceptedDuringScheduledWindowDecodedBytes=2;}]){const copy=structuredClone(rows);change(copy);assert.throws(()=>merge(copy));}
});
test('overflow/impossible snapshots refuse without partial mutation',()=>{
 const m=new Measurements();m.add(1,'200',3,true,false,true);const before=m.snapshot();for(const patch of [{acceptedDecodedBytes:4},{acceptedDuringScheduledWindowDecodedBytes:4},{acceptedDuringScheduledWindowDecodedBytes:2},{acceptedDuringScheduledWindow:2},{acceptedDecodedBytes:undefined}]){assert.throws(()=>m.merge({...before,...patch}));assert.deepEqual(m.snapshot(),before);}m.bytes=Number.MAX_SAFE_INTEGER;assert.throws(()=>m.add(1,'200',1,true,false,true),/overflow/);assert.equal(m.accepted,1);
});
