// Transport/load-generator verification only. This is NOT a Mailwoman backend.
import {createServer} from 'node:http';
import {writeFile,mkdir} from 'node:fs/promises';
import {resolve,dirname} from 'node:path';
import {pathToFileURL} from 'node:url';
import {measure,markdownReport} from './http-load.mjs';

export async function fixture({protection=false,delayMs=0}={}){
 let seen=0,cacheWarm=false;
 const server=createServer((req,res)=>{
  const finish=()=>{
   if(req.url==='/healthz'){res.end('ok');return;}
   if(req.headers.authorization!=='Bearer fixture-only'&&req.headers.cookie!=='session=fixture-only'){res.writeHead(401);res.end('denied');return;}
   if(req.url==='/error'){res.writeHead(503);res.end('fixture failure');return;}
   if(protection&&++seen%3===0){res.writeHead(429);res.end('fixture throttle');return;}
   res.setHeader('content-type','application/json');
   const cold=!cacheWarm;cacheWarm=true;
   res.end(JSON.stringify({ok:true,cold,kind:req.url==='/refresh'?'simulated-refresh':'simulated-read'}));
  };
  if(delayMs)setTimeout(finish,delayMs);else finish();
 });
 await new Promise(done=>server.listen(0,'127.0.0.1',done));
 return {base:`http://127.0.0.1:${server.address().port}`,reset(){cacheWarm=false;},async close(){server.closeAllConnections();await new Promise(done=>server.close(done));}};
}

export async function smoke(output='docs/perf/http-fixture-smoke.json'){
 const report={generatedAt:new Date().toISOString(),target:'isolated ephemeral loopback fixture',scenario:'simulated authenticated transport; NOT Mailwoman application capacity',model:'bounded open-loop fixed arrivals; includes generator drops',runtime:process.version,stages:[]};
 for(const protection of [false,true]){
  const host=await fixture({protection});
  try{
   for(const [index,arrivalRate] of [20000,50000,100000].entries()){
    if(index===0)host.reset();
    const result=await measure({base:host.base,concurrency:64,seconds:.3,timeoutMs:1000,arrivalRate,headers:{authorization:'Bearer fixture-only'},requests:[{name:'fixture-read',path:'/read',validate:body=>body.ok},{name:'fixture-concurrent-refresh',path:'/refresh',validate:body=>body.ok}]});
    report.stages.push({fixtureProtection:protection,cache:index===0?'initially cold then warm':'warm',...result});
   }
  }finally{await host.close();}
 }
 const path=resolve(output);await mkdir(dirname(path),{recursive:true});
 await writeFile(path,JSON.stringify(report,null,2)+'\n');await writeFile(path.replace(/\.json$/,'')+'.md',markdownReport(report));
 return report;
}
if(process.argv[1]&&import.meta.url===pathToFileURL(resolve(process.argv[1])).href){
 smoke(process.argv[2]).then(report=>console.log(JSON.stringify(report.stages.map(s=>({offered:s.arrivalRate,achieved:s.requestsPerSecond,accepted:s.accepted,dropped:s.droppedCapacity+s.droppedScheduler,protection:s.fixtureProtection})),null,2))).catch(error=>{console.error(error.message);process.exitCode=1;});
}
