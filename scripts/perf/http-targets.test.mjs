import test from 'node:test';
import assert from 'node:assert/strict';
import {classifyHost,parseTargetOrigin,parseAllowList,validateTargets,proveOwnership} from './http-targets.mjs';
import {measure} from './http-load.mjs';
import {createServer} from 'node:http';

const PATH='/__fixture-diagnostic-0123abcd',TOKEN='token-of-this-run-0123456789';
const response=(status,body={})=>({status,json:async()=>body,body:{cancel:async()=>{}}});

test('only literal loopback and private rig addresses are classified as targetable',()=>{
 assert.equal(classifyHost('127.0.0.1'),'loopback');
 for(const host of ['10.0.0.5','10.255.255.254','172.16.0.1','172.31.255.1','192.168.1.20','[fd00::5]','[fc00::1]'])assert.equal(classifyHost(host),'private',host);
 for(const host of ['8.8.8.8','1.1.1.1','172.15.0.1','172.32.0.1','192.169.0.1','11.0.0.1','100.64.0.1','169.254.169.254','0.0.0.0','224.0.0.1','127.0.0.2','255.255.255.255','localhost','example.com','10.0.0.5.nip.io','[::1]','[::ffff:10.0.0.5]','[2001:db8::1]','[fe80::1]','','010.0.0.5','10.0.0'])assert.equal(classifyHost(host),'refused',host);
});
test('a target origin must be a plain http origin with a literal address and explicit port',()=>{
 assert.equal(parseTargetOrigin('http://10.1.2.3:8080').origin,'http://10.1.2.3:8080');
 for(const text of ['https://10.1.2.3:8080','http://10.1.2.3','http://10.1.2.3:8080/x','http://10.1.2.3:8080/?a=1','http://u:p@10.1.2.3:8080','http://8.8.8.8:80','http://example.com:80','nonsense'])assert.throws(()=>parseTargetOrigin(text),text);
});
test('the allow-list holds private addresses with ports only',()=>{
 assert.deepEqual(parseAllowList('10.0.0.5:8080,10.0.0.6:8080'),['http://10.0.0.5:8080','http://10.0.0.6:8080']);
 assert.deepEqual(parseAllowList(''),[]);
 for(const bad of ['8.8.8.8:80','10.0.0.5','example.com:80','127.0.0.1:80','10.0.0.5:80/x'])assert.throws(()=>parseAllowList(bad),/private rig address/,bad);
});
test('a private address outside the allow-list and every public address are refused',()=>{
 assert.deepEqual(validateTargets(['http://127.0.0.1:8080']).map(t=>t.kind),['loopback']);
 assert.throws(()=>validateTargets(['http://10.0.0.5:8080']),/allow-list/);
 assert.throws(()=>validateTargets(['http://10.0.0.5:8080'],['10.0.0.6:8080']),/allow-list/);
 assert.throws(()=>validateTargets(['http://10.0.0.5:8081'],['10.0.0.5:8080']),/allow-list/);
 for(const url of ['http://8.8.8.8:80','http://93.184.216.34:80','http://169.254.169.254:80','http://example.com:80'])assert.throws(()=>validateTargets([url],['8.8.8.8:80']),/./,url);
 assert.throws(()=>validateTargets([]));assert.throws(()=>validateTargets(['http://10.0.0.5:8080','http://10.0.0.5:8080'],['10.0.0.5:8080']),/Duplicate/);
 assert.equal(validateTargets(['http://10.0.0.5:8080','http://10.0.0.6:8080'],'10.0.0.5:8080,10.0.0.6:8080').length,2);
});
test('ownership: token accepted, token-less request refused; anything else is rejected',async()=>{
 const targets=validateTargets(['http://10.0.0.5:8080'],['10.0.0.5:8080']);
 const gate=(accepts)=>async(url,init)=>{assert.equal(String(url),'http://10.0.0.5:8080'+PATH);return init.headers?.['x-fixture-token']===accepts?response(200,{pool:{}}):response(401);};
 assert.deepEqual(await proveOwnership(targets,{path:PATH,token:TOKEN,fetchImpl:gate(TOKEN)}),['http://10.0.0.5:8080']);
 // another system that does not hold this run's token
 await assert.rejects(proveOwnership(targets,{path:PATH,token:TOKEN,fetchImpl:gate('someone-elses-token-000000')}),/did not accept/);
 // answers 200 to everything: not a token gate
 await assert.rejects(proveOwnership(targets,{path:PATH,token:TOKEN,fetchImpl:async()=>response(200,{})}),/not token-gated/);
 // unreachable or non-JSON
 await assert.rejects(proveOwnership(targets,{path:PATH,token:TOKEN,fetchImpl:async()=>{throw new Error('ECONNREFUSED');}}),/did not accept/);
 await assert.rejects(proveOwnership(targets,{path:PATH,token:TOKEN,fetchImpl:async()=>({status:200,json:async()=>{throw new Error('x');},body:{cancel:async()=>{}}})}),/snapshot/);
 await assert.rejects(proveOwnership(targets,{path:'/other',token:TOKEN,fetchImpl:gate(TOKEN)}),/control path and token/);
 await assert.rejects(proveOwnership(targets,{path:PATH,token:'short',fetchImpl:gate(TOKEN)}),/control path and token/);
 // the run's own loopback fixture needs no probe
 let calls=0;await proveOwnership(validateTargets(['http://127.0.0.1:9']),{fetchImpl:async()=>{calls++;}});assert.equal(calls,0);
});
test('a public address never reaches the network: refusal precedes any request',async()=>{
 let calls=0;
 assert.throws(()=>validateTargets(['http://8.8.8.8:80'],['8.8.8.8:80']));
 assert.equal(calls,0);
});
async function counting(){
 let count=0;const server=createServer((req,res)=>{count++;res.end('ok');});
 await new Promise(done=>server.listen(0,'127.0.0.1',done));
 return {base:`http://127.0.0.1:${server.address().port}`,count:()=>count,async close(){server.closeAllConnections();await new Promise(done=>server.close(done));}};
}
test('rotation sends consecutive requests to each target in order',async()=>{
 const a=await counting(),b=await counting(),c=await counting();
 try{
  const result=await measure({base:a.base,bases:[a.base,b.base,c.base],concurrency:1,seconds:.5,arrivalRate:60,requests:[{name:'h',path:'/healthz'}]});
  assert.ok(result.requests>=15);assert.equal(result.accepted,result.requests);
  assert.equal(a.count()+b.count()+c.count(),result.requests);
  assert.ok(Math.max(a.count(),b.count(),c.count())-Math.min(a.count(),b.count(),c.count())<=1,`${a.count()}/${b.count()}/${c.count()}`);
 }finally{await a.close();await b.close();await c.close();}
 await assert.rejects(measure({base:'http://127.0.0.1:1',bases:[],concurrency:1,seconds:1,requests:[{name:'h',path:'/'}]}),/rotation/);
});
