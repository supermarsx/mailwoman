import test from 'node:test';
import assert from 'node:assert/strict';
import {createServer} from 'node:http';
import {prepareFixtureClients,fixturePrincipalIndex,startupReadinessReport,fixturePolicyReport,searchWriterReport,linkedLoginPolicyProbeReport,searchStorageReport,qualifiesSearchStorage} from './fixture-clients.mjs';
import {measure,markdownReport} from './http-load.mjs';
import {searchTimingReport,fixtureDiagnosticsReport} from './fixture-clients.mjs';
import {httpTelemetryReport,qualifiesHttpTelemetry} from './fixture-clients.mjs';

test('HTTP telemetry loss is visible and cannot qualify a new capacity result',()=>{
 const value={submitted:20,applied:18,dropped:0,depth:2,queueDepth:1,capacity:8192,running:true,closing:false};
 assert.deepEqual(httpTelemetryReport({...value,path:'/private',account:'private'}),value);
 assert.equal(httpTelemetryReport(undefined),null);
 assert.equal(qualifiesHttpTelemetry({}),false);
 assert.equal(qualifiesHttpTelemetry({before:{httpTelemetry:value},after:{httpTelemetry:{...value,submitted:30}}}),true);
 for(const patch of [{dropped:1},{running:false},{closing:true},{capacity:0}])assert.equal(qualifiesHttpTelemetry({before:{httpTelemetry:value},after:{httpTelemetry:{...value,...patch}}}),false);
 for(const patch of [{depth:-1},{capacity:8193},{queueDepth:8193},{dropped:NaN},{running:'true'}])assert.throws(()=>httpTelemetryReport({...value,...patch}));
 assert.deepEqual(fixtureDiagnosticsReport({httpTelemetry:{...value,secret:'private'}}).httpTelemetry,value);
});

test('search timing diagnostics are bounded aggregate fields and legacy remains unreported',()=>{
 const names=['writerLockWait','commit','readerReload','sealedRead','sealedDecode','sealedEncode','sealedWrite','directorySync'];
 const phases=Object.fromEntries(names.map(name=>[name,{calls:2,failures:1,totalMicros:20,maxMicros:15,bytes:12}]));
 const value={scope:'process',enabled:true,phases};
 assert.deepEqual(searchTimingReport({...value,secret:'private',phases:{...phases,privatePath:{}}}),value);
 assert.equal(searchTimingReport(undefined),null);
 assert.equal(searchTimingReport({...value,phases:{...phases,commit:{...phases.commit,failures:3,maxMicros:21}}}).phases.commit.failures,3); // Independently sampled atomics may cross during concurrent publication.
 const legacy={searchIndex:{commitAttempts:2}};assert.equal(fixtureDiagnosticsReport(legacy),legacy);
 assert.deepEqual(fixtureDiagnosticsReport({searchIndex:{timings:{...value,secret:'private'}}}).searchIndex.timings,value);
 for(const patch of [{calls:-1},{failures:-1},{totalMicros:NaN},{bytes:Number.MAX_SAFE_INTEGER+1},{maxMicros:Infinity}])assert.throws(()=>searchTimingReport({...value,phases:{...phases,commit:{...phases.commit,...patch}}}));
 for(const patch of [{enabled:'true'},{scope:'account'},{phases:{}}])assert.throws(()=>searchTimingReport({...value,...patch}));
});

const principals=()=>Array.from({length:4},(_,index)=>({cookie:`mw_session=private-session-${index}; mw_csrf=fixture-csrf-${index}`,accountId:`private-account-${index}`,clientAddress:`198.18.0.${index+1}`}));
const simulation={mode:'fixture-loopback-trusted-proxy',clientCount:4};

test('storage proof reports only verified aggregate mode and excludes RAM or legacy from qualification',()=>{
 const sealed={mode:'sealed-disk',encryptedAtRest:true,verified:true,proof:'sealed-constructor-and-encrypted-metadata'};
 const memory={mode:'memory',encryptedAtRest:false,verified:true,proof:'explicit-in-memory-constructor'};
 assert.deepEqual(searchStorageReport({...sealed,key:'private',path:'/private'}),sealed);
 assert.equal(searchStorageReport(undefined),null);
 assert.equal(qualifiesSearchStorage(null),false);
 assert.equal(qualifiesSearchStorage(searchStorageReport(memory)),false);
 assert.equal(qualifiesSearchStorage(searchStorageReport(sealed)),true);
 for(const patch of [{mode:'plain'},{verified:false},{encryptedAtRest:false},{proof:'guessed'}])assert.throws(()=>searchStorageReport({...sealed,...patch}));
});

