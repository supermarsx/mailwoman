// t32-h10: an admin address that names no section the admin can open
// (`/admin#/bogus`, or a section outside their role) goes to the dashboard,
// replacing the history entry, with a notice. The main column is never empty.

import { describe, it, expect, beforeEach } from 'vitest';
import { render, screen, fireEvent, within } from '@solidjs/testing-library';
import { AdminScreen } from './index.tsx';
import { mockAdminApi } from './testkit.tsx';
import { holding, offlineShell } from './shell/testkit.tsx';

const findRegion = (name: string): Promise<HTMLElement> => screen.findByRole('region', { name }, { timeout: 5000 });

beforeEach(() => {
  history.replaceState(null, '', '/admin');
});

describe('admin console: unknown sections', () => {
  it('/admin#/bogus lands on the dashboard with a notice, replacing the entry', async () => {
    history.replaceState(null, '', '/admin#/bogus');
    const before = history.length;
    render(() => <AdminScreen api={mockAdminApi()} {...offlineShell()} />);
    expect(await findRegion('Dashboard')).toBeInTheDocument();
    expect(`${location.pathname}${location.hash}`).toBe('/admin');
    expect(history.length).toBe(before);
    const notice = await screen.findByTestId('admin-unknown-section');
    expect(notice).toHaveAttribute('role', 'status');
    expect(notice).toHaveTextContent('does not exist');
    expect(notice).toHaveTextContent('Showing the dashboard');
  });

  it('a section outside the role lands on the dashboard the same way', async () => {
    history.replaceState(null, '', '/admin#/plugins');
    render(() => (
      <AdminScreen api={mockAdminApi()} {...offlineShell()} loadMe={holding(['dashboard.read', 'users.read'])} />
    ));
    expect(await findRegion('Dashboard')).toBeInTheDocument();
    expect(location.hash).toBe('');
    expect(await screen.findByTestId('admin-unknown-section')).toBeInTheDocument();
  });

  it('the notice goes away once the admin opens a section', async () => {
    history.replaceState(null, '', '/admin#/nope');
    render(() => <AdminScreen api={mockAdminApi()} {...offlineShell()} />);
    await screen.findByTestId('admin-unknown-section');
    const nav = screen.getByRole('navigation', { name: 'Admin sections' });
    fireEvent.click(within(nav).getByRole('link', { name: 'Users' }));
    expect(await findRegion('Users')).toBeInTheDocument();
    expect(screen.queryByTestId('admin-unknown-section')).toBeNull();
  });

  it('a known section shows no notice', async () => {
    history.replaceState(null, '', '/admin#/users');
    render(() => <AdminScreen api={mockAdminApi()} {...offlineShell()} />);
    expect(await findRegion('Users')).toBeInTheDocument();
    expect(screen.queryByTestId('admin-unknown-section')).toBeNull();
  });
});
