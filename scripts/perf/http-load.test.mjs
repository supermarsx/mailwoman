import test from 'node:test';
import assert from 'node:assert/strict';
import {createServer} from 'node:http';
import {Measurements,measure,mergeMeasurements} from './http-load.mjs';
import {fixture} from './http-load-fixture.mjs';
import {execFile} from 'node:child_process';
import {promisify} from 'node:util';

test('CLI requires explicit origin and matching remote staging acknowledgement',async()=>{
 const execute=promisify(execFile);
 for(const args of [[],['--base=https://example.invalid'],['--base=https://example.invalid','--allow-remote=true'],['--base=https://user:secret@example.invalid']]){
  await assert.rejects(execute(process.execPath,['scripts/perf/http-load.mjs',...args],{timeout:5000}),error=>{
   assert.ok(!error.stderr.includes('user:secret'));assert.ok(!error.stderr.includes('secret@example'));
   assert.match(error.stderr,/explicit|staging|plain HTTP/);return true;
  });
 }
});

test('latency percentiles retain failures, timeouts and exact status counts',()=>{
 const m=new Measurements();for(let n=1;n<=100;n++)m.add(n,'200',5,n!==100,n===100);
 const result=m.report(2);assert.equal(result.latencyMs.p50,50);assert.equal(result.latencyMs.p95,95);assert.equal(result.latencyMs.p99,99);assert.equal(result.requestsPerSecond,50);assert.equal(result.errors,1);assert.equal(result.timeouts,1);assert.equal(result.bytes,500);
});

test('raw measurement merging weights counts and sums before reconstructing percentiles',()=>{
 const fast=new Measurements(),slow=new Measurements();for(let n=0;n<100;n++)fast.add(1,'200',1,true);slow.add(9000,'503',7,false,true);
 const result=mergeMeasurements([fast.snapshot(),slow.snapshot()],2);
 assert.equal(result.requests,101);assert.equal(result.accepted,100);assert.equal(result.errors,1);assert.equal(result.timeouts,1);assert.equal(result.bytes,107);
 assert.equal(result.latencyMs.mean,9100/101);assert.equal(result.latencyMs.p99,1);assert.equal(result.latencyMs.max,9000);assert.equal(result.acceptedRequestsPerSecond,50);
 assert.deepEqual(result.statuses,{'200':100,'503':1});
 const zero=new Measurements();zero.add(0,'scheduled',0,true);assert.equal(new Measurements().merge(zero.snapshot()).report(1).latencyMs.p99,0);
 const before=fast.snapshot();for(const patch of [{count:101},{accepted:101},{timeouts:101},{totalMs:NaN},{histogram:[[1,100],[1,1]]},{statuses:{private_secret:100}}])assert.throws(()=>fast.merge({...before,...patch}),/Invalid bounded raw measurements/);
 assert.deepEqual(fast.snapshot(),before,'invalid snapshots must not partially mutate totals');
});

test('coordinated HTTP phases share a future deadline and expose credential-free raw histograms',async()=>{
 const arrivals=[];const server=createServer((_req,res)=>{arrivals.push(Date.now());res.end('{}');});await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
 try{
  const commonStartAtMs=Date.now()+80,base=`http://127.0.0.1:${server.address().port}`;
  const results=await Promise.all([0,1].map(index=>measure({base,requests:[{name:'read',path:'/',principalIndex:index,headers:{cookie:`private-token-${index}`}}],concurrency:1,seconds:.15,arrivalRate:100,commonStartAtMs,includeRawMeasurements:true})));
  assert.ok(arrivals.length>0&&arrivals.every(at=>at>=commonStartAtMs));
  for(const result of results){
   assert.equal(result.window.scheduledStartAtMs,commonStartAtMs);assert.equal(result.window.scheduledEndAtMs,commonStartAtMs+150);assert.ok(result.window.actualStartAtMs>=commonStartAtMs);assert.ok(result.window.startLagMs>=0);
   assert.equal(result.rawMeasurements.schema,1);assert.equal(result.rawMeasurements.aggregate.count,result.requests);assert.equal(result.rawMeasurements.aggregate.count,result.rawMeasurements.byRequest.read.count);assert.equal(result.rawMeasurements.scheduling.count,result.requests);assert.equal(result.rawMeasurements.arrivalLatency.count,result.requests);
   assert.ok(!JSON.stringify(result.rawMeasurements).includes('private-token'));
  }
  assert.equal(mergeMeasurements(results.map(result=>result.rawMeasurements.aggregate),.15).accepted,results.reduce((sum,result)=>sum+result.accepted,0));
 }finally{server.closeAllConnections();await new Promise(resolve=>server.close(resolve));}
});

