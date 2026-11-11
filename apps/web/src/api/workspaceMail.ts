import {apiFetch,type MailboxTarget} from './http.ts';
import {preferenceAccount,preferenceAccountIsCurrent} from './accountPreferenceLifecycle.ts';
import {createLinkedAccountsApi} from './linkedAccounts.ts';
import {withBase} from './basePath.ts';
import {CAP_CORE,CAP_MAIL,type Email,type JmapRequest,type JmapResponse,type Mailbox} from './jmap-types.ts';
import {responseFor} from './jmap.ts';

export interface WorkspaceMailbox extends MailboxTarget {
  label: string;
  current: boolean;
}
export async function workspaceMailboxes(activeLink:string|null):Promise<WorkspaceMailbox[]> {
  const owner=preferenceAccount();
  if(!owner?.storageAuthority)throw new Error('Mailbox authority unavailable');
  const api=createLinkedAccountsApi();
  try{
    const {primary,accounts,policy}=await api.list();
    if(!preferenceAccountIsCurrent(owner))throw new DOMException('Mailbox changed','AbortError');
    const primaryTarget:WorkspaceMailbox={linkId:null,accountId:primary.accountId,storageAuthority:primary.storageAuthority,owner,label:primary.address,current:activeLink===null};
    const linked=policy.enabled?accounts.flatMap(row=>row.status==='ready'&&row.storageAuthority?[{
      linkId:row.id,accountId:row.accountId,storageAuthority:row.storageAuthority,owner,label:row.displayName||row.address||row.accountId,current:row.id===activeLink,
    }]:[]):[];
    return [primaryTarget,...linked];
  }finally{api.dispose();}
}
function request(target:WorkspaceMailbox,path:string,signal:AbortSignal,init:RequestInit={}):Promise<Response>{
  return apiFetch(withBase(path),{...init,mailboxTarget:target,credentials:'same-origin',signal,redirect:'error'});
}
async function json<T>(response:Response):Promise<T>{
  if(!response.ok)throw new Error(`Mailbox request failed (${response.status})`);
  return await response.json() as T;
}
async function jmap(target:WorkspaceMailbox,methodCalls:JmapRequest['methodCalls'],signal:AbortSignal):Promise<JmapResponse>{
  return json(await request(target,'/jmap/api',signal,{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({using:[CAP_CORE,CAP_MAIL],methodCalls})}));
}
export async function workspaceFolders(target:WorkspaceMailbox,signal:AbortSignal):Promise<Mailbox[]>{
  const result=await jmap(target,[['Mailbox/get',{accountId:target.accountId,properties:['id','name','parentId','role','myRights','mailwomanSearchQuery']},'folders']],signal);
  return responseFor<{list:Mailbox[]}>(result,'folders').list;
}

export type UnifiedRole='all'|'inbox'|'sent'|'drafts'|'archive'|'junk'|'trash';
export interface UnifiedRow {target:WorkspaceMailbox;email:Email;folderId?:string}
export async function workspaceMessages(target:WorkspaceMailbox,boxes:Mailbox[],role:UnifiedRole,search:string,position:number,signal:AbortSignal):Promise<{rows:UnifiedRow[];next:number|null}>{
  const accountId=target.accountId;
  const folder=role==='all'?undefined:boxes.find(box=>box.role===role);
  if(role!=='all'&&!folder)return {rows:[],next:null};
  const limit=30;
  const filter={...(folder?{inMailbox:folder.id}:{}),...(search?{text:search}:{})};
  const response=await jmap(target,[
    ['Email/query',{accountId,filter,position,limit,sort:[{property:'receivedAt',isAscending:false}]},'query'],
    ['Email/get',{accountId,'#ids':{resultOf:'query',name:'Email/query',path:'/ids'},properties:['id','from','subject','preview','receivedAt','mailboxIds','keywords','hasAttachment']},'rows'],
  ],signal);
  const query=responseFor<{ids:string[];position:number}>(response,'query');
  const {list}=responseFor<{list:Email[]}>(response,'rows');
  return {
    rows:list.map(email=>({email,target,...(folder?{folderId:folder.id}:{})})),
    next:query.ids.length===limit?query.position+query.ids.length:null,
  };
}