test('search writer reports validate effective resources and strip private handoff fields',()=>{
 const value={requestedWorkers:1,indexingWorkers:1,memoryBudgetBytes:50000000};
 assert.deepEqual(searchWriterReport({...value,cookie:'private'}),value);
 assert.equal(searchWriterReport(undefined),null);
 assert.equal(searchWriterReport({...value,requestedWorkers:null,indexingWorkers:3}).indexingWorkers,3);
 for(const patch of [{indexingWorkers:0},{indexingWorkers:9},{indexingWorkers:4},{requestedWorkers:2},{memoryBudgetBytes:14999999},{memoryBudgetBytes:128000001}])assert.throws(()=>searchWriterReport({...value,...patch}));
});

test('loaded policy proof requires exact deny and restored authorization controls and redacts private fields',()=>{
 const proof={mode:'loaded',verified:true,key:'access.protocols',allowedProtocols:['imap'],locked:true,primedRequests:2,revokedRequests:2,revokedStatus:403,restoredRequests:2};
 assert.deepEqual(fixturePolicyReport({...proof,secret:'private-cookie'}),proof);
 assert.equal(fixturePolicyReport(undefined).verified,false);
 assert.equal(fixturePolicyReport({mode:'defaults',verified:false}).verified,false);
 for(const patch of [{verified:false},{key:'other'},{allowedProtocols:['imap','pop3']},{revokedStatus:429},{restoredRequests:0},{primedRequests:0},{locked:false}])assert.throws(()=>fixturePolicyReport({...proof,...patch}));
});

test('startup readiness reports only verified aggregate counts and never copies private handoff fields',()=>{
 const proof={verified:true,state:'ready',rebuildRunning:false,startupSeconds:3,indexWaitSeconds:2,seededMessages:200,expectedIndexDocuments:200,indexDocuments:200,zeroAccessExcludedMessages:0,rebuildDone:200,rebuildTotal:200};
 assert.equal(startupReadinessReport(undefined),null);
 assert.throws(()=>startupReadinessReport(undefined,{required:true}),/readiness proof is required/);
 assert.deepEqual(startupReadinessReport({...proof,cookie:'private-session',accounts:['private-account']}),proof);
 for(const overrides of [{verified:false},{rebuildRunning:true},{indexDocuments:199},{seededMessages:201},{indexWaitSeconds:NaN},{rebuildDone:500001}])assert.throws(()=>startupReadinessReport({...proof,...overrides}));
 assert.equal(startupReadinessReport({...proof,state:'policy-excluded',expectedIndexDocuments:0,indexDocuments:0,zeroAccessExcludedMessages:200}).state,'policy-excluded');
});

test('short prefixes cover representative users and every endpoint rotates through the whole population',()=>{
 for(const count of [1,3,100,10000])for(const operations of [1,5]){
  const seen=Array.from({length:operations},()=>new Set());
  for(let index=0;index<count*operations;index++)seen[index%operations].add(fixturePrincipalIndex(index,operations,count));
  assert.ok(seen.every(users=>users.size===count));
  if(count===100||count===10000){
   const prefix=new Set(Array.from({length:count},(_,index)=>fixturePrincipalIndex(index,operations,count)));
   assert.equal(prefix.size,count,'early arrivals must reach every representative user');
  }
 }
 for(const args of [[-1,5,100],[0,0,100],[0,5,10001],[NaN,5,100]])assert.throws(()=>fixturePrincipalIndex(...args));
});

test('forwarded simulation requires bounded explicit fixture opt-in and distinct sessions/accounts',()=>{
 const seeded=principals(),model=prepareFixtureClients(seeded,simulation);
 assert.equal(model.clientCount,4);assert.equal(model.workload,'multi-user-fixture');
 for(let index=0;index<4;index++)assert.equal(model.clients[index].headers['x-forwarded-for'],seeded[index].clientAddress);
 assert.throws(()=>prepareFixtureClients(seeded),/explicit proxy/);
 for(const mutate of [p=>{p[1].cookie=p[0].cookie;},p=>{p[1].cookie='mw_session=private-session-0; mw_csrf=different';},p=>{p[1].accountId=p[0].accountId;},p=>{delete p[1].clientAddress;},p=>{p[1].clientAddress=p[0].clientAddress;},p=>{p[1].clientAddress='8.8.8.8';},p=>{p[1].clientAddress='198.18.00.2';},p=>{p[1].clientAddress='198.18.0.2\r\nx-spoof: true';}]){
  const bad=principals();mutate(bad);assert.throws(()=>prepareFixtureClients(bad,simulation));
 }
 assert.throws(()=>prepareFixtureClients(seeded,{...simulation,clientCount:3}));
 assert.throws(()=>prepareFixtureClients(seeded,{...simulation,mode:'arbitrary-production-proxy'}));
 for(const csrfToken of ['%0d%0aCookie: private-secret','%zz',42])assert.throws(()=>prepareFixtureClients([{cookie:'mw_session=one',csrfToken}]),error=>error.message==='Invalid fixture CSRF token');
 const single=prepareFixtureClients([{cookie:'mw_session=one'}]);
 assert.equal(single.workload,'single-user-diagnostic');assert.equal(single.simulationMode,'single-loopback-peer');
 assert.equal(single.clients[0].headers['x-forwarded-for'],undefined);
});

