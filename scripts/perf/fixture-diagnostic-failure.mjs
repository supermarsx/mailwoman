// Fixed fixture-only failure evidence. Never accept arbitrary server error
// strings, identities, headers, SQL, paths or private handoff/environment data.
const CODES=new Set(['cache-audit-unavailable','cache-audit-quiescence-timeout']);
const MAX_BODY_BYTES=32768;
const invalid=()=>{throw new Error('Invalid fixture cache-audit failure evidence');};
const number=(value,min,max)=>{if(!Number.isSafeInteger(value)||value<min||value>max)invalid();return value;};
const boolean=value=>{if(typeof value!=='boolean')invalid();return value;};
export function cacheAuditFailureReport(value){
 if(!value||!CODES.has(value.error))invalid();
 const wait=value.sqlitePageCacheAuditWait;
 if(!wait||wait.budgetMs!==1000||wait.retryIntervalMs!==10||wait.deadlineIsCooperative!==true||!Array.isArray(wait.observations))invalid();
 const attempts=number(wait.attempts,0,100),elapsedMs=number(wait.elapsedMs,0,Number.MAX_SAFE_INTEGER);
 if(wait.observations.length>attempts||attempts-wait.observations.length>1)invalid();
 const observations=wait.observations.map((row,index)=>{
  if(!row||row.attempt!==index+1)invalid();
  const openBefore=number(row.openBefore,0,32),openAfter=number(row.openAfter,0,32);
  const sampledConnections=number(row.sampledConnections,0,32),failedConnections=number(row.failedConnections,0,32);
  const completeIdleCoverage=boolean(row.completeIdleCoverage),deadlineReached=boolean(row.deadlineReached);
  if(sampledConnections+failedConnections>openBefore||completeIdleCoverage&&(deadlineReached||failedConnections!==0||sampledConnections!==openBefore||openAfter!==openBefore))invalid();
  // A successful nonzero full observation cannot truthfully be a refusal.
  // Zero-open complete samples remain a real unsuccessful helper outcome.
  if(completeIdleCoverage&&sampledConnections>0)invalid();
  return {attempt:index+1,openBefore,openAfter,sampledConnections,failedConnections,completeIdleCoverage,deadlineReached};
 });
 return {error:value.error,sqlitePageCacheAuditWait:{attempts,elapsedMs,budgetMs:1000,retryIntervalMs:10,deadlineIsCooperative:true,observations}};
}
async function boundedFailure(response){
 if(response.status!==503||!/^application\/json(?:\s*;|$)/i.test(response.headers.get('content-type')??'')){
  await response.body?.cancel().catch(()=>{});return null;
 }
 const reader=response.body?.getReader();if(!reader)return null;
 const chunks=[];let bytes=0;
 try{
  for(;;){const part=await reader.read();if(part.done)break;bytes+=part.value.byteLength;
   if(bytes>MAX_BODY_BYTES){await reader.cancel();return null;}chunks.push(part.value);
  }
  const all=new Uint8Array(bytes);let offset=0;for(const part of chunks){all.set(part,offset);offset+=part.byteLength;}
  return cacheAuditFailureReport(JSON.parse(new TextDecoder('utf-8',{fatal:true}).decode(all)));
 }catch{return null;}finally{reader.releaseLock();}
}
class CacheAuditRefusal extends Error {
 constructor(evidence){super('Fixture cache audit refused');this.evidence=cacheAuditFailureReport(evidence);}
}
export async function throwDiagnosticSnapshotFailure(response){
 const evidence=await boundedFailure(response);
 if(evidence)throw new CacheAuditRefusal(evidence);
 throw new Error('Fixture diagnostic snapshot unavailable');
}
export function cacheAuditErrorEvidence(error){
 return error instanceof CacheAuditRefusal?cacheAuditFailureReport(error.evidence):null;
}
export function diagnosticFailureReport(error,kind='coordinated-native-http-fixture'){
 const evidence=cacheAuditErrorEvidence(error);
 if(!['coordinated-native-http-fixture','native-http-fixture'].includes(kind))invalid();
 return {schema:1,kind,target:'native fixture (unverified)',scenario:'generator coordination failed',runtime:process.version,
  model:'no successful capacity qualification',protection:'not verified',uniqueAuthenticatedPrincipals:null,stages:[],coordinationValid:false,
  capacityTarget:{httpRateMet:false,httpMultiUserTargetMet:false,met:false},
  reason:'Worker coordination or measurement failed; no capacity qualification.',...(evidence?{cacheAuditFailure:evidence}:{})};
}
