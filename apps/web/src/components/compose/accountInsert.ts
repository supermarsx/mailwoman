import {apiFetch} from '../../api/http.ts';
import {withBase} from '../../api/basePath.ts';
import {CAP_CORE,CAP_MAIL,type JmapResponse,type JmapSession} from '../../api/jmap-types.ts';
import {CAP_CALENDARS,CAP_CONTACTS,type ContactCard} from '../../api/pim-types.ts';
import {responseFor} from '../../api/jmap.ts';
import {eventsExport} from '../../modules/calendar/api.ts';
import {toVCardDocument} from '../../modules/contacts/vcard.ts';
import {preferenceAccountIsCurrent,type PreferenceAccount} from '../../api/accountPreferenceLifecycle.ts';

export type AccountItemKind='event'|'calendar'|'contact'|'message';
export interface AccountItem {id:string;label:string}
export interface AccountItemPage {items:AccountItem[];limited:boolean}
export interface AccountInsertService {
  capabilities(signal:AbortSignal):Promise<AccountItemKind[]>;
  list(kind:AccountItemKind,signal:AbortSignal):Promise<AccountItemPage>;
  file(kind:AccountItemKind,item:AccountItem,signal:AbortSignal):Promise<File>;
}
const MAX_BYTES=16*1024*1024,MAX_ITEMS=100,MAX_CALENDAR_EVENTS=1000;
const caps:Record<AccountItemKind,string>={event:CAP_CALENDARS,calendar:CAP_CALENDARS,contact:CAP_CONTACTS,message:CAP_MAIL};
const names:Record<AccountItemKind,string>={event:'CalendarEvent',calendar:'Calendar',contact:'ContactCard',message:'Email'};
/** No preloaded cross-account store data and no raw-message export-policy bypass. */
export function accountInsertService(owner:PreferenceAccount,fetcher:typeof apiFetch=apiFetch):AccountInsertService {
  function current():void {if(!owner.storageAuthority||!preferenceAccountIsCurrent(owner))throw new Error('accountChanged');}
  async function request(path:string,signal:AbortSignal,body?:unknown):Promise<Response>{
    current();
    const response=await fetcher(withBase(path),{credentials:'same-origin',signal,headers:{'x-mailwoman-account':owner.id,'x-mailwoman-storage-authority':owner.storageAuthority!,...(body===undefined?{}:{'content-type':'application/json'})},...(body===undefined?{}:{method:'POST',body:JSON.stringify(body)})});
    current();if(!response.ok)throw new Error('requestRefused');return response;
  }
  async function bytes(response:Response,signal:AbortSignal):Promise<Uint8Array<ArrayBuffer>> {
    if(!response.body)throw new Error('emptyResponse');const reader=response.body.getReader();const chunks:Uint8Array<ArrayBuffer>[]=[];let length=0;
    try{for(;;){current();signal.throwIfAborted();const part=await reader.read();if(part.done)break;length+=part.value.byteLength;if(length>MAX_BYTES)throw new Error('tooLarge');chunks.push(new Uint8Array(part.value));}}
    finally{await reader.cancel().catch(()=>undefined);reader.releaseLock();}
    current();const result=new Uint8Array(length);let offset=0;for(const chunk of chunks){result.set(chunk,offset);offset+=chunk.length;}return result;
  }
  async function json<T>(response:Response,signal:AbortSignal):Promise<T>{return JSON.parse(new TextDecoder().decode(await bytes(response,signal))) as T;}
  async function call<T>(kind:AccountItemKind,method:string,args:Record<string,unknown>,signal:AbortSignal):Promise<T>{
    const result=await json<JmapResponse>(await request('/jmap',signal,{using:[CAP_CORE,caps[kind]],methodCalls:[[method,{...args,accountId:owner.id},'item']]}),signal);
    const value=responseFor<T&{accountId?:string}>(result,'item');if(value.accountId!==owner.id)throw new Error('accountChanged');return value;
  }
  async function query(kind:AccountItemKind,signal:AbortSignal,calendarId?:string):Promise<{ids:string[];total?:number}>{
    return call(kind,`${names[kind]}/query`,{limit:calendarId?MAX_CALENDAR_EVENTS+1:MAX_ITEMS+1,filter:calendarId?{calendarId}:{},...(kind==='message'?{sort:[{property:'receivedAt',isAscending:false}]}:{})},signal);
  }
  async function capabilities(signal:AbortSignal):Promise<AccountItemKind[]>{
    const session=await json<JmapSession>(await request('/jmap/session',signal),signal);const account=session.accounts[owner.id];if(!account)throw new Error('accountChanged');
    return (Object.keys(caps) as AccountItemKind[]).filter(kind=>Object.hasOwn(session.capabilities,caps[kind])&&Object.hasOwn(account.accountCapabilities,caps[kind]));
  }
  async function list(kind:AccountItemKind,signal:AbortSignal):Promise<AccountItemPage>{
    let ids:string[]|null=null,limited=false;
    if(kind!=='calendar'){const queried=await query(kind,signal);limited=queried.ids.length>MAX_ITEMS||(queried.total??0)>MAX_ITEMS;ids=queried.ids.slice(0,MAX_ITEMS);}
    const result=await call<{list:Array<{id:string;title?:string;name?:string|{full?:string};subject?:string}>}>(kind,`${names[kind]}/get`,{ids,...(kind==='message'?{properties:['id','subject']}:{})},signal);
    limited ||= result.list.length>MAX_ITEMS;
    return {limited,items:result.list.slice(0,MAX_ITEMS).map(item=>({id:item.id,label:item.subject??item.title??(typeof item.name==='string'?item.name:item.name?.full)??item.id}))};
  }
  async function file(kind:AccountItemKind,item:AccountItem,signal:AbortSignal):Promise<File>{
    current();let content:BlobPart;let type:string,extension:string;
    if(kind==='message'){
      content=await bytes(await request(`/api/export/${encodeURIComponent(item.id)}?format=eml`,signal),signal);type='message/rfc822';extension='eml';
    }else if(kind==='contact'){
      const result=await call<{list:ContactCard[]}>('contact','ContactCard/get',{ids:[item.id]},signal);
      if(result.list.length!==1||result.list[0]?.id!==item.id)throw new Error('missingItem');
      content=toVCardDocument(result.list);type='text/vcard';extension='vcf';
    }else{
      let ids=[item.id];if(kind==='calendar'){const all=await query('event',signal,item.id);if(all.ids.length>MAX_CALENDAR_EVENTS||(all.total??0)>MAX_CALENDAR_EVENTS)throw new Error('tooManyEvents');ids=all.ids;}
      const built=eventsExport(owner.id,ids);const result=await json<JmapResponse>(await request('/jmap',signal,built),signal);const exported=responseFor<{accountId:string;blob:string}>(result,'export');if(exported.accountId!==owner.id||!exported.blob.startsWith('BEGIN:VCALENDAR'))throw new Error('invalidExport');
      content=exported.blob;type='text/calendar';extension='ics';
    }
    current();signal.throwIfAborted();const filename=(Array.from(item.label).map(char=>char.charCodeAt(0)<32?'_':char).join('').replace(/[\\/:*?"<>|]/g,'_').slice(0,100)||'attachment')+'.'+extension;
    const file=new File([content],filename,{type});if(file.size>MAX_BYTES)throw new Error('tooLarge');return file;
  }
  return {capabilities,list,file};
}