test('late coordinated workers retain an expired common deadline instead of opening a fresh window',async()=>{
 let dispatched=0;const server=createServer((_req,res)=>{dispatched++;res.end('{}');});await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
 try{
  const commonStartAtMs=Date.now()-100;
  const result=await measure({base:`http://127.0.0.1:${server.address().port}`,requests:[{name:'late',path:'/'}],concurrency:1,seconds:.05,arrivalRate:100,commonStartAtMs,includeRawMeasurements:true});
  assert.equal(dispatched,0);assert.equal(result.window.scheduledEndAtMs,commonStartAtMs+50);assert.ok(result.window.startLagMs>=100);assert.equal(result.offered,5);assert.equal(result.droppedScheduler,5);assert.equal(result.acceptedDuringScheduledWindow,0);
  assert.equal(result.rawMeasurements.aggregate.count,0);
  for(const invalid of [NaN,Date.now()+61000,Date.now()-61000,1.5])await assert.rejects(measure({base:'http://127.0.0.1',requests:[{name:'never',path:'/'}],concurrency:1,seconds:.01,commonStartAtMs:invalid}));
 }finally{server.closeAllConnections();await new Promise(resolve=>server.close(resolve));}
});

test('slow responses time out without an unbounded queue at 100k offered RPS',async()=>{
 const host=await fixture({delayMs:100});
 try{
  const result=await measure({base:host.base,concurrency:2,seconds:.15,timeoutMs:20,arrivalRate:100000,headers:{cookie:'session=fixture-only'},requests:[{name:'slow',path:'/read'}]});
  assert.equal(result.offered,15000);assert.equal(result.requests+result.droppedCapacity+result.droppedScheduler,15000);
  assert.ok(result.timeouts>0);assert.ok(result.droppedCapacity>0);assert.ok(result.peakOutstanding<=2);
  assert.ok(result.plannedArrivalLatencyMs.p99>=result.latencyMs.p50);
 }finally{await host.close();}
});

test('cookie/header authentication and errors are measured without leaking secrets',async()=>{
 const host=await fixture();
 try{
  for(const headers of [{cookie:'session=fixture-only'},{authorization:'Bearer fixture-only','x-private-header':'secret-header-value'}]){
   const result=await measure({base:host.base,concurrency:2,seconds:.1,arrivalRate:100,headers,requests:[{name:'read',path:'/read',validate:body=>body.ok},{name:'error',path:'/error'}]});
   assert.ok(result.accepted>0);assert.ok(result.errors>0);assert.ok(result.statuses[503]>0);
   assert.equal(result.rawMeasurements,undefined);assert.equal(result.window,undefined);
   assert.ok(!JSON.stringify(result).includes('fixture-only'));assert.ok(!JSON.stringify(result).includes('secret-header-value'));
  }
 }finally{await host.close();}
});

test('blocked generator reports scheduler misses instead of silently lowering offer',async()=>{
 const host=await fixture();
 try{
  const run=measure({base:host.base,concurrency:2,seconds:.25,arrivalRate:20000,requests:[{name:'health',path:'/healthz'}]});
  const until=performance.now()+160;while(performance.now()<until){}
  const result=await run;assert.equal(result.offered,5000);assert.ok(result.droppedScheduler>0);
  assert.equal(result.requests+result.droppedCapacity+result.droppedScheduler,5000);
 }finally{await host.close();}
});

