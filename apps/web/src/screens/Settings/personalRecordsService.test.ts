import {afterEach,expect,it,vi} from 'vitest';
import {SettingsService} from './service.ts';
import {activateAccountPreferences,suspendAccountPreferences} from '../../api/accountPreferenceLifecycle.ts';
afterEach(()=>suspendAccountPreferences());
it('binds signature and search calls to verified realm even with the same wire account id',async()=>{
 activateAccountPreferences('1','msa1_first_verified_authority');
 const fetcher=vi.fn(async(_input:string,_init?:RequestInit)=>new Response('{"signatures":[],"savedSearches":[]}',{status:200}));
 const service=new SettingsService(fetcher);
 await service.listSignatures();await service.listSavedSearches();
 for(const call of fetcher.mock.calls)expect(new Headers(call[1]?.headers).get('x-mailwoman-storage-authority')).toBe('msa1_first_verified_authority');
 suspendAccountPreferences();activateAccountPreferences('1','msa1_second_verified_authority');
 await service.listSignatures();expect(new Headers(fetcher.mock.calls[2]?.[1]?.headers).get('x-mailwoman-storage-authority')).toBe('msa1_second_verified_authority');
});
it('does not dispatch personal record calls without a verified authority',async()=>{
 activateAccountPreferences('1');const fetcher=vi.fn();const service=new SettingsService(fetcher);
 await expect(service.listSignatures()).rejects.toThrow();await expect(service.listSavedSearches()).rejects.toThrow();expect(fetcher).not.toHaveBeenCalled();
});
it('rejects a late personal-record acknowledgement after a same-id realm replacement',async()=>{
 activateAccountPreferences('1','msa1_first_verified_authority');let finish:(response:Response)=>void=()=>{};
 const service=new SettingsService(()=>new Promise<Response>(resolve=>{finish=resolve;}));
 const pending=service.listSignatures();const rejected=expect(pending).rejects.toThrow();
 suspendAccountPreferences();activateAccountPreferences('1','msa1_second_verified_authority');finish(new Response('{"signatures":[]}',{status:200}));await rejected;
});
