import test from 'node:test';
import assert from 'node:assert/strict';
import {campaignOptions,runCampaign} from './http-campaign.mjs';

const result=(rps=100,extra={})=>({acceptedRequestsPerSecond:rps,requests:100,accepted:100,errors:0,throttled:0,timeouts:0,statuses:{200:100},latencyMs:{p99:10},...extra});
test('long runs require explicit opt-in and bounded configuration',()=>{
 assert.throws(()=>campaignOptions({MW_LOAD_FIXTURE_MODE:'capacity'}),/LONG/);
 assert.throws(()=>campaignOptions({MW_LOAD_FIXTURE_SECONDS:'30'}),/Smoke/);
 const o=campaignOptions({MW_LOAD_FIXTURE_MODE:'capacity',MW_LOAD_FIXTURE_LONG:'1'});
 assert.equal(o.seconds,30);assert.ok(o.rates.includes(40000));assert.equal(o.warmupSeconds,10);
 assert.throws(()=>campaignOptions({MW_LOAD_FIXTURE_MODE:'ramp',MW_LOAD_FIXTURE_LONG:'1',MW_LOAD_FIXTURE_CONCURRENCY:'10000'}),/CONCURRENCY/);
});
test('ramp warms each endpoint, records plateau, and never jumps blindly to40k',async()=>{
 const options={...campaignOptions({MW_LOAD_FIXTURE_MODE:'capacity',MW_LOAD_FIXTURE_LONG:'1'}),concurrency:[1,4,16,64],warmupSeconds:1};
 const calls=[],phases=[];
 const r=await runCampaign({groups:{mixed:[{}]},options,run:async spec=>{calls.push(spec);return result(100);},diagnostics:async phase=>{phases.push(phase);return {safe:true};}});
 assert.equal(r.warmups.length,1);assert.equal(r.stages.length,3);
 assert.ok(r.decisions.some(d=>d.reason==='accepted-throughput-plateau'));
 assert.ok(r.decisions.some(d=>d.reason==='offered-rate-exceeds-observed-saturation'));
 assert.ok(calls.every(c=>c.arrivalRate===undefined));assert.deepEqual(phases,['before','after','before','after','before','after']);
});
test('429 and5xx stop concurrency escalation while preserving the failed stage',async()=>{
 for(const failure of [{throttled:1,statuses:{200:99,429:1}},{errors:1,statuses:{200:99,503:1}}]){
  const options={...campaignOptions({MW_LOAD_FIXTURE_MODE:'ramp',MW_LOAD_FIXTURE_LONG:'1'}),warmupSeconds:0};
  const r=await runCampaign({groups:{mixed:[{}]},options,run:async()=>result(100,failure)});
  assert.equal(r.stages.length,1);assert.ok(['throttle-budget','server-5xx'].includes(r.decisions[0].reason));
 }
});
