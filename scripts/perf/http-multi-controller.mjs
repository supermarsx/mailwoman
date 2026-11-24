import {requestInputsEvidence} from './request-inputs-experiment.mjs';
import {t34ModesEvidence} from './t34-modes.mjs';
import {enforcedPolicyReport,assertEnforcedPolicySnapshot} from './native-enforced-policy.mjs';
import {prepareFixtureRequests,CLIENT_HOT_GROUPS} from './fixture-requests.mjs';
import {preparedMailExperimentReport,preparedMailEvidence} from './prepared-mail-experiment.mjs';
import {throwDiagnosticSnapshotFailure,diagnosticFailureReport} from './fixture-diagnostic-failure.mjs';
import {campaignCacheExperiment,assertSqliteCacheWitness,cacheExperimentEvidence} from './sqlite-page-cache-witness.mjs';
import {qualifiesIndexPublication} from './index-publication-health.mjs';
// Controller for an explicitly owned native fixture. No arbitrary remote URL.
import {fork} from 'node:child_process';
import {mkdtemp,writeFile,unlink,rmdir,mkdir} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import {dirname,join,resolve} from 'node:path';
import {fileURLToPath,pathToFileURL} from 'node:url';
import {performance} from 'node:perf_hooks';
import {readPrivate,HANDOFF_LIMIT} from './external-control.mjs';
import {markdownReport,MAX_WORKERS_PER_HOST,MAX_WORKERS_TOTAL} from './http-load.mjs';
import {campaignOptions,runCampaign} from './http-campaign.mjs';
import {qualifiesHttpTelemetry,fixtureDiagnosticsReport,prepareFixtureClients,startupReadinessReport,fixturePolicyReport,searchWriterReport,linkedLoginPolicyProbeReport,searchStorageReport,qualifiesSearchStorage} from './fixture-clients.mjs';
import {workloadProfileReport,isRepresentativeKind,GOAL_MIX_NAME} from './representative-workload.mjs';
import {partitionPrincipals,validateStage,aggregateWorkers} from './http-multi-protocol.mjs';
import {verifyFixtureIsolation} from './fixture-isolation.mjs';
import {childChannel} from './http-multi-channel.mjs';
import {remoteHost} from './http-multi-host.mjs';
import {validateTargets,proveOwnership,parseAllowList} from './http-targets.mjs';
import {createH2Client,validateH2} from './http-h2-client.mjs';
import {goalOf,judgedAgainst,withinNetworkBudget,qualificationBlocked,goalMixShareOk} from './http-goal.mjs';

// `hostWorkers`: worker counts of the additional generator hosts (controller of controllers). `workers` is this host's own count.
export function validateAssignment(handoff,workers,allowedTargets=[],hostWorkers=[],h2){
 const tls={tls:h2?.mode==='tls'};
 if(!Array.isArray(hostWorkers)||hostWorkers.length>MAX_WORKERS_TOTAL||hostWorkers.some(count=>!Number.isInteger(count)||count<1||count>MAX_WORKERS_PER_HOST))throw new Error(`Each additional host runs 1–${MAX_WORKERS_PER_HOST} workers`);
 const totalWorkers=workers+hostWorkers.reduce((a,b)=>a+b,0);
 if(handoff?.engineMode!=='native'||!Number.isInteger(workers)||workers<(hostWorkers.length?0:1)||workers>MAX_WORKERS_PER_HOST||totalWorkers>MAX_WORKERS_TOTAL)throw new Error(`Explicit native fixture and 1–${MAX_WORKERS_PER_HOST} workers per host required`);
 // Default: one owned literal-loopback fixture. Several private rig targets only when the operator allow-listed each address.
 let targets;
 if(handoff.targets===undefined){
  const base=new URL(handoff.base);
  if(base.protocol!==(tls.tls?'https:':'http:')||base.hostname!=='127.0.0.1'||!base.port||base.pathname!=='/'||base.search||base.hash||base.username||base.password)throw new Error('Only an owned literal-loopback fixture is supported');
  targets=validateTargets([base.origin],[],tls);
 }else{
  try{targets=validateTargets(handoff.targets,allowedTargets,tls);}catch(error){throw new Error('Target is not an owned loopback or allow-listed private rig address: '+error.message);}
  if(handoff.base!==undefined&&handoff.base!==targets[0].origin&&handoff.base!==targets[0].origin+'/')throw new Error('Handoff base must equal the first target');
 }
 const clientModel=prepareFixtureClients(handoff.principals,handoff.clientSimulation);
 if(clientModel.clientCount!==handoff.principals.length||handoff.uniquePrincipalCount!==handoff.principals.length)throw new Error('Independent seeded identities and simulated clients required');
 const startupReadiness=startupReadinessReport(handoff.startupReadiness,{required:true}),policyEvidence=fixturePolicyReport(handoff.policyEvidence),workloadProfile=workloadProfileReport(handoff.workloadProfile,handoff.principals.length);
 if(!workloadProfile||startupReadiness.seededMessages!==workloadProfile.seededMessages||handoff.principals.some(p=>!Array.isArray(p.messageIds)||p.messageIds.length!==workloadProfile.messagesPerPrincipal||new Set(p.messageIds).size!==p.messageIds.length))throw new Error('Global seed proof and assigned message counts differ');
 if(!['release','debug','fast','unknown'].includes(handoff.buildProfile)||!['sqlite','postgres'].includes(handoff.backend))throw new Error('Invalid fixture build metadata');
 return {enforcedPolicy:enforcedPolicyReport(handoff.enforcedPolicy),base:targets[0].origin,targets,partitions:partitionPrincipals(handoff.principals,totalWorkers),totalWorkers,startupReadiness,policyEvidence,workloadProfile,clientModel,searchWriter:searchWriterReport(handoff.searchWriter),searchStorage:searchStorageReport(handoff.searchStorage),linkedLoginPolicyProbe:linkedLoginPolicyProbeReport(handoff.linkedLoginPolicyProbe)};
}

