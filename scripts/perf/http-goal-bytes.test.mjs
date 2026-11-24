import test from 'node:test';
import assert from 'node:assert/strict';
import {createServer} from 'node:http';
import {gzipSync} from 'node:zlib';
import http2 from 'node:http2';
import {qualifiesHttpTarget} from './http-multi-controller.mjs';
import {goalFromEnv,goalOption,judgedAgainst,withinNetworkBudget} from './http-goal.mjs';
import {campaignOptions} from './http-campaign.mjs';
import {measure,markdownReport,newBytesProfile,addResponseSize,addCompression,bytesSnapshot,mergeBytesSnapshots,bytesReport} from './http-load.mjs';

function eligibleStage(extra={}){
 const eligible={endpoint:'mixed',phase:'offered-rate',arrivalRate:40000,acceptedDuringScheduledWindowRps:40000,uniquePrincipalsAcceptedDuringScheduledWindow:10000,uniqueClientsAcceptedDuringScheduledWindow:10000,coordination:{valid:true},mutationStateTainted:false,scheduledSeconds:30,errors:0,timeouts:0,throttled:0,droppedCapacity:0,droppedScheduler:0};
 const health={submitted:20,applied:20,dropped:0,depth:0,queueDepth:0,capacity:8192,running:true,closing:false};
 const publication={running:true,poisoned:false,requiresFullRebuild:false,pendingJobs:0,pendingBytes:0,queuedJobs:0,submittedJobs:20,completedJobs:20,failedJobs:0,publishedBatches:8,publishedChanges:20,revalidations:0,pendingRepairMessages:0,erasingAccounts:0,maxPendingJobs:128,maxPendingBytes:16777216,maxBatchDocuments:4096};
 eligible.diagnostics={before:{httpTelemetry:health,searchIndex:{publication}},after:{httpTelemetry:{...health,submitted:40,applied:40},searchIndex:{publication:{...publication,submittedJobs:40,completedJobs:40}}}};
 return {...eligible,arrivalRate:100000,acceptedDuringScheduledWindowRps:100000,...extra};
}

test('the standing target is the default and campaign options stay unchanged without MW_LOAD_TARGET_*',()=>{
 const goal=goalFromEnv({});
 assert.deepEqual([goal.successfulRps,goal.simultaneousAuthenticatedUsers,goal.isDefault,goal.networkGbit],[100000,10000,true,undefined]);
 assert.deepEqual(goalOption({}),{});
 assert.equal(campaignOptions({}).goal,undefined);
 assert.equal(qualifiesHttpTarget({stages:[eligibleStage()],mutationStateTainted:false},true),true);
 assert.equal(qualifiesHttpTarget({stages:[eligibleStage({acceptedDuringScheduledWindowRps:99999})],mutationStateTainted:false},true),false);
 assert.equal(qualifiesHttpTarget({stages:[eligibleStage({uniquePrincipalsAcceptedDuringScheduledWindow:9999})],mutationStateTainted:false},true),false);
 assert.equal(judgedAgainst(goal,{loopback:true}).default,true);
 assert.equal(judgedAgainst(goal,{loopback:true}).note,undefined);
});

