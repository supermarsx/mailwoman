import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { fireEvent, render, screen, waitFor } from '@solidjs/testing-library';
import { createSignal } from 'solid-js';
import { PersonalPreferencesProvider, usePersonalPreferences, type PersonalPreferences, type AdminPreferencesApi } from './personalPreferences.tsx';
import { AdminScreen } from '../index.tsx';
import { PersonalPreferences as Page } from '../PersonalPreferences.tsx';
import { mockAdminApi } from '../testkit.tsx';
import { holding, offlineShell } from './testkit.tsx';
import { expectNoAxeViolations, expectNoInlineStyle } from './uxTestkit.ts';
import type { SectionDef } from './sections.ts';
import {toasts,resetToasts} from '../../../components/toast/toast.ts';

type Response = { data: Record<string, unknown>; etag: string | null };
function deferred() { let resolve!: (value: Response) => void; const promise = new Promise<Response>(yes => { resolve = yes; }); return { promise, resolve }; }
let prefs!: PersonalPreferences;
function Probe() { prefs = usePersonalPreferences(); return <span>{prefs.loading() ? 'loading' : 'ready'}</span>; }
beforeEach(() => { resetToasts();localStorage.clear(); history.replaceState(null, '', '/admin'); });
afterEach(() => vi.unstubAllGlobals());

it('queues edits while saving, advances ETags, and preserves unknown preferences', async () => {
  const pending = deferred();
  const save = vi.fn<AdminPreferencesApi['save']>().mockImplementationOnce(() => pending.promise).mockImplementation(async data => ({ data, etag: 'v3' }));
  render(() => <PersonalPreferencesProvider username="alice" api={{ load: async () => ({ data: { future: { retained: true } }, etag: 'v1' }), save }}><Probe /></PersonalPreferencesProvider>);
  await screen.findByText('ready');
  expect(toasts()).toHaveLength(0);
  prefs.appearance.setTheme('paper'); prefs.appearance.setDensity('compact');
  const notice=toasts()[0]!;expect(notice).toMatchObject({busy:true,persistent:true});
  expect(save).toHaveBeenCalledTimes(1);
  pending.resolve({ data: {}, etag: 'v2' });
  await waitFor(() => expect(save).toHaveBeenCalledTimes(2));
  expect(save.mock.calls[1]).toEqual([{ future: { retained: true }, appearance: expect.objectContaining({ theme: 'paper', density: 'compact' }) }, 'v2']);
  await waitFor(() => expect(prefs.saving()).toBe(false));
  expect(toasts()).toHaveLength(1);expect(toasts()[0]).toMatchObject({id:notice.id,tone:'success',busy:false,persistent:false});
  expect(localStorage.getItem('mw.admin.appearance')).toBeNull();
});

it('ignores stale load and save responses after account changes', async () => {
  const first = deferred(); const pending = deferred();
  const load = vi.fn<AdminPreferencesApi['load']>().mockImplementationOnce(() => first.promise).mockResolvedValue({ data: { appearance: { theme: 'paper', mode: 'light', accent: '' } }, etag: 'bob' });
  const save = vi.fn<AdminPreferencesApi['save']>().mockImplementationOnce(() => pending.promise).mockResolvedValue({ data: {}, etag: 'new' });
  const [username, setUsername] = createSignal('alice');
  render(() => <PersonalPreferencesProvider username={username()} api={{ load, save }}><Probe /></PersonalPreferencesProvider>);
  setUsername('bob'); await screen.findByText('ready');
  first.resolve({ data: { appearance: { theme: 'dark', mode: 'dark', accent: '' } }, etag: 'alice' });
  await Promise.resolve(); expect(prefs.appearance.prefs().theme).toBe('paper');
  prefs.patch({ locale: 'fr' }); setUsername('carol');
  await waitFor(() => expect(load).toHaveBeenCalledTimes(3));
  pending.resolve({ data: {}, etag: 'stale-bob' }); await Promise.resolve();
  prefs.patch({ locale: 'de' });
  await waitFor(() => expect(save).toHaveBeenCalledTimes(2));
  expect(save.mock.calls[1]?.[1]).toBe('bob');
});

