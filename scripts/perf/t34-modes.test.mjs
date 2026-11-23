import test from 'node:test';
import assert from 'node:assert/strict';
import {parseT34Selection,ARMS,armEnvLines,armSelection,t34ModesEvidence,sqlStatementsPerHttp,checkReport,buildCampaign,runOrder,SELECTORS} from './t34-modes.mjs';
import {indexPublicationReport} from './index-publication-health.mjs';

const pub=(o={})=>({running:true,poisoned:false,requiresFullRebuild:false,pendingJobs:0,pendingBytes:0,queuedJobs:0,submittedJobs:10,completedJobs:10,failedJobs:0,publishedBatches:5,publishedChanges:10,revalidations:0,pendingRepairMessages:0,erasingAccounts:0,maxPendingJobs:128,maxPendingBytes:16777216,maxBatchDocuments:4096,
 ackMode:'commit',earlyAck:false,journalReplayPending:false,journalRows:null,journalAcknowledgedRows:0,journalAckFailures:0,journalReplayedKeys:0,readFenceWaits:0,readFenceTimeouts:0,readFenceRefusals:0,draining:false,...o});
const guard=(o={})=>({supported:true,mode:'off',maxStalenessMs:null,polls:0,failures:0,rowsApplied:0,hits:0,misses:0,expedites:0,...o});
const wb=(o={})=>({supported:true,accounting:'synchronous',pending:0,flushed:0,batches:0,writeThroughs:0,writeThroughFailures:0,flushFailures:0,rejected:0,dropped:0,loads:0,degraded:false,...o});
const unsupported={supported:false,reason:'no public fixture witness in mw-server'};
const profile=(n)=>({enabled:true,queries:{read:{calls:n*10},write:{calls:n},other:{calls:n}}});
function snap({g=guard(),p=pub(),w=wb(),prof}={}){return {t34Modes:{schema:1,guardSnapshot:g,requestProtection:w},searchIndex:{publication:p},...(prof?{requestProfile:prof}:{})};}
function stage(endpoint,before,after,extra={}){return {endpoint,phase:'concurrency-ramp',concurrency:8,requests:100,accepted:100,diagnostics:{before,after},...extra};}
const handoff=(arm)=>{const m=armSelection(arm).modes;return {schema:1,requested:{guardSnapshot:{mode:m.guardSnapshot},indexPublication:{ack:m.indexAck},requestProtection:{accounting:m.requestProtection}},startup:{}};};
const inputs=(mode,consumed)=>({expectedMode:mode,startup:{mode},consumedFramesDelta:consumed});

