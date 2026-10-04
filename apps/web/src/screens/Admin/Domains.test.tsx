import { describe, it, expect, vi } from 'vitest';
import { fireEvent, screen } from '@solidjs/testing-library';
import { Domains } from './Domains.tsx';
import { mockAdminApi, renderWithAdmin } from './testkit.tsx';
import type { Domain } from '../../state/slices/admin.ts';

// `GET /admin/domains` sends `[{ "name": … }]` — `DomainDto` in
// crates/mw-server/src/admin.rs has the one field.
const D: Domain = { name: 'example.com' };

describe('Admin › Domains', () => {
  it('lists domains from the api', async () => {
    renderWithAdmin(() => <Domains />, mockAdminApi({ listDomains: vi.fn(async () => [D]) }));
    expect(await screen.findByText('example.com')).toBeInTheDocument();
  });

  it('registers a domain by name and reloads', async () => {
    const saveDomain = vi.fn(async () => undefined);
    const { api } = renderWithAdmin(() => <Domains />, mockAdminApi({ saveDomain }));
    fireEvent.input(screen.getByPlaceholderText('example.com'), { target: { value: ' new.test ' } });
    fireEvent.submit(screen.getByRole('form', { name: 'Add domain' }));
    await Promise.resolve();
    // The name, trimmed, and nothing else: `save_domain` takes it from the path.
    expect(saveDomain).toHaveBeenCalledTimes(1);
    expect(saveDomain).toHaveBeenCalledWith('new.test');
    expect(api.listDomains).toHaveBeenCalledTimes(2); // mount + after save
  });

  it('does not offer the upstream, allowlist or blocklist fields', async () => {
    const { container } = renderWithAdmin(() => <Domains />, mockAdminApi({ listDomains: vi.fn(async () => [D]) }));
    await screen.findByText('example.com');
    for (const label of ['Upstream (JSON)', 'Allowlist', 'Blocklist']) {
      expect(screen.queryByText(label)).toBeNull();
    }
    expect(container.querySelectorAll('textarea')).toHaveLength(0);
    // One text input: the domain name.
    expect(container.querySelectorAll('form input')).toHaveLength(1);
    expect(screen.queryByText(/allow \/ .*block/)).toBeNull();
  });

  it('deletes a domain', async () => {
    const deleteDomain = vi.fn(async () => undefined);
    renderWithAdmin(() => <Domains />, mockAdminApi({ listDomains: vi.fn(async () => [D]), deleteDomain }));
    fireEvent.click(await screen.findByRole('button', { name: 'Delete example.com' }));
    await Promise.resolve();
    expect(deleteDomain).toHaveBeenCalledWith('example.com');
  });
});
