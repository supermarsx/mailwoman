// Explicit fixture receipt validation only; never controls application authority.
import {GOAL_MIX_TABLE} from './representative-workload.mjs';
export const requestInputCounters=Object.freeze(['ordinaryContexts','readyFrames','consumedFrames','hintAttempts','buildAttempts','completedBuilds','firstFenceAttempts','refreshFenceAttempts']);
export function requestInputsExpected(value){
 if(value===undefined)return undefined;
 if(value!=='original'&&value!=='published')throw Error('Invalid expected request-input mode');
 return value;
}
export function requestInputsStatusReport(value){
 if(value===undefined)return null;
 if(!value||typeof value!=='object'||Array.isArray(value)||!['original','published'].includes(value.mode)||value.processImmutable!==true||requestInputCounters.some(key=>!Number.isSafeInteger(value[key])||value[key]<0))throw Error('Invalid request-input fixture witness');
 return {mode:value.mode,processImmutable:true,...Object.fromEntries(requestInputCounters.map(key=>[key,value[key]]))};
}
const shapes=Object.freeze({identity:['authenticated-identity'],mailboxes:['Mailbox/get'],query:['Email/query'],headers:['Email/get'],refresh:['concurrent-refresh-handler'],mixed:['authenticated-identity','Mailbox/get','Email/query','Email/get','concurrent-refresh-handler'],representative:['authenticated-identity','Mailbox/get','Email/query','Email/get','concurrent-refresh-handler','paged-query-and-headers','body-and-attachment-metadata','attachment-download','mark-read-and-restore','move-and-restore'],'paged-headers':['paged-query-and-headers'],reader:['body-and-attachment-metadata'],attachment:['attachment-download'],'flag-write':['mark-read-and-restore'],'move-write':['move-and-restore'],'jmap-session':['jmap-session'],'list-batch':['list-batch'],changes:['changes'],'account-polls':['account-preferences','appearance-customization','session-activity'],
 // Goal mix (all 17 shapes), each shape alone (bench rig), and the transport control (t62-n12).
 'goal-65-35':GOAL_MIX_TABLE.map(row=>row.name),...Object.fromEntries(GOAL_MIX_TABLE.map(row=>['goal-shape-'+row.name,[row.name]])),'transport-healthz':['transport-healthz']});
