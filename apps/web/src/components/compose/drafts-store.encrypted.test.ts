import {webcrypto} from 'node:crypto';
import {beforeEach,expect,it,vi} from 'vitest';
import {EncryptedCache,memoryBackend} from '../../offline/opfs.ts';
import {generateProfileKey} from '../../offline/crypto.ts';
import {createOfflineVault,type OfflineVault,type VaultEntry} from '../../offline/vault.ts';
const shared=vi.hoisted(()=>({vault:undefined as OfflineVault|undefined}));
vi.mock('../../offline/vault.ts',async original=>({...await original<typeof import('../../offline/vault.ts')>(),offlineVault:async()=>shared.vault!}));
const draft={id:'d1',accountId:'a',storageAuthority:'test_verified_owner_123456',to:'person@example.test',subject:'Private recovery subject',bodyHtml:'<p>Private recovery body</p>',bodyText:'Private recovery body',savedAt:1};
let records:Map<string,VaultEntry>,backend:ReturnType<typeof memoryBackend>;
beforeEach(async()=>{
 vi.stubGlobal('crypto',webcrypto);vi.resetModules();localStorage.clear();records=new Map();backend=memoryBackend();
 shared.vault=createOfflineVault(new EncryptedCache(backend,await generateProfileKey()),{
  async all(namespace){return [...records.values()].filter(row=>row.namespace===namespace);},
  async put(row){records.set(row.id,row);},async remove(id){records.delete(id);},
 });
});
it('recovers drafts after a module reload without writing plaintext to localStorage',async()=>{
 const first=await import('./drafts-store.ts');first.saveDraft(draft);await first.flushDrafts();
 expect(localStorage.getItem('mw.compose.drafts.v2')).toBeNull();
 const id=[...records.keys()][0]!;expect(new TextDecoder().decode((await backend.read(`vault/${id}.enc`))!)).not.toContain(draft.subject);
 vi.resetModules();const reloaded=await import('./drafts-store.ts');await reloaded.hydrateDrafts();
 expect(reloaded.listDrafts(draft.accountId,draft.storageAuthority)).toEqual([draft]);
});
it('seals legacy recovery data before removing it and retains it on failure',async()=>{
 localStorage.setItem('mw.compose.drafts.v2',JSON.stringify([draft]));
 const api=await import('./drafts-store.ts');expect(api.listDrafts(draft.accountId,draft.storageAuthority)).toEqual([draft]);
 await api.flushDrafts();expect(localStorage.getItem('mw.compose.drafts.v2')).toBeNull();
 expect(await shared.vault!.list('composer-drafts')).toEqual([{key:JSON.stringify([draft.storageAuthority,draft.accountId,draft.id]),value:draft}]);
});
it('does not discard a legacy draft if its encrypted replacement cannot be saved',async()=>{
 vi.spyOn(shared.vault!,'put').mockRejectedValue(new Error('disk full'));
 const raw=JSON.stringify([draft]);localStorage.setItem('mw.compose.drafts.v2',raw);
 const api=await import('./drafts-store.ts');api.listDrafts(draft.accountId,draft.storageAuthority);await api.flushDrafts();
 expect(localStorage.getItem('mw.compose.drafts.v2')).toBe(raw);expect(api.draftPersistenceState()).toBe('unavailable');
});
