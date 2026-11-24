// t34 server-mode selectors, A/B arms and positive evidence that an arm really ran its mode.
//
// Modes (all off by default, SQLite single-process only):
//   guard snapshot      MW_GUARD_SNAPSHOT=off|bounded, MW_GUARD_SNAPSHOT_MAX_STALENESS_MS
//   index journal ack   MW_INDEX_PUBLICATION_ACK=commit|journal and related knobs
//   write-behind        MW_REQUEST_PROTECTION_ACCOUNTING=synchronous|write-behind and related knobs
//   request inputs      MW_NATIVE_LOAD_REQUEST_INPUTS_MODE=original|published (fixture switch)
//
// Ranges mirror the server parsers; the Rust fixture (tests/common/t34_modes.rs) validates the same
// selectors before it creates or seeds a database. This module validates them on the harness side.
//
// Required production exposure (NOT present at the time of writing, so those witnesses report
// `supported:false` and an arm that claims the mode is marked invalid):
//   `mw_server::<fixture hook>() -> serde_json::Value` returning, process-global like
//   `request_inputs_fixture::snapshot()`, the objects below, under `t34Modes` in each diagnostics snapshot:
//   guardSnapshot:     {supported:true, mode:'off'|'bounded', maxStalenessMs, polls, failures, rowsApplied,
//                       hits, misses, expedites}      (EpochMirror::counters(); hits = reuses of a certified
//                       snapshot, misses = fallbacks to the fresh path)
//   requestProtection: {supported:true, accounting:'synchronous'|'write-behind', pending, flushed, batches,
//                       writeThroughs, writeThroughFailures, flushFailures, rejected, dropped, loads,
//                       degraded}                      (WriteBehindStatus, serialized camelCase)
// Journal evidence already exists: `searchIndex.publication` (PublicationStatus) carries ackMode, earlyAck,
// journalRows, journalAcknowledgedRows, journalAckFailures, journalReplayedKeys, readFenceWaits/Timeouts/Refusals.
import {readFile} from 'node:fs/promises';
import {pathToFileURL} from 'node:url';
import {requestInputsExpected,requestInputsGroupConsumes,requestInputsBackendSkipsPublished} from './request-inputs-experiment.mjs';
import {indexPublicationReport} from './index-publication-health.mjs';

const word=(name,values)=>({name,kind:'word',values});
const int=(name,min,max)=>({name,kind:'int',min,max});
export const SELECTORS=Object.freeze([
 word('MW_GUARD_SNAPSHOT',['off','bounded']),int('MW_GUARD_SNAPSHOT_MAX_STALENESS_MS',100,5000),
 word('MW_INDEX_PUBLICATION_ACK',['commit','journal']),int('MW_INDEX_PUBLICATION_WINDOW_MS',0,10),
 int('MW_INDEX_PUBLICATION_MAX_BATCH_JOBS',1,128),int('MW_INDEX_READ_FENCE_TIMEOUT_MS',100,30000),int('MW_INDEX_PUBLICATION_DRAIN_SECS',0,60),
 word('MW_REQUEST_PROTECTION_ACCOUNTING',['synchronous','write-behind']),int('MW_REQUEST_PROTECTION_FLUSH_MS',50,5000),int('MW_REQUEST_PROTECTION_MAX_PENDING',64,65536),
 word('MW_NATIVE_LOAD_REQUEST_INPUTS_MODE',['original','published']),word('MW_LOAD_FIXTURE_REQUEST_INPUTS_EXPECT',['original','published']),
]);
// A tuning knob without its mode would be a silent no-op, so it is refused.
const REQUIRES=Object.freeze({
 MW_GUARD_SNAPSHOT_MAX_STALENESS_MS:['MW_GUARD_SNAPSHOT','bounded'],
 MW_INDEX_READ_FENCE_TIMEOUT_MS:['MW_INDEX_PUBLICATION_ACK','journal'],
 MW_REQUEST_PROTECTION_FLUSH_MS:['MW_REQUEST_PROTECTION_ACCOUNTING','write-behind'],
 MW_REQUEST_PROTECTION_MAX_PENDING:['MW_REQUEST_PROTECTION_ACCOUNTING','write-behind'],
});
const unset=value=>value===undefined||value==='';
/** Validate and normalize selector values. Throws a message naming the variable. Unknown t34 names are ignored. */
export function parseT34Selection(env=process.env){
 const out={};
 for(const sel of SELECTORS){
  const raw=env[sel.name];if(unset(raw))continue;
  if(typeof raw!=='string')throw new Error(`${sel.name} must be a string`);
  if(sel.kind==='word'){if(!sel.values.includes(raw))throw new Error(`${sel.name}=${JSON.stringify(raw)} is invalid; use one of ${sel.values.join(', ')}`);}
  else if(!/^[0-9]{1,6}$/.test(raw)||Number(raw)<sel.min||Number(raw)>sel.max)throw new Error(`${sel.name}=${JSON.stringify(raw)} is invalid; use an integer from ${sel.min} to ${sel.max}`);
  out[sel.name]=raw;
 }
 for(const [knob,[mode,value]] of Object.entries(REQUIRES))if(out[knob]!==undefined&&out[mode]!==value)throw new Error(`${knob} requires ${mode}=${value}`);
 const a=out.MW_NATIVE_LOAD_REQUEST_INPUTS_MODE,b=out.MW_LOAD_FIXTURE_REQUEST_INPUTS_EXPECT;
 if(a!==undefined&&b!==undefined&&a!==b)throw new Error('MW_LOAD_FIXTURE_REQUEST_INPUTS_EXPECT must equal MW_NATIVE_LOAD_REQUEST_INPUTS_MODE');
 requestInputsExpected(b);
 return {env:out,modes:{
  guardSnapshot:out.MW_GUARD_SNAPSHOT??'off',indexAck:out.MW_INDEX_PUBLICATION_ACK??'commit',
  requestProtection:out.MW_REQUEST_PROTECTION_ACCOUNTING??'synchronous',requestInputs:a??b??'published'},
  // bounded snapshot rides on the published request inputs; under `original` it may be inert, which the evidence will show
  warnings:out.MW_GUARD_SNAPSHOT==='bounded'&&(a==='original')?['MW_GUARD_SNAPSHOT=bounded with request inputs `original`: the snapshot may be unused; the activity counters decide']:[]};
}

