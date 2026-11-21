import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {stripTypeScriptTypes} from 'node:module';
import {nativeReadContract as contract,nativeReadIntentHeaders,isNativeMetadataReadIntent} from './native-read-intent.mjs';
import {prepareFixtureRequests} from './fixture-requests.mjs';
import {preparedMailOptions,preparedMailExperimentReport,preparedMailStatusReport,preparedMailEvidence} from './prepared-mail-experiment.mjs';
const defaults={enabled:true,budgetMiB:64,maxEntries:256,maxAccounts:128,idleSeconds:30};
const request=()=>({using:['urn:ietf:params:jmap:core','urn:ietf:params:jmap:mail'],methodCalls:[['Email/get',{accountId:'owned',ids:['a','a','b'],properties:[...contract.properties]},'call']]});
test('Node declaration matches actual production pure client helper without importing browser runtime',async()=>{
 const source=readFileSync(new URL('../../apps/web/src/api/nativeReadIntent.ts',import.meta.url),'utf8').replace("import contract from './native-read-intent-contract.json';",`const contract=${JSON.stringify(contract)};`);
 const production=await import('data:text/javascript;base64,'+Buffer.from(stripTypeScriptTypes(source)).toString('base64'));
 const cases=[request()];
 for(const mutate of [v=>v.methodCalls[0][0]='Email/set',v=>v.methodCalls.push(['Email/query',{},'q']),v=>v.createdIds={},v=>v.methodCalls[0][1].ids=Array(51).fill('x'),v=>v.methodCalls[0][1].ids=['é'.repeat(513)],v=>v.methodCalls[0][1].unknown=true,v=>v.methodCalls[0][1].properties[0]=v.methodCalls[0][1].properties[1],v=>v.methodCalls[0][1].properties[0]='bodyValues',v=>v.methodCalls[0][1].fetchAllBodyValues=true,v=>v.methodCalls[0][1].fetchAllBodyValues=false,v=>v.methodCalls[0][2]='é'.repeat(513)]){const value=request();mutate(value);cases.push(value);}
 for(const value of cases){const before=JSON.stringify(value);assert.equal(isNativeMetadataReadIntent(value),production.isNativeMetadataReadIntent(value));assert.deepEqual(nativeReadIntentHeaders(value),production.nativeReadIntentHeaders(value));assert.equal(JSON.stringify(value),before);}
});
const principal={cookie:'private-fixture',csrfToken:'fixture',accountId:'owned',ownerAccountId:'owned',messageIds:['a','b'],inboxId:'inbox',archiveId:'archive',attachmentBlobId:'blob',bodySampleBytes:4096,attachmentSampleBytes:65536,clientAddress:'198.18.0.1'};
const build=representative=>prepareFixtureRequests({principals:[principal],clientSimulation:{mode:'fixture-loopback-trusted-proxy',clientCount:1},engineMode:'native',representative});
test('all ten real representative HTTP shapes remain; only standalone DEFAULT36 gets marker',()=>{
 const fixture=build(true);assert.equal(fixture.firstRequests.length,10);assert.deepEqual(fixture.readIntent.markedShapes,['Email/get']);assert.equal(fixture.readIntent.observedGuardConsumption,null);
 for(const spec of fixture.firstRequests){assert.equal(spec.headers[contract.header]===contract.value,spec.name==='Email/get');if(spec.body){const value=JSON.parse(spec.body);if(spec.name==='paged-query-and-headers'){assert.deepEqual(value.methodCalls.map(c=>c[0]),['Email/query','Email/get','Email/query','Email/get']);assert.equal(value.methodCalls[1][1]['#ids'].resultOf,'page1');assert.equal(value.methodCalls[3][1]['#ids'].resultOf,'page2');}if(['mark-read-and-restore','move-and-restore'].includes(spec.name)){assert.deepEqual(value.methodCalls.map(c=>c[0]),['Email/set','Email/get','Email/set','Email/get']);assert.equal(spec.statefulMutation,true);}}}
 assert.deepEqual(build(false).readIntent.markedShapes,[]);
});
test('immutable Docker source and baseline overlay retain actual canonical contracts and helper dependencies',()=>{
 const docker=readFileSync(new URL('./Dockerfile',import.meta.url),'utf8'),overlay=readFileSync(new URL('./prepare-fixture-overlay.mjs',import.meta.url),'utf8');
 for(const file of ['apps/web/src/api/native-read-intent-contract.json','apps/web/src/api/websocket-action-contract.json']){assert.ok(docker.includes(`COPY ${file} /src/${file}`));assert.ok(docker.indexOf(`COPY ${file}`)<docker.indexOf('FROM node:'));assert.ok(overlay.includes(`'${file}'`));}
 for(const file of ['scripts/perf/native-read-intent.mjs','scripts/perf/prepared-mail-experiment.mjs'])assert.ok(overlay.includes(`'${file}'`));
 const fixture=readFileSync(new URL('../../crates/mw-server/tests/native_load_fixture.rs',import.meta.url),'utf8');assert.ok(fixture.includes('prime_prepared_mail(&client,&base,&principals[0]'));assert.ok(fixture.indexOf('prepared_experiment["startup"]=prime_prepared_mail')<fixture.indexOf('write_private(&handoff.0, &data)'));assert.ok(!fixture.includes('configure_prepared_mail(')&&!fixture.includes('configure_prepared_mail_at_revision('));const capacity=fixture.indexOf('options.max_accounts>=count'),directory=fixture.indexOf('let dir = std::env::temp_dir().join');assert.ok(capacity>=0&&directory>=0&&capacity<directory, 'reject undersized requested working-set config before filesystem/Store/seeding work');
});
function status(){return {entries:1,accounts:1,pendingBuilds:0,invalidatedEntries:0,budgetBytes:64*1048576,maxEntries:256,maxAccounts:128,idleSeconds:30,enabled:true,...Object.fromEntries(['chargedBytes','admittedJobs','activePreparers','eligibleCandidates','readyCandidateHits','candidateMisses','completedBuilds','evictions','seedCalls','seededRows','rawResults','compatibilityValueResults','handledInvalidationHints','invalidationErrors','invalidationPollCalls','invalidationAckCalls'].map(k=>[k,0]))};}
const experiment=()=>({enabled:true,requested:defaults,resolvedPolicy:defaults,startup:{verified:true,attempts:2,elapsedMs:250,primedPrincipals:1,measuredRequests:0}});
test('explicit prepared policy supports actual bounded ten-thousand-account JSON, default-off and unknown legacy',()=>{
 assert.equal(preparedMailExperimentReport(undefined),null);assert.equal(preparedMailExperimentReport({enabled:false,requested:null,resolvedPolicy:{...defaults,enabled:false}}).enabled,false);
 assert.throws(()=>preparedMailExperimentReport({enabled:false,requested:null,resolvedPolicy:{...defaults,enabled:false},startup:{attempts:'private',elapsedMs:'private'}}));
 assert.equal(preparedMailOptions({...defaults,maxAccounts:10000,maxEntries:10000}).maxAccounts,10000);
 for(const v of [{...defaults,unsafe:true},{...defaults,maxAccounts:10001},{...defaults,budgetMiB:15},{...defaults,enabled:1}])assert.throws(()=>preparedMailOptions(v));
 assert.throws(()=>preparedMailExperimentReport({...experiment(),resolvedPolicy:{...defaults,enabled:false}}));assert.throws(()=>preparedMailExperimentReport({...experiment(),startup:undefined}));assert.throws(()=>preparedMailExperimentReport({...experiment(),startup:{...experiment().startup,measuredRequests:1}}));
});
test('status is scalar-only and missing/null counters stay unknown; actual config/raw emission required',()=>{
 const before=status(),after={...before,rawResults:3,readyCandidateHits:4};assert.deepEqual(preparedMailStatusReport({...before,private:'never report'}),before);assert.equal(preparedMailStatusReport(null),null);
 const stages=[{endpoint:'representative',phase:'concurrency-ramp',byRequest:{'Email/get':{requests:10}},diagnostics:{before:{preparedMail:before},after:{preparedMail:after}}}];const proof=preparedMailEvidence(experiment(),stages);assert.equal(proof.rawResultsDelta,3);assert.equal(proof.guardInputConsumption,'not measured by these counters');assert.equal(proof.qualified,true);const control=preparedMailEvidence(experiment(),[{endpoint:'identity',phase:'concurrency-ramp',byRequest:{'authenticated-identity':{requests:10}},diagnostics:{before:{preparedMail:after},after:{preparedMail:after}}}]);assert.equal(control.qualified,false);assert.equal(control.eligiblePhases,0);assert.equal(control.rawResultsDelta,0);
 for(const mutation of [v=>v[0].diagnostics.before.preparedMail=null,v=>v[0].diagnostics.after.preparedMail.enabled=false,v=>v[0].diagnostics.after.preparedMail.maxAccounts=100,v=>v[0].diagnostics.after.preparedMail.rawResults=0]){const copy=structuredClone(stages);mutation(copy);assert.throws(()=>preparedMailEvidence(experiment(),copy));}
 assert.throws(()=>preparedMailStatusReport({...before,rawResults:-1}));assert.throws(()=>preparedMailStatusReport({...before,rawResults:NaN}));
 const busy={...before,chargedBytes:1024,admittedJobs:1,activePreparers:1};assert.equal(preparedMailEvidence(experiment(),[{...stages[0],diagnostics:{before:{preparedMail:busy},after:{preparedMail:after}}}]).rawResultsDelta,3,'gauges may decrease normally');
});
