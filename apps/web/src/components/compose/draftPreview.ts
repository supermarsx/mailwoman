import {preferenceAccount,preferenceAccountIsCurrent,type PreferenceAccount} from '../../api/accountPreferenceLifecycle.ts';
import {accountDraftApi,localAccountDraft} from '../../api/accountDrafts.ts';
import {listDrafts,type StoredDraft} from './drafts-store.ts';
import {createSignal} from 'solid-js';

// Providers contain callbacks only. Previewing private/unsaved content never persists it.
const providers=new Map<symbol,{owner:PreferenceAccount;id:string;read:()=>StoredDraft|null}>();
export function registerDraftPreview(owner:PreferenceAccount,id:string,read:()=>StoredDraft|null):()=>void{
 const key=Symbol();providers.set(key,{owner,id,read});return ()=>{providers.delete(key);};
}
export async function loadDraftPreview(owner:PreferenceAccount,id:string,signal?:AbortSignal):Promise<StoredDraft>{
 if(!preferenceAccountIsCurrent(owner))throw new Error('accountChanged');
 const live=[...providers.values()].reverse().find(value=>value.id===id&&value.owner.generation===owner.generation&&value.owner.id===owner.id&&value.owner.storageAuthority===owner.storageAuthority&&preferenceAccountIsCurrent(value.owner));
 const snapshot=live?.read();
 if(snapshot){if(snapshot.id!==id||snapshot.accountId!==owner.id||snapshot.storageAuthority!==(owner.storageAuthority??''))throw new Error('accountChanged');return snapshot;}
 if(!owner.storageAuthority)throw new Error('draftMissing');
 const local=listDrafts(owner.id,owner.storageAuthority).find(value=>value.id===id);
 if(local)return structuredClone(local);
 const remote=await accountDraftApi(owner,signal).load(id);
 if(!preferenceAccountIsCurrent(owner))throw new Error('accountChanged');
 return localAccountDraft(remote);
}
const [request,setRequest]=createSignal<{id:string;owner:PreferenceAccount;message?:boolean}>();
export const draftPreviewRequest=request;
// Start after the initiating click reaches the shell activity guard.
export function requestDraftPreview(id:string):void{const owner=preferenceAccount();if(owner&&preferenceAccountIsCurrent(owner))queueMicrotask(()=>{if(preferenceAccountIsCurrent(owner))setRequest({id,owner});});}
export function requestDraftMessagePreview(id:string):void{const owner=preferenceAccount();if(owner&&preferenceAccountIsCurrent(owner))setRequest({id,owner,message:true});}
export function takeDraftPreviewRequest():{id:string;message?:boolean}|null{const value=request();setRequest(undefined);return value&&preferenceAccountIsCurrent(value.owner)?value:null;}
