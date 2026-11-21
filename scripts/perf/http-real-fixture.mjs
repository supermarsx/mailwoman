import {preparedMailExperimentReport,preparedMailEvidence} from './prepared-mail-experiment.mjs';
import {throwDiagnosticSnapshotFailure,cacheAuditErrorEvidence,diagnosticFailureReport} from './fixture-diagnostic-failure.mjs';
import {campaignCacheExperiment,assertSqliteCacheWitness,cacheExperimentEvidence} from './sqlite-page-cache-witness.mjs';
import {qualifiesIndexPublication} from './index-publication-health.mjs';
// Invoked only by the ignored Rust integration fixture. No arbitrary remote URL.
import {writeFile,mkdir} from 'node:fs/promises';
import {dirname} from 'node:path';
import {markdownReport} from './http-load.mjs';
import {campaignOptions,runCampaign} from './http-campaign.mjs';
import {qualifiesHttpTelemetry,fixtureDiagnosticsReport,prepareFixtureClients,startupReadinessReport,fixturePolicyReport,searchWriterReport,linkedLoginPolicyProbeReport,searchStorageReport,qualifiesSearchStorage} from './fixture-clients.mjs';
import {workloadProfileReport} from './representative-workload.mjs';
import {jmapFixtureRequest as request,prepareFixtureRequests} from './fixture-requests.mjs';
import {readPrivate,HANDOFF_LIMIT} from './external-control.mjs';
import {verifyFixtureIsolation} from './fixture-isolation.mjs';

