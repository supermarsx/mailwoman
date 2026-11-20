import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtemp,mkdir,writeFile,readFile,rm,link,chmod} from 'node:fs/promises';
import {join,resolve,sep,basename} from 'node:path';
import {tmpdir} from 'node:os';
import {randomUUID} from 'node:crypto';
import {createServer} from 'node:http';
import {execFile} from 'node:child_process';
import {promisify} from 'node:util';
import {validateReady,readPrivate,privateJson,findReady,runClaimed,reportHash} from './external-control.mjs';
const ready=()=>({version:1,runId:randomUUID(),expiresAtMs:Date.now()+120000,deadlineSeconds:90,handoffFile:'handoff.private.json',outputFile:'report.json',campaignEnv:{}});
async function fixture(){
 const root=await mkdtemp(join(tmpdir(),'mw-load-control-')),value=ready(),directory=join(root,`run-${value.runId}`);await mkdir(directory,{mode:0o700});
 return {root,directory,value,async clean(){const path=resolve(root);assert.ok(path.startsWith(resolve(tmpdir())+sep)&&basename(path).startsWith('mw-load-control-'));await rm(path,{recursive:true,force:true});}};
}
test('control validation rejects expiry, traversal, unknown env and invalid deadlines',()=>{
 const base=ready();assert.equal(validateReady(base),base);
 for(const patch of [{expiresAtMs:Date.now()-1},{expiresAtMs:Date.now()+2100000},{handoffFile:'../cookie'},{outputFile:'/outside'},{campaignEnv:{MW_SERVER_KEY:'secret'}},{deadlineSeconds:1801},{runId:'../../outside'}])assert.throws(()=>validateReady({...base,...patch}));
});
test('private reads reject hardlinks, oversized records and public Unix permissions',async()=>{
 const f=await fixture();try{
  const path=join(f.directory,'private');await writeFile(path,'private',{mode:0o600});
  await assert.rejects(readPrivate(path,2));
  await link(path,join(f.directory,'alias'));await assert.rejects(readPrivate(path));
  await rm(join(f.directory,'alias'));if(process.platform!=='win32'){await chmod(path,0o644);await assert.rejects(readPrivate(path));}
 }finally{await f.clean();}
});
test('discovery claims no stale run and refuses directory/run identity mismatch',async()=>{
 const f=await fixture();try{
  await privateJson(join(f.directory,'ready.json'),f.value);assert.equal((await findReady(f.root)).ready.runId,f.value.runId);
  await privateJson(join(f.directory,'claim.json'),{runId:f.value.runId},{exclusive:true});assert.equal(await findReady(f.root),null);
  await rm(join(f.directory,'claim.json'));await privateJson(join(f.directory,'ready.json'),{...f.value,expiresAtMs:Date.now()-1});assert.equal(await findReady(f.root),null);
  await privateJson(join(f.directory,'ready.json'),{...f.value,runId:randomUUID()});await assert.rejects(findReady(f.root));
 }finally{await f.clean();}
});
test('completion binds exact report bytes and generator failures stay redacted',async()=>{
 const f=await fixture();try{
  await privateJson(join(f.directory,'handoff.private.json'),{cookie:'private-secret'});
  const bytes=Buffer.from('{"accepted":10}');
  await runClaimed({directory:f.directory,ready:f.value,execute:async()=>{await writeFile(join(f.directory,'report.json'),bytes,{mode:0o600});return 0;}});
  let done=JSON.parse(await readFile(join(f.directory,'done.json'),'utf8'));assert.equal(done.status,'completed');assert.equal(done.reportHash,reportHash(bytes));assert.equal(done.reportBytes,bytes.length);
  await assert.rejects(runClaimed({directory:f.directory,ready:f.value,execute:async()=>{throw new Error('private-secret');}}),error=>!error.message.includes('private-secret'));
  done=JSON.parse(await readFile(join(f.directory,'done.json'),'utf8'));assert.equal(done.status,'error');assert.ok(!JSON.stringify(done).includes('private-secret'));
 }finally{await f.clean();}
});

test('sidecar entrypoint runs the real Node driver through private handoff and completion',async()=>{
 const f=await fixture(),cookie='private-fixture-cookie';
 const server=createServer(async(req,res)=>{
  if(req.url==='/.env'){res.writeHead(404);res.end();return;}
  if(req.headers.cookie!==cookie){res.writeHead(401);res.end();return;}
  res.setHeader('content-type','application/json');
  if(req.url==='/api/me'){res.end('{"accountId":"fixture-account"}');return;}
  let body='';for await(const chunk of req)body+=chunk;
  if(req.url==='/api/mail/refresh'){res.end('{"accountId":"fixture-account","upstream":false}');return;}
  const call=JSON.parse(body).methodCalls[0];res.end(JSON.stringify({methodResponses:[[call[0],{accountId:'fixture-account',ids:['fixture-message'],list:[{id:'fixture-message'}]},call[2]]]}));
 });
 await new Promise(done=>server.listen(0,'127.0.0.1',done));
 try{
  const handoff={base:`http://127.0.0.1:${server.address().port}`,engineMode:'proxy',protection:false,principals:[{cookie,accountId:'fixture-account',ownerAccountId:'fixture-account',messageIds:['fixture-message']}]};
  await privateJson(join(f.directory,'handoff.private.json'),handoff);
  await privateJson(join(f.directory,'ready.json'),{...f.value,campaignEnv:{MW_LOAD_FIXTURE_SECONDS:'0.1',MW_LOAD_FIXTURE_RATES:'100',MW_LOAD_FIXTURE_CONCURRENCY:'2'}});
  const output=await promisify(execFile)(process.execPath,['scripts/perf/http-external-driver.mjs',`--root=${f.root}`,'--campaigns=1','--wait-seconds=10'],{timeout:15000});
  assert.ok(!output.stdout.includes(cookie));
  const done=JSON.parse(await readFile(join(f.directory,'done.json'),'utf8'));assert.equal(done.status,'completed');
  const report=await readPrivate(join(f.directory,'report.json'));assert.equal(done.reportHash,reportHash(report));
  assert.ok(JSON.parse(report).stages[0].accepted>0);assert.ok(!report.toString().includes(cookie));
 }finally{server.closeAllConnections();await new Promise(done=>server.close(done));await f.clean();}
});