test('non-default targets replace the literals, are validated, and are announced',()=>{
 const options=campaignOptions({MW_LOAD_TARGET_RPS:'20000',MW_LOAD_TARGET_USERS:'5000'});
 assert.equal(options.goal.successfulRps,20000);
 assert.equal(options.goal.isDefault,false);
 const low=eligibleStage({arrivalRate:20000,acceptedDuringScheduledWindowRps:20000,uniquePrincipalsAcceptedDuringScheduledWindow:5000,uniqueClientsAcceptedDuringScheduledWindow:5000});
 assert.equal(qualifiesHttpTarget({options,stages:[low],mutationStateTainted:false},true),true);
 assert.equal(qualifiesHttpTarget({options,stages:[{...low,acceptedDuringScheduledWindowRps:19999}],mutationStateTainted:false},true),false);
 assert.equal(qualifiesHttpTarget({stages:[{...low,acceptedDuringScheduledWindowRps:25000}],mutationStateTainted:false},true),false,'25,000/s does not meet the standing 100,000');
 const high=campaignOptions({MW_LOAD_TARGET_RPS:'200000'});
 assert.equal(qualifiesHttpTarget({options:high,stages:[eligibleStage()],mutationStateTainted:false},true),false);
 const judged=judgedAgainst(options.goal,{loopback:false});
 assert.equal(judged.default,false);
 assert.match(judged.note,/NON-DEFAULT TARGET.*20000 requests\/s and 5000 users.*100000 \/ 10000/);
 const report={judgedAgainst:judged,target:'t',scenario:'s',runtime:'r',model:'m',stages:[]};
 assert.equal(Object.keys(report)[0],'judgedAgainst');
 assert.match(markdownReport(report).split('\n')[0],/^# HTTP load report - NON-DEFAULT TARGET: 20000 requests\/s, 5000 users$/);
 assert.equal(markdownReport({...report,judgedAgainst:judgedAgainst(goalFromEnv({}),{loopback:false})}).split('\n')[0],'# HTTP load report');
 for(const env of [{MW_LOAD_TARGET_RPS:'0'},{MW_LOAD_TARGET_RPS:'1.5'},{MW_LOAD_TARGET_RPS:'x'},{MW_LOAD_TARGET_USERS:'10001'},{MW_LOAD_NETWORK_GBIT:'0'},{MW_LOAD_NETWORK_GBIT:'abc'}])assert.throws(()=>goalFromEnv(env),/Invalid/,JSON.stringify(env));
});

test('network budget: required for a qualification run, optional otherwise, and loopback cannot qualify a network claim',()=>{
 assert.throws(()=>goalFromEnv({MW_LOAD_QUALIFICATION_RUN:'1'}),/MW_LOAD_NETWORK_GBIT/);
 const stage=gbit=>eligibleStage({transfer:{networkGbitBodyBytes:gbit}});
 assert.equal(qualifiesHttpTarget({stages:[eligibleStage()],mutationStateTainted:false},true),true);
 const budget=campaignOptions({MW_LOAD_NETWORK_GBIT:'10'});
 assert.equal(qualifiesHttpTarget({options:budget,stages:[stage(9.9)],mutationStateTainted:false},true),true);
 assert.equal(qualifiesHttpTarget({options:budget,stages:[stage(10.1)],mutationStateTainted:false},true),false);
 assert.equal(qualifiesHttpTarget({options:budget,stages:[eligibleStage()],mutationStateTainted:false},true),false,'a declared budget needs measured bytes');
 assert.equal(withinNetworkBudget(stage(10),budget.goal),true);
 const run=campaignOptions({MW_LOAD_NETWORK_GBIT:'10',MW_LOAD_QUALIFICATION_RUN:'1'});
 assert.equal(qualifiesHttpTarget({options:run,stages:[stage(1)],mutationStateTainted:false},true,{loopback:false}),true);
 assert.equal(qualifiesHttpTarget({options:run,stages:[stage(1)],mutationStateTainted:false},true,{loopback:true}),false);
 assert.equal(judgedAgainst(run.goal,{loopback:true}).network.qualifies,false);
 assert.match(judgedAgainst(run.goal,{loopback:true}).network.reason,/loopback or shared network namespace/);
 assert.equal(judgedAgainst(run.goal,{loopback:false}).network.qualifies,true);
 assert.equal(judgedAgainst(goalFromEnv({}),{loopback:false}).network.qualifies,false);
});

test('byte profile: sizes, p99 bound, request bytes, compression ratio; merging adds exactly',()=>{
 const profile=newBytesProfile();
 for(let i=0;i<99;i++){addResponseSize(profile,1000);addCompression(profile,{encoding:null,wire:1000,decoded:1000});}
 addResponseSize(profile,100000);
 addCompression(profile,{encoding:'gzip',wire:20000,decoded:100000});
 profile.requestBodyBytes=500;
 const report=bytesReport(profile,{seconds:2,windowDecodedBytes:199000});
 assert.equal(report.responseBytesPerSecond,99500);
 assert.equal(report.requestBodyBytesPerSecond,250);
 assert.ok(Math.abs(report.networkGbitBodyBytes-(99500+250)*8/1e9)<1e-12);
 assert.equal(report.responseSize.samples,100);
 assert.equal(report.responseSize.mean,1990);
 assert.ok(report.responseSize.p99UpperBound>=1000&&report.responseSize.p99UpperBound<1000*1.06);
 assert.deepEqual([report.compression.compressedResponses,report.compression.compressed,report.compression.compressedToUncompressedRatio],[1,true,.2]);
 const other=newBytesProfile();
 addResponseSize(other,50);
 addCompression(other,{encoding:'gzip',wire:null,decoded:50});
 const merged=mergeBytesSnapshots([bytesSnapshot(profile),bytesSnapshot(other)]);
 assert.equal(merged.count,101);
 assert.equal(merged.compressed,2);
 assert.equal(merged.measured,1);
 assert.equal(merged.encodings.gzip,2);
 assert.match(bytesReport(other,{seconds:1,windowDecodedBytes:50}).compression.ratioBasis,/unknown/);
 assert.equal(bytesReport(other,{seconds:1,windowDecodedBytes:50}).compression.compressedToUncompressedRatio,null);
 assert.throws(()=>mergeBytesSnapshots([{schema:1,sizes:[[1,2]],count:1,sum:1,requestBodyBytes:0,responses:1,compressed:0,measured:0,wireBytes:0,decodedBytes:0,encodings:{}}]),/Invalid byte profile/);
});

const asset=Buffer.from(JSON.stringify({ok:true,pad:'x'.repeat(3000)}));
const zipped=gzipSync(asset);

test('HTTP/1.1 stage reports consumed bytes, request bytes and compression (wire size from Content-Length)',async()=>{
 const server=createServer((req,res)=>{
  req.resume();
  if(req.url==='/gz'){res.writeHead(200,{'content-encoding':'gzip','content-type':'application/json','content-length':zipped.length});res.end(zipped);return;}
  res.writeHead(200,{'content-type':'application/json'});
  res.end(asset);
 });
 await new Promise(done=>server.listen(0,'127.0.0.1',done));
 const base='http://127.0.0.1:'+server.address().port;
 try{
  await (await fetch(base+'/p')).arrayBuffer();
  const plain=await measure({base,concurrency:4,seconds:1,arrivalRate:100,latePolicy:'dispatch',requests:[{name:'p',path:'/p',method:'POST',body:'12345',validate:v=>v.ok===true}]});
  assert.ok(plain.requests>50);
  assert.ok(Math.abs(plain.transfer.responseBytesPerSecond-plain.acceptedDuringScheduledWindow*asset.length)<asset.length*2);
  assert.ok(Math.abs(plain.transfer.requestBodyBytesPerSecond-plain.httpDispatchAttempts*5)<20);
  assert.equal(plain.transfer.responseSize.mean,asset.length);
  assert.equal(plain.transfer.compression.compressed,false);
  const gz=await measure({base,concurrency:4,seconds:1,arrivalRate:100,latePolicy:'dispatch',requests:[{name:'g',path:'/gz',validate:v=>v.ok===true}]});
  assert.equal(gz.errors,0);
  assert.equal(gz.transfer.responseSize.mean,asset.length,'validators and byte counts see the decoded body');
  assert.equal(gz.transfer.compression.compressedResponses,gz.accepted);
  assert.deepEqual(Object.keys(gz.transfer.compression.encodings),['gzip']);
  assert.ok(Math.abs(gz.transfer.compression.compressedToUncompressedRatio-zipped.length/asset.length)<1e-9);
 }finally{server.closeAllConnections();await new Promise(done=>server.close(done));}
});

test('HTTP/2 decodes gzip like fetch and reports the exact wire size',async()=>{
 const server=http2.createServer();
 server.on('stream',(stream,headers)=>{
  stream.on('error',()=>{});
  stream.resume();
  if(headers[':path']==='/gz'){stream.respond({':status':200,'content-encoding':'gzip','content-type':'application/json'});stream.end(zipped);return;}
  stream.respond({':status':200,'content-type':'application/json'});
  stream.end(asset);
 });
 await new Promise(done=>server.listen(0,'127.0.0.1',done));
 const base='http://127.0.0.1:'+server.address().port;
 try{
  const gz=await measure({base,h2:{mode:'h2c'},concurrency:4,seconds:1,arrivalRate:100,latePolicy:'dispatch',requests:[{name:'g',path:'/gz',validate:v=>v.ok===true}]});
  assert.equal(gz.errors,0);
  assert.ok(gz.accepted>50);
  assert.equal(gz.protocol,'h2c');
  assert.equal(gz.transfer.responseSize.mean,asset.length);
  assert.equal(gz.transfer.compression.ratioMeasuredResponses,gz.accepted);
  assert.ok(Math.abs(gz.transfer.compression.compressedToUncompressedRatio-zipped.length/asset.length)<1e-9);
 }finally{server.close();}
});

test('the Markdown report lists transfer, request bytes and compression per stage',()=>{
 const stage={phase:'offered-rate',endpoint:'mixed',concurrency:4,arrivalRate:100,requests:1,accepted:1,requestsPerSecond:1,acceptedRequestsPerSecond:1,latencyMs:{p50:1,p95:1,p99:1},errors:0,throttled:0,timeouts:0};
 const base={target:'t',scenario:'s',runtime:'r',model:'m',stages:[stage]};
 assert.ok(!markdownReport(base).includes('Transfer per stage'));
 const profile=newBytesProfile();
 addResponseSize(profile,2000);addCompression(profile,{encoding:'gzip',wire:500,decoded:2000});
 const text=markdownReport({...base,stages:[{...stage,transfer:bytesReport(profile,{seconds:1,windowDecodedBytes:2000})}]});
 assert.match(text,/Transfer per stage/);
 assert.match(text,/\| 1 of 1 \| 0\.250 \|/);
});
