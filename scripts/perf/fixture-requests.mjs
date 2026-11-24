// Private in-process request templates, shared by single and partitioned drivers.
// No network, main entrypoint, preflight or reporting side effects.
import {nativeReadIntentHeaders,nativeReadContract} from './native-read-intent.mjs';
import {prepareFixtureClients,fixturePrincipalIndex} from './fixture-clients.mjs';
import {representativeSpecs,goalMixSchedule,goalMixWriteSpecs,GOAL_MIX_NAME,GOAL_MIX_CYCLE} from './representative-workload.mjs';
import {DEFAULT_HEADER_PROPERTIES,MINIMAL_HEADER_PROPERTIES} from './fixture-properties.mjs';

export function jmapFixtureRequest(p,name,args,{native=false}={}){
 const body={using:['urn:ietf:params:jmap:core','urn:ietf:params:jmap:mail'],methodCalls:[[name,{accountId:p.accountId,...args},'load']]};
 return {name,path:'/jmap/api',method:'POST',headers:{'content-type':'application/json',...(native?nativeReadIntentHeaders(body):{})},body:JSON.stringify(body),validate:value=>value.methodResponses?.[0]?.[0]===name&&value.methodResponses?.[0]?.[1]?.accountId===p.accountId};
}

const MAIL_USING=['urn:ietf:params:jmap:core','urn:ietf:params:jmap:mail'];
// Client-hot routes the real web client calls on every load or poll (t50hp-e4).
// Request shapes are copied from apps/web (api/jmap.ts listMailbox,
// realtime/queryDelta.ts reconcileQuery, api/accountPreferences.ts,
// api/appearanceCustomization.ts, api/sessionActivity.ts). They are separate
// groups: they never join `requests`, `mixed` or `representative`.
export const CLIENT_HOT_GROUPS=Object.freeze({'jmap-session':['jmap-session'],'list-batch':['list-batch'],changes:['changes'],'account-polls':['account-preferences','appearance-customization','session-activity']});
export function clientHotSpecs(p){
 const owner=p.ownerAccountId??p.accountId,inbox=typeof p.inboxId==='string'&&p.inboxId?p.inboxId:undefined;
 const filter=inbox?{inMailbox:inbox}:{},sort=[{property:'receivedAt',isAscending:false}];
 const batch=(name,calls,validate)=>({name,path:'/jmap/api',method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({using:MAIL_USING,methodCalls:calls}),validate:value=>{
  const rows=value?.methodResponses;
  return Array.isArray(rows)&&rows.length===calls.length&&rows.every((row,index)=>row[0]===calls[index][0]&&row[2]===calls[index][2]&&row[1]?.accountId===p.accountId)&&validate(rows.map(row=>row[1]));
 }});
 const pageSize=Math.min(25,p.messageIds.length),noChange=row=>Array.isArray(row)&&row.length===0;
 const json=(name,path,validate,extra={})=>({name,path,validate,...extra});
 return [
  json('jmap-session','/jmap/session',v=>!!v&&typeof v==='object'&&typeof v.apiUrl==='string'&&!!v.accounts&&Object.hasOwn(v.accounts,p.accountId)),
  // The list batch of listMailbox() plus the unread count query (limit 0, notKeyword $seen).
  batch('list-batch',[
   ['Email/query',{accountId:p.accountId,filter,sort,limit:pageSize,calculateTotal:true},'q'],
   ['Email/get',{accountId:p.accountId,'#ids':{resultOf:'q',name:'Email/query',path:'/ids'},properties:DEFAULT_HEADER_PROPERTIES},'g'],
   ['Email/query',{accountId:p.accountId,filter:{...filter,notKeyword:'$seen'},limit:0,calculateTotal:true},'unread'],
  ],([query,get,unread])=>Array.isArray(query.ids)&&query.ids.length===pageSize&&typeof query.queryState==='string'&&Array.isArray(get.list)&&get.list.length===pageSize&&query.ids.every(id=>get.list.some(row=>row.id===id))&&Number.isSafeInteger(unread.total)&&unread.total>=0&&Array.isArray(unread.ids)&&unread.ids.length===0),
  // Poll with no change. The JMAP state is only known at run time, so a one-row
  // query supplies it by result reference (Email/query queryState and Email/changes
  // state are the same per-account Email state in the engine).
  batch('changes',[
   ['Email/query',{accountId:p.accountId,filter,sort,limit:1},'base'],
   ['Email/queryChanges',{accountId:p.accountId,filter,sort,'#sinceQueryState':{resultOf:'base',name:'Email/query',path:'/queryState'},maxChanges:1000,calculateTotal:true},'delta-query'],
   ['Email/changes',{accountId:p.accountId,'#sinceState':{resultOf:'base',name:'Email/query',path:'/queryState'},maxChanges:1000},'delta-emails'],
  ],([base,queried,changed])=>typeof base.queryState==='string'&&queried.oldQueryState===base.queryState&&queried.newQueryState===base.queryState&&noChange(queried.removed)&&noChange(queried.added)&&changed.oldState===base.queryState&&changed.newState===base.queryState&&noChange(changed.created)&&noChange(changed.updated)&&noChange(changed.destroyed)&&changed.hasMoreChanges!==true),
  json('account-preferences','/api/account/preferences',v=>v?.version===1&&!!v.preferences&&typeof v.preferences==='object'&&!Array.isArray(v.preferences)),
  json('appearance-customization',`/api/account/appearance/customization?accountId=${encodeURIComponent(owner)}`,v=>v?.ownerId===owner&&v.document?.version===1&&typeof v.compiledCss==='string'&&Array.isArray(v.lockedFields)),
  {name:'session-activity',path:'/api/account/session/activity',method:'POST',headers:{},validate:v=>v?.ok===true},
 ];
}

