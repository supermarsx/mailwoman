// Bench-rig endpoint groups added to the shared request factory (t62-n12): goal-shape-* and transport-healthz.
import test from 'node:test';
import assert from 'node:assert/strict';
import {prepareFixtureRequests,GOAL_SHAPE_GROUPS,TRANSPORT_GROUP,transportSpec} from './fixture-requests.mjs';
import {GOAL_MIX_TABLE,GOAL_MIX_NAME} from './representative-workload.mjs';
import {ENDPOINT_LABELS} from './http-multi-protocol.mjs';
import {GUARD_SNAPSHOT_LOOKUPS} from './t34-modes.mjs';
import {GOAL_SHAPE_ENDPOINTS} from './http-campaign.mjs';

function users(count=6,messages=50){
 return Array.from({length:count},(_,index)=>({cookie:`mw_session=private-${index}; mw_csrf=csrf-${index}`,csrfToken:`csrf-${index}`,accountId:`acct-${index}`,ownerAccountId:`acct-${index}`,
  messageIds:Array.from({length:messages},(_,n)=>`m-${index}-${n}`),inboxId:`inbox-${index}`,archiveId:`archive-${index}`,trashId:`trash-${index}`,draftsId:`drafts-${index}`,plantedQuarterly:10,attachmentBlobId:`blob-${index}`,attachmentSampleBytes:65536,bodySampleBytes:4096,clientAddress:`198.18.0.${index+1}`}));
}
const profile={kind:'representative',messagesPerPrincipal:50,bodySampleBytes:4096,attachmentSampleBytes:65536,seededMessages:50*6,seedSeconds:1};
const prepare=seeded=>prepareFixtureRequests({principals:seeded,clientSimulation:{mode:'fixture-loopback-trusted-proxy',clientCount:seeded.length},engineMode:'native',representative:true,workloadProfile:profile});

test('each goal-mix shape has a group of its own that is the mix template, rotating over every user',()=>{
 assert.deepEqual(GOAL_SHAPE_GROUPS,GOAL_SHAPE_ENDPOINTS);
 const seeded=users(),fixture=prepare(seeded),mix=fixture.groups[GOAL_MIX_NAME];
 for(const row of GOAL_MIX_TABLE){
  const group=fixture.groups['goal-shape-'+row.name];
  assert.equal(group.length,1,row.name);assert.equal(group[0].name,row.name);
  const slot=mix.findIndex(spec=>spec.name===row.name);
  const visited=new Set();
  for(let sequence=0;sequence<seeded.length;sequence++){
   const one=group[0].materialize(sequence,{requestCount:1});
   visited.add(one.principalIndex);
   assert.equal(one.headers.cookie,seeded[one.principalIndex].cookie);assert.equal(one.serializePrincipal,true);
   if(row.name!=='draft-discard'){
    // The mix rotates users per cycle with an offset of `slot`; pick the cycle that lands on the same user.
    const cycle=((one.principalIndex-slot)%seeded.length+seeded.length)%seeded.length;
    const fromMix=mix[slot].materialize(slot+100*cycle,{requestCount:100});
    assert.equal(fromMix.principalIndex,one.principalIndex);
    assert.equal(one.body,fromMix.body,'same template as the mix slot');assert.equal(one.path,fromMix.path);
   }
  }
  assert.equal(visited.size,seeded.length,row.name+' reaches every user');
 }
 // Existing groups are unchanged.
 assert.equal(mix.length,100);assert.equal(fixture.groups.mixed.length,5);
});

test('draft-discard alone alternates: it creates while nothing is saved, then destroys what it created',()=>{
 const group=prepare(users(2)).groups['goal-shape-draft-discard'];
 const first=group[0].materialize(0,{requestCount:1});
 assert.equal(JSON.parse(first.body).methodCalls[0][0],'Email/set');
 const respond={methodResponses:[['Email/set',{accountId:`acct-${first.principalIndex}`,created:{d1:{id:'d-1'}}},'save']]};
 assert.equal(first.validate(respond),true);
 const second=group[0].materialize(1,{requestCount:1});
 assert.deepEqual(JSON.parse(second.body).methodCalls[1][1].destroy,['d-1']);assert.equal(second.principalIndex,first.principalIndex);
});

