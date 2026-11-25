import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtemp,mkdir,writeFile,readFile,rm} from 'node:fs/promises';
import {join} from 'node:path';
import os from 'node:os';
import {compare,markdownComparison,metricsOfRow,metricsOfSummary,validateBaseline,bless,DEFAULT_THRESHOLD} from './rig-compare.mjs';
import {abOrder,runAb,compareCommand} from './bench-rig.mjs';
import {GOAL_MIX_TABLE} from './representative-workload.mjs';

const m=(o={})=>({source:'process',reads:8.19,writes:0.26,other:3.15,databaseCommits:0.3,upstreamCommands:0,errors:0,http503:0,http429:0,timeouts:0,usecCpuPerRequest:14705,rps:243.7,rpsPerConsumedCpu:72.1,p50:125,p99:527,contaminated:false,...o});
const run=(o={},shape='list-batch',backend='sqlite')=>({[backend]:{[shape]:m(o)}});
const verdictOf=(result,metric)=>result.rows[0].checks.find(c=>c.metric===metric&&!c.repeat)?.verdict;

test('deterministic counters regress on any increase at the stated resolution; equal and lower are not regressions',()=>{
 const ref=[run()];
 assert.equal(compare(ref,[run()]).rows[0].verdict,'same');
 for(const [metric,value] of [['reads',8.4],['writes',0.4],['other',3.3],['databaseCommits',0.5],['upstreamCommands',0.2],['errors',1],['http503',1],['http429',1],['timeouts',1]]){
  const result=compare(ref,[run({[metric]:value})]);
  assert.equal(result.rows[0].verdict,'REGRESSION',metric);assert.equal(verdictOf(result,metric),'regression');assert.equal(result.regressions,1);
 }
 // Process-wide per-request counts: second-decimal jitter is below the 0.1 resolution.
 assert.equal(verdictOf(compare(ref,[run({reads:8.22})]),'reads'),'same');
 // Request-scoped counts are compared at 0.01.
 assert.equal(verdictOf(compare([run({source:'request-profile'})],[run({source:'request-profile',reads:8.21})]),'reads'),'regression');
 assert.equal(verdictOf(compare(ref,[run({reads:7})]),'reads'),'improved');
 // A counter the reference does not have (an older image) is not compared.
 assert.equal(verdictOf(compare([run({databaseCommits:null})],[run({databaseCommits:5})]),'databaseCommits'),'not-compared');
 // A counter increase in the second repeat alone is still a regression.
 const second=compare(ref,[run(),run({errors:3})]);
 assert.equal(second.rows[0].verdict,'REGRESSION');assert.equal(second.rows[0].checks.find(c=>c.repeat===2).metric,'errors');
});

test('noisy numbers regress only beyond the threshold in every one of at least two repeats',()=>{
 assert.equal(DEFAULT_THRESHOLD,0.08);
 const ref=[run(),run()];
 const worse=compare(ref,[run({usecCpuPerRequest:16500}),run({usecCpuPerRequest:16200})]);
 assert.equal(verdictOf(worse,'usecCpuPerRequest'),'regression');assert.equal(worse.rows[0].verdict,'REGRESSION');
 // One repeat disagrees: not a regression.
 assert.equal(verdictOf(compare(ref,[run({usecCpuPerRequest:16500}),run({usecCpuPerRequest:15000})]),'usecCpuPerRequest'),'same');
 // Within 8%: not a regression.
 assert.equal(verdictOf(compare(ref,[run({p99:560}),run({p99:565})]),'p99'),'same');
 // Lower is worse for rps and rps per CPU.
 assert.equal(verdictOf(compare(ref,[run({rps:200}),run({rps:210})]),'rps'),'regression');
 assert.equal(verdictOf(compare(ref,[run({rpsPerConsumedCpu:80}),run({rpsPerConsumedCpu:81})]),'rpsPerConsumedCpu'),'improved');
 // One repeat only: worse is unconfirmed, the verdict is not a regression and nothing fails.
 const single=compare([run()],[run({p50:200})]);
 assert.equal(verdictOf(single,'p50'),'unconfirmed');assert.equal(single.rows[0].verdict,'unconfirmed');assert.equal(single.regressions,0);
 // A contaminated stage is not compared on noisy numbers.
 assert.equal(verdictOf(compare(ref,[run({p50:900,contaminated:true}),run({p50:900,contaminated:true})]),'p50'),'not-compared');
 // A shape without a reference row is reported, never silently passed as same.
 assert.equal(compare([run()],[run({},'draft-save')]).rows[0].verdict,'no-reference');
 assert.throws(()=>compare([],[run()]));assert.throws(()=>compare([run()],[run()],{threshold:0}));
});

test('summary rows map to the tracked numbers; plain rows only',()=>{
 const row={shape:'changes',instrumented:false,errors:0,throttled:2,timeouts:0,http503:4,acceptedRps:100,latencyMs:{p50:3,p99:9},usecCpuPerRequest:{server:900},rpsPerLogicalCpu:{consumedServer:111},contaminated:false,
  database:{available:true,source:'process',commitsAvailable:true,perRequest:{reads:5,writes:0,other:1,databaseCommits:0.2,upstreamCommands:0}}};
 assert.deepEqual(metricsOfRow(row),{source:'process',reads:5,writes:0,other:1,databaseCommits:0.2,upstreamCommands:0,errors:0,http503:4,http429:2,timeouts:0,usecCpuPerRequest:900,rps:100,rpsPerConsumedCpu:111,p50:3,p99:9,appMemoryEndBytes:null,oomEvents:null,contaminated:false});
 const s=metricsOfSummary({backend:'postgres',rows:[row,{...row,instrumented:true,shape:'list-batch'}]});
 assert.deepEqual(Object.keys(s.postgres),['changes']);
});

