import { describe, expect, it, vi } from 'vitest';
import { screen } from '@solidjs/testing-library';
import { Users } from './Users.tsx';
import { renderInShell } from './shell/testkit.tsx';
vi.mock('./shell/adminApi.ts', () => ({adminRequest:vi.fn(async () => ({data:{accountId:'opaque',address:'alice@example.test',label:'Alice (alice@example.test)'}}))}));
describe('user identity deep link', () => {
  it('loads the actual identity without showing provisioning or mutation forms', async () => {
    renderInShell(() => <Users />, {route:() => ({section:'users',rest:['opaque']})});
    expect(await screen.findByRole('heading',{name:'Alice (alice@example.test)'})).toBeInTheDocument();
    expect(screen.getByText('opaque')).toBeInTheDocument();
    expect(screen.queryByRole('form')).not.toBeInTheDocument();
    expect(screen.getByRole('link',{name:'Back to users'})).toBeInTheDocument();
  });
});
