import test from 'node:test';
import assert from 'node:assert/strict';
import {prepareFixtureRequests} from './fixture-requests.mjs';
import {GOAL_MIX_TABLE,GOAL_MIX_CYCLE,GOAL_MIX_NAME,goalMixSchedule,goalMixReport,goalMixWriteSpecs,goalMixSearchSpec,goalMixDraftSpecs,goalMixSeeded,mixWithDiagnostics,upstreamReport,requestInputsRatio,isRepresentativeKind,workloadProfileReport} from './representative-workload.mjs';
import {goalFromEnv,judgedAgainst,hostFacts,DEFAULT_TARGET_RPS,GOAL_WORKLOAD} from './http-goal.mjs';
import {campaignOptions} from './http-campaign.mjs';
import {qualifiesHttpTarget} from './http-multi-controller.mjs';
import {markdownReport} from './http-load.mjs';
import {MUTATING_ENDPOINTS} from './t34-modes.mjs';

const profile={kind:'representative',messagesPerPrincipal:50,bodySampleBytes:4096,attachmentSampleBytes:65536,seededMessages:50*6,seedSeconds:1};
function users(count=6,messages=50){
 return Array.from({length:count},(_,index)=>({cookie:`mw_session=private-${index}; mw_csrf=csrf-${index}`,csrfToken:`csrf-${index}`,accountId:`acct-${index}`,ownerAccountId:`acct-${index}`,
  messageIds:Array.from({length:messages},(_,n)=>`m-${index}-${n}`),inboxId:`inbox-${index}`,archiveId:`archive-${index}`,trashId:`trash-${index}`,draftsId:`drafts-${index}`,plantedQuarterly:10,attachmentBlobId:`blob-${index}`,attachmentSampleBytes:65536,bodySampleBytes:4096,clientAddress:`198.18.0.${index+1}`}));
}
const prepare=(seeded,workloadProfile=profile)=>prepareFixtureRequests({principals:seeded,clientSimulation:{mode:'fixture-loopback-trusted-proxy',clientCount:seeded.length},engineMode:'native',representative:true,workloadProfile});

