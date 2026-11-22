import test from 'node:test';
import assert from 'node:assert/strict';
import http from 'node:http';
import {createHash} from 'node:crypto';
import {readFile} from 'node:fs/promises';
import {once} from 'node:events';
import {handoff,phase,counters,qualify,request,validImage,COUNTERS,scheduledWindow,coordination} from './image-proxy-protocol.mjs';
import {runGenerator} from './image-proxy-benchmark.mjs';

const handoffInput=()=>({schema:1,kind:'real-signed-image-proxy',base:'http://127.0.0.1:32001',control:'http://127.0.0.1:32002',sourceAccounts:16,sourceSessions:16,lifetimeSecs:900,security:'enforced-default-limits',storeEncryption:'required',storeFileNotPlaintext:true,origin:'local-configured-connect-standin',originDelayMs:20,controls:{noSession:401,tamperedToken:403,privateTarget:403,originFetches:0,positiveSignedAccounts:16,positiveSignedStatus:200}});
const count=()=>Object.fromEntries(COUNTERS.map(k=>[k,0]));
const measured={accepted:16,errors:0,timeouts:0,throttled:0};
test('private handoff requires real-control evidence and exact loopback authorities',()=>{
 assert.equal(handoff(handoffInput()).sourceAccounts,16);
 for(const change of [{base:'http://evil.test:80'},{control:'http://127.0.0.1:80/steal'},{sourceAccounts:1},{security:'observe'},{storeEncryption:'optional'},{storeFileNotPlaintext:false},{controls:{noSession:200}}])assert.throws(()=>handoff({...handoffInput(),...change}));
});
test('phase capabilities are bounded, account-complete, and never arbitrary URLs',()=>{
 const input={schema:1,case:'ram',size:64,sourceAccounts:16,sourceSessions:16,originDelayMs:20,expectedBytes:25,expectedSha256:'a'.repeat(64),paths:Array.from({length:16},(_,i)=>({path:`/api/rc/i/private-${i}.sig`,principalIndex:i}))};
 assert.equal(phase(input,'ram',64).paths.length,16);
 for(const change of [{paths:input.paths.slice(1)},{paths:input.paths.map(x=>({...x,principalIndex:0}))},{expectedBytes:1<<20},{paths:input.paths.map(x=>({...x,path:'http://evil.test/'}))}])assert.throws(()=>phase({...input,...change},'ram',64));
});
test('actual outcome proof distinguishes RAM, disk, cold and single-flight',()=>{
 const before=count(),after={...count(),ramHits:16};
 assert.equal(qualify('ram',before,after,measured).qualified,true);
 assert.equal(qualify('ram',before,{...after,originFetches:1},measured).qualified,false);
 assert.equal(qualify('disk',before,{...count(),diskHits:16},measured).qualified,true);
 for(const change of [{ramBudgetBytes:1},{diskPendingBytes:1},{diskErrors:1},{ramHits:1},{diskHits:15}])assert.equal(qualify('disk',before,{...count(),diskHits:16,...change},measured).qualified,false);
 assert.equal(qualify('cold',before,{...count(),originFetches:16},measured).qualified,true);
 assert.equal(qualify('coalesced',before,{...count(),originFetches:16},{...measured,accepted:64}).qualified,true);
 assert.equal(qualify('coalesced',before,{...count(),originFetches:17},{...measured,accepted:64}).qualified,false);
 for(const change of [{errors:1},{timeouts:1},{throttled:1},{accepted:0}])assert.equal(qualify('ram',before,after,{...measured,...change}).qualified,false);
 assert.throws(()=>qualify('ram',{...before,ramHits:17},after,measured));
 assert.throws(()=>counters({...before,lastError:'private text'}));
});
test('real Node HTTP transport validates bytes/hash and refuses redirects/oversized bodies',async()=>{
 // This is a NODE transport contract, not actual Rust/image-handler proof.
 const payload=Buffer.from('synthetic-png-body');
 const server=http.createServer((req,res)=>{
  if(req.url==='/redirect'){res.writeHead(302,{location:'http://evil.test/'}).end();return;}
  if(req.url==='/oversize'){res.writeHead(200).end(Buffer.alloc(1025));return;}
  res.writeHead(200,{'content-type':'image/png'}).end(payload);
 });server.listen(0,'127.0.0.1');await once(server,'listening');
 try{
  const base=`http://127.0.0.1:${server.address().port}`;
  const proof={expectedBytes:payload.length,expectedSha256:createHash('sha256').update(payload).digest('hex')};
  assert.equal(validImage(await request(base,'/image',{principalIndex:0}),proof),true);
  assert.equal(validImage(await request(base,'/redirect'),proof),false);
  assert.equal(validImage(await request(base,'/image'),{...proof,expectedSha256:'0'.repeat(64)}),false);
  await assert.rejects(request(base,'/oversize',{maxBytes:1024}));
 }finally{server.closeAllConnections();await new Promise(resolve=>server.close(resolve));}
});
test('fixture source keeps production route absent and all measured cache proof public',async()=>{
 const root=new URL('../../',import.meta.url);
 const main=await readFile(new URL('crates/mw-server/tests/t32_remote_content.rs',root),'utf8');
 const source=await readFile(new URL('crates/mw-server/tests/t32_image_proxy_benchmark.rs',root),'utf8');
 assert.match(main,/#\[cfg\(feature = "load-test-fixtures"\)\]\s*#\[path = "t32_image_proxy_benchmark.rs"\]/);
 assert.match(source,/#\[ignore = "private real-image throughput fixture/);
 assert.match(source,/MW_RUN_IMAGE_PROXY_BENCHMARK/);assert.match(source,/enforced::install\(&store\)/);
 assert.match(source,/grant_remote_image/);assert.match(source,/\/api\/rc\/sign/);
 assert.doesNotMatch(source,/set_pinned_target|ip_allowed\s*\(|test_control.*allow|disable.*protection/i);
 assert.match(source,/"privacy.cacheBytes",json!\(if args.case=="disk"\{0\}/);
 assert.match(source,/encryption::connect_options\(path,key,false\)/);
 assert.match(source,/encryption\.key\.as_ref\(\)/);
 assert.match(source,/encryption.mode,mw_store::encryption::EncryptionMode::Required/);
 assert.doesNotMatch(source,/SqliteConnectOptions::new\(\)\.filename\(&db\)/);
 assert.doesNotMatch(source,/\.key_for_open\(/);
 const closed=source.indexOf('store.close().await;'),dropped=source.indexOf('drop(store);'),probe=source.indexOf('encryption::probe_file('),reopened=source.indexOf('let store=store_at(&db).await;',dropped),app=source.indexOf('build_app_full(AppConfig');
 assert(closed>=0&&closed<dropped&&dropped<probe&&probe<reopened&&reopened<app);
 assert.equal(source.indexOf('encryption::probe_file(',probe+1),-1,'no second header probe with active app ownership');
});
test('late workers retain the shared expired UTC deadline and issue zero HTTP requests',async()=>{
 const mapped=scheduledWindow(10000,1,10200,500);
 assert.equal(mapped.startMono,300);assert.equal(mapped.deadlineMono,1300);assert.equal(mapped.endAt,11000);assert.equal(mapped.readyBeforeStart,false);
 const valid={window:{startAt:10000,endAt:11000,readyBeforeStart:true,startLagMs:1,wallClockDriftMs:1}};
 assert.equal(coordination([valid],10000,1).qualified,true);
 for(const window of [{...valid.window,readyBeforeStart:false},{...valid.window,startLagMs:51},{...valid.window,endAt:11200},{...valid.window,wallClockDriftMs:101}])assert.equal(coordination([{window}],10000,1).qualified,false);
 let fetched=0;const server=http.createServer((_req,res)=>{fetched++;res.end('unexpected');});server.listen(0,'127.0.0.1');await once(server,'listening');
 try{
  const startAt=Date.now()-2000;
  const result=await runGenerator({base:`http://127.0.0.1:${server.address().port}`,proof:{expectedBytes:1,expectedSha256:'0'.repeat(64),paths:[{path:'/api/rc/i/expired.sig',principalIndex:0}]},seconds:1,concurrency:1,startAt,kind:'cold'});
  assert.equal(fetched,0);assert.equal(result.raw.count,0);assert.equal(result.raw.acceptedDuringScheduledWindow,0);
  assert.equal(result.window.startAt,startAt);assert.equal(result.window.endAt,startAt+1000);assert.equal(result.window.readyBeforeStart,false);assert.equal(coordination([result],startAt,1).qualified,false);
 }finally{server.closeAllConnections();await new Promise(resolve=>server.close(resolve));}
});
test('external generator validates its finite corpus without inheriting parent-only Node flags',async()=>{
 // Actual separate Node process and real HTTP; this synthetic receiver does
 // NOT establish Rust fixture, image transforms, grants or SSRF correctness.
 const payload=Buffer.from('synthetic-validated-image');let fetched=0;
 const server=http.createServer((req,res)=>{fetched++;res.writeHead(200,{'content-type':'image/png'}).end(payload);});
 server.listen(0,'127.0.0.1');await once(server,'listening');
 try{
  const savedArgs=process.execArgv;process.execArgv=[...savedArgs,'--input-type=module'];
  let result;try{
  result=await runGenerator({base:`http://127.0.0.1:${server.address().port}`,proof:{expectedBytes:payload.length,expectedSha256:createHash('sha256').update(payload).digest('hex'),paths:Array.from({length:16},(_,i)=>({path:`/api/rc/i/test-${i}.sig`,principalIndex:i}))},seconds:1,concurrency:4,startAt:Date.now()+500,kind:'cold'});
  }finally{process.execArgv=savedArgs;}
  assert.equal(fetched,16);assert.equal(result.raw.accepted,16);assert.equal(result.raw.errors,0);assert.equal(result.raw.timeouts,0);assert.equal(result.raw.acceptedDuringScheduledWindow,16);assert.equal(result.raw.acceptedDecodedBytes,payload.length*16);assert.equal(result.corpusExhausted,true);assert(result.resources.peakOutstanding<=4);
 }finally{server.closeAllConnections();await new Promise(resolve=>server.close(resolve));}
});