test('success completed during drain is excluded from actual in-window accepted rate',async()=>{
 const host=await fixture({delayMs:150});
 try{
  const result=await measure({base:host.base,concurrency:1,seconds:.05,timeoutMs:1000,arrivalRate:20,headers:{cookie:'session=fixture-only'},requests:[{name:'drain',path:'/read'}]});
  assert.equal(result.offered,1);assert.equal(result.accepted,1);
  assert.equal(result.acceptedDuringScheduledWindow,0);assert.equal(result.acceptedDuringScheduledWindowRps,0);
  assert.ok(result.acceptedRequestsPerSecond>0);assert.ok(result.elapsedSeconds>result.scheduledSeconds);
 }finally{await host.close();}
});

test('principal rotation preserves fixed labels, validates each identity and redacts credentials',async()=>{
 const server=createServer((req,res)=>{const p=req.headers['x-test-principal'];res.setHeader('content-type','application/json');res.end(JSON.stringify({principal:p,valid:req.headers.cookie===`private-cookie-${p}`}));});
 await new Promise(done=>server.listen(0,'127.0.0.1',done));
 try{
  const result=await measure({base:`http://127.0.0.1:${server.address().port}`,concurrency:2,seconds:.2,arrivalRate:100,requests:[{name:'fixed-operation',path:'/',materialize:index=>{const principalIndex=index%3;return {name:'private-cookie-leak',principalIndex,headers:{cookie:`private-cookie-${principalIndex}`,'x-test-principal':String(principalIndex)},validate:v=>v.valid&&v.principal===String(principalIndex)};}}]});
  assert.equal(result.uniquePrincipalsAttempted,3);assert.equal(result.uniquePrincipalsAccepted,3);assert.equal(result.errors,0);
  assert.deepEqual(Object.keys(result.byRequest),['fixed-operation']);assert.ok(!JSON.stringify(result).includes('private-cookie'));
 }finally{server.closeAllConnections();await new Promise(done=>server.close(done));}
});
test('actual concurrent HTTP run reports application errors despite HTTP200',async()=>{
 const server=createServer((req,res)=>{res.setHeader('content-type','application/json');res.end(JSON.stringify({ok:req.url==='/good'}));});
 await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
 try{
 const result=await measure({base:`http://127.0.0.1:${server.address().port}`,concurrency:3,seconds:1,requests:[{name:'good',path:'/good',validate:body=>body.ok},{name:'bad',path:'/bad',validate:body=>body.ok}]});
 assert.ok(result.requests>10);assert.ok(result.errors>0);assert.equal(result.byRequest.good.errors,0);assert.equal(result.byRequest.bad.errors,result.byRequest.bad.requests);assert.equal(result.timeouts,0);
 assert.ok(result.peakOutstanding<=3);assert.ok(result.peakOutstanding>0);
 assert.ok(result.generator.cpuUserMs>=0);assert.ok(result.generator.rssAfterBytes>0);
 }finally{server.closeAllConnections();await new Promise(resolve=>server.close(resolve));}
});
test('explicitly accepted throttling remains visible rather than counting as successful mail',async()=>{
 const server=createServer((_req,res)=>{res.writeHead(429,{'content-type':'application/json'});res.end('{"error":"limit"}');});await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
 try{const result=await measure({base:`http://127.0.0.1:${server.address().port}`,concurrency:2,seconds:1,requests:[{name:'bounded',path:'/',statuses:[200,429],validate:()=>false}]});assert.ok(result.throttled>0);assert.equal(result.throttled,result.requests);assert.equal(result.errors,0);}finally{server.closeAllConnections();await new Promise(resolve=>server.close(resolve));}
});

test('fixed arrivals report missed capacity instead of reducing the offered rate',async()=>{
 let active=0,peak=0;
 const server=createServer((_req,res)=>{active++;peak=Math.max(peak,active);setTimeout(()=>{active--;res.end('ok');},120);});
 await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
 try{
  const result=await measure({base:`http://127.0.0.1:${server.address().port}`,concurrency:2,seconds:1,arrivalRate:100,requests:[{name:'slow',path:'/'}]});
  assert.equal(result.offered,100);
  assert.equal(result.requests+result.droppedCapacity+result.droppedScheduler,100);
  assert.ok(result.droppedCapacity>0);assert.ok(result.requests>0);
  assert.ok(peak<=2);assert.equal(result.peakOutstanding,2);assert.equal(result.errors,0);
 }finally{server.closeAllConnections();await new Promise(resolve=>server.close(resolve));}
});