test('the weight table is one reviewed table: 65 reads and 35 writes per 100 requests, mutations per request stated per row',()=>{
 const reads=GOAL_MIX_TABLE.filter(row=>row.kind==='read'),writes=GOAL_MIX_TABLE.filter(row=>row.kind==='write');
 assert.equal(reads.reduce((n,row)=>n+row.weight,0),65);
 assert.equal(writes.reduce((n,row)=>n+row.weight,0),35);
 assert.deepEqual(Object.fromEntries(GOAL_MIX_TABLE.map(row=>[row.name,row.weight])),{'authenticated-identity':6,'jmap-session':5,'list-batch':15,'Email/get':9,'body-and-attachment-metadata':10,changes:9,'account-preferences':4,'appearance-customization':3,'search-planted-term':4,'mark-read-and-restore':7,'mark-unread-and-restore':4,'flag-and-restore':6,'unflag-and-restore':3,'move-and-restore':8,'delete-to-trash-and-restore':5,'draft-save':1,'draft-discard':1});
 assert.ok(reads.every(row=>row.mutationsPerRequest===0));
 assert.deepEqual(writes.map(row=>[row.name,row.mutationsPerRequest]).filter(([,n])=>n!==2),[['draft-save',1],['draft-discard',1]],'toggle, move and trash writes carry 2 mutations; the two draft requests carry 1 each');
 assert.equal(new Set(GOAL_MIX_TABLE.map(row=>row.name)).size,GOAL_MIX_TABLE.length);
 assert.ok(Object.isFrozen(GOAL_MIX_TABLE)&&GOAL_MIX_TABLE.every(Object.isFrozen));
 // the draft pair holds exactly two write slots
 assert.equal(writes.filter(row=>/^draft-/.test(row.name)).reduce((n,row)=>n+row.weight,0),2);
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
 assert.deepEqual(bodies.map(update=>Object.keys(update)[0]),['m-0-2','m-0-1','m-0-3','m-0-4']);
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

test('the per-stage mix report: achieved split, verified mutations and per-user line',()=>{
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
 assert.equal(report.perUser.users,null);
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

test('goal seeding is required: fixtures without trash, drafts, search term or five messages have no goal group',()=>{
 assert.equal(goalMixSeeded(users(1)[0]),true);
 for(const patch of [{trashId:undefined},{draftsId:undefined},{plantedQuarterly:undefined},{messageIds:['a','b','c','d']}])assert.equal(goalMixSeeded({...users(1)[0],...patch}),false);
 const bare=users(2).map(({trashId,draftsId,plantedQuarterly,...rest})=>rest);
 assert.equal(prepare(bare).groups[GOAL_MIX_NAME],undefined);
 assert.equal(prepare(bare).groups.representative.length,10);
});

const ok=(id)=>({accountId:'a',updated:{[id]:null}});
const inFolders=(id,folders)=>({accountId:'a',list:[{id,mailboxIds:Object.fromEntries(folders.map(f=>[f,true]))}]});
test('delete-to-trash moves one message to trash and back, verified by read-back',()=>{
 const p=users(1)[0],spec=goalMixWriteSpecs(p).find(row=>row.name==='delete-to-trash-and-restore');
 assert.ok(spec&&spec.statefulMutation&&spec.serializePrincipal);
 const id='m-0-4',good=[ok(id),inFolders(id,['trash-0']),ok(id),inFolders(id,['inbox-0'])];
 assert.equal(spec.validate(respond(spec,good)),true);
 assert.equal(spec.validate(respond(spec,[good[0],inFolders(id,['trash-0','inbox-0']),good[2],good[3]])),false,'must leave the inbox');
 assert.equal(spec.validate(respond(spec,[good[0],good[1],good[2],inFolders(id,['trash-0'])])),false,'must be restored');
 assert.equal(goalMixWriteSpecs({...p,trashId:undefined}).some(row=>row.name==='delete-to-trash-and-restore'),false);
});

test('search of the planted term must find exactly the seeded count',()=>{
 const p=users(1)[0],spec=goalMixSearchSpec(p),ids=Array.from({length:10},(_,i)=>'s'+i);
 const rows=(total,list)=>respond(spec,[{ids,total},{list:list.map(id=>({id}))}]);
 assert.equal(spec.validate(rows(10,ids)),true);
 assert.equal(spec.validate(rows(9,ids)),false);
 assert.equal(spec.validate(rows(10,ids.slice(1))),false);
 assert.deepEqual(JSON.parse(spec.body).methodCalls[0][1].filter,{inMailbox:'inbox-0',text:'quarterly'});
 assert.throws(()=>goalMixSearchSpec({...p,plantedQuarterly:undefined}));
});

test('draft save is two requests: a create, then a get + destroy of the created id, one mutation each',()=>{
 const p=users(1)[0],created=[],{save,discard}=goalMixDraftSpecs(p,{onCreated:id=>created.push(id)});
 assert.ok(save.statefulMutation&&discard('x').statefulMutation);
 assert.deepEqual(JSON.parse(save.body).methodCalls.map(c=>c[0]),['Email/set']);
 assert.ok(JSON.parse(save.body).methodCalls[0][1].create.d1.mailboxIds['drafts-0']);
 assert.equal(JSON.stringify(JSON.parse(save.body)).includes('#d1'),false,'no creation back-reference');
 assert.equal(save.validate(respond(save,[{created:{d1:{id:'draft-9'}}}])),true);
 assert.deepEqual(created,['draft-9']);
 assert.equal(save.validate(respond(save,[{notCreated:{d1:{type:'forbidden'}}}])),false);
 assert.equal(created.length,1,'a failed save hands nothing on');
 const next=discard('draft-9');
 assert.deepEqual(JSON.parse(next.body).methodCalls.map(c=>c[0]),['Email/get','Email/set']);
 assert.deepEqual(JSON.parse(next.body).methodCalls[1][1].destroy,['draft-9']);
 const good=[{list:[{id:'draft-9',mailboxIds:{'drafts-0':true},keywords:{$draft:true}}]},{destroyed:['draft-9']}];
 assert.equal(next.validate(respond(next,good)),true);
 assert.equal(next.validate(respond(next,[good[0],{destroyed:[]}])),false);
 assert.equal(next.validate(respond(next,[{list:[]},good[1]])),false);
});

test('a discard slot destroys the oldest saved draft for the user that saved it; with none it creates',()=>{
 const seeded=users(4),group=prepare(seeded).groups[GOAL_MIX_NAME];
 const save=group.findIndex(spec=>spec.name==='draft-save'),discardSlot=group.findIndex(spec=>spec.name==='draft-discard');
 assert.ok(save>=0&&discardSlot>=0);
 const first=group[discardSlot].materialize(discardSlot,{requestCount:100});
 assert.equal(JSON.parse(first.body).methodCalls[0][0],'Email/set','nothing saved yet: the slot creates a draft');
 const made=group[save].materialize(save+100,{requestCount:100});
 assert.equal(made.validate(respond(made,[{created:{d1:{id:'d-77'}}}])),true);
 const destroy=group[discardSlot].materialize(discardSlot+200,{requestCount:100});
 assert.equal(destroy.principalIndex,made.principalIndex);
 assert.deepEqual(JSON.parse(destroy.body).methodCalls[1][1].destroy,['d-77']);
 assert.equal(destroy.headers.cookie,seeded[made.principalIndex].cookie);
 assert.equal(destroy.serializePrincipal,true);
 assert.equal(JSON.parse(group[discardSlot].materialize(discardSlot+300,{requestCount:100}).body).methodCalls[0][0],'Email/set','the pool is consumed');
});

test('upstream commands per second come from the diagnostics delta, split by kind and per write request',()=>{
 const before={commands:1000,fetch:600,store:100,move:40,expunge:10,append:20,list:230,fetchedMessages:5},after={commands:2500,fetch:1300,store:400,move:140,expunge:30,append:40,list:590,fetchedMessages:9};
 const up=upstreamReport(before,after,{seconds:10,writeRequests:100});
 assert.equal(up.available,true);
 assert.equal(up.commandsPerSecond,150);
 assert.deepEqual(up.perKindPerSecond,{store:30,move:10,expunge:2,append:2,fetch:70,list:36});
 assert.equal(up.writeCommands,300+100+20+20);
 assert.equal(up.writeCommandsPerWriteRequest,4.4);
 assert.equal(upstreamReport(before,{...after,commands:5},{seconds:10,writeRequests:1}).available,false,'a reset counter is not a delta');
 assert.equal(upstreamReport(undefined,after,{seconds:10,writeRequests:1}).available,false);
 const stat=(requests,accepted)=>({requests,accepted,acceptedDuringScheduledWindow:accepted});
 const mix=goalMixReport({'list-batch':stat(65,65),'mark-read-and-restore':stat(35,35)},10,{users:5,meanLatencyMs:20});
 assert.equal(mix.perUser.acceptedRequestsPerSecondPerUser,2);
 assert.equal(mix.perUser.serialisedCeilingPerUser,50);
 const ratios=requestInputsRatio({ordinaryRequestInputs:{consumedFrames:10,ordinaryContexts:100}},{ordinaryRequestInputs:{consumedFrames:70,ordinaryContexts:200}});
 assert.deepEqual(ratios,{consumedFramesPerContext:.6,consumedFrames:60,ordinaryContexts:100});
 const full=mixWithDiagnostics(mix,{before:{upstreamCommands:before,ordinaryRequestInputs:{consumedFrames:10,ordinaryContexts:100}},after:{upstreamCommands:after,ordinaryRequestInputs:{consumedFrames:70,ordinaryContexts:200}}},10);
 assert.equal(full.upstreamCommandsPerSecond,150);
 assert.equal(full.upstream.writeCommandsPerWriteRequest,440/35);
 assert.equal(full.requestInputs.consumedFramesPerContext,.6);
 const none=mixWithDiagnostics(mix,{before:{},after:{}},10);
 assert.equal(none.upstreamCommandsPerSecond,null);
 assert.equal(none.upstream.available,false);
 assert.equal(none.requestInputs,null);
 assert.equal(mixWithDiagnostics(undefined,{},10),undefined);
});

test('t34 modes treat goal-65-35 as a mutating endpoint group and keep the existing ones',()=>{
 assert.deepEqual([...MUTATING_ENDPOINTS],['representative','flag-write','move-write','goal-65-35']);
 for(const read of ['mixed','identity','list-batch','changes','account-polls'])assert.ok(!MUTATING_ENDPOINTS.includes(read));
});
