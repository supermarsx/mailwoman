import {expandOfflineCalendar} from './calendarOffline.ts';
import {projectPending,pendingObject} from './pendingObjects.ts';
import type {QueuedItem} from './outboxCore.ts';
import {matchesCachedSearch} from './querySearch.ts';
import type {JmapRequest,JmapResponse,Email,Mailbox,FilterCondition} from '../api/jmap-types.ts';
import type {OfflineVault} from './vault.ts';
import {offlineLimits} from './runtime.ts';

type ObjectRow=Record<string,unknown>&{id:string};
const allowed=new Set(['Email','Mailbox','Thread','EmailSubmission','Calendar','CalendarEvent','Task','Note','AddressBook','Contact','ContactCard','ContactGroup','Identity']);
export const readOnlyJmap=(body:JmapRequest):boolean=>body.methodCalls.every(([name])=>{const [type,method]=name.split('/');return allowed.has(type!)&&(['get','query'].includes(method!)||name==='CalendarEvent/expand'||name==='CalendarEvent/itip'&&body.methodCalls.every(call=>call[1]['apply']!==true));});
export async function rememberJmap(vault:OfflineVault,namespace:string,request:JmapRequest,response:JmapResponse):Promise<void> {
 for(const [method,args,callId] of response.methodResponses){
  const [type,operation]=method.split('/');
  if(!type||!allowed.has(type))continue;
  const scope=`${namespace}:jmap:${JSON.stringify(args['accountId'])}`;
  const remove=async(id:string)=>{await vault.remove(`${scope}:objects:${type}`,id);if(type==='Email')await vault.remove(`${scope}:headers`,id);};
  if(operation==='set'||operation==='changes'){
   await vault.clear(`${namespace}:queries`);
   for(const id of Array.isArray(args['destroyed'])?args['destroyed']:[])if(typeof id==='string')await remove(id);
   if(operation==='set'){
    const input=request.methodCalls.find(call=>call[2]===callId)?.[1];
    const created=args['created'];
    for(const [key,raw] of Object.entries((created??{}) as Record<string,Record<string,unknown>>)){
     const source=(input?.['create'] as Record<string,Record<string,unknown>>|undefined)?.[key];
     if(source&&typeof raw['id']==='string')await vault.put(`${scope}:objects:${type}`,raw['id'],{...pendingObject(type,raw['id'],source),...raw,mailwomanQueued:false},offlineLimits({budgetGroup:namespace}));
    }
    const updated=args['updated'];
    for(const id of updated&&typeof updated==='object'?Object.keys(updated):[]){
     const patch=(input?.['update'] as Record<string,Record<string,unknown>>|undefined)?.[id];
     const old=await vault.get<ObjectRow>(`${scope}:objects:${type}`,id);if(!old||!patch)continue;
     const value=structuredClone(old);
     for(const [path,item] of Object.entries(patch)){
      const parts=path.split('/').map(part=>part.replace(/~1/g,'/').replace(/~0/g,'~'));
      let target:Record<string,unknown>=value;
      for(const part of parts.slice(0,-1)){if(!target[part]||typeof target[part]!=='object')target[part]={};target=target[part] as Record<string,unknown>;}
      const key=parts.at(-1)!;if(item===null)delete target[key];else target[key]=item;
     }
     const limits=offlineLimits({budgetGroup:namespace,pinned:value['pinned']===true,flagged:(value['keywords'] as Record<string,unknown>|undefined)?.['$flagged']===true});
     await vault.put(`${scope}:objects:${type}`,id,value,limits);
     if(type==='Email'){const {bodyValues:_body,htmlBody:_html,textBody:_text,attachments:_attachments,...header}=value;await vault.put(`${scope}:headers`,id,header,limits);}
    }
   }
   // Partial get states do not advance a collection baseline: a later /changes
   // still has to account for other objects not present in this response.
   continue;
  }
  if(operation==='expand'){const input=request.methodCalls.find(call=>call[2]===callId)?.[1];if(input)await vault.put(`${namespace}:expansions`,JSON.stringify(input),{request:input,response:args},offlineLimits({budgetGroup:namespace}));continue;}
  if(operation!=='get'||!Array.isArray(args['list']))continue;
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
  for(const id of Array.isArray(args['notFound'])?args['notFound']:[]){if(typeof id==='string')await remove(id);}
 }
 if(readOnlyJmap(request))await vault.put(`${namespace}:queries`,JSON.stringify(request),response,offlineLimits({budgetGroup:namespace}));
}
function match(email:Email,filter:Record<string,unknown>|undefined,mailboxes:Mailbox[]=[],username?:string):boolean {
 if(!filter)return true;
 if(Array.isArray(filter['conditions'])){
  const values=filter['conditions'].map(condition=>match(email,condition as Record<string,unknown>,mailboxes,username));
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
  else if(key==='text'){if(!matchesCachedSearch(email,String(value),mailboxes,username))return false;}
  else if(['subject','from','to','cc','bcc'].includes(key)){
   const text=key==='text'?JSON.stringify([email.subject,email.from,email.to,email.cc,email.preview]):JSON.stringify(email[key as keyof Email]);
   if(!String(text??'').toLocaleLowerCase().includes(String(value).toLocaleLowerCase()))return false;
  }else if(!['inMailbox','inMailboxOtherThan','hasKeyword','notKeyword','after','before','minSize','maxSize','hasAttachment','text','subject','from','to','cc','bcc'].includes(key))return false;
 }
 return true;
}
/** Offline responses are local views over cached objects, never server success
 * acknowledgements for writes. Missing bodies are reported as not cached. */