// ---- selectors ----
test('unset selection is all off and published',()=>{
 const s=parseT34Selection({});
 assert.deepEqual(s.modes,{guardSnapshot:'off',indexAck:'commit',requestProtection:'synchronous',requestInputs:'published'});
 assert.deepEqual(s.env,{});
});
test('every selector rejects out-of-range, malformed and wrongly cased values naming the variable',()=>{
 const bad={MW_GUARD_SNAPSHOT:['on','Bounded','bounded '],MW_GUARD_SNAPSHOT_MAX_STALENESS_MS:['99','5001','1e3','+200','-1'],MW_INDEX_PUBLICATION_ACK:['Journal','async'],
  MW_INDEX_PUBLICATION_WINDOW_MS:['11','-1'],MW_INDEX_PUBLICATION_MAX_BATCH_JOBS:['0','129'],MW_INDEX_READ_FENCE_TIMEOUT_MS:['99','30001'],MW_INDEX_PUBLICATION_DRAIN_SECS:['61'],
  MW_REQUEST_PROTECTION_ACCOUNTING:['write_behind','async'],MW_REQUEST_PROTECTION_FLUSH_MS:['49','5001'],MW_REQUEST_PROTECTION_MAX_PENDING:['63','65537'],
  MW_NATIVE_LOAD_REQUEST_INPUTS_MODE:['composite','Original'],MW_LOAD_FIXTURE_REQUEST_INPUTS_EXPECT:['x']};
 assert.equal(Object.keys(bad).length,SELECTORS.length);
 for(const [name,values] of Object.entries(bad))for(const value of values)assert.throws(()=>parseT34Selection({[name]:value}),new RegExp(name),`${name}=${value}`);
});
test('boundary values are accepted',()=>{
 assert.equal(parseT34Selection({MW_GUARD_SNAPSHOT:'bounded',MW_GUARD_SNAPSHOT_MAX_STALENESS_MS:'100'}).env.MW_GUARD_SNAPSHOT_MAX_STALENESS_MS,'100');
 parseT34Selection({MW_GUARD_SNAPSHOT:'bounded',MW_GUARD_SNAPSHOT_MAX_STALENESS_MS:'5000',MW_INDEX_PUBLICATION_ACK:'journal',MW_INDEX_READ_FENCE_TIMEOUT_MS:'30000',MW_REQUEST_PROTECTION_ACCOUNTING:'write-behind',MW_REQUEST_PROTECTION_FLUSH_MS:'50',MW_REQUEST_PROTECTION_MAX_PENDING:'65536'});
});
test('a tuning knob without its mode is refused instead of silently doing nothing',()=>{
 assert.throws(()=>parseT34Selection({MW_GUARD_SNAPSHOT_MAX_STALENESS_MS:'500'}),/requires MW_GUARD_SNAPSHOT=bounded/);
 assert.throws(()=>parseT34Selection({MW_GUARD_SNAPSHOT:'off',MW_GUARD_SNAPSHOT_MAX_STALENESS_MS:'500'}),/requires/);
 assert.throws(()=>parseT34Selection({MW_INDEX_READ_FENCE_TIMEOUT_MS:'500'}),/requires MW_INDEX_PUBLICATION_ACK=journal/);
 assert.throws(()=>parseT34Selection({MW_REQUEST_PROTECTION_FLUSH_MS:'500'}),/requires/);
 assert.throws(()=>parseT34Selection({MW_NATIVE_LOAD_REQUEST_INPUTS_MODE:'original',MW_LOAD_FIXTURE_REQUEST_INPUTS_EXPECT:'published'}),/must equal/);
});
test('bounded snapshot under original request inputs is allowed but warned about',()=>{
 assert.equal(parseT34Selection({MW_GUARD_SNAPSHOT:'bounded',MW_NATIVE_LOAD_REQUEST_INPUTS_MODE:'original'}).warnings.length,1);
});

// ---- arms and campaign ----
test('arms: each mode alone, all, all + original, every arm explicit and valid',()=>{
 assert.deepEqual(Object.keys(ARMS),['baseline','original','snapshot','journal','write-behind','all','all-original']);
 const on=a=>{const m=armSelection(a).modes;return [m.guardSnapshot==='bounded',m.indexAck==='journal',m.requestProtection==='write-behind',m.requestInputs==='original'];};
 assert.deepEqual(on('baseline'),[false,false,false,false]);assert.deepEqual(on('original'),[false,false,false,true]);
 assert.deepEqual(on('snapshot'),[true,false,false,false]);assert.deepEqual(on('journal'),[false,true,false,false]);assert.deepEqual(on('write-behind'),[false,false,true,false]);
 assert.deepEqual(on('all'),[true,true,true,false]);assert.deepEqual(on('all-original'),[true,true,true,true]);
 assert.ok(armEnvLines('baseline').includes('MW_GUARD_SNAPSHOT=off'));
 assert.throws(()=>armSelection('nope'),/Unknown arm/);
});
test('campaign keeps tonight\'s shape, alternates order and adds the WebSocket pair and labelled instrumented runs',()=>{
 const c=buildCampaign();
 assert.deepEqual([c.shape.users,c.shape.messagesPerUser,c.shape.cpus,c.shape.securityProfile,c.shape.seconds,c.shape.repeats],[100,50,4,'enforced',30,2]);
 assert.deepEqual(c.shape.endpoints,['identity','query','headers','representative']);assert.deepEqual(c.shape.concurrency,[8,32,64]);
 assert.equal(c.runs.length,14);assert.ok(c.runs.every(r=>r.requestProfile===false));
 const arms=Object.keys(ARMS);
 assert.deepEqual(c.runs.slice(0,7).map(r=>r.arm),arms);assert.deepEqual(c.runs.slice(7).map(r=>r.arm),[...arms].reverse());
 assert.equal(new Set(c.runs.map(r=>r.id)).size,14);
 assert.ok(c.instrumentedRuns.every(r=>r.requestProfile===true)&&c.instrumentedRuns.length===7);
 assert.deepEqual(c.websocketIdleHold.map(w=>[w.connections,w.env.MW_GUARD_SNAPSHOT]),[[1000,'off'],[1000,'bounded']]);
 assert.deepEqual(runOrder(['a','b'],2).map(r=>r.arm),['a','b','b','a']);
});

