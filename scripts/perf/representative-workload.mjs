// Separate read/write profile. Every item is ONE actual authenticated HTTP
// request; a JMAP batch's methods are not misreported as additional HTTP RPS.
import {DEFAULT_HEADER_PROPERTIES,READER_PROPERTIES,READER_FETCH_OPTIONS,fixturePropertyProfile} from './fixture-properties.mjs';
const goldenAttachments=new Map();let goldenAllocations=0;
export function attachmentPatternCacheStats(){return {entries:goldenAttachments.size,bytes:[...goldenAttachments.values()].reduce((sum,bytes)=>sum+bytes.length,0),allocations:goldenAllocations};}
export function attachmentPatternValidator(size){
 if(!Number.isInteger(size)||size<4||size>262144)throw new Error('Invalid bounded attachment sample size');
 // Validators retain only the size, never an evicted buffer per principal.
 return bytes=>{
  if(!Buffer.isBuffer(bytes)||bytes.length!==size)return false;
  let expected=goldenAttachments.get(size);
  if(!expected){
   if(goldenAttachments.size===4)goldenAttachments.delete(goldenAttachments.keys().next().value);
   expected=Buffer.alloc(size,'ABCD');goldenAttachments.set(size,expected);goldenAllocations++;
  }
  return bytes.equals(expected);
 };
}
// The goal mix is seeded exactly like the representative workload (same folders, messages, attachment).
export const GOAL_MIX_NAME='goal-65-35';
export const isRepresentativeKind=kind=>kind==='representative'||kind===GOAL_MIX_NAME;
export function workloadProfileReport(value,principalCount){
 if(value===undefined)return null;
 if(!value||!['baseline','representative',GOAL_MIX_NAME].includes(value.kind)||!Number.isInteger(value.messagesPerPrincipal)||value.messagesPerPrincipal<2||value.messagesPerPrincipal>100||value.kind==='baseline'&&value.messagesPerPrincipal!==2||value.seededMessages!==value.messagesPerPrincipal*principalCount||value.seededMessages>500000||!Number.isFinite(value.seedSeconds)||value.seedSeconds<0)throw new Error('Invalid fixture workload profile');
 if(isRepresentativeKind(value.kind)&&(!Number.isInteger(value.bodySampleBytes)||value.bodySampleBytes<27||value.bodySampleBytes>16384||!Number.isInteger(value.attachmentSampleBytes)||value.attachmentSampleBytes<4||value.attachmentSampleBytes>262144))throw new Error('Invalid fixture sample sizes');
 return {kind:value.kind,messagesPerPrincipal:value.messagesPerPrincipal,seededMessages:value.seededMessages,seedSeconds:value.seedSeconds,...(isRepresentativeKind(value.kind)?{bodySampleBytes:value.bodySampleBytes,attachmentSampleBytes:value.attachmentSampleBytes,ordinaryBodyPayloadBytes:26}:{}),sendsMeasured:false,upstreamNetworkMeasured:false,requestPropertySets:fixturePropertyProfile(isRepresentativeKind(value.kind)?'representative':value.kind)};
}
export function representativeSpecs(p){
 if(!p||typeof p.accountId!=='string'||!Array.isArray(p.messageIds)||p.messageIds.length<2||p.messageIds.length>100||new Set(p.messageIds).size!==p.messageIds.length||typeof p.inboxId!=='string'||!p.inboxId||typeof p.archiveId!=='string'||!p.archiveId||p.archiveId===p.inboxId||typeof p.attachmentBlobId!=='string'||!p.attachmentBlobId)throw new Error('Representative fixture requires seeded folders, messages and attachment');
 const id=p.messageIds[0],known=new Set(p.messageIds),call=(method,args,tag)=>[method,{accountId:p.accountId,...args},tag];
 const batch=(name,calls,validate)=>({name,path:'/jmap/api',method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({using:['urn:ietf:params:jmap:core','urn:ietf:params:jmap:mail'],methodCalls:calls}),serializePrincipal:true,validate:value=>{
  const rows=value?.methodResponses;
  return Array.isArray(rows)&&rows.length===calls.length&&rows.every((row,index)=>row[0]===calls[index][0]&&row[2]===calls[index][2]&&row[1]?.accountId===p.accountId)&&validate(rows.map(row=>row[1]));
 }});
 const owned=(row,wanted)=>Array.isArray(row.list)&&row.list.length===wanted.length&&new Set(row.list.map(item=>item.id)).size===wanted.length&&wanted.every(id=>row.list.some(item=>item.id===id))&&(!row.notFound||row.notFound.length===0);
 const get=(properties,tag)=>call('Email/get',{ids:[id],properties},tag);
 const changed=row=>row.updated&&Object.hasOwn(row.updated,id)&&(!row.notUpdated||Object.keys(row.notUpdated).length===0);
 const inFolder=(row,folder)=>owned(row,[id])&&row.list[0].mailboxIds?.[folder]===true&&Object.keys(row.list[0].mailboxIds).length===1;
 const expectedBody='Representative body sample.'+'x'.repeat((p.bodySampleBytes??27)-27);
 const bodyMatches=part=>{
  if(typeof part?.value!=='string'||part.isTruncated===true||part.isEncodingProblem===true)return false;
  // The parser may preserve the single MIME boundary line ending; the
  // configured payload excludes that delimiter, not arbitrary whitespace.
  const value=part.value.endsWith('\r\n')?part.value.slice(0,-2):part.value.endsWith('\n')?part.value.slice(0,-1):part.value;
  return value===expectedBody&&Buffer.byteLength(value)===(p.bodySampleBytes??27);
 };
 const pageSize=Math.min(25,p.messageIds.length),nextSize=Math.min(25,Math.max(0,p.messageIds.length-pageSize));
 return [
  batch('paged-query-and-headers',[
   call('Email/query',{limit:pageSize,position:0,calculateTotal:true},'page1'),
   call('Email/get',{'#ids':{resultOf:'page1',name:'Email/query',path:'/ids'},properties:DEFAULT_HEADER_PROPERTIES},'headers1'),
   call('Email/query',{limit:25,position:pageSize,calculateTotal:true},'page2'),
   call('Email/get',{'#ids':{resultOf:'page2',name:'Email/query',path:'/ids'},properties:DEFAULT_HEADER_PROPERTIES},'headers2'),
  ],([first,headers1,second,headers2])=>Array.isArray(first.ids)&&Array.isArray(second.ids)&&first.ids.length===pageSize&&second.ids.length===nextSize&&first.total===known.size&&second.total===known.size&&first.position===0&&second.position===pageSize&&new Set([...first.ids,...second.ids]).size===pageSize+nextSize&&[...first.ids,...second.ids].every(id=>known.has(id))&&owned(headers1,first.ids)&&owned(headers2,second.ids)),
  batch('body-and-attachment-metadata',[call('Email/get',{ids:[id],properties:READER_PROPERTIES,...READER_FETCH_OPTIONS},'body')],([row])=>owned(row,[id])&&Object.values(row.list[0].bodyValues??{}).some(bodyMatches)&&row.list[0].attachments?.some(part=>part.blobId===p.attachmentBlobId&&part.name==='sample.txt')),
  {name:'attachment-download',path:`/jmap/download/${encodeURIComponent(p.accountId)}/${encodeURIComponent(p.attachmentBlobId)}/sample.txt`,serializePrincipal:true,validateBytes:attachmentPatternValidator(p.attachmentSampleBytes??4)},
  batch('mark-read-and-restore',[
   call('Email/set',{update:{[id]:{'keywords/$seen':true}}},'read'),get(['id','keywords'],'readback'),
   call('Email/set',{update:{[id]:{'keywords/$seen':null}}},'restore'),get(['id','keywords'],'restored'),
  ],([set,read,restore,restored])=>changed(set)&&changed(restore)&&owned(read,[id])&&read.list[0].keywords?.$seen===true&&owned(restored,[id])&&!restored.list[0].keywords?.$seen),
  batch('move-and-restore',[
   call('Email/set',{update:{[id]:{[`mailboxIds/${p.archiveId}`]:true,[`mailboxIds/${p.inboxId}`]:null}}},'move'),get(['id','mailboxIds'],'moved'),
   call('Email/set',{update:{[id]:{[`mailboxIds/${p.inboxId}`]:true,[`mailboxIds/${p.archiveId}`]:null}}},'restore'),get(['id','mailboxIds'],'restored'),
  ],([move,moved,restore,restored])=>changed(move)&&inFolder(moved,p.archiveId)&&changed(restore)&&inFolder(restored,p.inboxId)),
 ].map(spec=>({...spec,...(['mark-read-and-restore','move-and-restore'].includes(spec.name)?{statefulMutation:true}:{})}));
}

