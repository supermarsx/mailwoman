import {createEffect,createMemo,createSignal,onCleanup,type Accessor} from 'solid-js';
import {observePreferenceAccount,preferenceAccount,preferenceAccountIsCurrent} from '../../api/accountPreferenceLifecycle.ts';
import {accountDraftApi,type AccountDraftDocument} from '../../api/accountDrafts.ts';
import {DRAFTS_CHANGED,listDrafts,isDraftSource,type DraftChange} from './drafts-store.ts';
import {liveReplyDrafts} from './liveReplyDrafts.ts';

export interface ReplyDraft {id:string;emailId:string;savedAt:number}
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
    for(const draft of remote())if(!removed.has(draft.id)&&draft.accountId===id&&draft.storageAuthority===owner.storageAuthority&&!localIds.has(draft.id)&&isDraftSource(draft.summary.source)&&draft.summary.source.keyword==='$answered')add({id:draft.id,emailId:draft.summary.source.emailId,savedAt:draft.savedAt*1000});
    for(const draft of local)if(draft.source?.keyword==='$answered')add({id:draft.id,emailId:draft.source.emailId,savedAt:draft.savedAt});
    // An active composer wins over saved copies, including privacy-protected
    // replies that intentionally never enter local or remote draft storage.
    for(const draft of liveReplyDrafts(owner))index.set(draft.emailId,{id:draft.id,emailId:draft.emailId,savedAt:draft.savedAt});
    return index;
  });
}
