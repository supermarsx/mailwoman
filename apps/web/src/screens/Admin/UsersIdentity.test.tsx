import { describe, expect, it, vi } from 'vitest';
import { screen } from '@solidjs/testing-library';
import { Users } from './Users.tsx';
import {renderWithAdmin,mockAdminApi} from './testkit.tsx';
import {adminRequest} from './shell/adminApi.ts';
vi.mock('./shell/adminApi.ts', () => ({adminRequest:vi.fn(async () => ({data:{accountId:'opaque',address:'alice@example.test',label:'Alice (alice@example.test)'}}))}));
describe('user identity deep link', () => {
  it('shows a proven alias login separately from its retained administrative record',async()=>{
    vi.mocked(adminRequest).mockResolvedValueOnce({status:200,etag:null,data:{accountId:'alias-account',address:'alice@parent.test',directoryAddress:'alice@parent.test',loginAddress:'alice@alias.test',legacyBinding:true,label:'alice@parent.test'}});
    renderWithAdmin(()=><Users accountId="alias-account"/>,mockAdminApi());
    expect(await screen.findByRole('heading',{name:'alice@alias.test'})).toBeInTheDocument();
    expect(screen.getByText(/Existing administrative record:.*alice@parent.test/)).toBeInTheDocument();
    expect(screen.getByText('alias-account')).toBeInTheDocument();
  });
  it('loads the actual identity without showing provisioning or mutation forms', async () => {
    renderWithAdmin(() => <Users accountId="opaque"/>,mockAdminApi());
    expect(await screen.findByRole('heading',{name:'Alice (alice@example.test)'})).toBeInTheDocument();
    expect(screen.getByText('opaque')).toBeInTheDocument();
    expect(screen.queryByRole('form')).not.toBeInTheDocument();
    expect(screen.getByRole('link',{name:'Back to users'})).toBeInTheDocument();
  });
});
