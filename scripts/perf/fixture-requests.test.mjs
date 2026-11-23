import test from 'node:test';
import assert from 'node:assert/strict';
import {prepareFixtureRequests,CLIENT_HOT_GROUPS} from './fixture-requests.mjs';
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

test('client-hot groups copy the real client request shapes, validate responses and leave existing groups unchanged',()=>{
 const seeded=principals().map((p,index)=>({...p,inboxId:`inbox-${index}`}));
 const input={principals:seeded,clientSimulation:{mode:'fixture-loopback-trusted-proxy',clientCount:4},engineMode:'native'};
 const plain=prepareFixtureRequests(input),fixture=prepareFixtureRequests(input);
 assert.deepEqual(Object.keys(CLIENT_HOT_GROUPS),['jmap-session','list-batch','changes','account-polls']);
 assert.equal(fixture.requests.length,5);assert.equal(fixture.firstRequests.length,5);assert.equal(fixture.groups.mixed.length,5);
 assert.deepEqual(fixture.groups.mixed.map(spec=>spec.name),['authenticated-identity','Mailbox/get','Email/query','Email/get','concurrent-refresh-handler']);
 for(const [group,names] of Object.entries(CLIENT_HOT_GROUPS))assert.deepEqual(fixture.groups[group].map(spec=>spec.name),names);
 const at=(group,i=0)=>fixture.groups[group][i].materialize(0,{requestCount:fixture.groups[group].length}),p=seeded[0],acct=p.accountId;
 const session=at('jmap-session');assert.equal(session.path,'/jmap/session');assert.equal(session.method??'GET','GET');assert.equal(session.headers.cookie,p.cookie);
 assert.equal(session.validate({apiUrl:'/jmap/api',accounts:{[acct]:{}}}),true);assert.equal(session.validate({apiUrl:'/jmap/api',accounts:{other:{}}}),false);assert.equal(session.validate({accounts:{[acct]:{}}}),false);assert.equal(session.validate(null),false);
 const list=at('list-batch'),body=JSON.parse(list.body);
 assert.deepEqual(body.methodCalls.map(call=>[call[0],call[2]]),[['Email/query','q'],['Email/get','g'],['Email/query','unread']]);
 assert.deepEqual(body.methodCalls[0][1].filter,{inMailbox:'inbox-0'});assert.equal(body.methodCalls[0][1].limit,2);assert.equal(body.methodCalls[0][1].calculateTotal,true);
 assert.deepEqual(body.methodCalls[1][1]['#ids'],{resultOf:'q',name:'Email/query',path:'/ids'});assert.deepEqual(body.methodCalls[1][1].properties,DEFAULT_HEADER_PROPERTIES);
 assert.deepEqual(body.methodCalls[2][1],{accountId:acct,filter:{inMailbox:'inbox-0',notKeyword:'$seen'},limit:0,calculateTotal:true});
 const ids=p.messageIds,good=[['Email/query',{accountId:acct,ids,queryState:'1'},'q'],['Email/get',{accountId:acct,list:ids.map(id=>({id}))},'g'],['Email/query',{accountId:acct,ids:[],total:1},'unread']];
 assert.equal(list.validate({methodResponses:good}),true);
 assert.equal(list.validate({methodResponses:good.slice(0,2)}),false);assert.equal(list.validate({methodResponses:[good[0],['Email/get',{accountId:acct,list:[{id:'x'},{id:'y'}]},'g'],good[2]]}),false);
 assert.equal(list.validate({methodResponses:[good[0],good[1],['Email/query',{accountId:'foreign',ids:[],total:1},'unread']]}),false);
 const changes=at('changes'),calls=JSON.parse(changes.body).methodCalls;
 assert.deepEqual(calls.map(call=>[call[0],call[2]]),[['Email/query','base'],['Email/queryChanges','delta-query'],['Email/changes','delta-emails']]);
 assert.deepEqual(calls[1][1]['#sinceQueryState'],{resultOf:'base',name:'Email/query',path:'/queryState'});assert.deepEqual(calls[2][1]['#sinceState'],{resultOf:'base',name:'Email/query',path:'/queryState'});assert.equal(calls[1][1].maxChanges,1000);assert.equal(calls[2][1].maxChanges,1000);
 const quiet=[['Email/query',{accountId:acct,ids:[ids[0]],queryState:'7'},'base'],['Email/queryChanges',{accountId:acct,oldQueryState:'7',newQueryState:'7',removed:[],added:[]},'delta-query'],['Email/changes',{accountId:acct,oldState:'7',newState:'7',hasMoreChanges:false,created:[],updated:[],destroyed:[]},'delta-emails']];
 assert.equal(changes.validate({methodResponses:quiet}),true);
 const moved=structuredClone(quiet);moved[2][1].updated=['x'];assert.equal(changes.validate({methodResponses:moved}),false);
 const advanced=structuredClone(quiet);advanced[1][1].newQueryState='8';assert.equal(changes.validate({methodResponses:advanced}),false);
 const polls=fixture.groups['account-polls'].map((spec,i)=>at('account-polls',i)),[prefs,appearance,activity]=polls;
 assert.equal(prefs.path,'/api/account/preferences');assert.equal(prefs.validate({version:1,preferences:{}}),true);assert.equal(prefs.validate({version:2,preferences:{}}),false);assert.equal(prefs.validate({version:1,preferences:[]}),false);
 assert.equal(appearance.path,`/api/account/appearance/customization?accountId=${acct}`);assert.equal(appearance.validate({ownerId:acct,document:{version:1},compiledCss:'',lockedFields:[]}),true);assert.equal(appearance.validate({ownerId:'foreign',document:{version:1},compiledCss:'',lockedFields:[]}),false);
 assert.equal(activity.method,'POST');assert.equal(activity.path,'/api/account/session/activity');assert.equal(activity.body,undefined);assert.equal(activity.headers['x-csrf-token'],'csrf-0');assert.equal(activity.validate({ok:true}),true);assert.equal(activity.validate({ok:false}),false);
 assert.ok(!JSON.stringify(plain.readIntent).includes('jmap-session'),'read-intent shapes still describe only the original requests');
 assert.ok(fixture.groups['account-polls'].every(spec=>spec.statefulMutation===undefined&&spec.serializePrincipal===undefined));
});
