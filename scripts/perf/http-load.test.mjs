import test from 'node:test';
import assert from 'node:assert/strict';
import {createServer} from 'node:http';
import {Measurements,measure} from './http-load.mjs';

test('latency percentiles retain failures, timeouts and exact status counts',()=>{
 const m=new Measurements();for(let n=1;n<=100;n++)m.add(n,'200',5,n!==100,n===100);
 const result=m.report(2);assert.equal(result.latencyMs.p50,50);assert.equal(result.latencyMs.p95,95);assert.equal(result.latencyMs.p99,99);assert.equal(result.requestsPerSecond,50);assert.equal(result.errors,1);assert.equal(result.timeouts,1);assert.equal(result.bytes,500);
});
test('actual concurrent HTTP run reports application errors despite HTTP200',async()=>{
 const server=createServer((req,res)=>{res.setHeader('content-type','application/json');res.end(JSON.stringify({ok:req.url==='/good'}));});
 await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
 try{
 const result=await measure({base:`http://127.0.0.1:${server.address().port}`,concurrency:3,seconds:1,requests:[{name:'good',path:'/good',validate:body=>body.ok},{name:'bad',path:'/bad',validate:body=>body.ok}]});
 assert.ok(result.requests>10);assert.ok(result.errors>0);assert.equal(result.byRequest.good.errors,0);assert.equal(result.byRequest.bad.errors,result.byRequest.bad.requests);assert.equal(result.timeouts,0);
 }finally{server.closeAllConnections();await new Promise(resolve=>server.close(resolve));}
});
test('explicitly accepted throttling remains visible rather than counting as successful mail',async()=>{
 const server=createServer((_req,res)=>{res.writeHead(429,{'content-type':'application/json'});res.end('{"error":"limit"}');});await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
 try{const result=await measure({base:`http://127.0.0.1:${server.address().port}`,concurrency:2,seconds:1,requests:[{name:'bounded',path:'/',statuses:[200,429],validate:()=>false}]});assert.ok(result.throttled>0);assert.equal(result.throttled,result.requests);assert.equal(result.errors,0);}finally{server.closeAllConnections();await new Promise(resolve=>server.close(resolve));}
});
