import {describe,it,expect,vi} from 'vitest';
import {fireEvent,screen} from '@solidjs/testing-library';
import {BanList} from './BanList.tsx';
import {mockAdminApi,renderWithAdmin} from './testkit.tsx';
import type {BanEntry} from '../../state/slices/admin.ts';
const BAN: BanEntry={ip:'198.51.100.9',reason:'brute-force',bannedAt:'2026-07-14T00:00:00Z',expiresAt:null};
describe('Security Centre IP bans',()=>{
  it('explains active bans, trusted source addresses and health exemptions', async () => {
    renderWithAdmin(() => <BanList />);
    const note = await screen.findByTestId('admin-obs-bans-note');
    expect(note).toHaveTextContent('block application requests');
    expect(note).toHaveTextContent('Expired entries no longer block');
    expect(note).toHaveTextContent('Health checks remain available');
    expect(note).toHaveTextContent('configured proxies');
  });

  it('lists bans and can unban', async () => {
    const removeBan = vi.fn(async () => undefined);
    renderWithAdmin(() => <BanList />, mockAdminApi({ listBans: vi.fn(async () => [BAN]), removeBan }));
    expect(await screen.findByText('198.51.100.9')).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: 'Unban 198.51.100.9' }));
    await Promise.resolve();
    expect(removeBan).toHaveBeenCalledWith('198.51.100.9');
  });
 it('loads bans independently of audit permission',async()=>{
   const api=mockAdminApi({listAudit:vi.fn(async()=>{throw new Error('forbidden');})});
   renderWithAdmin(()=><BanList/>,api);await screen.findByText('No active bans.');
   expect(api.listAudit).not.toHaveBeenCalled();
 });
});
