import test from 'node:test';
import assert from 'node:assert/strict';
import {fork} from 'node:child_process';
import {mkdtemp,writeFile,readFile,unlink,rmdir,access} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import {join,resolve} from 'node:path';
import {EventEmitter} from 'node:events';
import {randomUUID} from 'node:crypto';
import {runClaimed,reportHash} from './external-control.mjs';
import {Measurements} from './http-load.mjs';
import {splitBudget,partitionPrincipals,validateStage,mergeSnapshots,aggregateWorkers} from './http-multi-protocol.mjs';
import {validateAssignment,runMultiCampaign,qualifiesHttpTarget,writeMultiArtifacts} from './http-multi-controller.mjs';
import {generatorWorkers,httpGeneratorLaunch} from './http-generator-launch.mjs';

const stage={concurrency:4,seconds:1,timeoutMs:1000,arrivalRate:100},startAtMs=100000;
const partitions=[{index:0,count:2},{index:1,count:2}];
function result(index){
 const aggregate=new Measurements(),scheduling=new Measurements();
 for(let n=0;n<50;n++){aggregate.add(index?100:1,'200',4,true);scheduling.add(1,'scheduled',0,true);}
 return {schema:1,workerIndex:index,partitionCount:2,measurement:{window:{requestedStartAtMs:startAtMs,scheduledStartAtMs:startAtMs,scheduledEndAtMs:startAtMs+1000,actualStartAtMs:startAtMs+1,completedAtMs:startAtMs+1010,startLagMs:1},rawMeasurements:{schema:1,aggregate:aggregate.snapshot(),byRequest:{'Email/query':aggregate.snapshot()},scheduling:scheduling.snapshot(),arrivalLatency:aggregate.snapshot()},concurrency:2,scheduledSeconds:1,arrivalRate:50,offered:50,droppedCapacity:0,droppedScheduler:0,acceptedDuringScheduledWindow:50,uniquePrincipalsAttempted:2,uniquePrincipalsAccepted:2,uniquePrincipalsAcceptedDuringScheduledWindow:2,uniqueClientsAttempted:2,uniqueClientsAccepted:2,uniqueClientsAcceptedDuringScheduledWindow:2,peakOutstanding:2,mutationStateTainted:false,unknownMutationOutcomes:0,taintedPrincipals:0,httpDispatchAttempts:50,blockedAfterMutationTaint:0,abortedUnscheduled:0,generator:{cpuUserMs:20,cpuSystemMs:10,rssBeforeBytes:1000,rssAfterBytes:1200,heapAfterBytes:500}}};
}
function handoff(count=4){return {base:'http://127.0.0.1:12345',engineMode:'native',backend:'sqlite',buildProfile:'release',uniquePrincipalCount:count,principals:Array.from({length:count},(_,i)=>({cookie:`mw_session=private-session-${i}; mw_csrf=safe-${i}`,csrfToken:`safe-${i}`,accountId:`private-account-${i}`,messageIds:[`private-mail-${i}-1`,`private-mail-${i}-2`],inboxId:`private-inbox-${i}`,clientAddress:`198.18.0.${i+1}`})),clientSimulation:{mode:'fixture-loopback-trusted-proxy',clientCount:count},startupReadiness:{verified:true,state:'ready',startupSeconds:1,indexWaitSeconds:0,seededMessages:count*2,expectedIndexDocuments:count*2,indexDocuments:count*2,zeroAccessExcludedMessages:0,rebuildRunning:false,rebuildDone:count*2,rebuildTotal:count*2},policyEvidence:{mode:'loaded',verified:true,key:'access.protocols',allowedProtocols:['imap'],locked:true,primedRequests:2,revokedRequests:2,revokedStatus:403,restoredRequests:2},workloadProfile:{kind:'baseline',messagesPerPrincipal:2,seededMessages:count*2,seedSeconds:1}};}