// ---- evidence: baseline / off ----
test('baseline with a supported witness and zero activity is off-verified; unsupported witnesses are off-unwitnessed',()=>{
 const st=[stage('identity',snap(),snap())];
 const e=t34ModesEvidence(handoff('baseline'),st,{requestInputsEvidence:inputs('published',5)});
 assert.equal(e.valid,true);assert.equal(e.modes.guardSnapshot.status,'off-verified');assert.equal(e.modes.indexPublication.status,'off-verified');assert.equal(e.modes.requestProtection.status,'off-verified');
 const bare=t34ModesEvidence(handoff('baseline'),[stage('identity',snap({g:unsupported,w:unsupported}),snap({g:unsupported,w:unsupported}))]);
 assert.equal(bare.valid,true);assert.equal(bare.modes.guardSnapshot.status,'off-unwitnessed');
});
test('an off arm that shows activity is invalid',()=>{
 const busy=snap({g:guard({polls:3,hits:9}),p:pub({readFenceWaits:2}),w:wb({flushed:4})});
 const e=t34ModesEvidence(handoff('baseline'),[stage('identity',snap(),busy)]);
 assert.equal(e.valid,false);
 for(const name of ['guardSnapshot','indexPublication','requestProtection'])assert.equal(e.modes[name].status,'invalid',name);
 const wrong=t34ModesEvidence(handoff('baseline'),[stage('identity',snap({g:guard({mode:'bounded'})}),snap({g:guard({mode:'bounded'})}))]);
 assert.equal(wrong.valid,false);assert.match(wrong.invalidReasons.join(),/guard snapshot on although/);
});

// ---- evidence: guard snapshot ----
test('guard snapshot arm is valid only with polls, reuses and lookups in every stage',()=>{
 const g0=guard({mode:'bounded',polls:5,hits:10,misses:2}),g1=guard({mode:'bounded',polls:40,hits:900,misses:30,expedites:1,rowsApplied:3});
 const ok=t34ModesEvidence(handoff('snapshot'),[stage('identity',snap({g:g0}),snap({g:g1}))],{requestInputsEvidence:inputs('published',9)});
 assert.equal(ok.valid,true,ok.invalidReasons.join());assert.equal(ok.modes.guardSnapshot.reuses,890);assert.equal(ok.modes.guardSnapshot.fallbacksToFreshPath,28);
 const nopoll=t34ModesEvidence(handoff('snapshot'),[stage('identity',snap({g:g0}),snap({g:{...g1,polls:5}}))]);
 assert.match(nopoll.invalidReasons.join(),/zero mirror polls/);
 const nohit=t34ModesEvidence(handoff('snapshot'),[stage('identity',snap({g:g0}),snap({g:{...g1,hits:10,misses:2}}))]);
 assert.match(nohit.invalidReasons.join(),/zero snapshot reuses/);
 const second=t34ModesEvidence(handoff('snapshot'),[stage('identity',snap({g:g0}),snap({g:g1})),stage('query',snap({g:g1}),snap({g:{...g1,polls:60}}))]);
 assert.match(second.invalidReasons.join(),/no snapshot lookups in query/);
 const off=t34ModesEvidence(handoff('snapshot'),[stage('identity',snap({g:guard()}),snap({g:guard()}))]);
 assert.equal(off.valid,false);assert.match(off.invalidReasons.join(),/not bounded/);
});
test('a snapshot arm on a fixture without the witness is invalid, never silently off',()=>{
 const e=t34ModesEvidence(handoff('snapshot'),[stage('identity',snap({g:unsupported}),snap({g:unsupported}))]);
 assert.equal(e.valid,false);assert.match(e.invalidReasons.join(),/unsupported or missing/);
 const legacy=t34ModesEvidence(handoff('snapshot'),[stage('identity',{},{})]);assert.equal(legacy.valid,false);
});
test('guard counters going backwards (process restart) invalidate the arm',()=>{
 const e=t34ModesEvidence(handoff('snapshot'),[stage('identity',snap({g:guard({mode:'bounded',polls:50,hits:50,misses:1})}),snap({g:guard({mode:'bounded',polls:3,hits:90,misses:1})}))]);
 assert.match(e.invalidReasons.join(),/backwards/);
});

