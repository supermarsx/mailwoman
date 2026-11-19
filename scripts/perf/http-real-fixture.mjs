// Invoked only by the ignored Rust integration fixture. No arbitrary remote URL.
import {readFile,writeFile,mkdir} from 'node:fs/promises';
import {dirname} from 'node:path';
import {measure,markdownReport} from './http-load.mjs';

try{
 const handoff=JSON.parse(await readFile(process.argv[2],'utf8')),base=new URL(handoff.base);
 if(base.protocol!=='http:'||base.hostname!=='127.0.0.1'||base.pathname!=='/'||base.username||base.password)throw new Error('Fixture must use literal ephemeral loopback');
 const headers={cookie:handoff.cookie},csrf=handoff.cookie.split('; ').find(c=>c.startsWith('mw_csrf='));
 if(csrf)headers['x-csrf-token']=decodeURIComponent(csrf.slice(8));
 const session=await fetch(new URL('/jmap/session',base),{headers,redirect:'error',signal:AbortSignal.timeout(10000)});
 if(!session.ok)throw new Error(`Fixture session refused (${session.status})`);
 const account=(await session.json()).primaryAccounts?.['urn:ietf:params:jmap:mail'];
 if(!account)throw new Error('Fixture has no primary mail account');
 const identityResponse=await fetch(new URL('/api/me',base),{headers,redirect:'error',signal:AbortSignal.timeout(10000)});
 if(!identityResponse.ok)throw new Error(`Fixture identity refused (${identityResponse.status})`);
 const ownerAccount=(await identityResponse.json()).accountId;
 if(!ownerAccount)throw new Error('Fixture has no Mailwoman owner account');
 const request=(name,args)=>({name,path:'/jmap/api',method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({using:['urn:ietf:params:jmap:core','urn:ietf:params:jmap:mail'],methodCalls:[[name,{accountId:account,...args},'load']]}),validate:value=>value.methodResponses?.[0]?.[0]===name});
 const discovery=request('Email/query',{limit:2});
 const discoveryResponse=await fetch(new URL(discovery.path,base),{method:discovery.method,headers:{...headers,...discovery.headers},body:discovery.body,redirect:'error',signal:AbortSignal.timeout(10000)});
 if(!discoveryResponse.ok)throw new Error(`Fixture message discovery refused (${discoveryResponse.status})`);
 const discoveryBody=await discoveryResponse.json(),messageIds=discoveryBody.methodResponses?.[0]?.[1]?.ids;
 if(!discovery.validate(discoveryBody)||!Array.isArray(messageIds)||!messageIds.length||messageIds.some(id=>typeof id!=='string'||!id))throw new Error('Fixture requires seeded messages for Email/get');
 const sampledIds=messageIds.slice(0,2),emailGet=request('Email/get',{ids:sampledIds,properties:['id','subject','from','receivedAt','preview']});
 const emailMethodValid=emailGet.validate;emailGet.validate=value=>emailMethodValid(value)&&Array.isArray(value.methodResponses[0][1].list)&&sampledIds.every(id=>value.methodResponses[0][1].list.some(row=>row.id===id));
 const requests=[{name:'authenticated-identity',path:'/api/me'},request('Mailbox/get',{}),request('Email/query',{limit:25}),emailGet,{name:'concurrent-refresh-handler',path:'/api/mail/refresh',method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({accountId:ownerAccount}),validate:v=>v.accountId===ownerAccount&&v.upstream===false}];
 // Prove every endpoint works once before saturation. An always-broken method
 // must not hide behind successful identity reads in the aggregate counters.
 for(const spec of requests){
  const response=await fetch(new URL(spec.path,base),{method:spec.method??'GET',headers:{...headers,...spec.headers},...(spec.body!==undefined?{body:spec.body}:{}),redirect:'error',signal:AbortSignal.timeout(10000)});
  if(response.status!==200){await response.body?.cancel();throw new Error(`Fixture ${spec.name} preflight refused (${response.status})`);}
  const value=await response.json();if(spec.validate&&!spec.validate(value))throw new Error(`Fixture ${spec.name} preflight failed application validation`);
 }
 const rates=(process.env.MW_LOAD_FIXTURE_RATES??'100,500,1000').split(',').map(Number),seconds=Number(process.env.MW_LOAD_FIXTURE_SECONDS??1);
 if(!rates.length||rates.length>3||rates.some(n=>!Number.isFinite(n)||n<1||n>100000)||!Number.isFinite(seconds)||seconds<.1||seconds>5)throw new Error('Invalid bounded fixture campaign');
 const report={generatedAt:new Date().toISOString(),target:'ephemeral loopback Mailwoman router + local JMAP mock',scenario:'real authenticated middleware/HTTP/proxy with SQLite; simulated upstream; no TLS/reverse proxy',protection:handoff.protection?'rate/adaptive/user-agent observe; scanner enforce':'rate/adaptive/user-agent/scanner off',refreshSemantics:'authenticated refresh route only; proxy response upstream:false; no upstream reconciliation',sessionRateLimits:'deployment defaults retained; refusals reported',runtime:process.version,model:'bounded open-loop; generator drops reported; not production capacity',stages:[]};
 for(const [index,arrivalRate] of rates.entries()){
  report.stages.push({cachePreparation:index===0?'fresh server after sign-in/session discovery':'warmed by earlier stage',...await measure({base,headers,requests,concurrency:32,seconds,timeoutMs:5000,arrivalRate})});
 }
 const probe=await fetch(new URL('/.env',base),{redirect:'error',signal:AbortSignal.timeout(5000)});
 report.scannerProbeStatus=probe.status;await probe.body?.cancel();
 if(handoff.protection&&![403,429].includes(probe.status))throw new Error(`Scanner enforcement unexpectedly returned ${probe.status}`);
 const output=process.argv[3];await mkdir(dirname(output),{recursive:true});
 await writeFile(output,JSON.stringify(report,null,2)+'\n');await writeFile(output.replace(/\.json$/,'')+'.md',markdownReport(report));
 if(!report.stages.some(stage=>stage.accepted>0))throw new Error('No authenticated request succeeded');
}catch(error){console.error(error.message);process.exitCode=1;}