it('retains unsaved edits on conflict until explicit reload', async () => {
  const load = vi.fn<AdminPreferencesApi['load']>().mockResolvedValue({ data: {}, etag: 'v1' });
  render(() => <PersonalPreferencesProvider username="alice" api={{ load, save: async () => { throw new Error('412: changed in another browser'); } }}><Probe /></PersonalPreferencesProvider>);
  await screen.findByText('ready'); prefs.appearance.setTheme('paper');
  await waitFor(() => expect(prefs.error()).toContain('412'));
  expect(prefs.appearance.prefs().theme).toBe('paper'); expect(load).toHaveBeenCalledTimes(1);
  await prefs.reload(); expect(prefs.appearance.prefs().theme).toBe('dark');
});

it('offers personal preferences without system capabilities and persists layout edits', async () => {
  history.replaceState(null, '', '/admin#/preferences');
  const save = vi.fn<AdminPreferencesApi['save']>().mockImplementation(async data => ({ data, etag: 'v2' }));
  const sections: SectionDef[] = [{ id: 'preferences', group: 'personal', icon: 'settings', label: () => 'My preferences', component: Page }];
  render(() => <AdminScreen api={mockAdminApi()} {...offlineShell()} loadMe={holding([])} preferencesApi={{ load: async () => ({ data: {}, etag: 'v1' }), save }} sections={sections} />);
  await screen.findByRole('heading', { name: 'My preferences' });
  const density = await screen.findByLabelText('Interface density');
  await waitFor(() => expect(density).not.toBeDisabled());
  fireEvent.change(density, { target: { value: 'compact' } });
  await waitFor(() => expect(save).toHaveBeenCalledWith(expect.objectContaining({ appearance: expect.objectContaining({ density: 'compact' }) }), 'v1'));
  expect(screen.queryByRole('button', { name: 'Appearance' })).toBeNull();
  expect(screen.getByText(/credentials are managed by the server/)).toBeInTheDocument();
  await expectNoAxeViolations(screen.getByTestId('admin-personal-preferences'));
  expectNoInlineStyle(screen.getByTestId('admin-personal-preferences'));
});

it('disables edits after a failed save and explains that reload discards changes', async () => {
  history.replaceState(null, '', '/admin#/preferences');
  render(() => <AdminScreen api={mockAdminApi()} {...offlineShell()} preferencesApi={{ load: async () => ({ data: {}, etag: 'v1' }), save: async () => { throw new Error('412: changed elsewhere'); } }} sections={[{ id: 'preferences', group: 'personal', icon: 'preferences', label: () => 'My preferences', component: Page }]} />);
  const density = await screen.findByLabelText('Interface density');
  await waitFor(() => expect(density).not.toBeDisabled());
  fireEvent.change(density, { target: { value: 'compact' } });
  await screen.findByText('412: changed elsewhere');
  expect(density).toBeDisabled();
  expect(screen.getByText(/Reloading discards these unsaved changes/)).toBeInTheDocument();
  fireEvent.click(screen.getByRole('button', { name: 'Discard changes and reload' }));
  await waitFor(() => expect(density).not.toBeDisabled());
  expect(density).toHaveValue('cozy');
});

it('navigates preference subtabs by keyboard and retains values across panels', async () => {
  history.replaceState(null, '', '/admin#/preferences/layout');
  render(() => <AdminScreen api={mockAdminApi()} {...offlineShell()} preferencesApi={{ load: async () => ({ data: {}, etag: 'v1' }), save: async data => ({data,etag:'v2'}) }} sections={[{ id: 'preferences', group: 'personal', icon: 'preferences', label: () => 'My preferences', component: Page }]} />);
  const layout=await screen.findByRole('tab',{name:'Reading and layout'});
  expect(layout).toHaveAttribute('aria-selected','true');
  expect(screen.getAllByRole('tabpanel')).toHaveLength(1);
  const density=screen.getByRole('combobox',{name:'Interface density'});
  await waitFor(()=>expect(density).not.toBeDisabled());
  fireEvent.change(density,{target:{value:'compact'}});
  layout.focus();fireEvent.keyDown(layout,{key:'ArrowRight'});
  await waitFor(()=>expect(screen.getByRole('tab',{name:'Dashboard'})).toHaveAttribute('aria-selected','true'));
  expect(location.hash).toBe('#/preferences/dashboard');
  await waitFor(()=>expect(screen.getByRole('tab',{name:'Dashboard'})).toHaveFocus());
  fireEvent.keyDown(screen.getByRole('tab',{name:'Dashboard'}),{key:'Home'});
  await waitFor(()=>expect(screen.getByRole('tab',{name:'Appearance'})).toHaveAttribute('aria-selected','true'));
  fireEvent.click(layout);
  await waitFor(()=>expect(screen.getByRole('combobox',{name:'Interface density'})).toHaveValue('compact'));
  expect(screen.getByRole('combobox',{name:'Interface density'})).toBe(density);
  await expectNoAxeViolations(screen.getByTestId('admin-personal-preferences'));
});

