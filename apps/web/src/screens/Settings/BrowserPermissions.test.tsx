import { afterEach, describe, expect, it, vi } from 'vitest';
import { fireEvent, render, screen, waitFor } from '@solidjs/testing-library';
import { BrowserPermissions } from './BrowserPermissions.tsx';
import { activateAccountPreferences, suspendAccountPreferences } from '../../api/accountPreferenceLifecycle.ts';
import { t } from '../../test/i18n.ts';
void t;
const request=vi.hoisted(()=>vi.fn());
vi.mock('../../api/http.ts',()=>({apiFetch:request}));

afterEach(() => { request.mockReset(); delete (navigator as unknown as {serviceWorker?:unknown}).serviceWorker; vi.unstubAllGlobals(); suspendAccountPreferences(); delete (navigator as unknown as { storage?: unknown }).storage; });
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
 await waitFor(()=>expect(screen.getByText(t('settings-permissions-push-unregistered'))).toBeInTheDocument());
 expect(new Headers(request.mock.calls[1]?.[1]?.headers).get('x-mailwoman-storage-authority')).toBe('msa1_second_verified_authority');
});
