import { beforeEach,afterEach,describe,it,expect,vi } from 'vitest';
import { render,screen,waitFor } from '@solidjs/testing-library';
import { AdminScreen } from './index.tsx';
import { mockAdminApi } from './testkit.tsx';
import { offlineShell } from './shell/testkit.tsx';
import { adminRequest } from './shell/adminApi.ts';
import { forgetAdminSession,markAdminSessionActive } from '../../api/adminSession.ts';
import { createAdminSlice,type AdminSession } from '../../state/slices/admin.ts';
describe('administrator expired-session gate',()=>{
 beforeEach(()=>{history.replaceState(null,'','/admin#/settings');forgetAdminSession();});
 afterEach(()=>{vi.restoreAllMocks();forgetAdminSession();});
 it('never promotes a delayed successful probe after another request ended the session',async()=>{
  let resolve!:(value:AdminSession)=>void;
  const api=mockAdminApi({session:()=>new Promise<AdminSession>(done=>{resolve=done;})});
  const admin=createAdminSlice(api);await admin.login('root','password');
  const pending=admin.loadSession();api.onSessionEnded?.();
  expect(admin.session()).toBeNull();resolve({username:'root'});await pending;
  expect(admin.session()).toBeNull();expect(admin.sessionEnded()).toBe(true);
 });
 it('recognizes a prior tab session at initial probe without showing generic failure',async()=>{
  markAdminSessionActive();render(()=><AdminScreen api={mockAdminApi({session:async()=>null})} {...offlineShell()}/>);
  expect(await screen.findByRole('form',{name:'Admin sign in'})).toBeInTheDocument();
  expect(screen.getByTestId('admin-session-ended')).toHaveTextContent('Sign in again');expect(location.hash).toBe('#/settings');
 });
 it('handles a session expiring between the initial probe and capability loading',async()=>{
  vi.stubGlobal('fetch',vi.fn(async()=>new Response('{"code":"unauthorized"}',{status:401})));
  render(()=><AdminScreen api={mockAdminApi()} {...offlineShell()} loadMe={async()=>(await adminRequest<never>('/me')).data}/>);
  expect(await screen.findByRole('form',{name:'Admin sign in'})).toBeInTheDocument();
  expect(screen.getByTestId('admin-session-ended')).toBeInTheDocument();
 });
 it('moves to sign in when a modern adminRequest returns401 after loading',async()=>{
  history.replaceState(null,'','/admin');
  render(()=><AdminScreen api={mockAdminApi()} {...offlineShell()}/>);
  await screen.findByRole('navigation',{name:'Admin sections'});
  vi.stubGlobal('fetch',vi.fn(async()=>new Response('{"code":"unauthorized"}',{status:401})));
  await expect(adminRequest('/settings')).rejects.toMatchObject({status:401});
  await waitFor(()=>expect(screen.getByRole('form',{name:'Admin sign in'})).toBeInTheDocument());
  expect(screen.getByTestId('admin-session-ended')).toBeInTheDocument();expect(screen.queryByRole('navigation',{name:'Admin sections'})).toBeNull();
 });
 it('keeps an authenticated console mounted when a request is forbidden',async()=>{
  history.replaceState(null,'','/admin');render(()=><AdminScreen api={mockAdminApi()} {...offlineShell()}/>);
  await screen.findByRole('navigation',{name:'Admin sections'});
  vi.stubGlobal('fetch',vi.fn(async()=>new Response('{"code":"forbidden"}',{status:403})));
  await expect(adminRequest('/settings')).rejects.toMatchObject({status:403});
  expect(screen.getByRole('navigation',{name:'Admin sections'})).toBeInTheDocument();expect(screen.queryByRole('form',{name:'Admin sign in'})).toBeNull();
 });
});