interface TransferFormats {engineMode:boolean;available:boolean;maxBytes:number}
export async function transferFormats(target:WorkspaceMailbox,signal:AbortSignal):Promise<TransferFormats>{
  return json(await request(target,'/api/import/formats',signal));
}
async function boundedMessage(response:Response,cap:number):Promise<Blob>{
  if(!response.ok||!response.body)throw new Error('Source export failed');
  const reader=response.body.getReader();
  const chunks:Uint8Array<ArrayBuffer>[]=[];
  let size=0;
  try{
    while(true){
      const {done,value}=await reader.read();
      if(done)break;
      size+=value.byteLength;
      if(size>cap)throw new Error('Message exceeds the destination import limit');
      chunks.push(new Uint8Array(value));
    }
  }catch(error){await reader.cancel().catch(()=>undefined);throw error;}
  finally{reader.releaseLock();}
  if(!size)throw new Error('Message export was empty');
  return new Blob(chunks,{type:'message/rfc822'});
}
async function sourceEmail(source:WorkspaceMailbox,emailId:string,signal:AbortSignal):Promise<{email:Email;state:string}>{
  const response=await jmap(source,[['Email/get',{accountId:source.accountId,ids:[emailId],properties:['id','blobId','keywords','mailboxIds']},'source']],signal);
  const {list,state}=responseFor<{list:Email[];state:string}>(response,'source');
  const email=list.find(row=>row.id===emailId);
  if(!email||!state)throw new Error('Source message unavailable');
  return {email,state};
}
export type TransferOutcome='copied'|'moved'|'import-unconfirmed'|'source-unconfirmed';
export async function transferWorkspaceMessage(input:{
  source:WorkspaceMailbox;target:WorkspaceMailbox;emailId:string;folderId:string;move:boolean;signal:AbortSignal;allowed:()=>boolean;
}):Promise<TransferOutcome>{
  const {source,target,emailId,folderId,move,signal,allowed}=input;
  const authorize=()=>{if(!allowed()||signal.aborted||!preferenceAccountIsCurrent(source.owner)||target.owner!==source.owner)throw new DOMException('Mailbox access changed','AbortError');};
  authorize();
  if(source.linkId===target.linkId||source.storageAuthority===target.storageAuthority)throw new Error('Choose a different account');
  const [{email},destination,formats]=await Promise.all([sourceEmail(source,emailId,signal),workspaceFolders(target,signal),transferFormats(target,signal)]);
  if(!formats.engineMode||!formats.available||!Number.isSafeInteger(formats.maxBytes)||formats.maxBytes<=0
    ||!destination.some(box=>box.id===folderId&&box.mailwomanSearchQuery===undefined&&box.myRights?.mayAddItems===true))throw new Error('Destination is unavailable');
  const originals=Object.keys(email.mailboxIds).filter(id=>email.mailboxIds[id]===true).sort();
  let trash:Mailbox|undefined;
  if(move){
    if(email.keywords?.['$draft']===true||!email.blobId||!originals.length)throw new Error('Source message cannot be moved');
    const boxes=await workspaceFolders(source,signal);
    trash=boxes.find(box=>box.role==='trash'&&box.myRights?.mayAddItems===true);
    if(!trash||originals.includes(trash.id)||originals.some(id=>!boxes.some(box=>box.id===id&&box.myRights?.mayRemoveItems===true)))throw new Error('Source cannot be moved to Trash');
  }
  const raw=await boundedMessage(await request(source,`/api/export/${encodeURIComponent(emailId)}?format=eml`,signal),Math.min(formats.maxBytes,64*1024*1024));
  authorize();
  try{
    const imported=await json<{imported:number;failed:number;total:number;mailboxId:string}>(await request(target,`/api/import/eml?mailboxId=${encodeURIComponent(folderId)}`,signal,{
      method:'POST',headers:{'content-type':'message/rfc822'},body:raw,
    }));
    if(imported.imported!==1||imported.failed!==0||imported.total!==1||imported.mailboxId!==folderId)return 'import-unconfirmed';
  }catch{return 'import-unconfirmed';}
  if(!move||!trash)return 'copied';
  try{
    authorize();
    const latest=await sourceEmail(source,emailId,signal);
    const locations=Object.keys(latest.email.mailboxIds).filter(id=>latest.email.mailboxIds[id]===true).sort();
    if(latest.email.blobId!==email.blobId||latest.email.keywords?.['$draft']===true||JSON.stringify(locations)!==JSON.stringify(originals))return 'source-unconfirmed';
    authorize();
    const response=await jmap(source,[['Email/set',{accountId:source.accountId,ifInState:latest.state,update:{[emailId]:{mailboxIds:{[trash.id]:true}}}},'move']],signal);
    const result=responseFor<{updated?:Record<string,unknown>}>(response,'move');
    if(!result.updated||!Object.hasOwn(result.updated,emailId))return 'source-unconfirmed';
    return 'moved';
  }catch{return 'source-unconfirmed';}
}
