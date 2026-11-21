// Additive proof for the explicit future enforced profile. Legacy evidence is unknown.
export function parsedProtectionPolicy(value){
 if(value?.kind!=='native-protection-policy-v1'||value.noExemptions!==true||value.defaultsExceptModes!==true)throw Error('Missing nonexempt parsed policy');
 const p=value.parsed;
 if(!p||['rateMode','adaptiveMode','probePathMode'].some(k=>p[k]!=='enforce')||p.storageFailure!=='closed')throw Error('Protection must enforce and fail closed');
 const fields=['clientRequests','accountRequests','adminRequests','rateWindowSecs','normalCapPerBucket','signals','throttleSecs','blockSecs','decaySecs'];
 const defaults={clientRequests:600,accountRequests:1800,adminRequests:1200,rateWindowSecs:60,normalCapPerBucket:1000,signals:7,throttleSecs:3,blockSecs:300,decaySecs:86400};
 if(fields.some(k=>p[k]!==defaults[k]))throw Error('Invalid or changed native policy budget');
 const flags=['adaptiveClients','adaptiveAccounts','adaptiveAdmins'];
 if(flags.some(k=>p[k]!==true))throw Error('Adaptive subjects must remain enabled');
 const window=name=>{const w=p[name],keys=['seconds','minimum_samples','absolute_anomalies','ratio_percent'];const expected=name==='short'?[300,30,20,50]:[604800,100,100,25];if(!w||keys.some((k,i)=>w[k]!==expected[i]))throw Error('Invalid adaptive window');return Object.fromEntries(keys.map(k=>[k,w[k]]));};
 const u=p.userAgent,keys=['weakRequests','crawlerRequests','windowSecs'];
 if(!u||u.mode!=='enforce'||u.scope!=='anonymous'||u.blockBots!==true||u.adaptive!==true||u.claimedOnly!==true||u.maxAction!=='throttle'||keys.some((k,i)=>u[k]!==[600,300,60][i])||!Array.isArray(u.categories)||u.categories.length!==7||new Set(u.categories).size!==7||u.categories.some(x=>!['missing','uncommon','automation','search','ai-training','ai-search','ai-user'].includes(x)))throw Error('Invalid native user-agent policy');
 return {kind:value.kind,noExemptions:true,defaultsExceptModes:true,parsed:{rateMode:'enforce',adaptiveMode:'enforce',probePathMode:'enforce',storageFailure:'closed',...Object.fromEntries(fields.map(k=>[k,p[k]])),...Object.fromEntries(flags.map(k=>[k,true])),short:window('short'),long:window('long'),userAgent:{mode:'enforce',scope:'anonymous',blockBots:true,adaptive:true,claimedOnly:true,maxAction:'throttle',...Object.fromEntries(keys.map(k=>[k,u[k]])),categories:[...u.categories]}}};
}
export function enforcedPolicyReport(value){
 if(value==null)return null;
 const expected={trustedExternalHealth:404,untrustedForwardedHealth:200,automation:429,scanner:429,scannerFollowup:429,rateAllowed:600,rateRefused:429};
 if(value.kind!=='native-enforced-policy-v1'||value.outsideMeasurement!==true||value.physicalSourceDiversityProven!==false||value.adaptiveControl!=='separate-real-router-test-required'||Object.entries(expected).some(([k,v])=>value.controls?.[k]!==v))throw Error('Invalid enforced-profile controls');
 return {kind:value.kind,policy:parsedProtectionPolicy(value.policy),controls:expected,adaptiveControl:'separate-real-router-test-required',outsideMeasurement:true,physicalSourceDiversityProven:false};
}
export function assertEnforcedPolicySnapshot(snapshot,experiment){
 if(experiment&&JSON.stringify(parsedProtectionPolicy(snapshot?.enforcedPolicySnapshot))!==JSON.stringify(experiment.policy))throw Error('Fresh measured-phase protection policy differs or is missing');
}