// ---- arms -----------------------------------------------------------------------------------
const SNAP={MW_GUARD_SNAPSHOT:'bounded'},JOURNAL={MW_INDEX_PUBLICATION_ACK:'journal'},WB={MW_REQUEST_PROTECTION_ACCOUNTING:'write-behind'};
const inputs=mode=>({MW_NATIVE_LOAD_REQUEST_INPUTS_MODE:mode,MW_LOAD_FIXTURE_REQUEST_INPUTS_EXPECT:mode});
// Explicit values everywhere (also for "off"), so the effective value is recorded and compared.
export const ARMS=Object.freeze({
 baseline:{description:'all t34 modes off, request inputs published (production default)',env:{MW_GUARD_SNAPSHOT:'off',MW_INDEX_PUBLICATION_ACK:'commit',MW_REQUEST_PROTECTION_ACCOUNTING:'synchronous',...inputs('published')}},
 original:{description:'request inputs original, t34 modes off',env:{MW_GUARD_SNAPSHOT:'off',MW_INDEX_PUBLICATION_ACK:'commit',MW_REQUEST_PROTECTION_ACCOUNTING:'synchronous',...inputs('original')}},
 snapshot:{description:'guard snapshot bounded only',env:{...SNAP,MW_INDEX_PUBLICATION_ACK:'commit',MW_REQUEST_PROTECTION_ACCOUNTING:'synchronous',...inputs('published')}},
 journal:{description:'journaled index acknowledgement only',env:{MW_GUARD_SNAPSHOT:'off',...JOURNAL,MW_REQUEST_PROTECTION_ACCOUNTING:'synchronous',...inputs('published')}},
 'write-behind':{description:'write-behind request-protection accounting only',env:{MW_GUARD_SNAPSHOT:'off',MW_INDEX_PUBLICATION_ACK:'commit',...WB,...inputs('published')}},
 all:{description:'all three t34 modes, request inputs published',env:{...SNAP,...JOURNAL,...WB,...inputs('published')}},
 'all-original':{description:'all three t34 modes plus request inputs original',env:{...SNAP,...JOURNAL,...WB,...inputs('original')}},
});
export function armSelection(name){
 const arm=ARMS[name];if(!arm)throw new Error(`Unknown arm ${JSON.stringify(name)}; use ${Object.keys(ARMS).join(', ')}`);
 return parseT34Selection(arm.env);
}
/** `KEY=value` lines for a Docker env file. */
export function armEnvLines(name){const {env}=armSelection(name);return Object.entries(env).map(([k,v])=>`${k}=${v}`);}

