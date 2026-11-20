import {beforeEach,afterEach,expect,it,vi} from 'vitest';
import {activateAccountPreferences,preferenceAccount,suspendAccountPreferences} from '../api/accountPreferenceLifecycle.ts';
import {configureOfflinePolicy,DEFAULT_OFFLINE_POLICY,reportOffline} from './runtime.ts';
import {offlineHttpRead} from './httpCache.ts';
import {offlineBlobRead} from './blobCache.ts';
// The shared setup can preload these caches through HTTP/session modules.
vi.hoisted(()=>vi.resetModules());
const state=vi.hoisted(()=>({rows:new Map<string,unknown>(),fail:false}));
vi.mock('./vault.ts',()=>({offlineVault:async()=>{if(state.fail)throw new Error('Storage disabled');return {get:async(namespace:string,key:string)=>state.rows.get(namespace+key),put:async(namespace:string,key:string,value:unknown)=>{state.rows.set(namespace+key,value);return true;}};}}));
beforeEach(()=>{vi.stubGlobal('Headers',new Response().headers.constructor);state.rows.clear();state.fail=false;reportOffline(false);activateAccountPreferences('a','verified_owner_123456');configureOfflinePolicy({...DEFAULT_OFFLINE_POLICY,enabled:true},preferenceAccount()!);});
afterEach(()=>{suspendAccountPreferences();reportOffline(false);vi.unstubAllGlobals();});
const unavailable=async():Promise<Response>=>{throw new TypeError('Offline');};
it('restores authenticated settings after network loss but respects an explicit refusal',async()=>{
 const owner=preferenceAccount();
 await offlineHttpRead('/api/settings',undefined,owner,async()=>new Response('{"settings":{}}',{headers:{'content-type':'application/json'}}));
 expect(await (await offlineHttpRead('/api/settings',undefined,owner,unavailable)).json()).toEqual({settings:{}});
 expect((await offlineHttpRead('/api/settings',undefined,owner,async()=>new Response('revoked',{status:401}))).status).toBe(401);
 await expect(offlineHttpRead('/api/admin/secret',undefined,owner,unavailable)).rejects.toThrow('Offline');
 await expect(offlineHttpRead('/api/settings',{method:'PUT'},owner,unavailable)).rejects.toThrow('Offline');
});
it('returns a successful network response when encrypted storage is unavailable',async()=>{
 state.fail=true;
 expect(await (await offlineHttpRead('/api/settings',undefined,preferenceAccount(),async()=>new Response('{"live":true}',{headers:{'content-type':'application/json'}}))).json()).toEqual({live:true});
 expect(await (await offlineBlobRead('/jmap/download/a/file/name','/jmap/download/a/file/name',undefined,preferenceAccount(),async()=>new Response('live file'))).text()).toBe('live file');
});
it('restores cached attachments and PDF byte ranges without caching refusals',async()=>{
 const owner=preferenceAccount(),url='/jmap/download/a/file/name';
 await offlineBlobRead(url,url,undefined,owner,async()=>new Response('0123456789',{headers:{'content-type':'application/pdf'}}));
 const restored=await offlineBlobRead(url,url,{headers:{range:'bytes=2-5'}},owner,unavailable);
 expect(restored.status).toBe(206);expect(restored.headers.get('content-range')).toBe('bytes 2-5/10');expect(await restored.text()).toBe('2345');
 expect((await offlineBlobRead(url,url,undefined,owner,async()=>new Response('forbidden',{status:403}))).status).toBe(403);
});