try{
 const handoff=JSON.parse((await readPrivate(process.argv[2],HANDOFF_LIMIT)).toString('utf8')),base=new URL(handoff.base);
 const options=campaignOptions();
 const preparedMailExperiment=preparedMailExperimentReport(handoff.preparedMailExperiment);
 const cacheExperiment=campaignCacheExperiment(handoff,options);
 const startupReadiness=startupReadinessReport(handoff.startupReadiness,{required:handoff.engineMode==='native'});
 const policyEvidence=fixturePolicyReport(handoff.policyEvidence);
 const searchWriter=searchWriterReport(handoff.searchWriter);
 const searchStorage=searchStorageReport(handoff.searchStorage);
 const linkedLoginPolicyProbe=linkedLoginPolicyProbeReport(handoff.linkedLoginPolicyProbe);
 if(base.protocol!=='http:'||base.hostname!=='127.0.0.1'||base.pathname!=='/'||base.username||base.password)throw new Error('Fixture must use literal ephemeral loopback');
 const principals=handoff.principals??[{cookie:handoff.cookie}];
 const initialClients=prepareFixtureClients(principals,handoff.clientSimulation);
 const first=principals[0],headers=initialClients.clients[0].headers;
 if(!first.accountId){
  const response=await fetch(new URL('/jmap/session',base),{headers,redirect:'error',signal:AbortSignal.timeout(10000)});
  if(!response.ok)throw new Error(`Fixture session refused (${response.status})`);
  first.accountId=(await response.json()).primaryAccounts?.['urn:ietf:params:jmap:mail'];
 }
 if(!first.ownerAccountId){
  const response=await fetch(new URL('/api/me',base),{headers,redirect:'error',signal:AbortSignal.timeout(10000)});
  if(!response.ok)throw new Error(`Fixture identity refused (${response.status})`);
  first.ownerAccountId=(await response.json()).accountId;
 }
 if(!first.messageIds){
  const spec=request(first,'Email/query',{limit:2}),response=await fetch(new URL(spec.path,base),{method:spec.method,headers:{...headers,...spec.headers},body:spec.body,redirect:'error',signal:AbortSignal.timeout(10000)});
  if(!response.ok)throw new Error(`Fixture message discovery refused (${response.status})`);
  const value=await response.json();if(!spec.validate(value))throw new Error('Fixture message discovery failed');first.messageIds=value.methodResponses[0][1].ids;
 }
 if(principals.some(p=>typeof p.accountId!=='string'||!p.accountId||!Array.isArray(p.messageIds)||!p.messageIds.length||p.messageIds.some(id=>typeof id!=='string'||!id)))throw new Error('Fixture requires bounded seeded message IDs per principal');
 if(new Set(principals.map(p=>p.accountId)).size!==principals.length)throw new Error('Principal accounts must be distinct');
 const native=handoff.engineMode==='native';
 const workloadProfile=workloadProfileReport(handoff.workloadProfile,principals.length);
 const representative=workloadProfile?.kind==='representative';
 if(representative&&(!native||principals.some(p=>p.messageIds.length!==workloadProfile.messagesPerPrincipal||p.bodySampleBytes!==workloadProfile.bodySampleBytes||p.attachmentSampleBytes!==workloadProfile.attachmentSampleBytes)))throw new Error('Invalid representative workload profile');
 // Prepare bounded request templates outside measured phases, avoiding five
 // JSON serializations per dispatch when exercising 10,000 principals.
 const {requests,groups,clientModel,firstRequests,readIntent}=prepareFixtureRequests({principals,clientSimulation:handoff.clientSimulation,engineMode:native?'native':'proxy',representative,workloadProfile});
 const isolationEvidence=native?await verifyFixtureIsolation({base,principals,clients:clientModel.clients}):{verified:false,status:'not-measured',requests:0};
 // Every endpoint must work before saturation. Seeded native fixtures additionally
 // prove their first/last account bindings in their Rust setup.
 for(const spec of firstRequests){
  const response=await fetch(new URL(spec.path,base),{method:spec.method??'GET',headers:{...headers,...spec.headers},...(spec.body!==undefined?{body:spec.body}:{}),redirect:'error',signal:AbortSignal.timeout(10000)});
  if(response.status!==200){await response.body?.cancel();throw new Error(`Fixture ${spec.name} preflight refused (${response.status})`);}
  if(spec.validateBytes){if(!spec.validateBytes(Buffer.from(await response.arrayBuffer())))throw new Error(`Fixture ${spec.name} preflight failed content validation`);}
  else {const value=await response.json();if(spec.validate&&!spec.validate(value))throw new Error(`Fixture ${spec.name} preflight failed application validation`);}
 }
 const diagnostics=async(phase)=>{
  if(!handoff.diagnostics)return null;
  const url=new URL(handoff.diagnostics.path,base),init={headers:{'x-fixture-token':handoff.diagnostics.token},redirect:'error',signal:AbortSignal.timeout(5000)};
  if(phase==='before'){const reset=await fetch(url,{...init,method:'POST'});if(!reset.ok)throw new Error('Private fixture diagnostic reset unavailable');await reset.body?.cancel();}
  const response=await fetch(url,init);
  if(!response.ok)await throwDiagnosticSnapshotFailure(response);const snapshot=fixtureDiagnosticsReport(await response.json());if(cacheExperiment)assertSqliteCacheWitness(snapshot,cacheExperiment);return snapshot;
 };
 const report={generatedAt:new Date().toISOString(),target:native?'ephemeral loopback Mailwoman router + seeded native engine':'ephemeral loopback Mailwoman router + local JMAP mock',scenario:`real authenticated middleware/HTTP/${native?'native-engine':'proxy'} with ${handoff.backend??'sqlite'}; fixture data; no TLS/reverse proxy`,backend:handoff.backend??'sqlite',buildProfile:handoff.buildProfile??'unknown',uniqueAuthenticatedPrincipals:principals.length,engineMode:native?'native':'proxy',protection:typeof handoff.protection==='string'?handoff.protection:handoff.protection?'rate/adaptive/user-agent observe; scanner enforce':'rate/adaptive/user-agent/scanner off',refreshSemantics:native?'native fixture backend refresh; upstream:true':'authenticated refresh route only; proxy response upstream:false; no upstream reconciliation',effectiveFixtureBudgets:handoff.budgets??{},runtimeWorkerThreads:handoff.runtimeWorkerThreads??4,runtimeWorkerStackBytes:handoff.runtimeWorkerStackBytes??null,generatorPlacement:handoff.generatorPlacement??'child process in same resource group',runtime:process.version,model:options.mode==='smoke'?'bounded open-loop overload smoke':'warmed concurrency ramp, followed by rates bounded by observed successful throughput; no capacity claim',stages:[]};
 Object.assign(report,{readIntent,startupReadiness,policyEvidence,workloadProfile,isolationEvidence,searchWriter,searchStorage,linkedLoginPolicyProbe},await runCampaign({base,headers,groups,options,diagnostics}));
 report.preparedMailExperiment=preparedMailEvidence(preparedMailExperiment,report.stages);
 if(cacheExperiment)report.sqliteCacheExperiment=cacheExperimentEvidence(cacheExperiment,report.stages);
 report.clientModel={workload:clientModel.workload,simulationMode:clientModel.simulationMode,configuredPrincipals:principals.length,configuredSimulatedClients:clientModel.clientCount,distinctPhysicalGeneratorHostsMeasured:false,actualPrincipalCoverage:Math.max(0,...report.stages.map(s=>s.uniquePrincipalsAccepted??0)),actualSimulatedClientCoverage:Math.max(0,...report.stages.map(s=>s.uniqueClientsAccepted??0))};
 report.capacityTarget={successfulRps:40000,simultaneousAuthenticatedUsers:10000,verifiedUniquePrincipals:Math.max(0,...report.stages.map(s=>s.uniquePrincipalsAccepted??0)),httpRateMet:report.stages.some(s=>s.phase==='offered-rate'&&s.arrivalRate>=40000&&s.acceptedDuringScheduledWindowRps>=40000&&s.errors===0&&s.throttled===0&&s.timeouts===0&&(s.droppedCapacity+s.droppedScheduler)===0&&s.scheduledSeconds>=30)};
 report.capacityTarget.httpMultiUserTargetMet=report.stages.some(s=>['mixed','representative'].includes(s.endpoint)&&s.phase==='offered-rate'&&s.arrivalRate>=40000&&s.acceptedDuringScheduledWindowRps>=40000&&s.uniquePrincipalsAcceptedDuringScheduledWindow>=10000&&s.uniqueClientsAcceptedDuringScheduledWindow>=10000&&s.errors===0&&s.throttled===0&&s.timeouts===0&&(s.droppedCapacity+s.droppedScheduler)===0&&s.scheduledSeconds>=30&&qualifiesHttpTelemetry(s.diagnostics)&&qualifiesIndexPublication(s.diagnostics));
 report.capacityTarget.productionPolicyQualified=native&&handoff.buildProfile==='release'&&startupReadiness?.verified===true&&policyEvidence.verified&&isolationEvidence.verified;
 report.capacityTarget.productionStorageQualified=qualifiesSearchStorage(searchStorage);
 report.capacityTarget.productionTelemetryQualified=report.stages.length>0&&report.stages.every(stage=>qualifiesHttpTelemetry(stage.diagnostics));
 report.capacityTarget.httpMultiUserTargetMet=report.capacityTarget.httpMultiUserTargetMet&&report.capacityTarget.productionPolicyQualified&&report.capacityTarget.productionStorageQualified&&!report.mutationStateTainted;
 report.capacityTarget.productionIndexPublicationQualified=report.stages.length>0&&report.stages.every(stage=>qualifiesIndexPublication(stage.diagnostics));
 report.capacityTarget.httpRateMet=report.capacityTarget.httpRateMet&&!report.mutationStateTainted&&report.capacityTarget.productionTelemetryQualified&&report.capacityTarget.productionIndexPublicationQualified;
 report.capacityTarget.representativeWorkloadMeasured=report.stages.some(s=>s.endpoint==='representative');
 report.capacityTarget.persistentWebSocketSessionsMeasured=false;report.capacityTarget.met=false;
 const probe=await fetch(new URL('/.env',base),{redirect:'error',signal:AbortSignal.timeout(5000)});
 report.scannerProbeStatus=probe.status;await probe.body?.cancel();
 if((native||handoff.protection===true)&&![403,429].includes(probe.status))throw new Error(`Scanner enforcement unexpectedly returned ${probe.status}`);
 const output=process.argv[3];await mkdir(dirname(output),{recursive:true});
 await writeFile(output,JSON.stringify(report,null,2)+'\n',{mode:0o600});await writeFile(output.replace(/\.json$/,'')+'.md',markdownReport(report),{mode:0o600});
 if(report.mutationStateTainted)throw new Error('Campaign stopped after an unknown mutation outcome; inspect the redacted failure report');
 if(!report.stages.some(stage=>stage.accepted>0))throw new Error('No authenticated request succeeded');
}catch(error){
 if(cacheAuditErrorEvidence(error)){
  const output=process.argv[3],failure=diagnosticFailureReport(error,'native-http-fixture');
  await mkdir(dirname(output),{recursive:true});
  await writeFile(output,JSON.stringify(failure,null,2)+'\n',{mode:0o600});
  await writeFile(output.replace(/\.json$/,'')+'.md',markdownReport(failure),{mode:0o600});
 }
 console.error(error.message);process.exitCode=1;
}
