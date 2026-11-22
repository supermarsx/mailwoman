import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtemp,mkdir,writeFile,readFile,rm,link,chmod,open} from 'node:fs/promises';
import {join,resolve,sep,basename} from 'node:path';
import {tmpdir} from 'node:os';
import {randomUUID} from 'node:crypto';
import {createServer} from 'node:http';
import {execFile} from 'node:child_process';
import {promisify} from 'node:util';
import {validateReady,readPrivate,privateJson,findReady,runClaimed,reportHash,CONTROL_LIMIT,HANDOFF_LIMIT} from './external-control.mjs';
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

test('largest supported representative handoff fits64MiB while exceeding the old32MiB cap',()=>{
 const identifier=index=>index.toString(16).padStart(8,'0')+'a'.repeat(56);
 const principals=Array.from({length:10000},(_,index)=>{
  const accountId=identifier(index),token=identifier(index+10000),csrfToken=identifier(index+20000);
  return {cookie:`mw_session=${token}; mw_csrf=${csrfToken}`,csrfToken,accountId,ownerAccountId:accountId,messageIds:Array.from({length:50},(_,uid)=>index.toString(16).padStart(8,'0')+uid.toString(16).padStart(8,'0')+'e'.repeat(48)),inboxId:identifier(index+30000),archiveId:identifier(index+40000),attachmentBlobId:`${identifier(index+50000)}.2`,attachmentSampleBytes:65536,bodySampleBytes:4096,clientAddress:`198.18.${Math.floor((index+1)/256)}.${(index+1)%256}`};
 });
 const bytes=Buffer.byteLength(JSON.stringify({base:'http://127.0.0.1:1234/',engineMode:'native',clientSimulation:{mode:'fixture-loopback-trusted-proxy',clientCount:10000},principals}));
 assert.equal(principals.length*principals[0].messageIds.length,500000);
 assert.ok(bytes>32*1024*1024,'maximum supported shape must exercise the former failure');
 assert.ok(bytes<=HANDOFF_LIMIT,'maximum supported shape must fit the shared handoff cap');
 assert.equal(CONTROL_LIMIT,65536,'ordinary controls keep their original64KiB cap');
});

test('Rust publisher and WebSocket reader share the bounded private handoff contract',async()=>{
 const publisher=await readFile('crates/mw-server/tests/common/request_load_control.rs','utf8');
 const reader=await readFile('crates/mw-server/examples/websocket-load.rs','utf8');
 assert.match(publisher,/HANDOFF_LIMIT:\s*u64\s*=\s*64\s*\*\s*1024\s*\*\s*1024/);
 assert.match(publisher,/CONTROL_LIMIT:\s*u64\s*=\s*65536/);
 assert.match(reader,/MAX_MANIFEST:\s*u64\s*=\s*64\s*\*\s*1024\s*\*\s*1024/);
 assert.equal(HANDOFF_LIMIT,64*1024*1024);
});

test('private handoff accepts exactly64MiB and refuses the next byte before executing the generator',async()=>{
 const f=await fixture();try{
  const path=join(f.directory,'handoff.private.json'),file=await open(path,'wx',0o600);
  try{await file.truncate(HANDOFF_LIMIT);}finally{await file.close();}
  assert.equal((await readPrivate(path,HANDOFF_LIMIT)).length,HANDOFF_LIMIT);
  await assert.rejects(readPrivate(path),error=>error.message==='Invalid external fixture file');
  await assert.rejects(privateJson(join(f.directory,'control.json'),{padding:'x'.repeat(CONTROL_LIMIT)}),/control record too large/);
  const larger=await open(path,'r+');try{await larger.truncate(HANDOFF_LIMIT+1);}finally{await larger.close();}
  let executed=false;
  await assert.rejects(runClaimed({directory:f.directory,ready:f.value,execute:async()=>{executed=true;return 0;}}),error=>error.message==='Invalid external fixture file');
  assert.equal(executed,false);
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
  await runClaimed({directory:f.directory,ready:f.value,execute:async()=>{await writeFile(join(f.directory,'report.json'),bytes,{mode:0o600});await writeFile(join(f.directory,'report.md'),'# Fixture report\n\nAccepted: 10.\n',{mode:0o600});return 0;}});
  let done=JSON.parse(await readFile(join(f.directory,'done.json'),'utf8'));assert.equal(done.status,'completed');assert.equal(done.reportHash,reportHash(bytes));assert.equal(done.reportBytes,bytes.length);
  await assert.rejects(runClaimed({directory:f.directory,ready:f.value,execute:async()=>{throw new Error('private-secret');}}),error=>!error.message.includes('private-secret'));
  done=JSON.parse(await readFile(join(f.directory,'done.json'),'utf8'));assert.equal(done.status,'error');assert.ok(!JSON.stringify(done).includes('private-secret'));
 }finally{await f.clean();}
});

