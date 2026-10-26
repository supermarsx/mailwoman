import { afterEach,beforeEach,describe,it,expect,vi } from 'vitest';
import { adminSessionGeneration,observeAdminResponse,onAdminSessionEnded,markAdminSessionActive,forgetAdminSession,hadAdminSession,adminResponseIsCurrent } from './adminSession.ts';
import { apiFetch } from './http.ts';
describe('administrator expiry observer',()=>{
 beforeEach(()=>{history.replaceState(null,'','/admin#/settings');forgetAdminSession();});
 afterEach(()=>{forgetAdminSession();vi.restoreAllMocks();vi.unstubAllGlobals();});
 it('ends one current session once for an admin401 and preserves its return route',()=>{
  markAdminSessionActive();const epoch=adminSessionGeneration();const ended=vi.fn();const stop=onAdminSessionEnded(ended);
  observeAdminResponse('/admin/me',401,epoch);observeAdminResponse('/admin/settings',401,epoch);
  expect(ended).toHaveBeenCalledTimes(1);expect(hadAdminSession()).toBe(true);expect(location.hash).toBe('#/settings');expect(adminResponseIsCurrent(epoch)).toBe(true);stop();
 });
 it('never treats403, another origin, mailbox401, or a lookalike path as admin expiry',()=>{
  markAdminSessionActive();const epoch=adminSessionGeneration();const ended=vi.fn();const stop=onAdminSessionEnded(ended);
  for(const [path,status] of [['/admin/me',403],['https://untrusted.example/admin/me',401],['/api/me',401],['/administrator/me',401]] as const)observeAdminResponse(path,status,epoch);
  expect(ended).not.toHaveBeenCalled();stop();
 });
 it('excludes credential/factor failures and ordinary initial signed-out probes',()=>{
  markAdminSessionActive();const epoch=adminSessionGeneration();const ended=vi.fn();const stop=onAdminSessionEnded(ended);
  for(const path of ['/admin/login','/admin/login/totp','/admin/login/passkey/finish','/admin/logout','/admin/session'])observeAdminResponse(path,401,epoch);
  expect(ended).not.toHaveBeenCalled();stop();
 });
 it('ignores a delayed old-session401 after fresh authentication',()=>{
  markAdminSessionActive();const epoch=adminSessionGeneration();markAdminSessionActive();const ended=vi.fn();const stop=onAdminSessionEnded(ended);
  observeAdminResponse('/admin/me',401,epoch);expect(ended).not.toHaveBeenCalled();expect(adminResponseIsCurrent(epoch)).toBe(false);stop();
 });
 it('recognizes an expired admin session during CSRF-token recovery',()=>{
  markAdminSessionActive();const ended=vi.fn();const stop=onAdminSessionEnded(ended);
  observeAdminResponse('/admin/session',401,adminSessionGeneration(),true);expect(ended).toHaveBeenCalledOnce();stop();
 });
 it('does not turn an anonymous failed request into an expiry message',()=>{
  const ended=vi.fn();const stop=onAdminSessionEnded(ended);observeAdminResponse('/admin/me',401,adminSessionGeneration());expect(ended).not.toHaveBeenCalled();expect(hadAdminSession()).toBe(false);stop();
 });
 it('does not retry an unauthorized request or credentials',async()=>{
  markAdminSessionActive();const ended=vi.fn();const stop=onAdminSessionEnded(ended);
  const fetcher=vi.fn(async()=>new Response('{}',{status:401}));vi.stubGlobal('fetch',fetcher);
  expect((await apiFetch('/admin/me')).status).toBe(401);expect(fetcher).toHaveBeenCalledOnce();expect(ended).toHaveBeenCalledOnce();stop();
 });
 it('ends an expired session discovered during a rejected write token refresh',async()=>{
  markAdminSessionActive();const ended=vi.fn();const stop=onAdminSessionEnded(ended);
  const fetcher=vi.fn().mockResolvedValueOnce(new Response('{"error":"missing or invalid CSRF token"}',{status:403})).mockResolvedValueOnce(new Response('{}',{status:401}));vi.stubGlobal('fetch',fetcher);
  expect((await apiFetch('/admin/settings',{method:'PUT',body:'{}'})).status).toBe(403);
  expect(fetcher).toHaveBeenCalledTimes(2);expect(fetcher.mock.calls[1]?.[0]).toBe('/admin/session');expect(ended).toHaveBeenCalledOnce();stop();
 });
});
