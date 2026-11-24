// Generator calibration, Node vs native, against the same mw-loadgen calibration
// server in the same session: (a) a trivial GET without validation, (b) the
// goal-65-35 mix with full validation against a replay server that returns recorded
// (or synthetic) responses for exactly the requests the mix sends. Reports CPU
// microseconds per request and requests per CPU-second for each generator.
// NOT a Mailwoman measurement. Usage:
//  node calibrate.mjs --mode=trivial|goal [--corpus=<private replay corpus>] [--handoff=<private>] [--protocol=h1|h2c]
//    [--generators=node,native] [--seconds=10] [--concurrency=64] [--rate=N] [--threads=1] [--server-threads=2] [--out=file]
import {spawn} from 'node:child_process';
import {mkdtempSync,rmSync,writeFileSync,readFileSync,existsSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {setTimeout as sleep} from 'node:timers/promises';
import {measure} from '../http-load.mjs';
import {createNativeRun,nativeBinary,runBinary,nativeStageResult} from './native-measure.mjs';
import {syntheticPrincipals,syntheticSimulation,prepare,nodeSpecByName} from './equivalence/common.mjs';
import {good} from './equivalence/synthetic-responses.mjs';
import {GOAL_MIX_NAME} from '../representative-workload.mjs';

const args=Object.fromEntries(process.argv.slice(2).map(a=>{const i=a.indexOf('=');return [a.slice(2,i),a.slice(i+1)];}));
const mode=args.mode??'trivial',protocol=args.protocol??'h1',seconds=Number(args.seconds??10),concurrency=Number(args.concurrency??64);
const rate=args.rate===undefined?undefined:Number(args.rate),threads=Number(args.threads??1),generators=(args.generators??'node,native').split(',');
if(seconds>120)throw new Error('Calibration phases are limited to 120 s on the shared host');
const h2=protocol==='h2c'?{mode:'h2c',preset:'proxy-like',perClient:false,connections:4,maxStreams:100}:undefined;
const dir=mkdtempSync(join(tmpdir(),'mw-native-calibration-'));
const bin=nativeBinary();
let server;
try{
 let principals,clientSimulation,corpusFile=join(dir,'replay.private.json');
 if(mode==='goal'){
  if(args.recorded){
   // real-shaped replay: N clones of the recorded principal 0 (live-session corpus); every id string of principal 0
   // is renamed per clone in the clone's requests (built by Node's templates) and in the recorded responses.
   const h=JSON.parse(readFileSync(args.handoff,'utf8')),p0=h.principals[0],n=Number(args.principals??200);
   const recorded=JSON.parse(readFileSync(args.recorded,'utf8')).filter(e=>e.principalIndex===0);
   const idsOf=p=>[p.accountId,p.ownerAccountId,...p.messageIds,p.inboxId,p.archiveId,p.trashId,p.draftsId,p.attachmentBlobId].filter(v=>typeof v==='string');
   principals=Array.from({length:n},(_,k)=>({...p0,cookie:`mw_session=clone${k}s; mw_csrf=clone${k}c`,csrfToken:`clone${k}c`,clientAddress:`198.18.${(k+1)>>8}.${(k+1)&255}`,
    accountId:p0.accountId+'c'+k,ownerAccountId:(p0.ownerAccountId??p0.accountId)+'c'+k,messageIds:p0.messageIds.map(id=>id+'c'+k),inboxId:p0.inboxId+'c'+k,archiveId:p0.archiveId+'c'+k,trashId:p0.trashId+'c'+k,draftsId:p0.draftsId+'c'+k,attachmentBlobId:p0.attachmentBlobId+'c'+k}));
   clientSimulation=syntheticSimulation(n);
   const prepared=prepare({principals,clientSimulation}),entries=[],original=idsOf(p0);
   const rename=(value,map)=>typeof value==='string'?(map.get(value)??value):Array.isArray(value)?value.map(v=>rename(v,map)):value&&typeof value==='object'?Object.fromEntries(Object.entries(value).map(([k,v])=>[map.get(k)??k,rename(v,map)])):value;
   principals.forEach((p,k)=>{
    const map=new Map(original.map((id,i)=>[id,idsOf(p)[i]]));
    for(const e of recorded){
     const spec=nodeSpecByName(prepared.groups,principals,e.name,k,e.draftId);
     const headers={...prepared.clientModel.clients[k].headers,...spec.headers},bytes=Buffer.from(e.bodyBase64,'base64');
     const binary=e.name==='attachment-download';
     entries.push({method:spec.method??'GET',path:spec.path,cookie:headers.cookie,requestBody:spec.body??'',status:e.status,contentType:binary?'application/octet-stream':'application/json',
      ...(binary?{responseBase64:e.bodyBase64}:{response:JSON.stringify(rename(JSON.parse(bytes.toString('utf8')),map))})});
    }
   });
   writeFileSync(corpusFile,JSON.stringify(entries),{mode:0o600});
  }else if(args.handoff){const h=JSON.parse(readFileSync(args.handoff,'utf8'));principals=h.principals;clientSimulation=h.clientSimulation;}
  else{const n=Number(args.principals??20);principals=syntheticPrincipals(n);clientSimulation=syntheticSimulation(n);}
  if(args.recorded){}
  else if(args.corpus)corpusFile=args.corpus;
  else{
   // synthetic replay corpus: one recorded response per (principal, goal shape) plus the discard of the draft every save returns
   const prepared=prepare({principals,clientSimulation}),entries=[];
   const names=[...new Set(prepared.groups[GOAL_MIX_NAME].map(s=>s.name))];
   principals.forEach((p,i)=>{
    for(const name of [...names,'draft-discard(id)']){
     const draftId=name==='draft-discard(id)'?'D-new-1':undefined,spec=nodeSpecByName(prepared.groups,principals,draftId?'draft-discard':name,i,draftId);
     const headers={...prepared.clientModel.clients[i].headers,...spec.headers};
     entries.push({method:spec.method??'GET',path:spec.path,cookie:headers.cookie,requestBody:spec.body??'',status:200,contentType:'application/json',response:JSON.stringify(good(draftId?'draft-discard':name,p,spec,draftId))});
    }
   });
   writeFileSync(corpusFile,JSON.stringify(entries),{mode:0o600});
  }
 }
 const portFile=join(dir,'port'),statsFile=join(dir,'server-stats.json');
 server=spawn(bin,['serve',`--mode=${mode==='goal'?'replay':'constant'}`,...(mode==='goal'?[`--corpus=${corpusFile}`]:[]),`--threads=${args['server-threads']??2}`,`--seconds=${Math.ceil(seconds*generators.length*2+60)}`,`--port-file=${portFile}`,`--stats-file=${statsFile}`],{stdio:'ignore',windowsHide:true});
 for(let i=0;i<200&&!existsSync(portFile);i++)await sleep(50);
 const base=`http://127.0.0.1:${readFileSync(portFile,'utf8').trim()}`;
 const results=[];
 const summary=(generator,r)=>({generator,mode,protocol,threads:generator==='native'?threads:1,concurrency,arrivalRate:rate??'closed-loop',seconds,requests:r.requests,accepted:r.accepted,errors:r.errors,statuses:r.statuses,
  rps:+r.requestsPerSecond.toFixed(1),cpuLogicalCores:+r.generator.cpuLogicalCores.toFixed(3),cpuMicrosPerRequest:+((r.generator.cpuUserMs+r.generator.cpuSystemMs)*1000/r.requests).toFixed(1),
  rpsPerCpuCore:+(r.requests/((r.generator.cpuUserMs+r.generator.cpuSystemMs)/1000)).toFixed(0),latencyMs:r.latencyMs,schedulingLagP99Ms:r.schedulingLagMs?.p99,generatorLimited:r.generatorLimited,lagMaxMsBySecond:r.native?.schedulingLagMaxMsBySecond?.map(v=>+v.toFixed(2))});
 if(mode==='trivial'){
  const requests=[{name:'fixture-read',path:'/read'}];
  for(const generator of generators){
   if(generator==='node'){results.push(summary('node',await measure({base,requests,concurrency,seconds,timeoutMs:5000,...(rate?{arrivalRate:rate,latePolicy:'dispatch'}:{}),...(h2?{h2}:{})})));continue;}
   const stage=join(dir,'stage.json'),out=join(dir,'out.json');
   writeFileSync(stage,JSON.stringify({base,concurrency,seconds,timeoutMs:5000,...(rate?{arrivalRate:rate,latePolicy:'dispatch'}:{}),...(h2?{h2}:{}),threads,runtime:args.runtime??'multi',...(args.tick?{tickMicros:Number(args.tick)}:{}),plain:requests.map(r=>({name:r.name,path:r.path}))}));
   await runBinary(bin,['stage',`--stage=${stage}`,`--out=${out}`],{timeoutMs:(seconds+70)*1000});
   results.push(summary('native',nativeStageResult(JSON.parse(readFileSync(out,'utf8')),{requests,seconds,concurrency,arrivalRate:rate,latePolicy:rate?'dispatch':'drop'})));
  }
 }else{
  const prepared=prepare({principals,clientSimulation}),requests=prepared.groups[GOAL_MIX_NAME];
  for(const generator of generators){
   const options={base,headers:prepared.headers,requests,concurrency,seconds,timeoutMs:5000,...(rate?{arrivalRate:rate,latePolicy:'dispatch'}:{}),...(h2?{h2}:{})};
   if(generator==='node'){results.push(summary('node',await measure(options)));continue;}
   const native=createNativeRun({principals,clientSimulation,engineMode:'native',representative:true,groups:prepared.groups,enforcedPolicy:null,env:{...process.env,MW_NATIVE_GENERATOR_THREADS:String(threads),MW_NATIVE_GENERATOR_RATE_GUARD:args['rate-guard']==='on'?'on':'off',...(args['clients-per-user']?{MW_NATIVE_GENERATOR_CLIENTS_PER_USER:args['clients-per-user']}:{}),...(args.runtime?{MW_NATIVE_GENERATOR_RUNTIME:args.runtime}:{}),...(args.tick?{MW_NATIVE_GENERATOR_TICK_US:args.tick}:{})}});
   try{const r=await native.run(options);results.push({...summary('native',r),uniqueClientsAccepted:r.uniqueClientsAccepted,uniquePrincipalsAccepted:r.uniquePrincipalsAccepted,rateGuard:r.native.rateGuard,validationUs:r.native.validationUs,schedulingLagUs:r.native.schedulingLagUs,latencyUs:r.native.latencyUs});}finally{native.close();}
  }
 }
 await sleep(200);
 const report={generatedAt:new Date().toISOString(),note:'calibration server (mw-loadgen serve), not Mailwoman; generator CPU from the generator process only',server:{threads:Number(args['server-threads']??2)},results};
 if(args.out)writeFileSync(args.out,JSON.stringify(report,null,1)+'\n');
 console.log(JSON.stringify(results,null,1));
}finally{server?.kill();await sleep(100);rmSync(dir,{recursive:true,force:true});}
