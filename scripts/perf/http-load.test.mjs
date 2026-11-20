import test from 'node:test';
import assert from 'node:assert/strict';
import {createServer} from 'node:http';
import {Measurements,measure} from './http-load.mjs';
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
