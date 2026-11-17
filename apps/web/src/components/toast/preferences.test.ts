import {afterEach,expect,it} from 'vitest';
import {loadSettings,clearSettings} from '../../api/settings.ts';
import {activateAccountPreferences,suspendAccountPreferences} from '../../api/accountPreferenceLifecycle.ts';
import {toast,toasts,resetToasts} from './toast.ts';
afterEach(()=>{resetToasts();clearSettings();suspendAccountPreferences();});
it('applies toast preferences to Open alerts while retaining undo/retry controls',async()=>{
 activateAccountPreferences('a','toast_preference_authority');
 await loadSettings({refresh:true,fetcher:async()=>Response.json({settings:{'notifications.toasts.enabled':{value:false,source:'user',locked:false}}})});
 toast.info('New mail',{action:{label:'Open',run:()=>{}},respectPreferences:true});expect(toasts()).toHaveLength(0);
 toast.error('Retry required',{action:{label:'Retry',run:()=>{}}});expect(toasts()).toHaveLength(1);
});
