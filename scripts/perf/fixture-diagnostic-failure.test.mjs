import test from 'node:test';
import assert from 'node:assert/strict';
import {cacheAuditFailureReport,throwDiagnosticSnapshotFailure,cacheAuditErrorEvidence,diagnosticFailureReport} from './fixture-diagnostic-failure.mjs';
const partial=(n=1)=>({attempt:n,openBefore:8,openAfter:8,sampledConnections:6,failedConnections:0,completeIdleCoverage:false,deadlineReached:false});
function failure(code='cache-audit-quiescence-timeout'){
 return {error:code,sqlitePageCacheAuditWait:{attempts:2,elapsedMs:1000,budgetMs:1000,retryIntervalMs:10,deadlineIsCooperative:true,observations:[partial(1),partial(2)]}};
}
const response=(value,status=503)=>new Response(JSON.stringify(value),{status,headers:{'content-type':'application/json'}});
async function refusal(value,status=503){try{await throwDiagnosticSnapshotFailure(response(value,status));assert.fail('always refuses');}catch(error){return error;}}

test('only fixed cache audit refusals retain bounded aggregate evidence and never qualify',async()=>{
 for(const code of ['cache-audit-unavailable','cache-audit-quiescence-timeout']){
  const raw=failure(code);raw.cookie='private';raw.sqlitePageCacheAuditWait.path='private';raw.sqlitePageCacheAuditWait.observations[0].pointer='private';
  const error=await refusal(raw),safe=cacheAuditErrorEvidence(error);
  assert.deepEqual(safe,cacheAuditFailureReport(failure(code)));assert.equal(JSON.stringify(safe).includes('private'),false);
  for(const kind of ['native-http-fixture','coordinated-native-http-fixture']){
   const report=diagnosticFailureReport(error,kind);
   assert.equal(report.coordinationValid,false);assert.deepEqual(report.stages,[]);
   assert.equal(report.capacityTarget.httpRateMet,false);assert.equal(report.capacityTarget.httpMultiUserTargetMet,false);assert.equal(report.capacityTarget.met,false);
   assert.deepEqual(report.cacheAuditFailure,safe);assert.equal(JSON.stringify(report).includes('private'),false);
  }
 }
});

test('zero-open outcomes and an interrupted final attempt stay reportable without becoming success',async()=>{
 const value=failure('cache-audit-unavailable');value.sqlitePageCacheAuditWait.observations=[{attempt:1,openBefore:0,openAfter:0,sampledConnections:0,failedConnections:0,completeIdleCoverage:true,deadlineReached:false}];
 value.sqlitePageCacheAuditWait.elapsedMs=60001;
 assert.equal(cacheAuditFailureReport(value).sqlitePageCacheAuditWait.elapsedMs,60001);
 assert.equal(cacheAuditFailureReport(value).sqlitePageCacheAuditWait.attempts,2);
 const error=await refusal(value);assert.ok(cacheAuditErrorEvidence(error));assert.equal(diagnosticFailureReport(error).capacityTarget.met,false);
});

test('status, media type, UTF8, JSON and unknown codes cannot launder arbitrary server errors',async()=>{
 for(const [status,value] of [[400,failure()],[401,failure()],[500,failure()],[503,{...failure(),error:'private-arbitrary-error'}]]){
  assert.equal(cacheAuditErrorEvidence(await refusal(value,status)),null);
 }
 for(const body of ['not-json',new Uint8Array([0xff,0xfe])]){
  let error;try{await throwDiagnosticSnapshotFailure(new Response(body,{status:503,headers:{'content-type':'application/json'}}));}catch(value){error=value;}
  assert.equal(cacheAuditErrorEvidence(error),null);
 }
 let cancelled=false;
 const body=new ReadableStream({cancel(){cancelled=true;}});
 await assert.rejects(throwDiagnosticSnapshotFailure(new Response(body,{status:503,headers:{'content-type':'text/plain'}})));
 assert.equal(cancelled,true);
});

test('streaming body limit cancels oversized responses before parsing or retaining contents',async()=>{
 let chunks=0,cancelled=false;
 const stream=new ReadableStream({pull(controller){chunks++;controller.enqueue(new Uint8Array(8192).fill(65));},cancel(){cancelled=true;}});
 let error;try{await throwDiagnosticSnapshotFailure(new Response(stream,{status:503,headers:{'content-type':'application/json'}}));}catch(value){error=value;}
 assert.equal(cancelled,true);assert.ok(chunks<=7);assert.equal(cacheAuditErrorEvidence(error),null);
 assert.equal(JSON.stringify(diagnosticFailureReport(error)).includes('AAAA'),false);
});

test('exact finite shape rejects overflow, malformed coverage, false full success and excess outcomes',()=>{
 for(const mutate of [v=>v.sqlitePageCacheAuditWait.attempts=101,v=>v.sqlitePageCacheAuditWait.elapsedMs=Number.MAX_SAFE_INTEGER+1,v=>v.sqlitePageCacheAuditWait.elapsedMs=NaN,
  v=>v.sqlitePageCacheAuditWait.budgetMs=2000,v=>v.sqlitePageCacheAuditWait.deadlineIsCooperative=false,
  v=>v.sqlitePageCacheAuditWait.observations[0].attempt=2,v=>v.sqlitePageCacheAuditWait.observations[0].openBefore=33,
  v=>v.sqlitePageCacheAuditWait.observations[0].sampledConnections=9,v=>v.sqlitePageCacheAuditWait.observations[0].deadlineReached=null,
  v=>{v.sqlitePageCacheAuditWait.observations[0]={...partial(),sampledConnections:8,completeIdleCoverage:true};},
  v=>v.sqlitePageCacheAuditWait.observations=[],v=>v.sqlitePageCacheAuditWait.observations.push(partial(3))]){
  const value=failure();mutate(value);assert.throws(()=>cacheAuditFailureReport(value));
 }
});

test('untrusted Error properties never become report evidence and typed evidence is revalidated',async()=>{
 const forged=new Error('private credentials');forged.evidence=failure();
 assert.equal(cacheAuditErrorEvidence(forged),null);assert.equal(JSON.stringify(diagnosticFailureReport(forged)).includes('private credentials'),false);
 const real=await refusal(failure());real.evidence.sqlitePageCacheAuditWait.attempts=1000;
 assert.throws(()=>cacheAuditErrorEvidence(real));
});
