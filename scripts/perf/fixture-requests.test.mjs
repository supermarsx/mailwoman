import test from 'node:test';
import assert from 'node:assert/strict';
import {prepareFixtureRequests} from './fixture-requests.mjs';
import {DEFAULT_HEADER_PROPERTIES,MINIMAL_HEADER_PROPERTIES} from './fixture-properties.mjs';

function principals(){return Array.from({length:4},(_,index)=>({cookie:`mw_session=private-${index}; mw_csrf=csrf-${index}`,accountId:`private-account-${index}`,ownerAccountId:`private-account-${index}`,messageIds:[`private-message-${index}-a`,`private-message-${index}-b`],clientAddress:`198.18.0.${index+1}`}));}

test('shared factory prepares distinct authenticated request groups without reporting or network side effects',()=>{
 const seeded=principals(),fixture=prepareFixtureRequests({principals:seeded,clientSimulation:{mode:'fixture-loopback-trusted-proxy',clientCount:4},engineMode:'native'});
 assert.equal(fixture.groups.mixed.length,5);assert.equal(fixture.groups.identity.length,1);assert.equal(fixture.firstRequests.length,5);assert.equal(fixture.clientModel.clientCount,4);
 assert.deepEqual(JSON.parse(fixture.firstRequests[3].body).methodCalls[0][1].properties,MINIMAL_HEADER_PROPERTIES);
 const visited=new Set();
 for(let sequence=0;sequence<20;sequence++){
  const group=fixture.groups.mixed,spec=group[sequence%group.length].materialize(sequence,{requestCount:group.length}),p=seeded[spec.principalIndex];visited.add(spec.principalIndex);
  assert.equal(spec.headers.cookie,p.cookie);assert.equal(spec.headers['x-forwarded-for'],p.clientAddress);assert.equal(spec.headers['x-csrf-token'],`csrf-${spec.principalIndex}`);
  if(spec.path==='/api/me'){assert.equal(spec.validate({accountId:p.accountId}),true);assert.equal(spec.validate({accountId:'foreign'}),false);}
  if(spec.path==='/jmap/api'){const call=JSON.parse(spec.body).methodCalls[0];assert.equal(call[1].accountId,p.accountId);assert.equal(spec.validate({methodResponses:[[call[0],{accountId:'foreign'},call[2]]]}),false);}
 }
 assert.equal(visited.size,4);
});

test('local representative partitions preserve the global workload proof and exact sample parameters',()=>{
 const seeded=principals().slice(0,2).map((p,index)=>({...p,inboxId:`private-inbox-${index}`,archiveId:`private-archive-${index}`,attachmentBlobId:`private-blob-${index}.2`,attachmentSampleBytes:65536,bodySampleBytes:4096}));
 const profile={kind:'representative',messagesPerPrincipal:2,bodySampleBytes:4096,attachmentSampleBytes:65536,seededMessages:20000};
 const input={principals:seeded,clientSimulation:{mode:'fixture-loopback-trusted-proxy',clientCount:2},engineMode:'native',representative:true,workloadProfile:profile};
 const prepared=prepareFixtureRequests(input);
 assert.equal(prepared.groups.mixed.length,5);assert.equal(prepared.groups.representative.length,10);assert.equal(prepared.firstRequests.length,10);assert.equal(profile.seededMessages,20000);
 assert.deepEqual(JSON.parse(prepared.firstRequests[3].body).methodCalls[0][1].properties,DEFAULT_HEADER_PROPERTIES);
 assert.ok(prepared.groups.representative.every(spec=>spec.serializePrincipal===true));
 assert.throws(()=>prepareFixtureRequests({...input,principals:seeded.map(p=>({...p,attachmentSampleBytes:4}))}),/global profile/);
 assert.throws(()=>prepareFixtureRequests({...input,engineMode:'proxy'}));
 assert.throws(()=>prepareFixtureRequests({...input,principals:[seeded[0],seeded[0]]}));
});