test('real HTTP distributes cookies and simulated clients and reports counts without identities',async()=>{
 const seeded=principals(),model=prepareFixtureClients(seeded,simulation),seen=new Set();
 const server=createServer((req,res)=>{
  const index=seeded.findIndex(p=>p.cookie===req.headers.cookie);
  const valid=index>=0&&seeded[index].clientAddress===req.headers['x-forwarded-for'];
  if(valid)seen.add(index);
  res.setHeader('content-type','application/json');res.end(JSON.stringify({valid,index}));
 });
 await new Promise(done=>server.listen(0,'127.0.0.1',done));
 try{
  for(const requestCount of [1,5]){
   const requests=Array.from({length:requestCount},(_,operation)=>({name:`operation-${operation}`,path:'/',materialize:(sequence,{requestCount})=>{
    const index=Math.floor(sequence/requestCount)%seeded.length;
    return {...model.clients[index],validate:v=>v.valid&&v.index===index};
   }}));
   const result=await measure({base:`http://127.0.0.1:${server.address().port}`,concurrency:4,seconds:.3,arrivalRate:200,requests});
   assert.equal(result.errors,0);assert.equal(result.uniquePrincipalsAttempted,4);assert.equal(result.uniquePrincipalsAccepted,4);
   assert.equal(result.uniqueClientsAttempted,4);assert.equal(result.uniqueClientsAccepted,4);
   assert.equal(result.participantDistribution.principals.accepted.requests,result.accepted);
   assert.equal(result.participantDistribution.simulatedClients.accepted.requests,result.accepted);
   assert.ok(result.participantDistribution.principals.attempted.max-result.participantDistribution.principals.attempted.min<=requestCount);
   const serialized=JSON.stringify(result);assert.ok(!serialized.includes('private-'));assert.ok(!serialized.includes('198.18.'));
   const markdown=markdownReport({target:'ephemeral fixture',scenario:'multi-user',runtime:'node',model:'fixture',stages:[result]});
   assert.match(markdown,/Distinct users/);assert.match(markdown,/Simulated clients/);
  }
  assert.equal(seen.size,4);
 }finally{server.closeAllConnections();await new Promise(done=>server.close(done));}
});

test('drain completions cannot establish in-window user or simulated-client coverage',async()=>{
 const server=createServer((_req,res)=>setTimeout(()=>{res.setHeader('content-type','application/json');res.end('{"ok":true}');},120));
 await new Promise(done=>server.listen(0,'127.0.0.1',done));
 try{
  const result=await measure({base:`http://127.0.0.1:${server.address().port}`,concurrency:1,seconds:.05,arrivalRate:20,requests:[{name:'late',path:'/',principalIndex:0,clientIndex:0,validate:v=>v.ok}]});
  assert.equal(result.uniquePrincipalsAccepted,1);assert.equal(result.uniqueClientsAccepted,1);
  assert.equal(result.uniquePrincipalsAcceptedDuringScheduledWindow,0);assert.equal(result.uniqueClientsAcceptedDuringScheduledWindow,0);
 }finally{server.closeAllConnections();await new Promise(done=>server.close(done));}
});

test('linked-login guard evidence requires real denial restoration and cleanup without private fields',()=>{
 const valid={verified:true,testedPrincipals:2,deniedStatus:401,ownerStatus:200,restoredStatus:200,originalPolicyRestored:true,sourceRemoved:true,linksRemoved:true,measuredPrimariesUnchanged:true};
 assert.deepEqual(linkedLoginPolicyProbeReport({...valid,cookie:'private-session',accountId:'private-account'}),valid);
 assert.equal(linkedLoginPolicyProbeReport(undefined),null);
 assert.deepEqual(linkedLoginPolicyProbeReport({verified:false,reason:'single-principal',testedPrincipals:0,cookie:'private'}),{verified:false,reason:'single-principal',testedPrincipals:0});
 for(const patch of [{deniedStatus:403},{ownerStatus:401},{restoredStatus:401},{originalPolicyRestored:false},{sourceRemoved:false},{linksRemoved:false},{measuredPrimariesUnchanged:false},{testedPrincipals:1},{verified:false}])assert.throws(()=>linkedLoginPolicyProbeReport({...valid,...patch}));
 assert.throws(()=>linkedLoginPolicyProbeReport({verified:false,reason:'single-principal',testedPrincipals:2}));
});