// ---- evidence ---------------------------------------------------------------------------------
// Endpoint groups whose phases write (journal activity must be shown for them).
export const MUTATING_ENDPOINTS=Object.freeze(['representative','flag-write','move-write','goal-65-35']);
const MUTATING=new Set(MUTATING_ENDPOINTS);
// Per endpoint group: does the group's routes consult the bounded guard snapshot (hit or miss)?
// Observed live: the /api/account/* polls never consult it. Everything else is expected to,
// and so is any group not listed (strict by default). Published request inputs are decided by
// requestInputsGroupConsumes (request-inputs-experiment.mjs), the same known subset the receipt uses.
export const GUARD_SNAPSHOT_LOOKUPS=Object.freeze({mixed:true,identity:true,mailboxes:true,query:true,headers:true,refresh:true,representative:true,'paged-headers':true,reader:true,attachment:true,'flag-write':true,'move-write':true,
 'jmap-session':true,'list-batch':true,changes:true,'account-polls':false,
 // Bench-rig groups (t62-n12): the transport control never reaches a guard; the two account polls alone never consult it.
 'transport-healthz':false,'goal-shape-account-preferences':false,'goal-shape-appearance-customization':false});
const expectsLookups=endpoint=>GUARD_SNAPSHOT_LOOKUPS[endpoint]!==false;
const GUARD_COUNTERS=['polls','failures','rowsApplied','hits','misses','expedites'];
const WB_COUNTERS=['flushed','batches','writeThroughs','writeThroughFailures','flushFailures','rejected','dropped','loads'];
const nonneg=v=>Number.isSafeInteger(v)&&v>=0;
function guardWitness(value){
 if(value===undefined||value===null)return {supported:false,reason:'missing'};
 if(value.supported!==true)return {supported:false,reason:String(value.reason??'unsupported').slice(0,120)};
 if(!['off','bounded'].includes(value.mode)||GUARD_COUNTERS.some(k=>!nonneg(value[k])))return {supported:false,reason:'malformed'};
 return {supported:true,mode:value.mode,maxStalenessMs:value.maxStalenessMs??null,...Object.fromEntries(GUARD_COUNTERS.map(k=>[k,value[k]]))};
}
function protectionWitness(value){
 if(value===undefined||value===null)return {supported:false,reason:'missing'};
 if(value.supported!==true)return {supported:false,reason:String(value.reason??'unsupported').slice(0,120)};
 if(!['synchronous','write-behind'].includes(value.accounting)||!nonneg(value.pending)||typeof value.degraded!=='boolean'||WB_COUNTERS.some(k=>!nonneg(value[k])))return {supported:false,reason:'malformed'};
 return {supported:true,accounting:value.accounting,pending:value.pending,degraded:value.degraded,...Object.fromEntries(WB_COUNTERS.map(k=>[k,value[k]]))};
}
function publication(snapshot){try{return indexPublicationReport(snapshot?.searchIndex?.publication)??null;}catch{return null;}}
const sum=(rows,pick)=>rows.reduce((t,r)=>t+pick(r),0);
const delta=(a,b)=>(a??0)-(b??0);

/** Per stage before/after witnesses, tolerant of legacy reports (everything absent => unsupported). */
function observe(stage){
 const before=stage?.diagnostics?.before,after=stage?.diagnostics?.after;
 return {endpoint:stage?.endpoint,phase:stage?.phase,concurrency:stage?.concurrency??null,accepted:stage?.accepted??0,requests:stage?.requests??0,
  guard:{before:guardWitness(before?.t34Modes?.guardSnapshot),after:guardWitness(after?.t34Modes?.guardSnapshot)},
  protection:{before:protectionWitness(before?.t34Modes?.requestProtection),after:protectionWitness(after?.t34Modes?.requestProtection)},
  publication:{before:publication(before),after:publication(after)}};
}
function monotonic(list,counters){return list.every(o=>o.before.supported&&o.after.supported&&counters.every(k=>o.after[k]>=o.before[k]));}

