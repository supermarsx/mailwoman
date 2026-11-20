import test from 'node:test';
import assert from 'node:assert/strict';
import {createServer} from 'node:http';
import {representativeSpecs,workloadProfileReport,attachmentPatternValidator,attachmentPatternCacheStats} from './representative-workload.mjs';
import {DEFAULT_HEADER_PROPERTIES,READER_PROPERTIES,READER_FETCH_OPTIONS,fixturePropertyProfile} from './fixture-properties.mjs';
import {measure} from './http-load.mjs';
import {runCampaign} from './http-campaign.mjs';

const principal={accountId:'private-account',messageIds:Array.from({length:50},(_,i)=>`private-message-${i}`),inboxId:'private-inbox',archiveId:'private-archive',attachmentBlobId:'private-message-0.2'};

test('attachment pattern uses native exact comparison and shares at most four bounded golden buffers',()=>{
 for(const size of [65536,262144]){
  const validate=attachmentPatternValidator(size),good=Buffer.alloc(size,'ABCD');
  assert.equal(validate(good),true);
  for(const offset of [0,Math.floor(size/2),size-1]){const bad=Buffer.from(good);bad[offset]^=1;assert.equal(validate(bad),false);}
  assert.equal(validate(good.subarray(1)),false);
 }
 const good=Buffer.alloc(65536,'ABCD'),before=attachmentPatternCacheStats().allocations;
 for(let principal=0;principal<10000;principal++)assert.equal(attachmentPatternValidator(65536)(good),true);
 assert.ok(attachmentPatternCacheStats().allocations-before<=1,'ten thousand validators must share the same size buffer');
 for(const size of [4,5,6,7,8,9])assert.equal(attachmentPatternValidator(size)(Buffer.alloc(size,'ABCD')),true);
 const stats=attachmentPatternCacheStats();assert.ok(stats.entries<=4);assert.ok(stats.bytes<=4*262144);
 for(const size of [3,262145,NaN,4.5])assert.throws(()=>attachmentPatternValidator(size));
});
test('workload metadata is bounded, internally consistent and aggregate only',()=>{
 const proof={kind:'representative',messagesPerPrincipal:50,seededMessages:500000,seedSeconds:30,bodySampleBytes:4096,attachmentSampleBytes:65536,ordinaryBodyPayloadBytes:26,sendsMeasured:false,upstreamNetworkMeasured:false};
 assert.deepEqual(workloadProfileReport({...proof,cookie:'private-secret'},10000),{...proof,requestPropertySets:fixturePropertyProfile('representative')});
 const baseline=workloadProfileReport({kind:'baseline',messagesPerPrincipal:2,seededMessages:4,seedSeconds:1},2);
 assert.deepEqual(baseline.requestPropertySets,fixturePropertyProfile('baseline'));
 for(const patch of [{kind:'unknown'},{messagesPerPrincipal:101},{seededMessages:500001},{seedSeconds:NaN},{kind:'baseline'}])assert.throws(()=>workloadProfileReport({...proof,...patch},10000));
});
function projection(spec,p=principal){
 const calls=JSON.parse(spec.body).methodCalls;
 let seen=false,folder=p.inboxId;const results={};
 const methodResponses=calls.map(([method,args,tag])=>{
  let result={accountId:p.accountId};
  if(method==='Email/query')Object.assign(result,{ids:p.messageIds.slice(args.position,args.position+args.limit),position:args.position,total:p.messageIds.length});
  if(method==='Email/set'){
   const patch=args.update[p.messageIds[0]];
   if(Object.hasOwn(patch,'keywords/$seen'))seen=patch['keywords/$seen']===true;
   for(const [key,value]of Object.entries(patch))if(key.startsWith('mailboxIds/')&&value===true)folder=key.slice(11);
   result.updated={[p.messageIds[0]]:null};result.notUpdated={};
  }
  if(method==='Email/get'){
   const ids=args.ids??results[args['#ids'].resultOf].ids;
   result.list=ids.map(id=>({id,keywords:seen?{$seen:true}:{},mailboxIds:{[folder]:true},bodyValues:{1:{value:'Representative body sample.'}},attachments:[{name:'sample.txt',blobId:p.attachmentBlobId}]}));
   result.notFound=[];
  }
  results[tag]=result;return [method,result,tag];
 });
 return {methodResponses};
}

