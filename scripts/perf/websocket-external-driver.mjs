// Fixture-only controller: keep private heartbeat/control alive until every
// workload finishes. No session credentials are arguments, stdout or reports.
import {join,dirname,resolve} from 'node:path';
import {fileURLToPath,pathToFileURL} from 'node:url';
import {lstat,realpath,writeFile} from 'node:fs/promises';
import {spawn} from 'node:child_process';
import {setTimeout as sleep} from 'node:timers/promises';
import {findReady,privateJson,readPrivate,runClaimed,REPORT_LIMIT} from './external-control.mjs';
import {prepareFixtureClients,startupReadinessReport} from './fixture-clients.mjs';

const scriptDirectory=dirname(fileURLToPath(import.meta.url));
const number=(value,fallback,min,max)=>{const parsed=Number(value??fallback);if(!Number.isInteger(parsed)||parsed<min||parsed>max)throw new Error('WebSocket controller options exceed fixture bounds');return parsed;};
export function options(args){
 if(!args.root||!args.binary||Object.keys(args).some(key=>!['root','binary','connections','hold-seconds','warm-seconds','ramp-per-second','max-pending','workers','with-http','wait-seconds'].includes(key)))throw new Error('Explicit private root and WebSocket binary are required');
 if(args['with-http']!==undefined&&!['true','false'].includes(args['with-http']))throw new Error('Choose explicit true or false for with-http');
 const result={root:args.root,binary:args.binary,connections:number(args.connections,10000,1,10000),hold:number(args['hold-seconds'],60,1,1800),warm:number(args['warm-seconds'],10,0,300),ramp:number(args['ramp-per-second'],100,1,5000),pending:number(args['max-pending'],128,1,512),workers:number(args.workers,2,1,32),wait:number(args['wait-seconds'],1800,10,3600),withHttp:args['with-http']==='true'};
 if(result.connections===10000&&result.hold<60)throw new Error('Ten-thousand-user campaigns require sixty steady seconds');
 return result;
}
function child(command,args,env,launch){
 const process=launch(command,args,{env,stdio:['ignore','ignore','ignore']});
 let settled=false;
 const done=new Promise(resolve=>{const finish=code=>{if(!settled){settled=true;resolve(code);}};process.once('error',()=>finish(-1));process.once('exit',code=>finish(code??-1));});
 return {done,stop:()=>{if(!settled)process.kill('SIGKILL');},settled:()=>settled};
}
export async function runWorkloads({directory,ready,config,launch=spawn}){
 const manifestPath=join(directory,ready.handoffFile),manifest=JSON.parse(await readPrivate(manifestPath,32*1024*1024));
 const clients=prepareFixtureClients(manifest.principals,manifest.clientSimulation);
 const startupReadiness=startupReadinessReport(manifest.startupReadiness,{required:manifest.engineMode==='native'});
 if(manifest.engineMode!=='native'||clients.clientCount<config.connections||clients.simulationMode!=='fixture-loopback-trusted-proxy')throw new Error('Independent multi-client WebSockets require the native proxy fixture');
 const websocketPath=join(directory,'websocket.report.json'),receiptPath=join(directory,'websocket.ready.json'),httpPath=join(directory,'http.report.json');
 const env={...process.env,...ready.campaignEnv},children=[];
 const deadline=Math.min(ready.expiresAtMs,Date.now()+(ready.deadlineSeconds+5)*1000);
 let deadlineExpired=false;
 const timer=setTimeout(()=>{deadlineExpired=true;for(const process of children)process.stop();},Math.max(1,deadline-Date.now()));
 try{
  const ws=child(config.binary,['--manifest',manifestPath,'--report',websocketPath,'--ready-file',receiptPath,'--protocol','push','--connections',String(config.connections),'--hold-seconds',String(config.hold),'--warm-seconds',String(config.warm),'--ramp-per-second',String(config.ramp),'--max-pending',String(config.pending),'--workers',String(config.workers)],env,launch);children.push(ws);
  let receipt=null,httpWindow=null,httpExit=null;
  if(config.withHttp){
   while(!receipt){
    try{receipt=JSON.parse(await readPrivate(receiptPath));}catch(error){if(error.code!=='ENOENT')throw error;}
    if(receipt)break;
    if(ws.settled()||deadlineExpired)throw new Error('WebSockets did not reach the verified steady phase');
    await sleep(50);
   }
   const now=Date.now();
   if(receipt.phase!=='steady-hold'||!Number.isSafeInteger(receipt.startedAtMs)||!Number.isSafeInteger(receipt.untilMs)||receipt.startedAtMs>now||receipt.untilMs<=now||receipt.authenticatedDistinctPrincipals!==config.connections||receipt.simulatedDistinctClients!==config.connections)throw new Error('Invalid WebSocket steady-phase receipt');
   httpWindow={startedAtMs:now,completedAtMs:null};
   const http=child(process.execPath,[join(scriptDirectory,'http-real-fixture.mjs'),manifestPath,httpPath],env,launch);children.push(http);
   httpExit=await http.done;httpWindow.completedAtMs=Date.now();
  }
  const wsExit=await ws.done;
  const websocket=JSON.parse(await readPrivate(websocketPath,REPORT_LIMIT));
  const http=config.withHttp?JSON.parse(await readPrivate(httpPath,REPORT_LIMIT)):null;
  const summary=websocket.summary??{};
  const simultaneousWindowVerified=Boolean(config.withHttp&&receipt&&httpWindow.startedAtMs>=summary.steadyStartedAtMs&&httpWindow.completedAtMs<=summary.steadyUntilMs&&summary.steadyEndedAtMs>=httpWindow.completedAtMs);
  const websocketPassed=wsExit===0&&summary.passed===true&&summary.simulatedDistinctClientsAtHoldEnd===config.connections;
  const httpPassed=!config.withHttp||httpExit===0&&!http.mutationStateTainted&&http.stages?.length>0&&http.stages.every(stage=>stage.errors===0&&stage.throttled===0&&stage.timeouts===0);
  const passed=!deadlineExpired&&websocketPassed&&httpPassed&&(!config.withHttp||simultaneousWindowVerified);
  return {schema:1,scenario:config.withHttp?'simultaneous multi-client HTTP and WebSocket fixture':'independent multi-client WebSocket fixture',passed,startupReadiness,simultaneousWindowVerified,httpWindow,distinctPhysicalGeneratorHostsMeasured:false,websocket,http,capacityTarget:{simultaneousAuthenticatedUsers:10000,successfulHttpRps:40000,met:Boolean(passed&&config.withHttp&&config.connections===10000&&summary.steadyHoldSeconds>=60&&summary.pushDeliveryVerified===true&&http.capacityTarget?.httpMultiUserTargetMet===true)}};
 }finally{
  clearTimeout(timer);for(const process of children)process.stop();
  await Promise.all(children.map(process=>process.done));
 }
}
async function main(){
 const args=Object.fromEntries(process.argv.slice(2).map(arg=>{const at=arg.indexOf('=');if(at<0)throw new Error('Use --name=value');return [arg.slice(2,at),arg.slice(at+1)];}));
 const config=options(args),root=await realpath(resolve(config.root)),metadata=await lstat(root);
 if(!metadata.isDirectory()||process.platform!=='win32'&&(metadata.uid!==process.getuid()||(metadata.mode&0o022)!==0))throw new Error('Scratch root must be owned and not writable by others');
 const started=Date.now();let found;
 while(!(found=await findReady(root))){if(Date.now()-started>config.wait*1000)throw new Error('WebSocket fixture readiness deadline exceeded');await sleep(250);}
 const {directory,ready}=found;
 await privateJson(join(directory,'claim.json'),{runId:ready.runId,claimedAtMs:Date.now()},{exclusive:true});
 await runClaimed({directory,ready,execute:async()=>{
  const report=await runWorkloads({directory,ready,config});
  await writeFile(join(directory,ready.outputFile),JSON.stringify(report,null,2)+'\n',{mode:0o600});
  await writeFile(join(directory,'report.md'),`# WebSocket fixture report\n\nPassed: ${report.passed}. Simultaneous HTTP window verified: ${report.simultaneousWindowVerified}. Combined 10,000-user / 40,000-RPS target met: ${report.capacityTarget.met}.\n\nSee the credential-free JSON for individual HTTP and WebSocket results. Simulated client addresses do not represent physical generator hosts.\n`,{mode:0o600});
  return report.passed?0:1;
 }});
 console.log('WebSocket fixture controller completed; credential-free report written');
}
if(process.argv[1]&&import.meta.url===pathToFileURL(resolve(process.argv[1])).href)main().catch(()=>{console.error('WebSocket fixture controller failed; inspect its private redacted reports');process.exitCode=1;});
