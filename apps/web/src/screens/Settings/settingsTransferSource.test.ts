import {afterEach,expect,it,vi} from 'vitest';
import {readSettingsSource} from './settingsTransferSource.ts';
import {apiFetch} from '../../api/http.ts';
import {activateAccountPreferences,preferenceAccount,suspendAccountPreferences} from '../../api/accountPreferenceLifecycle.ts';
import type {WorkspaceMailbox} from '../../api/workspaceMail.ts';
vi.mock('../../api/http.ts',()=>({apiFetch:vi.fn()}));
afterEach(()=>{vi.resetAllMocks();suspendAccountPreferences();});
it('reads portable effective preferences using the verified source authority without writing or switching accounts',async()=>{
 activateAccountPreferences('active','active-authority');const target:WorkspaceMailbox={owner:preferenceAccount()!,linkId:'source-link',accountId:'source',storageAuthority:'source-authority',label:'Work',current:false};
 vi.mocked(apiFetch).mockImplementation(async url=>Response.json(url.endsWith('/settings')?{settings:{'reader.zoom':{value:125,source:'default',locked:false,inherited:true},'mail.readReceipts':{value:'ask',source:'user',locked:false}}}:url.endsWith('/preferences')?{preferences:{keyboardPreset:'vim',folderHidden:['private-id'],analyticsConsent:true}}:{appearance:null}));
 const request=new AbortController(),backup=await readSettingsSource(target,request.signal);
 expect(backup.source.storageAuthority).toBe('source-authority');expect(backup.items).toContainEqual({kind:'setting',key:'reader.zoom',value:125});expect(backup.items).toContainEqual({kind:'interface',key:'keyboardPreset',value:'vim'});
 expect(backup.items.some(item=>['folderHidden','analyticsConsent','mail.readReceipts'].includes(item.key))).toBe(false);
 for(const [,options] of vi.mocked(apiFetch).mock.calls){expect(options?.mailboxTarget).toBe(target);expect(options?.signal).toBe(request.signal);expect(options?.method).toBeUndefined();}
 expect(preferenceAccount()?.id).toBe('active');
});
it('does not prepare a partial backup when any source endpoint refuses access',async()=>{
 activateAccountPreferences('active','active-authority');const target:WorkspaceMailbox={owner:preferenceAccount()!,linkId:'source-link',accountId:'source',storageAuthority:'source-authority',label:'Work',current:false};
 vi.mocked(apiFetch).mockResolvedValue(new Response('',{status:403}));await expect(readSettingsSource(target,new AbortController().signal)).rejects.toThrow('Source unavailable');
});
