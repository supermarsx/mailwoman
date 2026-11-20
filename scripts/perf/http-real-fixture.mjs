// Invoked only by the ignored Rust integration fixture. No arbitrary remote URL.
import {writeFile,mkdir} from 'node:fs/promises';
import {dirname} from 'node:path';
import {markdownReport} from './http-load.mjs';
import {campaignOptions,runCampaign} from './http-campaign.mjs';
import {prepareFixtureClients,fixturePrincipalIndex,startupReadinessReport,fixturePolicyReport} from './fixture-clients.mjs';
import {representativeSpecs,workloadProfileReport} from './representative-workload.mjs';
import {readPrivate,HANDOFF_LIMIT} from './external-control.mjs';

try{
 const handoff=JSON.parse((await readPrivate(process.argv[2],HANDOFF_LIMIT)).toString('utf8')),base=new URL(handoff.base);
 const startupReadiness=startupReadinessReport(handoff.startupReadiness,{required:handoff.engineMode==='native'});
 const policyEvidence=fixturePolicyReport(handoff.policyEvidence);
 if(base.protocol!=='http:'||base.hostname!=='127.0.0.1'||base.pathname!=='/'||base.username||base.password)throw new Error('Fixture must use literal ephemeral loopback');
 const principals=handoff.principals??[{cookie:handoff.cookie}];
 const clientModel=prepareFixtureClients(principals,handoff.clientSimulation);
 const first=principals[0],headers=clientModel.clients[0].headers;
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
 const request=(p,name,args)=>({name,path:'/jmap/api',method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({using:['urn:ietf:params:jmap:core','urn:ietf:params:jmap:mail'],methodCalls:[[name,{accountId:p.accountId,...args},'load']]}),validate:value=>value.methodResponses?.[0]?.[0]===name&&value.methodResponses?.[0]?.[1]?.accountId===p.accountId});
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
 const specs=p=>{
  const ids=p.messageIds.slice(0,2),emailGet=request(p,'Email/get',{ids,properties:['id','subject','from','receivedAt','preview']}),valid=emailGet.validate;
  emailGet.validate=value=>valid(value)&&Array.isArray(value.methodResponses[0][1].list)&&ids.every(id=>value.methodResponses[0][1].list.some(row=>row.id===id));
  const baseline=[{name:'authenticated-identity',path:'/api/me',validate:v=>v.accountId===(p.ownerAccountId??p.accountId)},request(p,'Mailbox/get',{}),request(p,'Email/query',{limit:25}),emailGet,{name:'concurrent-refresh-handler',path:'/api/mail/refresh',method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({accountId:p.ownerAccountId??p.accountId}),validate:v=>v.accountId===(p.ownerAccountId??p.accountId)&&v.upstream===native}];
  return representative?[...baseline,...representativeSpecs(p)].map(spec=>({...spec,serializePrincipal:true})):baseline;
 };
 // Prepare bounded request templates outside measured phases, avoiding five
 // JSON serializations per dispatch when exercising 10,000 principals.
 const prepared=principals.map((p,index)=>{const headers=clientModel.clients[index].headers;return specs(p).map(spec=>({...spec,headers:{...headers,...spec.headers}}));});
 const requests=prepared[0].map((spec,type)=>({...spec,materialize:(sequence,{requestCount})=>{const principalIndex=fixturePrincipalIndex(sequence,requestCount,principals.length);return {...prepared[principalIndex][type],principalIndex,clientIndex:clientModel.clients[principalIndex].clientIndex};}}));
 // Every endpoint must work before saturation. Seeded native fixtures additionally
 // prove their first/last account bindings in their Rust setup.
 for(const spec of specs(first)){
  const response=await fetch(new URL(spec.path,base),{method:spec.method??'GET',headers:{...headers,...spec.headers},...(spec.body!==undefined?{body:spec.body}:{}),redirect:'error',signal:AbortSignal.timeout(10000)});
  if(response.status!==200){await response.body?.cancel();throw new Error(`Fixture ${spec.name} preflight refused (${response.status})`);}
  if(spec.validateBytes){if(!spec.validateBytes(Buffer.from(await response.arrayBuffer())))throw new Error(`Fixture ${spec.name} preflight failed content validation`);}
  else {const value=await response.json();if(spec.validate&&!spec.validate(value))throw new Error(`Fixture ${spec.name} preflight failed application validation`);}
 }
 const options=campaignOptions();
 const groups={mixed:requests.slice(0,5),identity:[requests[0]],mailboxes:[requests[1]],query:[requests[2]],headers:[requests[3]],refresh:[requests[4]],...(representative?{representative:requests}:{})};
 const diagnostics=async(phase)=>{
  if(!handoff.diagnostics)return null;
  const url=new URL(handoff.diagnostics.path,base),init={headers:{'x-fixture-token':handoff.diagnostics.token},redirect:'error',signal:AbortSignal.timeout(5000)};
  if(phase==='before'){const reset=await fetch(url,{...init,method:'POST'});if(!reset.ok)throw new Error('Private fixture diagnostic reset unavailable');await reset.body?.cancel();}
  const response=await fetch(url,init);
  if(!response.ok)throw new Error('Private fixture diagnostics unavailable');return response.json();
 };
 const report={generatedAt:new Date().toISOString(),target:native?'ephemeral loopback Mailwoman router + seeded native engine':'ephemeral loopback Mailwoman router + local JMAP mock',scenario:`real authenticated middleware/HTTP/${native?'native-engine':'proxy'} with ${handoff.backend??'sqlite'}; fixture data; no TLS/reverse proxy`,backend:handoff.backend??'sqlite',buildProfile:handoff.buildProfile??'unknown',uniqueAuthenticatedPrincipals:principals.length,engineMode:native?'native':'proxy',protection:typeof handoff.protection==='string'?handoff.protection:handoff.protection?'rate/adaptive/user-agent observe; scanner enforce':'rate/adaptive/user-agent/scanner off',refreshSemantics:native?'native fixture backend refresh; upstream:true':'authenticated refresh route only; proxy response upstream:false; no upstream reconciliation',effectiveFixtureBudgets:handoff.budgets??{},runtimeWorkerThreads:handoff.runtimeWorkerThreads??4,runtimeWorkerStackBytes:handoff.runtimeWorkerStackBytes??null,generatorPlacement:handoff.generatorPlacement??'child process in same resource group',runtime:process.version,model:options.mode==='smoke'?'bounded open-loop overload smoke':'warmed concurrency ramp, followed by rates bounded by observed successful throughput; no capacity claim',stages:[]};
 Object.assign(report,{startupReadiness,policyEvidence,workloadProfile},await runCampaign({base,headers,groups,options,diagnostics}));
 report.clientModel={workload:clientModel.workload,simulationMode:clientModel.simulationMode,configuredPrincipals:principals.length,configuredSimulatedClients:clientModel.clientCount,distinctPhysicalGeneratorHostsMeasured:false,actualPrincipalCoverage:Math.max(0,...report.stages.map(s=>s.uniquePrincipalsAccepted??0)),actualSimulatedClientCoverage:Math.max(0,...report.stages.map(s=>s.uniqueClientsAccepted??0))};
 report.capacityTarget={successfulRps:40000,simultaneousAuthenticatedUsers:10000,verifiedUniquePrincipals:Math.max(0,...report.stages.map(s=>s.uniquePrincipalsAccepted??0)),httpRateMet:report.stages.some(s=>s.phase==='offered-rate'&&s.arrivalRate>=40000&&s.acceptedDuringScheduledWindowRps>=40000&&s.errors===0&&s.throttled===0&&s.timeouts===0&&(s.droppedCapacity+s.droppedScheduler)===0&&s.scheduledSeconds>=30)};
 report.capacityTarget.httpMultiUserTargetMet=report.stages.some(s=>['mixed','representative'].includes(s.endpoint)&&s.phase==='offered-rate'&&s.arrivalRate>=40000&&s.acceptedDuringScheduledWindowRps>=40000&&s.uniquePrincipalsAcceptedDuringScheduledWindow>=10000&&s.uniqueClientsAcceptedDuringScheduledWindow>=10000&&s.errors===0&&s.throttled===0&&s.timeouts===0&&(s.droppedCapacity+s.droppedScheduler)===0&&s.scheduledSeconds>=30);
 report.capacityTarget.productionPolicyQualified=native&&handoff.buildProfile==='release'&&startupReadiness?.verified===true&&policyEvidence.verified;
 report.capacityTarget.httpMultiUserTargetMet=report.capacityTarget.httpMultiUserTargetMet&&report.capacityTarget.productionPolicyQualified&&!report.mutationStateTainted;
 report.capacityTarget.httpRateMet=report.capacityTarget.httpRateMet&&!report.mutationStateTainted;
 report.capacityTarget.representativeWorkloadMeasured=report.stages.some(s=>s.endpoint==='representative');
 report.capacityTarget.persistentWebSocketSessionsMeasured=false;report.capacityTarget.met=false;
 const probe=await fetch(new URL('/.env',base),{redirect:'error',signal:AbortSignal.timeout(5000)});
 report.scannerProbeStatus=probe.status;await probe.body?.cancel();
 if((native||handoff.protection===true)&&![403,429].includes(probe.status))throw new Error(`Scanner enforcement unexpectedly returned ${probe.status}`);
 const output=process.argv[3];await mkdir(dirname(output),{recursive:true});
 await writeFile(output,JSON.stringify(report,null,2)+'\n',{mode:0o600});await writeFile(output.replace(/\.json$/,'')+'.md',markdownReport(report),{mode:0o600});
 if(report.mutationStateTainted)throw new Error('Campaign stopped after an unknown mutation outcome; inspect the redacted failure report');
 if(!report.stages.some(stage=>stage.accepted>0))throw new Error('No authenticated request succeeded');
}catch(error){console.error(error.message);process.exitCode=1;}
