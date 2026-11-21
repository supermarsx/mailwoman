// Scalar-only experiment receipts; no new production configuration defaults.
const configKeys=['enabled','budgetMiB','maxEntries','maxAccounts','idleSeconds'];
export function preparedMailOptions(value){
 if(!value||typeof value!=='object'||Array.isArray(value)||Object.keys(value).length!==5||Object.keys(value).some(k=>!configKeys.includes(k))||typeof value.enabled!=='boolean')throw Error('Invalid prepared mail fixture options');
 for(const [k,min,max]of [['budgetMiB',16,16384],['maxEntries',1,100000],['maxAccounts',1,10000],['idleSeconds',1,3600]])if(!Number.isSafeInteger(value[k])||value[k]<min||value[k]>max)throw Error('Invalid prepared mail fixture bounds');
 return Object.fromEntries(configKeys.map(k=>[k,value[k]]));
}
export function preparedMailExperimentReport(value){
 if(value==null)return null;
 if(!value||typeof value.enabled!=='boolean')throw Error('Invalid prepared mail experiment');
 const resolvedPolicy=preparedMailOptions(value.resolvedPolicy),requested=value.requested===null?null:preparedMailOptions(value.requested);
 if(value.enabled!==Boolean(requested)||value.enabled&&(!requested.enabled||JSON.stringify(requested)!==JSON.stringify(resolvedPolicy))||!value.enabled&&resolvedPolicy.enabled)throw Error('Prepared mail policy not resolved as requested');
 const startup=value.startup??null;
 if(!value.enabled&&startup!==null)throw Error('Disabled prepared mail never has startup priming proof');
 if(value.enabled&&(!startup||startup.verified!==true||!Number.isSafeInteger(startup.attempts)||startup.attempts<1||startup.attempts>40||!Number.isSafeInteger(startup.elapsedMs)||startup.elapsedMs<0||startup.elapsedMs>15000||startup.primedPrincipals!==1||startup.measuredRequests!==0))throw Error('Missing actual prepared mail startup proof');
 return {enabled:value.enabled,requested,resolvedPolicy,startup:startup?{verified:true,attempts:startup.attempts,elapsedMs:startup.elapsedMs,primedPrincipals:1,measuredRequests:0}:null,scope:'explicit fixture policy experiment; not production default or working-set-fit proof'};
}
const nullable=['entries','accounts','pendingBuilds','invalidatedEntries','budgetBytes','maxEntries','maxAccounts','idleSeconds'];
const counters=['chargedBytes','admittedJobs','activePreparers','eligibleCandidates','readyCandidateHits','candidateMisses','completedBuilds','evictions','seedCalls','seededRows','rawResults','compatibilityValueResults','handledInvalidationHints','invalidationErrors','invalidationPollCalls','invalidationAckCalls'];
export function preparedMailStatusReport(v){
 if(v==null)return null;
 if(!v||!(v.enabled===null||typeof v.enabled==='boolean'))throw Error('Invalid prepared mail status');
 for(const k of nullable)if(!(v[k]===null||Number.isSafeInteger(v[k])&&v[k]>=0))throw Error('Invalid prepared mail nullable counter');
 for(const k of counters)if(!Number.isSafeInteger(v[k])||v[k]<0)throw Error('Invalid prepared mail counter');
 return {enabled:v.enabled,...Object.fromEntries([...nullable,...counters].map(k=>[k,v[k]]))};
}
export function preparedMailEvidence(value,stages){
 const experiment=preparedMailExperimentReport(value);if(!experiment)return null;
 const config=experiment.resolvedPolicy,observations=[];let rawResults=0,readyHits=0,eligiblePhases=0;
 for(const stage of stages){const before=preparedMailStatusReport(stage.diagnostics?.before?.preparedMail),after=preparedMailStatusReport(stage.diagnostics?.after?.preparedMail);
  if(experiment.enabled){for(const s of [before,after])if(!s||s.enabled!==true||s.budgetBytes!==config.budgetMiB*1048576||s.maxEntries!==config.maxEntries||s.maxAccounts!==config.maxAccounts||s.idleSeconds!==config.idleSeconds)throw Error('Missing actual applied prepared mail configuration');
   for(const k of counters.slice(3))if(after[k]<before[k])throw Error('Prepared mail counter reset during stage');
  }
  const rawDelta=before&&after?after.rawResults-before.rawResults:null,hitDelta=before&&after?after.readyCandidateHits-before.readyCandidateHits:null;
  if(rawDelta!==null&&rawDelta>=0)rawResults+=rawDelta;if(hitDelta!==null&&hitDelta>=0)readyHits+=hitDelta;
  const eligibleShape=['headers','mixed','representative'].includes(stage.endpoint);
  const eligibleAttempts=eligibleShape?stage.byRequest?.['Email/get']?.requests:0;
  if(experiment.enabled&&(!Number.isSafeInteger(eligibleAttempts)||eligibleAttempts<0))throw Error('Missing marked eligible request attempt proof');
  if(experiment.enabled&&eligibleAttempts>0){eligiblePhases++;if(!(rawDelta>0))throw Error('Marked eligible phase without observed raw result emission');}
  observations.push({endpoint:stage.endpoint,phase:stage.phase,before,after,eligibleRequestAttempts:eligibleAttempts??null,rawResultObserved:rawDelta>0,rawResultsDelta:rawDelta,readyCandidateHitsDelta:hitDelta});
 }
 if(experiment.enabled&&!observations.length)throw Error('Missing prepared mail phase observations');
 return {...experiment,observations,eligiblePhases,qualified:experiment.enabled&&eligiblePhases>0,rawResultsDelta:rawResults,readyCandidateHitsDelta:readyHits,rawResultObserved:rawResults>0,guardInputConsumption:'not measured by these counters',window:'before/after includes drain; not in-window operation count'};
}
