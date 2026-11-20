import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtemp,writeFile,rm} from 'node:fs/promises';
import {join} from 'node:path';
import {tmpdir} from 'node:os';
import {spawn} from 'node:child_process';
import {options,runWorkloads} from './websocket-external-driver.mjs';

test('WebSocket controller requires explicit isolated inputs and bounded parameters',()=>{
 for(const args of [{},{root:'x'},{root:'x',binary:'ws',connections:'10001'},{root:'x',binary:'ws',connections:'10000','hold-seconds':'1'},{root:'x',binary:'ws','with-http':'maybe'},{root:'x',binary:'ws',secret:'ignored'}])assert.throws(()=>options(args));
 assert.equal(options({root:'x',binary:'ws',connections:'100','hold-seconds':'1'}).withHttp,false);
});

async function campaign({http=false,overlap=true,simulate=true}={}){
 const directory=await mkdtemp(join(tmpdir(),'mw-ws-controller-'));
 try{
  const handoff={engineMode:'native',startupReadiness:{verified:true,state:'ready',rebuildRunning:false,startupSeconds:1,indexWaitSeconds:1,seededMessages:4,expectedIndexDocuments:4,indexDocuments:4,zeroAccessExcludedMessages:0,rebuildDone:4,rebuildTotal:4},clientSimulation:simulate?{mode:'fixture-loopback-trusted-proxy',clientCount:2}:undefined,principals:[0,1].map(i=>({cookie:`mw_session=private-${i}`,accountId:`private-account-${i}`,clientAddress:simulate?`198.18.0.${i+1}`:undefined}))};
  await writeFile(join(directory,'handoff.private.json'),JSON.stringify(handoff),{mode:0o600});
  const fakeWs=join(directory,'fake-ws.mjs'),fakeHttp=join(directory,'fake-http.mjs');
  await writeFile(fakeWs,`import {writeFile} from 'node:fs/promises';const args=Object.fromEntries(Array.from({length:(process.argv.length-2)/2},(_,i)=>[process.argv[i*2+2],process.argv[i*2+3]]));const start=Date.now(),until=start+${overlap?5000:1};await writeFile(args['--ready-file'],JSON.stringify({schema:1,phase:'steady-hold',startedAtMs:start,untilMs:until,authenticatedDistinctPrincipals:2,simulatedDistinctClients:2}),{mode:0o600});await new Promise(r=>setTimeout(r,1000));await writeFile(args['--report'],JSON.stringify({summary:{passed:true,simulatedDistinctClientsAtHoldEnd:2,steadyStartedAtMs:start,steadyUntilMs:until,steadyEndedAtMs:Date.now(),steadyHoldSeconds:1,pushDeliveryVerified:true}}),{mode:0o600});`);
  await writeFile(fakeHttp,`import {writeFile} from 'node:fs/promises';await new Promise(r=>setTimeout(r,20));await writeFile(process.argv[3],JSON.stringify({stages:[{errors:0,throttled:0,timeouts:0}],capacityTarget:{httpMultiUserTargetMet:false}}),{mode:0o600});`);
  const config=options({root:directory,binary:'fake-websocket-binary',connections:'2','hold-seconds':'1','with-http':String(http)});
  const launch=(command,args,opts)=>command===config.binary?spawn(process.execPath,[fakeWs,...args],opts):spawn(process.execPath,[fakeHttp,...args.slice(1)],opts);
  return await runWorkloads({directory,ready:{handoffFile:'handoff.private.json',deadlineSeconds:10,expiresAtMs:Date.now()+20000,campaignEnv:{}},config,launch});
 }finally{await rm(directory,{recursive:true,force:true});}
}
test('controller keeps a real child alive for WS-only and redacts fixture credentials',async()=>{
 const result=await campaign();assert.equal(result.passed,true);assert.equal(result.http,null);assert.equal(result.simultaneousWindowVerified,false);assert.equal(result.capacityTarget.met,false);
 assert.ok(!JSON.stringify(result).includes('private-'));assert.ok(!JSON.stringify(result).includes('198.18.'));
});
test('combined controller starts HTTP only inside the authenticated WS steady window',async()=>{
 const result=await campaign({http:true});assert.equal(result.passed,true);assert.equal(result.simultaneousWindowVerified,true);assert.ok(result.httpWindow.completedAtMs>=result.httpWindow.startedAtMs);assert.equal(result.capacityTarget.met,false);
});
test('expired steady windows and missing simulation cannot establish simultaneous load',async()=>{
 await assert.rejects(campaign({http:true,overlap:false}),/steady-phase receipt/);
 await assert.rejects(campaign({simulate:false}),/native proxy fixture/);
});
