// Bench rig (t62-n12): regression comparison of a candidate run against a committed baseline or a parent-image run.
//
// Rule (standing user order "do your best to not regress"):
//  - deterministic counters regress on ANY increase: statements per request (reads, writes, other = BEGIN and the
//    rest), database commits, upstream commands per request, and the error / 503 / 429 / timeout counts. Per-request
//    counters are compared at a stated resolution: 0.01 when request-scoped, 0.1 when process-wide (those include
//    background work and the drain after the window, so their second decimal is not reproducible).
//  - noisy numbers (µs CPU per request, rps, rps per consumed CPU, p50, p99) regress only when worse by more than the
//    threshold (default 8%) in EVERY repeat, and only when there are at least two repeats (the campaign's A B B A
//    rule). With one repeat a worse value is reported as `unconfirmed`, never as a regression.
//  - a contaminated stage (quiet-host witness) is not compared on noisy numbers.
// The baseline file is never changed except by an explicit bless with a reason, which is recorded in the file.
import {readFile,writeFile} from 'node:fs/promises';

export const DEFAULT_THRESHOLD=0.08;
export const COUNTERS=Object.freeze(['reads','writes','other','databaseCommits','upstreamCommands','errors','http503','http429','timeouts','oomEvents']);
export const NOISY=Object.freeze({usecCpuPerRequest:'higher-is-worse',rps:'lower-is-worse',rpsPerConsumedCpu:'lower-is-worse',p50:'higher-is-worse',p99:'higher-is-worse',appMemoryEndBytes:'higher-is-worse'});
// Per-metric thresholds over the default: app memory at the stage end (steady state) 10% (t63 memory audit).
export const METRIC_THRESHOLDS=Object.freeze({appMemoryEndBytes:0.10});
const PER_REQUEST=new Set(['reads','writes','other','databaseCommits','upstreamCommands']);

const num=v=>Number.isFinite(v)?v:null;
// Tracked numbers of one rig summary row (rig-shapes legRows) in the baseline's vocabulary.
export function metricsOfRow(row){
 const d=row.database?.available?row.database:null,p=d?.perRequest??{};
 const statuses=row.statuses??{};
 const app=Object.values(row.memory?.containers??{}).find(c=>c.role==='app'&&c.available);
 return {source:d?.source??null,
  reads:num(p.reads),writes:num(p.writes),other:num(p.other),databaseCommits:d?.commitsAvailable?num(p.databaseCommits):null,upstreamCommands:num(p.upstreamCommands),
  errors:num(row.errors),http503:num(row.http503??statuses['503']??null),http429:num(row.throttled),timeouts:num(row.timeouts),
  usecCpuPerRequest:num(row.usecCpuPerRequest?.server),rps:num(row.acceptedRps),rpsPerConsumedCpu:num(row.rpsPerLogicalCpu?.consumedServer),
  p50:num(row.latencyMs?.p50),p99:num(row.latencyMs?.p99),appMemoryEndBytes:num(app?.currentEndBytes),oomEvents:app?num((app.events?.oom??0)+(app.events?.oom_kill??0)):null,contaminated:row.contaminated===true};
}
// summary.json -> {backend:{shape:metrics}}; plain (not instrumented) rows only, first row per shape.
export function metricsOfSummary(summary){
 const out={};
 for(const row of summary?.rows??[]){
  if(row.instrumented||!row.shape)continue;
  const backend=summary.backend??'sqlite';out[backend]??={};
  out[backend][row.shape]??=metricsOfRow(row);
 }
 return out;
}

function counterVerdict(name,reference,candidate){
 const resolution=PER_REQUEST.has(name)?(candidate.source==='request-scoped'||candidate.source==='request-profile'?0.01:0.1):1;
 const q=v=>Math.round(v/resolution)*resolution;
 const a=num(reference[name]),b=num(candidate[name]);
 if(a===null||b===null)return {metric:name,kind:'counter',reference:a,candidate:b,verdict:'not-compared',note:a===null?'no reference value':'no candidate value'};
 const worse=q(b)>q(a)+resolution/1000;
 return {metric:name,kind:'counter',reference:a,candidate:b,resolution,verdict:worse?'regression':q(b)<q(a)-resolution/1000?'improved':'same'};
}
function noisyVerdict(name,references,candidates,threshold){
 const direction=NOISY[name],pairs=candidates.map((c,i)=>[references[Math.min(i,references.length-1)],c]);
 const changes=pairs.map(([r,c])=>{const a=num(r?.[name]),b=num(c?.[name]);if(a===null||b===null||a===0||r.contaminated||c.contaminated)return null;return (b-a)/a;});
 const usable=changes.filter(v=>v!==null);
 const worse=v=>direction==='higher-is-worse'?v>threshold:v< -threshold,better=v=>direction==='higher-is-worse'?v< -threshold:v>threshold;
 const base={metric:name,kind:'noisy',direction,threshold,reference:pairs.map(([r])=>num(r?.[name])),candidate:pairs.map(([,c])=>num(c?.[name])),changes:changes.map(v=>v===null?null:Number(v.toFixed(4)))};
 if(!usable.length)return {...base,verdict:'not-compared',note:pairs.some(([r,c])=>r?.contaminated||c?.contaminated)?'contaminated stage':'no value'};
 if(usable.every(worse))return {...base,verdict:usable.length>=2&&usable.length===changes.length?'regression':'unconfirmed',...(usable.length<2?{note:'one repeat: a noisy regression needs two alternating repeats that agree'}:{})};
 if(usable.length>=2&&usable.every(better))return {...base,verdict:'improved'};
 return {...base,verdict:'same'};
}

