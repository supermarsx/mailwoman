import {beforeEach,describe,expect,it,vi} from 'vitest';
import {activateAccountPreferences,preferenceAccount,suspendAccountPreferences} from './accountPreferenceLifecycle.ts';
import {resolveSender} from './senderResolution.ts';
const request=vi.hoisted(()=>vi.fn());
vi.mock('./http.ts',()=>({apiFetch:request}));
vi.mock('../i18n/index.ts',()=>({t:(key:string)=>key,isolate:(value:string)=>value}));
describe('one-message sender resolution',()=>{
 beforeEach(()=>{request.mockReset();suspendAccountPreferences();activateAccountPreferences('a','0123456789abcdef');});
 it('checks every send and carries the account authority without creating an identity',async()=>{
  request.mockResolvedValue(new Response(JSON.stringify({email:'alias@example.org',identityId:null})));
  expect(await resolveSender('alias@example.org',preferenceAccount())).toEqual({email:'alias@example.org',identityId:null});
  expect(request).toHaveBeenCalledWith('/api/account/sender/resolve',expect.objectContaining({method:'POST',body:'{"email":"alias@example.org"}',headers:expect.objectContaining({'x-mailwoman-storage-authority':'0123456789abcdef'})}));
 });
 it('uses an exact upstream identity for proxies',async()=>{
  request.mockResolvedValue(new Response(JSON.stringify({email:'alias@example.org',identityId:'upstream-1'})));
  expect((await resolveSender('alias@example.org',preferenceAccount())).identityId).toBe('upstream-1');
 });
 it('refuses address lists and header injection before requests',async()=>{
  for(const email of ['a@example.org,b@example.org','a@example.org\r\nBcc: b@example.org','Name <a@example.org>',''])await expect(resolveSender(email,preferenceAccount())).rejects.toThrow('mail-compose-from-invalid');
  expect(request).not.toHaveBeenCalled();
 });
 it('does not retry an authorization refusal',async()=>{
  request.mockResolvedValue(new Response('{}',{status:403}));
  await expect(resolveSender('alias@example.org',preferenceAccount())).rejects.toThrow('mail-compose-from-refused');
  expect(request).toHaveBeenCalledTimes(1);
 });
 it('rejects a response arriving after an account switch',async()=>{
  request.mockImplementation(async()=>{suspendAccountPreferences();activateAccountPreferences('b','fedcba9876543210');return new Response(JSON.stringify({email:'alias@example.org',identityId:null}));});
  await expect(resolveSender('alias@example.org',preferenceAccount())).rejects.toThrow('mail-compose-account-changed');
 });
 it('refuses a substituted address or malformed identity',async()=>{
  for(const result of [{email:'other@example.org',identityId:null},{email:'alias@example.org',identityId:''}]){request.mockResolvedValue(new Response(JSON.stringify(result)));await expect(resolveSender('alias@example.org',preferenceAccount())).rejects.toThrow('mail-compose-from-unavailable');}
 });
});