test('workers get disjoint complete bounded partitions and exact global budgets',()=>{
 const original=Array.from({length:10000},(_,i)=>({secret:`private-${i}`})),parts=partitionPrincipals(original,16);
 assert.equal(parts.reduce((n,p)=>n+p.count,0),10000);assert.equal(new Set(parts.flatMap(p=>p.principals)).size,10000);
 assert.deepEqual(splitBudget(100,3),[34,33,33]);assert.equal(splitBudget(4096,16).reduce((a,b)=>a+b),4096);
 assert.deepEqual(validateStage({workers:3,concurrency:10,seconds:30,timeoutMs:1000,arrivalRate:40000}),{concurrency:[4,3,3],rates:[13334,13333,13333]});
 for(const patch of [{workers:17},{concurrency:1},{concurrency:4097},{seconds:.5},{seconds:61},{arrivalRate:1},{arrivalRate:100001}])assert.throws(()=>validateStage({...stage,workers:2,...patch}));
});
test('optional launch integration preserves the original single driver and bounds multiprocess selection',()=>{
 const input={directory:'/owned/private',ready:{handoffFile:'handoff.private.json',outputFile:'report.json'}};
 const single=httpGeneratorLaunch(input),multi=httpGeneratorLaunch({...input,workers:4});
 assert.ok(single.script.endsWith('http-real-fixture.mjs'));assert.equal(single.args.length,2);assert.equal(single.workers,1);
 assert.ok(multi.script.endsWith('http-multi-controller.mjs'));assert.equal(multi.args.length,3);assert.equal(multi.args[2],'--workers=4');
 assert.ok(multi.args[0].startsWith('--handoff='));assert.ok(multi.args[1].startsWith('--output='));
 for(const invalid of [0,17,1.5,'bad',''])assert.throws(()=>generatorWorkers(invalid));
});
test('weighted histograms and shared window counts merge without percentile averaging or leaking identities',()=>{
 const merged=aggregateWorkers({results:[result(1),result(0)],partitions:partitions.map(p=>({...p,privateCookie:'do-not-publish'})),stage,startAtMs});
 assert.equal(merged.requests,100);assert.equal(merged.acceptedDuringScheduledWindowRps,100);assert.equal(merged.latencyMs.p95,100);assert.equal(merged.latencyMs.mean,50.5);
 assert.equal(merged.uniquePrincipalsAccepted,4);assert.equal(merged.uniqueClientsAccepted,4);assert.equal(merged.generator.cpuUserMs,40);assert.equal(merged.generator.rssAfterBytes,2400);assert.equal(merged.offered,100);assert.equal(merged.concurrency,4);
 assert.ok(!JSON.stringify(merged).includes('do-not-publish'));assert.ok(!JSON.stringify(merged).includes('private-'));
});
test('missing, duplicate, skewed, malformed or overstated worker results cannot qualify',()=>{
 assert.throws(()=>aggregateWorkers({results:[result(0)],partitions,stage,startAtMs}));
 assert.throws(()=>aggregateWorkers({results:[result(0),result(0)],partitions,stage,startAtMs}));
 const patches=[r=>{r.partitionCount=3;},r=>{r.measurement.window.startLagMs=51;},r=>{r.measurement.window.scheduledEndAtMs++;},r=>{r.measurement.rawMeasurements.aggregate.histogram[0][1]--;},r=>{r.measurement.rawMeasurements.aggregate.accepted=51;},r=>{r.measurement.rawMeasurements.aggregate.timeouts=1;},r=>{r.measurement.uniquePrincipalsAccepted=3;},r=>{r.measurement.offered=51;},r=>{r.measurement.rawMeasurements.byRequest={'private-secret':r.measurement.rawMeasurements.aggregate};},r=>{r.measurement.rawMeasurements.byRequest['Email/query'].errors=1;}];
 for(const patch of patches){const rows=[result(0),result(1)];patch(rows[0]);assert.throws(()=>aggregateWorkers({results:rows,partitions,stage,startAtMs}));}
});
test('an early mutation abort retains counts and a later taint invalidates earlier eligible stages',()=>{
 const aborted=result(0),aggregate=new Measurements(),scheduling=new Measurements();
 for(let i=0;i<10;i++){aggregate.add(1,i===9?'network':'200',i===9?0:4,i!==9,i===9);scheduling.add(1,'scheduled',0,true);}
 Object.assign(aborted.measurement,{mutationStateTainted:true,unknownMutationOutcomes:1,taintedPrincipals:1,httpDispatchAttempts:10,abortedUnscheduled:40,offered:10,acceptedDuringScheduledWindow:9});
 aborted.measurement.window.completedAtMs=startAtMs+500;
 aborted.measurement.rawMeasurements={schema:1,aggregate:aggregate.snapshot(),byRequest:{'Email/query':aggregate.snapshot()},scheduling:scheduling.snapshot(),arrivalLatency:aggregate.snapshot()};
 const merged=aggregateWorkers({results:[aborted,result(1)],partitions,stage,startAtMs});
 assert.equal(merged.mutationStateTainted,true);assert.equal(merged.unknownMutationOutcomes,1);assert.equal(merged.abortedUnscheduled,40);assert.equal(merged.offered,60);assert.equal(merged.plannedArrivals,100);assert.equal(merged.requests,60);assert.equal(merged.errors,1);
 const eligible={endpoint:'mixed',phase:'offered-rate',arrivalRate:40000,acceptedDuringScheduledWindowRps:40000,uniquePrincipalsAcceptedDuringScheduledWindow:10000,uniqueClientsAcceptedDuringScheduledWindow:10000,coordination:{valid:true},mutationStateTainted:false,scheduledSeconds:30,errors:0,timeouts:0,throttled:0,droppedCapacity:0,droppedScheduler:0};
 assert.equal(qualifiesHttpTarget({stages:[eligible],mutationStateTainted:false},true),true);
 assert.equal(qualifiesHttpTarget({stages:[eligible],mutationStateTainted:true},true),false);
 assert.equal(qualifiesHttpTarget({stages:[eligible],mutationStateTainted:false},false),false);
});
test('raw histogram merge uses the same strict validator as the actual generator',()=>{
 const raw=result(0).measurement.rawMeasurements.aggregate;assert.equal(mergeSnapshots([raw,raw],1).requests,100);
 for(const patch of [{histogram:[[1001,50]]},{histogram:[[1,49]]},{accepted:51},{timeouts:1},{statuses:{'private-cookie':50}},{totalMs:0,maximumMs:1}])assert.throws(()=>mergeSnapshots([{...raw,...patch}],1));
});
test('global seed and loaded-policy proof stay global while partitions remain private',()=>{
 const value=handoff();const proof=validateAssignment(value,2);
 const writer={requestedWorkers:1,indexingWorkers:1,memoryBudgetBytes:50000000};
 const storage={mode:'sealed-disk',encryptedAtRest:true,verified:true,proof:'sealed-constructor-and-encrypted-metadata'};
 assert.deepEqual(validateAssignment({...value,searchStorage:{...storage,path:'/private'}},2).searchStorage,storage);
 assert.equal(validateAssignment(value,2).searchStorage,null);
 assert.throws(()=>validateAssignment({...value,searchStorage:{...storage,encryptedAtRest:false}},2));
 assert.deepEqual(validateAssignment({...value,searchWriter:{...writer,cookie:'private'}},2).searchWriter,writer);
 assert.throws(()=>validateAssignment({...value,searchWriter:{...writer,indexingWorkers:4}},2));
 assert.equal(proof.startupReadiness.seededMessages,8);assert.equal(proof.workloadProfile.seededMessages,8);assert.equal(proof.policyEvidence.verified,true);assert.deepEqual(proof.partitions.map(p=>p.count),[2,2]);
 for(const patch of [{base:'https://example.test'},{uniquePrincipalCount:5},{workloadProfile:{...value.workloadProfile,seededMessages:6}}])assert.throws(()=>validateAssignment({...value,...patch},2));
 const duplicate=handoff();duplicate.principals[2].cookie=duplicate.principals[0].cookie;assert.throws(()=>validateAssignment(duplicate,2));
 const messages=handoff();messages.principals[0].messageIds.pop();assert.throws(()=>validateAssignment(messages,2));
});
test('controller rejects a crashed startup worker and confirms cleanup of its exact private files',async()=>{
 const paths=[],children=[];
 const launch=(_script,args)=>{
  const child=new EventEmitter();child.connected=true;child.send=()=>{};child.kill=()=>{child.connected=false;queueMicrotask(()=>child.emit('exit',1));};paths.push(args[0]);children.push(child);
  setImmediate(()=>child.emit('exit',1));return child;
 };
 const options={concurrency:[2],rates:[100],seconds:1,timeoutMs:1000};
 await assert.rejects(runMultiCampaign({handoff:handoff(),workers:2,options,launch,isolationProbe:async()=>({verified:true,status:'verified',requests:2,directions:2})}),/Worker/);
 assert.equal(children.length,2);for(const path of paths)await assert.rejects(access(path));
});
test('actual worker subprocess rejects invalid private assignment without network or secret output',async()=>{
 const directory=await mkdtemp(join(tmpdir(),'mw-worker-invalid-')),path=join(directory,'invalid.private.json');
 await writeFile(path,JSON.stringify({schema:1,token:'private-control-secret'}),{mode:0o600});
 try{
  const child=fork(resolve('scripts/perf/http-multi-worker.mjs'),[path],{stdio:['ignore','pipe','pipe','ipc'],execArgv:[]}),messages=[];let output='';
  child.on('message',message=>messages.push(message));child.stdout.on('data',chunk=>output+=chunk);child.stderr.on('data',chunk=>output+=chunk);
  const code=await new Promise((resolve,reject)=>{const timer=setTimeout(()=>{child.kill('SIGKILL');reject(new Error('worker did not stop'));},5000);child.once('exit',code=>{clearTimeout(timer);resolve(code);});child.once('error',reject);});
  assert.equal(code,1);assert.equal(messages[0].type,'failed');assert.ok(!JSON.stringify(messages).includes('private-control-secret'));assert.ok(!output.includes('private-control-secret'));
 }finally{await unlink(path);await rmdir(directory);}
});
test('two actual subprocesses obey barrier, exact allocation and global proof without sending HTTP',async()=>{
 const options={mode:'smoke',concurrency:[2],rates:[100],seconds:1,timeoutMs:1000,endpoints:['query'],warmupSeconds:0,deadlineSeconds:10,maxErrorRate:0,maxThrottleRate:0,maxP99Ms:100,allowOverload:false,adaptive:true};
 const launch=(_script,args,options)=>fork(resolve('scripts/perf/test-fixtures/http-multi-protocol-child.mjs'),args,options);
 const report=await runMultiCampaign({handoff:handoff(),workers:2,options,launch,startLeadMs:500,scannerProbe:async()=>403,isolationProbe:async()=>({verified:true,status:'verified',requests:2,directions:2})});
 assert.equal(report.scannerProbeStatus,403);assert.equal(report.stages.length,1);assert.equal(report.stages[0].offered,100);assert.equal(report.stages[0].acceptedDuringScheduledWindow,100);assert.equal(report.stages[0].uniquePrincipalsAccepted,4);
 assert.equal(report.stages[0].coordination.valid,true);assert.deepEqual(report.partitionSizes,[2,2]);assert.equal(report.startupReadiness.seededMessages,8);assert.equal(report.workloadProfile.seededMessages,8);assert.equal(report.policyEvidence.verified,true);assert.equal(report.capacityTarget.met,false);
 assert.ok(!JSON.stringify(report).includes('private-'));
 const artifactDir=await mkdtemp(join(tmpdir(),'mw-multi-artifacts-'));
 try{
  await writeFile(join(artifactDir,'handoff.private.json'),JSON.stringify(handoff()),{mode:0o600});
  const ready={version:1,runId:randomUUID(),expiresAtMs:Date.now()+60000,deadlineSeconds:10,handoffFile:'handoff.private.json',outputFile:'report.json',campaignEnv:{}};
  await runClaimed({directory:artifactDir,ready,execute:async()=>{await writeMultiArtifacts(join(artifactDir,'report.json'),report);return 0;}});
  const jsonBytes=await readFile(join(artifactDir,'report.json')),markdownBytes=await readFile(join(artifactDir,'report.md')),done=JSON.parse(await readFile(join(artifactDir,'done.json'),'utf8'));
  assert.equal(done.status,'completed');assert.equal(done.reportHash,reportHash(jsonBytes));assert.equal(done.markdownHash,reportHash(markdownBytes));
  const markdown=markdownBytes.toString('utf8');assert.match(markdown,/# HTTP load report/);assert.match(markdown,/not measured/);assert.ok(!markdown.includes('undefined'));assert.ok(!markdown.includes('private-'));
  const exported=JSON.parse(jsonBytes);for(const key of ['target','scenario','runtime','model','protection'])assert.equal(typeof exported[key],'string');assert.equal(exported.uniqueAuthenticatedPrincipals,4);
 }finally{for(const name of ['handoff.private.json','report.json','report.md','heartbeat.json','done.json'])await unlink(join(artifactDir,name));await rmdir(artifactDir);}
 await assert.rejects(runMultiCampaign({handoff:handoff(),workers:2,options,launch,startLeadMs:500,scannerProbe:async()=>200,isolationProbe:async()=>({verified:true,status:'verified',requests:2,directions:2})}),/scanner enforcement/);
 let launched=false;
 await assert.rejects(runMultiCampaign({handoff:handoff(),workers:2,options,launch:()=>{launched=true;throw new Error('unexpected launch');},startLeadMs:500,isolationProbe:async()=>({verified:false,status:'failed',requests:2})}),/account isolation/);
 assert.equal(launched,false,'failed isolation must stop before launching traffic workers');
});
