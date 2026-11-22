// Private signed capabilities stay in memory/IPC and never enter reports.
import http from 'node:http';
import {createHash} from 'node:crypto';
export const CASES=Object.freeze(['cold','coalesced','ram','disk']);
export const COUNTERS=Object.freeze(['ramHits','diskHits','misses','ramUsedBytes','ramBudgetBytes','diskUsedBytes','diskBudgetBytes','diskPendingBytes','diskErrors','ramEntries','diskEntries','originFetches']);
const agents=new Map();
const requireValue=(yes)=>{if(!yes)throw Error('Invalid private image fixture proof');};
export function loopback(value){const u=new URL(value);requireValue(u.protocol==='http:'&&u.hostname==='127.0.0.1'&&u.port&&u.username===''&&u.password===''&&u.pathname==='/'&&!u.search&&!u.hash);return u.origin;}
export function handoff(value){
 requireValue(value?.schema===1&&value.kind==='real-signed-image-proxy'&&value.sourceAccounts===16&&value.sourceSessions===16&&value.lifetimeSecs===900&&value.security==='enforced-default-limits'&&value.storeEncryption==='required'&&value.storeFileNotPlaintext===true&&value.origin==='local-configured-connect-standin'&&value.originDelayMs===20);
 requireValue(value.controls?.noSession===401&&value.controls?.tamperedToken===403&&value.controls?.privateTarget===403&&value.controls?.originFetches===0&&value.controls?.positiveSignedAccounts===16&&value.controls?.positiveSignedStatus===200);
 return {base:loopback(value.base),control:loopback(value.control),sourceAccounts:16,sourceSessions:16,security:value.security,originDelayMs:20};
}
export function phase(value,expectedCase,size){
 requireValue(CASES.includes(expectedCase)&&[64,128].includes(size)&&value?.schema===1&&value.case===expectedCase&&value.size===size&&value.sourceAccounts===16&&value.sourceSessions===16&&value.originDelayMs===20);
 requireValue(Number.isSafeInteger(value.expectedBytes)&&value.expectedBytes>0&&value.expectedBytes<=262144&&/^[a-f0-9]{64}$/.test(value.expectedSha256));
 requireValue(Array.isArray(value.paths)&&value.paths.length===(expectedCase==='cold'?2048:16));
 const seen=new Set();for(const row of value.paths){requireValue(row&&Number.isInteger(row.principalIndex)&&row.principalIndex>=0&&row.principalIndex<16&&typeof row.path==='string'&&row.path.length<16384&&/^\/api\/rc\/i\/[A-Za-z0-9_.-]+$/.test(row.path)&&!seen.has(row.path));seen.add(row.path);}
 requireValue(new Set(value.paths.map(x=>x.principalIndex)).size===16);
 return value;
}
export function counters(value){requireValue(value&&Object.keys(value).sort().join(',')===[...COUNTERS].sort().join(','));for(const key of COUNTERS)requireValue(Number.isSafeInteger(value[key])&&value[key]>=0);return value;}
export function delta(before,after){counters(before);counters(after);const result={};for(const key of ['ramHits','diskHits','misses','diskErrors','originFetches']){requireValue(after[key]>=before[key]);result[key]=after[key]-before[key];}return result;}
export function qualify(kind,before,after,measurement){
 const d=delta(before,after);requireValue(CASES.includes(kind));
 const accepted=measurement.accepted;
 if(!Number.isSafeInteger(accepted)||accepted<1||measurement.errors!==0||measurement.timeouts!==0||measurement.throttled!==0)return {qualified:false,reason:'http-errors-or-no-accepted-responses',delta:d};
 const known=kind==='ram'?d.originFetches===0&&d.ramHits>=accepted:
  kind==='disk'?before.ramBudgetBytes===0&&after.ramBudgetBytes===0&&before.diskPendingBytes===0&&after.diskPendingBytes===0&&d.originFetches===0&&d.diskHits>=accepted&&d.ramHits===0:
  kind==='cold'?d.originFetches===accepted:
  accepted===64&&d.originFetches===16;
 return {qualified:known&&d.diskErrors===0,reason:known?'observed-outcome':'cache-or-origin-outcome-not-proven',delta:d};
}
// Bounded response reader, no automatic redirects. Actual HTTP peer binding is
// a local test source; it is not external network/client diversity evidence.
export function request(base,path,{method='GET',body,token,principalIndex,timeoutMs=10000,maxBytes=262144}={}){
 const origin=loopback(base);requireValue(typeof path==='string'&&path.startsWith('/')&&!path.startsWith('//')&&path.length<16384);
 if(token!==undefined)requireValue(/^[a-f0-9]{64}$/.test(token));
 if(principalIndex!==undefined)requireValue(Number.isInteger(principalIndex)&&principalIndex>=0&&principalIndex<16);
 return new Promise((resolve,reject)=>{
  const localAddress=principalIndex===undefined?'127.0.0.1':`127.0.0.${principalIndex+2}`;
  if(!agents.has(localAddress))agents.set(localAddress,new http.Agent({keepAlive:true,maxSockets:64,maxFreeSockets:16}));
  const failed=(timeout=false)=>{const error=Error(timeout?'Image request timed out':'Image request failed');error.code=timeout?'ETIMEDOUT':'EIMAGE';return error;};
  let timedOut=false;
  const req=http.request(new URL(path,origin),{method,agent:agents.get(localAddress),localAddress,headers:{'user-agent':'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0.0.0 Safari/537.36',...(body!==undefined?{'content-type':'application/json'}:{}),...(token?{'x-mw-image-bench-token':token}:{})}},res=>{
   let bytes=0;const chunks=[];res.on('data',chunk=>{bytes+=chunk.length;if(bytes>maxBytes){res.destroy();req.destroy();reject(Error('Bounded image response exceeded'));}else chunks.push(chunk);});
   res.on('error',()=>reject(Error('Image response failed')));res.on('end',()=>resolve({status:res.statusCode,bytes:Buffer.concat(chunks),contentType:res.headers['content-type']??''}));
  });req.setTimeout(timeoutMs,()=>{timedOut=true;req.destroy(failed(true));});req.on('error',()=>reject(failed(timedOut)));if(body!==undefined)req.write(JSON.stringify(body));req.end();
 });
}
export function validImage(response,proof){return response.status===200&&response.contentType.startsWith('image/png')&&response.bytes.length===proof.expectedBytes&&createHash('sha256').update(response.bytes).digest('hex')===proof.expectedSha256;}
// Signed wall delta maps EVERY worker onto the same absolute window, even
// after a late spawn. Never clamp a past start into a new full-duration run.
export function scheduledWindow(startAt,seconds,wallNow,monoNow){
 requireValue(Number.isSafeInteger(startAt)&&startAt>0&&Number.isInteger(seconds)&&seconds>=1&&seconds<=10&&Number.isSafeInteger(wallNow)&&Number.isFinite(monoNow));
 const startMono=monoNow+(startAt-wallNow);
 return {startAt,endAt:startAt+seconds*1000,startMono,deadlineMono:startMono+seconds*1000,readyBeforeStart:wallNow<=startAt};
}
export function coordination(parts,startAt,seconds){
 requireValue(Array.isArray(parts)&&parts.length>=1&&parts.length<=4);
 const windows=parts.map(x=>x.window);
 const qualified=windows.every(w=>w&&w.startAt===startAt&&w.endAt===startAt+seconds*1000&&w.readyBeforeStart===true&&Number.isFinite(w.startLagMs)&&w.startLagMs>=0&&w.startLagMs<=50&&Number.isFinite(w.wallClockDriftMs)&&Math.abs(w.wallClockDriftMs)<=100);
 return {qualified,readyBeforeScheduledStart:windows.every(w=>w?.readyBeforeStart===true),maximumStartLagMs:Math.max(...windows.map(w=>Number.isFinite(w?.startLagMs)?w.startLagMs:Infinity)),startLagToleranceMs:50,wallClockDriftToleranceMs:100};
}
