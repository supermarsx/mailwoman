import {apiFetch} from './http.ts';
import {withBase} from './basePath.ts';
import {observePreferenceAccount,preferenceAccountIsCurrent,type PreferenceAccount} from './accountPreferenceLifecycle.ts';
import {untrack} from 'solid-js';
import type {StoredDraft} from '../components/compose/drafts-store.ts';
export interface DraftEditorState {
 version:1;identityId:string;fromOverride?:string;importance?:'high'|'normal'|'low';to:string;cc:string;bcc:string;subject:string;bodyFormat:'plain'|'html';bodyHtml:string;bodyText:string;
 editorMode?:'rich'|'html';
 source?:NonNullable<StoredDraft['source']>;
 noticeMode?:'auto'|'none'|'manual';noticeIds?:string[];
 insertedSignatures:string[];inReplyTo:string[];references:string[];
 attachments:NonNullable<StoredDraft['attachments']>;inline:NonNullable<StoredDraft['inline']>;
 attachPublicKey?:boolean;includeFileHashes?:boolean;sentMailboxId?:string;
 /** The Reply-To typed in the composer, and the message options (JSON text); both come back when the draft is reopened. */
 replyTo?:string;messageOptions?:string;
 requestReceipt:boolean;requestDeliveryReceipt:boolean;openTracking:boolean;protectedContent:boolean;
}
export interface AccountDraftDocument {accountId:string;storageAuthority:string;id:string;revision:string|null;emailId:string|null;savedAt:number;status:'saved'|'uncertain'|'refused';cleanupPending:boolean;summary:{subject?:string;bodyFormat?:string;source?:NonNullable<StoredDraft['source']>};state?:DraftEditorState}
export interface AccountDraftList {accountId:string;storageAuthority:string;supported:boolean;reason?:string|null;maximumDrafts?:number;items:AccountDraftDocument[]}
export class DraftSaveError extends Error {constructor(readonly code:string,readonly retryAfterMs=30_000){super(code);this.name='DraftSaveError';}}
interface DraftListCache {key:string;until:number;value?:AccountDraftList;error?:unknown;pending?:Promise<AccountDraftList>;controller:AbortController}
let draftListCache:DraftListCache|undefined;
observePreferenceAccount(()=>{draftListCache?.controller.abort();draftListCache=undefined;});
function listForCaller(pending:Promise<AccountDraftList>,signal?:AbortSignal):Promise<AccountDraftList>{
 return new Promise((resolve,reject)=>{
  const abort=()=>{signal?.removeEventListener('abort',abort);reject(new DOMException('Request aborted','AbortError'));};
  if(signal?.aborted){abort();return;}
  signal?.addEventListener('abort',abort,{once:true});
  pending.then(value=>resolve(structuredClone(value)),reject).finally(()=>signal?.removeEventListener('abort',abort));
 });
}
export function draftEditorState(draft:StoredDraft):DraftEditorState{return {version:1,...(draft.noticeMode?{noticeMode:draft.noticeMode,noticeIds:draft.noticeIds??[]}:{}),...(draft.source?{source:draft.source}:{}),attachPublicKey:draft.attachPublicKey??false,includeFileHashes:draft.includeFileHashes??false,...(draft.sentMailboxId?{sentMailboxId:draft.sentMailboxId}:{}),...(draft.replyTo?{replyTo:draft.replyTo}:{}),...(draft.messageOptions?{messageOptions:draft.messageOptions}:{}),...(draft.importance?{importance:draft.importance}:{}),...(draft.editorMode?{editorMode:draft.editorMode}:{}),identityId:draft.identityId??'',...(draft.fromOverride===undefined?{}:{fromOverride:draft.fromOverride}),to:draft.to,cc:draft.cc??'',bcc:draft.bcc??'',subject:draft.subject,bodyFormat:draft.bodyFormat??'html',bodyHtml:draft.bodyHtml,bodyText:draft.bodyText,insertedSignatures:draft.insertedSignatures??[],inReplyTo:draft.inReplyTo??[],references:draft.references??[],attachments:draft.attachments??[],inline:draft.inline??[],requestReceipt:draft.requestReceipt??false,requestDeliveryReceipt:draft.requestDeliveryReceipt??false,openTracking:draft.openTracking??false,protectedContent:false};}
export function localAccountDraft(document:AccountDraftDocument):StoredDraft{
 if(!document.state)throw new DraftSaveError('draftStateUnavailable');
 return {...document.state,serverStatus:document.status,...(document.revision?{serverRevision:document.revision}:{}),id:document.id,accountId:document.accountId,storageAuthority:document.storageAuthority,savedAt:document.savedAt*1000};
}
export function accountDraftApi(owner:PreferenceAccount,signal?:AbortSignal){
 async function request<T>(path:string,method='GET',body?:unknown,requestSignal=signal):Promise<T>{
  if(!owner.storageAuthority||!preferenceAccountIsCurrent(owner))throw new DraftSaveError('accountChanged');
  const storageAuthority=owner.storageAuthority;
  const response=await untrack(()=>apiFetch(withBase(`/api/account/drafts${path}`),{method,headers:{'content-type':'application/json','x-mailwoman-account':owner.id,'x-mailwoman-storage-authority':storageAuthority},...(requestSignal?{signal:requestSignal}:{}),...(body===undefined?{}:{body:JSON.stringify(body)})}));
  if(!preferenceAccountIsCurrent(owner))throw new DraftSaveError('accountChanged');
  const value=await response.json().catch(()=>({})) as Record<string,unknown>;
  if(!preferenceAccountIsCurrent(owner))throw new DraftSaveError('accountChanged');
  if(!response.ok){const retry=response.headers.get('retry-after'),seconds=retry?Number(retry):NaN;const delay=Number.isFinite(seconds)?seconds*1000:retry?Date.parse(retry)-Date.now():30_000;throw new DraftSaveError(typeof value.code==='string'?value.code:'draftSaveFailed',Math.max(30_000,Number.isFinite(delay)?delay:30_000));}
  if(value.accountId!==owner.id||value.storageAuthority!==owner.storageAuthority)throw new DraftSaveError('accountChanged');
  if(method!=='GET')draftListCache=undefined;
  return value as T;
 }
 function list():Promise<AccountDraftList>{
  if(!owner.storageAuthority||!preferenceAccountIsCurrent(owner))return Promise.reject(new DraftSaveError('accountChanged'));
  if(signal?.aborted)return Promise.reject(new DOMException('Request aborted','AbortError'));
  const key=JSON.stringify([owner.id,owner.storageAuthority,owner.generation]);
  const existing=draftListCache;
  if(existing?.key===key){
   if(existing.pending)return listForCaller(existing.pending,signal);
   if(existing.until>Date.now())return existing.value?listForCaller(Promise.resolve(existing.value),signal):Promise.reject(existing.error);
  }
  const entry:DraftListCache={key,until:0,controller:new AbortController()};draftListCache=entry;
  const timeout=setTimeout(()=>entry.controller.abort(),10_000);
  entry.pending=request<AccountDraftList>('','GET',undefined,entry.controller.signal).then(value=>{
   if(draftListCache===entry){entry.value=value;entry.until=Date.now()+15_000;}return value;
  },error=>{if(draftListCache===entry){entry.error=error;entry.until=Date.now()+(error instanceof DraftSaveError?error.retryAfterMs:30_000);}throw error;}).finally(()=>{clearTimeout(timeout);delete entry.pending;});
  return listForCaller(entry.pending,signal);
 }
 return {resolve:(emailId:string)=>request<{draft:AccountDraftDocument|null}>(`/resolve?emailId=${encodeURIComponent(emailId)}`),discard:(id:string,expectedRevision:string|null)=>request<{deleted:boolean}>(`/${encodeURIComponent(id)}`,'DELETE',{expectedRevision}),list,load:(id:string)=>request<AccountDraftDocument>(`/${encodeURIComponent(id)}`),save:async(id:string,operationId:string,expectedRevision:string|null,state:DraftEditorState)=>{
  let outgoing=state;
  if(JSON.stringify(state).includes('mw-local:')){
   const [{prepareLocalFiles},{createConfiguredClient},{uploadBlob},{offlineServerBase}]=await Promise.all([import('../offline/localFiles.ts'),import('./transport.ts'),import('./jmap.ts'),import('../offline/cacheScope.ts')]);
   if(!owner.storageAuthority||!preferenceAccountIsCurrent(owner))throw new DraftSaveError('accountChanged');
   const session=await createConfiguredClient().session();
   const resolved=await prepareLocalFiles({using:[],methodCalls:[['Draft/set',{accountId:owner.id,state},'draft']]},{accountId:owner.id,storageAuthority:owner.storageAuthority,base:offlineServerBase()},file=>uploadBlob(session.uploadUrl,owner.id,file));
   outgoing=resolved.methodCalls[0]![1]['state'] as DraftEditorState;
  }
  return request<AccountDraftDocument>(`/${encodeURIComponent(id)}`,'PUT',{operationId,expectedRevision,state:outgoing});
 },cleanup:(id:string)=>request<AccountDraftDocument>(`/${encodeURIComponent(id)}/cleanup`,'POST')};
}
