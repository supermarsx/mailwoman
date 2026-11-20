import {CAP_CALENDARS,CAP_TASKS,CAP_NOTES,CAP_CONTACTS} from '../api/pim-types.ts';
import type {Client} from '../api/client.ts';
import {CAP_CORE,CAP_MAIL,type Email,type EmailBodyPart,type JmapSession} from '../api/jmap-types.ts';
import {HEADER_PROPERTIES,BODY_PROPERTIES,responseFor} from '../api/jmap.ts';
import {apiFetch} from '../api/http.ts';
import {preferenceAccountIsCurrent,type PreferenceAccount} from '../api/accountPreferenceLifecycle.ts';
import {offlinePolicy,isOffline} from './runtime.ts';
import {offlineVault} from './vault.ts';
import {offlineServerBase} from './cacheScope.ts';
import type {SyncPolicy} from './syncPolicy.ts';

/** Bounded, abortable read-ahead of the chosen header window. Browsing older
 * mail still works online; this policy controls the durable offline working set. */
export async function synchronizeOffline(client:Client,owner:PreferenceAccount,policy:SyncPolicy,signal:AbortSignal):Promise<void>{
 const current=()=>!signal.aborted&&preferenceAccountIsCurrent(owner)&&offlinePolicy(owner).enabled&&!isOffline();
 if(!current()||!owner.storageAuthority)return;
 const vault=await offlineVault();
 const session=await client.session(),using=[CAP_CORE,CAP_MAIL];
 if(!current())return;
 await synchronizePim(client,owner.id,session,signal,current,async()=>(await vault.usage(JSON.stringify(['offline-device',offlineServerBase()]))).bytes<offlinePolicy(owner).budgetMb*1024*1024);
 if(!current())return;
 const after=policy.headersWindow==='all'?undefined:new Date(Date.now()-Number(policy.headersWindow)*86400000).toISOString();
 let previousPage='';
 for(let position=0;current();){
  if(!current()||(await vault.usage(JSON.stringify(['offline-device',offlineServerBase()]))).bytes>=offlinePolicy(owner).budgetMb*1024*1024)return;
  const response=await client.jmap({using,methodCalls:[['Email/query',{accountId:owner.id,filter:after?{text:`(after:${Math.floor(Date.parse(after)/1000)} OR is:flagged OR pinned:true)`}:{},sort:[{property:'receivedAt',isAscending:false}],position,limit:50},'sync-query'],['Email/get',{accountId:owner.id,'#ids':{resultOf:'sync-query',name:'Email/query',path:'/ids'},properties:[...HEADER_PROPERTIES],mailwomanBodyUse:false},'sync-headers']]},{signal,pace:'background'});
  if(!current())return;
  const query=responseFor<{ids:string[]}>(response,'sync-query'),headers=responseFor<{list:Email[]}>(response,'sync-headers');
  if(!Array.isArray(query.ids)||!Array.isArray(headers.list)||query.ids.length===0)return;
  const signature=JSON.stringify(query.ids);if(signature===previousPage)return;previousPage=signature;
  const protectedIds=new Set(headers.list.filter(email=>email.pinned||email.keywords?.['$flagged']).map(email=>email.id));
  const bodyIds=policy.bodies==='prefetch'||policy.attachments!=='on-demand'?query.ids:query.ids.filter(id=>protectedIds.has(id));
  for(let offset=0;offset<bodyIds.length;offset+=5){
   const batch=bodyIds.slice(offset,offset+5),fetchBodies=policy.bodies==='prefetch'||batch.some(id=>protectedIds.has(id));
   if(!current()||(await vault.usage(JSON.stringify(['offline-device',offlineServerBase()]))).bytes>=offlinePolicy(owner).budgetMb*1024*1024)return;
   // Offline read-ahead is not evidence that a person opened this message.
   const bodies=await client.jmap({using,methodCalls:[['Email/get',{accountId:owner.id,ids:batch,mailwomanBodyUse:false,...(fetchBodies?{properties:[...BODY_PROPERTIES],fetchHTMLBodyValues:true,fetchTextBodyValues:true,maxBodyValueBytes:1000000}:{properties:['id','attachments','bodyStructure']})},'sync-bodies']]},{signal,pace:'background'});
   if(policy.attachments!=='on-demand')for(const email of responseFor<{list:(Email&{attachments?:EmailBodyPart[];bodyStructure?:EmailBodyPart})[]}>(bodies,'sync-bodies').list){
    for(const part of [...(email.attachments??[]),...inlineParts(email.bodyStructure)]){
     if(!current())return;
     if(!part.blobId||part.size>10*1024*1024||policy.attachments==='inline'&&part.disposition!=='inline')continue;
     const target=downloadUrl(session,owner.id,part.blobId,part.name??'attachment',part.type??'application/octet-stream');
     // The attachment policy endpoint makes the same permission/risk decision
     // as an explicit preview; no warning is acknowledged automatically.
     const configured=offlineServerBase(),root=new URL(configured.startsWith('native:')?configured.slice(7):configured);
     const url=new URL(target,root.href+'/');if(url.origin!==root.origin||!url.pathname.startsWith(root.pathname.replace(/\/$/,'')+'/jmap/'))continue;
     url.searchParams.set('emailId',email.id);url.searchParams.set('preview','true');
     const response=await apiFetch(url.href,{credentials:'same-origin',headers:{'x-mailwoman-body-use':'false'},signal,pace:'background'});
     await response.body?.cancel();
    }
   }
  }
  position+=query.ids.length;if(query.ids.length<50)return;
  // Yield so visible requests and interaction take precedence.
  await new Promise<void>(resolve=>setTimeout(resolve,100));
 }
}
function downloadUrl(session:JmapSession,accountId:string,blobId:string,name:string,type:string):string{return session.downloadUrl.replaceAll('{accountId}',encodeURIComponent(accountId)).replaceAll('{blobId}',encodeURIComponent(blobId)).replaceAll('{name}',encodeURIComponent(name)).replaceAll('{type}',encodeURIComponent(type));}

