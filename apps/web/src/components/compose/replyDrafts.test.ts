import {beforeEach,afterEach,expect,it,vi} from 'vitest';
import {createRoot,createSignal} from 'solid-js';
import {waitFor} from '@solidjs/testing-library';
import {activateAccountPreferences,suspendAccountPreferences,preferenceAccount} from '../../api/accountPreferenceLifecycle.ts';
import {registerLiveReplyDraft} from './liveReplyDrafts.ts';
import {draftEditorState,localAccountDraft,type AccountDraftDocument} from '../../api/accountDrafts.ts';
import {createReplyDraftIndex} from './replyDrafts.ts';
import {clearDrafts,deleteDraft,saveDraft,listDrafts,type StoredDraft} from './drafts-store.ts';
const authority='test-storage-authority-a';
const draft=(id:string,over:Partial<StoredDraft>={}):StoredDraft=>({id,accountId:'a',storageAuthority:authority,to:'person@example.org',subject:'Reply',bodyHtml:'<p>Reply text</p>',bodyText:'Reply text',savedAt:1,source:{emailId:'message',keyword:'$answered'},...over});
const remote=(id:string,over:Partial<AccountDraftDocument>={}):AccountDraftDocument=>({id,accountId:'a',storageAuthority:authority,revision:'r1',emailId:'server-draft',savedAt:2,status:'saved',cleanupPending:false,summary:{source:{emailId:'message',keyword:'$answered'}},...over});
let dispose:()=>void;
beforeEach(()=>{localStorage.clear();activateAccountPreferences('a',authority);vi.stubGlobal('fetch',vi.fn(async()=>new Response(JSON.stringify({accountId:'a',storageAuthority:authority,supported:true,items:[]}))));});
afterEach(()=>{dispose?.();suspendAccountPreferences();vi.unstubAllGlobals();});
function index(){return createRoot(stop=>{dispose=stop;const [account,setAccount]=createSignal<string|null>('a');return {value:createReplyDraftIndex(account),setAccount};});}
it('updates pending state on save/delete without falsely treating a draft or forward as answered',async()=>{
 const state=index();saveDraft(draft('reply'));expect(state.value().get('message')?.id).toBe('reply');
 saveDraft(draft('forward',{source:{emailId:'other',keyword:'$forwarded'}}));const legacy=draft('legacy');delete legacy.source;saveDraft(legacy);
 expect([...state.value().keys()]).toEqual(['message']);deleteDraft('reply','a',authority);expect(state.value().size).toBe(0);
 expect(vi.mocked(fetch)).toHaveBeenCalledTimes(1);clearDrafts();expect(state.value().size).toBe(0);
});
it('persists exact source metadata through local and restored account draft state',()=>{
 const saved=draft('reply');saveDraft(saved);expect(listDrafts('a',authority)[0]?.source).toEqual(saved.source);
 const restored=localAccountDraft(remote('reply',{state:draftEditorState(saved)}));expect(restored.source).toEqual(saved.source);
 saveDraft(draft('invalid',{source:{emailId:'',keyword:'$answered'}}));expect(listDrafts('a',authority)).toHaveLength(1);
});
it('prefers a live reply without persistence and falls back to its saved copy when closed',()=>{
 const state=index(),release=registerLiveReplyDraft(preferenceAccount()!,'live',{emailId:'message',keyword:'$answered'});
 try{expect(state.value().get('message')?.id).toBe('live');expect(listDrafts('a',authority)).toEqual([]);saveDraft(draft('saved',{savedAt:Date.now()+1000}));expect(state.value().get('message')?.id).toBe('live');}
 finally{release();}
 expect(state.value().get('message')?.id).toBe('saved');
 const old=registerLiveReplyDraft(preferenceAccount()!,'previous',{emailId:'other',keyword:'$answered'});
 activateAccountPreferences('a','replacement-storage-authority');expect(state.value().size).toBe(0);old();
});
it('includes remote-only own-account replies and prefers the current local source for the same draft',async()=>{
 vi.stubGlobal('fetch',vi.fn(async()=>new Response(JSON.stringify({accountId:'a',storageAuthority:authority,supported:true,items:[remote('remote'),remote('foreign',{storageAuthority:'other-storage-authority'}),remote('legacy',{summary:{}})]}))));
 const state=index();await waitFor(()=>expect(state.value().get('message')?.id).toBe('remote'));
 saveDraft(draft('remote',{source:{emailId:'new-message',keyword:'$answered'}}));expect(state.value().has('message')).toBe(false);expect(state.value().get('new-message')?.id).toBe('remote');
});
it('drops late remote metadata after account authority changes or a local deletion',async()=>{
 let release!:(response:Response)=>void;vi.stubGlobal('fetch',vi.fn(()=>new Promise<Response>(resolve=>{release=resolve;})));
 const state=index();await waitFor(()=>expect(fetch).toHaveBeenCalledOnce());deleteDraft('late','a',authority);
 release(new Response(JSON.stringify({accountId:'a',storageAuthority:authority,supported:true,items:[remote('late')]})));await new Promise(resolve=>setTimeout(resolve,0));expect(state.value().size).toBe(0);
 saveDraft(draft('local'));expect(state.value().size).toBe(1);activateAccountPreferences('a','replacement-storage-authority');expect(state.value().size).toBe(0);
 state.setAccount('b');expect(state.value().size).toBe(0);
});