export async function readJmapOffline(vault:OfflineVault,namespace:string,request:JmapRequest,username?:string,pending:QueuedItem[]=[]):Promise<JmapResponse|undefined> {
 if(!readOnlyJmap(request))return;
 const responses:JmapResponse['methodResponses']=[];
 for(const [method,args,callId] of request.methodCalls){
  const [type,operation]=method.split('/'),accountId=args['accountId'];
  const scope=`${namespace}:jmap:${JSON.stringify(accountId)}`;
  const state=await vault.get<string>(`${scope}:states`,type!)??'offline';
  let ids=args['ids'] as string[]|null|undefined;
  const reference=args['#ids'] as {resultOf:string;name?:string;path?:string}|undefined;
  if(reference){
   const source=responses.find(row=>row[2]===reference.resultOf&&(!reference.name||row[0]===reference.name));
   if(!source)return;
   if(reference.path==='/ids')ids=source[1]['ids'] as string[]|undefined;
   else if(reference.path==='/list/*/emailIds')ids=(source[1]['list'] as {emailIds:string[]}[]|undefined)?.flatMap(row=>row.emailIds);
   else return;
   if(!Array.isArray(ids))return;
  }
  if(method==='CalendarEvent/itip'){const exact=await vault.get<JmapResponse>(`${namespace}:queries`,JSON.stringify(request));if(!exact)return;return exact;}
  if(operation==='expand'){
   const exact=await vault.get<{response:Record<string,unknown>}>(`${namespace}:expansions`,JSON.stringify(args));
   let held=exact?.response;
   if(!held){
    const start=Date.parse(String(args['start'])),end=Date.parse(String(args['end']));
    for(const {value} of await vault.list<{request:Record<string,unknown>;response:Record<string,unknown>}>(`${namespace}:expansions`)){
     if(!value.request||value.request['calendarId']!==args['calendarId']||value.request['accountId']!==accountId||Date.parse(String(value.request['start']))>start||Date.parse(String(value.request['end']))<end||!Array.isArray(value.response?.['list']))continue;
     const wanted=args['ids'] as string[]|undefined,covered=value.request['ids'] as string[]|undefined;
     if(covered&&(!wanted||wanted.some(id=>!covered.includes(id))))continue;
     held={...value.response,list:(value.response['list'] as Record<string,unknown>[]).filter(row=>(!wanted||wanted.includes(String(row['eventId'])))&&Date.parse(String(row['instanceEnd']))>start&&Date.parse(String(row['instanceStart']))<end)};break;
    }
   }
   const cachedEvents=(await vault.list<ObjectRow>(`${scope}:objects:CalendarEvent`)).map(row=>row.value);
   const projected=projectPending(cachedEvents,'CalendarEvent',String(accountId),pending);
   const changed=new Set(projected.filter(row=>row['mailwomanQueued']===true).map(row=>row.id));
   const removed=new Set(cachedEvents.filter(row=>!projected.some(value=>value.id===row.id)).map(row=>row.id));
   const wanted=args['ids'] as string[]|undefined;
   const selected=projected.filter(row=>(!wanted||wanted.includes(row.id))&&(!args['calendarId']||row['calendarId']===args['calendarId']));
   if(!held){held={accountId,list:await expandOfflineCalendar(selected,String(args['start']),String(args['end']))};}
   else if(changed.size||removed.size){held={...held,list:[...((held['list']??[]) as ObjectRow[]).filter(row=>!changed.has(String(row['eventId']))&&!removed.has(String(row['eventId']))),...await expandOfflineCalendar(selected.filter(row=>changed.has(row.id)),String(args['start']),String(args['end']))]};}
   responses.push([method,{...held,mailwomanOffline:true},callId]);continue;
  }
  if(operation==='get'){
   const all=ids==null?(await vault.list<ObjectRow>(`${scope}:objects:${type}`)).map(row=>row.value):await Promise.all(ids.map(id=>vault.get<ObjectRow>(`${scope}:objects:${type}`,id)));
   const needsBody=type==='Email'&&(args['fetchAllBodyValues']===true||(args['properties'] as string[]|undefined)?.includes('bodyValues'));
   const projected=projectPending(all.filter((row):row is ObjectRow=>!!row),type!,String(accountId),pending);
   const stored=projected.filter(row=>(ids==null||ids.includes(row.id))&&(!needsBody||row['bodyValues']!==undefined));
   const properties=args['properties'] as string[]|undefined;
   const rows=properties?stored.map(row=>Object.fromEntries([['id',row.id],...properties.filter(key=>key in row).map(key=>[key,row[key]])]) as ObjectRow):stored;
   responses.push([method,{accountId,state,list:rows,notFound:ids?.filter(id=>!rows.some(row=>row.id===id))??[]},callId]);
  }else{
   const cached=await vault.list<ObjectRow>(type==='Email'?`${scope}:headers`:`${scope}:objects:${type}`);
   let rows=projectPending(cached.map(row=>row.value),type!,String(accountId),pending);
   if(type==='Email'){
    const objects=await vault.list<ObjectRow>(`${scope}:objects:Email`),byId=new Map(rows.map(row=>[row.id,row]));for(const {value} of objects)byId.set(value.id,{...byId.get(value.id),...value});
    const mailboxes=(await vault.list<Mailbox>(`${scope}:objects:Mailbox`)).map(row=>row.value);
    rows=projectPending([...byId.values()],type!,String(accountId),pending).filter(row=>match(row as unknown as Email,args['filter'] as FilterCondition as Record<string,unknown>|undefined,mailboxes,username));
   }
   else rows=rows.filter(row=>matchPim(row,args['filter'] as Record<string,unknown>|undefined));
   const sort=args['sort'] as {property:string;isAscending?:boolean}[]|undefined;
   if(sort)rows.sort((a,b)=>{for(const field of sort){const order=(row:ObjectRow)=>field.property==='from'?JSON.stringify(row['from']??''):field.property==='importance'?(({high:2,low:0} as Record<string,number>)[String(row['mailwomanPriority'])]??1):field.property.split('/').reduce<unknown>((value,key)=>value&&typeof value==='object'?(value as Record<string,unknown>)[key]:undefined,row);const left=order(a),right=order(b);const compared=typeof left==='number'&&typeof right==='number'?left-right:String(left??'').localeCompare(String(right??''));if(compared)return field.isAscending===false?-compared:compared;}return a.id.localeCompare(b.id);});
   if(args['collapseThreads']===true&&type==='Email'){const seen=new Set<string>();rows=rows.filter(row=>{const key=String(row['threadId']??row.id);if(seen.has(key))return false;seen.add(key);return true;});}
   const position=Math.max(0,Number(args['position'])||0),limit=Math.max(0,Number(args['limit']??rows.length));
   responses.push([method,{accountId,queryState:'offline',canCalculateChanges:false,position,total:rows.length,ids:rows.slice(position,position+limit).map(row=>row.id)},callId]);
  }
 }
 return {methodResponses:responses,sessionState:'offline'};
}

function matchPim(row:ObjectRow,filter:Record<string,unknown>|undefined):boolean{
 if(!filter)return true;
 for(const [key,value] of Object.entries(filter)){
  if(value===null||value===undefined)continue;
  if(key==='text'){if(!JSON.stringify(row).toLocaleLowerCase().includes(String(value).toLocaleLowerCase()))return false;}
  else if(key==='isCompleted'){if((row['status']==='completed'||row['percentComplete']===100)!==value)return false;}
  else if(key==='after'){if(String(row['start']??row['due']??row['updatedAt']??'')<String(value))return false;}
  else if(key==='before'){if(String(row['start']??row['due']??row['updatedAt']??'')>=String(value))return false;}
  else if(key in row){if(Array.isArray(row[key])?!((row[key] as unknown[]).includes(value)):row[key]!==value)return false;}
  else return false;
 }
 return true;
}