function guardVerdict(expected,rows){
 const reasons=[],deltas=Object.fromEntries(GUARD_COUNTERS.map(k=>[k,sum(rows,r=>delta(r.guard.after[k],r.guard.before[k]))]));
 const observed=rows.map(r=>r.guard),supported=observed.length>0&&observed.every(o=>o.before.supported&&o.after.supported);
 if(supported&&!monotonic(observed,GUARD_COUNTERS))reasons.push('guard snapshot counters went backwards (process restarted?)');
 if(expected==='bounded'){
  if(!supported)reasons.push('guard snapshot witness unsupported or missing: the mode cannot be shown active');
  else{
   if(observed.some(o=>o.before.mode!=='bounded'||o.after.mode!=='bounded'))reasons.push('server reports the guard snapshot is not bounded');
   if(deltas.polls<=0)reasons.push('zero mirror polls');
   if(deltas.hits<=0&&rows.some(r=>r.accepted>0&&expectsLookups(r.endpoint)))reasons.push('zero snapshot reuses (hits)');
   for(const r of rows)if(r.accepted>0&&expectsLookups(r.endpoint)&&delta(r.guard.after.hits,r.guard.before.hits)+delta(r.guard.after.misses,r.guard.before.misses)<=0)reasons.push(`no snapshot lookups in ${r.endpoint} c=${r.concurrency}`);
   if(deltas.failures>0&&deltas.hits===0)reasons.push('mirror polls failed and nothing was reused');
  }
  return {expected,status:reasons.length?'invalid':'active',reasons,deltas,fallbacksToFreshPath:deltas.misses,reuses:deltas.hits};
 }
 if(supported){
  if(observed.some(o=>o.before.mode!=='off'||o.after.mode!=='off'))reasons.push('server reports the guard snapshot on although the arm is off');
  for(const k of ['polls','hits','misses','rowsApplied','expedites'])if(deltas[k]>0)reasons.push(`off arm recorded guard snapshot work (${k}=${deltas[k]})`);
  return {expected,status:reasons.length?'invalid':'off-verified',reasons,deltas};
 }
 return {expected,status:'off-unwitnessed',reasons,deltas};
}

function journalVerdict(expected,rows){
 const reasons=[],pubs=rows.map(r=>r.publication),have=pubs.length>0&&pubs.every(p=>p.before?.ackMode&&p.after?.ackMode);
 const d=k=>sum(rows,r=>delta(r.publication.after?.[k],r.publication.before?.[k]));
 const deltas=have?{acknowledgedRows:d('journalAcknowledgedRows'),ackFailures:d('journalAckFailures'),replayedKeys:d('journalReplayedKeys'),readFenceWaits:d('readFenceWaits'),readFenceTimeouts:d('readFenceTimeouts'),readFenceRefusals:d('readFenceRefusals'),submittedJobs:d('submittedJobs'),completedJobs:d('completedJobs')}:null;
 const last=pubs.at(-1)?.after,first=pubs[0]?.before;
 const mutating=rows.filter(r=>MUTATING.has(r.endpoint)&&r.accepted>0);
 if(!have)reasons.push(expected==='journal'?'index publication journal evidence missing':'index publication acknowledgement mode not reported');
 else if(expected==='journal'){
  if(pubs.some(p=>p.before.ackMode!=='journal'||p.after.ackMode!=='journal'))reasons.push('server reports acknowledgement mode other than journal');
  if(pubs.some(p=>!p.before.earlyAck||!p.after.earlyAck))reasons.push('early acknowledgement is not active (trigger capture not enabled)');
  if(pubs.some(p=>p.before.journalReplayPending||p.after.journalReplayPending))reasons.push('startup journal replay still pending');
  if(first.journalRows!==0)reasons.push(`journal rows at the readiness gate is ${first.journalRows}, not 0`);
  if(deltas.ackFailures>0)reasons.push(`journal acknowledgement failures (${deltas.ackFailures})`);
  if(deltas.readFenceTimeouts>0)reasons.push(`read fence timeouts (${deltas.readFenceTimeouts})`);
  if(!mutating.length)reasons.push('workload has no mutating endpoint: journal activity cannot be shown');
  else if(deltas.acknowledgedRows<=0||deltas.submittedJobs<=0)reasons.push('mutating phases ran but zero journal rows were applied');
 }else{
  if(pubs.some(p=>p.before.ackMode!=='commit'||p.after.ackMode!=='commit'||p.before.earlyAck||p.after.earlyAck))reasons.push('server reports journal acknowledgement although the arm is commit');
  for(const k of ['acknowledgedRows','readFenceWaits','readFenceTimeouts'])if(deltas[k]>0)reasons.push(`commit arm recorded journal work (${k}=${deltas[k]})`);
 }
 return {expected,status:reasons.length?'invalid':expected==='journal'?'active':'off-verified',reasons,deltas,
  rowsPending:last?.journalRows??null,rowsAtReadinessGate:first?.journalRows??null,rowsWritten:deltas?deltas.acknowledgedRows+(last?.journalRows??0):null,mutatingStages:mutating.length};
}

