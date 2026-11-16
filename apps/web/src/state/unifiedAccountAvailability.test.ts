import {afterEach,expect,it,vi} from 'vitest';
import {activateAccountPreferences,suspendAccountPreferences} from '../api/accountPreferenceLifecycle.ts';
import {setUnifiedConfig,unifiedAccountEnabled} from './unifiedAccountPreferences.ts';
import {requestUnifiedMailbox,setUnifiedAvailable,unifiedMailboxRequest,takeUnifiedMailboxRequest} from './unifiedMailbox.ts';
const fixture=vi.hoisted(()=>({link:null as string|null,enabled:false}));
vi.mock('../api/settings.ts',()=>({settings:()=>({'mail.unifiedAccount':{value:fixture.enabled}})}));
vi.mock('../api/linkedMailboxSession.ts',()=>({activeLinkedMailbox:()=>fixture.link,onLinkedMailboxChange:()=>()=>undefined}));
afterEach(()=>{suspendAccountPreferences();setUnifiedConfig({});setUnifiedAvailable(false);fixture.link=null;fixture.enabled=false;});
it('shows and opens a verified parent virtual account from a linked mailbox despite its local default being off',()=>{
 activateAccountPreferences('child','msa1_child_authority');fixture.link='work';setUnifiedAvailable(true);setUnifiedConfig({enabled:true});
 expect(unifiedAccountEnabled()).toBe(true);requestUnifiedMailbox();expect(unifiedMailboxRequest()).toBeDefined();expect(takeUnifiedMailboxRequest()).toBe(true);
});
it('preserves a parent policy denial and the requirement for multiple linked accounts',()=>{
 activateAccountPreferences('child','msa1_child_authority');fixture.link='work';fixture.enabled=true;setUnifiedConfig({enabled:false});setUnifiedAvailable(true);
 expect(unifiedAccountEnabled()).toBe(false);requestUnifiedMailbox();expect(unifiedMailboxRequest()).toBeUndefined();
 setUnifiedConfig({enabled:true});setUnifiedAvailable(false);requestUnifiedMailbox();expect(unifiedMailboxRequest()).toBeUndefined();
});
it('uses live primary settings when returning to the primary mailbox',()=>{setUnifiedConfig({enabled:true});fixture.enabled=false;expect(unifiedAccountEnabled()).toBe(false);fixture.enabled=true;expect(unifiedAccountEnabled()).toBe(true);});
