import test from 'node:test';
import assert from 'node:assert/strict';
import {runDefinition,stageCounters,shapeOf,legRows,weightedMix,markdownSummary,R1_MODES} from './rig-shapes.mjs';
import {GOAL_MIX_TABLE} from './representative-workload.mjs';
import {campaignOptions,GOAL_SHAPE_ENDPOINTS} from './http-campaign.mjs';
import {parseT34Selection} from './t34-modes.mjs';
import {allocate} from './rig-cpuset.mjs';

test('R0a is one transport leg; R1 is shapes, mix and the instrumented shapes leg; envs pass the generator and mode parsers',()=>{
 const r0a=runDefinition('r0a',{appCpus:4});
 assert.deepEqual(r0a.legs.map(leg=>leg.id),['transport']);
 assert.equal(r0a.legs[0].env.MW_LOAD_FIXTURE_ENDPOINTS,'transport-healthz');
 assert.match(r0a.description,/transport control only/);assert.match(r0a.legs[0].note,/probe pool|8 permits/);
 const r1=runDefinition('r1',{appCpus:4});
 assert.deepEqual(r1.legs.map(leg=>[leg.id,leg.image,leg.instrumented]),[['release-shapes','release',false],['release-mix','release',false],['diagnostics-shapes','diagnostics',true]]);
 assert.equal(r1.legs[2].env.MW_FIXTURE_REQUEST_PROFILE,'1');assert.equal(r1.legs[0].env.MW_FIXTURE_REQUEST_PROFILE,'0');
 for(const leg of [...r0a.legs,...r1.legs]){
  const options=campaignOptions(leg.env);
  assert.equal(options.adaptive,false);
  assert.ok(Object.values(leg.env).every(value=>typeof value==='string'&&value.length<=256),'handoff environment values stay within 256 characters');
  assert.equal(leg.env.MW_LOAD_FIXTURE_WORKERS,'4');assert.equal(leg.env.MW_NATIVE_LOAD_SECURITY_PROFILE,'enforced');assert.equal(leg.env.MW_DB_ENCRYPTION,'required');
  assert.equal(leg.env.MW_DB_KEY,undefined,'no secret in the definition');
 }
 assert.deepEqual(campaignOptions(r1.legs[0].env).endpoints,GOAL_SHAPE_ENDPOINTS);
 assert.equal(GOAL_SHAPE_ENDPOINTS.length,17);
 assert.deepEqual(campaignOptions(r1.legs[1].env).concurrency,[64]);assert.equal(campaignOptions(r1.legs[1].env).seconds,60);
 assert.equal(r1.legs[0].env.MW_NATIVE_LOAD_PRINCIPALS,'1000');assert.equal(r1.legs[0].env.MW_SQLITE_WRITER,'gate');
 assert.equal(parseT34Selection(r1.legs[0].env).modes.requestProtection,'write-behind');assert.equal(parseT34Selection(r1.legs[0].env).modes.guardSnapshot,'bounded');
 const pg=runDefinition('r1',{backend:'postgres'});
 assert.equal(pg.legs[0].env.MW_LOAD_FIXTURE_BACKEND,'postgres');assert.equal(pg.legs[0].env.MW_REQUEST_PROTECTION_ACCOUNTING,'synchronous');assert.equal(pg.legs[0].env.MW_SQLITE_WRITER,undefined);
 assert.equal(runDefinition('r1',{smoke:true}).legs[0].env.MW_LOAD_FIXTURE_SECONDS,'5');
 assert.throws(()=>runDefinition('r2'));assert.throws(()=>runDefinition('r1',{backend:'mysql'}));
 assert.equal(R1_MODES.MW_GUARD_SNAPSHOT,'bounded');
});

test('goal-shapes alias: bounded, no repeats, and the unchanged endpoints still parse as before',()=>{
 const env={MW_LOAD_FIXTURE_MODE:'ramp',MW_LOAD_FIXTURE_LONG:'1'};
 assert.deepEqual(campaignOptions({...env,MW_LOAD_FIXTURE_ENDPOINTS:'transport-healthz,goal-shapes,goal-65-35'}).endpoints,['transport-healthz',...GOAL_SHAPE_ENDPOINTS,'goal-65-35']);
 assert.throws(()=>campaignOptions({...env,MW_LOAD_FIXTURE_ENDPOINTS:'goal-shapes,goal-shape-draft-save'}),/repeats/);
 assert.throws(()=>campaignOptions({...env,MW_LOAD_FIXTURE_ENDPOINTS:'goal-shape-unknown'}));
 assert.deepEqual(campaignOptions({...env,MW_LOAD_FIXTURE_ENDPOINTS:'identity,identity'}).endpoints,['identity','identity']);
});

const ops=(reads,writes,other,tx,auto)=>({read:{calls:reads},write:{calls:writes},other:{calls:other},acquire:{calls:0},commits:{sqlite:{calls:tx},postgres:{calls:0}},autocommitWrites:auto});
const up=n=>({store:n,move:0,expunge:0,append:0,fetch:0,list:0,commands:n});
const diag=(before,after,profile)=>({before:{pool:{operations:before},upstreamCommands:up(0),...(profile?{requestProfile:{enabled:true,queries:ops(0,0,0,0,0)}}:{})},after:{pool:{operations:after},upstreamCommands:up(200),...(profile?{requestProfile:{enabled:true,queries:profile}}:{})}});