// Conservative known subset only. Email/get labels cover both marked native
// headers and unmarked ordinary headers, which Rust can support; labels alone
// do not distinguish them. Exclusion is not proof of server ineligibility.
const eligible=new Set(['authenticated-identity','Mailbox/get','Email/query','paged-query-and-headers','body-and-attachment-metadata','mark-read-and-restore','move-and-restore']);
// Whether a campaign endpoint group contains at least one request in the conservative
// known subset above, i.e. whether published inputs are expected to be consumed for it.
export function requestInputsGroupConsumes(endpoint){const names=shapes[endpoint];return Array.isArray(names)&&names.some(name=>eligible.has(name));}
const add=(a,b)=>{const sum=a+b;if(!Number.isSafeInteger(sum))throw Error('Request-input receipt counter overflow');return sum;};
// Published inputs are built and consumed only on the SQLite store (the server
// skips them when `!store.uses_sqlite()`, request_inputs.rs). Only a backend the
// fixture handoff names explicitly as something else relaxes the consumption
// assertion; an unnamed or sqlite backend stays strict. The relaxation is itself
// strict the other way: a non-SQLite run must consume nothing.
export function requestInputsBackendSkipsPublished(backend){
 if(backend===undefined||backend===null)return false;
 if(typeof backend!=='string'||!/^[a-z][a-z0-9_-]{0,31}$/.test(backend))throw Error('Invalid request-input backend');
 return backend!=='sqlite';
}
// A stage ran with a live guard snapshot mirror when the fixture's t34 witness reports bounded mode at both
// edges and the mirror polled during the stage without new poll failures.
export function mirrorLive(stage){
 const a=stage?.diagnostics?.before?.t34Modes?.guardSnapshot,b=stage?.diagnostics?.after?.t34Modes?.guardSnapshot;
 const n=v=>Number.isSafeInteger(v)&&v>=0;
 return Boolean(a?.supported===true&&b?.supported===true&&a.mode==='bounded'&&b.mode==='bounded'&&n(a.polls)&&n(b.polls)&&b.polls>a.polls&&n(a.failures)&&n(b.failures)&&b.failures===a.failures);
}
export function requestInputsEvidence(handoff,stages,expected,{backend}={}){
 expected=requestInputsExpected(expected);
 const skipsPublished=requestInputsBackendSkipsPublished(backend);
 // Legacy c19 or unqualified runs retain their previous contract. Merely
 // receiving a new-server witness never opts a run into an A/B assertion.
 if(expected===undefined)return null;
 const startup=requestInputsStatusReport(handoff);
 if(!startup||startup.mode!==expected)throw Error('Missing matching request-input startup mode');
 const check=value=>{const status=requestInputsStatusReport(value);if(!status||status.mode!==expected)throw Error('Missing matching request-input phase mode');if(expected==='original'&&requestInputCounters.some(key=>status[key]!==0))throw Error('Original request-input path recorded candidate work');return status;};
 check(startup);
 if(!Array.isArray(stages)||!stages.length)throw Error('Missing request-input workload observations');
 const observations=[];let liveMirrorPhases=0,eligiblePhases=0,eligibleAccepted=0,consumedFramesDelta=0,previous=startup;
 for(const stage of stages){
  const names=shapes[stage?.endpoint];
  if(!names||!['concurrency-ramp','offered-rate'].includes(stage.phase)||!stage.byRequest||typeof stage.byRequest!=='object'||Array.isArray(stage.byRequest)||Object.keys(stage.byRequest).length!==names.length||Object.keys(stage.byRequest).some(name=>!names.includes(name)))throw Error('Invalid request-input workload shape proof');
  if(stage.requestInputsWindow?.schema!==1||stage.requestInputsWindow.kind!=='workload-and-drain-only'||stage.requestInputsWindow.eligibleControlRequests!==0)throw Error('Unproven request-input workload/control boundary');
  const before=check(stage.diagnostics?.before?.ordinaryRequestInputs),after=check(stage.diagnostics?.after?.ordinaryRequestInputs);
  const deltas={};for(const key of requestInputCounters){if(before[key]<previous[key]||after[key]<before[key])throw Error('Request-input process counter reset');deltas[key]=after[key]-before[key];}
  let accepted=0;
  for(const name of names){const sample=stage.byRequest[name];if(!sample||!Number.isSafeInteger(sample.requests)||sample.requests<0||!Number.isSafeInteger(sample.acceptedDuringScheduledWindow)||sample.acceptedDuringScheduledWindow<0||sample.acceptedDuringScheduledWindow>sample.requests)throw Error('Missing request-input accepted HTTP proof');if(eligible.has(name))accepted=add(accepted,sample.acceptedDuringScheduledWindow);}
  // PostgreSQL publishes inputs only while the guard snapshot mirror is live (bounded mode, cbd54c358):
  // consumption is then expected as on SQLite; without a live mirror it must stay zero.
  const live=skipsPublished&&mirrorLive(stage);
  if(accepted>0){
   if(skipsPublished&&!live){if(expected==='published'&&deltas.consumedFrames!==0)throw Error('Published inputs were consumed on a backend that does not publish them without a live guard snapshot mirror');}
   else eligiblePhases++;
  }
  if(live)liveMirrorPhases++;
  eligibleAccepted=add(eligibleAccepted,accepted);consumedFramesDelta=add(consumedFramesDelta,deltas.consumedFrames);
  // Per-stage consumption is data, not a gate: a frame is ready only while the session's last_seen
  // touch is not due (flush interval, default 15 s) and a built entry exists, so at low per-user rates
  // many requests legitimately take the original path.
  observations.push({endpoint:stage.endpoint,phase:stage.phase,...(skipsPublished?{liveMirror:live}:{}),knownSubsetAcceptedDuringScheduledWindow:accepted,consumedFramesPerContext:deltas.ordinaryContexts>0?+(deltas.consumedFrames/deltas.ordinaryContexts).toFixed(4):null,before,after,deltas});previous=after;
 }
 // The requirement: when published inputs are expected (SQLite, at least one consuming stage) the
 // path must be exercised at all during the run. Partial or zero consumption in one stage is allowed.
 if(expected==='published'&&eligiblePhases>0&&consumedFramesDelta===0)throw Error('Eligible accepted HTTP phases did not consume published inputs in the whole run');
 return {expectedMode:expected,startup,observations,knownSubsetPhases:eligiblePhases,knownSubsetAcceptedDuringScheduledWindow:eligibleAccepted,consumedFramesDelta,qualified:eligiblePhases>0,exercise:eligiblePhases>0?'known-subset-http-observed':skipsPublished?'not-applicable-backend':'unexercised',backend:skipsPublished?backend:'sqlite-or-unnamed',...(skipsPublished?{liveMirrorPhases}:{}),eligibilityScope:'conservative known subset; excludes Email/get labels without proving those requests unsupported',controlAuthenticationBound:0,window:'process before/after includes drain; not in-window consumption or per-request attribution'};
}
