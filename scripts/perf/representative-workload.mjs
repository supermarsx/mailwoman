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

// ---- goal-65-35: the user's goal mix (100,000 requests/s, about 65% reads / 35% writes BY HTTP REQUEST COUNT) ----
// One entry is one actual HTTP request. Every template is validated: either an existing one (identity, jmap-session,
// list batch, Email/get, reader body, changes, preferences, appearance, mark-read, move) or one built like them and
// smoke-validated against a real seeded fixture (t60): mark-unread, flag, unflag, delete-to-trash, draft save, search.
// Weights are per 100 request slots: exactly 65 reads and 35 writes in every 100 consecutive requests.
// `mutationsPerRequest` is the number of message mutations (Email/set creates, updates or destroys) one request carries.
//  - a toggle/move/trash write request is one POST /jmap/api: Email/set, Email/get, Email/set, Email/get = 2 mutations
//    and 2 read-backs;
//  - draft save is TWO requests: `draft-save` (one Email/set create = 1 mutation) and `draft-discard` (Email/get of
//    the created draft, then Email/set destroy = 1 mutation). They hold one slot each (2 write slots). A discard slot
//    destroys the oldest draft this generator process created; before any draft exists (the first cycle) it creates one
//    instead, so that slot is then counted under the discard label while carrying a create.
export const GOAL_MIX_TABLE=Object.freeze([
 {name:'authenticated-identity',kind:'read',weight:6,mutationsPerRequest:0},
 {name:'jmap-session',kind:'read',weight:5,mutationsPerRequest:0},
 {name:'list-batch',kind:'read',weight:15,mutationsPerRequest:0},
 {name:'Email/get',kind:'read',weight:9,mutationsPerRequest:0},
 {name:'body-and-attachment-metadata',kind:'read',weight:10,mutationsPerRequest:0},
 {name:'changes',kind:'read',weight:9,mutationsPerRequest:0},
 {name:'account-preferences',kind:'read',weight:4,mutationsPerRequest:0},
 {name:'appearance-customization',kind:'read',weight:3,mutationsPerRequest:0},
 {name:'search-planted-term',kind:'read',weight:4,mutationsPerRequest:0},
 {name:'mark-read-and-restore',kind:'write',weight:7,mutationsPerRequest:2},
 {name:'mark-unread-and-restore',kind:'write',weight:4,mutationsPerRequest:2},
 {name:'flag-and-restore',kind:'write',weight:6,mutationsPerRequest:2},
 {name:'unflag-and-restore',kind:'write',weight:3,mutationsPerRequest:2},
 {name:'move-and-restore',kind:'write',weight:8,mutationsPerRequest:2},
 {name:'delete-to-trash-and-restore',kind:'write',weight:5,mutationsPerRequest:2},
 {name:'draft-save',kind:'write',weight:1,mutationsPerRequest:1},
 {name:'draft-discard',kind:'write',weight:1,mutationsPerRequest:1},
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

const USING=['urn:ietf:params:jmap:core','urn:ietf:params:jmap:mail'];
// A user can run the goal mix when the fixture seeded trash and drafts folders, the planted search term and five messages.
export const goalMixSeeded=p=>Boolean(p&&Array.isArray(p.messageIds)&&p.messageIds.length>=5&&typeof p.accountId==='string'&&typeof p.inboxId==='string'&&typeof p.trashId==='string'&&p.trashId&&typeof p.draftsId==='string'&&p.draftsId&&Number.isInteger(p.plantedQuarterly)&&p.plantedQuarterly>=0);
function mailBatch(p,name,calls,validate,extra={}){
 return {name,path:'/jmap/api',method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({using:USING,methodCalls:calls}),serializePrincipal:true,...extra,validate:value=>{
  const rows=value?.methodResponses;
  return Boolean(Array.isArray(rows)&&rows.length===calls.length&&rows.every((row,index)=>row[0]===calls[index][0]&&row[2]===calls[index][2]&&row[1]?.accountId===p.accountId)&&validate(rows.map(row=>row[1])));
 }};
}
const callFor=p=>(method,args,tag)=>[method,{accountId:p.accountId,...args},tag];

// Write shapes added for the goal mix. Each sets, reads back, sets again, reads back; the validators check both
// read-backs and both updates, so an unverified or lost mutation is an error (and taints the stage).
// goalMixWriteSpecs(p) returns the stateless shapes only (the draft pair needs run-time state, see goalMixDraftSpecs).
export function goalMixWriteSpecs(p){
 if(!p||!Array.isArray(p.messageIds)||p.messageIds.length<4||typeof p.accountId!=='string')throw new Error('The goal mix needs at least four seeded messages per user');
 const call=callFor(p),batch=(name,calls,validate)=>mailBatch(p,name,calls,validate,{statefulMutation:true});
 const changed=(row,id)=>row.updated&&Object.hasOwn(row.updated,id)&&(!row.notUpdated||Object.keys(row.notUpdated).length===0);
 const one=(row,id)=>Array.isArray(row.list)&&row.list.length===1&&row.list[0].id===id&&(!row.notFound||row.notFound.length===0);
 const toggle=(name,id,keyword,first,second)=>batch(name,[
  call('Email/set',{update:{[id]:{['keywords/'+keyword]:first}}},'first'),call('Email/get',{ids:[id],properties:['id','keywords']},'firstback'),
  call('Email/set',{update:{[id]:{['keywords/'+keyword]:second}}},'second'),call('Email/get',{ids:[id],properties:['id','keywords']},'secondback'),
 ],([a,ab,b,bb])=>changed(a,id)&&changed(b,id)&&one(ab,id)&&one(bb,id)&&Boolean(ab.list[0].keywords?.[keyword])===(first===true)&&Boolean(bb.list[0].keywords?.[keyword])===(second===true));
 // Distinct messages per shape; per-user serialisation keeps shapes that share a message from interleaving.
 const specs=[
  toggle('mark-unread-and-restore',p.messageIds[2],'$seen',null,true),
  toggle('flag-and-restore',p.messageIds[1],'$flagged',true,null),
  toggle('unflag-and-restore',p.messageIds[3],'$flagged',null,true),
 ];
 if(goalMixSeeded(p)){
  const id=p.messageIds[4],inFolder=(row,folder)=>one(row,id)&&row.list[0].mailboxIds?.[folder]===true&&Object.keys(row.list[0].mailboxIds).length===1;
  specs.push(batch('delete-to-trash-and-restore',[
   call('Email/set',{update:{[id]:{['mailboxIds/'+p.trashId]:true,['mailboxIds/'+p.inboxId]:null}}},'trash'),call('Email/get',{ids:[id],properties:['id','mailboxIds']},'trashed'),
   call('Email/set',{update:{[id]:{['mailboxIds/'+p.inboxId]:true,['mailboxIds/'+p.trashId]:null}}},'restore'),call('Email/get',{ids:[id],properties:['id','mailboxIds']},'restored'),
  ],([a,ab,b,bb])=>changed(a,id)&&inFolder(ab,p.trashId)&&changed(b,id)&&inFolder(bb,p.inboxId)));
 }
 return specs;
}

// Search of the planted term in the inbox: the total must equal the seeded count and the page must be fetched.
export function goalMixSearchSpec(p){
 if(!goalMixSeeded(p))throw new Error('The goal mix search needs the planted search term seeding');
 const call=callFor(p);
 return mailBatch(p,'search-planted-term',[
  call('Email/query',{filter:{inMailbox:p.inboxId,text:'quarterly'},sort:[{property:'receivedAt',isAscending:false}],limit:50,calculateTotal:true},'q'),
  call('Email/get',{'#ids':{resultOf:'q',name:'Email/query',path:'/ids'},properties:['id','subject','receivedAt']},'g'),
 ],([q,g])=>q.total===p.plantedQuarterly&&Array.isArray(q.ids)&&q.ids.length===Math.min(50,p.plantedQuarterly)&&Array.isArray(g.list)&&g.list.length===q.ids.length);
}

// Draft save is two requests. The save creates a draft in the drafts folder and hands (user, id) to `onCreated`;
// the discard reads that draft back and destroys it. A creation back-reference in one request is not used: the
// engine does not resolve it (t60). Each save leaves a copy in the upstream Drafts folder (known engine defect).
export function goalMixDraftSpecs(p,{onCreated}){
 if(!goalMixSeeded(p))throw new Error('Draft save needs the drafts folder seeding');
 const call=callFor(p);
 const save=mailBatch(p,'draft-save',[
  call('Email/set',{create:{d1:{mailboxIds:{[p.draftsId]:true},keywords:{$draft:true,$seen:true},to:[{email:'someone@native-load.invalid'}],subject:'Goal mix draft',bodyValues:{b1:{value:'Draft body.'}},textBody:[{partId:'b1',type:'text/plain'}]}}},'save'),
 ],([row])=>{
  const id=row.created?.d1?.id;
  if(typeof id!=='string'||!id||row.notCreated&&Object.keys(row.notCreated).length)return false;
  onCreated?.(id);return true;
 },{statefulMutation:true});
 const discard=id=>mailBatch(p,'draft-discard',[
  call('Email/get',{ids:[id],properties:['id','mailboxIds','keywords']},'saved'),
  call('Email/set',{destroy:[id]},'discard'),
 ],([saved,gone])=>Array.isArray(saved.list)&&saved.list.length===1&&saved.list[0].mailboxIds?.[p.draftsId]===true&&saved.list[0].keywords?.$draft===true&&Array.isArray(gone.destroyed)&&gone.destroyed.includes(id)&&(!gone.notDestroyed||Object.keys(gone.notDestroyed).length===0),{statefulMutation:true});
 return {save,discard};
}

// Per-stage mix accounting from the fixed per-shape counters. Returns undefined unless every shape is in the table.
// `facts` (optional): {users, meanLatencyMs} for the per-user line.
export function goalMixReport(byRequest,seconds,facts={}){
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
 const users=Number.isInteger(facts.users)&&facts.users>0?facts.users:null,meanMs=Number.isFinite(facts.meanLatencyMs)&&facts.meanLatencyMs>0?facts.meanLatencyMs:null;
 return {schema:1,mix:GOAL_MIX_NAME,
  attempted:{reads:tally.read.requests,writes:tally.write.requests,readShare:total?tally.read.requests/total:0,writeShare:total?tally.write.requests/total:0},
  acceptedInWindow:{reads:tally.read.acceptedInWindow,writes:tally.write.acceptedInWindow,readShare:accepted?tally.read.acceptedInWindow/accepted:0,writeShare:accepted?tally.write.acceptedInWindow/accepted:0},
  readRequestsPerSecond:tally.read.acceptedInWindow/seconds,writeRequestsPerSecond:tally.write.acceptedInWindow/seconds,
  mutationsAttempted,mutationsPerSecond:mutationsVerified/seconds,
  mutationsBasis:'verified message mutations in the scheduled window: 2 per accepted toggle/move/trash write request, 1 per draft-save and per draft-discard',
  perUser:{users,acceptedRequestsPerSecondPerUser:users?accepted/seconds/users:null,
   serialisedCeilingPerUser:meanMs?1000/meanMs:null,
   note:'each user\'s requests are serialised (one in flight per user), so a user cannot exceed 1000 / mean latency ms requests per second; the ceiling uses the stage mean latency'},
  upstreamCommandsPerSecond:null,upstreamCommandsNote:'attached from the fixture diagnostics before/after snapshots (see upstream); null when they are missing'};
}

// Upstream command rate from the cumulative process-wide counters of the fixture diagnostics (before/after the stage).
// Commands include those caused by reads (fetch); `writeCommandsPerWriteRequest` counts store, move, append and expunge only.
const UPSTREAM_KINDS=['store','move','expunge','append','fetch','list'];
export function upstreamReport(before,after,{seconds,writeRequests}){
 const counters=(before&&after&&typeof before==='object'&&typeof after==='object')?UPSTREAM_KINDS.concat('commands'):null;
 if(!counters||counters.some(key=>!Number.isSafeInteger(before[key])||!Number.isSafeInteger(after[key])||after[key]<before[key]))return {available:false,reason:'upstream command counters missing or reset in the stage diagnostics'};
 const delta=Object.fromEntries(counters.map(key=>[key,after[key]-before[key]]));
 const writeCommands=delta.store+delta.move+delta.append+delta.expunge;
 return {available:true,basis:'delta of cumulative process-wide counters between the stage\'s before and after snapshots; includes drain after the window',
  commands:delta.commands,commandsPerSecond:delta.commands/seconds,
  perKindPerSecond:Object.fromEntries(UPSTREAM_KINDS.map(key=>[key,delta[key]/seconds])),perKind:Object.fromEntries(UPSTREAM_KINDS.map(key=>[key,delta[key]])),
  writeCommands,writeCommandsPerWriteRequest:writeRequests>0?writeCommands/writeRequests:null,commandsPerWriteRequestBasis:'store + move + append + expunge divided by write requests attempted in the stage'};
}
// consumedFrames per ordinary context for the stage (the request-inputs counters of the diagnostics), as the request-inputs evidence reports it.
export function requestInputsRatio(before,after){
 const a=before?.ordinaryRequestInputs,b=after?.ordinaryRequestInputs;
 if(!a||!b||![a.consumedFrames,a.ordinaryContexts,b.consumedFrames,b.ordinaryContexts].every(Number.isSafeInteger))return null;
 const contexts=b.ordinaryContexts-a.ordinaryContexts,frames=b.consumedFrames-a.consumedFrames;
 return contexts>0&&frames>=0?{consumedFramesPerContext:+(frames/contexts).toFixed(4),consumedFrames:frames,ordinaryContexts:contexts}:null;
}
// Added to a stage's `mix` once its diagnostics are known.
export function mixWithDiagnostics(mix,diagnostics,seconds){
 if(!mix)return mix;
 const upstream=upstreamReport(diagnostics?.before?.upstreamCommands,diagnostics?.after?.upstreamCommands,{seconds,writeRequests:mix.attempted.writes});
 return {...mix,upstream,upstreamCommandsPerSecond:upstream.available?upstream.commandsPerSecond:null,requestInputs:requestInputsRatio(diagnostics?.before,diagnostics?.after)};
}
