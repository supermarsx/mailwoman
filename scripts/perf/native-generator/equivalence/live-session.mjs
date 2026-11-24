// Live equivalence session against ONE running fixture (external-driver mode:
// MW_LOAD_FIXTURE_EXTERNAL_DRIVER=1, MW_LOAD_FIXTURE_EXTERNAL_ROOT=<root>). It claims
// the run like http-external-driver.mjs, then
//  1. records one response per (principal, shape) of every group for the first
//     --record principals with Node's own templates (and the discard of every saved
//     draft), into a private corpus usable by validators-compare.mjs and as a replay
//     corpus for calibrate.mjs;
//  2. runs the Node campaign (http-real-fixture.mjs) and then the native campaign
//     (MW_LOAD_GENERATOR=native) with the fixture's campaign environment;
//  3. compares per stage and shape: requests, accepted, errors, status distribution.
// The native report is handed back to the fixture as the run's report.
// Usage: node live-session.mjs --root=<external root> --out=<private dir> [--record=5] [--wait=1800]
import {spawn} from 'node:child_process';
import {join} from 'node:path';
import {writeFileSync,readFileSync,copyFileSync} from 'node:fs';
import {setTimeout as sleep} from 'node:timers/promises';
import {findReady,privateJson,runClaimed,readPrivate,HANDOFF_LIMIT} from '../../external-control.mjs';
import {prepare,nodeSpecByName,shapeIndex} from './common.mjs';
import {goalMixDraftSpecs} from '../../representative-workload.mjs';

const args=Object.fromEntries(process.argv.slice(2).map(a=>{const i=a.indexOf('=');return [a.slice(2,i),a.slice(i+1)];}));
const scripts=join(import.meta.dirname,'..','..');
let found=null;const until=Date.now()+Number(args.wait??1800)*1000;
while(!(found=await findReady(args.root))){if(Date.now()>until)throw new Error('no fixture run appeared');await sleep(500);}
const {directory,ready}=found;
await privateJson(join(directory,'claim.json'),{runId:ready.runId,claimedAtMs:Date.now()},{exclusive:true});
const out=args.out;
const child=(env,output)=>new Promise(resolve=>{
 const c=spawn(process.execPath,[join(scripts,'http-real-fixture.mjs'),join(directory,ready.handoffFile),output],{env:{...process.env,...ready.campaignEnv,...env},stdio:['ignore','ignore','pipe']});
 let err='';c.stderr.on('data',d=>{if(err.length<4000)err+=d;});c.once('exit',code=>resolve({code,err:err.trim().slice(0,600)}));
});
await runClaimed({directory,ready,execute:async()=>{
 const handoff=JSON.parse((await readPrivate(join(directory,ready.handoffFile),HANDOFF_LIMIT)).toString('utf8'));
 const principals=handoff.principals,base=handoff.base;
 const prepared=prepare({principals,clientSimulation:handoff.clientSimulation});
 // 1. recording
 const corpus=[],replay=[];
 const send=async(spec,i)=>{
  const headers={...prepared.headers,...prepared.clientModel.clients[i].headers,...spec.headers};
  const response=await fetch(new URL(spec.path,base),{method:spec.method??'GET',headers,...(spec.body!==undefined?{body:spec.body}:{}),redirect:'error',signal:AbortSignal.timeout(30000)});
  const bytes=Buffer.from(await response.arrayBuffer());
  replay.push({method:spec.method??'GET',path:spec.path,cookie:headers.cookie,requestBody:spec.body??'',status:response.status,contentType:response.headers.get('content-type')??'',responseBase64:bytes.toString('base64')});
  return {status:response.status,bytes};
 };
 const names=[...shapeIndex(prepared.groups).keys()];
 const recordCount=Math.min(principals.length,Number(args.record??5));
 for(let i=0;i<recordCount;i++)for(const name of names){
  const spec=nodeSpecByName(prepared.groups,principals,name,i);
  const {status,bytes}=await send(spec,i);
  corpus.push({principalIndex:i,name,status,bodyBase64:bytes.toString('base64')});
  if(name==='draft-save'||name==='draft-discard'){
   const id=JSON.parse(bytes.toString('utf8'))?.methodResponses?.[0]?.[1]?.created?.d1?.id;
   if(typeof id==='string'){const r=await send(goalMixDraftSpecs(principals[i],{}).discard(id),i);corpus.push({principalIndex:i,name:'draft-discard',draftId:id,status:r.status,bodyBase64:r.bytes.toString('base64')});}
  }
 }
 writeFileSync(join(out,'corpus.private.json'),JSON.stringify(corpus),{mode:0o600});
 writeFileSync(join(out,'replay.private.json'),JSON.stringify(replay),{mode:0o600});
 writeFileSync(join(out,'handoff.private.json'),JSON.stringify(handoff),{mode:0o600});
 // 2. the two campaigns
 const node=await child({MW_LOAD_GENERATOR:'node'},join(out,'node-report.json'));
 const native=await child({MW_LOAD_GENERATOR:'native'},join(directory,ready.outputFile));
 try{copyFileSync(join(directory,ready.outputFile),join(out,'native-report.json'));copyFileSync(join(directory,'report.md'),join(out,'native-report.md'));}catch{}
 // 3. comparison
 const load=path=>{try{return JSON.parse(readFileSync(path,'utf8'));}catch{return null;}};
 const a=load(join(out,'node-report.json')),b=load(join(out,'native-report.json'));
 const rows=[];
 for(const [k,stage] of (a?.stages??[]).entries()){
  const other=b?.stages?.[k];
  for(const [name,s] of Object.entries(stage.byRequest??{})){const t=other?.byRequest?.[name];rows.push({stage:k,endpoint:stage.endpoint,phase:stage.phase,arrivalRate:stage.arrivalRate??null,name,node:{requests:s.requests,accepted:s.accepted,errors:s.errors,statuses:s.statuses,p50:s.latencyMs.p50},native:t?{requests:t.requests,accepted:t.accepted,errors:t.errors,statuses:t.statuses,p50:t.latencyMs.p50}:null});}
 }
 const summary={nodeExit:node.code,nodeError:node.err||null,nativeExit:native.code,nativeError:native.err||null,recorded:corpus.length,
  stages:{node:a?.stages?.length??0,native:b?.stages?.length??0},tainted:{node:a?.mutationStateTainted??null,native:b?.mutationStateTainted??null},
  everyRejectedOnBothOrNeither:rows.every(r=>r.native&&(r.node.errors===0)===(r.native.errors===0)),rows};
 writeFileSync(join(out,'comparison.json'),JSON.stringify(summary,null,1));
 console.log(JSON.stringify({nodeExit:node.code,nativeExit:native.code,nodeError:summary.nodeError,nativeError:summary.nativeError,recorded:corpus.length,stages:summary.stages,every:summary.everyRejectedOnBothOrNeither}));
 return native.code;
}});
