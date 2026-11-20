import {lstat,open,rename,readdir} from 'node:fs/promises';
import {join,dirname,basename} from 'node:path';
import {randomUUID,createHash} from 'node:crypto';
export const CONTROL_LIMIT=65536,REPORT_LIMIT=64*1024*1024;
const uuid=/^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/;
export function validateReady(value,now=Date.now()){
 if(value?.version!==1||!uuid.test(value.runId??'')||!Number.isSafeInteger(value.expiresAtMs)||value.expiresAtMs<=now||value.expiresAtMs>now+2000000)throw new Error('Invalid or expired external fixture control');
 if(value.handoffFile!=='handoff.private.json'||value.outputFile!=='report.json')throw new Error('Invalid external fixture filenames');
 if(!Number.isInteger(value.deadlineSeconds)||value.deadlineSeconds<10||value.deadlineSeconds>1800)throw new Error('Invalid external fixture deadline');
 const allowed=new Set(['MW_LOAD_FIXTURE_MODE','MW_LOAD_FIXTURE_LONG','MW_LOAD_FIXTURE_SECONDS','MW_LOAD_FIXTURE_WARMUP_SECONDS','MW_LOAD_FIXTURE_DEADLINE_SECONDS','MW_LOAD_FIXTURE_CONCURRENCY','MW_LOAD_FIXTURE_ENDPOINTS','MW_LOAD_FIXTURE_RATES','MW_LOAD_FIXTURE_TIMEOUT_MS','MW_LOAD_FIXTURE_MAX_ERROR_RATE','MW_LOAD_FIXTURE_MAX_THROTTLE_RATE','MW_LOAD_FIXTURE_MAX_P99_MS','MW_LOAD_FIXTURE_ALLOW_OVERLOAD','MW_LOAD_FIXTURE_ADAPTIVE']);
 if(!value.campaignEnv||Object.entries(value.campaignEnv).some(([key,text])=>!allowed.has(key)||typeof text!=='string'||text.length>256))throw new Error('Invalid external fixture environment');
 return value;
}
export async function readPrivate(path,limit=CONTROL_LIMIT){
 const before=await lstat(path);if(!before.isFile()||before.isSymbolicLink()||before.nlink!==1||before.size>limit)throw new Error('Invalid external fixture file');
 if(process.platform!=='win32'&&((before.mode&0o077)!==0||before.uid!==process.getuid()))throw new Error('External fixture file is not private to this user');
 const file=await open(path,'r');
 try{const after=await file.stat();if(after.ino!==before.ino||after.dev!==before.dev||after.size>limit)throw new Error('External fixture file changed');return await file.readFile();}finally{await file.close();}
}
export async function privateJson(path,value,{exclusive=false}={}){
 const bytes=Buffer.from(JSON.stringify(value));if(bytes.length>CONTROL_LIMIT)throw new Error('External control record too large');
 if(exclusive){const file=await open(path,'wx',0o600);try{await file.writeFile(bytes);await file.sync();}finally{await file.close();}return;}
 const temporary=join(dirname(path),`.${basename(path)}.${randomUUID()}.tmp`),file=await open(temporary,'wx',0o600);
 try{await file.writeFile(bytes);await file.sync();}finally{await file.close();}
 await rename(temporary,path);
}
export async function findReady(root,now=Date.now()){
 const entries=await readdir(root,{withFileTypes:true});if(entries.length>1024)throw new Error('External scratch root contains too many entries');
 for(const entry of entries){
  if(!entry.isDirectory()||!entry.name.startsWith('run-')||!uuid.test(entry.name.slice(4)))continue;
  const directory=join(root,entry.name),metadata=await lstat(directory);
  if(metadata.isSymbolicLink()||process.platform!=='win32'&&((metadata.mode&0o077)!==0||metadata.uid!==process.getuid()))throw new Error('External run directory is not private');
  try{await lstat(join(directory,'claim.json'));continue;}catch(error){if(error.code!=='ENOENT')throw error;}
  let ready;try{ready=JSON.parse(await readPrivate(join(directory,'ready.json')));}catch(error){if(error.code==='ENOENT')continue;throw error;}
  if(ready.expiresAtMs<=now)continue;
  validateReady(ready,now);if(entry.name!==`run-${ready.runId}`)throw new Error('External run identity mismatch');
  return {directory,ready};
 }
 return null;
}
export function reportHash(bytes){return createHash('sha256').update(bytes).digest('hex');}
export async function runClaimed({directory,ready,execute}){
 validateReady(ready);
 await readPrivate(join(directory,ready.handoffFile),32*1024*1024);
 const heartbeat=()=>privateJson(join(directory,'heartbeat.json'),{runId:ready.runId,atMs:Date.now()});
 await heartbeat();const timer=setInterval(()=>{void heartbeat().catch(()=>{});},5000);
 try{
  const exitCode=await execute();
  if(exitCode!==0){await privateJson(join(directory,'done.json'),{runId:ready.runId,status:'error',reason:'generator-failed',exitCode:Number.isInteger(exitCode)?exitCode:-1});throw new Error('External generator failed');}
  const bytes=await readPrivate(join(directory,ready.outputFile),REPORT_LIMIT);
  JSON.parse(bytes.toString('utf8'));
  await privateJson(join(directory,'done.json'),{runId:ready.runId,status:'completed',reportHash:reportHash(bytes),reportBytes:bytes.length});
 }catch{
  await privateJson(join(directory,'done.json'),{runId:ready.runId,status:'error',reason:'generator-failed'}).catch(()=>{});
  throw new Error('External generator failed; no secrets copied to logs');
 }finally{clearInterval(timer);}
}
