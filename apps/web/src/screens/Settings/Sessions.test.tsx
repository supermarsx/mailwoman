import { render, screen, fireEvent, waitFor } from '@solidjs/testing-library';
import { expect, it, vi } from 'vitest';
import { Sessions } from './Sessions.tsx';
import { SettingsService } from './service.ts';
import { activateAccountPreferences, suspendAccountPreferences } from '../../api/accountPreferenceLifecycle.ts';
it('keeps the current session safe and targets the selected device handle', async () => {
  const service = new SettingsService();
  vi.spyOn(service, 'sessions').mockResolvedValue([
    { handle: 'current', username: 'alice@example.org', createdAt: 'today', lastSeen: 'now', current: true, deviceLabel: 'This laptop', userAgent:'Full browser technical details' },
    { handle: 'other', username: 'alice@example.org', createdAt: 'yesterday', lastSeen: 'background poll', lastActivity: 'user interaction', current: false, deviceLabel: 'Phone', ip: '192.0.2.1', country: 'GB' },
  ]);
  const revoke = vi.spyOn(service, 'revokeSession').mockResolvedValue(undefined);
  render(() => <Sessions service={service} />);
  await screen.findByText('This laptop');
  expect(screen.getByText('Full browser technical details')).not.toBeVisible();fireEvent.click(screen.getByText('Client details'));expect(screen.getByText('Full browser technical details')).toBeVisible();
  expect(screen.getByRole('table', { name: 'Active sessions' })).toBeInTheDocument();
  expect(screen.getByText('Last active user interaction')).toBeInTheDocument();
  expect(screen.queryByText('Last active background poll')).not.toBeInTheDocument();
  expect(screen.queryByRole('button', { name: 'Sign out This laptop' })).not.toBeInTheDocument();
  fireEvent.input(screen.getByRole('searchbox'), { target: { value: '192.0.2.1' } });
  expect(screen.queryByText('This laptop')).not.toBeInTheDocument();
  fireEvent.click(screen.getByRole('button', { name: 'Sign out Phone' }));
  await waitFor(() => expect(revoke).toHaveBeenCalledWith('other'));
});
it('renders a bounded error without throwing while session loading fails', async () => {
  const service = new SettingsService();
  vi.spyOn(service, 'sessions').mockRejectedValue(new Error('private backend failure'));
  render(() => <Sessions service={service} />);
  expect(await screen.findByRole('alert')).toHaveTextContent('The session could not be changed.');
  expect(screen.queryByText('private backend failure')).not.toBeInTheDocument();
});
it('confirms selected other sessions, protects the current one and reports partial failure',async()=>{
 const service=new SettingsService();
 vi.spyOn(service,'sessions').mockResolvedValue(['current','one','two','three'].map(handle=>({handle,username:'alice@example.org',createdAt:'today',lastSeen:'now',current:handle==='current',deviceLabel:handle})));
 const revoke=vi.spyOn(service,'revokeSession').mockImplementation(async handle=>{if(handle==='two')throw{status:503};});
 render(()=><Sessions service={service}/>);await screen.findByText('three');
 expect(screen.getByRole('checkbox',{name:'Select session for current'})).toBeDisabled();
 fireEvent.click(screen.getByRole('checkbox',{name:'Select all visible other sessions'}));
 fireEvent.click(screen.getByRole('button',{name:/Sign out selected sessions \(/}));
 expect(screen.getByRole('alertdialog')).toHaveTextContent('This session stays signed in');
 expect(revoke).not.toHaveBeenCalled();fireEvent.click(screen.getByRole('button',{name:'Cancel'}));expect(revoke).not.toHaveBeenCalled();
 fireEvent.click(screen.getByRole('button',{name:/Sign out selected sessions \(/}));fireEvent.click(screen.getByRole('button',{name:/^Sign out selected sessions$/}));
 expect(await screen.findByRole('alert')).toHaveTextContent(/Signed out:.*1.*Could not confirm:.*1.*Not attempted:.*1/);
 expect(revoke.mock.calls.map(call=>call[0])).toEqual(['one','two']);
});
it('does not continue or publish an old batch after the account changes',async()=>{
 activateAccountPreferences('bulk-first');
 const service=new SettingsService();let finish:()=>void=()=>{};
 vi.spyOn(service,'sessions').mockResolvedValue(['one','two'].map(handle=>({handle,username:'first',createdAt:'today',lastSeen:'now',current:false,deviceLabel:handle})));
 const revoke=vi.spyOn(service,'revokeSession').mockImplementation(()=>new Promise<void>(resolve=>{finish=resolve;}));
 render(()=><Sessions service={service}/>);await screen.findByText('two');
 fireEvent.click(screen.getByRole('checkbox',{name:'Select all visible other sessions'}));
 fireEvent.click(screen.getByRole('button',{name:/Sign out selected sessions \(/}));
 fireEvent.click(screen.getByRole('button',{name:/^Sign out selected sessions$/}));
 await waitFor(()=>expect(revoke).toHaveBeenCalledOnce());
 suspendAccountPreferences();activateAccountPreferences('bulk-second');finish();
 await waitFor(()=>expect(screen.queryByRole('alertdialog')).not.toBeInTheDocument());
 expect(revoke).toHaveBeenCalledOnce();expect(screen.queryByText(/Signed out:/)).not.toBeInTheDocument();
 suspendAccountPreferences();
});

it('announces the initial session load and exposes a named keyboard scroll region',async()=>{
 const service=new SettingsService();let release!:(value:[])=>void;vi.spyOn(service,'sessions').mockImplementation(()=>new Promise(resolve=>{release=resolve;}));
 render(()=><Sessions service={service}/>);expect(screen.getByTestId('async-pending')).toBeVisible();expect(screen.getByRole('region',{name:'Active sessions table'})).toHaveAttribute('tabindex','0');
 release([]);await waitFor(()=>expect(screen.queryByTestId('async-pending')).not.toBeInTheDocument());
});
