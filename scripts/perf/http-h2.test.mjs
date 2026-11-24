import test from 'node:test';
import assert from 'node:assert/strict';
import http from 'node:http';
import http2 from 'node:http2';
import https from 'node:https';
import {mkdtemp,readFile,rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {execFileSync} from 'node:child_process';
import {measure} from './http-load.mjs';
import {validateH2,h2FromEnv,createH2Client,mergeH2Reports,h2Burst,H2_PRESETS} from './http-h2-client.mjs';
import {campaignOptions} from './http-campaign.mjs';
import {parseTargetOrigin,parseAllowList,validateTargets,proveOwnership} from './http-targets.mjs';
import {runMultiCampaign} from './http-multi-controller.mjs';
import {handoffFor} from './test-fixtures/multi-handoff.mjs';

const {NGHTTP2_REFUSED_STREAM}=http2.constants;
const sleep=ms=>new Promise(done=>setTimeout(done,ms));

// One handler for both protocols so parity means the same bytes from the same code.
function respond(path,headers,send,state){
 if(path.startsWith('/ok')||path==='/slow'||path==='/goaway'||path==='/echo'){
  state.addresses?.add(headers['x-forwarded-for']);
  const reply=()=>send(200,{'content-type':'application/json'},JSON.stringify({ok:true,path}));
  path==='/slow'?setTimeout(reply,40):reply();
  return;
 }
 if(path==='/bad'){send(200,{'content-type':'application/json'},JSON.stringify({ok:false}));return;}
 if(path.startsWith('/burst')){
  state.active++;state.peak=Math.max(state.peak,state.active);
  const busy=state.active>(state.busyAbove??Infinity);
  setTimeout(()=>{state.active--;busy?send(503,{'content-type':'application/json'},JSON.stringify({code:'assetLoadBusy'})):send(200,{'content-type':'text/plain'},'asset');},30);
  return;
 }
 send(404,{},'');
}
async function h2Server({maxConcurrentStreams=32,secure,busyAbove}={}){
 const state={addresses:new Set(),sessions:0,streams:0,active:0,peak:0,busyAbove};
 const server=secure?http2.createSecureServer({...secure,settings:{maxConcurrentStreams}}):http2.createServer({settings:{maxConcurrentStreams}});
 server.on('session',session=>{state.sessions++;session.on('error',()=>{});});
 server.on('sessionError',()=>{});
 server.on('stream',(stream,headers)=>{
  state.streams++;stream.on('error',()=>{});
  const path=headers[':path'];
  if(path==='/refuse'){stream.close(NGHTTP2_REFUSED_STREAM);return;}
  respond(path,headers,(status,extra,body)=>{if(stream.destroyed)return;stream.respond({':status':status,...extra});stream.end(body);if(path==='/goaway')stream.session.goaway();},state);
 });
 await new Promise(done=>server.listen(0,'127.0.0.1',done));
 return {state,port:server.address().port,origin:(secure?'https':'http')+'://127.0.0.1:'+server.address().port,async close(){server.closeAllConnections?.();await new Promise(done=>server.close(done));}};
}
async function h1Server(){
 const state={addresses:new Set(),active:0,peak:0};
 const server=http.createServer((req,res)=>respond(req.url,req.headers,(status,extra,body)=>{res.writeHead(status,extra);res.end(body);},state));
 await new Promise(done=>server.listen(0,'127.0.0.1',done));
 return {state,origin:'http://127.0.0.1:'+server.address().port,async close(){server.closeAllConnections();await new Promise(done=>server.close(done));}};
}
const h2c=(extra={})=>({mode:'h2c',...extra});
const template=(path,extra={})=>({name:'t',path,validate:value=>value.ok===true,...extra});

test('configuration: HTTP/1.1 stays the default; TLS needs an explicit CA or the loudly named rig-only flag',()=>{
 assert.equal(h2FromEnv({}),undefined);
 assert.equal(campaignOptions({}).h2,undefined);
 assert.deepEqual(campaignOptions({MW_LOAD_FIXTURE_HTTP2:'h2c'}).h2,{mode:'h2c',preset:'proxy-like',perClient:false,connections:4,maxStreams:100});
 assert.equal(validateH2({mode:'h2c',preset:'browser-like'}).perClient,true);
 assert.deepEqual(Object.keys(H2_PRESETS),['browser-like','proxy-like']);
 const pem='-----BEGIN CERTIFICATE-----\nabc\n-----END CERTIFICATE-----';
 assert.throws(()=>validateH2({mode:'tls'}),/exactly one/);
 assert.throws(()=>validateH2({mode:'tls',caPem:pem,rigOnlyInsecureTlsNoVerify:true}),/exactly one/);
 assert.throws(()=>validateH2({mode:'tls',caPem:'not a certificate'}),/exactly one/);
 assert.equal(validateH2({mode:'tls',caPem:pem}).rigOnlyInsecureTlsNoVerify,undefined);
 assert.equal(validateH2({mode:'tls',rigOnlyInsecureTlsNoVerify:true}).rigOnlyInsecureTlsNoVerify,true);
 assert.throws(()=>validateH2({mode:'h2c',caPem:pem}),/TLS options/);
 assert.throws(()=>validateH2({mode:'h2c',rigOnlyInsecureTlsNoVerify:true}),/TLS options/);
 assert.throws(()=>validateH2({mode:'h3'}));assert.throws(()=>validateH2({mode:'h2c',preset:'x'}));
 assert.throws(()=>validateH2({mode:'h2c',connections:65}));assert.throws(()=>validateH2({mode:'h2c',maxStreams:0}));
 // the flag is only reachable through its long name
 assert.equal(h2FromEnv({MW_LOAD_FIXTURE_HTTP2:'tls',MW_LOAD_RIG_ONLY_INSECURE_TLS_NO_VERIFY:'1'}).rigOnlyInsecureTlsNoVerify,true);
 assert.throws(()=>h2FromEnv({MW_LOAD_FIXTURE_HTTP2:'tls',MW_LOAD_RIG_ONLY_INSECURE_TLS_NO_VERIFY:'true'}),/exactly one/);
});

test('protocol parity: the same template, validator and counters run over HTTP/1.1 and HTTP/2',async()=>{
 const a=await h1Server(),b=await h2Server();
 try{
  // first use of fetch in a process can stall the loop for a while on a cold start; keep that out of the comparison
  await (await fetch(a.origin+'/ok')).arrayBuffer();
  const run=(base,h2)=>measure({base,latePolicy:'dispatch',concurrency:8,seconds:.5,arrivalRate:200,requests:[template('/ok'),template('/bad'),{name:'t3',path:'/ok/3'}],...(h2?{h2}:{})});
  const one=await run(a.origin),two=await run(b.origin,h2c());
  assert.equal(one.protocol,'http/1.1');assert.equal(two.protocol,'h2c');assert.equal(one.http2,undefined);
  for(const result of [one,two]){assert.ok(result.requests>=50,'requests '+result.requests+' '+result.protocol+' '+JSON.stringify([result.droppedScheduler,result.statuses,result.timeouts,result.errors,result.peakOutstanding,result.schedulingLagMs,result.elapsedSeconds,result.latencyMs]));assert.equal(result.accepted+result.errors,result.requests);assert.ok(result.errors>0);assert.equal(result.statuses['200'],result.requests);}
  // identical request pattern: a third of the traffic fails validation in both
  assert.ok(Math.abs(one.errors/one.requests-two.errors/two.requests)<.08);
  assert.equal(two.http2.streams,two.requests);assert.equal(two.http2.connectionsOpened,4);
  }finally{await a.close();await b.close();}
});

test('a full stream limit queues in the generator and counts as scheduling lag, without drops',async()=>{
 const server=await h2Server({maxConcurrentStreams:2});
 try{
  const result=await measure({base:server.origin,concurrency:100,seconds:.6,arrivalRate:150,latePolicy:'dispatch',requests:[template('/slow')],h2:h2c({connections:1,maxStreams:100})});
  assert.equal(result.http2.serverMaxConcurrentStreams,2);
  assert.ok(result.http2.peakStreamsInFlight<=2,'peak '+result.http2.peakStreamsInFlight);
  assert.ok(result.http2.queuedStreams>0);assert.ok(result.http2.queueWaitMs.max>40);
  assert.equal(result.droppedCapacity,0);assert.equal(result.errors,0);assert.equal(result.accepted,result.requests);
  assert.ok(result.schedulingLagMs.max>=result.http2.queueWaitMs.max*0.5,'lag includes the queue');
  assert.equal(result.generatorLimited,true);
  assert.equal(server.state.sessions,1);
 }finally{await server.close();}
});

test('GOAWAY: the connection is replaced and counted, later requests still succeed',async()=>{
 const server=await h2Server();
 try{
  const result=await measure({base:server.origin,concurrency:1,seconds:.6,timeoutMs:2000,requests:[template('/goaway')],h2:h2c({connections:1})});
  assert.ok(result.http2.goaway>=1);assert.ok(result.http2.reconnects>=1);assert.ok(result.http2.connectionsOpened>=2);
  assert.ok(result.accepted>=2);assert.ok(server.state.sessions>=2);
  assert.equal(result.http2.connectionsOpened,1+result.http2.reconnects);
 }finally{await server.close();}
});

test('REFUSED_STREAM is counted as a refusal and an error, never as a local cancel',async()=>{
 const server=await h2Server();
 try{
  const result=await measure({base:server.origin,concurrency:2,seconds:.3,requests:[{name:'r',path:'/refuse'}],h2:h2c({connections:1})});
  assert.ok(result.requests>5);assert.equal(result.errors,result.requests);
  assert.equal(result.http2.refusedStreams,result.requests);assert.equal(result.http2.rstStream[NGHTTP2_REFUSED_STREAM],result.requests);
  assert.equal(result.http2.localCancels,0);assert.equal(result.http2.rstStreamTotal,result.requests);
 }finally{await server.close();}
});

test('presets: browser-like keeps one pool per simulated client, proxy-like shares a few connections; client addresses survive',async()=>{
 const server=await h2Server();
 try{
  const requests=[template('/echo',{materialize:index=>({clientIndex:index%5,headers:{'x-forwarded-for':'198.18.0.'+(index%7+1)}})})];
  const browser=await measure({base:server.origin,concurrency:10,seconds:.4,arrivalRate:200,requests,h2:h2c({preset:'browser-like'})});
  assert.equal(browser.http2.preset,'browser-like');assert.equal(browser.http2.pools,5);assert.equal(browser.http2.connectionsOpened,5);
  assert.ok(browser.http2.streamsPerConnection.min>3);
  const sessionsAfterBrowser=server.state.sessions;
  const proxy=await measure({base:server.origin,concurrency:10,seconds:.4,arrivalRate:200,requests,h2:h2c({preset:'proxy-like',connections:3})});
  assert.equal(proxy.http2.pools,1);assert.equal(proxy.http2.connectionsOpened,3);assert.equal(server.state.sessions-sessionsAfterBrowser,3);
  assert.equal(proxy.uniqueClientsAttempted,5);
  assert.equal(server.state.addresses.size,7,'distinct forwarded addresses reach the server through shared connections');
  assert.equal(proxy.http2.streamsPerConnection.total,proxy.requests);
 }finally{await server.close();}
});

test('burst: N distinct GETs on one connection, with the server\'s own 503 reasons reported',async()=>{
 const server=await h2Server({maxConcurrentStreams:32,busyAbove:8});
 try{
  const paths=Array.from({length:24},(_,i)=>'/burst/'+i);
  const result=await h2Burst({origin:server.origin,paths,h2:h2c()});
  assert.equal(result.requests,24);assert.equal(result.connection.connectionsOpened,1);assert.equal(server.state.sessions,1);
  assert.equal(result.statuses['200']+result.statuses['503'],24);assert.ok(result.statuses['503']>0);
  assert.equal(result.http503Reasons.assetLoadBusy,result.statuses['503']);
  assert.ok(server.state.peak>8,'streams overlapped on the connection');
  assert.ok(result.results.every(row=>paths.includes(row.path)));
  await assert.rejects(h2Burst({origin:server.origin,paths:['/a','/a'],h2:h2c()}),/distinct/);
  const small=await h2Server({maxConcurrentStreams:4});
  try{
   const many=Array.from({length:16},(_,i)=>'/burst/'+i);
   const polite=await h2Burst({origin:small.origin,paths:many,h2:h2c()});
   assert.equal(polite.statuses['200'],16);assert.ok(polite.connection.queuedStreams>0);assert.equal(polite.connection.connectionsOpened,1);
   const rude=await h2Burst({origin:small.origin,paths:many,h2:h2c(),exceedServerLimit:true});
   assert.equal(rude.connection.connectionsOpened,1);assert.equal(rude.connection.queuedStreams,0);
  }finally{await small.close();}
 }finally{await server.close();}
});

test('target rules apply unchanged to HTTP/2: public, non-listed and non-owned addresses are refused',async()=>{
 assert.throws(()=>parseTargetOrigin('https://8.8.8.8:8443',{tls:true}));
 assert.throws(()=>parseTargetOrigin('http://10.0.0.5:8443',{tls:true}));
 assert.throws(()=>parseTargetOrigin('https://10.0.0.5:8443'));
 assert.equal(parseTargetOrigin('https://10.0.0.5:8443',{tls:true}).kind,'private');
 assert.deepEqual(parseAllowList('10.0.0.5:8443',{tls:true}),['https://10.0.0.5:8443']);
 assert.deepEqual(parseAllowList(['http://10.0.0.5:8080']),['http://10.0.0.5:8080'],'origin-form entries (as handed to workers) are accepted');
 assert.throws(()=>validateTargets(['https://10.0.0.5:8443'],[],{tls:true}),/allow-list/);
 assert.throws(()=>validateTargets(['https://8.8.8.8:8443'],['8.8.8.8:8443'],{tls:true}));
 const options={mode:'smoke',concurrency:[4],rates:[100],seconds:1,timeoutMs:1000,endpoints:['query'],warmupSeconds:0,deadlineSeconds:30,maxErrorRate:0,maxThrottleRate:0,maxP99Ms:100,allowOverload:false,adaptive:true,h2:validateH2({mode:'h2c'})};
 const diag={path:'/__fixture-diagnostic-0123abcd',token:'run-control-token-0123456789'};
 let launched=false;
 const input={workers:2,options,launch:()=>{launched=true;throw new Error('no launch');},startLeadMs:500,allowedTargets:['10.0.0.5:8080']};
 const target={...handoffFor(4),base:'http://10.0.0.5:8080',targets:['http://10.0.0.5:8080'],diagnostics:diag};
 await assert.rejects(runMultiCampaign({...input,handoff:target,targetFetch:async()=>({status:401,ok:false,json:async()=>({}),body:{cancel:async()=>{}}})}),/control token/);
 await assert.rejects(runMultiCampaign({...input,handoff:{...target,targets:['http://10.0.0.7:8080']},targetFetch:async()=>{throw new Error('must not be contacted');}}),/allow-listed private/);
 await assert.rejects(runMultiCampaign({...input,allowedTargets:['8.8.8.8:80'],handoff:{...target,targets:['http://8.8.8.8:80']},targetFetch:async()=>{throw new Error('must not be contacted');}}),/allow-listed private|private rig/);
 assert.equal(launched,false);
});

test('ownership proof can run over HTTP/2 itself (same function, h2 one-shot transport)',async()=>{
 const path='/__fixture-diagnostic-0123abcd',token='run-control-token-0123456789';
 const server=http2.createServer();
 server.on('stream',(stream,headers)=>{const ok=headers[':path']===path&&headers['x-fixture-token']===token;stream.respond({':status':ok?200:401,'content-type':'application/json'});stream.end(ok?'{"pool":{}}':'{}');});
 await new Promise(done=>server.listen(0,'127.0.0.1',done));
 const client=createH2Client(h2c({connections:1}));
 try{
  const origin='http://127.0.0.1:'+server.address().port;
  const probe=(url,init)=>client.oneShot(url,init);
  const sample=await probe(origin+path,{headers:{'x-fixture-token':token}});
  assert.equal(sample.status,200);assert.deepEqual(await sample.json(),{pool:{}});
  assert.equal((await probe(origin+path)).status,401);
  // a non-loopback address would go through proveOwnership with this probe; loopback is the run's own fixture
  assert.deepEqual(await proveOwnership(validateTargets([origin]),{fetchImpl:probe}),[origin]);
 }finally{client.close();await new Promise(done=>server.close(done));}
});

test('worker reports from several processes merge; a mixed protocol set is refused',()=>{
 const report=(extra={})=>({schema:1,protocol:'h2c',transport:'h2c',preset:'proxy-like',perClientConnections:false,connectionsPerPool:4,pools:1,maxStreamsConfigured:100,serverMaxConcurrentStreams:32,connectionsOpened:4,connectionsClosed:4,reconnects:0,connectErrors:0,alpnFailures:0,goaway:1,rstStream:{7:2},rstStreamTotal:2,refusedStreams:2,localCancels:0,streams:100,peakStreamsInFlight:8,queuedStreams:3,queueWaitMs:{total:9,max:5},streamsPerConnection:{connections:4,min:10,max:40,total:100},flowControlStalls:'not exposed by node:http2',...extra});
 const merged=mergeH2Reports([report(),report({serverMaxConcurrentStreams:16,streamsPerConnection:{connections:4,min:2,max:50,total:100},rstStream:{7:1,8:1}})]);
 assert.equal(merged.streams,200);assert.equal(merged.connectionsOpened,8);assert.equal(merged.serverMaxConcurrentStreams,16);
 assert.deepEqual(merged.rstStream,{7:3,8:1});assert.deepEqual(merged.streamsPerConnection,{connections:8,min:2,max:50,total:200});assert.equal(merged.queueWaitMs.max,5);
 assert.throws(()=>mergeH2Reports([report(),report({preset:'browser-like'})]),/different HTTP\/2 models/);
 assert.equal(mergeH2Reports([undefined]),undefined);
});

let certs=null;
try{
 const dir=await mkdtemp(join(tmpdir(),'mw-h2-cert-'));
 execFileSync('openssl',['req','-x509','-newkey','rsa:2048','-nodes','-keyout',join(dir,'k.pem'),'-out',join(dir,'c.pem'),'-days','2','-subj','/CN=rig','-addext','subjectAltName=IP:127.0.0.1'],{stdio:'ignore'});
 certs={key:await readFile(join(dir,'k.pem'),'utf8'),cert:await readFile(join(dir,'c.pem'),'utf8')};
 await rm(dir,{recursive:true,force:true});
}catch{/* openssl unavailable: TLS tests are skipped with a visible reason */}

test('h2 over TLS: ALPN, certificate verification against the rig CA, wrong CA refused, explicit insecure flag',{skip:certs?false:'openssl is not available to create a throwaway rig CA'},async()=>{
 const server=await h2Server({secure:certs});
 try{
  const run=h2=>measure({base:server.origin,concurrency:4,seconds:.3,requests:[template('/ok')],h2});
  const good=await run({mode:'tls',caPem:certs.cert});
  assert.equal(good.protocol,'h2');assert.ok(good.accepted>5);assert.equal(good.errors,0);assert.equal(good.http2.transport,'tls');
  const other=execFileSync('openssl',['req','-x509','-newkey','rsa:2048','-nodes','-keyout','-','-days','2','-subj','/CN=other','-addext','subjectAltName=IP:127.0.0.1'],{stdio:['ignore','pipe','ignore']}).toString();
  const otherCert=other.slice(other.indexOf('-----BEGIN CERTIFICATE-----'));
  // a certificate the rig CA did not sign stops the stage before any load is sent
  await assert.rejects(run({mode:'tls',caPem:otherCert}),/self-signed|certificate/i);
  const insecure=await run({mode:'tls',rigOnlyInsecureTlsNoVerify:true});
  assert.ok(insecure.accepted>5);
  // a TLS listener without h2 never reaches the application: ALPN is required
  const plain=https.createServer({...certs},(req,res)=>res.end('{}'));
  await new Promise(done=>plain.listen(0,'127.0.0.1',done));
  try{
   await assert.rejects(measure({base:'https://127.0.0.1:'+plain.address().port,concurrency:1,seconds:.2,requests:[template('/ok')],h2:{mode:'tls',caPem:certs.cert}}),/h2|closed|ALPN|protocol|not ready/i);
  }finally{plain.close();}
 }finally{await server.close();}
});

test('workers re-validate the HTTP/2 configuration and its scheme against the target rules',async()=>{
 const {runWorker}=await import('./http-multi-worker.mjs');
 const {writeFile,mkdtemp:mk,unlink,rmdir}=await import('node:fs/promises');
 const dir=await mk(join(tmpdir(),'mw-h2-worker-')),path=join(dir,'w.private.json');
 try{
  const never={prepare:()=>{throw new Error('reached');},send(){},onMessage(){}};
  for(const [data,message] of [
   [{base:'http://127.0.0.1:9',h2:{mode:'tls'}},/HTTP\/2 configuration/],
   [{base:'http://127.0.0.1:9',h2:{mode:'tls',rigOnlyInsecureTlsNoVerify:true}},/Invalid worker target/],
   [{base:'https://127.0.0.1:9',h2:{mode:'h2c'}},/Invalid worker target/],
   [{base:'https://8.8.8.8:9',h2:{mode:'tls',rigOnlyInsecureTlsNoVerify:true}},/Invalid worker target/],
   [{base:'https://10.0.0.5:9',targets:['https://10.0.0.5:9'],h2:{mode:'tls',rigOnlyInsecureTlsNoVerify:true}},/Invalid worker target/],
  ]){
   await writeFile(path,JSON.stringify({schema:1,workerIndex:0,engineMode:'native',principals:[{}],...data}),{mode:0o600});
   await assert.rejects(runWorker(path,never),message,JSON.stringify(data));
   await unlink(path);
  }
 }finally{await rmdir(dir,{recursive:true}).catch(()=>{});}
});

test('the Markdown report states protocol, connections and refusals of HTTP/2 stages only',async()=>{
 const {markdownReport}=await import('./http-load.mjs');
 const stage={phase:'offered-rate',endpoint:'mixed',concurrency:4,arrivalRate:100,requests:1,accepted:1,requestsPerSecond:1,acceptedRequestsPerSecond:1,latencyMs:{p50:1,p95:1,p99:1},errors:0,throttled:0,timeouts:0};
 const base={target:'t',scenario:'s',runtime:'r',model:'m',stages:[stage]};
 assert.ok(!markdownReport(base).includes('HTTP/2 stages'));
 const text=markdownReport({...base,stages:[{...stage,http2:{protocol:'h2c',preset:'proxy-like',connectionsOpened:4,reconnects:1,serverMaxConcurrentStreams:32,queuedStreams:7,goaway:1,rstStreamTotal:2,refusedStreams:2}}]});
 assert.match(text,/h2c, proxy-like, 4 connections opened \(1 reconnects\), server stream limit 32, 7 streams queued/);
});