test('missing, empty and invalid UTF8 Markdown never publish completed status',async()=>{
 for(const content of [null,Buffer.from('  \n'),Buffer.from([255,254])]){
  const f=await fixture();try{
   await privateJson(join(f.directory,'handoff.private.json'),{cookie:'private-secret'});
   await assert.rejects(runClaimed({directory:f.directory,ready:f.value,execute:async()=>{
    await writeFile(join(f.directory,'report.json'),'{"accepted":1}',{mode:0o600});
    if(content!==null)await writeFile(join(f.directory,'report.md'),content,{mode:0o600});
    return 0;
   }}));
   const done=JSON.parse(await readFile(join(f.directory,'done.json'),'utf8'));
   assert.equal(done.status,'error');assert.ok(!JSON.stringify(done).includes('private-secret'));
  }finally{await f.clean();}
 }
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


test('request-input expectation external control permits only the optional exact modes',()=>{
 const base=ready();assert.equal(validateReady(base).campaignEnv.MW_LOAD_FIXTURE_REQUEST_INPUTS_EXPECT,undefined);
 for(const mode of ['original','published'])assert.equal(validateReady({...base,campaignEnv:{MW_LOAD_FIXTURE_REQUEST_INPUTS_EXPECT:mode}}).campaignEnv.MW_LOAD_FIXTURE_REQUEST_INPUTS_EXPECT,mode);
 for(const value of ['',null,true,'composite','Original','published ',' published'])assert.throws(()=>validateReady({...base,campaignEnv:{MW_LOAD_FIXTURE_REQUEST_INPUTS_EXPECT:value}}));
 for(const key of ['MW_NATIVE_LOAD_REQUEST_INPUTS_MODE','MW_NATIVE_LOAD_ORDINARY_AUTH_MODE','MW_SERVER_KEY','MW_TEST_PG'])assert.throws(()=>validateReady({...base,campaignEnv:{[key]:'private-never-forward'}}));
});
test('request-input expectation Rust publisher retains exact whitelist and strict Unicode/mode guard',async()=>{
 const source=await readFile('crates/mw-server/tests/common/request_load_control.rs','utf8');
 const names=[...source.match(/for key in \[([\s\S]*?)\] \{/)[1].matchAll(/"([A-Z0-9_]+)"/g)].map(match=>match[1]);
 assert.equal(names.filter(key=>key==='MW_LOAD_FIXTURE_REQUEST_INPUTS_EXPECT').length,1);
 assert.ok(!names.includes('MW_NATIVE_LOAD_REQUEST_INPUTS_MODE'));assert.ok(!names.includes('MW_SERVER_KEY'));
 assert.match(source,/Err\(std::env::VarError::NotUnicode\(_\)\) if key == "MW_LOAD_FIXTURE_REQUEST_INPUTS_EXPECT"/);
 assert.match(source,/!matches!\(value.as_str\(\), "original" \| "published"\)/);
});
test('request-input expectation actually reaches external sidecar real child for both modes',async()=>{
 for(const mode of ['original','published']){
  const f=await fixture(),cookie='private-forwarding-cookie';let counter=0,measuring=false;
  const witness=()=>({mode,processImmutable:true,...Object.fromEntries(['ordinaryContexts','readyFrames','consumedFrames','hintAttempts','buildAttempts','completedBuilds','firstFenceAttempts','refreshFenceAttempts'].map(key=>[key,counter]))});
  const server=createServer(async(req,res)=>{
   if(req.url==='/__fixture-diagnostic-aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa'){
    if(req.headers['x-fixture-token']!=='private-diagnostic-token'){res.statusCode=403;res.end();return;}
    if(req.method==='POST'){measuring=true;res.end();return;}
    res.setHeader('content-type','application/json');res.end(JSON.stringify({ordinaryRequestInputs:witness()}));return;
   }
   if(req.url==='/.env'){res.statusCode=404;res.end();return;}
   if(req.headers.cookie!==cookie){res.statusCode=401;res.end();return;}
   if(mode==='published'&&measuring)counter++;
   res.setHeader('content-type','application/json');
   if(req.url==='/api/me'){res.end('{"accountId":"fixture-account"}');return;}
   let body='';for await(const chunk of req)body+=chunk;
   if(req.url==='/api/mail/refresh'){res.end('{"accountId":"fixture-account","upstream":false}');return;}
   const call=JSON.parse(body).methodCalls[0];res.end(JSON.stringify({methodResponses:[[call[0],{accountId:'fixture-account',ids:['fixture-message'],list:[{id:'fixture-message'}]},call[2]]]}));
  });await new Promise(done=>server.listen(0,'127.0.0.1',done));
  try{
   const handoff={base:`http://127.0.0.1:${server.address().port}`,engineMode:'proxy',protection:false,principals:[{cookie,accountId:'fixture-account',ownerAccountId:'fixture-account',messageIds:['fixture-message']}],ordinaryRequestInputs:witness(),diagnostics:{path:'/__fixture-diagnostic-aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa',token:'private-diagnostic-token'}};
   await privateJson(join(f.directory,'handoff.private.json'),handoff);
   await privateJson(join(f.directory,'ready.json'),{...f.value,campaignEnv:{MW_LOAD_FIXTURE_SECONDS:'0.1',MW_LOAD_FIXTURE_RATES:'10',MW_LOAD_FIXTURE_CONCURRENCY:'1',MW_LOAD_FIXTURE_ENDPOINTS:'identity',MW_LOAD_FIXTURE_REQUEST_INPUTS_EXPECT:mode}});
   const env=Object.fromEntries(['SystemRoot','WINDIR','TEMP','TMP','PATH'].filter(key=>process.env[key]!==undefined).map(key=>[key,process.env[key]]));
   const output=await promisify(execFile)(process.execPath,['scripts/perf/http-external-driver.mjs',`--root=${f.root}`,'--campaigns=1','--wait-seconds=10'],{env,timeout:15000});
   const report=JSON.parse(await readPrivate(join(f.directory,'report.json')));
   assert.equal(report.ordinaryRequestInputs.expectedMode,mode);assert.equal(report.ordinaryRequestInputs.qualified,true,'missing forwarding would leave receipt null');
   if(mode==='original')assert.equal(report.ordinaryRequestInputs.consumedFramesDelta,0);else assert.ok(report.ordinaryRequestInputs.consumedFramesDelta>0);
   assert.equal(JSON.parse(await readFile(join(f.directory,'done.json'),'utf8')).status,'completed');
   for(const text of [output.stdout,output.stderr,JSON.stringify(report)]){assert.ok(!text.includes(cookie));assert.ok(!text.includes('private-diagnostic-token'));}
  }finally{server.closeAllConnections();await new Promise(done=>server.close(done));await f.clean();}
 }
});
test('request-input expectation invalid external value refuses before child claim or HTTP',async()=>{
 const f=await fixture();let requests=0;const server=createServer((_req,res)=>{requests++;res.end();});await new Promise(done=>server.listen(0,'127.0.0.1',done));
 try{
  await privateJson(join(f.directory,'handoff.private.json'),{base:`http://127.0.0.1:${server.address().port}`,cookie:'private-invalid-cookie'});
  await privateJson(join(f.directory,'ready.json'),{...f.value,campaignEnv:{MW_LOAD_FIXTURE_REQUEST_INPUTS_EXPECT:'published '}});
  const env=Object.fromEntries(['SystemRoot','WINDIR','TEMP','TMP','PATH'].filter(key=>process.env[key]!==undefined).map(key=>[key,process.env[key]]));
  await assert.rejects(promisify(execFile)(process.execPath,['scripts/perf/http-external-driver.mjs',`--root=${f.root}`,'--campaigns=1','--wait-seconds=10'],{env,timeout:5000}),error=>error.code===1&&!error.stderr.includes('private-invalid-cookie'));
  assert.equal(requests,0);await assert.rejects(readFile(join(f.directory,'claim.json')),error=>error.code==='ENOENT');
 }finally{server.closeAllConnections();await new Promise(done=>server.close(done));await f.clean();}
});
