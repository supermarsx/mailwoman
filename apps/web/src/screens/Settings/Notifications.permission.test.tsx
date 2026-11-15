import {afterEach,expect,it,vi} from 'vitest';
import {cleanup,fireEvent,render,screen,waitFor} from '@solidjs/testing-library';
import {Notifications} from './Notifications.tsx';
import type {NotificationConfig} from './types.ts';
import {SettingsService} from './service.ts';
import {activateAccountPreferences,suspendAccountPreferences} from '../../api/accountPreferenceLifecycle.ts';
import {clearSettings,loadSettings} from '../../api/settings.ts';
const config={enabled:false,rules:[],quietHours:{enabled:false,start:'22:00',end:'07:00'}};
afterEach(()=>{cleanup();suspendAccountPreferences();clearSettings();});
async function mount(locked=false,save=vi.fn<(config:NotificationConfig)=>Promise<void>>(async()=>{})){
 activateAccountPreferences('acct','notification_settings_realm');const writes:unknown[]=[];
 const entry=(value:boolean)=>({value,locked,source:locked?'global':'default'});
 await loadSettings({fetcher:async(_url,init)=>{if(init?.method==='PUT'){const body=JSON.parse(String(init.body));writes.push(body);return new Response(JSON.stringify({...entry(body.value),source:'user'}));}return new Response(JSON.stringify({settings:{'notifications.view.enabled':entry(true),'notifications.view.showInSidebar':entry(true),'notifications.enabled':entry(false),'notifications.askPermission':entry(true)}}));},refresh:true});
 const service={notifications:async()=>config,saveNotifications:save} as unknown as SettingsService;
 const view=render(()=><Notifications service={service}/>);const checkbox=await screen.findByRole('checkbox',{name:'Offer notification permission on new devices'});
 return {checkbox,writes,save,view};
}
it('defaults to offering permission and saves the user override immediately',async()=>{
 const {checkbox,writes,save}=await mount();await waitFor(()=>expect(checkbox).toBeEnabled());expect(checkbox).toBeChecked();fireEvent.click(checkbox);
 await waitFor(()=>expect(writes).toEqual([{value:false}]));expect(checkbox).not.toBeChecked();expect(save).not.toHaveBeenCalled();
});
it('keeps an administrator-held prompt preference read-only',async()=>{
 const {checkbox,writes}=await mount(true);expect(checkbox).toBeChecked();expect(checkbox).toBeDisabled();fireEvent.click(checkbox);expect(writes).toEqual([]);
});

it('saves view visibility separately from mail notification rules and disables the sidebar control while hidden',async()=>{
 const {writes,save}=await mount();const view=screen.getByRole('checkbox',{name:'Enable the Notifications view'});const sidebar=screen.getByRole('checkbox',{name:'Show Notifications in the sidebar'});
 expect(view).toBeChecked();expect(sidebar).toBeEnabled();fireEvent.click(view);
 await waitFor(()=>expect(view).not.toBeChecked());await waitFor(()=>expect(sidebar).toBeDisabled());expect(writes).toEqual([{value:false}]);expect(save).not.toHaveBeenCalled();
});
it('honours administrator locks on both view controls',async()=>{
 await mount(true);for(const name of ['Enable the Notifications view','Show Notifications in the sidebar'])expect(screen.getByRole('checkbox',{name})).toBeDisabled();
});

it('automatically saves notification changes without a Save button',async()=>{
 const {save}=await mount();const enabled=screen.getByRole('checkbox',{name:'Show notifications for new mail'});await waitFor(()=>expect(enabled).toBeEnabled());fireEvent.click(enabled);
 expect(screen.queryByTestId('notif-save')).toBeNull();await waitFor(()=>expect(save).toHaveBeenCalledWith(expect.objectContaining({enabled:true})));
 await screen.findByTestId('notif-saved');
});
it('serializes auto-saves and persists edits made while the previous request is in flight',async()=>{
 let finish!:()=>void;const save=vi.fn<(config:NotificationConfig)=>Promise<void>>().mockImplementationOnce(()=>new Promise<void>(resolve=>{finish=resolve;})).mockResolvedValue(undefined);
 await mount(false,save);fireEvent.click(screen.getByRole('checkbox',{name:'Show notifications for new mail'}));await waitFor(()=>expect(save).toHaveBeenCalledTimes(1));
 const quiet=document.getElementById('notif-quiet-enabled')!;expect(quiet).toBeEnabled();fireEvent.click(quiet);finish();
 await waitFor(()=>expect(save).toHaveBeenCalledTimes(2));expect(save.mock.calls[1]?.[0]).toMatchObject({enabled:true,quietHours:{enabled:true}});
});
it('flushes a pending auto-save when leaving the section',async()=>{
 const {save,view}=await mount();fireEvent.click(screen.getByRole('checkbox',{name:'Show notifications for new mail'}));view.unmount();
 await waitFor(()=>expect(save).toHaveBeenCalledWith(expect.objectContaining({enabled:true})));
});