test('representative workload validates pages, actual bodies/attachment bytes, mutations and restoration',()=>{
 const specs=representativeSpecs(principal);
 assert.equal(specs.length,5);
 for(const spec of specs){
  assert.equal(spec.serializePrincipal,true);
  if(spec.validateBytes){assert.equal(spec.validateBytes(Buffer.from('ABCD')),true);assert.equal(spec.validateBytes(Buffer.from('wrong')),false);continue;}
  const response=projection(spec);assert.equal(spec.validate(response),true,spec.name);
  response.methodResponses[0][1].accountId='foreign';assert.equal(spec.validate(response),false,spec.name);
 }
 const pages=specs[0],bad=projection(pages);bad.methodResponses[2][1].ids=bad.methodResponses[0][1].ids;assert.equal(pages.validate(bad),false);
 const mark=specs[3],notUpdated=projection(mark);delete notUpdated.methodResponses[0][1].updated[principal.messageIds[0]];assert.equal(mark.validate(notUpdated),false);
 const move=specs[4],notMoved=projection(move);notMoved.methodResponses[1][1].list[0].mailboxIds={[principal.inboxId]:true};assert.equal(move.validate(notMoved),false);
 assert.throws(()=>representativeSpecs({...principal,archiveId:principal.inboxId}));
 assert.throws(()=>representativeSpecs({...principal,attachmentBlobId:null}));
});

test('representative page and reader requests use the full default webmail property sets',()=>{
 const specs=representativeSpecs(principal),page=JSON.parse(specs[0].body).methodCalls;
 for(const call of page.filter(call=>call[0]==='Email/get'))assert.deepEqual(call[1].properties,DEFAULT_HEADER_PROPERTIES);
 const reader=JSON.parse(specs[1].body).methodCalls[0][1];
 assert.deepEqual(reader.properties,READER_PROPERTIES);
 for(const [key,value]of Object.entries(READER_FETCH_OPTIONS))assert.equal(reader[key],value);
 assert.equal(reader.fetchAllBodyValues,undefined);
});

test('body validation requires the configured exact payload, accepting only one MIME boundary ending',()=>{
 const configured={...principal,bodySampleBytes:4096},spec=representativeSpecs(configured)[1];
 const correct='Representative body sample.'+'x'.repeat(4096-27);
 for(const ending of ['', '\n', '\r\n']){
  const response=projection(spec);response.methodResponses[0][1].list[0].bodyValues[1].value=correct+ending;
  assert.equal(spec.validate(response),true);
 }
 for(const body of [correct.slice(0,-1),correct+'x',correct+'\r\n\r\n','prefix'+correct,correct.replace(/x$/,'z')]){
  const response=projection(spec);response.methodResponses[0][1].list[0].bodyValues[1].value=body;
  assert.equal(spec.validate(response),false);
 }
 const truncated=projection(spec);truncated.methodResponses[0][1].list[0].bodyValues[1]={value:correct,isTruncated:true};assert.equal(spec.validate(truncated),false);
});

