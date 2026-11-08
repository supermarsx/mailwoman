import {preferenceAccountIsCurrent,type PreferenceAccount} from '../../api/accountPreferenceLifecycle.ts';
import {DRAFTS_CHANGED,isDraftSource,type DraftSource} from './drafts-store.ts';

/** Memory only: no recipient, subject, body, encryption state or persisted data. */
interface LiveReplyDraft {id:string;emailId:string;savedAt:number;owner:PreferenceAccount;discard?:()=>Promise<void>}
const entries=new Map<symbol,LiveReplyDraft>();
const changed=()=>window.dispatchEvent(new Event(DRAFTS_CHANGED));
export function registerLiveReplyDraft(owner:PreferenceAccount,id:string,source:DraftSource|undefined,discard?:()=>Promise<void>):()=>void {
  if(!id||!preferenceAccountIsCurrent(owner)||!isDraftSource(source)||source.keyword!=='$answered')return ()=>{};
  const key=Symbol();entries.set(key,{id,emailId:source.emailId,savedAt:Date.now(),owner,...(discard?{discard}:{})});changed();
  return ()=>{if(entries.delete(key))changed();};
}
export function liveReplyDrafts(owner:PreferenceAccount):ReadonlyArray<LiveReplyDraft> {
  return [...entries.values()].filter(entry=>entry.owner.generation===owner.generation&&entry.owner.id===owner.id&&entry.owner.storageAuthority===owner.storageAuthority&&preferenceAccountIsCurrent(entry.owner));
}