function protectionVerdict(expected,rows){
 const reasons=[],obs=rows.map(r=>r.protection),supported=obs.length>0&&obs.every(o=>o.before.supported&&o.after.supported);
 const deltas=Object.fromEntries(WB_COUNTERS.map(k=>[k,sum(rows,r=>delta(r.protection.after[k],r.protection.before[k]))]));
 const last=obs.at(-1)?.after;
 if(supported&&!monotonic(obs,WB_COUNTERS))reasons.push('write-behind counters went backwards (process restarted?)');
 if(expected==='write-behind'){
  if(!supported)reasons.push('write-behind witness unsupported or missing: the mode cannot be shown active');
  else{
   if(obs.some(o=>o.before.accounting!=='write-behind'||o.after.accounting!=='write-behind'))reasons.push('server reports accounting other than write-behind');
   if(deltas.flushed+deltas.writeThroughs<=0)reasons.push('zero observations flushed or written through');
   if(deltas.writeThroughFailures>0||deltas.flushFailures>0)reasons.push('write-behind storage failures');
   if(obs.some(o=>o.after.degraded))reasons.push('write-behind accounting degraded');
  }
  const warnings=[];if(deltas.dropped>0)warnings.push(`${deltas.dropped} observations dropped (queue full)`);if(deltas.rejected>0)warnings.push(`${deltas.rejected} observations rejected`);
  return {expected,status:reasons.length?'invalid':'active',reasons,warnings,deltas,queued:supported?deltas.flushed+deltas.dropped+deltas.rejected+(last?.pending??0):null,pendingAtEnd:last?.pending??null};
 }
 if(supported){
  if(obs.some(o=>o.before.accounting!=='synchronous'||o.after.accounting!=='synchronous'))reasons.push('server reports write-behind although the arm is synchronous');
  for(const k of ['flushed','batches','writeThroughs','dropped','rejected'])if(deltas[k]>0)reasons.push(`synchronous arm recorded write-behind work (${k}=${deltas[k]})`);
  return {expected,status:reasons.length?'invalid':'off-verified',reasons,deltas};
 }
 return {expected,status:'off-unwitnessed',reasons,deltas};
}

function inputsVerdict(expected,evidence,rows=[]){
 if(expected===undefined)return {expected:null,status:'unclaimed',reasons:[]};
 const reasons=[];
 if(!evidence)reasons.push('request-input evidence missing');
 else{
  if(evidence.expectedMode!==expected||evidence.startup?.mode!==expected)reasons.push(`request inputs mode is not ${expected}`);
  // Published inputs exist only on SQLite; a handoff naming another backend must consume none.
  const noPublish=requestInputsBackendSkipsPublished(evidence.backend==='sqlite-or-unnamed'?undefined:evidence.backend);
  const needed=rows.length===0||rows.some(r=>r.accepted>0&&requestInputsGroupConsumes(r.endpoint));
  if(expected==='published'&&noPublish){if(evidence.consumedFramesDelta!==0)reasons.push('published request inputs consumed frames on a backend that does not publish them');}
  else if(expected==='published'&&needed&&!(evidence.consumedFramesDelta>0))reasons.push('published request inputs consumed zero frames');
  if(expected==='original'&&evidence.consumedFramesDelta!==0)reasons.push('original request inputs recorded candidate work');
 }
 return {expected,status:reasons.length?'invalid':evidence&&expected==='published'&&requestInputsBackendSkipsPublished(evidence.backend==='sqlite-or-unnamed'?undefined:evidence.backend)?'not-applicable-backend':'active',reasons,consumedFramesDelta:evidence?.consumedFramesDelta??null};
}