// ---- goal-65-35: the user's goal mix (30,000 requests/s, about 65% reads / 35% writes BY HTTP REQUEST COUNT) ----
// One entry is one actual HTTP request. A WRITE request is one POST /jmap/api carrying four JMAP method calls:
// Email/set (mutation 1), Email/get (read-back verifying it), Email/set (mutation 2), Email/get (read-back).
// So a write request holds exactly `mutationsPerRequest` = 2 message updates (each touches one message) and 2 read-backs.
// Every template is an existing validated one (identity, jmap-session, list batch, Email/get, reader body, changes,
// preferences, appearance) or a write shape built like the validated mark-read/move pairs.
// NOT in the mix, because no validated template or seeded folder exists: delete-to-trash (the fixture seeds no trash
// folder), draft save (no drafts folder), search (no validated search shape), and the session-activity POST (its
// read/write class is unclear). Add them only together with seeding and validators.
// Weights are per 100 requests, so every 100 consecutive requests contain exactly 65 reads and 35 writes.
export const GOAL_MIX_TABLE=Object.freeze([
 {name:'authenticated-identity',kind:'read',weight:6,mutationsPerRequest:0},
 {name:'jmap-session',kind:'read',weight:6,mutationsPerRequest:0},
 {name:'list-batch',kind:'read',weight:16,mutationsPerRequest:0},
 {name:'Email/get',kind:'read',weight:10,mutationsPerRequest:0},
 {name:'body-and-attachment-metadata',kind:'read',weight:10,mutationsPerRequest:0},
 {name:'changes',kind:'read',weight:9,mutationsPerRequest:0},
 {name:'account-preferences',kind:'read',weight:4,mutationsPerRequest:0},
 {name:'appearance-customization',kind:'read',weight:4,mutationsPerRequest:0},
 {name:'mark-read-and-restore',kind:'write',weight:8,mutationsPerRequest:2},
 {name:'mark-unread-and-restore',kind:'write',weight:5,mutationsPerRequest:2},
 {name:'flag-and-restore',kind:'write',weight:8,mutationsPerRequest:2},
 {name:'unflag-and-restore',kind:'write',weight:4,mutationsPerRequest:2},
 {name:'move-and-restore',kind:'write',weight:10,mutationsPerRequest:2},
].map(row=>Object.freeze(row)));
export const GOAL_MIX_CYCLE=100;
const weightOf=kind=>GOAL_MIX_TABLE.filter(row=>row.kind===kind).reduce((n,row)=>n+row.weight,0);
if(weightOf('read')!==65||weightOf('write')!==35)throw new Error('The goal mix table must weigh 65 reads and 35 writes');

