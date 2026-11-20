import test from 'node:test';
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {validateReady} from './external-control.mjs';
import {campaignOptions,runCampaign} from './http-campaign.mjs';
import {fixtureDiagnosticsReport} from './fixture-clients.mjs';
import {sqlitePageCacheReport,sqlitePageCacheAuditReport,cacheExperimentReport,cacheExperimentFromEnv,campaignCacheExperiment,assertSqliteCacheWitness,cacheExperimentEvidence} from './sqlite-page-cache-witness.mjs';
const experiment=(target=256)=>({kind:'sqlite-page-cache',requestedAggregateMiB:target});
function snapshot(target=256){
 const share=target===null?null:target*1024/2;
 const connections=[1,2].map(connectionEpoch=>({connectionEpoch,cacheUsedBytes:4096,cacheHits:7,cacheMisses:2,cacheWrites:1,cacheSpills:0}));
 const sample={openBefore:2,openAfter:2,sampledConnections:2,completeIdleCoverage:true,deadlineReached:false,connections};
 const targets={aggregateMiB:target,plannedMaximumConnections:2,perConnectionKiB:share,auxiliaryConnectionKiB:target===null?null:2000,driverDefaultUnchanged:target===null,hardMemoryLimit:false};
 return {pool:{maximum:2,sqlitePageCache:{...targets,plannedForegroundBytes:target===null?null:target*1024*1024,restartRequired:true,idleSample:structuredClone(sample)}},sqlitePageCacheAudit:{schema:1,scope:'foreground-pool-main-target',generationPinned:true,...targets,deadlineMs:25,deadlineIsCooperative:true,counterScope:'all-attached-pagers-per-handle',sample:{...sample,failedConnections:0,connections:connections.map(row=>({...row,mainCacheSize:share===null?-2000:-share}))}}};
}
test('legacy and baseline options stay absent; experiments require explicit matching controller and native handoff',()=>{
 assert.equal(sqlitePageCacheReport(undefined),null);assert.equal(sqlitePageCacheAuditReport(null),null);
 assert.equal(Object.hasOwn(campaignOptions({}),'sqliteCacheExperiment'),false);
 assert.equal(campaignCacheExperiment({engineMode:'native',backend:'sqlite'},{}),null);
 for(const target of [null,16,256,1024]){
  const env=target===null?'null':String(target),options=campaignOptions({MW_LOAD_FIXTURE_SQLITE_CACHE_EXPECT:env});
  assert.deepEqual(options.sqliteCacheExperiment,experiment(target));
  assert.deepEqual(campaignCacheExperiment({engineMode:'native',backend:'sqlite',sqliteCacheExperiment:experiment(target)},options),experiment(target));
 }
 for(const raw of ['',null,'NULL','0','15','1025','0256','+256','256.0',' 256'])assert.throws(()=>cacheExperimentFromEnv(raw));
 for(const handoff of [{engineMode:'proxy',backend:'sqlite'},{engineMode:'native',backend:'postgres'},{engineMode:'native',backend:'sqlite'}])assert.throws(()=>campaignCacheExperiment(handoff,{sqliteCacheExperiment:experiment()}));
 assert.throws(()=>campaignCacheExperiment({engineMode:'native',backend:'sqlite',sqliteCacheExperiment:experiment()},{}));
 assert.throws(()=>campaignCacheExperiment({engineMode:'native',backend:'sqlite',sqliteCacheExperiment:experiment()}, {sqliteCacheExperiment:experiment(null)}));
});
test('actual targets and per-handle counters survive repeated sanitizer passes without copying secrets',()=>{
 for(const target of [null,256]){
  const value=snapshot(target);value.sqlitePageCacheAudit.key='secret-key';value.sqlitePageCacheAudit.sample.connections[0].pointer='secret-pointer';value.pool.sqlitePageCache.cookie='secret-cookie';
  const clean=fixtureDiagnosticsReport(fixtureDiagnosticsReport(value));
  assert.equal(JSON.stringify(clean).includes('secret-'),false);
  assert.equal(assertSqliteCacheWitness(clean,experiment(target)).sample.connections.length,2);
  assert.equal(clean.sqlitePageCacheAudit.sample.connections[0].mainCacheSize,target===null?-2000:-131072);
  assert.equal(clean.sqlitePageCacheAudit.sample.connections[0].cacheSpills,0);
  assert.deepEqual(sqlitePageCacheAuditReport(clean.sqlitePageCacheAudit),clean.sqlitePageCacheAudit);
 }
});
test('custom configuration alone, partial/busy/failed sampling and unknown counters never qualify',()=>{
 const changes=[v=>delete v.sqlitePageCacheAudit,v=>delete v.pool.sqlitePageCache,
 v=>{v.sqlitePageCacheAudit.sample.completeIdleCoverage=false;},
 v=>{const s=v.sqlitePageCacheAudit.sample;s.connections.pop();s.sampledConnections=1;s.completeIdleCoverage=false;},
 v=>{v.sqlitePageCacheAudit.sample.deadlineReached=true;},
 v=>{v.sqlitePageCacheAudit.sample.failedConnections=1;},
 v=>{v.sqlitePageCacheAudit.sample.connections[0].cacheMisses=null;},
 v=>{v.sqlitePageCacheAudit.sample.connections[0].connectionEpoch=null;},
 v=>{v.sqlitePageCacheAudit.sample.connections[0].mainCacheSize=-2000;},
 v=>{v.sqlitePageCacheAudit.sample.connections[0].mainCacheSize=32768;},
 v=>{v.sqlitePageCacheAudit.perConnectionKiB=32768;},
 v=>{v.sqlitePageCacheAudit.auxiliaryConnectionKiB=1000;},
 v=>{v.pool.maximum=4;},v=>{v.sqlitePageCacheAudit.generationPinned=false;},
 v=>{v.sqlitePageCacheAudit.deadlineIsCooperative=false;},v=>{v.sqlitePageCacheAudit.hardMemoryLimit=true;}];
 for(const change of changes){const value=snapshot();change(value);assert.throws(()=>assertSqliteCacheWitness(value,experiment()));}
 assert.throws(()=>assertSqliteCacheWitness(snapshot(),experiment(null)));
});
test('bounded shapes reject duplicate epochs, malformed counts, over-cap data and nonfinite values',()=>{
 for(const mutate of [v=>{v.sample.connections[1].connectionEpoch=1;},v=>{v.sample.sampledConnections=1;},v=>{v.sample.openAfter=33;},v=>{v.sample.connections[0].cacheHits=NaN;},v=>{v.sample.connections[0].cacheMisses=-1;},v=>{v.sample.connections[0].cacheWrites=2147483648;},v=>{v.sample.connections[0].mainCacheSize=Infinity;},v=>{v.sample.connections=Array.from({length:33},(_,i)=>({...v.sample.connections[0],connectionEpoch:i+1}));}]){
  const value=snapshot().sqlitePageCacheAudit;mutate(value);assert.throws(()=>sqlitePageCacheAuditReport(value));
 }
 const bad=snapshot().pool.sqlitePageCache;bad.plannedForegroundBytes=1;assert.throws(()=>sqlitePageCacheReport(bad));
 assert.throws(()=>cacheExperimentReport({kind:'sqlite-page-cache',requestedAggregateMiB:undefined}));
});
test('zero open handles and replaced/partial boundary reports cannot satisfy an experiment',()=>{
 const value=snapshot();Object.assign(value.sqlitePageCacheAudit.sample,{openBefore:0,openAfter:0,sampledConnections:0,connections:[]});
 assert.throws(()=>assertSqliteCacheWitness(value,experiment()));
 const valid=snapshot(),stages=[{diagnostics:{before:valid,after:structuredClone(valid)}}];
 assert.equal(cacheExperimentEvidence(experiment(),stages).verified,true);
 assert.equal(cacheExperimentEvidence(null,stages),null);
 assert.throws(()=>cacheExperimentEvidence(experiment(),[]));
 assert.throws(()=>cacheExperimentEvidence(experiment(),[{diagnostics:{before:valid,after:snapshot(null)}}]));
 assert.throws(()=>cacheExperimentEvidence(experiment(),[{diagnostics:{before:valid}}]));
});
test('an experiment audits outside each measured call and requires every before/after boundary',async()=>{
 const order=[],options=campaignOptions({MW_LOAD_FIXTURE_RATES:'1',MW_LOAD_FIXTURE_CONCURRENCY:'1',MW_LOAD_FIXTURE_SECONDS:'.1',MW_LOAD_FIXTURE_SQLITE_CACHE_EXPECT:'256'});
 const campaign=await runCampaign({groups:{mixed:[{}]},options,diagnostics:async phase=>{order.push(phase);return fixtureDiagnosticsReport(snapshot());},run:async()=>{order.push('measure');return {accepted:1,acceptedRequestsPerSecond:1,errors:0,throttled:0,timeouts:0,statuses:{200:1},latencyMs:{p99:1},droppedCapacity:0,droppedScheduler:0,offered:1};}});
 assert.deepEqual(order,['before','measure','after']);assert.equal(cacheExperimentEvidence(options.sqliteCacheExperiment,campaign.stages).verified,true);
});
test('overlay and private option forwarding include the witness without changing frozen CPU helpers',async()=>{
 const overlay=await readFile(new URL('./prepare-fixture-overlay.mjs',import.meta.url),'utf8');
 assert.ok(overlay.includes("'scripts/perf/sqlite-page-cache-witness.mjs'"));
 const control=await readFile(new URL('../../crates/mw-server/tests/common/request_load_control.rs',import.meta.url),'utf8');
 assert.ok(control.includes('"MW_LOAD_FIXTURE_SQLITE_CACHE_EXPECT"'));
});

test('private control forwards only the bounded expectation option and retains secret-env rejection',()=>{
 const base={version:1,runId:'12345678-1234-4234-8234-123456789abc',expiresAtMs:1001000,deadlineSeconds:90,handoffFile:'handoff.private.json',outputFile:'report.json',campaignEnv:{MW_LOAD_FIXTURE_SQLITE_CACHE_EXPECT:'256'}};
 assert.equal(validateReady(base,1000000),base);
 assert.deepEqual(campaignOptions(base.campaignEnv).sqliteCacheExperiment,experiment());
 assert.throws(()=>validateReady({...base,campaignEnv:{...base.campaignEnv,MW_DB_POOL:'private-server-configuration'}},1000000));
 assert.throws(()=>validateReady({...base,campaignEnv:{MW_LOAD_FIXTURE_SQLITE_CACHE_EXPECT:'x'.repeat(257)}},1000000));
});
