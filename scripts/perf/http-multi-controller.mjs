import {qualifiesIndexPublication} from './index-publication-health.mjs';
// Controller for an explicitly owned native fixture. No arbitrary remote URL.
import {fork} from 'node:child_process';
import {mkdtemp,writeFile,unlink,rmdir,mkdir} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import {dirname,join,resolve} from 'node:path';
import {fileURLToPath,pathToFileURL} from 'node:url';
import {performance} from 'node:perf_hooks';
import {readPrivate,HANDOFF_LIMIT} from './external-control.mjs';
import {markdownReport} from './http-load.mjs';
import {campaignOptions,runCampaign} from './http-campaign.mjs';
import {qualifiesHttpTelemetry,fixtureDiagnosticsReport,prepareFixtureClients,startupReadinessReport,fixturePolicyReport,searchWriterReport,linkedLoginPolicyProbeReport,searchStorageReport,qualifiesSearchStorage} from './fixture-clients.mjs';
import {workloadProfileReport} from './representative-workload.mjs';
import {partitionPrincipals,validateStage,aggregateWorkers} from './http-multi-protocol.mjs';
import {verifyFixtureIsolation} from './fixture-isolation.mjs';

export function validateAssignment(handoff,workers){
 if(handoff?.engineMode!=='native'||!Number.isInteger(workers)||workers<1||workers>16)throw new Error('Explicit native fixture and 1–16 workers required');
 const base=new URL(handoff.base);
 if(base.protocol!=='http:'||base.hostname!=='127.0.0.1'||!base.port||base.pathname!=='/'||base.search||base.hash||base.username||base.password)throw new Error('Only an owned literal-loopback fixture is supported');
 const clientModel=prepareFixtureClients(handoff.principals,handoff.clientSimulation);
 if(clientModel.clientCount!==handoff.principals.length||handoff.uniquePrincipalCount!==handoff.principals.length)throw new Error('Independent seeded identities and simulated clients required');
 const startupReadiness=startupReadinessReport(handoff.startupReadiness,{required:true}),policyEvidence=fixturePolicyReport(handoff.policyEvidence),workloadProfile=workloadProfileReport(handoff.workloadProfile,handoff.principals.length);
 if(!workloadProfile||startupReadiness.seededMessages!==workloadProfile.seededMessages||handoff.principals.some(p=>!Array.isArray(p.messageIds)||p.messageIds.length!==workloadProfile.messagesPerPrincipal||new Set(p.messageIds).size!==p.messageIds.length))throw new Error('Global seed proof and assigned message counts differ');
 if(!['release','debug','fast','unknown'].includes(handoff.buildProfile)||!['sqlite','postgres'].includes(handoff.backend))throw new Error('Invalid fixture build metadata');
 return {base:base.origin,partitions:partitionPrincipals(handoff.principals,workers),startupReadiness,policyEvidence,workloadProfile,clientModel,searchWriter:searchWriterReport(handoff.searchWriter),searchStorage:searchStorageReport(handoff.searchStorage),linkedLoginPolicyProbe:linkedLoginPolicyProbeReport(handoff.linkedLoginPolicyProbe)};
}

function childChannel(child,index){
 const inbox=[],waiters=[];let failed=false,exited=false;
 const fail=()=>{failed=true;for(const waiter of waiters.splice(0))waiter.reject(new Error('Worker exited or failed'));};
 child.on('message',message=>{
  if(message?.type==='failed'){fail();return;}
  const at=waiters.findIndex(waiter=>waiter.type===message?.type&&waiter.stageId===message.stageId);
  if(at<0){if(inbox.length>8)fail();else inbox.push(message);return;}
  waiters.splice(at,1)[0].resolve(message);
 });
 child.once('error',fail);child.once('exit',()=>{exited=true;fail();});
 return {index,send:message=>{if(failed||!child.connected)throw new Error('Worker is unavailable');child.send(message);},wait(type,stageId,timeoutMs){
  if(failed)return Promise.reject(new Error('Worker is unavailable'));
  const at=inbox.findIndex(message=>message?.type===type&&message.stageId===stageId);
  if(at>=0)return Promise.resolve(inbox.splice(at,1)[0]);
  return new Promise((resolve,reject)=>{
   let timer;const waiter={type,stageId,resolve:value=>{clearTimeout(timer);resolve(value);},reject:error=>{clearTimeout(timer);reject(error);}};
   waiters.push(waiter);timer=setTimeout(()=>{const at=waiters.indexOf(waiter);if(at>=0)waiters.splice(at,1);reject(new Error('Worker coordination deadline exceeded'));},timeoutMs);
  });
 },async stop(){
  if(exited)return;
  child.kill('SIGKILL');
  await new Promise((resolve,reject)=>{const timer=setTimeout(()=>reject(new Error('Worker exit unconfirmed')),5000);child.once('exit',()=>{clearTimeout(timer);resolve();});if(exited){clearTimeout(timer);resolve();}});
 }};
}

