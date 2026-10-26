import { render, screen, fireEvent, waitFor } from '@solidjs/testing-library';
import { expect, it, vi } from 'vitest';
import { AdminSessions } from './Sessions.tsx';
import { adminRequest } from './shell/adminApi.ts';
vi.mock('./shell/adminApi.ts', () => ({ adminRequest: vi.fn() }));
it('revokes only previewed handles and clears preview when filters change', async () => {
  const listing = { sessions: [{ handle: 'opaque', accountId: 'a', username: 'user@example.org', lastSeen: 'today', clientType: 'browser', userAgent: 'Browser', ip: '192.0.2.1' }], truncated: false };
  vi.mocked(adminRequest).mockResolvedValue({ data: listing, status: 200, etag: null });
  render(() => <AdminSessions />);
  fireEvent.click(screen.getByRole('button', { name: 'Preview session revocation' }));
  fireEvent.click(await screen.findByRole('button', { name: 'Revoke these sessions' }));
  await waitFor(() => expect(adminRequest).toHaveBeenCalledWith('/sessions/revoke', { method: 'POST', body: { filter: {}, dryRun: false, handles: ['opaque'] } }));
  await waitFor(() => expect(screen.queryByRole('button', { name: 'Revoke these sessions' })).toBeNull());
  await waitFor(() => expect(screen.getByRole('button', { name: 'Preview session revocation' })).not.toBeDisabled());
  fireEvent.click(screen.getByRole('button', { name: 'Preview session revocation' }));
  await screen.findByRole('button', { name: 'Revoke these sessions' });
  fireEvent.input(screen.getByLabelText('Domain filter'), { target: { value: 'other.org' } });
  expect(screen.queryByRole('button', { name: 'Revoke these sessions' })).toBeNull();
});

it('filters displayed metadata and revokes only a confirmed row handle', async () => {
  const listing = { sessions: [{ handle: 'one', accountId: 'a', username: 'alice@example.org', lastSeen: 'today', clientType: 'browser', userAgent: 'Firefox', ip: '192.0.2.1' }, { handle: 'two', accountId: 'b', username: 'bob@other.org', lastSeen: 'today', clientType: 'native', userAgent: 'Desktop', ip: '192.0.2.2' }], truncated: false };
  vi.mocked(adminRequest).mockResolvedValue({ data: listing, status: 200, etag: null });
  render(() => <AdminSessions />);
  await screen.findByText('alice@example.org');
  expect(screen.getByRole('table', { name: 'Mailbox sessions' })).toBeInTheDocument();
  fireEvent.input(screen.getByLabelText('Domain filter'), { target: { value: 'example.org' } });
  expect(screen.queryByText('bob@other.org')).not.toBeInTheDocument();
  fireEvent.click(screen.getByRole('button', { name: 'Revoke session for alice@example.org' }));
  expect(screen.getByRole('alertdialog')).toBeInTheDocument();
  fireEvent.click(screen.getByRole('button', { name: /^Revoke session$/ }));
  await waitFor(() => expect(adminRequest).toHaveBeenCalledWith('/sessions/revoke', { method: 'POST', body: { filter: { handle: 'one' }, dryRun: false, handles: ['one'] } }));
});
it('reports partial selected revocations and stops after fresh-proof refusal',async()=>{
 const rows=['one','two','three'].map(handle=>({handle,accountId:'a',username:'alice@example.org',lastSeen:'today',clientType:'browser',userAgent:handle,ip:'192.0.2.1'}));
 const listing={data:{sessions:rows,truncated:false},status:200,etag:null};
 vi.mocked(adminRequest).mockReset().mockResolvedValue(listing).mockResolvedValueOnce(listing).mockResolvedValueOnce({data:{revoked:1},status:200,etag:null}).mockRejectedValueOnce({status:403});
 render(()=><AdminSessions/>);await screen.findByText('one');
 fireEvent.click(screen.getByRole('checkbox',{name:'Select all visible sessions'}));fireEvent.click(screen.getByRole('button',{name:/Revoke selected sessions/}));
 expect(screen.getByRole('alertdialog')).toBeInTheDocument();fireEvent.click(screen.getByRole('button',{name:'Confirm selected revocations'}));
 expect(await screen.findByRole('alert')).toHaveTextContent(/Revoked:.*1.*Could not confirm:.*1.*Not attempted:.*1/);
 const writes=vi.mocked(adminRequest).mock.calls.filter(([path])=>path==='/sessions/revoke');
 expect(writes.map(([,init])=>(init?.body as {handles:string[]}).handles)).toEqual([['one'],['two']]);
});
