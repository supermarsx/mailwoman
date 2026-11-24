import test from 'node:test';
import assert from 'node:assert/strict';
import {prepareFixtureRequests} from './fixture-requests.mjs';
import {GOAL_MIX_TABLE,GOAL_MIX_CYCLE,GOAL_MIX_NAME,goalMixSchedule,goalMixReport,goalMixWriteSpecs,isRepresentativeKind,workloadProfileReport} from './representative-workload.mjs';
import {goalFromEnv,judgedAgainst,hostFacts,DEFAULT_TARGET_RPS,GOAL_WORKLOAD} from './http-goal.mjs';
import {campaignOptions} from './http-campaign.mjs';
import {qualifiesHttpTarget} from './http-multi-controller.mjs';
import {markdownReport} from './http-load.mjs';

const profile={kind:'representative',messagesPerPrincipal:50,bodySampleBytes:4096,attachmentSampleBytes:65536,seededMessages:50*6,seedSeconds:1};
function users(count=6,messages=50){
 return Array.from({length:count},(_,index)=>({cookie:`mw_session=private-${index}; mw_csrf=csrf-${index}`,csrfToken:`csrf-${index}`,accountId:`acct-${index}`,ownerAccountId:`acct-${index}`,
  messageIds:Array.from({length:messages},(_,n)=>`m-${index}-${n}`),inboxId:`inbox-${index}`,archiveId:`archive-${index}`,attachmentBlobId:`blob-${index}`,attachmentSampleBytes:65536,bodySampleBytes:4096,clientAddress:`198.18.0.${index+1}`}));
}
const prepare=(seeded,workloadProfile=profile)=>prepareFixtureRequests({principals:seeded,clientSimulation:{mode:'fixture-loopback-trusted-proxy',clientCount:seeded.length},engineMode:'native',representative:true,workloadProfile});

test('the weight table is one reviewed table: 65 reads and 35 writes per 100 requests, 2 mutations per write request',()=>{
 const reads=GOAL_MIX_TABLE.filter(row=>row.kind==='read'),writes=GOAL_MIX_TABLE.filter(row=>row.kind==='write');
 assert.equal(reads.reduce((n,row)=>n+row.weight,0),65);
 assert.equal(writes.reduce((n,row)=>n+row.weight,0),35);
 assert.ok(writes.every(row=>row.mutationsPerRequest===2)&&reads.every(row=>row.mutationsPerRequest===0));
 assert.equal(new Set(GOAL_MIX_TABLE.map(row=>row.name)).size,GOAL_MIX_TABLE.length);
 assert.ok(Object.isFrozen(GOAL_MIX_TABLE)&&GOAL_MIX_TABLE.every(Object.isFrozen));
 // nothing unvalidated: no trash, draft or search shape
 assert.ok(!GOAL_MIX_TABLE.some(row=>/trash|draft|search/i.test(row.name)));
});

test('the schedule is deterministic, exact per cycle and spreads writes through it',()=>{
 const order=goalMixSchedule();
 assert.equal(order.length,GOAL_MIX_CYCLE);
 assert.deepEqual(order,goalMixSchedule());
 for(const row of GOAL_MIX_TABLE)assert.equal(order.filter(name=>name===row.name).length,row.weight,row.name);
 const kind=new Map(GOAL_MIX_TABLE.map(row=>[row.name,row.kind]));
 assert.equal(order.filter(name=>kind.get(name)==='write').length,35);
 // no run of more than 4 consecutive writes, and every 20 consecutive slots hold 3 to 11 writes
 let run=0,longest=0;for(const name of [...order,...order]){run=kind.get(name)==='write'?run+1:0;longest=Math.max(longest,run);}
 assert.ok(longest<=4,'longest write run '+longest);
 for(let start=0;start<80;start++){const writes=order.slice(start,start+20).filter(name=>kind.get(name)==='write').length;assert.ok(writes>=3&&writes<=11,`window ${start}: ${writes}`);}
});

