import type {JmapRequest,JmapResponse,Email,FilterCondition} from '../api/jmap-types.ts';
import type {OfflineVault} from './vault.ts';
import {offlineLimits} from './runtime.ts';

type ObjectRow=Record<string,unknown>&{id:string};
const allowed=new Set(['Email','Mailbox','Thread','EmailSubmission','Calendar','CalendarEvent','Task','Note','AddressBook','Contact','Identity']);
export const readOnlyJmap=(body:JmapRequest):boolean=>body.methodCalls.every(([name])=>{const [type,method]=name.split('/');return allowed.has(type!)&&['get','query'].includes(method!);});
export async function rememberJmap(vault:OfflineVault,namespace:string,request:JmapRequest,response:JmapResponse):Promise<void> {
 if(!readOnlyJmap(request))return;
 for(const [method,args] of response.methodResponses){
  const [type,operation]=method.split('/');
  if(!type||!allowed.has(type)||operation!=='get'||!Array.isArray(args['list']))continue;
  const scope=`${namespace}:jmap:${JSON.stringify(args['accountId'])}`;
  if(typeof args['state']==='string')await vault.put(`${scope}:states`,type,args['state']);
  for(const row of args['list'] as ObjectRow[]){
   if(typeof row.id!=='string')continue;
   const store=`${scope}:objects:${type}`,old=await vault.get<ObjectRow>(store,row.id),value={...old,...row};
   const limits=offlineLimits({budgetGroup:namespace,pinned:value['pinned']===true,flagged:(value['keywords'] as Record<string,unknown>|undefined)?.['$flagged']===true});
   await vault.put(store,row.id,value,limits);
   if(type==='Email'){
    const {bodyValues:_body,htmlBody:_html,textBody:_text,attachments:_attachments,...header}=value;
    await vault.put(`${scope}:headers`,row.id,header,limits);
   }
  }
  for(const id of Array.isArray(args['notFound'])?args['notFound']:[]){if(typeof id==='string')await vault.remove(`${scope}:objects:${type}`,id);}
 }
 await vault.put(`${namespace}:queries`,JSON.stringify(request),response,offlineLimits({budgetGroup:namespace}));
}
function match(email:Email,filter:Record<string,unknown>|undefined):boolean {
 if(!filter)return true;
 if(Array.isArray(filter['conditions'])){
  const values=filter['conditions'].map(condition=>match(email,condition as Record<string,unknown>));
  return filter['operator']==='AND'?values.every(Boolean):filter['operator']==='OR'?values.some(Boolean):filter['operator']==='NOT'?!values.some(Boolean):false;
 }
 for(const [key,value] of Object.entries(filter)){
  if(key==='inMailbox'&&!email.mailboxIds?.[String(value)])return false;
  else if(key==='inMailboxOtherThan'&&Array.isArray(value)&&Object.keys(email.mailboxIds??{}).every(id=>value.includes(id)))return false;
  else if(key==='hasKeyword'&&email.keywords?.[String(value)]!==true)return false;
  else if(key==='notKeyword'&&email.keywords?.[String(value)]===true)return false;
  else if(key==='after'&&email.receivedAt<String(value))return false;
  else if(key==='before'&&email.receivedAt>=String(value))return false;
  else if(key==='minSize'&&(email.size??0)<Number(value))return false;
  else if(key==='maxSize'&&(email.size??0)>=Number(value))return false;
  else if(key==='hasAttachment'&&email.hasAttachment!==value)return false;
  else if(['text','subject','from','to','cc','bcc'].includes(key)){
   const text=key==='text'?JSON.stringify([email.subject,email.from,email.to,email.cc,email.preview]):JSON.stringify(email[key as keyof Email]);
   if(!String(text??'').toLocaleLowerCase().includes(String(value).toLocaleLowerCase()))return false;
  }else if(!['inMailbox','inMailboxOtherThan','hasKeyword','notKeyword','after','before','minSize','maxSize','hasAttachment','text','subject','from','to','cc','bcc'].includes(key))return false;
 }
 return true;
}
/** Offline responses are local views over cached objects, never server success
 * acknowledgements for writes. Missing bodies are reported as not cached. */
export async function readJmapOffline(vault:OfflineVault,namespace:string,request:JmapRequest):Promise<JmapResponse|undefined> {
 if(!readOnlyJmap(request))return;
 const responses:JmapResponse['methodResponses']=[];
 for(const [method,args,callId] of request.methodCalls){
  const [type,operation]=method.split('/'),accountId=args['accountId'];
  const scope=`${namespace}:jmap:${JSON.stringify(accountId)}`;
  const state=await vault.get<string>(`${scope}:states`,type!)??'offline';
  let ids=args['ids'] as string[]|null|undefined;
  const reference=args['#ids'] as {resultOf:string}|undefined;
  if(reference)ids=responses.find(row=>row[2]===reference.resultOf)?.[1]['ids'] as string[]|undefined;
  if(operation==='get'){
   const all=ids==null?(await vault.list<ObjectRow>(`${scope}:objects:${type}`)).map(row=>row.value):await Promise.all(ids.map(id=>vault.get<ObjectRow>(`${scope}:objects:${type}`,id)));
   const needsBody=type==='Email'&&(args['fetchAllBodyValues']===true||(args['properties'] as string[]|undefined)?.includes('bodyValues'));
   const rows=all.filter((row):row is ObjectRow=>!!row&&(!needsBody||row['bodyValues']!==undefined));
   responses.push([method,{accountId,state,list:rows,notFound:ids?.filter(id=>!rows.some(row=>row.id===id))??[]},callId]);
  }else{
   const cached=await vault.list<ObjectRow>(type==='Email'?`${scope}:headers`:`${scope}:objects:${type}`);
   let rows=cached.map(row=>row.value);
   if(type==='Email')rows=rows.filter(row=>match(row as unknown as Email,args['filter'] as FilterCondition as Record<string,unknown>|undefined));
   else if(args['filter']){
    const exact=await vault.get<JmapResponse>(`${namespace}:queries`,JSON.stringify(request));
    if(exact)return exact;
    return;
   }
   const sort=args['sort'] as {property:string;isAscending?:boolean}[]|undefined;
   if(sort)rows.sort((a,b)=>{for(const field of sort){const left=a[field.property],right=b[field.property];const compared=typeof left==='number'&&typeof right==='number'?left-right:String(left??'').localeCompare(String(right??''));if(compared)return field.isAscending===false?-compared:compared;}return a.id.localeCompare(b.id);});
   const position=Math.max(0,Number(args['position'])||0),limit=Math.max(0,Number(args['limit']??rows.length));
   responses.push([method,{accountId,queryState:'offline',canCalculateChanges:false,position,total:rows.length,ids:rows.slice(position,position+limit).map(row=>row.id)},callId]);
  }
 }
 return {methodResponses:responses,sessionState:'offline'};
}
