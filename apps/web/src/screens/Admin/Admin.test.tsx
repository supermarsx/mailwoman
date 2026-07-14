import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent, screen } from '@solidjs/testing-library';
import { AdminScreen } from './index.tsx';
import { mockAdminApi } from './testkit.tsx';

describe('AdminScreen (gate + nav)', () => {
  it('renders the sign-in gate when there is no admin session', async () => {
    const api = mockAdminApi({ session: vi.fn(async () => null) });
    render(() => <AdminScreen api={api} />);
    expect(await screen.findByRole('form', { name: 'Admin sign in' })).toBeInTheDocument();
  });

  it('renders the panel with all §19 sections when a session exists', async () => {
    render(() => <AdminScreen api={mockAdminApi()} />);
    // Default section (Domains) is shown; every section nav entry is present.
    expect(await screen.findByRole('button', { name: 'Domains' })).toBeInTheDocument();
    for (const label of ['Users', 'Security policy', 'Integrations', 'Observability', 'Appearance']) {
      expect(screen.getByRole('button', { name: label })).toBeInTheDocument();
    }
  });

  it('switching the nav changes the visible section', async () => {
    render(() => <AdminScreen api={mockAdminApi()} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Observability' }));
    expect(await screen.findByRole('region', { name: 'Observability' })).toBeInTheDocument();
  });

  it('the section nav is keyboard operable via arrow keys (roving tabindex)', async () => {
    render(() => <AdminScreen api={mockAdminApi()} />);
    const domains = await screen.findByRole('button', { name: 'Domains' });
    const nav = domains.closest('nav');
    expect(nav).not.toBeNull();
    domains.focus();
    fireEvent.keyDown(nav!, { key: 'ArrowDown' });
    expect(document.activeElement).toBe(screen.getByRole('button', { name: 'Users' }));
  });

  it('signs in from the gate', async () => {
    let signedIn = false;
    const api = mockAdminApi({
      session: vi.fn(async () => (signedIn ? { username: 'root' } : null)),
      login: vi.fn(async () => {
        signedIn = true;
        return { username: 'root' };
      }),
    });
    render(() => <AdminScreen api={api} />);
    const form = await screen.findByRole('form', { name: 'Admin sign in' });
    fireEvent.submit(form);
    await Promise.resolve();
    expect(api.login).toHaveBeenCalled();
  });
});