test('the goal group is built from existing validated templates, serialised per user, 65/35 over any 100 requests',()=>{
 const seeded=users(),fixture=prepare(seeded),group=fixture.groups[GOAL_MIX_NAME];
 assert.equal(group.length,100);
 assert.ok(group.every(spec=>spec.serializePrincipal===true));
 const kind=new Map(GOAL_MIX_TABLE.map(row=>[row.name,row.kind]));
 assert.equal(group.filter(spec=>kind.get(spec.name)==='write').length,35);
 assert.ok(group.filter(spec=>kind.get(spec.name)==='write').every(spec=>spec.statefulMutation===true&&spec.method==='POST'));
 assert.ok(group.filter(spec=>kind.get(spec.name)==='read').every(spec=>!spec.statefulMutation));
 // the existing groups are unchanged
 assert.equal(fixture.groups.representative.length,10);
 assert.equal(fixture.groups.mixed.length,5);
 // sequencing: the same 100-request cycle visits every user, each carrying its own cookie and simulated address
 const seen=new Set();
 for(let sequence=0;sequence<600;sequence++){
  const spec=group[sequence%100].materialize(sequence,{requestCount:100}),p=seeded[spec.principalIndex];
  seen.add(spec.principalIndex);
  assert.equal(spec.headers.cookie,p.cookie);assert.equal(spec.headers['x-forwarded-for'],p.clientAddress);
  assert.equal(spec.name,group[sequence%100].name);
 }
 assert.equal(seen.size,6);
 // the three added write shapes use distinct messages, never the one the mark-read/move pair uses
 const bodies=goalMixWriteSpecs(seeded[0]).map(spec=>JSON.parse(spec.body).methodCalls[0][1].update);
 assert.deepEqual(bodies.map(update=>Object.keys(update)[0]),['m-0-2','m-0-1','m-0-3']);
 // fixtures with fewer than four messages simply have no goal group
 assert.equal(prepare(users(2,3),{...profile,messagesPerPrincipal:3,seededMessages:6}).groups[GOAL_MIX_NAME],undefined);
 assert.ok(isRepresentativeKind('representative')&&isRepresentativeKind(GOAL_MIX_NAME)&&!isRepresentativeKind('baseline'));
 assert.equal(workloadProfileReport({...profile,kind:GOAL_MIX_NAME,seededMessages:300},6).kind,GOAL_MIX_NAME);
});

const set=(id,ok=true)=>ok?{accountId:'a',updated:{[id]:null}}:{accountId:'a',notUpdated:{[id]:{type:'forbidden'}}};
const read=(id,keywords)=>({accountId:'a',list:[{id,keywords}]});
const respond=(spec,rows)=>({methodResponses:JSON.parse(spec.body).methodCalls.map((call,index)=>[call[0],{...rows[index],accountId:JSON.parse(spec.body).methodCalls[0][1].accountId},call[2]])});

test('each added write shape verifies both mutations by read-back; a lost or wrong mutation fails',()=>{
 const [unread,flag,unflag]=goalMixWriteSpecs(users(1)[0]);
 for(const [spec,id,keyword,first,second] of [[unread,'m-0-2','$seen',false,true],[flag,'m-0-1','$flagged',true,false],[unflag,'m-0-3','$flagged',false,true]]){
  const kw=on=>on?{[keyword]:true}:{};
  const good=[set(id),read(id,kw(first)),set(id),read(id,kw(second))];
  assert.equal(spec.validate(respond(spec,good)),true,spec.name);
  assert.equal(spec.validate(respond(spec,[good[0],read(id,kw(!first)),good[2],good[3]])),false,spec.name+' first read-back');
  assert.equal(spec.validate(respond(spec,[good[0],good[1],good[2],read(id,kw(!second))])),false,spec.name+' second read-back');
  assert.equal(spec.validate(respond(spec,[set(id,false),good[1],good[2],good[3]])),false,spec.name+' lost update');
  assert.equal(spec.validate(respond(spec,[good[0],good[1],set(id,false),good[3]])),false,spec.name+' lost restore');
  assert.equal(spec.validate({methodResponses:[]}),false);
  assert.equal(spec.statefulMutation,true);
 }
});

test('the per-stage mix report: achieved split, verified mutations and the unmeasured upstream rate',()=>{
 const stat=(requests,accepted)=>({requests,accepted,acceptedDuringScheduledWindow:accepted});
 const byRequest={'list-batch':stat(300,300),'Email/get':stat(50,50),'mark-read-and-restore':stat(100,98),'move-and-restore':stat(50,50)};
 const report=goalMixReport(byRequest,10);
 assert.deepEqual([report.attempted.reads,report.attempted.writes],[350,150]);
 assert.ok(Math.abs(report.attempted.writeShare-0.3)<1e-9);
 assert.deepEqual([report.acceptedInWindow.reads,report.acceptedInWindow.writes],[350,148]);
 assert.equal(report.mutationsAttempted,300);
 assert.equal(report.mutationsPerSecond,(98+50)*2/10);
 assert.equal(report.readRequestsPerSecond,35);
 assert.equal(report.upstreamCommandsPerSecond,null);
 assert.match(report.upstreamCommandsNote,/not exposed/);
 assert.equal(goalMixReport({...byRequest,'Mailbox/get':stat(1,1)},10),undefined,'unknown shapes are not classified');
 assert.equal(goalMixReport({},10),undefined);
});