// references / candidates: arrays of {backend:{shape:metrics}} (one per repeat). A baseline file is one reference.
export function compare(references,candidates,{threshold=DEFAULT_THRESHOLD}={}){
 if(!Array.isArray(references)||!references.length||!Array.isArray(candidates)||!candidates.length)throw new Error('compare needs at least one reference and one candidate');
 if(!(threshold>0&&threshold<1))throw new Error('Invalid threshold');
 const rows=[];
 for(const backend of Object.keys(candidates[0])){
  for(const shape of Object.keys(candidates[0][backend])){
   const refs=references.map(r=>r[backend]?.[shape]).filter(Boolean),cands=candidates.map(c=>c[backend]?.[shape]).filter(Boolean);
   if(!refs.length){rows.push({backend,shape,verdict:'no-reference',checks:[]});continue;}
   const checks=[...COUNTERS.map(name=>counterVerdict(name,refs[0],cands[0])),...Object.keys(NOISY).map(name=>noisyVerdict(name,refs,cands,METRIC_THRESHOLDS[name]??threshold))];
   // A counter must not increase in ANY candidate repeat.
   for(const extra of cands.slice(1))for(const name of COUNTERS){const v=counterVerdict(name,refs[0],extra);if(v.verdict==='regression')checks.push({...v,repeat:cands.indexOf(extra)+1});}
   const verdict=checks.some(c=>c.verdict==='regression')?'REGRESSION':checks.some(c=>c.verdict==='unconfirmed')?'unconfirmed':checks.some(c=>c.verdict==='improved')?'improved':'same';
   rows.push({backend,shape,verdict,checks});
  }
 }
 return {schema:1,threshold,repeats:candidates.length,rows,regressions:rows.filter(r=>r.verdict==='REGRESSION').length,
  rule:'counters (including OOM events): any increase; noisy (including app memory at the stage end): worse than the threshold in every one of at least two alternating repeats'};
}

const fmt=v=>v===null||v===undefined?'–':Array.isArray(v)?v.map(fmt).join(' / '):typeof v==='number'?String(Number(v.toFixed(2))):String(v);
export function markdownComparison(result){
 const lines=[`Comparison: ${result.repeats} candidate repeat(s); noisy threshold ${Math.round(result.threshold*100)}% (app memory at stage end ${Math.round(METRIC_THRESHOLDS.appMemoryEndBytes*100)}%); ${result.regressions} shape(s) regressed.`,'',
  '| Backend | Shape | Verdict | Regressed / unconfirmed metrics (reference -> candidate) |','| --- | --- | --- | --- |'];
 for(const row of result.rows){
  const flagged=row.checks.filter(c=>c.verdict==='regression'||c.verdict==='unconfirmed').map(c=>`${c.metric}${c.verdict==='unconfirmed'?' (unconfirmed)':''}${c.repeat?` [repeat ${c.repeat}]`:''}: ${fmt(c.reference)} -> ${fmt(c.candidate)}`);
  lines.push(`| ${row.backend} | ${row.shape} | ${row.verdict} | ${flagged.join('; ')||'–'} |`);
 }
 lines.push('',`Rule: ${result.rule}.`);
 return lines.join('\n')+'\n';
}

export function validateBaseline(value){
 if(value?.schema!==1||!value.rows||typeof value.rows!=='object'||!Array.isArray(value.blessed)||!value.blessed.length)throw new Error('Invalid baseline file');
 for(const [backend,shapes] of Object.entries(value.rows))for(const [shape,m] of Object.entries(shapes))if(!m||typeof m!=='object')throw new Error(`Invalid baseline row ${backend}/${shape}`);
 return value;
}
export async function readBaseline(path){return validateBaseline(JSON.parse(await readFile(path,'utf8')));}
// Replace the baseline rows of the candidate's backends with the candidate's numbers. Only with a reason.
export function bless(baseline,candidate,{reason,from,now=new Date()}){
 if(typeof reason!=='string'||reason.trim().length<10)throw new Error('--bless needs --reason="<why the new numbers are the baseline>" (at least 10 characters)');
 const rows={...baseline.rows};
 for(const [backend,shapes] of Object.entries(candidate))rows[backend]={...rows[backend],...Object.fromEntries(Object.entries(shapes).map(([shape,m])=>[shape,Object.fromEntries(Object.entries(m).filter(([k])=>k!=='contaminated'))]))};
 return {...baseline,rows,blessed:[...baseline.blessed,{at:now.toISOString(),reason:reason.trim(),from:from??null,backends:Object.keys(candidate)}]};
}
export async function writeBaseline(path,value){await writeFile(path,JSON.stringify(validateBaseline(value),null,1)+'\n');}
