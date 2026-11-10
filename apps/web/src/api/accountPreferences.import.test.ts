import {afterEach,expect,it,vi} from 'vitest';
import {waitFor} from '@solidjs/testing-library';
import {activateAccountPreferences,preferenceAccount,suspendAccountPreferences} from './accountPreferenceLifecycle.ts';
import {accountUiPreferences,accountUiPreferenceStatus,importAccountUiPreferences,setAccountUiPreferences,startAccountUiPreferences,retryAccountUiPreferences,type AccountUiPreferences,type AccountPreferencesApi} from './accountPreferences.ts';
let stop:(()=>void)|undefined;
afterEach(()=>{stop?.();stop=undefined;suspendAccountPreferences();});
it('validates imports on the server before touching local preferences and keeps later saves healthy after rejection',async()=>{
 let stored:AccountUiPreferences={direction:'ltr',keyboardPreset:'default'};let refused=true;
 const api:AccountPreferencesApi={load:async()=>({preferences:stored,etag:'"1"'}),save:async value=>{if(refused)throw new Error('invalid');stored=value;return {preferences:stored,etag:'"2"'};}};
 activateAccountPreferences('import-a');const owner=preferenceAccount()!;stop=startAccountUiPreferences(api);
 await waitFor(()=>expect(accountUiPreferenceStatus()).toBe('synced'));
 await expect(importAccountUiPreferences({keyboardPreset:'vim'},owner)).rejects.toThrow();
 expect(accountUiPreferences().keyboardPreset).toBe('default');
 refused=false;setAccountUiPreferences({direction:'rtl'});await retryAccountUiPreferences();
 expect(stored).toEqual({direction:'rtl',keyboardPreset:'default'});
 await importAccountUiPreferences({keyboardPreset:'vim'},owner);
 expect(accountUiPreferences()).toEqual({direction:'rtl',keyboardPreset:'vim'});
});
it('does not send a save after the account changes during the import preflight',async()=>{
 let loads=0;const save=vi.fn();
 const api:AccountPreferencesApi={load:async()=>{if(++loads===2)activateAccountPreferences('import-b');return {preferences:{direction:'ltr'},etag:'"1"'};},save};
 activateAccountPreferences('import-a');const owner=preferenceAccount()!;stop=startAccountUiPreferences(api);
 await waitFor(()=>expect(accountUiPreferenceStatus()).toBe('synced'));
 await expect(importAccountUiPreferences({direction:'rtl'},owner)).rejects.toThrow();expect(save).not.toHaveBeenCalled();
});
