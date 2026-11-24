import test from 'node:test';
import assert from 'node:assert/strict';
import {EventEmitter} from 'node:events';
import {Measurements} from './http-load.mjs';
import {clusterAuthority,clusterOptions,clusterRequest,calibrationStageBudget,aggregateCalibrationWorkers,clusterCalibrationReport,calibrationChannel} from './http-cluster-calibration-protocol.mjs';
import {aggregateServerMetrics} from './http-cluster-calibration-server.mjs';
import {serverMetricsReport} from './http-cluster-calibration-client.mjs';
import {JSON_BYTES} from './http-calibration-protocol.mjs';
const authority={schema:1,kind:'synthetic-cluster-transport-calibration',token:'c'.repeat(64),controlBase:'http://127.0.0.1:11000',dataBases:['http://127.0.0.1:11001','http://127.0.0.1:11002'],serverWorkers:2,expiresAtMs:10000};

test('cluster authority and phase options remain private literal-loopback and bounded',()=>{
 assert.equal(clusterAuthority(authority,0).serverWorkers,2);
 for(const patch of [{controlBase:'http://remote.invalid'},{dataBases:['http://localhost:11001','http://127.0.0.1:11002']},{dataBases:['http://127.0.0.1:11000','http://127.0.0.1:11002']},{serverWorkers:17},{token:'invalid'},{expiresAtMs:0},{expiresAtMs:1000000}])assert.throws(()=>clusterAuthority({...authority,...patch},0));
 const base={handoff:'private.json',output:'report.json'};assert.equal(clusterOptions(base).workers,4);
 for(const patch of [{workers:65},{workers:0},{seconds:.5},{seconds:3601},{rates:'100001'},{rates:'100,100'},{concurrency:20001},{workers:8,concurrency:7},{target:'http://external.invalid'}])assert.throws(()=>clusterOptions({...base,...patch}));
 const split=calibrationStageBudget({workers:3,concurrency:7},100);assert.deepEqual(split,{concurrency:[3,2,2],rates:[34,33,33]});
});

test('synthetic requests distribute across data workers and preserve exact payload validators',()=>{
 const spec=clusterRequest('synthetic-json-4k',authority.dataBases,0);
 assert.equal(spec.materialize(0).path,'http://127.0.0.1:11001/json');assert.equal(spec.materialize(1).path,'http://127.0.0.1:11002/json');
 assert.equal(spec.validate(JSON.parse(JSON_BYTES)),true);assert.equal(spec.validate({}),false);
 assert.throws(()=>clusterRequest('arbitrary',authority.dataBases,0));assert.throws(()=>clusterRequest('synthetic-json-4k',['http://external.invalid'],0));
});

function samples(){return [0,1].map(index=>{
 const aggregate=new Measurements(),scheduling=new Measurements();for(let n=0;n<50;n++){aggregate.add(index===0?1:7,'200',JSON_BYTES.length,true);scheduling.add(0,'scheduled',0,true);}
 const raw=aggregate.snapshot();return {schema:1,index,measurement:{window:{requestedStartAtMs:10000,scheduledStartAtMs:10000,scheduledEndAtMs:11000,actualStartAtMs:10003,completedAtMs:11001,startLagMs:3},rawMeasurements:{schema:1,aggregate:raw,byRequest:{'synthetic-json-4k':structuredClone(raw)},scheduling:scheduling.snapshot(),arrivalLatency:structuredClone(raw)},mutationStateTainted:false,concurrency:2,arrivalRate:50,offered:50,scheduledSeconds:1,droppedCapacity:0,droppedScheduler:0,acceptedDuringScheduledWindow:50,peakOutstanding:2,generator:{cpuUserMs:1,cpuSystemMs:2,rssBeforeBytes:100,rssAfterBytes:110,heapAfterBytes:10}}};
});}
const phase={workers:2,seconds:1,concurrency:4,rate:100,endpoint:'synthetic-json-4k',startAtMs:10000};
test('coordinated synthetic aggregation merges raw buckets and exact budgets, never worker percentiles',()=>{
 const result=aggregateCalibrationWorkers(samples(),phase);
 assert.equal(result.requests,100);assert.equal(result.offered,100);assert.equal(result.acceptedDuringScheduledWindowRps,100);assert.equal(result.latencyMs.p95,7);assert.equal(result.latencyMs.mean,4);assert.equal(result.transportWindowSustained,true);assert.equal(result.peakOutstandingUpperBound,4);
 assert.throws(()=>aggregateCalibrationWorkers(samples(),{...phase,observedAtMs:10000}),/Inconsistent calibration counters/);
 for(const mutate of [rows=>{rows.pop();},rows=>{rows[1].index=0;},rows=>{rows[1].measurement.offered=51;},rows=>{rows[1].measurement.window.startLagMs=500;},rows=>{rows[1].measurement.window.scheduledEndAtMs=12000;},rows=>{rows[1].measurement.rawMeasurements.aggregate.histogram=[[1,1]];}]){const rows=samples();mutate(rows);assert.throws(()=>aggregateCalibrationWorkers(rows,phase));}
});

test('server reset snapshots sum processes and reports never expose authority or application proof',()=>{
 const perWorker=[0,1].map(index=>({index,requests:50,bytes:50*JSON_BYTES.length,elapsedSeconds:1,cpuUserMs:1,cpuSystemMs:2,rssBytes:100,eventLoopDelayMs:{mean:1,p99:2,max:3,secret:authority.token},secret:authority.token}));
 const metrics=aggregateServerMetrics(perWorker,{cpuUserMs:1,cpuSystemMs:1,elapsedSeconds:1,rssBytes:10,secret:authority.token});
 assert.equal(metrics.requests,100);assert.equal(metrics.cpuUserMs,2);
 const sanitized=serverMetricsReport(metrics,2);assert.ok(!JSON.stringify(sanitized).includes(authority.token));
 assert.deepEqual(sanitized.perWorker[0].eventLoopDelayMs,{mean:1,p99:2,max:3});
 const report=clusterCalibrationReport({workers:2,seconds:1,concurrency:4,rates:[100]},[{...aggregateCalibrationWorkers(samples(),phase),server:sanitized}]);
 assert.equal(report.applicationCapacityMeasured,false);assert.equal(report.mailwomanCapacityClaim,false);assert.equal(report.securityProof,false);assert.ok(!JSON.stringify(report).includes(authority.token));
 assert.throws(()=>serverMetricsReport({...metrics,requests:101},2));assert.throws(()=>aggregateServerMetrics([perWorker[0],perWorker[0]],{}));
});

test('private calibration IPC fails closed on child failure and confirms shutdown',async()=>{
 const child=new EventEmitter();child.connected=true;child.send=()=>{};child.kill=()=>queueMicrotask(()=>child.emit('exit',0));
 const channel=calibrationChannel(child);child.emit('message',{type:'ready'});assert.equal((await channel.wait('ready',undefined)).type,'ready');
 const pending=channel.wait('result',1);child.emit('message',{type:'failed',reason:authority.token});await assert.rejects(pending,error=>!error.message.includes(authority.token));assert.throws(()=>channel.send({type:'start'}));await channel.stop();
});
