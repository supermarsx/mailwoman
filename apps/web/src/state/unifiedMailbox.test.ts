import {afterEach,expect,it,vi} from 'vitest';
import {activateAccountPreferences,suspendAccountPreferences} from '../api/accountPreferenceLifecycle.ts';
import {setUnifiedConfig,activeUnifiedGroup,setActiveUnifiedGroup} from './unifiedAccountPreferences.ts';
import {requestUnifiedMailbox,setUnifiedAvailable,unifiedFolderRole} from './unifiedMailbox.ts';
vi.mock('../api/settings.ts',()=>({settings:()=>({'mail.unifiedAccount':{value:true}})}));
afterEach(()=>{suspendAccountPreferences();setUnifiedConfig({});setActiveUnifiedGroup(null);setUnifiedAvailable(false);});
it('preserves the selected named group when opening its sidebar folders',()=>{
 activateAccountPreferences('a','msa1_primary_authority');setUnifiedAvailable(true);setUnifiedConfig({groups:JSON.stringify([{id:'work',name:'Work',accounts:['*'],folders:['*']}])});
 requestUnifiedMailbox(undefined,'work');requestUnifiedMailbox('sent');expect(activeUnifiedGroup()).toBe('work');expect(unifiedFolderRole()).toBe('sent');
});
it('uses the configured startup group but keeps an explicitly selected default group',()=>{
 activateAccountPreferences('a','msa1_primary_authority');setUnifiedAvailable(true);setUnifiedConfig({primaryGroup:'work',groups:JSON.stringify([{id:'work',name:'Work',accounts:['*'],folders:['*']}])});
 requestUnifiedMailbox();expect(activeUnifiedGroup()).toBe('work');requestUnifiedMailbox(undefined,null);requestUnifiedMailbox('trash');expect(activeUnifiedGroup()).toBeNull();expect(unifiedFolderRole()).toBe('trash');
});
