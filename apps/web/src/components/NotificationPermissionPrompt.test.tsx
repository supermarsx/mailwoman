import {afterEach,expect,it,vi} from 'vitest';
import {cleanup,fireEvent,render,screen,waitFor} from '@solidjs/testing-library';
import {NotificationPermissionPrompt} from './NotificationPermissionPrompt.tsx';
import {activateAccountPreferences,suspendAccountPreferences} from '../api/accountPreferenceLifecycle.ts';
import {clearSettings,loadSettings,type SettingEntry} from '../api/settings.ts';
import type {NotificationConfig} from '../screens/Settings/types.ts';
import {toast} from './toast/toast.ts';
vi.mock('./toast/toast.ts',()=>({toast:{success:vi.fn(),error:vi.fn()}}));
const config:NotificationConfig={enabled:false,rules:[],quietHours:{enabled:false,start:'22:00',end:'07:00'}};
const entry=(value:boolean,locked=false):SettingEntry=>({value,locked,source:locked?'global':'default'});
async function seed(options:{ask?:boolean;lockedOff?:boolean;permission?:NotificationPermission}={}) {
 activateAccountPreferences('account','verified_realm_0001');
 const policy={'notifications.askPermission':entry(options.ask??true),'notifications.enabled':entry(false,options.lockedOff)};
 await loadSettings({fetcher:async()=>new Response(JSON.stringify({settings:policy}),{headers:{'content-type':'application/json'}}),refresh:true});
 const request=vi.fn(async()=> 'granted' as NotificationPermission);
 vi.stubGlobal('Notification',{permission:options.permission??'default',requestPermission:request});
 const service={notifications:vi.fn(async()=>config),saveNotifications:vi.fn(async(_value:NotificationConfig)=>{})};
 return {request,service};
}
afterEach(()=>{cleanup();suspendAccountPreferences();clearSettings();sessionStorage.clear();vi.restoreAllMocks();vi.unstubAllGlobals();vi.mocked(toast.error).mockClear();vi.mocked(toast.success).mockClear();});

it('offers permission by default and requests it only from Allow, preserving fresh notification rules',async()=>{
 const {request,service}=await seed();
 const updated:NotificationConfig={...config,rules:[{id:'mute',label:'Mute',match:'quiet',action:'mute'}]};
 service.notifications.mockResolvedValueOnce(config).mockResolvedValue(updated);
 render(()=><NotificationPermissionPrompt service={service}/>);
 const button=await screen.findByRole('button',{name:'Allow notifications'});expect(request).not.toHaveBeenCalled();expect(service.saveNotifications).not.toHaveBeenCalled();
 fireEvent.click(button);await waitFor(()=>expect(service.saveNotifications).toHaveBeenCalledExactlyOnceWith({...updated,enabled:true}));
 expect(request).toHaveBeenCalledOnce();expect(screen.queryByRole('dialog')).toBeNull();expect(toast.success).toHaveBeenCalledOnce();
});
it.each([{ask:false},{lockedOff:true},{permission:'granted' as const},{permission:'denied' as const}])('does not offer or request permission when policy/device already decides it: %o',async options=>{
 const {request,service}=await seed(options);render(()=><NotificationPermissionPrompt service={service}/>);
 await waitFor(()=>expect(screen.queryByRole('dialog')).toBeNull());
 expect(request).not.toHaveBeenCalled();expect(service.saveNotifications).not.toHaveBeenCalled();
});
it('keeps Not now dismissed for this verified account in the tab session without changing preferences',async()=>{
 const {request,service}=await seed();const first=render(()=><NotificationPermissionPrompt service={service}/>);
 fireEvent.click(await screen.findByRole('button',{name:'Not now'}));expect(screen.queryByRole('dialog')).toBeNull();first.unmount();
 render(()=><NotificationPermissionPrompt service={service}/>);await new Promise(resolve=>setTimeout(resolve,0));
 expect(screen.queryByRole('dialog')).toBeNull();expect(request).not.toHaveBeenCalled();expect(service.saveNotifications).not.toHaveBeenCalled();
});
it('never enables delivery when the device refuses permission',async()=>{
 const {request,service}=await seed();request.mockResolvedValue('denied');render(()=><NotificationPermissionPrompt service={service}/>);
 fireEvent.click(await screen.findByRole('button',{name:'Allow notifications'}));
 await waitFor(()=>expect(screen.queryByRole('dialog')).toBeNull());expect(service.saveNotifications).not.toHaveBeenCalled();expect(toast.success).not.toHaveBeenCalled();
});
it('ignores a permission result after the authenticated account generation changes',async()=>{
 const {request,service}=await seed();let resolve!:(value:NotificationPermission)=>void;request.mockImplementation(()=>new Promise(done=>{resolve=done;}));
 render(()=><NotificationPermissionPrompt service={service}/>);fireEvent.click(await screen.findByRole('button',{name:'Allow notifications'}));
 suspendAccountPreferences();activateAccountPreferences('account','different_realm_0002');resolve('granted');await new Promise(done=>setTimeout(done,0));
 expect(service.saveNotifications).not.toHaveBeenCalled();expect(toast.success).not.toHaveBeenCalled();
});
it('does not mutate notification settings after the prompt is unmounted',async()=>{
 const {request,service}=await seed();let resolve!:(value:NotificationPermission)=>void;request.mockImplementation(()=>new Promise(done=>{resolve=done;}));
 const view=render(()=><NotificationPermissionPrompt service={service}/>);fireEvent.click(await screen.findByRole('button',{name:'Allow notifications'}));view.unmount();resolve('granted');await new Promise(done=>setTimeout(done,0));
 expect(service.saveNotifications).not.toHaveBeenCalled();expect(toast.success).not.toHaveBeenCalled();
});
it('shows a toast when permission is granted but saving notification preferences fails',async()=>{
 const {service}=await seed();service.saveNotifications.mockRejectedValue(new Error('offline'));render(()=><NotificationPermissionPrompt service={service}/>);
 fireEvent.click(await screen.findByRole('button',{name:'Allow notifications'}));await waitFor(()=>expect(toast.error).toHaveBeenCalledOnce());expect(toast.success).not.toHaveBeenCalled();
});
