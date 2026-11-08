import {createEffect,createMemo,createSignal,onCleanup,type Accessor} from 'solid-js';
import {observePreferenceAccount,preferenceAccount,preferenceAccountIsCurrent,type PreferenceAccount} from '../../api/accountPreferenceLifecycle.ts';
import {accountDraftApi,type AccountDraftDocument} from '../../api/accountDrafts.ts';
import {DRAFTS_CHANGED,listDrafts,deleteDraft,isDraftSource,type DraftChange} from './drafts-store.ts';
import {liveReplyDrafts} from './liveReplyDrafts.ts';

export interface ReplyDraft {id:string;emailId:string;savedAt:number;serverRevision?:string|null;serverStatus?:string}
/** Only explicit source metadata identifies a pending reply. Subjects/thread ids
 * are not evidence that an arbitrary draft answers this particular message. */
export function createReplyDraftIndex(account:Accessor<string|null>):Accessor<ReadonlyMap<string,ReplyDraft>> {
  let request=0;
  const removed=new Set<string>();
  const [revision,bump]=createSignal(0),[authority,refreshAuthority]=createSignal(0),[remote,setRemote]=createSignal<AccountDraftDocument[]>([]);
  const changed=(event?:Event)=>{
    const change=event instanceof CustomEvent?event.detail as DraftChange|undefined:undefined,owner=preferenceAccount();
    if(change?.kind==='clear'){request++;setRemote([]);}
    else if(change?.accountId===account()&&change.storageAuthority===owner?.storageAuthority){if(change.kind==='delete'&&change.id){removed.add(change.id);setRemote(old=>old.filter(draft=>draft.id!==change.id));}else if(change.kind==='save'&&change.id)removed.delete(change.id);}
    bump(value=>value+1);
  };
  window.addEventListener(DRAFTS_CHANGED,changed);window.addEventListener('storage',changed);
  const unwatch=observePreferenceAccount(()=>{changed();refreshAuthority(value=>value+1);});
  onCleanup(()=>{unwatch();window.removeEventListener(DRAFTS_CHANGED,changed);window.removeEventListener('storage',changed);});
  createEffect(()=>{
    authority();const id=account(),owner=preferenceAccount();const generation=++request;removed.clear();setRemote([]);
    if(!id||!owner?.storageAuthority||owner.id!==id)return;
    const controller=new AbortController();onCleanup(()=>controller.abort());
    // One bounded account summary request, never a per-row/message fetch.
    const timer=setTimeout(()=>controller.abort(),10_000);onCleanup(()=>clearTimeout(timer));
    void accountDraftApi(owner,controller.signal).list().then(result=>{
      if(generation===request&&preferenceAccountIsCurrent(owner)&&account()===id)setRemote(result.items);
    }).catch(()=>{/* Local saved replies remain available offline. */}).finally(()=>clearTimeout(timer));
  });
  return createMemo(()=>{
    revision();const id=account(),owner=preferenceAccount(),index=new Map<string,ReplyDraft>();
    if(!id||!owner||owner.id!==id||!preferenceAccountIsCurrent(owner))return index;
    const add=(draft:ReplyDraft)=>{const previous=index.get(draft.emailId);if(!previous||previous.savedAt<draft.savedAt)index.set(draft.emailId,draft);};
    const local=listDrafts(id,owner.storageAuthority),localIds=new Set(local.map(draft=>draft.id));
    for(const draft of remote())if(!removed.has(draft.id)&&draft.accountId===id&&draft.storageAuthority===owner.storageAuthority&&!localIds.has(draft.id)&&isDraftSource(draft.summary.source)&&draft.summary.source.keyword==='$answered')add({id:draft.id,emailId:draft.summary.source.emailId,savedAt:draft.savedAt*1000,serverRevision:draft.revision,serverStatus:draft.status});
    for(const draft of local)if(draft.source?.keyword==='$answered')add({id:draft.id,emailId:draft.source.emailId,savedAt:draft.savedAt,...(draft.serverRevision?{serverRevision:draft.serverRevision}:{}),...(draft.serverStatus?{serverStatus:draft.serverStatus}:{})});
    // An active composer wins over saved copies, including privacy-protected
    // replies that intentionally never enter local or remote draft storage.
    for(const draft of liveReplyDrafts(owner))index.set(draft.emailId,{id:draft.id,emailId:draft.emailId,savedAt:draft.savedAt});
    return index;
  });
}

/** Discard exactly the copy the reader offered, never a newer account revision.
 * Active composers own their writer and cleanup, including unsaved private replies. */
export async function discardReplyDraft(owner:PreferenceAccount,draft:ReplyDraft):Promise<void>{
 if(!preferenceAccountIsCurrent(owner)||!owner.storageAuthority)throw new Error('accountChanged');
 const live=liveReplyDrafts(owner).find(value=>value.id===draft.id&&value.emailId===draft.emailId);
 if(live){if(!live.discard)throw new Error('draftBusy');await live.discard();return;}
 if(draft.serverStatus==='uncertain'||draft.serverStatus==='refused')throw new Error('draftConflict');
 const local=listDrafts(owner.id,owner.storageAuthority).find(value=>value.id===draft.id);
 if(local&&(local.serverRevision!==draft.serverRevision||local.savedAt!==draft.savedAt||local.source?.emailId!==draft.emailId||local.source.keyword!=='$answered'))throw new Error('draftConflict');
 if(draft.serverRevision!==undefined){
  const api=accountDraftApi(owner),remote=await api.load(draft.id);
  if(remote.revision!==draft.serverRevision||remote.summary.source?.emailId!==draft.emailId||remote.summary.source.keyword!=='$answered')throw new Error('draftConflict');
  await api.discard(draft.id,draft.serverRevision);
 }else if(!local)throw new Error('draftMissing');
 if(!preferenceAccountIsCurrent(owner))throw new Error('accountChanged');
 const latest=listDrafts(owner.id,owner.storageAuthority).find(value=>value.id===draft.id);
 if(latest&&(latest.savedAt!==draft.savedAt||latest.serverRevision!==draft.serverRevision))throw new Error('draftConflict');
 deleteDraft(draft.id,owner.id,owner.storageAuthority);
}
