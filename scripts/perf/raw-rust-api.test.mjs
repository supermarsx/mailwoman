import test from 'node:test';
import assert from 'node:assert/strict';
import {readFile,mkdtemp,writeFile,rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import {join,resolve,sep,basename} from 'node:path';
import {createServer} from 'node:http';
import {spawn} from 'node:child_process';
import {rawAuthority,rawOptions,rawRequests,rawMetrics,rawReport,rawStage,RAW_JSON} from './raw-rust-api-protocol.mjs';
import {runRawCalibration,startOwnedRawServer} from './raw-rust-api-client.mjs';
const options=()=>rawOptions({server:process.execPath,output:'private-report.json',seconds:'1',rates:'100',concurrency:'4',workers:'2','server-workers':'1',lifetime:'60'});
const authority=()=>({schema:1,kind:'raw-rust-axum-api',dataBase:'http://127.0.0.1:12345',controlBase:'http://127.0.0.1:12346',token:'a'.repeat(64),expiresAtMs:Date.now()+60000,runtimeWorkers:1});
test('raw Rust authority rejects remote paths reused origins expiry and weak tokens',()=>{
 assert.equal(rawAuthority(authority()).runtimeWorkers,1);
 for(const patch of [{kind:'synthetic-cluster-transport-calibration'},{dataBase:'http://localhost:12345'},{dataBase:'http://127.0.0.1:12345/path'},{controlBase:'http://127.0.0.1:12345'},{expiresAtMs:0},{token:'A'.repeat(64)},{runtimeWorkers:33}])assert.throws(()=>rawAuthority({...authority(),...patch}));
});
test('raw Rust options keep rates workers lifetime and outstanding work finite',()=>{
 const base={server:process.execPath,output:'x'};
 assert.equal(rawOptions(base).rates.at(-1),100000);
 for(const patch of [{server:'relative-binary'},{workers:17},{'server-workers':33},{seconds:31},{concurrency:4097},{rates:'100,100'},{rates:'100001'},{lifetime:10},{token:'secret'}])assert.throws(()=>rawOptions({...base,...patch}));
 for(const patch of [{endpoint:'synthetic-json-4k'},{rate:0},{concurrency:4097},{seconds:0}])assert.throws(()=>rawStage({id:0,endpoint:'raw-rust-json-11b',rate:100,concurrency:1,seconds:1,...patch}));
});
test('raw Rust payloads are exact11-byte JSON and truly empty204 bytes',()=>{
 const[json,empty]=rawRequests();assert.equal(RAW_JSON.length,11);assert.equal(json.validateBytes(Buffer.from('{"ok":true}')),true);assert.equal(json.validateBytes(Buffer.from('{"ok":false}')),false);assert.equal(empty.validateBytes(Buffer.alloc(0)),true);assert.equal(empty.validateBytes(Buffer.from(' ')),false);
});
test('raw handler counters never invent wire delivery or server resource measurements',()=>{
 const metrics={schema:1,kind:'raw-rust-axum-api',handlerInvocations:1,handlerBytesOffered:11,elapsedSeconds:1,runtimeWorkers:1,cpuUserMs:null,cpuSystemMs:null,rssBytes:null};
 assert.equal(rawMetrics({...metrics,token:'never-export'},1).rssBytes,null);assert.ok(!JSON.stringify(rawMetrics({...metrics,token:'never-export'},1)).includes('never-export'));
 for(const patch of [{handlerInvocations:-1},{handlerBytesOffered:NaN},{cpuUserMs:0},{rssBytes:0},{runtimeWorkers:2}])assert.throws(()=>rawMetrics({...metrics,...patch},1));
 const report=rawReport(options(),[]);assert.equal(report.ceilingEstablished,false);assert.equal(report.applicationCapacityMeasured,false);assert.equal(report.securityProof,false);assert.equal(report.wireResponseBytes,null);assert.equal(report.payloadBytes.json,11);
});
test('Rust source is feature-only and keeps normal mock and application routes unchanged',async()=>{
 const cargo=await readFile('crates/mw-mock-jmap/Cargo.toml','utf8'),source=await readFile('crates/mw-mock-jmap/examples/raw_api_calibration.rs','utf8');
 assert.match(cargo,/\[\[example\]\][\s\S]*name = "raw-api-calibration"[\s\S]*required-features = \["load-test-fixtures"\]/);
 assert.match(source,/MW_RUN_RAW_RUST_API_CALIBRATION/);assert.match(source,/create_new\(true\)/);assert.match(source,/mode\(0o600\)/);assert.match(source,/127\.0\.0\.1:0/g);
 assert.ok(!source.includes('mw_server::'));assert.ok(!source.includes('mw_store::'));
 for(const path of ['crates/mw-mock-jmap/src/lib.rs','crates/mw-mock-jmap/src/main.rs'])assert.ok(!(await readFile(path,'utf8')).includes('raw_api_calibration'));
});
async function receiver({badEmpty=false}={}){
 let requests=0,bytes=0,started=performance.now();const token='a'.repeat(64);
 const data=createServer((req,res)=>{
  if(req.headers['x-calibration-token']!==token){res.statusCode=404;res.end();return;}
  if(req.url==='/json'){requests++;bytes+=11;res.setHeader('content-type','application/json');res.end(RAW_JSON);return;}
  if(req.url==='/empty'){requests++;res.statusCode=badEmpty?200:204;res.end();return;}res.statusCode=404;res.end();
 });
 const control=createServer((req,res)=>{
  if(req.headers['x-calibration-token']!==token){res.statusCode=404;res.end();return;}
  if(req.url==='/__reset'){requests=0;bytes=0;started=performance.now();res.statusCode=204;res.end();return;}
  if(req.url==='/__metrics'){res.setHeader('content-type','application/json');res.end(JSON.stringify({schema:1,kind:'raw-rust-axum-api',handlerInvocations:requests,handlerBytesOffered:bytes,elapsedSeconds:(performance.now()-started)/1000,runtimeWorkers:1,cpuUserMs:null,cpuSystemMs:null,rssBytes:null}));return;}res.statusCode=404;res.end();
 });
 await Promise.all([data,control].map(server=>new Promise(done=>server.listen(0,'127.0.0.1',done))));
 return {handoff:{...authority(),dataBase:'http://127.0.0.1:'+data.address().port,controlBase:'http://127.0.0.1:'+control.address().port},async close(){await Promise.all([data,control].map(server=>new Promise(done=>{server.closeAllConnections();server.close(done);})))}};
}
test('real multi-process generator measures tiny/204 framing without fake principals or old payload labels',async()=>{
 // Synthetic Node receiver verifies driver orchestration only; actual Rust
 // execution is a separate required root gate, never inferred from this test.
 const app=await receiver();try{
  const report=await runRawCalibration({handoff:app.handoff,options:options(),startLeadMs:500});
  assert.equal(report.stages.length,2);assert.equal(report.ceilingEstablished,false);
  for(const stage of report.stages){assert.ok(stage.acceptedDuringScheduledWindow>0);assert.equal(stage.errors,0);assert.ok(Number.isFinite(stage.latencyMs.p50));assert.ok(Number.isFinite(stage.latencyMs.p95));assert.ok(Number.isFinite(stage.latencyMs.p99));assert.equal(stage.server.cpuUserMs,null);assert.equal(stage.wireResponseBytes,null);assert.ok(stage.server.handlerInvocations>=stage.accepted);}
  assert.equal(report.stages[0].acceptedDecodedBytes,report.stages[0].accepted*11);assert.equal(report.stages[1].acceptedDecodedBytes,0);assert.ok(!JSON.stringify(report).includes(app.handoff.token));assert.ok(!JSON.stringify(report).includes('synthetic-json-4k'));assert.ok(!JSON.stringify(report).includes('synthetic-binary-64k'));
 }finally{await app.close();}
});
test('real preflight refuses a200 empty body presented as the204 endpoint',async()=>{
 const app=await receiver({badEmpty:true});try{await assert.rejects(runRawCalibration({handoff:app.handoff,options:options(),startLeadMs:500}),/framing/);}finally{await app.close();}
});
test('owned child startup validates private handoff and stops the original process',async()=>{
 const directory=await mkdtemp(join(tmpdir(),'mw-raw-child-test-')),script=join(directory,'receiver.cjs');
 const source=[
  'const http=require("node:http"),fs=require("node:fs");const token=process.env.MW_RAW_API_CALIBRATION_TOKEN;',
  'if(process.env.MW_RUN_RAW_RUST_API_CALIBRATION!=="1"||!/^[a-f0-9]{64}$/.test(token))process.exit(2);',
  'let data,control;const allowed=req=>req.headers["x-calibration-token"]===token;',
  'data=http.createServer((req,res)=>{if(!allowed(req)){res.statusCode=404;res.end();return;}res.setHeader("content-type","application/json");res.end("{\\"ok\\":true}");});',
  'control=http.createServer((req,res)=>{if(!allowed(req)){res.statusCode=404;res.end();return;}if(req.url!=="/__stop"){res.statusCode=404;res.end();return;}res.statusCode=204;res.end();setTimeout(()=>{data.closeAllConnections();control.closeAllConnections();data.close(()=>control.close(()=>process.exit(0)));},10);});',
  'data.listen(0,"127.0.0.1",()=>control.listen(0,"127.0.0.1",()=>{fs.writeFileSync(process.env.MW_RAW_API_CALIBRATION_HANDOFF,JSON.stringify({schema:1,kind:"raw-rust-axum-api",dataBase:"http://127.0.0.1:"+data.address().port,controlBase:"http://127.0.0.1:"+control.address().port,token,expiresAtMs:Date.now()+60000,runtimeWorkers:Number(process.env.MW_RAW_API_RUNTIME_WORKERS)}),{flag:"wx",mode:384});console.log("{\\"ready\\":true,\\"rawRustAxum\\":true}");}));',
  'setTimeout(()=>process.exit(3),30000).unref();',
 ].join('\n');await writeFile(script,source,{mode:0o600});let child;
 try{
  // The injected Node process is an explicit lifecycle test double, not proof
  // of Rust endpoint execution or throughput.
  const owned=await startOwnedRawServer({server:process.execPath,options:options(),spawnServer:(_exe,_args,config)=>(child=spawn(process.execPath,[script],config))});
  const response=await fetch(new URL('/json',owned.authority.dataBase),{headers:{'x-calibration-token':owned.authority.token}});assert.deepEqual(await response.json(),{ok:true});await owned.close();assert.equal(child.exitCode,0);
 }finally{
  if(child&&child.exitCode===null){child.kill('SIGKILL');await new Promise(done=>child.once('exit',done));}
  const target=resolve(directory);assert.ok(target.startsWith(resolve(tmpdir())+sep)&&basename(target).startsWith('mw-raw-child-test-'));await rm(target,{recursive:true,force:true});
 }
});