test('the standing goal is 30,000 requests/s; the host option lets a loopback run qualify throughput but never a network claim',()=>{
 assert.equal(DEFAULT_TARGET_RPS,30000);
 const goal=goalFromEnv({});
 assert.deepEqual([goal.successfulRps,goal.simultaneousAuthenticatedUsers,goal.isDefault],[30000,10000,true]);
 const stage=extra=>({endpoint:GOAL_MIX_NAME,phase:'offered-rate',arrivalRate:30000,acceptedDuringScheduledWindowRps:30000,uniquePrincipalsAcceptedDuringScheduledWindow:10000,uniqueClientsAcceptedDuringScheduledWindow:10000,coordination:{valid:true},mutationStateTainted:false,scheduledSeconds:30,errors:0,timeouts:0,throttled:0,droppedCapacity:0,droppedScheduler:0,
  mix:{acceptedInWindow:{writeShare:.35}},diagnostics:null,...extra});
 const env={MW_LOAD_QUALIFICATION_RUN:'1',MW_LOAD_QUALIFICATION_HOST:'local',MW_LOAD_HOST_DOCKER_VCPUS:'12',MW_LOAD_HOST_CPUSET_APP:'0-3',MW_LOAD_HOST_CPUSET_DB:'4-5',MW_LOAD_HOST_CPUSET_GENERATOR:'6-11'};
 const local=goalFromEnv(env);
 assert.equal(local.qualificationHost,'local');
 assert.equal(local.networkGbit,undefined,'no budget is required on the designated base hardware');
 assert.throws(()=>goalFromEnv({MW_LOAD_QUALIFICATION_RUN:'1'}),/MW_LOAD_NETWORK_GBIT/);
 assert.throws(()=>goalFromEnv({MW_LOAD_QUALIFICATION_HOST:'remote'}),/MW_LOAD_QUALIFICATION_HOST/);
 const options=campaignOptions(env);
 const telemetryFree=[stage()];
 // telemetry and index checks are separate rules; neutralise them with the same minimal diagnostics the other tests use
 const health={submitted:20,applied:20,dropped:0,depth:0,queueDepth:0,capacity:8192,running:true,closing:false};
 const publication={running:true,poisoned:false,requiresFullRebuild:false,pendingJobs:0,pendingBytes:0,queuedJobs:0,submittedJobs:20,completedJobs:20,failedJobs:0,publishedBatches:8,publishedChanges:20,revalidations:0,pendingRepairMessages:0,erasingAccounts:0,maxPendingJobs:128,maxPendingBytes:16777216,maxBatchDocuments:4096};
 const diagnostics={before:{httpTelemetry:health,searchIndex:{publication}},after:{httpTelemetry:{...health,submitted:40,applied:40},searchIndex:{publication:{...publication,submittedJobs:40,completedJobs:40}}}};
 const full=extra=>({...telemetryFree[0],diagnostics,...extra});
 const ok=(opts,s,loopback=true)=>qualifiesHttpTarget({options:opts,stages:[s],mutationStateTainted:false},true,{loopback});
 assert.equal(ok(options,full()),true,'loopback qualifies throughput when this machine is designated');
 const withoutHost=campaignOptions({MW_LOAD_QUALIFICATION_RUN:'1',MW_LOAD_NETWORK_GBIT:'10'});
 assert.equal(ok(withoutHost,full({transfer:{networkGbitBodyBytes:1}})),false,'a loopback qualification run without the option cannot qualify');
 assert.equal(ok(options,full({acceptedDuringScheduledWindowRps:29999})),false);
 assert.equal(ok(options,full({mix:{acceptedInWindow:{writeShare:.5}}})),false,'the goal mix stage must hold about 35% writes');
 assert.equal(ok(options,full({mix:undefined})),false);
 assert.equal(ok(options,full({endpoint:'mixed',mix:undefined})),true,'the older mixed endpoints keep qualifying the plain rate rule');
 const judged=judgedAgainst(options.goal,{loopback:true,env,system:{availableParallelism:()=>16,platform:()=>'win32',totalmem:()=>34359738368,cpus:()=>[]}});
 assert.equal(judged.network.qualifies,false);
 assert.match(judged.network.reason,/cannot qualify a network claim.*throughput on the designated base hardware may still qualify/);
 assert.equal(judged.qualificationHost,'local');
 assert.deepEqual([judged.host.logicalCores,judged.host.dockerVmVcpus,judged.host.cpusets],[16,12,{app:'0-3',database:'4-5',generator:'6-11'}]);
 assert.equal(judged.workload,GOAL_WORKLOAD);
 assert.match(judged.workload,/65% reads and 35% writes/);
 // undeclared facts are null, never guessed; malformed values are ignored
 const bare=hostFacts({MW_LOAD_HOST_CPUSET_APP:'0;rm'},{availableParallelism:()=>8,platform:()=>'linux',totalmem:()=>1,cpus:()=>[]});
 assert.deepEqual([bare.dockerVmVcpus,bare.cpusets.app,bare.cpusets.database],[null,null,null]);
 // reports show the facts in their first key
 const report={judgedAgainst:judged,target:'t',scenario:'s',runtime:'r',model:'m',stages:[]};
 assert.equal(Object.keys(report)[0],'judgedAgainst');
 assert.equal(markdownReport(report).split('\n')[0],'# HTTP load report');
});

test('MW_LOAD_FIXTURE_ENDPOINTS accepts the goal group and rejects an unknown one',()=>{
 const env={MW_LOAD_FIXTURE_MODE:'capacity',MW_LOAD_FIXTURE_LONG:'1',MW_LOAD_FIXTURE_ENDPOINTS:GOAL_MIX_NAME};
 assert.deepEqual(campaignOptions(env).endpoints,[GOAL_MIX_NAME]);
 assert.throws(()=>campaignOptions({...env,MW_LOAD_FIXTURE_ENDPOINTS:'goal-70-30'}),/ENDPOINTS/);
});
