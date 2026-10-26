import { afterEach, expect, it, vi } from 'vitest';
import { SettingsService } from './service.ts';
import { activateAccountPreferences, suspendAccountPreferences } from '../../api/accountPreferenceLifecycle.ts';

afterEach(()=>suspendAccountPreferences());
it('binds revoke-others to the reviewed account and aborts late completion after replacement',async()=>{
 activateAccountPreferences('reviewed-account');let release:(response:Response)=>void=()=>{};
 const fetcher=vi.fn((_input:string,_init?:RequestInit)=>new Promise<Response>(resolve=>{release=resolve;}));
 const service=new SettingsService(fetcher);const pending=service.revokeOtherSessions();
 const rejected=expect(pending).rejects.toMatchObject({name:'AbortError'});
 expect(new Headers(fetcher.mock.calls[0]?.[1]?.headers).get('X-Mailwoman-Account')).toBe('reviewed-account');
 const signal=fetcher.mock.calls[0]?.[1]?.signal;expect(signal?.aborted).toBe(false);
 suspendAccountPreferences();activateAccountPreferences('replacement-account');expect(signal?.aborted).toBe(true);
 release(new Response('{"ok":true}',{status:200}));await rejected;
});
