import test from 'node:test';
import assert from 'node:assert/strict';
import {splitBudget,validateStage,partitionPrincipals} from './http-multi-protocol.mjs';
import {generatorWorkers} from './http-generator-launch.mjs';
import {validateAssignment} from './http-multi-controller.mjs';
import {campaignOptions} from './http-campaign.mjs';
import {measure,MAX_WORKERS_PER_HOST,MAX_OUTSTANDING,MAX_STAGE_SECONDS} from './http-load.mjs';

const stage={concurrency:10000,seconds:3600,timeoutMs:1000};
test('limits for the 40,000 rps rig are explicit and validated at the edges',()=>{
 assert.deepEqual([MAX_WORKERS_PER_HOST,MAX_OUTSTANDING,MAX_STAGE_SECONDS],[64,20000,3600]);
 // 40,000 rps x 250 ms = 10,000 outstanding, across 64 workers, for an hour.
 const budget=validateStage({...stage,workers:64,arrivalRate:40000});
 assert.equal(budget.concurrency.reduce((a,b)=>a+b),10000);assert.equal(budget.rates.reduce((a,b)=>a+b),40000);assert.equal(budget.rates.length,64);
 assert.equal(generatorWorkers(64),64);
 assert.equal(splitBudget(40000,64).length,64);
});
test('values past each limit are refused',()=>{
 assert.throws(()=>generatorWorkers(65));assert.throws(()=>generatorWorkers(0));
 for(const patch of [{workers:257},{workers:0},{concurrency:20001},{seconds:3601},{seconds:0},{arrivalRate:100001},{timeoutMs:30001}])assert.throws(()=>validateStage({...stage,workers:64,...patch}),/Invalid coordinated stage/,JSON.stringify(patch));
 assert.throws(()=>validateAssignment({engineMode:'native',base:'http://127.0.0.1:1/'},65));
});
test('per-process measurement and campaign options accept the raised caps and nothing beyond',async()=>{
 await assert.rejects(measure({base:'http://127.0.0.1:1',requests:[{name:'x',path:'/'}],concurrency:20001,seconds:1}),/Invalid bounded/);
 await assert.rejects(measure({base:'http://127.0.0.1:1',requests:[{name:'x',path:'/'}],concurrency:1,seconds:3601}),/Invalid bounded/);
 const env={MW_LOAD_FIXTURE_MODE:'capacity',MW_LOAD_FIXTURE_LONG:'1'};
 const raised=campaignOptions({...env,MW_LOAD_FIXTURE_SECONDS:'3600',MW_LOAD_FIXTURE_CONCURRENCY:'10000,20000',MW_LOAD_FIXTURE_DEADLINE_SECONDS:'14400'});
 assert.equal(raised.seconds,3600);assert.deepEqual(raised.concurrency,[10000,20000]);
 assert.throws(()=>campaignOptions({...env,MW_LOAD_FIXTURE_SECONDS:'3601'}));assert.throws(()=>campaignOptions({...env,MW_LOAD_FIXTURE_CONCURRENCY:'20001'}));
 // Defaults are the ones the earlier campaigns used.
 const defaults=campaignOptions(env);assert.equal(defaults.seconds,30);assert.deepEqual(defaults.concurrency,[1,4,16,64,256]);assert.equal(defaults.deadlineSeconds,600);
});
test('10,000 users still partition across 64 workers with no overlap',()=>{
 const parts=partitionPrincipals(Array.from({length:10000},(_,i)=>i),64);
 assert.equal(parts.reduce((n,p)=>n+p.count,0),10000);assert.equal(new Set(parts.flatMap(p=>p.principals)).size,10000);
 assert.throws(()=>partitionPrincipals(Array.from({length:10001},(_,i)=>i),64));
});