// Deterministic smooth weighted round robin: the same 100-slot order every time, writes spread through the cycle.
export function goalMixSchedule(){
 const credit=new Map(GOAL_MIX_TABLE.map(row=>[row.name,0])),order=[];
 for(let slot=0;slot<GOAL_MIX_CYCLE;slot++){
  let best=null;
  for(const row of GOAL_MIX_TABLE){credit.set(row.name,credit.get(row.name)+row.weight);if(!best||credit.get(row.name)>credit.get(best.name))best=row;}
  credit.set(best.name,credit.get(best.name)-GOAL_MIX_CYCLE);order.push(best.name);
 }
 return order;
}

// Write shapes added for the goal mix. Each sets, reads back, sets again, reads back; the validators check both
// read-backs and both updates, so an unverified or lost mutation is an error (and taints the stage).
export function goalMixWriteSpecs(p){
 if(!p||!Array.isArray(p.messageIds)||p.messageIds.length<4||typeof p.accountId!=='string')throw new Error('The goal mix needs at least four seeded messages per user');
 const call=(method,args,tag)=>[method,{accountId:p.accountId,...args},tag];
 const batch=(name,calls,validate)=>({name,path:'/jmap/api',method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({using:['urn:ietf:params:jmap:core','urn:ietf:params:jmap:mail'],methodCalls:calls}),serializePrincipal:true,statefulMutation:true,validate:value=>{
  const rows=value?.methodResponses;
  return Boolean(Array.isArray(rows)&&rows.length===calls.length&&rows.every((row,index)=>row[0]===calls[index][0]&&row[2]===calls[index][2]&&row[1]?.accountId===p.accountId)&&validate(rows.map(row=>row[1])));
 }});
 const changed=(row,id)=>row.updated&&Object.hasOwn(row.updated,id)&&(!row.notUpdated||Object.keys(row.notUpdated).length===0);
 const one=(row,id)=>Array.isArray(row.list)&&row.list.length===1&&row.list[0].id===id&&(!row.notFound||row.notFound.length===0);
 const toggle=(name,id,keyword,first,second)=>batch(name,[
  call('Email/set',{update:{[id]:{['keywords/'+keyword]:first}}},'first'),call('Email/get',{ids:[id],properties:['id','keywords']},'firstback'),
  call('Email/set',{update:{[id]:{['keywords/'+keyword]:second}}},'second'),call('Email/get',{ids:[id],properties:['id','keywords']},'secondback'),
 ],([a,ab,b,bb])=>changed(a,id)&&changed(b,id)&&one(ab,id)&&one(bb,id)&&Boolean(ab.list[0].keywords?.[keyword])===(first===true)&&Boolean(bb.list[0].keywords?.[keyword])===(second===true));
 // Distinct messages per shape; per-user serialisation keeps shapes that share a message from interleaving.
 return [
  toggle('mark-unread-and-restore',p.messageIds[2],'$seen',null,true),
  toggle('flag-and-restore',p.messageIds[1],'$flagged',true,null),
  toggle('unflag-and-restore',p.messageIds[3],'$flagged',null,true),
 ];
}

