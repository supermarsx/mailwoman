import test from 'node:test';
import assert from 'node:assert/strict';
import {createServer} from 'node:http';
import {once} from 'node:events';
import {verifyFixtureIsolation} from './fixture-isolation.mjs';

const principals=[{accountId:'account-a',messageIds:['mail-a']},{accountId:'account-b',messageIds:['mail-b']}];
const clients=[{headers:{cookie:'private-a','x-csrf-token':'csrf-a'}},{headers:{cookie:'private-b','x-csrf-token':'csrf-b'}}];
async function serverFor(transform){
 const observed=[];
 const server=createServer(async(req,res)=>{
  const chunks=[];for await(const chunk of req)chunks.push(chunk);
  const payload=JSON.parse(Buffer.concat(chunks)),[method,args,tag]=payload.methodCalls[0];
  observed.push({method,args,cookie:req.headers.cookie,csrf:req.headers['x-csrf-token']});
  const own=req.headers.cookie==='private-a'?'mail-a':'mail-b',foreign=own==='mail-a'?'mail-b':'mail-a';
  const result={accountId:args.accountId,list:[{id:own},{id:own}],notFound:[foreign]};
  res.writeHead(200,{'content-type':'application/json'});
  res.end(JSON.stringify({methodResponses:[[method,transform?transform(result,foreign):result,tag]]}));
 });
 server.listen(0,'127.0.0.1');await once(server,'listening');
 return {base:`http://127.0.0.1:${server.address().port}`,observed,close:async()=>{server.closeAllConnections();await new Promise(resolve=>server.close(resolve));}};
}
test('real HTTP checks both account boundaries with each session and preserves owned duplicates',async()=>{
 const fixture=await serverFor();try{
  const proof=await verifyFixtureIsolation({...fixture,principals,clients});
  assert.deepEqual(proof,{verified:true,status:'verified',requests:2,directions:2});
  assert.deepEqual(fixture.observed.map(row=>row.cookie),['private-a','private-b']);
  assert.deepEqual(fixture.observed.map(row=>row.csrf),['csrf-a','csrf-b']);
  assert.deepEqual(fixture.observed[0].args.ids,['mail-a','mail-b','mail-a']);
  assert.equal(fixture.observed[0].args.mailwomanBodyUse,false);
  for(const secret of ['private-a','csrf-a','account-a','mail-a'])assert.equal(JSON.stringify(proof).includes(secret),false);
 }finally{await fixture.close();}
});
test('real HTTP foreign-mail disclosure fails without printing returned mail or credentials',async()=>{
 const fixture=await serverFor((result,foreign)=>({...result,list:[...result.list,{id:foreign,subject:'private subject'}],notFound:[]}));
 try{await assert.rejects(verifyFixtureIsolation({...fixture,principals,clients}),error=>error.message==='Fixture account isolation was not observed');}
 finally{await fixture.close();}
});
test('missing owned rows, wrong account echoes and malformed missing lists cannot prove isolation',async()=>{
 for(const transform of [result=>({...result,list:[]}),result=>({...result,accountId:'other'}),result=>({...result,notFound:[]})]){
  const fixture=await serverFor(transform);try{await assert.rejects(verifyFixtureIsolation({...fixture,principals,clients}));}finally{await fixture.close();}
 }
});
test('one principal is explicitly unverified, external targets are refused and responses stay bounded',async()=>{
 let called=false;const request=async()=>{called=true;return new Response('x'.repeat(1024*1024+1));};
 const local={base:'http://127.0.0.1:12345',principals,clients,request};
 assert.deepEqual(await verifyFixtureIsolation({...local,principals:principals.slice(0,1),clients:clients.slice(0,1)}),{verified:false,status:'insufficient-principals',requests:0});
 assert.equal(called,false);
 await assert.rejects(verifyFixtureIsolation({...local,base:'https://example.test'}));assert.equal(called,false);
 await assert.rejects(verifyFixtureIsolation(local));assert.equal(called,true);
});
