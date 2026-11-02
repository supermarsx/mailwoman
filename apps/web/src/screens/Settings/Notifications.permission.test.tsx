import {afterEach,expect,it,vi} from 'vitest';
import {cleanup,fireEvent,render,screen,waitFor} from '@solidjs/testing-library';
import {Notifications} from './Notifications.tsx';
import {SettingsService} from './service.ts';
import {activateAccountPreferences,suspendAccountPreferences} from '../../api/accountPreferenceLifecycle.ts';
import {clearSettings,loadSettings} from '../../api/settings.ts';
const config={enabled:false,rules:[],quietHours:{enabled:false,start:'22:00',end:'07:00'}};
afterEach(()=>{cleanup();suspendAccountPreferences();clearSettings();});
async function mount(locked=false){
 activateAccountPreferences('acct','notification_settings_realm');const writes:unknown[]=[];
 const entry=(value:boolean)=>({value,locked,source:locked?'global':'default'});
 await loadSettings({fetcher:async(_url,init)=>{if(init?.method==='PUT'){const body=JSON.parse(String(init.body));writes.push(body);return new Response(JSON.stringify({...entry(body.value),source:'user'}));}return new Response(JSON.stringify({settings:{'notifications.enabled':entry(false),'notifications.askPermission':entry(true)}}));},refresh:true});
 const save=vi.fn();const service={notifications:async()=>config,saveNotifications:save} as unknown as SettingsService;
 render(()=><Notifications service={service}/>);const checkbox=await screen.findByRole('checkbox',{name:'Offer notification permission on new devices'});
 return {checkbox,writes,save};
}
it('defaults to offering permission and saves the user override immediately',async()=>{
 const {checkbox,writes,save}=await mount();await waitFor(()=>expect(checkbox).toBeEnabled());expect(checkbox).toBeChecked();fireEvent.click(checkbox);
 await waitFor(()=>expect(writes).toEqual([{value:false}]));expect(checkbox).not.toBeChecked();expect(save).not.toHaveBeenCalled();
});
it('keeps an administrator-held prompt preference read-only',async()=>{
 const {checkbox,writes}=await mount(true);expect(checkbox).toBeChecked();expect(checkbox).toBeDisabled();fireEvent.click(checkbox);expect(writes).toEqual([]);
});
