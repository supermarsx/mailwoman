// Sidecar entrypoint: same private network namespace, independent cgroup.
import {join,resolve} from 'node:path';
import {lstat,realpath} from 'node:fs/promises';
import {spawn} from 'node:child_process';
import {setTimeout as sleep} from 'node:timers/promises';
import {findReady,privateJson,runClaimed} from './external-control.mjs';
import {generatorWorkers,httpGeneratorLaunch} from './http-generator-launch.mjs';

const args=Object.fromEntries(process.argv.slice(2).map(arg=>{const at=arg.indexOf('=');if(at<0)throw new Error('Use --name=value');return [arg.slice(2,at),arg.slice(at+1)];}));
try{
 if(!args.root||Object.keys(args).some(key=>!['root','campaigns','wait-seconds','workers'].includes(key)))throw new Error('Explicit --root required');
 const workers=generatorWorkers(args.workers??process.env.MW_LOAD_GENERATOR_WORKERS??1);
 const root=await realpath(resolve(args.root)),rootMeta=await lstat(root);
 if(!rootMeta.isDirectory()||process.platform!=='win32'&&(rootMeta.uid!==process.getuid()||(rootMeta.mode&0o022)!==0))throw new Error('Scratch root must be owned and not writable by other users');
 const campaigns=Number(args.campaigns??1),wait=Number(args['wait-seconds']??1800);
 if(!Number.isInteger(campaigns)||campaigns<1||campaigns>2||!Number.isFinite(wait)||wait<10||wait>3600)throw new Error('Invalid sidecar bounds');
 let waitingSince=Date.now(),completed=0;
 while(completed<campaigns){
  const found=await findReady(root);if(!found){if(Date.now()-waitingSince>wait*1000)throw new Error('External fixture readiness deadline exceeded');await sleep(250);continue;}
  const {directory,ready}=found;
  try{await privateJson(join(directory,'claim.json'),{runId:ready.runId,claimedAtMs:Date.now()},{exclusive:true});}catch(error){if(error.code==='EEXIST')continue;throw error;}
  await runClaimed({directory,ready,execute:async()=>{
   const launch=httpGeneratorLaunch({directory,ready,workers});
   const child=spawn(process.execPath,[launch.script,...launch.args],{env:{...process.env,...ready.campaignEnv},stdio:['ignore','ignore','ignore']});
   const timer=setTimeout(()=>child.kill('SIGKILL'),Math.max(1,Math.min((ready.deadlineSeconds+5)*1000,ready.expiresAtMs-Date.now())));
   const result=await new Promise(resolve=>{child.once('error',()=>resolve(-1));child.once('exit',code=>resolve(code??-1));});clearTimeout(timer);return result;
  }});
  completed++;waitingSince=Date.now();console.log(JSON.stringify({campaign:completed,status:'completed'}));
 }
}catch(error){console.error(error.message);process.exitCode=1;}