test('the committed baseline is valid, covers every SQLite goal shape and the mix, and changes only by bless with a reason',async()=>{
 const baseline=validateBaseline(JSON.parse(await readFile(new URL('./rig-baseline.json',import.meta.url),'utf8')));
 for(const row of GOAL_MIX_TABLE)assert.ok(baseline.rows.sqlite[row.name],row.name);
 assert.ok(baseline.rows.sqlite['goal-65-35']);assert.ok(baseline.rows.postgres['goal-65-35']);
 assert.match(baseline.source.image,/5717b070f/);assert.equal(baseline.blessed.length,1);
 assert.throws(()=>bless(baseline,run(),{reason:'short'}),/reason/);
 const next=bless(baseline,run({reads:1}),{reason:'engine statements removed by e13 (verified)',from:'x',now:new Date('2026-11-25T00:00:00Z')});
 assert.equal(next.rows.sqlite['list-batch'].reads,1);assert.equal(next.rows.sqlite['list-batch'].contaminated,undefined);
 assert.equal(next.rows.sqlite['changes'],baseline.rows.sqlite['changes']);
 assert.deepEqual(next.blessed.at(-1),{at:'2026-11-25T00:00:00.000Z',reason:'engine statements removed by e13 (verified)',from:'x',backends:['sqlite']});
 assert.throws(()=>validateBaseline({schema:1,rows:{},blessed:[]}));
});

test('compare command: baseline by default, exit-worthy regressions, bless writes the reason; A B B A order',async()=>{
 const root=await mkdtemp(join(os.tmpdir(),'rig-compare-'));
 try{
  const summary=(o)=>({backend:'sqlite',rows:[{shape:'list-batch',kind:'read',instrumented:false,errors:0,throttled:0,timeouts:0,http503:0,acceptedRps:243.7,latencyMs:{p50:125,p99:527},usecCpuPerRequest:{server:14705},rpsPerLogicalCpu:{consumedServer:72.1},contaminated:false,
   database:{available:true,source:'process',commitsAvailable:true,perRequest:{reads:8.19,writes:0.26,other:3.15,databaseCommits:0.3,upstreamCommands:0,...o}}}]});
  const dir=n=>join(root,n);
  for(const [name,o] of [['same',{}],['worse',{reads:9.5}]]){await mkdir(dir(name));await writeFile(join(dir(name),'summary.json'),JSON.stringify(summary(o)));}
  const baselinePath=join(root,'baseline.json');
  await writeFile(baselinePath,JSON.stringify({schema:1,rows:{sqlite:{'list-batch':m()}},blessed:[{at:'x',reason:'initial baseline for the test',from:null}]}));
  let out='';const write=t=>{out+=t;};
  assert.equal((await compareCommand({candidate:dir('same')},{baselinePath,write})).regressions,0);
  const bad=await compareCommand({candidate:dir('worse'),out:join(root,'cmp')},{baselinePath,write});
  assert.equal(bad.regressions,1);assert.match(out,/\| sqlite \| list-batch \| REGRESSION \| reads: 8.19 -> 9.5 \|/);
  assert.match(await readFile(join(root,'cmp.md'),'utf8'),/REGRESSION/);
  // The baseline file was not touched by compare.
  assert.equal(JSON.parse(await readFile(baselinePath,'utf8')).rows.sqlite['list-batch'].reads,8.19);
  await assert.rejects(compareCommand({candidate:dir('worse'),bless:true},{baselinePath,write}),/reason/);
  await compareCommand({candidate:dir('worse'),bless:true,reason:'accepted: new index reads measured and reviewed'},{baselinePath,write});
  const blessed=JSON.parse(await readFile(baselinePath,'utf8'));
  assert.equal(blessed.rows.sqlite['list-batch'].reads,9.5);assert.equal(blessed.blessed.at(-1).reason,'accepted: new index reads measured and reviewed');
  // Parent-run reference instead of the baseline file.
  assert.equal((await compareCommand({candidate:dir('worse'),reference:dir('same')},{baselinePath:join(root,'missing.json'),write})).regressions,1);
  await assert.rejects(compareCommand({candidate:dir('worse'),reference:dir('same'),bless:true,reason:'0123456789ab'},{baselinePath,write}),/--reference/);
 }finally{await rm(root,{recursive:true,force:true});}
 const order=abOrder('r1-x',{parent:{image:'A'},candidate:{image:'B'}});
 assert.deepEqual(order.map(s=>[s.runId,s.image]),[['r1-x-a1','A'],['r1-x-b1','B'],['r1-x-b2','B'],['r1-x-a2','A']]);
 const ran=[];const summaryFor=image=>({backend:'sqlite',rows:[{shape:'changes',instrumented:false,errors:0,throttled:0,timeouts:0,http503:0,acceptedRps:100,latencyMs:{p50:3,p99:9},usecCpuPerRequest:{server:image==='B'?1200:1000},rpsPerLogicalCpu:{consumedServer:100},contaminated:false,database:{available:true,source:'process',commitsAvailable:true,perRequest:{reads:5,writes:0,other:1,databaseCommits:0,upstreamCommands:0}}}]});
 const result=await runAb(order,async step=>{ran.push(step.tag);return summaryFor(step.image);});
 assert.deepEqual(ran,['a1','b1','b2','a2']);assert.equal(result.repeats,2);assert.equal(result.regressions,1);
 assert.match(markdownComparison(result),/usecCpuPerRequest: 1000 \/ 1000 -> 1200 \/ 1200/);
});
