import { afterEach, describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen, waitFor } from '@solidjs/testing-library';
import { BrowserPermissions } from './BrowserPermissions.tsx';
import { activateAccountPreferences, suspendAccountPreferences } from '../../api/accountPreferenceLifecycle.ts';
import { t } from '../../test/i18n.ts';
void t;
const request=vi.hoisted(()=>vi.fn());
const pushSubscribe=vi.hoisted(()=>vi.fn());
vi.mock('../../platform/index.ts',()=>({getPlatform:()=>({pushSubscribe})}));
const errorToast=vi.hoisted(()=>vi.fn());
vi.mock('../../components/toast/toast.ts',()=>({toast:{error:errorToast}}));
vi.mock('../../api/http.ts',()=>({apiFetch:request}));

afterEach(() => { request.mockReset();errorToast.mockReset();pushSubscribe.mockReset(); delete (navigator as unknown as {serviceWorker?:unknown}).serviceWorker; vi.unstubAllGlobals(); suspendAccountPreferences(); delete (navigator as unknown as { storage?: unknown }).storage; });
describe('explicit browser storage permissions', () => {
  it('reports real status and only requests persistence on an allowed click', async () => {
    const persist=vi.fn(async()=>true);
    Object.defineProperty(navigator,'storage',{configurable:true,value:{persist,persisted:async()=>false,estimate:async()=>({usage:1048576})}});
    render(() => <BrowserPermissions enabled={false} pushAllowed={false} persistentStorageAllowed={true} />);
    await screen.findByText('Current site storage: 1.0 MiB.');
    expect(persist).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button',{name:'Request persistent storage'}));
    await waitFor(()=>expect(persist).toHaveBeenCalledOnce());
    expect(await screen.findByText('The browser has granted persistent storage for this site.')).toBeInTheDocument();
  });
  it('does not request storage when administrator policy refuses it', async () => {
    const persist=vi.fn(async()=>true);
    Object.defineProperty(navigator,'storage',{configurable:true,value:{persist,persisted:async()=>false,estimate:async()=>({})}});
    activateAccountPreferences('account');
    render(() => <BrowserPermissions enabled={true} pushAllowed={false} persistentStorageAllowed={false} />);
    expect(screen.getByRole('button',{name:'Request persistent storage'})).toBeDisabled();
    expect(persist).not.toHaveBeenCalled();
  });
});

it('does not attribute a browser push endpoint or late old registration to another realm',async()=>{
 vi.stubGlobal('PushManager',class {});vi.stubGlobal('crypto',{subtle:{digest:async()=>new Uint8Array(32).buffer}});
 Object.defineProperty(navigator,'serviceWorker',{configurable:true,value:{getRegistration:async()=>({pushManager:{getSubscription:async()=>({endpoint:'https://push.example.test/browser'})}})}});
 let release:(response:Response)=>void=()=>{};
 request.mockImplementationOnce(()=>new Promise<Response>(resolve=>{release=resolve;})).mockImplementation(async()=>new Response('{"registered":false,"allowed":false}',{status:200}));
 activateAccountPreferences('1','msa1_first_verified_authority');
 render(()=><BrowserPermissions enabled={true} pushAllowed={true} persistentStorageAllowed={false}/>);
 await waitFor(()=>expect(request).toHaveBeenCalledTimes(1));
 suspendAccountPreferences();activateAccountPreferences('1','msa1_second_verified_authority');
 await waitFor(()=>expect(request).toHaveBeenCalledTimes(2));release(new Response('{"registered":true,"allowed":true}',{status:200}));
 await waitFor(()=>expect(screen.getByText(t('settings-permissions-push-unlinked'))).toBeInTheDocument());
 expect(new Headers(request.mock.calls[1]?.[1]?.headers).get('x-mailwoman-storage-authority')).toBe('msa1_second_verified_authority');
});

it('reports a browser action failure once as a toast without an inline alert',async()=>{
 Object.defineProperty(navigator,'storage',{configurable:true,value:{persist:vi.fn(async()=>{throw new Error('blocked');}),persisted:async()=>false,estimate:async()=>({})}});
 render(()=><BrowserPermissions enabled={true} pushAllowed={false} persistentStorageAllowed={true}/>);
 fireEvent.click(screen.getByRole('button',{name:'Request persistent storage'}));
 await waitFor(()=>expect(errorToast).toHaveBeenCalledWith(t('settings-permissions-error')));
 expect(errorToast).toHaveBeenCalledTimes(1);expect(screen.queryByRole('alert')).toBeNull();
});