it('uses personal security endpoints without writing secrets to preferences', async () => {
  history.replaceState(null, '', '/admin#/preferences');
  const calls: { path: string; body: unknown }[] = [];
  let totpEnrolled = false;
  vi.stubGlobal('fetch', vi.fn(async (input: string | URL | Request, init?: RequestInit) => {
    const path = String(input);
    if(path.endsWith('/admin/me')) return new Response(JSON.stringify({totpEnrolled}),{status:200,headers:{'Content-Type':'application/json'}});
    if (!path.includes('/admin/me/')) return new Response('{}', { status: 200, headers: { 'Content-Type': 'application/json' } });
    calls.push({ path, body: init?.body ? JSON.parse(String(init.body)) : null });
    if(path.endsWith('/passkeys')) return new Response(JSON.stringify({passkeys:[],recoveryRemaining:0}),{status:200,headers:{'Content-Type':'application/json'}});
    if(path.endsWith('/totp/confirm')) {totpEnrolled=true;return new Response(JSON.stringify({ok:true,recoveryCodes:[]}),{status:200,headers:{'Content-Type':'application/json'}});}
    return path.endsWith('/totp') ? new Response(JSON.stringify({ secret: 'TESTSETUPKEY',otpauthUri:'otpauth://totp/test?secret=TESTSETUPKEY' }), { status: 200, headers: { 'Content-Type': 'application/json' } }) : new Response(null, { status: 204 });
  }));
  const save = vi.fn<AdminPreferencesApi['save']>().mockImplementation(async data => ({ data, etag: 'v2' }));
  render(() => <AdminScreen api={mockAdminApi()} {...offlineShell()} loadMe={async () => ({ kind: 'listed', me: { username: 'alice', accountId: 'account-alice', displayName: null, roles: [], capabilities: [], domainScope: null, totpEnrolled: false } })} preferencesApi={{ load: async () => ({ data: {}, etag: 'v1' }), save }} sections={[{ id: 'preferences', group: 'personal', icon: 'preferences', label: () => 'My preferences', component: Page }]} />);
  fireEvent.click(await screen.findByRole('tab', { name: 'Personal security' }));
  fireEvent.click(await screen.findByRole('button', { name: 'Manage authenticator and recovery codes' }));
  fireEvent.click(await screen.findByRole('button', { name: 'Set up authenticator' }));
  expect(await screen.findByLabelText('Manual setup key')).toHaveValue('TESTSETUPKEY');
  fireEvent.input(screen.getByLabelText('Authenticator code'), { target: { value: '123456' } });
  fireEvent.click(screen.getByRole('button', { name: 'Confirm authenticator' }));
  await screen.findByText('An authenticator is enabled for your admin account.');
  expect(calls).toContainEqual({ path: '/admin/me/totp/confirm', body: { code: '123456' } });
  expect(screen.queryByLabelText('Manual setup key')).toBeNull();
  await waitFor(()=>expect(screen.getByRole('button',{name:'Back to my preferences'})).not.toBeDisabled());
  fireEvent.click(screen.getByRole('button',{name:'Back to my preferences'}));
  await screen.findByLabelText('Current password');
  fireEvent.input(screen.getByLabelText('Current password'), { target: { value: 'old-fixture' } });
  fireEvent.input(screen.getByLabelText('New password'), { target: { value: 'new-fixture' } });
  fireEvent.submit(screen.getByRole('button', { name: 'Change password' }).closest('form')!);
  await waitFor(() => expect(calls).toContainEqual({ path: '/admin/me/password', body: { currentPassword: 'old-fixture', newPassword: 'new-fixture' } }));
  await waitFor(() => expect(screen.getByLabelText('New password')).toHaveValue(''));
  expect(save).not.toHaveBeenCalled();
});
