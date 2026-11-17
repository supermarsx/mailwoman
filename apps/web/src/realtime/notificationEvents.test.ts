import {afterEach,expect,it,vi} from 'vitest';
import {activateAccountPreferences,suspendAccountPreferences} from '../api/accountPreferenceLifecycle.ts';
const fixture=vi.hoisted(()=>({values:{} as Record<string,unknown>,quiet:false,notify:vi.fn(async()=>{})}));
vi.mock('../api/notificationControls.ts',()=>({alertValue:(key:string,fallback:unknown)=>fixture.values[key]??fallback}));
vi.mock('../api/notificationRuntime.ts',()=>({connectionToastAllowed:()=>!fixture.quiet}));
vi.mock('../platform/index.ts',()=>({getPlatform:()=>({notify:fixture.notify})}));
vi.mock('../platform/capabilities.ts',()=>({capabilityEnabled:()=>false}));
import {notifyEvent} from './notificationEvents.ts';
import {toasts,resetToasts} from '../components/toast/toast.ts';
afterEach(()=>{resetToasts();suspendAccountPreferences();fixture.values={};fixture.quiet=false;fixture.notify.mockClear();vi.unstubAllGlobals();});
it('routes send outcomes independently to native, toast, both or no alert',async()=>{
 activateAccountPreferences('a','event_notification_authority');vi.stubGlobal('Notification',{permission:'granted'});
 fixture.values['notifications.events.sent']='native';notifyEvent('sent','Accepted','success');await Promise.resolve();expect(fixture.notify).toHaveBeenCalledOnce();expect(toasts()).toHaveLength(0);
 fixture.values['notifications.events.failed']='both';notifyEvent('failed','Failed','error');await Promise.resolve();expect(fixture.notify).toHaveBeenCalledTimes(2);expect(toasts()[0]?.message).toBe('Failed');
 fixture.values['notifications.events.queued']='none';notifyEvent('queued','Queued');expect(toasts()).toHaveLength(1);
 fixture.quiet=true;notifyEvent('failed','Quiet');await Promise.resolve();expect(fixture.notify).toHaveBeenCalledTimes(2);
});