function readyPush(permission:'default'|'denied'|'granted'='default'){
 vi.stubGlobal('PushManager',class {});const requestPermission=vi.fn(async()=>permission==='default'?'granted':permission);
 vi.stubGlobal('Notification',{permission,requestPermission});Object.defineProperty(navigator,'serviceWorker',{configurable:true,value:{getRegistration:async()=>undefined}});
 activateAccountPreferences('a','msa1_push_settings_verified_authority');pushSubscribe.mockResolvedValue({transport:'webpush',endpoint:'https://push.example/browser',keys:{p256dh:'P',auth:'A'},appId:null,expiresAt:null});request.mockResolvedValue(new Response('{}'));
 render(()=><BrowserPermissions enabled pushAllowed persistentStorageAllowed={false}/>);return {requestPermission};
}
it('requests permission on Register and then binds the subscription to the verified account',async()=>{
 const {requestPermission}=readyPush();const button=screen.getByRole('button',{name:'Register background notifications'});fireEvent.click(button);
 await waitFor(()=>expect(pushSubscribe).toHaveBeenCalledOnce());expect(requestPermission).toHaveBeenCalledOnce();
 await screen.findByText(t('settings-permissions-push-registered'));expect(button).toBeDisabled();expect(errorToast).not.toHaveBeenCalled();
 expect(request).toHaveBeenCalledWith('/api/push/subscribe',expect.objectContaining({method:'POST',headers:expect.objectContaining({'x-mailwoman-storage-authority':'msa1_push_settings_verified_authority'})}));
});
it('explains denied notification permission without attempting a push subscription',async()=>{
 readyPush('denied');fireEvent.click(screen.getByRole('button',{name:'Register background notifications'}));
 await waitFor(()=>expect(errorToast).toHaveBeenCalledWith(t('settings-notif-permission-denied')));expect(pushSubscribe).not.toHaveBeenCalled();expect(request).not.toHaveBeenCalled();
});
it('reports server refusal separately from browser permission failures',async()=>{
 readyPush('granted');request.mockResolvedValue(new Response('',{status:503}));fireEvent.click(screen.getByRole('button',{name:'Register background notifications'}));
 await waitFor(()=>expect(errorToast).toHaveBeenCalledWith(t('settings-permissions-push-server-error')));
});

it('does not remove another account binding when registration belongs to a different owner',async()=>{
 readyPush('granted');request.mockResolvedValue(new Response('',{status:409}));fireEvent.click(screen.getByRole('button',{name:'Register background notifications'}));
 await waitFor(()=>expect(errorToast).toHaveBeenCalledWith(t('settings-permissions-push-other-registration')));
 expect(screen.getByRole('button',{name:'Register background notifications'})).toBeDisabled();expect(screen.getByRole('button',{name:t('settings-permissions-push-remove')})).toBeDisabled();
});

it('requests permission synchronously from the click before starting push or server work',async()=>{
 const {requestPermission}=readyPush();let grant!:(value:'granted')=>void;requestPermission.mockImplementation(()=>new Promise<'granted'>(resolve=>{grant=resolve;}));
 fireEvent.click(screen.getByRole('button',{name:'Register background notifications'}));
 expect(requestPermission).toHaveBeenCalledOnce();expect(pushSubscribe).not.toHaveBeenCalled();expect(request).not.toHaveBeenCalled();
 grant('granted');await waitFor(()=>expect(pushSubscribe).toHaveBeenCalledOnce());await screen.findByText(t('settings-permissions-push-registered'));
});

it('uses account-specific registration status and preserves the shared endpoint when removing only this account',async()=>{
 vi.stubGlobal('PushManager',class {});vi.stubGlobal('crypto',{subtle:{digest:async()=>new Uint8Array(32).buffer}});
 const unsubscribe=vi.fn();Object.defineProperty(navigator,'serviceWorker',{configurable:true,value:{getRegistration:async()=>({pushManager:{getSubscription:async()=>({endpoint:'https://push.example/browser',unsubscribe})}})}});
 activateAccountPreferences('child','msa1_child_push_authority');request.mockImplementation(async(_url:string,init?:RequestInit)=>new Response(JSON.stringify(init?.method==='POST'?{}:{registered:true,accountRegistered:false,allowed:true})));
 render(()=><BrowserPermissions enabled pushAllowed persistentStorageAllowed={false}/>);await screen.findByText(t('settings-permissions-push-unlinked'));expect(screen.getByRole('button',{name:'Register background notifications'})).toBeEnabled();
 suspendAccountPreferences();request.mockImplementation(async(_url:string,init?:RequestInit)=>new Response(JSON.stringify(init?.method==='POST'?{}:{registered:true,accountRegistered:true,allowed:true})));activateAccountPreferences('child','msa1_child_push_authority');
 await screen.findByText(t('settings-permissions-push-registered'));fireEvent.click(screen.getByRole('button',{name:t('settings-permissions-push-remove')}));await screen.findByText(t('settings-permissions-push-unlinked'));
 expect(unsubscribe).not.toHaveBeenCalled();expect(request).toHaveBeenCalledWith('/api/push/unsubscribe',expect.objectContaining({method:'POST',headers:expect.objectContaining({'x-mailwoman-storage-authority':'msa1_child_push_authority'})}));
});