test('the transport group is GET /healthz from a loopback client address, labelled and outside every other group',()=>{
 const fixture=prepare(users(2));
 assert.deepEqual(fixture.groups[TRANSPORT_GROUP].map(spec=>spec.path),['/healthz']);
 const spec=transportSpec();
 assert.equal(spec.headers['x-forwarded-for'],'127.0.0.1');assert.equal(spec.method,undefined);
 assert.equal(spec.validateBytes(Buffer.from('ok')),true);assert.equal(spec.validateBytes(Buffer.from('<!doctype html>')),false);
 assert.ok(ENDPOINT_LABELS.includes(TRANSPORT_GROUP));
 assert.equal(GUARD_SNAPSHOT_LOOKUPS[TRANSPORT_GROUP],false);
 for(const name of Object.keys(fixture.groups).filter(name=>name!==TRANSPORT_GROUP))assert.ok(fixture.groups[name].every(s=>s.path!=='/healthz'),name);
 // A fixture without the goal seeding has no shape groups, but still the transport group.
 const bare=prepareFixtureRequests({principals:users(2,3),clientSimulation:{mode:'fixture-loopback-trusted-proxy',clientCount:2},engineMode:'native',representative:true,workloadProfile:{...profile,messagesPerPrincipal:3,seededMessages:6}});
 assert.equal(bare.groups['goal-shape-list-batch'],undefined);assert.ok(bare.groups[TRANSPORT_GROUP]);
});

test('request-input evidence knows the goal mix, every single-shape group and the transport group',async()=>{
 const {requestInputsGroupConsumes,requestInputsEvidence}=await import('./request-inputs-experiment.mjs');
 assert.equal(requestInputsGroupConsumes('goal-65-35'),true);
 assert.equal(requestInputsGroupConsumes('goal-shape-authenticated-identity'),true);
 assert.equal(requestInputsGroupConsumes('goal-shape-list-batch'),false);
 assert.equal(requestInputsGroupConsumes('transport-healthz'),false);
 const status={mode:'published',processImmutable:true,ordinaryContexts:0,readyFrames:0,consumedFrames:0,hintAttempts:0,buildAttempts:0,completedBuilds:0,firstFenceAttempts:0,refreshFenceAttempts:0};
 const stage={endpoint:'transport-healthz',phase:'concurrency-ramp',byRequest:{'transport-healthz':{requests:10,acceptedDuringScheduledWindow:10}},requestInputsWindow:{schema:1,kind:'workload-and-drain-only',eligibleControlRequests:0},diagnostics:{before:{ordinaryRequestInputs:status},after:{ordinaryRequestInputs:status}}};
 assert.doesNotThrow(()=>requestInputsEvidence(status,[stage],'published',{backend:'sqlite'}));
});

test('PostgreSQL: published inputs are expected exactly when the guard snapshot mirror is live, and must be zero otherwise',async()=>{
 const {requestInputsEvidence:evidence,mirrorLive,requestInputCounters:keys}=await import('./request-inputs-experiment.mjs');
 const witness=(count)=>({mode:'published',processImmutable:true,...Object.fromEntries(keys.map(key=>[key,count]))});
 const guard=(mode,polls,failures=0)=>({supported:true,mode,maxStalenessMs:1000,polls,failures,rowsApplied:0,hits:0,misses:0,expedites:0});
 const stage=(before,after,{mode='bounded',polls=[1,5],failures=[0,0]}={})=>({endpoint:'identity',phase:'offered-rate',byRequest:{'authenticated-identity':{requests:2,acceptedDuringScheduledWindow:1}},requestInputsWindow:{schema:1,kind:'workload-and-drain-only',eligibleControlRequests:0},
  diagnostics:{before:{ordinaryRequestInputs:witness(before),t34Modes:{guardSnapshot:guard(mode,polls[0],failures[0])}},after:{ordinaryRequestInputs:witness(after),t34Modes:{guardSnapshot:guard(mode,polls[1],failures[1])}}}});
 // Live mirror: consumption is expected and counted as on SQLite.
 const live=evidence(witness(10),[stage(10,11)],'published',{backend:'postgres'});
 assert.equal(live.qualified,true);assert.equal(live.liveMirrorPhases,1);assert.equal(live.observations[0].liveMirror,true);
 // Live mirror and nothing consumed in the whole run: refused, as on SQLite.
 assert.throws(()=>evidence(witness(10),[stage(10,10)],'published',{backend:'postgres'}),/did not consume/);
 // No live mirror (off, no polls, or new poll failures): consumption must be zero.
 for(const options of [{mode:'off'},{polls:[5,5]},{failures:[0,2]}]){
  assert.equal(mirrorLive(stage(10,11,options)),false);
  assert.throws(()=>evidence(witness(10),[stage(10,11,options)],'published',{backend:'postgres'}),/without a live guard snapshot mirror/);
  assert.equal(evidence(witness(10),[stage(10,10,options)],'published',{backend:'postgres'}).exercise,'not-applicable-backend');
 }
 // A stage without the t34 witness is not live.
 assert.equal(mirrorLive({diagnostics:{before:{},after:{}}}),false);
});
