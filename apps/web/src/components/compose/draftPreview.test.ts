import {afterEach,expect,it,vi} from 'vitest';
import {activateAccountPreferences,preferenceAccount,suspendAccountPreferences} from '../../api/accountPreferenceLifecycle.ts';
import {clearDrafts,saveDraft,type StoredDraft} from './drafts-store.ts';
import {loadDraftPreview,registerDraftPreview} from './draftPreview.ts';
const draft=(body='Saved body'):StoredDraft=>({id:'reply',accountId:'a',storageAuthority:'draft-preview-authority-123',to:'you@example.test',subject:'Reply',bodyText:body,bodyHtml:`<p>${body}</p>`,savedAt:1});
afterEach(()=>{suspendAccountPreferences();clearDrafts();});
it('reads the latest live copy without changing or persisting the saved draft',async()=>{
 activateAccountPreferences('a','draft-preview-authority-123');const owner=preferenceAccount()!;saveDraft(draft());
 const read=vi.fn(()=>draft('Latest unsaved reply')),release=registerDraftPreview(owner,'reply',read);
 try{expect((await loadDraftPreview(owner,'reply')).bodyText).toBe('Latest unsaved reply');expect(localStorage.getItem('mw.compose.drafts.v2')).toContain('Saved body');expect(localStorage.getItem('mw.compose.drafts.v2')).not.toContain('Latest unsaved');}finally{release();}
 expect((await loadDraftPreview(owner,'reply')).bodyText).toBe('Saved body');
});
it('supports private live previews without storage authority and refuses them after account replacement',async()=>{
 activateAccountPreferences('a');const owner=preferenceAccount()!,release=registerDraftPreview(owner,'reply',()=>({...draft('Private unsaved reply'),storageAuthority:''}));
 try{expect((await loadDraftPreview(owner,'reply')).bodyText).toBe('Private unsaved reply');activateAccountPreferences('b');await expect(loadDraftPreview(owner,'reply')).rejects.toThrow('accountChanged');}finally{release();}
});