// Per-stage mix accounting from the fixed per-shape counters. Returns undefined unless every shape is in the table.
export function goalMixReport(byRequest,seconds){
 const names=Object.keys(byRequest??{});
 if(!names.length||!Number.isFinite(seconds)||seconds<=0||names.some(name=>!GOAL_MIX_TABLE.some(row=>row.name===name)))return undefined;
 const tally={read:{requests:0,accepted:0,acceptedInWindow:0},write:{requests:0,accepted:0,acceptedInWindow:0}};
 let mutationsAttempted=0,mutationsVerified=0;
 for(const row of GOAL_MIX_TABLE){
  const stat=byRequest[row.name];if(!stat)continue;
  const bucket=tally[row.kind];
  bucket.requests+=stat.requests;bucket.accepted+=stat.accepted;bucket.acceptedInWindow+=stat.acceptedDuringScheduledWindow;
  mutationsAttempted+=stat.requests*row.mutationsPerRequest;mutationsVerified+=stat.acceptedDuringScheduledWindow*row.mutationsPerRequest;
 }
 const total=tally.read.requests+tally.write.requests,accepted=tally.read.acceptedInWindow+tally.write.acceptedInWindow;
 return {schema:1,mix:GOAL_MIX_NAME,
  attempted:{reads:tally.read.requests,writes:tally.write.requests,readShare:total?tally.read.requests/total:0,writeShare:total?tally.write.requests/total:0},
  acceptedInWindow:{reads:tally.read.acceptedInWindow,writes:tally.write.acceptedInWindow,readShare:accepted?tally.read.acceptedInWindow/accepted:0,writeShare:accepted?tally.write.acceptedInWindow/accepted:0},
  readRequestsPerSecond:tally.read.acceptedInWindow/seconds,writeRequestsPerSecond:tally.write.acceptedInWindow/seconds,
  mutationsAttempted,mutationsPerSecond:mutationsVerified/seconds,
  mutationsBasis:'verified message updates (2 per accepted write request) in the scheduled window',
  upstreamCommandsPerSecond:null,upstreamCommandsNote:'not exposed: the fixture diagnostics carry no upstream command counter, so none is estimated'};
}
