import {apiFetch} from './http.ts';
import {withBase} from './basePath.ts';
import {preferenceAccountIsCurrent,type PreferenceAccount} from './accountPreferenceLifecycle.ts';
import type {StoredDraft} from '../components/compose/drafts-store.ts';
export interface DraftEditorState {
 version:1;identityId:string;fromOverride?:string;importance?:'high'|'normal'|'low';to:string;cc:string;bcc:string;subject:string;bodyFormat:'plain'|'html';bodyHtml:string;bodyText:string;
 insertedSignatures:string[];inReplyTo:string[];references:string[];
 attachments:NonNullable<StoredDraft['attachments']>;inline:NonNullable<StoredDraft['inline']>;
 requestReceipt:boolean;requestDeliveryReceipt:boolean;openTracking:boolean;protectedContent:boolean;
}
export interface AccountDraftDocument {accountId:string;storageAuthority:string;id:string;revision:string|null;emailId:string|null;savedAt:number;status:'saved'|'uncertain'|'refused';cleanupPending:boolean;summary:{subject?:string;bodyFormat?:string};state?:DraftEditorState}
export interface AccountDraftList {accountId:string;storageAuthority:string;supported:boolean;reason?:string|null;maximumDrafts?:number;items:AccountDraftDocument[]}
export class DraftSaveError extends Error {constructor(readonly code:string){super(code);this.name='DraftSaveError';}}
export function draftEditorState(draft:StoredDraft):DraftEditorState{return {version:1,...(draft.importance?{importance:draft.importance}:{}),identityId:draft.identityId??'',...(draft.fromOverride===undefined?{}:{fromOverride:draft.fromOverride}),to:draft.to,cc:draft.cc??'',bcc:draft.bcc??'',subject:draft.subject,bodyFormat:draft.bodyFormat??'html',bodyHtml:draft.bodyHtml,bodyText:draft.bodyText,insertedSignatures:draft.insertedSignatures??[],inReplyTo:draft.inReplyTo??[],references:draft.references??[],attachments:draft.attachments??[],inline:draft.inline??[],requestReceipt:draft.requestReceipt??false,requestDeliveryReceipt:draft.requestDeliveryReceipt??false,openTracking:draft.openTracking??false,protectedContent:false};}
export function localAccountDraft(document:AccountDraftDocument):StoredDraft{
 if(!document.state)throw new DraftSaveError('draftStateUnavailable');
 return {...document.state,serverStatus:document.status,...(document.revision?{serverRevision:document.revision}:{}),id:document.id,accountId:document.accountId,storageAuthority:document.storageAuthority,savedAt:document.savedAt*1000};
}
export function accountDraftApi(owner:PreferenceAccount,signal?:AbortSignal){
 async function request<T>(path:string,method='GET',body?:unknown):Promise<T>{
  if(!owner.storageAuthority||!preferenceAccountIsCurrent(owner))throw new DraftSaveError('accountChanged');
  const response=await apiFetch(withBase(`/api/account/drafts${path}`),{method,headers:{'content-type':'application/json','x-mailwoman-account':owner.id,'x-mailwoman-storage-authority':owner.storageAuthority},...(signal?{signal}:{}),...(body===undefined?{}:{body:JSON.stringify(body)})});
  if(!preferenceAccountIsCurrent(owner))throw new DraftSaveError('accountChanged');
  const value=await response.json() as Record<string,unknown>;
  if(!response.ok)throw new DraftSaveError(typeof value.code==='string'?value.code:'draftSaveFailed');
  if(value.accountId!==owner.id||value.storageAuthority!==owner.storageAuthority)throw new DraftSaveError('accountChanged');
  return value as T;
 }
 return {resolve:(emailId:string)=>request<{draft:AccountDraftDocument|null}>(`/resolve?emailId=${encodeURIComponent(emailId)}`),discard:(id:string,expectedRevision:string|null)=>request<{deleted:boolean}>(`/${encodeURIComponent(id)}`,'DELETE',{expectedRevision}),list:()=>request<AccountDraftList>(''),load:(id:string)=>request<AccountDraftDocument>(`/${encodeURIComponent(id)}`),save:(id:string,operationId:string,expectedRevision:string|null,state:DraftEditorState)=>request<AccountDraftDocument>(`/${encodeURIComponent(id)}`,'PUT',{operationId,expectedRevision,state}),cleanup:(id:string)=>request<AccountDraftDocument>(`/${encodeURIComponent(id)}/cleanup`,'POST')};
}
