import test from 'node:test';
import assert from 'node:assert/strict';
import {createServer} from 'node:http';
import {fork} from 'node:child_process';
import {mkdtemp,writeFile,readFile,rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {fileURLToPath} from 'node:url';
import {requestInputCounters as keys,requestInputsStatusReport as status,requestInputsEvidence as evidence} from './request-inputs-experiment.mjs';
import {campaignOptions} from './http-campaign.mjs';
import {fixtureDiagnosticsReport} from './fixture-clients.mjs';
import {runMultiCampaign} from './http-multi-controller.mjs';
const witness=(mode='published',count=0)=>({mode,processImmutable:true,...Object.fromEntries(keys.map(key=>[key,count]))});
const windowProof=()=>({schema:1,kind:'workload-and-drain-only',eligibleControlRequests:0});
const stage=(mode='published',before=0,after=1)=>({endpoint:'identity',phase:'offered-rate',byRequest:{'authenticated-identity':{requests:2,acceptedDuringScheduledWindow:1}},requestInputsWindow:windowProof(),diagnostics:{before:{ordinaryRequestInputs:witness(mode,before)},after:{ordinaryRequestInputs:witness(mode,after)}}});
test('optional expectation is strict and legacy c19 remains unqualified without opt in',()=>{
 assert.equal(evidence(undefined,[],undefined),null);assert.equal(status(undefined),null);
 assert.equal(campaignOptions({}).requestInputsExpected,undefined);
 for(const mode of ['original','published'])assert.equal(campaignOptions({MW_LOAD_FIXTURE_REQUEST_INPUTS_EXPECT:mode}).requestInputsExpected,mode);
 for(const mode of ['','composite','Original',' published','published '])assert.throws(()=>campaignOptions({MW_LOAD_FIXTURE_REQUEST_INPUTS_EXPECT:mode}));
});
test('witness whitelist has only current eight monotonic counters and immutable mode',()=>{
 const raw={...witness(),private:'must-not-leak',cookie:'private-cookie'};
 assert.deepEqual(status(raw),witness());assert.equal(JSON.stringify(fixtureDiagnosticsReport({ordinaryRequestInputs:raw})).includes('private'),false);
 for(const change of [{mode:'composite'},{processImmutable:false},{buildAttempts:undefined},{readyFrames:-1},{hintAttempts:NaN},{consumedFrames:2**53}])assert.throws(()=>status({...witness(),...change}));
});
test('original requires every startup and phase counter zero including worker builds',()=>{
 assert.equal(evidence(witness('original'),[stage('original',0,0)],'original').qualified,true);
 for(const key of keys){const start=witness('original');start[key]=1;assert.throws(()=>evidence(start,[stage('original',0,0)],'original'));const s=stage('original',0,0);s.diagnostics.after.ordinaryRequestInputs[key]=1;assert.throws(()=>evidence(witness('original'),[s],'original'));}
});
test('published requires measured eligible acceptance plus phase consumption not startup or build alone',()=>{
 const s=stage('published',10,11);const p=evidence(witness('published',9),[s],'published');assert.equal(p.qualified,true);assert.equal(p.consumedFramesDelta,1);assert.equal(p.knownSubsetAcceptedDuringScheduledWindow,1);
 const startupOnly=stage('published',10,10);assert.throws(()=>evidence(witness('published',10),[startupOnly],'published'));
 startupOnly.diagnostics.after.ordinaryRequestInputs.buildAttempts=11;assert.throws(()=>evidence(witness('published',10),[startupOnly],'published'));
});
test('positive header counters and other out-of-model routes remain conservatively unqualified',()=>{
 for(const [endpoint,name]of [['headers','Email/get'],['attachment','attachment-download'],['refresh','concurrent-refresh-handler']]){
  const s=stage();s.endpoint=endpoint;s.byRequest={[name]:{requests:2,acceptedDuringScheduledWindow:1}};
  const proof=evidence(witness(),[s],'published');assert.equal(proof.qualified,false);assert.equal(proof.exercise,'unexercised');assert.equal(proof.consumedFramesDelta,1,'positive process use does not make an excluded request label qualify');assert.match(proof.eligibilityScope,/known subset/);
 }
 const s=stage('published',1,1);s.byRequest['authenticated-identity'].acceptedDuringScheduledWindow=0;assert.equal(evidence(witness('published',1),[s],'published').qualified,false);
});
test('mode changes resets missing stats arbitrary labels and unknown controls refuse',()=>{
 for(const mutate of [s=>delete s.diagnostics.before.ordinaryRequestInputs,s=>s.diagnostics.after.ordinaryRequestInputs.mode='original',s=>s.diagnostics.after.ordinaryRequestInputs.consumedFrames=0,s=>s.endpoint='private-url',s=>s.byRequest={'private-label':{requests:1,acceptedDuringScheduledWindow:1}},s=>s.byRequest['authenticated-identity'].acceptedDuringScheduledWindow=3,s=>delete s.requestInputsWindow,s=>s.requestInputsWindow.eligibleControlRequests=1]){
  const s=stage('published',1,2);mutate(s);assert.throws(()=>evidence(witness('published',1),[s],'published'));
 }
 const next=stage('published',1,3);assert.throws(()=>evidence(witness(),[stage('published',0,2),next],'published'));
});

// Real driver and real Fetch/Node HTTP, synthetic disposable receiver only.
// These verify orchestration, not application auth/performance or capacity.
async function receiver(mode,{workloadConsumes=true}={}){
 let lifetime=0,measuring=false,phaseRequests=0;const order=[];
 const server=createServer(async(req,res)=>{
  let text='';for await(const chunk of req)text+=chunk;
  const answer=value=>{res.setHeader('content-type','application/json');res.end(JSON.stringify(value));};
  if(req.url==='/__fixture-diagnostic-aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa'){
   if(req.method==='POST'){order.push('before-reset');measuring=true;res.end();return;}
   order.push(measuring?'snapshot':'outside-snapshot');answer({ordinaryRequestInputs:witness(mode,lifetime)});return;
  }
  if(req.url==='/.env'){order.push('scanner');res.statusCode=403;res.end();return;}
  order.push(measuring?'workload':'preflight');if(measuring)phaseRequests++;
  if(mode==='published'&&(!measuring||workloadConsumes))lifetime++;
  if(req.url==='/api/me'){answer({accountId:'owned'});return;}
  if(req.url==='/api/mail/refresh'){answer({accountId:'owned',upstream:true});return;}
  if(req.url==='/jmap/api'){
   const input=JSON.parse(text);answer({methodResponses:input.methodCalls.map(([method,args,id])=>[method,{accountId:args.accountId,...(method==='Email/query'?{ids:['mail1','mail2']}:{}),...(method==='Email/get'?{list:(args.ids??[]).map(id=>({id})),notFound:[]}:{} )},id])});return;
  }
  res.statusCode=404;res.end();
 });
 await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
 const h={base:`http://127.0.0.1:${server.address().port}`,engineMode:'native',backend:'sqlite',buildProfile:'release',uniquePrincipalCount:1,
  principals:[{cookie:'mw_session=fixture; mw_csrf=csrf',csrfToken:'csrf',accountId:'owned',ownerAccountId:'owned',messageIds:['mail1','mail2'],inboxId:'inbox',clientAddress:'198.18.0.1'}],clientSimulation:{mode:'fixture-loopback-trusted-proxy',clientCount:1},
  startupReadiness:{verified:true,state:'ready',startupSeconds:1,indexWaitSeconds:0,seededMessages:2,expectedIndexDocuments:2,indexDocuments:2,zeroAccessExcludedMessages:0,rebuildRunning:false,rebuildDone:2,rebuildTotal:2},workloadProfile:{kind:'baseline',messagesPerPrincipal:2,seededMessages:2,seedSeconds:1},
  ordinaryRequestInputs:witness(mode),diagnostics:{path:'/__fixture-diagnostic-aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa',token:'fixture-only'}};
 return {handoff:h,order,phaseRequests:()=>phaseRequests,close:()=>new Promise(resolve=>{server.closeAllConnections();server.close(resolve);})};
}
async function realSingle(mode,{workloadConsumes=true,endpoint='identity'}={}){
 const app=await receiver(mode,{workloadConsumes});const dir=await mkdtemp(join(tmpdir(),'mw-input-receipt-'));
 try{
  const handoff=join(dir,'handoff.private.json'),output=join(dir,'report.json');await writeFile(handoff,JSON.stringify(app.handoff),{mode:0o600});
  const env=Object.fromEntries(['SystemRoot','WINDIR','TEMP','TMP','PATH'].filter(k=>process.env[k]!==undefined).map(k=>[k,process.env[k]]));
  Object.assign(env,{MW_LOAD_FIXTURE_MODE:'smoke',MW_LOAD_FIXTURE_SECONDS:'0.1',MW_LOAD_FIXTURE_RATES:'10',MW_LOAD_FIXTURE_CONCURRENCY:'1',MW_LOAD_FIXTURE_ENDPOINTS:endpoint,MW_LOAD_FIXTURE_REQUEST_INPUTS_EXPECT:mode});
  const child=fork(fileURLToPath(new URL('./http-real-fixture.mjs',import.meta.url)),[handoff,output],{env,stdio:['ignore','pipe','pipe','ipc'],execArgv:[]});let stderr='';child.stderr.on('data',b=>stderr+=b);child.stdout.resume();
  const code=await new Promise((resolve,reject)=>{const timeout=setTimeout(()=>{child.kill();reject(Error('driver timeout'));},15000);child.once('error',reject);child.once('exit',code=>{clearTimeout(timeout);resolve(code);});});
  if(!workloadConsumes){assert.equal(code,1);assert.match(stderr,/did not consume/);assert.ok(!stderr.includes('fixture-only'));return;}
  assert.equal(code,0,stderr);const report=JSON.parse(await readFile(output,'utf8'));assert.equal(report.ordinaryRequestInputs.expectedMode,mode);assert.ok(app.phaseRequests()>0);
  assert.ok(app.order.indexOf('preflight')<app.order.indexOf('before-reset'));assert.equal(app.order.at(-1),'scanner');
  if(endpoint==='headers')assert.equal(report.ordinaryRequestInputs.qualified,false);else assert.equal(report.ordinaryRequestInputs.qualified,true);
 }finally{await app.close();await rm(dir,{recursive:true,force:true});}
}
test('actual single driver original and published validate phase use with real loopback Fetch',async()=>{await realSingle('original');await realSingle('published');});
test('actual single driver rejects startup/preflight-only consumption and marks header-only unexercised',async()=>{await realSingle('published',{workloadConsumes:false});await realSingle('published',{endpoint:'headers'});});
test('actual multi controller and real worker preserve measured mode/consumption receipt',async()=>{
 const app=await receiver('published');
 try{
  const options=campaignOptions({MW_LOAD_FIXTURE_MODE:'smoke',MW_LOAD_FIXTURE_SECONDS:'1',MW_LOAD_FIXTURE_RATES:'10',MW_LOAD_FIXTURE_CONCURRENCY:'1',MW_LOAD_FIXTURE_ENDPOINTS:'identity',MW_LOAD_FIXTURE_REQUEST_INPUTS_EXPECT:'published'});
  const report=await runMultiCampaign({handoff:app.handoff,workers:1,options,startLeadMs:500});
  assert.equal(report.ordinaryRequestInputs.qualified,true);assert.ok(report.ordinaryRequestInputs.consumedFramesDelta>0);assert.equal(report.ordinaryRequestInputs.controlAuthenticationBound,0);
  assert.equal(app.order.at(-1),'scanner');assert.ok(app.phaseRequests()>0);
 }finally{await app.close();}
});
