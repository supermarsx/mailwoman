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

  it('renders the panel with its sections when a session exists', async () => {
    render(() => <AdminScreen api={mockAdminApi()} />);
    // Default section (Domains) is shown; every section nav entry is present.
    expect(await screen.findByRole('button', { name: 'Domains' })).toBeInTheDocument();
    for (const label of ['Users', 'Integrations', 'Observability', 'Require two-factor', 'Egress', 'UI plugins']) {
      expect(screen.getByRole('button', { name: label })).toBeInTheDocument();
    }
  });

  // Both screens held only controls that saved a value nothing applied; with the
  // controls removed there was nothing left to navigate to (26.20, t28-e8).
  it('has no Security policy or Appearance section', async () => {
    const api = mockAdminApi();
    render(() => <AdminScreen api={api} />);
    await screen.findByRole('button', { name: 'Domains' });
    expect(screen.queryByRole('button', { name: 'Security policy' })).toBeNull();
    expect(screen.queryByRole('button', { name: 'Appearance' })).toBeNull();
    // Visiting every section reads neither stored record.
    for (const b of screen.getAllByRole('button')) {
      if (b.closest('nav') && b.textContent !== 'Sign out') fireEvent.click(b);
    }
    await Promise.resolve();
    expect(api.getSecurityPolicy).not.toHaveBeenCalled();
    expect(api.getAppearance).not.toHaveBeenCalled();
    expect(api.getObservability).not.toHaveBeenCalled();
  });

  it('switching the nav changes the visible section', async () => {
    render(() => <AdminScreen api={mockAdminApi()} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Observability' }));
    expect(await screen.findByRole('region', { name: 'Observability' })).toBeInTheDocument();
  });

  it('the UI plugins entry shows that screen in place of the section that was open', async () => {
    render(() => <AdminScreen api={mockAdminApi()} />);
    fireEvent.click(await screen.findByRole('button', { name: 'UI plugins' }));
    expect(await screen.findByRole('region', { name: 'UI plugins' })).toBeInTheDocument();
    expect(screen.queryByRole('region', { name: 'Domains' })).toBeNull();
    // Leaving it brings the chosen section back and takes the screen away.
    fireEvent.click(screen.getByRole('button', { name: 'Observability' }));
    expect(await screen.findByRole('region', { name: 'Observability' })).toBeInTheDocument();
    expect(screen.queryByRole('region', { name: 'UI plugins' })).toBeNull();
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
