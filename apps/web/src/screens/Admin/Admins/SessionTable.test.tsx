import { render, screen, fireEvent,waitFor } from '@solidjs/testing-library';
import { expect, it, vi } from 'vitest';
import { formatDateTime } from '../../../i18n';
import { AdminSessionTable } from './SessionTable.tsx';
it('shows real deadlines, marks the current session and respects revoke permissions', () => {
  const revoke = vi.fn();
  render(() => <AdminSessionTable sessions={[{ id: 'opaque', adminId: 'root', createdAt: '2026-10-25', idleExpiresAt: 1800000000, expiresAt: 1800003600, current: true }]} accounts={[]} busy={false} canRevoke={false} onRevoke={revoke} />);
  expect(screen.getByRole('table', { name: 'Administrator sessions' })).toBeInTheDocument();
  expect(screen.getByText('This session')).toBeInTheDocument();
  expect(screen.getByText(formatDateTime(new Date(1800000000 * 1000)))).toBeInTheDocument();
  const button = screen.getByRole('button', { name: 'Revoke session for root' });
  expect(button).toBeDisabled(); fireEvent.click(button); expect(revoke).not.toHaveBeenCalled();
});
it('selects only visible administrator sessions and passes a reviewed snapshot',async()=>{
 const batch=vi.fn();const sessions=[{id:'mine',adminId:'root',createdAt:'2026-10-25',current:true},{id:'other',adminId:'alice',createdAt:'2026-10-25',current:false}];
 render(()=><AdminSessionTable sessions={sessions} accounts={[]} busy={false} canRevoke={true} onRevoke={()=>{}} onRevokeMany={batch}/>);
 fireEvent.input(screen.getByRole('searchbox'),{target:{value:'alice'}});
 await waitFor(()=>expect(screen.queryByRole('checkbox',{name:'Select session for root'})).toBeNull());
 fireEvent.click(screen.getByRole('checkbox',{name:'Select all visible sessions'}));
 fireEvent.click(screen.getByRole('button',{name:/Revoke selected sessions/}));
 expect(batch).toHaveBeenCalledWith([sessions[1]]);
});
