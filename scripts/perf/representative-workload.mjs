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
export function workloadProfileReport(value,principalCount){
 if(value===undefined)return null;
 if(!value||!['baseline','representative'].includes(value.kind)||!Number.isInteger(value.messagesPerPrincipal)||value.messagesPerPrincipal<2||value.messagesPerPrincipal>100||value.kind==='baseline'&&value.messagesPerPrincipal!==2||value.seededMessages!==value.messagesPerPrincipal*principalCount||value.seededMessages>500000||!Number.isFinite(value.seedSeconds)||value.seedSeconds<0)throw new Error('Invalid fixture workload profile');
 if(value.kind==='representative'&&(!Number.isInteger(value.bodySampleBytes)||value.bodySampleBytes<27||value.bodySampleBytes>16384||!Number.isInteger(value.attachmentSampleBytes)||value.attachmentSampleBytes<4||value.attachmentSampleBytes>262144))throw new Error('Invalid fixture sample sizes');
 return {kind:value.kind,messagesPerPrincipal:value.messagesPerPrincipal,seededMessages:value.seededMessages,seedSeconds:value.seedSeconds,...(value.kind==='representative'?{bodySampleBytes:value.bodySampleBytes,attachmentSampleBytes:value.attachmentSampleBytes,ordinaryBodyPayloadBytes:26}:{}),sendsMeasured:false,upstreamNetworkMeasured:false,requestPropertySets:fixturePropertyProfile(value.kind)};
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