/** Statements per HTTP request from request-profile query counters; null (labelled) when uninstrumented. */
export function sqlStatementsPerHttp(stages){
 const per=[];let total=0,requests=0,instrumented=true;
 for(const s of stages){
  const b=s?.diagnostics?.before?.requestProfile,a=s?.diagnostics?.after?.requestProfile;
  const on=Boolean(a?.enabled&&b?.enabled&&a.queries&&b.queries);
  if(!on){instrumented=false;per.push({endpoint:s?.endpoint,concurrency:s?.concurrency??null,statementsPerHttp:null});continue;}
  const calls=k=>delta(a.queries[k]?.calls,b.queries[k]?.calls);
  const statements=calls('read')+calls('write')+calls('other'),n=Math.max(1,s.requests??0);
  total+=statements;requests+=n;per.push({endpoint:s.endpoint,concurrency:s.concurrency??null,statements,requests:s.requests??0,statementsPerHttp:+(statements/n).toFixed(2)});
 }
 return {instrumented:instrumented&&stages.length>0,label:instrumented&&stages.length>0?'instrumented (request profile on): process-wide statement counter including background work; not an uninstrumented throughput run':'uninstrumented run: no per-request SQL counts',
  statementsPerHttp:instrumented&&requests?+(total/requests).toFixed(2):null,stages:per};
}

/** `handoff` is the fixture's `t34Modes` block; returns null for legacy handoffs without one. */
export function t34ModesEvidence(handoff,stages,{requestInputsEvidence}={}){
 if(handoff===undefined||handoff===null)return null;
 const req=handoff.requested;
 const claimed={guardSnapshot:req?.guardSnapshot?.mode,indexAck:req?.indexPublication?.ack,requestProtection:req?.requestProtection?.accounting};
 const invalid=reason=>({schema:1,requested:req??null,startup:null,valid:false,invalidReasons:[reason],modes:{},sqlPerHttp:{instrumented:false,statementsPerHttp:null,stages:[]},stages:[]});
 if(handoff.schema!==1||!['off','bounded'].includes(claimed.guardSnapshot)||!['commit','journal'].includes(claimed.indexAck)||!['synchronous','write-behind'].includes(claimed.requestProtection))return invalid('invalid t34 mode handoff');
 if(!Array.isArray(stages)||!stages.length)return invalid('no measured stages: no t34 mode evidence');
 const rows=stages.map(observe);
 const modes={guardSnapshot:guardVerdict(claimed.guardSnapshot,rows),indexPublication:journalVerdict(claimed.indexAck,rows),requestProtection:protectionVerdict(claimed.requestProtection,rows),
  requestInputs:inputsVerdict(requestInputsEvidence?.expectedMode??handoff.requestInputsExpected,requestInputsEvidence,rows)};
 const reasons=Object.entries(modes).flatMap(([name,v])=>v.reasons.map(r=>`${name}: ${r}`));
 const sql=sqlStatementsPerHttp(stages);
 return {schema:1,requested:req,startup:handoff.startup??null,valid:reasons.length===0,invalidReasons:reasons,modes,sqlPerHttp:sql,
  stages:rows.map(r=>({endpoint:r.endpoint,phase:r.phase,concurrency:r.concurrency,accepted:r.accepted,
   guard:r.guard.after.supported?{hits:delta(r.guard.after.hits,r.guard.before.hits),misses:delta(r.guard.after.misses,r.guard.before.misses),polls:delta(r.guard.after.polls,r.guard.before.polls)}:null,
   journal:r.publication.after?.ackMode?{acknowledgedRows:delta(r.publication.after.journalAcknowledgedRows,r.publication.before?.journalAcknowledgedRows),readFenceWaits:delta(r.publication.after.readFenceWaits,r.publication.before?.readFenceWaits),readFenceTimeouts:delta(r.publication.after.readFenceTimeouts,r.publication.before?.readFenceTimeouts)}:null,
   writeBehind:r.protection.after.supported?{flushed:delta(r.protection.after.flushed,r.protection.before.flushed),writeThroughs:delta(r.protection.after.writeThroughs,r.protection.before.writeThroughs),dropped:delta(r.protection.after.dropped,r.protection.before.dropped),rejected:delta(r.protection.after.rejected,r.protection.before.rejected)}:null}))};
}