// ---- evidence: journal ----
const jBefore=pub({ackMode:'journal',earlyAck:true,journalRows:0}),jAfter=pub({ackMode:'journal',earlyAck:true,journalRows:2,submittedJobs:30,completedJobs:30,journalAcknowledgedRows:20,readFenceWaits:4});
test('journal arm needs early ack, a clean readiness gate and applied rows in the mutating phases',()=>{
 const ok=t34ModesEvidence(handoff('journal'),[stage('representative',snap({p:jBefore}),snap({p:jAfter}))],{requestInputsEvidence:inputs('published',1)});
 assert.equal(ok.valid,true,ok.invalidReasons.join());assert.equal(ok.modes.indexPublication.rowsPending,2);assert.equal(ok.modes.indexPublication.rowsAtReadinessGate,0);assert.equal(ok.modes.indexPublication.rowsWritten,22);
 const mk=(b,a)=>t34ModesEvidence(handoff('journal'),[stage('representative',snap({p:b}),snap({p:a}))]).invalidReasons.join();
 assert.match(mk(jBefore,{...jAfter,journalAcknowledgedRows:0,submittedJobs:10,completedJobs:10}),/zero journal rows/);
 assert.match(mk({...jBefore,journalRows:3},jAfter),/readiness gate is 3/);
 assert.match(mk({...jBefore,journalRows:null},jAfter),/readiness gate/);
 assert.match(mk(jBefore,{...jAfter,earlyAck:false}),/early acknowledgement/);
 assert.match(mk(jBefore,{...jAfter,ackMode:'commit'}),/other than journal/);
 assert.match(mk(jBefore,{...jAfter,readFenceTimeouts:1}),/fence timeouts/);
 assert.match(mk(jBefore,{...jAfter,journalAckFailures:1}),/acknowledgement failures/);
 assert.match(mk({...jBefore,journalReplayPending:true},jAfter),/replay still pending/);
});
test('journal arm on a read-only workload cannot show activity and is invalid',()=>{
 const e=t34ModesEvidence(handoff('journal'),[stage('identity',snap({p:jBefore}),snap({p:jAfter}))]);
 assert.match(e.invalidReasons.join(),/no mutating endpoint/);
});
test('journal arm without journal fields (legacy fixture) is invalid',()=>{
 const legacy={...pub()};for(const k of ['ackMode','earlyAck','journalReplayPending','journalRows','journalAcknowledgedRows','journalAckFailures','journalReplayedKeys','readFenceWaits','readFenceTimeouts','readFenceRefusals','draining'])delete legacy[k];
 const e=t34ModesEvidence(handoff('journal'),[stage('representative',snap({p:legacy}),snap({p:legacy}))]);
 assert.match(e.invalidReasons.join(),/evidence missing/);
});
test('publication report carries journal fields and rejects malformed ones',()=>{
 const r=indexPublicationReport(jAfter);assert.equal(r.ackMode,'journal');assert.equal(r.journalRows,2);assert.equal(r.readFenceWaits,4);
 assert.equal(indexPublicationReport(pub({journalRows:null})).journalRows,null);
 for(const patch of [{ackMode:'x'},{earlyAck:1},{readFenceWaits:-1},{journalRows:-1}])assert.throws(()=>indexPublicationReport(pub(patch)));
});