test('real representative HTTP requests serialize one user and report actual HTTP requests only',async()=>{
 const specs=representativeSpecs(principal).map(spec=>({...spec,principalIndex:0,clientIndex:0}));
 let active=0,maximum=0,requests=0;
 const server=createServer(async(req,res)=>{
  maximum=Math.max(maximum,++active);requests++;
  const chunks=[];for await(const chunk of req)chunks.push(chunk);
  await new Promise(resolve=>setTimeout(resolve,5));
  const spec=specs.find(spec=>spec.path===req.url&&spec.body===Buffer.concat(chunks).toString());
  if(req.method==='GET'){res.end('ABCD');}else if(spec){res.setHeader('content-type','application/json');res.end(JSON.stringify(projection(spec)));}else{res.statusCode=400;res.end('{}');}
  active--;
 });
 await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
 try{
  const report=await measure({base:`http://127.0.0.1:${server.address().port}`,requests:specs,concurrency:8,seconds:.2,timeoutMs:2000});
  assert.equal(report.errors,0);assert.equal(maximum,1);assert.equal(report.requests,requests);assert.equal(report.accepted,requests);assert.equal(report.uniquePrincipalsAccepted,1);
  assert.equal(Object.keys(report.byRequest).length,5);
  assert.ok(!JSON.stringify(report).includes('private-'));
 }finally{server.closeAllConnections();await new Promise(resolve=>server.close(resolve));}
});

test('representative binary downloads validate 64KiB and bounded larger payloads without the JSON cap',async()=>{
 let size=65536;
 const server=createServer((_req,res)=>{const bytes=Buffer.alloc(size);for(let i=0;i<size;i++)bytes[i]=[65,66,67,68][i%4];res.end(bytes);});
 await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
 try{
  for(size of [65536,262144]){
   const spec=representativeSpecs({...principal,attachmentSampleBytes:size})[2];
   const result=await measure({base:`http://127.0.0.1:${server.address().port}`,requests:[{...spec,principalIndex:0}],concurrency:1,seconds:.05});
   assert.equal(result.errors,0);assert.ok(result.accepted>0);assert.equal(result.bytes,result.requests*size);
  }
 }finally{server.closeAllConnections();await new Promise(resolve=>server.close(resolve));}
});

test('an actual timed-out server mutation taints the campaign and blocks queued reuse and later clean stages',async()=>{
 let mutations=0,cleanRequests=0,changed=false,finishMutation;
 const finished=new Promise(resolve=>{finishMutation=resolve;});
 const server=createServer(async(req,res)=>{
  if(req.url==='/mutation'){
   mutations++;await new Promise(resolve=>setTimeout(resolve,150));changed=true;
   res.end('{"ok":true}');finishMutation();
  }else{cleanRequests++;res.end('{"ok":true}');}
 });
 await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
 try{
  const mutation={name:'mutation',path:'/mutation',method:'POST',principalIndex:0,serializePrincipal:true,statefulMutation:true,validate:value=>value.ok};
  const clean={name:'clean',path:'/clean',principalIndex:0,serializePrincipal:true,validate:value=>value.ok};
  const result=await runCampaign({base:`http://127.0.0.1:${server.address().port}`,groups:{representative:[mutation,clean],identity:[clean]},options:{mode:'ramp',seconds:.3,warmupSeconds:0,deadlineSeconds:10,timeoutMs:40,concurrency:[4,8],endpoints:['representative','identity'],rates:[100],adaptive:false,maxErrorRate:1,maxThrottleRate:1,maxP99Ms:10000,allowOverload:true}});
  await finished;
  assert.equal(changed,true,'server mutation finishes after the client times out');
  assert.equal(mutations,1);assert.equal(cleanRequests,0,'tainted queued requests and later clean phases must not dispatch');
  assert.equal(result.mutationStateTainted,true);assert.equal(result.stages.length,1);
  const stage=result.stages[0];assert.equal(stage.mutationStateTainted,true);assert.equal(stage.unknownMutationOutcomes,1);assert.equal(stage.taintedPrincipals,1);assert.equal(stage.httpDispatchAttempts,1);assert.equal(stage.blockedAfterMutationTaint,3);assert.equal(stage.accepted,0);
  assert.ok(result.decisions.some(decision=>decision.reason==='unknown-mutation-outcome'));
  assert.ok(!JSON.stringify(result).includes('private-'));
 }finally{server.closeAllConnections();await new Promise(resolve=>server.close(resolve));}
});
