// The native stage object has the measure() schema. Needs a built mw-loadgen
// (MW_NATIVE_GENERATOR_BIN or target/release); skipped without one.
import test from 'node:test';
import assert from 'node:assert/strict';
import {createServer} from 'node:http';
import {mkdtempSync,writeFileSync,readFileSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {measure} from '../http-load.mjs';
import {nativeBinary,runBinary,nativeStageResult,nativePlan,groupLayout,rateLimitsOf} from './native-measure.mjs';

let bin=null;try{bin=nativeBinary();}catch{/* not built */}
const shape=value=>value&&typeof value==='object'&&!Array.isArray(value)?Object.fromEntries(Object.keys(value).sort().map(k=>[k,shape(value[k])])):typeof value;

test('rate limits come from the enforced policy, else the defaults',()=>{
 assert.deepEqual(rateLimitsOf({policy:{parsed:{clientRequests:600,accountRequests:1800,rateWindowSecs:60}}}).source,'handoff enforced policy');
 assert.equal(rateLimitsOf(null).clientRequests,600);
 assert.deepEqual(groupLayout({g:[{name:'a',serializePrincipal:true},{name:'b',statefulMutation:true}]}),{g:[{name:'a',serialize:true,stateful:false},{name:'b',serialize:false,stateful:true}]});
 assert.throws(()=>nativePlan({principals:[],engineMode:'native',representative:true,groups:{},env:{MW_NATIVE_GENERATOR_CLIENTS_PER_USER:'0'}}),/CLIENTS_PER_USER/);
});

test('native stage object has the measure() keys',{skip:!bin&&'mw-loadgen not built'},async()=>{
 const server=createServer((req,res)=>{res.setHeader('content-type','application/json');res.end('{"ok":true}');});
 await new Promise(r=>server.listen(0,'127.0.0.1',r));
 const base=`http://127.0.0.1:${server.address().port}`,dir=mkdtempSync(join(tmpdir(),'mw-native-test-'));
 try{
  const requests=[{name:'read',path:'/read'}];
  for(const options of [{},{arrivalRate:200,latePolicy:'dispatch'}]){
   const node=await measure({base,requests,concurrency:4,seconds:0.5,timeoutMs:2000,...options});
   writeFileSync(join(dir,'stage.json'),JSON.stringify({base,concurrency:4,seconds:0.5,timeoutMs:2000,...options,plain:[{name:'read',path:'/read',validate:'ok-true'}]}));
   await runBinary(bin,['stage',`--stage=${join(dir,'stage.json')}`,`--out=${join(dir,'out.json')}`],{timeoutMs:30000});
   const native=nativeStageResult(JSON.parse(readFileSync(join(dir,'out.json'),'utf8')),{requests,seconds:0.5,concurrency:4,...options});
   const {native:_,generatorImplementation,generator,...rest}=native,{generator:nodeGenerator,...nodeRest}=node;
   assert.equal(generatorImplementation,'native');
   assert.deepEqual(shape(rest),shape(nodeRest));
   assert.deepEqual(Object.keys(nodeGenerator).filter(k=>!(k in generator)),[]);
   assert.ok(native.accepted>0&&native.errors===0);
  }
 }finally{server.close();rmSync(dir,{recursive:true,force:true});}
});
