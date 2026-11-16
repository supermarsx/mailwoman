import {afterEach,expect,it,vi} from 'vitest';
import {activateAccountPreferences,suspendAccountPreferences} from './accountPreferenceLifecycle.ts';
import {setPrimaryMailboxAuthority} from './linkedMailboxSession.ts';
import {providerCallbackCandidate,pendingProviderFlow,rememberProviderFlow,forgetProviderFlow,providerHttpsUrl,providerRequest} from './providerOauth.ts';
import {apiFetch} from './http.ts';
vi.mock('./http.ts',()=>({apiFetch:vi.fn()}));
afterEach(()=>{forgetProviderFlow();suspendAccountPreferences();setPrimaryMailboxAuthority(undefined);vi.restoreAllMocks();});
it('recognizes provider responses on the query-free root redirect without mistaking mail links for callbacks',()=>{
 expect(providerCallbackCandidate('?code=opaque&state=correlation')).toBe(true);expect(providerCallbackCandidate('?error=access_denied&state=correlation')).toBe(true);expect(providerCallbackCandidate('?provider-oauth-callback=1')).toBe(true);
 for(const search of ['','?message=123&folder=inbox','?code=unrelated','?state=unrelated'])expect(providerCallbackCandidate(search)).toBe(false);
});
it('keeps redirect correlation bound to the primary authority and expiry',()=>{
 activateAccountPreferences('a','msa1_primary_authority_a');rememberProviderFlow({flowId:'flow',expiresIn:600});expect(pendingProviderFlow()).toBe('flow');
 activateAccountPreferences('a','msa1_primary_authority_b');expect(pendingProviderFlow()).toBeUndefined();activateAccountPreferences('a','msa1_primary_authority_a');expect(pendingProviderFlow()).toBe('flow');
 vi.spyOn(Date,'now').mockReturnValue(Date.now()+601000);expect(pendingProviderFlow()).toBeUndefined();
});
it('rejects non-HTTPS and credential-bearing provider navigation',()=>{
 expect(providerHttpsUrl('https://accounts.google.com/o/oauth2/v2/auth?state=opaque')).toContain('https://accounts.google.com/');for(const value of ['javascript:alert(1)','http://example.org','https://user:password@example.org','not a URL'])expect(providerHttpsUrl(value)).toBeUndefined();
});
it('cannot restore correlation or dispatch provider operations while signed out',async()=>{
 activateAccountPreferences('a','msa1_primary_authority_a');setPrimaryMailboxAuthority('msa1_primary_authority_a');rememberProviderFlow({flowId:'flow',expiresIn:600});suspendAccountPreferences();
 expect(pendingProviderFlow()).toBeUndefined();expect(()=>rememberProviderFlow({flowId:'other',expiresIn:600})).toThrow('signedOut');
 vi.mocked(apiFetch).mockClear();await expect(providerRequest('/complete','POST',{flowId:'flow',code:'secret',state:'state'})).rejects.toMatchObject({code:'signedOut'});expect(apiFetch).not.toHaveBeenCalled();
});
it('sends only the primary authority and refuses a response after account change',async()=>{
 activateAccountPreferences('a','msa1_primary_authority_a');setPrimaryMailboxAuthority('primary_authority');vi.mocked(apiFetch).mockImplementation(async(_url,init)=>{expect(new Headers(init?.headers).get('x-mailwoman-storage-authority')).toBe('primary_authority');expect(init?.redirect).toBe('error');activateAccountPreferences('b','msa1_primary_authority_b');return Response.json({connected:true});});await expect(providerRequest()).rejects.toMatchObject({name:'AbortError'});
});
it('refuses a delayed JSON body after the authority changes during parsing',async()=>{
 activateAccountPreferences('a','msa1_primary_authority_a');setPrimaryMailboxAuthority('msa1_primary_authority_a');
 let release!:(value:unknown)=>void,reading!:()=>void;const began=new Promise<void>(resolve=>{reading=resolve;});
 vi.mocked(apiFetch).mockResolvedValue({ok:true,status:200,json:()=>{reading();return new Promise(resolve=>{release=resolve;});}} as Response);
 const pending=providerRequest();await began;activateAccountPreferences('b','msa1_primary_authority_b');release({connected:true});
 await expect(pending).rejects.toMatchObject({name:'AbortError'});
});
