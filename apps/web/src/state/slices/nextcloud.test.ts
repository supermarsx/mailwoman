import {beforeEach,describe,expect,it,vi} from 'vitest';
import {createRoot} from 'solid-js';
import {activateAccountPreferences,suspendAccountPreferences} from '../../api/accountPreferenceLifecycle.ts';
import {createNextcloudSlice} from './nextcloud.ts';
beforeEach(()=>{suspendAccountPreferences();activateAccountPreferences('alice');});
describe('Nextcloud link availability',()=>{
  it('checks local metadata only and clears availability on account changes',async()=>{
    const fetcher=vi.fn(async()=>new Response(JSON.stringify({accountId:'alice',links:[{ready:true}]})));
    let dispose!:()=>void;const slice=createRoot(stop=>{dispose=stop;return createNextcloudSlice(fetcher);});
    await slice.ensureEnabled();expect(slice.enabled()).toBe(true);
    expect(fetcher).toHaveBeenCalledWith('/api/account/nextcloud');
    suspendAccountPreferences();activateAccountPreferences('bob');expect(slice.enabled()).toBe(false);
    await slice.ensureEnabled();expect(slice.enabled()).toBe(false);dispose();
  });
  it('refreshes after link changes and ignores a superseded availability response',async()=>{
    const responses:Array<(response:Response)=>void>=[];
    const fetcher=vi.fn(()=>new Promise<Response>(resolve=>responses.push(resolve)));
    let dispose!:()=>void;const slice=createRoot(stop=>{dispose=stop;return createNextcloudSlice(fetcher);});
    const previous=slice.ensureEnabled();
    globalThis.dispatchEvent(new Event('mw-nextcloud-links-changed'));
    expect(fetcher).toHaveBeenCalledTimes(2);
    responses[1]!(new Response(JSON.stringify({accountId:'alice',links:[]})));
    await slice.ensureEnabled();expect(slice.enabled()).toBe(false);
    responses[0]!(new Response(JSON.stringify({accountId:'alice',links:[{ready:true}]})));
    await previous;expect(slice.enabled()).toBe(false);
    dispose();globalThis.dispatchEvent(new Event('mw-nextcloud-links-changed'));
    expect(fetcher).toHaveBeenCalledTimes(2);
  });
  it('ignores metadata returned after an account switch',async()=>{
    let complete!:(response:Response)=>void;const fetcher=vi.fn(()=>new Promise<Response>(resolve=>{complete=resolve;}));
    let dispose!:()=>void;const slice=createRoot(stop=>{dispose=stop;return createNextcloudSlice(fetcher);});
    const pending=slice.ensureEnabled();suspendAccountPreferences();activateAccountPreferences('bob');
    complete(new Response(JSON.stringify({accountId:'alice',links:[{ready:true}]})));await pending;
    expect(slice.enabled()).toBe(false);dispose();
  });
});