// ---- evidence: write-behind ----
test('write-behind arm needs flushed or written-through observations and a healthy queue',()=>{
 const b=wb({accounting:'write-behind'}),a=wb({accounting:'write-behind',flushed:500,batches:9,writeThroughs:12,loads:3,pending:4});
 const ok=t34ModesEvidence(handoff('write-behind'),[stage('identity',snap({w:b}),snap({w:a}))],{requestInputsEvidence:inputs('published',1)});
 assert.equal(ok.valid,true,ok.invalidReasons.join());assert.equal(ok.modes.requestProtection.queued,504);assert.equal(ok.modes.requestProtection.deltas.writeThroughs,12);
 const mk=(patch)=>t34ModesEvidence(handoff('write-behind'),[stage('identity',snap({w:b}),snap({w:{...a,...patch}}))]);
 assert.match(mk({flushed:0,writeThroughs:0}).invalidReasons.join(),/zero observations/);
 assert.match(mk({accounting:'synchronous'}).invalidReasons.join(),/other than write-behind/);
 assert.match(mk({flushFailures:1}).invalidReasons.join(),/storage failures/);
 assert.match(mk({degraded:true}).invalidReasons.join(),/degraded/);
 const dropped=mk({dropped:7,rejected:1});assert.equal(dropped.valid,true);assert.equal(dropped.modes.requestProtection.warnings.length,2);
 const none=t34ModesEvidence(handoff('write-behind'),[stage('identity',snap({w:unsupported}),snap({w:unsupported}))]);
 assert.equal(none.valid,false);assert.match(none.invalidReasons.join(),/unsupported or missing/);
});

// ---- request inputs ----
test('request-input claims are checked against the existing receipt',()=>{
 const st=[stage('identity',snap(),snap())];
 assert.equal(t34ModesEvidence(handoff('baseline'),st,{requestInputsEvidence:inputs('published',0)}).valid,false);
 assert.equal(t34ModesEvidence(handoff('original'),st,{requestInputsEvidence:inputs('original',0)}).valid,true);
 assert.equal(t34ModesEvidence(handoff('original'),st,{requestInputsEvidence:inputs('original',4)}).valid,false);
 assert.equal(t34ModesEvidence(handoff('original'),st,{}).modes.requestInputs.status,'unclaimed');
 assert.equal(t34ModesEvidence(handoff('original'),st,{requestInputsEvidence:null}).modes.requestInputs.status,'unclaimed');
});

// ---- SQL per HTTP ----
test('SQL statements per HTTP are per-arm, labelled, and null when uninstrumented',()=>{
 const st=[stage('identity',snap({prof:profile(0)}),snap({prof:profile(100)}),{requests:100}),stage('query',snap({prof:profile(100)}),snap({prof:profile(300)}),{requests:100})];
 const s=sqlStatementsPerHttp(st);
 assert.equal(s.instrumented,true);assert.match(s.label,/^instrumented/);assert.equal(s.stages[0].statementsPerHttp,12);assert.equal(s.stages[1].statementsPerHttp,24);assert.equal(s.statementsPerHttp,18);
 const off=sqlStatementsPerHttp([stage('identity',snap({prof:{enabled:false}}),snap({prof:{enabled:false}}))]);
 assert.equal(off.instrumented,false);assert.equal(off.statementsPerHttp,null);assert.match(off.label,/^uninstrumented/);
 const e=t34ModesEvidence(handoff('baseline'),st);assert.equal(e.sqlPerHttp.statementsPerHttp,18);
});

// ---- report checks ----
test('checkReport rejects a report whose fixture ran another arm or lacks evidence',()=>{
 const good=report('snapshot');assert.equal(checkReport(good,'snapshot').valid,true,JSON.stringify(checkReport(good,'snapshot')));
 assert.equal(checkReport(good,'baseline').valid,false);
 assert.equal(checkReport({},'baseline').valid,false);
 assert.match(checkReport({...good,ordinaryRequestInputs:{expectedMode:'original'}},'snapshot').reasons.join(),/request inputs/);
 const inv=report('snapshot',{g:guard()});assert.equal(checkReport(inv,'snapshot').valid,false);
});
function report(arm,over={}){
 const g0=guard({mode:'bounded',polls:1,hits:1}),g1=guard({mode:'bounded',polls:9,hits:99,misses:1});
 const stages=[stage('identity',snap({g:over.g??g0}),snap({g:over.g??g1}))];
 const ri=inputs('published',5);
 return {ordinaryRequestInputs:ri,t34Modes:t34ModesEvidence(handoff(arm),stages,{requestInputsEvidence:ri})};
}
test('missing stages or handoff never throw: legacy has no evidence, empty is invalid',()=>{
 assert.equal(t34ModesEvidence(undefined,[]),null);
 assert.equal(t34ModesEvidence(handoff('baseline'),[]).valid,false);
 assert.equal(t34ModesEvidence({schema:2},[stage('identity',{},{})]).valid,false);
});