export function prepareFixtureRequests({principals,clientSimulation,engineMode,representative=false,workloadProfile}){
 if(typeof representative!=='boolean'||!['native','proxy'].includes(engineMode)||representative&&engineMode!=='native')throw new Error('Invalid private fixture request mode');
 const clientModel=prepareFixtureClients(principals,clientSimulation),native=engineMode==='native';
 if(principals.some(p=>typeof p.accountId!=='string'||!p.accountId||!Array.isArray(p.messageIds)||p.messageIds.length<1||p.messageIds.length>100||p.messageIds.some(id=>typeof id!=='string'||!id))||new Set(principals.map(p=>p.accountId)).size!==principals.length)throw new Error('Fixture requests require distinct seeded account/message bindings');
 if(representative&&workloadProfile&&principals.some(p=>p.messageIds.length!==workloadProfile.messagesPerPrincipal||p.bodySampleBytes!==workloadProfile.bodySampleBytes||p.attachmentSampleBytes!==workloadProfile.attachmentSampleBytes))throw new Error('Representative principal does not match the global profile');
 const specs=p=>{
  const ids=p.messageIds.slice(0,2),emailGet=jmapFixtureRequest(p,'Email/get',{ids,properties:representative?DEFAULT_HEADER_PROPERTIES:MINIMAL_HEADER_PROPERTIES},{native}),valid=emailGet.validate;
  emailGet.validate=value=>valid(value)&&Array.isArray(value.methodResponses[0][1].list)&&ids.every(id=>value.methodResponses[0][1].list.some(row=>row.id===id));
  const baseline=[{name:'authenticated-identity',path:'/api/me',validate:v=>v.accountId===(p.ownerAccountId??p.accountId)},jmapFixtureRequest(p,'Mailbox/get',{}),jmapFixtureRequest(p,'Email/query',{limit:25}),emailGet,{name:'concurrent-refresh-handler',path:'/api/mail/refresh',method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({accountId:p.ownerAccountId??p.accountId}),validate:v=>v.accountId===(p.ownerAccountId??p.accountId)&&v.upstream===native}];
  return representative?[...baseline,...representativeSpecs(p)].map(spec=>({...spec,serializePrincipal:true})):baseline;
 };
 const prepared=principals.map((p,index)=>specs(p).map(spec=>({...spec,headers:{...clientModel.clients[index].headers,...spec.headers}})));
 const hot=principals.map((p,index)=>clientHotSpecs(p).map(spec=>({...spec,headers:{...clientModel.clients[index].headers,...spec.headers}})));
 const hotRequests=Object.fromEntries(hot[0].map((spec,type)=>[spec.name,{...spec,materialize:(sequence,{requestCount})=>{
  const principalIndex=fixturePrincipalIndex(sequence,requestCount,principals.length);
  return {...hot[principalIndex][type],principalIndex,clientIndex:clientModel.clients[principalIndex].clientIndex};
 }}]));
 const requests=prepared[0].map((spec,type)=>({...spec,materialize:(sequence,{requestCount})=>{
  const principalIndex=fixturePrincipalIndex(sequence,requestCount,principals.length);
  return {...prepared[principalIndex][type],principalIndex,clientIndex:clientModel.clients[principalIndex].clientIndex};
 }}));
 const groups={...Object.fromEntries(Object.entries(CLIENT_HOT_GROUPS).map(([group,names])=>[group,names.map(name=>hotRequests[name])])),mixed:requests.slice(0,5),identity:[requests[0]],mailboxes:[requests[1]],query:[requests[2]],headers:[requests[3]],refresh:[requests[4]],...(representative?{
  representative:requests,'paged-headers':[requests[5]],reader:[requests[6]],attachment:[requests[7]],'flag-write':[requests[8]],'move-write':[requests[9]],
 }:{})};

 // goal-65-35: 100-slot deterministic schedule of existing validated templates (plus three write shapes built like them).
 // Every entry is serialised per user, so writes for one user never interleave with each other or with the reads that
 // assume stable state (the change poll in particular).
 // Needs four seeded messages per user (three write shapes use distinct messages); smaller fixtures simply have no such group.
 const goalSlots=representative&&principals.every(p=>p.messageIds.length>=4)?(()=>{
  const order=goalMixSchedule();
  const perPrincipal=principals.map((p,index)=>{
   const byName=new Map([...specs(p),...clientHotSpecs(p),...goalMixWriteSpecs(p)].map(spec=>[spec.name,spec]));
   return order.map(name=>{
    const spec=byName.get(name);
    if(!spec)throw new Error('Goal mix shape has no template: '+name);
    return {...spec,serializePrincipal:true,headers:{...clientModel.clients[index].headers,...spec.headers}};
   });
  });
  return perPrincipal[0].map((spec,slot)=>({...spec,materialize:(sequence)=>{
   const principalIndex=fixturePrincipalIndex(sequence,GOAL_MIX_CYCLE,principals.length);
   return {...perPrincipal[principalIndex][slot],principalIndex,clientIndex:clientModel.clients[principalIndex].clientIndex};
  }}));
 })():null;
 if(goalSlots)groups[GOAL_MIX_NAME]=goalSlots;
 const marked=prepared[0].filter(spec=>spec.headers[nativeReadContract.header]===nativeReadContract.value).map(spec=>spec.name);
 const readIntent={contractSchema:nativeReadContract.schema,header:nativeReadContract.header,value:nativeReadContract.value,markedShapes:marked,totalShapes:prepared[0].length,scope:'actual constructed request headers; eligibility declaration only, not observed guard reuse',observedGuardConsumption:null};
 return {readIntent,requests,groups,headers:clientModel.clients[0].headers,clientModel,firstRequests:prepared[0]};
}