export function qualifiesHttpTarget(campaign,qualified){
 return qualified&&!campaign.mutationStateTainted&&campaign.stages.some(stage=>['mixed','representative'].includes(stage.endpoint)&&stage.phase==='offered-rate'&&stage.arrivalRate>=40000&&stage.acceptedDuringScheduledWindowRps>=40000&&stage.uniquePrincipalsAcceptedDuringScheduledWindow>=10000&&stage.uniqueClientsAcceptedDuringScheduledWindow>=10000&&stage.coordination.valid&&!stage.mutationStateTainted&&stage.scheduledSeconds>=30&&stage.errors===0&&stage.timeouts===0&&stage.throttled===0&&stage.droppedCapacity===0&&stage.droppedScheduler===0&&qualifiesHttpTelemetry(stage.diagnostics)&&qualifiesIndexPublication(stage.diagnostics));
}

export async function runMultiCampaign({handoff,workers,options,launch=fork,startLeadMs=2000,maxStartLagMs=50,scannerProbe,isolationProbe}){
 const verified=validateAssignment(handoff,workers);
 if(!Number.isInteger(startLeadMs)||startLeadMs<500||startLeadMs>5000)throw new Error('Invalid readiness lead');
 if(!Number.isInteger(options.warmupSeconds??0)||(options.warmupSeconds??0)<0||(options.warmupSeconds??0)>60)throw new Error('Coordinated warmup must use integer seconds');
 for(const concurrency of options.concurrency)validateStage({workers,concurrency,seconds:options.seconds,timeoutMs:options.timeoutMs});
 for(const arrivalRate of options.rates)validateStage({workers,concurrency:options.concurrency.at(-1),seconds:options.seconds,timeoutMs:options.timeoutMs,arrivalRate});
 const observedIsolation=await (isolationProbe??verifyFixtureIsolation)({base:verified.base,principals:handoff.principals,clients:verified.clientModel.clients});
 const isolated=observedIsolation?.verified===true&&observedIsolation.status==='verified'&&observedIsolation.requests===2&&observedIsolation.directions===2;
 const singlePrincipal=handoff.principals.length===1&&observedIsolation?.verified===false&&observedIsolation.status==='insufficient-principals'&&observedIsolation.requests===0;
 if(!isolated&&!singlePrincipal)throw new Error('Fixture account isolation was not observed');
 const isolationEvidence=isolated?{verified:true,status:'verified',requests:2,directions:2}:{verified:false,status:'insufficient-principals',requests:0};
 const directory=await mkdtemp(join(tmpdir(),'mw-multi-http-')),files=[],children=[];let stageId=0;
 const cpuStarted=process.cpuUsage(),controllerStarted=performance.now(),rssBeforeBytes=process.memoryUsage().rss;
 try{
  for(const part of verified.partitions){
   const path=join(directory,`worker-${part.index}.private.json`);files.push(path);
   await writeFile(path,JSON.stringify({schema:1,workerIndex:part.index,engineMode:'native',base:verified.base,principals:part.principals,clientSimulation:{mode:'fixture-loopback-trusted-proxy',clientCount:part.count},representative:verified.workloadProfile?.kind==='representative',workloadProfile:verified.workloadProfile}),{mode:0o600,flag:'wx'});
   // Inherit only OS runtime essentials, never server/DSN/token environment.
   const env=Object.fromEntries(['SystemRoot','WINDIR','TEMP','TMP','TZ'].filter(key=>process.env[key]!==undefined).map(key=>[key,process.env[key]]));
   const child=launch(join(dirname(fileURLToPath(import.meta.url)),'http-multi-worker.mjs'),[path],{env,stdio:['ignore','ignore','ignore','ipc'],execArgv:[]});children.push(childChannel(child,part.index));
  }
  const ready=await Promise.all(children.map(child=>child.wait('ready',undefined,120000)));
  if(ready.some((value,index)=>value.schema!==1||value.workerIndex!==index||value.partitionCount!==verified.partitions[index].count))throw new Error('Worker identity allocation mismatch');
  const diagnostics=async phase=>{
   if(!handoff.diagnostics)return null;
   const path=handoff.diagnostics.path;if(typeof path!=='string'||!/^\/__fixture-diagnostic-[a-f0-9-]+$/.test(path))throw new Error('Invalid diagnostic control path');
   const init={headers:{'x-fixture-token':handoff.diagnostics.token},redirect:'error',signal:AbortSignal.timeout(5000)};
   if(phase==='before'){const reset=await fetch(new URL(path,verified.base),{...init,method:'POST'});if(!reset.ok)throw new Error('Diagnostic reset failed');await reset.body?.cancel();}
   const response=await fetch(new URL(path,verified.base),init);if(!response.ok)throw new Error('Diagnostic snapshot failed');return fixtureDiagnosticsReport(await response.json());
  };
  const run=async requestedStage=>{
   // runCampaign warms at one outstanding request. A partitioned warmup needs
   // one slot per worker; measured ramp budgets were validated unchanged above.
   const stage=requestedStage.concurrency<workers&&requestedStage.arrivalRate===undefined
    ?{...requestedStage,concurrency:workers}:requestedStage;
   const endpoint=stage.requests[0].name,budget=validateStage({...stage,workers}),id=stageId++;
   for(const child of children)child.send({type:'prepare',stageId:id,endpoint,concurrency:budget.concurrency[child.index],seconds:stage.seconds,timeoutMs:stage.timeoutMs,...(stage.arrivalRate===undefined?{}:{arrivalRate:budget.rates[child.index]})});
   const acknowledgments=await Promise.all(children.map(child=>child.wait('stage-ready',id,10000)));
   if(acknowledgments.some((ack,index)=>ack.workerIndex!==index))throw new Error('Worker readiness mismatch');
   const startAtMs=Date.now()+startLeadMs;
   for(const child of children)child.send({type:'start',stageId:id,startAtMs});
   const results=await Promise.all(children.map(child=>child.wait('result',id,startLeadMs+(stage.seconds+stage.timeoutMs/1000+10)*1000)));
   const observedAtMs=Date.now();
   if(results.some(row=>row.measurement?.window?.completedAtMs>observedAtMs+maxStartLagMs))throw new Error('Worker completion is ahead of the controller clock');
   return aggregateWorkers({results,partitions:verified.partitions,stage,startAtMs,maxStartLagMs});
  };
  const names=['mixed','identity','mailboxes','query','headers','refresh',...(verified.workloadProfile?.kind==='representative'?['representative']:[])],groups=Object.fromEntries(names.map(name=>[name,[{name}]]));
  const campaign=await runCampaign({base:verified.base,headers:{},groups,options,run,diagnostics});
  // Preserve the single native driver's actual enforcement check. This
  // anonymous probe is outside measured phases and cannot add successful RPS.
  const scannerProbeStatus=await (scannerProbe??(async()=>{
   const response=await fetch(new URL('/.env',verified.base),{redirect:'error',signal:AbortSignal.timeout(5000)});
   await response.body?.cancel();return response.status;
  }))();
  if(![403,429].includes(scannerProbeStatus))throw new Error('Native scanner enforcement was not observed');
  const elapsedSeconds=(performance.now()-controllerStarted)/1000,cpu=process.cpuUsage(cpuStarted);
  const policyQualified=handoff.buildProfile==='release'&&verified.policyEvidence.verified&&verified.startupReadiness.verified&&isolationEvidence.verified;
  const storageQualified=qualifiesSearchStorage(verified.searchStorage);
  const telemetryQualified=campaign.stages.length>0&&campaign.stages.every(stage=>qualifiesHttpTelemetry(stage.diagnostics));
  const qualified=policyQualified&&storageQualified;
  const met=qualifiesHttpTarget(campaign,qualified);
  return {schema:1,kind:'coordinated-native-http-fixture',generatedAt:new Date().toISOString(),target:'ephemeral loopback Mailwoman router + seeded native engine',scenario:'real authenticated middleware/HTTP/native-engine with '+handoff.backend+'; fixture data; no TLS/reverse proxy',runtime:process.version,model:'bounded coordinated processes with disjoint users and a shared measurement window; no capacity claim',protection:verified.policyEvidence.verified?'administrator defaults plus verified locked access.protocols policy':'administrator defaults; loaded policy not verified',uniqueAuthenticatedPrincipals:handoff.principals.length,principalCountMeaning:'configured distinct seeded principals; actual successful HTTP coverage is reported per stage',backend:handoff.backend,buildProfile:handoff.buildProfile,workers,configuredGlobalPrincipals:handoff.principals.length,scannerProbeStatus,isolationEvidence,partitionSizes:verified.partitions.map(part=>part.count),startupReadiness:verified.startupReadiness,searchWriter:verified.searchWriter,searchStorage:verified.searchStorage,linkedLoginPolicyProbe:verified.linkedLoginPolicyProbe,policyEvidence:verified.policyEvidence,workloadProfile:verified.workloadProfile,...campaign,controller:{cpuUserMs:cpu.user/1000,cpuSystemMs:cpu.system/1000,elapsedSeconds,rssBeforeBytes,rssAfterBytes:process.memoryUsage().rss},capacityTarget:{successfulRps:40000,simultaneousAuthenticatedUsers:10000,httpMultiUserTargetMet:met,productionPolicyQualified:policyQualified,productionStorageQualified:storageQualified,productionTelemetryQualified:telemetryQualified,productionIndexPublicationQualified:campaign.stages.length>0&&campaign.stages.every(stage=>qualifiesIndexPublication(stage.diagnostics)),persistentWebSocketSessionsMeasured:false,met:false},distinctPhysicalGeneratorHostsMeasured:false};
 }finally{
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
 if(Object.keys(args).some(key=>!['handoff','output','workers'].includes(key))||!args.handoff||!args.output)throw new Error('Explicit private handoff and output required');
 await mkdir(dirname(resolve(args.output)),{recursive:true});
 const workers=Number(args.workers??4),options=campaignOptions({...process.env,MW_LOAD_FIXTURE_CONCURRENCY:process.env.MW_LOAD_FIXTURE_CONCURRENCY??String(workers)});
 try{
  const report=await runMultiCampaign({handoff:JSON.parse(await readPrivate(args.handoff,HANDOFF_LIMIT)),workers,options});
  await writeMultiArtifacts(args.output,report);
  if(report.mutationStateTainted||!report.stages.some(stage=>stage.accepted>0))process.exitCode=1;
 }catch{
  const failure={schema:1,kind:'coordinated-native-http-fixture',target:'coordinated native fixture (unverified)',scenario:'generator coordination failed',runtime:process.version,model:'no successful capacity qualification',protection:'not verified',uniqueAuthenticatedPrincipals:null,stages:[],coordinationValid:false,capacityTarget:{httpMultiUserTargetMet:false,met:false},reason:'Worker coordination or measurement failed; no capacity qualification.'};
  await writeMultiArtifacts(args.output,failure);
  throw new Error('Coordinated generator failed');
 }
}
if(process.argv[1]&&import.meta.url===pathToFileURL(resolve(process.argv[1])).href)main().catch(()=>{console.error('Coordinated generator failed; no credentials logged.');process.exitCode=1;});