test('stage counters: process-wide deltas, or request-scoped when the request profile was on',()=>{
 const stage={requests:100,scheduledSeconds:10,diagnostics:diag(ops(1000,10,5,3,1),ops(2100,210,105,103,51))};
 const c=stageCounters(stage);
 assert.equal(c.source,'process');assert.deepEqual(c.perRequest,{reads:11,writes:2,other:1,statements:14,transactionCommits:1,autocommitWrites:0.5,databaseCommits:1.5,upstreamCommands:2,upstreamWriteCommands:2});
 const profiled=stageCounters({...stage,diagnostics:diag(ops(0,0,0,0,0),ops(99999,0,0,0,0),ops(500,100,50,40,10))});
 assert.equal(profiled.source,'request-profile');assert.equal(profiled.perRequest.reads,5);assert.equal(profiled.perRequest.databaseCommits,0.5);
 // A fixture without the commit counter: statements still reported, commits marked unavailable.
 const old=stageCounters({...stage,diagnostics:diag({read:{calls:0},write:{calls:0},other:{calls:0}},{read:{calls:100},write:{calls:0},other:{calls:0}})});
 assert.equal(old.perRequest.reads,1);assert.equal(old.commitsAvailable,false);assert.equal(old.perRequest.databaseCommits,null);
 assert.equal(stageCounters({requests:1}).available,false);
});

test('rows, weighted mix and the summary table carry the plan columns and the labels',()=>{
 assert.deepEqual(shapeOf('goal-shape-list-batch'),{name:'list-batch',kind:'read',weight:15});
 assert.equal(shapeOf('transport-healthz').kind,'transport');assert.equal(shapeOf('goal-65-35').kind,'mix');
 const plan=allocate({app:4,generator:2,rig:1},{vcpus:26});
 const samples=Array.from({length:30},(_,i)=>({schema:1,tMs:i*500,containers:{app:{cpu:{usage_usec:i*1e6,user_usec:i*1e6,system_usec:0,nr_periods:0,nr_throttled:0,throttled_usec:0}},generator:{cpu:{usage_usec:i*5e5,user_usec:0,system_usec:0,nr_periods:0,nr_throttled:0,throttled_usec:0}}},others:{},procStat:{cpus:Object.fromEntries(Array.from({length:26},(_,c)=>[c,[0,i*50]]))}}));
 const stageFor=(endpoint,ms)=>({endpoint,concurrency:32,phase:'concurrency-ramp',requests:1000,acceptedDuringScheduledWindow:1000,acceptedDuringScheduledWindowRps:200,scheduledSeconds:5,errors:0,throttled:0,timeouts:0,latencyMs:{p50:ms,p99:ms*3},coordination:{scheduledStartAtMs:1000,scheduledEndAtMs:6000},diagnostics:diag(ops(0,0,0,0,0),ops(5000,1000,500,300,100))});
 const report={kind:'coordinated-native-http-fixture',stages:[...GOAL_MIX_TABLE.map(row=>stageFor('goal-shape-'+row.name,2)),stageFor('goal-65-35',3)]};
 const leg={id:'release-shapes',instrumented:false};
 const rows=legRows(report,{leg,samples,hostSamples:[],plan,labels:{app:'app',generator:'generator'}});
 assert.equal(rows.length,18);
 const first=rows[0];
 assert.equal(first.shape,'authenticated-identity');assert.equal(first.usecCpuPerRequest.server,10000);assert.equal(first.usecCpuPerRequest.generator,5000);
 assert.equal(first.rpsPerLogicalCpu.allottedServer,50);assert.equal(first.database.perRequest.databaseCommits,0.4);assert.equal(first.contaminated,false);
 const mix=weightedMix(rows);
 assert.equal(mix.weightedServerUsecPerRequest,10000);assert.deepEqual(mix.missingShapes,[]);assert.equal(mix.measuredMix.usecCpuPerRequest,10000);
 const md=markdownSummary({run:'r1',backend:'sqlite',description:'d',host:{vmVcpus:26},cpusets:{app:'1-4'},rows,mix,notes:['n']});
 assert.match(md,/\| Leg \| Shape \| Kind \| W \| C \| Requests \| Reads\/req \| Writes\/req \| Other\/req \| Commits\/req \(tx \+ autocommit\) \| Upstream cmds\/req \| Server µs CPU\/req/);
 assert.match(md,/\| release-shapes \| list-batch \| read \| 15 \| 32 \| 1000 \| 5 \| 1 \| 0.5 \| 0.4 \(0.3 \+ 0.1\) \| 0.2 \| 10000 \| 5000 \| 200 \| 50 \| 2.00 \| 6.00 \| 0 \| – \| – \| – \| quiet \|/);
 assert.match(md,/Weighted server µs CPU per request from the shapes: 10000/);
 const transport=legRows({stages:[stageFor('transport-healthz',1)]},{leg:{id:'transport',instrumented:false},samples,hostSamples:[],plan,labels:{app:'app'}});
 assert.match(transport[0].label,/transport control only/);
 const instrumented=legRows({stages:[stageFor('goal-shape-changes',1)]},{leg:{id:'diagnostics-shapes',instrumented:true},samples,hostSamples:[],plan,labels:{app:'app'}});
 assert.match(instrumented[0].label,/counts only/);
 assert.equal(weightedMix(instrumented).weightedServerUsecPerRequest,null,'instrumented rows never feed the weighted mix');
});