function inlineParts(root:EmailBodyPart|undefined):EmailBodyPart[]{
 const stack=root?[root]:[],result:EmailBodyPart[]=[];
 for(let count=0;stack.length&&count<1000;count++){const part=stack.pop()!;if(part.disposition==='inline'||part.cid)result.push(part);if(part.subParts)stack.push(...part.subParts);}
 return result;
}

async function synchronizePim(client:Client,accountId:string,session:JmapSession,signal:AbortSignal,current:()=>boolean,room:()=>Promise<boolean>):Promise<void>{
 const get=async(type:string,using:string[])=>{if(!current()||!await room())return;await client.jmap({using,methodCalls:[[`${type}/get`,{accountId,ids:null},'sync-collections']]},{signal,pace:'background'});};
 await get('Mailbox',[CAP_CORE,CAP_MAIL]);await get('Identity',[CAP_CORE,CAP_MAIL]);
 const families:[string,string,string[]][]=[['Calendar',CAP_CALENDARS,[]],['AddressBook',CAP_CONTACTS,[]],['ContactGroup',CAP_CONTACTS,[]],['CalendarEvent',CAP_CALENDARS,[]],['Task',CAP_TASKS,[CAP_CALENDARS]],['Note',CAP_NOTES,[]],['ContactCard',CAP_CONTACTS,[]]];
 for(const [type,capability,extra] of families){
  if(!current()||!await room())return;
  if(!(capability in session.capabilities)&&!(capability in (session.accounts[accountId]?.accountCapabilities??{})))continue;
  const using=[CAP_CORE,capability,...extra];
  try{
   if(['Calendar','AddressBook','ContactGroup'].includes(type)){await get(type,using);continue;}
   let previous='';
   for(let position=0;current()&&await room();position+=50){
    const response=await client.jmap({using,methodCalls:[[`${type}/query`,{accountId,position,limit:50},'sync-pim-query'],[`${type}/get`,{accountId,'#ids':{resultOf:'sync-pim-query',name:`${type}/query`,path:'/ids'}},'sync-pim-get']]},{signal,pace:'background'});
    const query=responseFor<{ids:string[]}>(response,'sync-pim-query');if(!Array.isArray(query.ids)||!query.ids.length)break;
    const signature=JSON.stringify(query.ids);if(signature===previous)break;previous=signature;if(query.ids.length<50)break;
   }
   if(type==='CalendarEvent'&&current()&&await room()){
    const now=new Date(),start=new Date(now.getFullYear(),now.getMonth()-1,1).toISOString(),end=new Date(now.getFullYear()+1,now.getMonth()+2,1).toISOString();
    await client.jmap({using,methodCalls:[['CalendarEvent/expand',{accountId,start,end},'sync-expanded']]},{signal,pace:'background'});
   }
  }catch{if(!current())return;/* Unsupported families do not prevent mail sync. */}
 }
}
