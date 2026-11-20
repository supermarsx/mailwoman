// Private ephemeral-fixture identities. Never accept arbitrary forwarded clients
// in the public load CLI, or copy session/address identifiers into reports.
import {isIP} from 'node:net';

export function fixturePolicyReport(value){
 if(value===undefined)return {mode:'unverified',verified:false};
 if(value?.mode==='defaults'&&value.verified===false)return {mode:'defaults',verified:false};
 if(value?.mode!=='loaded'||value.verified!==true||value.key!=='access.protocols'||value.locked!==true||!Array.isArray(value.allowedProtocols)||value.allowedProtocols.length!==1||value.allowedProtocols[0]!=='imap'||value.primedRequests!==2||value.revokedRequests!==2||value.revokedStatus!==403||value.restoredRequests!==2)throw new Error('Invalid loaded fixture policy proof');
 return {mode:'loaded',verified:true,key:'access.protocols',allowedProtocols:['imap'],locked:true,primedRequests:2,revokedRequests:2,revokedStatus:403,restoredRequests:2};
}

// Whitelist measured aggregate readiness metadata; never copy arbitrary private
// handoff fields into public reports, even for a fixture-controlled file.
export function searchWriterReport(value){
 if(value===undefined)return null;
 if(!value||!Number.isSafeInteger(value.indexingWorkers)||value.indexingWorkers<1||value.indexingWorkers>8||!Number.isSafeInteger(value.memoryBudgetBytes)||value.memoryBudgetBytes<15000000||value.memoryBudgetBytes>128000000||Math.floor(value.memoryBudgetBytes/value.indexingWorkers)<15000000||!(value.requestedWorkers===null||Number.isSafeInteger(value.requestedWorkers)&&value.requestedWorkers===value.indexingWorkers))throw new Error('Invalid bounded search writer configuration');
 return {requestedWorkers:value.requestedWorkers,indexingWorkers:value.indexingWorkers,memoryBudgetBytes:value.memoryBudgetBytes};
}

export function startupReadinessReport(value,{required=false}={}){
 if(value===undefined){if(required)throw new Error('Native fixture startup readiness proof is required');return null;}
 const fields=['startupSeconds','indexWaitSeconds','seededMessages','expectedIndexDocuments','indexDocuments','zeroAccessExcludedMessages','rebuildDone','rebuildTotal'];
 if(!value||value.verified!==true||!['ready','policy-excluded'].includes(value.state)||value.rebuildRunning!==false||fields.some(field=>!Number.isFinite(value[field])||value[field]<0))throw new Error('Invalid fixture startup readiness proof');
 for(const field of fields.slice(2))if(!Number.isSafeInteger(value[field])||value[field]>500000)throw new Error('Invalid fixture startup readiness counts');
 if(value.indexDocuments!==value.expectedIndexDocuments||value.expectedIndexDocuments+value.zeroAccessExcludedMessages!==value.seededMessages||value.state==='policy-excluded'&&value.expectedIndexDocuments!==0)throw new Error('Incomplete fixture startup readiness proof');
 return Object.fromEntries([['verified',true],['state',value.state],['rebuildRunning',false],...fields.map(field=>[field,value[field]])]);
}

// Keep endpoint labels interleaved while spreading early arrivals over users.
// Every endpoint visits every principal over a complete bounded cycle; short
// overload phases must not concentrate all accepted traffic on the first users.
export function fixturePrincipalIndex(sequence,requestCount,principalCount){
 if(!Number.isSafeInteger(sequence)||sequence<0||!Number.isInteger(requestCount)||requestCount<1||requestCount>100||!Number.isInteger(principalCount)||principalCount<1||principalCount>10000)throw new Error('Invalid fixture rotation bounds');
 return (Math.floor(sequence/requestCount)+(sequence%requestCount)*Math.ceil(principalCount/requestCount))%principalCount;
}

export function prepareFixtureClients(principals,simulation){
 if(!Array.isArray(principals)||principals.length<1||principals.length>10000)throw new Error('Invalid bounded principal handoff');
 const cookies=new Set(),sessions=new Set(),accounts=new Set();
 for(const p of principals){
  if(!p||typeof p.cookie!=='string'||!p.cookie||p.cookie.length>8192||/[\x00-\x1f\x7f]/.test(p.cookie)||cookies.has(p.cookie))throw new Error('Invalid or duplicate fixture credentials');
  cookies.add(p.cookie);
  const tokens=p.cookie.split(';').map(c=>c.trim()).filter(c=>c.startsWith('mw_session='));
  if(principals.length>1&&(tokens.length!==1||tokens[0]==='mw_session='||sessions.has(tokens[0])))throw new Error('Distinct fixture sessions are required');
  if(tokens.length===1)sessions.add(tokens[0]);
  if(principals.length>1&&(typeof p.accountId!=='string'||!p.accountId||accounts.has(p.accountId)))throw new Error('Distinct fixture accounts are required');
  accounts.add(p.accountId);
 }
 const simulated=simulation!==undefined;
 if(simulated&&(!simulation||simulation.mode!=='fixture-loopback-trusted-proxy'||!Number.isInteger(simulation.clientCount)||simulation.clientCount!==principals.length))throw new Error('Invalid fixture client simulation');
 if(!simulated&&principals.some(p=>p.clientAddress!==undefined))throw new Error('Forwarded fixture clients require explicit proxy simulation');
 const addresses=new Set();
 const clients=principals.map((p,index)=>{
  const headers={cookie:p.cookie},csrf=p.csrfToken??p.cookie.split(';').map(c=>c.trim()).find(c=>c.startsWith('mw_csrf='))?.slice(8);
  if(csrf){try{if(typeof csrf!=='string'||csrf.length>8192)throw new Error();const decoded=decodeURIComponent(csrf);if(/[\x00-\x1f\x7f]/.test(decoded))throw new Error();headers['x-csrf-token']=decoded;}catch{throw new Error('Invalid fixture CSRF token');}}
  if(simulated){
   const address=p.clientAddress;
   if(typeof address!=='string'||isIP(address)!==4||!/^198\.(18|19)\./.test(address)||addresses.has(address))throw new Error('Distinct benchmark-range fixture clients are required');
   addresses.add(address);headers['x-forwarded-for']=address;
  }
  return {headers,principalIndex:index,...(simulated?{clientIndex:index}:{})};
 });
 return {clients,clientCount:addresses.size,simulationMode:simulated?simulation.mode:'single-loopback-peer',workload:principals.length===1?'single-user-diagnostic':'multi-user-fixture'};
}