/** Does a finished report belong to the arm it claims, and is it valid? */
export function checkReport(report,armName){
 const expected=armSelection(armName),reasons=[];
 const t=report?.t34Modes;
 if(!t)return {arm:armName,valid:false,reasons:['report has no t34Modes evidence (legacy fixture)']};
 const q=t.requested??{};
 const same=[['guard snapshot',q.guardSnapshot?.mode,expected.modes.guardSnapshot],['index acknowledgement',q.indexPublication?.ack,expected.modes.indexAck],['protection accounting',q.requestProtection?.accounting,expected.modes.requestProtection]];
 for(const [name,got,want] of same)if(got!==want)reasons.push(`${name}: fixture requested ${got}, arm needs ${want}`);
 if(report.ordinaryRequestInputs?.expectedMode!==expected.modes.requestInputs)reasons.push(`request inputs: report expects ${report.ordinaryRequestInputs?.expectedMode??'nothing'}, arm needs ${expected.modes.requestInputs}`);
 if(!t.valid)reasons.push(...t.invalidReasons);
 return {arm:armName,valid:reasons.length===0,reasons,instrumented:t.sqlPerHttp?.instrumented??false,sqlStatementsPerHttp:t.sqlPerHttp?.statementsPerHttp??null};
}

// ---- campaign -------------------------------------------------------------------------------
export const CAMPAIGN_SHAPE=Object.freeze({users:100,messagesPerUser:50,cpus:4,cpuset:'0-3',memory:'4g',securityProfile:'enforced',policy:'loaded',workload:'representative',
 endpoints:['identity','query','headers','representative'],concurrency:[8,32,64],seconds:30,warmupSeconds:20,repeats:2,generator:{cpus:2,cpuset:'8-9',memory:'1g',workers:2}});
/** Alternating run order: A B B A for two arms, generalized as forward then reverse sweeps per repeat pair. */
export function runOrder(arms,repeats){
 const out=[];
 for(let r=0;r<repeats;r++){const sweep=r%2===0?arms:[...arms].reverse();for(const arm of sweep)out.push({arm,repeat:r+1});}
 return out;
}
export function buildCampaign(){
 const armNames=Object.keys(ARMS),runs=runOrder(armNames,CAMPAIGN_SHAPE.repeats).map((x,i)=>({id:`m${String(i+1).padStart(2,'0')}`,...x,requestProfile:false,env:ARMS[x.arm].env}));
 // One instrumented run per arm (separately labelled) supplies SQL statements per HTTP; never mixed into the RPS comparison.
 const instrumented=armNames.map((arm,i)=>({id:`p${String(i+1).padStart(2,'0')}`,arm,repeat:1,requestProfile:true,env:ARMS[arm].env,endpoints:['identity','query','representative'],concurrency:[8]}));
 const ws=['off','bounded'].map((mode,i)=>({id:`w${i+1}`,arm:mode==='off'?'baseline':'snapshot',connections:1000,rampPerSecond:100,holdSeconds:200,principals:10000,workload:'baseline (2 messages/account)',memory:'8g',env:{...ARMS[mode==='off'?'baseline':'snapshot'].env}}));
 return {schema:1,description:'t34 A/B: each mode alone, all combined, all combined with original request inputs; same shape as the 2026-11-23 baseline',
  shape:CAMPAIGN_SHAPE,arms:Object.fromEntries(Object.entries(ARMS).map(([k,v])=>[k,{description:v.description,env:v.env}])),
  evidence:{requiredPerArm:'every run must carry t34Modes.valid=true; check with `node scripts/perf/t34-modes.mjs check <native-100.json> --arm <arm>`',invalidMeans:'a claimed mode showed no activity, an off arm showed activity, or the witness is unsupported'},
  runs,instrumentedRuns:instrumented,websocketIdleHold:ws};
}

// ---- CLI ------------------------------------------------------------------------------------
async function main(argv){
 const [cmd,a,b,c]=argv;
 if(cmd==='validate-env'){const s=parseT34Selection(process.env);console.log(JSON.stringify(s));return;}
 if(cmd==='env'){console.log(armEnvLines(a).join('\n'));return;}
 if(cmd==='campaign'){console.log(JSON.stringify(buildCampaign(),null,1));return;}
 if(cmd==='check'){
  if(b!=='--arm'||!c)throw new Error('usage: check <report.json> --arm <name>');
  const result=checkReport(JSON.parse(await readFile(a,'utf8')),c);console.log(JSON.stringify(result,null,1));process.exitCode=result.valid?0:1;return;
 }
 throw new Error('usage: t34-modes.mjs validate-env | env <arm> | campaign | check <report.json> --arm <name>');
}
if(process.argv[1]&&import.meta.url===pathToFileURL(process.argv[1]).href)main(process.argv.slice(2)).catch(e=>{console.error(e.message);process.exitCode=1;});
