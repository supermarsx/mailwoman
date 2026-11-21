// Private in-process request templates, shared by single and partitioned drivers.
// No network, main entrypoint, preflight or reporting side effects.
import {nativeReadIntentHeaders,nativeReadContract} from './native-read-intent.mjs';
import {prepareFixtureClients,fixturePrincipalIndex} from './fixture-clients.mjs';
import {representativeSpecs} from './representative-workload.mjs';
import {DEFAULT_HEADER_PROPERTIES,MINIMAL_HEADER_PROPERTIES} from './fixture-properties.mjs';

export function jmapFixtureRequest(p,name,args,{native=false}={}){
 const body={using:['urn:ietf:params:jmap:core','urn:ietf:params:jmap:mail'],methodCalls:[[name,{accountId:p.accountId,...args},'load']]};
 return {name,path:'/jmap/api',method:'POST',headers:{'content-type':'application/json',...(native?nativeReadIntentHeaders(body):{})},body:JSON.stringify(body),validate:value=>value.methodResponses?.[0]?.[0]===name&&value.methodResponses?.[0]?.[1]?.accountId===p.accountId};
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
 const requests=prepared[0].map((spec,type)=>({...spec,materialize:(sequence,{requestCount})=>{
  const principalIndex=fixturePrincipalIndex(sequence,requestCount,principals.length);
  return {...prepared[principalIndex][type],principalIndex,clientIndex:clientModel.clients[principalIndex].clientIndex};
 }}));
 const groups={mixed:requests.slice(0,5),identity:[requests[0]],mailboxes:[requests[1]],query:[requests[2]],headers:[requests[3]],refresh:[requests[4]],...(representative?{representative:requests}:{})};
 const marked=prepared[0].filter(spec=>spec.headers[nativeReadContract.header]===nativeReadContract.value).map(spec=>spec.name);
 const readIntent={contractSchema:nativeReadContract.schema,header:nativeReadContract.header,value:nativeReadContract.value,markedShapes:marked,totalShapes:prepared[0].length,scope:'actual constructed request headers; eligibility declaration only, not observed guard reuse',observedGuardConsumption:null};
 return {readIntent,requests,groups,headers:clientModel.clients[0].headers,clientModel,firstRequests:prepared[0]};
}
