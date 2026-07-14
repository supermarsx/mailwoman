import { describe, it, expect, vi } from 'vitest';
import { fireEvent, screen } from '@solidjs/testing-library';
import { Domains } from './Domains.tsx';
import { mockAdminApi, renderWithAdmin } from './testkit.tsx';
import type { Domain } from '../../state/slices/admin.ts';

const D: Domain = { name: 'example.com', upstreamJson: '{}', allowlist: ['a@x'], blocklist: [] };

describe('Admin › Domains', () => {
  it('lists domains from the api', async () => {
    renderWithAdmin(() => <Domains />, mockAdminApi({ listDomains: vi.fn(async () => [D]) }));
    expect(await screen.findByText('example.com')).toBeInTheDocument();
  });

  it('saves a new domain and reloads', async () => {
    const saveDomain = vi.fn(async () => undefined);
    const { api } = renderWithAdmin(() => <Domains />, mockAdminApi({ saveDomain }));
    fireEvent.input(screen.getByPlaceholderText('example.com'), { target: { value: 'new.test' } });
    fireEvent.submit(screen.getByRole('form', { name: 'Add domain' }));
    await Promise.resolve();
    expect(saveDomain).toHaveBeenCalledWith(expect.objectContaining({ name: 'new.test' }));
    expect(api.listDomains).toHaveBeenCalledTimes(2); // mount + after save
  });

  it('deletes a domain', async () => {
    const deleteDomain = vi.fn(async () => undefined);
    renderWithAdmin(() => <Domains />, mockAdminApi({ listDomains: vi.fn(async () => [D]), deleteDomain }));
    fireEvent.click(await screen.findByRole('button', { name: 'Delete example.com' }));
    await Promise.resolve();
    expect(deleteDomain).toHaveBeenCalledWith('example.com');
  });
});