// A failure after the load finished (scanner probe, receipt or evidence checks)
// must not discard the measurement. The original error is wrapped with its
// message (only when it is a plain fixed-text sentence, never paths or values)
// and the raw stages the campaign already collected.
const SAFE_MESSAGE=/^[A-Za-z0-9 .,'()_-]{1,200}$/;
export class PostRunFailure extends Error{
 constructor(error,campaign){
  const text=typeof error?.message==='string'&&SAFE_MESSAGE.test(error.message)?error.message:'unclassified post-run failure';
  super(text);this.name='PostRunFailure';this.failureMessage=text;this.rawCampaign=campaign;this.cause=error;
 }
}
export function postRunFailure(error,campaign){return error instanceof PostRunFailure?error:new PostRunFailure(error,campaign);}
// Bring a remote worker's window (host clock) onto the controller's clock.
export function shiftWindow(result,offsetMs){
 if(!offsetMs||!result?.measurement?.window)return result;
 const window={...result.measurement.window};
 for(const key of ['requestedStartAtMs','scheduledStartAtMs','scheduledEndAtMs','actualStartAtMs','completedAtMs'])if(Number.isFinite(window[key]))window[key]-=offsetMs;
 return {...result,measurement:{...result.measurement,window}};
}
export function failureReportFor(error){
 const report=diagnosticFailureReport(error);
 if(error instanceof PostRunFailure)return {...report,failedAfterLoad:true,failureMessage:error.failureMessage,rawCampaign:error.rawCampaign};
 return report;
}

// The target comes from campaign.options.goal (MW_LOAD_TARGET_RPS / _USERS / MW_LOAD_NETWORK_GBIT); defaults are the standing 40,000 / 10,000.
// `endpoints` narrows which stage groups count (the goal-mix claim uses only goal-65-35, whose accepted write share must be 30-40%).
export function qualifiesHttpTarget(campaign,qualified,{loopback=false,endpoints=['mixed','representative',GOAL_MIX_NAME]}={}){
 const goal=goalOf(campaign.options);
 return qualified&&!qualificationBlocked(goal,{loopback})&&!campaign.mutationStateTainted&&campaign.stages.some(stage=>endpoints.includes(stage.endpoint)&&(stage.endpoint!==GOAL_MIX_NAME||goalMixShareOk(stage))&&stage.phase==='offered-rate'&&stage.arrivalRate>=goal.successfulRps&&stage.acceptedDuringScheduledWindowRps>=goal.successfulRps&&stage.uniquePrincipalsAcceptedDuringScheduledWindow>=goal.simultaneousAuthenticatedUsers&&stage.uniqueClientsAcceptedDuringScheduledWindow>=goal.simultaneousAuthenticatedUsers&&withinNetworkBudget(stage,goal)&&stage.coordination.valid&&!stage.mutationStateTainted&&stage.scheduledSeconds>=30&&stage.errors===0&&stage.timeouts===0&&stage.throttled===0&&stage.droppedCapacity===0&&stage.droppedScheduler===0&&stage.generatorLimited!==true&&qualifiesHttpTelemetry(stage.diagnostics)&&qualifiesIndexPublication(stage.diagnostics));
}

export async function runMultiCampaign({handoff,workers,options,launch=fork,startLeadMs=2000,maxStartLagMs=50,scannerProbe,isolationProbe,allowedTargets=[],targetFetch,hosts=[],hostFactory=remoteHost}){
 const h2=options.h2===undefined?undefined:validateH2(options.h2);
 // Probes (ownership proof, diagnostics, isolation, scanner) use the campaign's protocol and the same target rules.
 const probeClient=h2?createH2Client(h2):null;
 targetFetch??=probeClient?(url,init)=>probeClient.oneShot(url,init):fetch;
 const verified=validateAssignment(handoff,workers,allowedTargets,hosts.map(host=>host.workers),h2);
 const totalWorkers=verified.totalWorkers;
 const provenTargets=await proveOwnership(verified.targets,{path:handoff.diagnostics?.path,token:handoff.diagnostics?.token,fetchImpl:targetFetch});
 const preparedMailExperiment=preparedMailExperimentReport(handoff.preparedMailExperiment);
 const readIntent=prepareFixtureRequests({principals:handoff.principals.slice(0,1),clientSimulation:{mode:'fixture-loopback-trusted-proxy',clientCount:1},engineMode:'native',representative:isRepresentativeKind(verified.workloadProfile?.kind),workloadProfile:verified.workloadProfile}).readIntent;
 const cacheExperiment=campaignCacheExperiment(handoff,options);
 if(!Number.isInteger(startLeadMs)||startLeadMs<500||startLeadMs>5000)throw new Error('Invalid readiness lead');
 if(!Number.isInteger(options.warmupSeconds??0)||(options.warmupSeconds??0)<0||(options.warmupSeconds??0)>60)throw new Error('Coordinated warmup must use integer seconds');
 for(const concurrency of options.concurrency)validateStage({workers:totalWorkers,concurrency,seconds:options.seconds,timeoutMs:options.timeoutMs});
 for(const arrivalRate of options.rates)validateStage({workers:totalWorkers,concurrency:options.concurrency.at(-1),seconds:options.seconds,timeoutMs:options.timeoutMs,arrivalRate});
 const observedIsolation=await (isolationProbe??verifyFixtureIsolation)({base:verified.base,principals:handoff.principals,clients:verified.clientModel.clients,...(h2?{request:targetFetch}:{})});
 const isolated=observedIsolation?.verified===true&&observedIsolation.status==='verified'&&observedIsolation.requests===2&&observedIsolation.directions===2;
 const singlePrincipal=handoff.principals.length===1&&observedIsolation?.verified===false&&observedIsolation.status==='insufficient-principals'&&observedIsolation.requests===0;
 if(!isolated&&!singlePrincipal)throw new Error('Fixture account isolation was not observed');
 const isolationEvidence=isolated?{verified:true,status:'verified',requests:2,directions:2}:{verified:false,status:'insufficient-principals',requests:0};
 const directory=await mkdtemp(join(tmpdir(),'mw-multi-http-')),files=[],children=[];let stageId=0;
 const workerData=part=>({schema:1,workerIndex:part.index,engineMode:'native',base:verified.base,principals:part.principals,...(h2?{h2}:{}),...(verified.targets.length>1||verified.targets[0].kind!=='loopback'?{targets:provenTargets,allowedTargets:parseAllowList(allowedTargets)}:{}),clientSimulation:{mode:'fixture-loopback-trusted-proxy',clientCount:part.count},representative:isRepresentativeKind(verified.workloadProfile?.kind),workloadProfile:verified.workloadProfile});
 const handles=[],hostRecords=[];
 const cpuStarted=process.cpuUsage(),controllerStarted=performance.now(),rssBeforeBytes=process.memoryUsage().rss;
 try{
  for(const part of verified.partitions.slice(0,workers)){
   const path=join(directory,`worker-${part.index}.private.json`);files.push(path);
   await writeFile(path,JSON.stringify(workerData(part)),{mode:0o600,flag:'wx'});
   // Inherit only OS runtime essentials, never server/DSN/token environment.
   const env=Object.fromEntries(['SystemRoot','WINDIR','TEMP','TMP','TZ'].filter(key=>process.env[key]!==undefined).map(key=>[key,process.env[key]]));
   const child=launch(join(dirname(fileURLToPath(import.meta.url)),'http-multi-worker.mjs'),[path],{env,stdio:['ignore','ignore','ignore','ipc'],execArgv:[]});children.push(childChannel(child,part.index));
  }
  // Additional generator hosts: this controller is the controller of controllers. Each host's clock offset is
  // measured here and again before every stage; workers there get the same disjoint slices and raw histograms come back.
  let nextPartition=workers;
  for(const [hostIndex,host] of hosts.entries()){
   const handle=hostFactory({origin:host.origin,token:host.token});handles.push(handle);
   const clock=await handle.measureClock(),slice=verified.partitions.slice(nextPartition,nextPartition+host.workers);nextPartition+=host.workers;
   await handle.init(slice.map(part=>({workerIndex:part.index,data:workerData(part)})));
   const channels=slice.map(part=>handle.channel(part.index,clock.offsetMs));children.push(...channels);
   hostRecords.push({id:'host-'+(hostIndex+1),origin:handle.origin??host.origin,workerIndexes:slice.map(part=>part.index),handle,channels,clock});
  }
  const ready=await Promise.all(children.map(child=>child.wait('ready',undefined,120000)));
  if(ready.some((value,index)=>value.schema!==1||value.workerIndex!==index||value.partitionCount!==verified.partitions[index].count))throw new Error('Worker identity allocation mismatch');
  const diagnostics=async phase=>{
   if(!handoff.diagnostics)return null;
   const path=handoff.diagnostics.path;if(typeof path!=='string'||!/^\/__fixture-diagnostic-[a-f0-9-]+$/.test(path))throw new Error('Invalid diagnostic control path');
   const init={headers:{'x-fixture-token':handoff.diagnostics.token},redirect:'error',signal:AbortSignal.timeout(5000)};
   if(phase==='before'){const reset=await targetFetch(new URL(path,verified.base),{...init,method:'POST'});if(!reset.ok)throw new Error('Diagnostic reset failed');await reset.body?.cancel();}
   const response=await targetFetch(new URL(path,verified.base),init);if(!response.ok)await throwDiagnosticSnapshotFailure(response);const snapshot=fixtureDiagnosticsReport(await response.json());if(cacheExperiment)assertSqliteCacheWitness(snapshot,cacheExperiment);assertEnforcedPolicySnapshot(snapshot,verified.enforcedPolicy);return snapshot;
  };
  const run=async requestedStage=>{
   // runCampaign warms at one outstanding request. A partitioned warmup needs
   // one slot per worker; measured ramp budgets were validated unchanged above.
   const stage=requestedStage.concurrency<totalWorkers&&requestedStage.arrivalRate===undefined
    ?{...requestedStage,concurrency:totalWorkers}:requestedStage;
   const endpoint=stage.requests[0].name,budget=validateStage({...stage,workers:totalWorkers}),id=stageId++;
   for(const record of hostRecords){record.clock=await record.handle.measureClock();for(const channel of record.channels)channel.clockOffsetMs=record.clock.offsetMs;}
   const clockToleranceMs=hostRecords.reduce((n,record)=>Math.max(n,record.clock.uncertaintyMs+1),1);
   for(const child of children)child.send({type:'prepare',stageId:id,endpoint,concurrency:budget.concurrency[child.index],seconds:stage.seconds,timeoutMs:stage.timeoutMs,...(stage.arrivalRate===undefined?{}:{arrivalRate:budget.rates[child.index],...(stage.latePolicy===undefined?{}:{latePolicy:stage.latePolicy})})});
   const acknowledgments=await Promise.all(children.map(child=>child.wait('stage-ready',id,10000)));
   if(acknowledgments.some((ack,index)=>ack.workerIndex!==index))throw new Error('Worker readiness mismatch');
   const startAtMs=Date.now()+startLeadMs;
   for(const child of children)child.send({type:'start',stageId:id,startAtMs:startAtMs+(child.clockOffsetMs??0)});
   const results=(await Promise.all(children.map(child=>child.wait('result',id,startLeadMs+(stage.seconds+stage.timeoutMs/1000+10)*1000)))).map((row,index)=>shiftWindow(row,children[index].clockOffsetMs??0));
   const observedAtMs=Date.now();
   if(results.some(row=>row.measurement?.window?.completedAtMs>observedAtMs+maxStartLagMs+clockToleranceMs-1))throw new Error('Worker completion is ahead of the controller clock');
   return aggregateWorkers({results,partitions:verified.partitions,stage,startAtMs,maxStartLagMs,clockToleranceMs,...(hostRecords.length?{hosts:hostRecords.map(record=>({id:record.id,origin:record.origin,workerIndexes:record.workerIndexes,...record.clock}))}:{})});
  };
  const names=['mixed','identity','mailboxes','query','headers','refresh',...Object.keys(CLIENT_HOT_GROUPS),...(isRepresentativeKind(verified.workloadProfile?.kind)?['representative','paged-headers','reader','attachment','flag-write','move-write',GOAL_MIX_NAME]:[])],groups=Object.fromEntries(names.map(name=>[name,[{name}]]));
  let campaign;
  try{campaign=await runCampaign({base:verified.base,headers:{},groups,options,run,diagnostics});}catch(error){throw postRunFailure(error,error?.partialCampaign??{stages:[]});}
  try{
  // Preserve the single native driver's actual enforcement check. This
  // anonymous probe is outside measured phases and cannot add successful RPS.
  const scannerProbeStatus=await (scannerProbe??(async()=>{
   const response=await targetFetch(new URL('/.env',verified.base),{redirect:'error',signal:AbortSignal.timeout(5000)});
   await response.body?.cancel();return response.status;
  }))();
  if(![403,429].includes(scannerProbeStatus))throw new Error('Native scanner enforcement was not observed');
  const elapsedSeconds=(performance.now()-controllerStarted)/1000,cpu=process.cpuUsage(cpuStarted);
  const policyQualified=handoff.buildProfile==='release'&&verified.policyEvidence.verified&&verified.startupReadiness.verified&&isolationEvidence.verified;
  const storageQualified=qualifiesSearchStorage(verified.searchStorage);
  const telemetryQualified=campaign.stages.length>0&&campaign.stages.every(stage=>qualifiesHttpTelemetry(stage.diagnostics));
  const qualified=policyQualified&&storageQualified;
  const goal=goalOf(options),loopback=verified.targets.some(target=>target.kind==='loopback'),met=qualifiesHttpTarget(campaign,qualified,{loopback});
  return {judgedAgainst:judgedAgainst(goal,{loopback}),enforcedPolicy:verified.enforcedPolicy,readIntent,ordinaryRequestInputs:requestInputsEvidence(handoff.ordinaryRequestInputs,campaign.stages,options.requestInputsExpected,{backend:handoff.backend}),t34Modes:t34ModesEvidence(handoff.t34Modes,campaign.stages,{requestInputsEvidence:requestInputsEvidence(handoff.ordinaryRequestInputs,campaign.stages,options.requestInputsExpected,{backend:handoff.backend})}),preparedMailExperiment:preparedMailEvidence(preparedMailExperiment,campaign.stages),...(cacheExperiment?{sqliteCacheExperiment:cacheExperimentEvidence(cacheExperiment,campaign.stages)}:{}),schema:1,kind:'coordinated-native-http-fixture',generatedAt:new Date().toISOString(),target:'ephemeral loopback Mailwoman router + seeded native engine',scenario:'real authenticated middleware/HTTP/native-engine with '+handoff.backend+'; fixture data; no TLS/reverse proxy',runtime:process.version,model:'bounded coordinated processes with disjoint users and a shared measurement window; no capacity claim',protection:verified.policyEvidence.verified?'administrator defaults plus verified locked access.protocols policy':'administrator defaults; loaded policy not verified',uniqueAuthenticatedPrincipals:handoff.principals.length,principalCountMeaning:'configured distinct seeded principals; actual successful HTTP coverage is reported per stage',backend:handoff.backend,buildProfile:handoff.buildProfile,workers,configuredGlobalPrincipals:handoff.principals.length,scannerProbeStatus,isolationEvidence,partitionSizes:verified.partitions.map(part=>part.count),startupReadiness:verified.startupReadiness,searchWriter:verified.searchWriter,searchStorage:verified.searchStorage,linkedLoginPolicyProbe:verified.linkedLoginPolicyProbe,policyEvidence:verified.policyEvidence,workloadProfile:verified.workloadProfile,...campaign,targets:verified.targets.map(target=>({origin:target.origin,kind:target.kind,ownershipProof:target.kind==='loopback'?'own loopback fixture':'control token accepted and token gate confirmed'})),targetRotation:verified.targets.length>1?'client-side, one target per request in order':'single target',controller:{cpuUserMs:cpu.user/1000,cpuSystemMs:cpu.system/1000,elapsedSeconds,rssBeforeBytes,rssAfterBytes:process.memoryUsage().rss},capacityTarget:{successfulRps:goal.successfulRps,simultaneousAuthenticatedUsers:goal.simultaneousAuthenticatedUsers,httpMultiUserTargetMet:met,goalMixMet:qualifiesHttpTarget(campaign,qualified,{loopback,endpoints:[GOAL_MIX_NAME]}),productionPolicyQualified:policyQualified,productionStorageQualified:storageQualified,productionTelemetryQualified:telemetryQualified,productionIndexPublicationQualified:campaign.stages.length>0&&campaign.stages.every(stage=>qualifiesIndexPublication(stage.diagnostics)),persistentWebSocketSessionsMeasured:false,met:false},distinctPhysicalGeneratorHostsMeasured:hosts.length>0,generatorHosts:[{id:'controller-host',workers,workerIndexes:verified.partitions.slice(0,workers).map(part=>part.index)},...hostRecords.map(record=>({id:record.id,origin:record.origin,workers:record.workerIndexes.length,workerIndexes:record.workerIndexes,clock:record.clock}))]};
  }catch(error){throw postRunFailure(error,campaign);}
 }finally{
  probeClient?.close();
  await Promise.allSettled(handles.map(handle=>handle.shutdown()));
  const stopped=await Promise.allSettled(children.map(child=>child.stop()));
  if(stopped.some(value=>value.status==='rejected'))throw new Error('Worker shutdown unconfirmed; private files retained');
  for(const path of files)await unlink(path);await rmdir(directory);
 }
}

export async function writeMultiArtifacts(output,report){
 const markdownPath=output.replace(/\.json$/,'')+'.md';
 if(markdownPath===output)throw new Error('Distinct JSON and Markdown report paths required');
 await mkdir(dirname(resolve(output)),{recursive:true});
 const markdown=markdownReport(report);
 await writeFile(output,JSON.stringify(report,null,2)+'\n',{mode:0o600});
 await writeFile(markdownPath,markdown,{mode:0o600});
}

async function main(){
 const args=Object.fromEntries(process.argv.slice(2).map(arg=>{const at=arg.indexOf('=');if(at<0)throw new Error();return [arg.slice(2,at),arg.slice(at+1)];}));
 if(Object.keys(args).some(key=>!['handoff','output','workers','allow-targets','hosts-file','http2','h2-preset','h2-connections','h2-max-streams','h2-ca-file','h2-servername'].includes(key))||!args.handoff||!args.output)throw new Error('Explicit private handoff and output required');
 await mkdir(dirname(resolve(args.output)),{recursive:true});
 const workers=Number(args.workers??4),flagEnv=Object.fromEntries([['http2','MW_LOAD_FIXTURE_HTTP2'],['h2-preset','MW_LOAD_FIXTURE_H2_PRESET'],['h2-connections','MW_LOAD_FIXTURE_H2_CONNECTIONS'],['h2-max-streams','MW_LOAD_FIXTURE_H2_MAX_STREAMS'],['h2-ca-file','MW_LOAD_FIXTURE_H2_CA_FILE'],['h2-servername','MW_LOAD_FIXTURE_H2_SERVERNAME']].filter(([flag])=>args[flag]!==undefined).map(([flag,name])=>[name,args[flag]])),options=campaignOptions({...process.env,...flagEnv,MW_LOAD_FIXTURE_CONCURRENCY:process.env.MW_LOAD_FIXTURE_CONCURRENCY??String(workers)});
 try{
  const report=await runMultiCampaign({handoff:JSON.parse(await readPrivate(args.handoff,HANDOFF_LIMIT)),workers,options,allowedTargets:parseAllowList(args['allow-targets']??''),hosts:args['hosts-file']?JSON.parse(await readPrivate(args['hosts-file'],HANDOFF_LIMIT)):[]});
  await writeMultiArtifacts(args.output,report);
  if(report.mutationStateTainted||!report.stages.some(stage=>stage.accepted>0))process.exitCode=1;
 }catch(error){
  const failure=failureReportFor(error);
  await writeMultiArtifacts(args.output,failure);
  throw new Error('Coordinated generator failed');
 }
}
if(process.argv[1]&&import.meta.url===pathToFileURL(resolve(process.argv[1])).href)main().catch(()=>{console.error('Coordinated generator failed; no credentials logged.');process.exitCode=1;});
