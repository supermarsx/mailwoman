import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, fireEvent, screen, waitFor, within } from '@solidjs/testing-library';
import { AdminScreen } from './index.tsx';
import { mockAdminApi } from './testkit.tsx';
import { holding, offlineShell } from './shell/testkit.tsx';

const nav = (): HTMLElement => screen.getByRole('navigation', { name: 'Admin sections' });
// Each section is its own lazy chunk; under the full parallel suite the import can
// take longer than the default 1 s lookup.
const findRegion = (name: string): Promise<HTMLElement> => screen.findByRole('region', { name }, { timeout: 5000 });

beforeEach(() => {
  history.replaceState(null, '', '/admin');
  document.documentElement.removeAttribute('dir');
});

describe('AdminScreen (gate + shell)', () => {
  it('renders the sign-in gate when there is no admin session', async () => {
    const api = mockAdminApi({ session: vi.fn(async () => null) });
    render(() => <AdminScreen api={api} {...offlineShell()} />);
    expect(await screen.findByRole('form', { name: 'Admin sign in' })).toBeInTheDocument();
  });

  it('lands on the dashboard and lists every section in its group', async () => {
    render(() => <AdminScreen api={mockAdminApi()} {...offlineShell()} />);
    expect(await findRegion('Dashboard')).toBeInTheDocument();
    for (const [group, labels] of [
      ['Overview', ['Dashboard']],
      ['People', ['Users', 'Domains', 'Require two-factor']],
      ['Security', ['Single sign-on']],
      ['Configuration', ['AI assistant', 'Integrations', 'Plugins', 'UI plugins', 'Egress']],
      ['Operations', ['Observability', 'Maintenance', 'Server metadata']],
    ] as const) {
      const g = within(nav()).getByRole('group', { name: group });
      for (const label of labels) expect(within(g).getByRole('link', { name: label })).toBeInTheDocument();
    }
    expect(within(nav()).getByRole('link', { name: 'Dashboard' })).toHaveAttribute('aria-current', 'page');
  });

  it('every sidebar item carries one decorative icon and a text label', async () => {
    render(() => <AdminScreen api={mockAdminApi()} {...offlineShell()} />);
    await findRegion('Dashboard');
    const links = within(nav()).getAllByRole('link');
    expect(links.length).toBe(15);
    const icons = new Set<string>();
    for (const a of links) {
      const svgs = a.querySelectorAll('svg');
      expect(svgs.length).toBe(1);
      expect(svgs[0]).toHaveAttribute('aria-hidden', 'true');
      icons.add(svgs[0]!.getAttribute('data-icon') ?? '');
      expect(a.textContent?.trim().length).toBeGreaterThan(0);
    }
    // One distinct icon per section.
    expect(icons.size).toBe(links.length);
  });

  it('has no Appearance section; the Security policy section reads through its own client, not the shared one', async () => {
    const api = mockAdminApi();
    render(() => <AdminScreen api={api} {...offlineShell()} />);
    await findRegion('Dashboard');
    expect(within(nav()).getByRole('link', { name: 'Security policy' })).toBeInTheDocument();
    expect(screen.queryByRole('link', { name: 'Appearance' })).toBeNull();
    for (const a of within(nav()).getAllByRole('link')) fireEvent.click(a);
    await Promise.resolve();
    expect(api.getSecurityPolicy).not.toHaveBeenCalled();
    expect(api.getAppearance).not.toHaveBeenCalled();
    expect(api.getObservability).not.toHaveBeenCalled();
  });

  it('a nav click opens the section, puts it in the URL and marks it current', async () => {
    render(() => <AdminScreen api={mockAdminApi()} {...offlineShell()} />);
    fireEvent.click(await within(await screen.findByRole('navigation', { name: 'Admin sections' })).findByRole('link', { name: 'Observability' }));
    expect(await findRegion('Observability')).toBeInTheDocument();
    expect(location.pathname + location.hash).toBe('/admin#/observability');
    expect(within(nav()).getByRole('link', { name: 'Observability' })).toHaveAttribute('aria-current', 'page');
    expect(within(nav()).getByRole('link', { name: 'Dashboard' })).not.toHaveAttribute('aria-current');
  });

  it('a section path in the URL opens that section directly (deep link)', async () => {
    history.replaceState(null, '', '/admin#/ui-plugins');
    render(() => <AdminScreen api={mockAdminApi()} {...offlineShell()} />);
    expect(await findRegion('UI plugins')).toBeInTheDocument();
    expect(screen.queryByRole('region', { name: 'Dashboard' })).toBeNull();
  });

  it('Back returns to the previous section', async () => {
    render(() => <AdminScreen api={mockAdminApi()} {...offlineShell()} />);
    await findRegion('Dashboard');
    fireEvent.click(within(nav()).getByRole('link', { name: 'Observability' }));
    await findRegion('Observability');
    history.replaceState(null, '', '/admin');
    window.dispatchEvent(new PopStateEvent('popstate'));
    expect(await findRegion('Dashboard')).toBeInTheDocument();
  });

  it('nested sections get breadcrumbs back to the dashboard and the section', async () => {
    history.replaceState(null, '', '/admin#/domains/example.org');
    render(() => <AdminScreen api={mockAdminApi()} {...offlineShell()} />);
    const crumbs = await screen.findByRole('navigation', { name: 'Breadcrumb' });
    expect(within(crumbs).getByRole('link', { name: 'Dashboard' })).toHaveAttribute('href', '/admin');
    expect(within(crumbs).getByRole('link', { name: 'Domains' })).toHaveAttribute('href', '/admin#/domains');
    expect(within(crumbs).getByText('example.org')).toHaveAttribute('aria-current', 'page');
  });

  it('an unknown section says so and offers the dashboard', async () => {
    history.replaceState(null, '', '/admin#/nope');
    render(() => <AdminScreen api={mockAdminApi()} {...offlineShell()} />);
    expect(await screen.findByTestId('admin-unknown-section')).toHaveTextContent('does not exist');
  });

  it('the section nav is keyboard operable via arrow keys (roving tabindex)', async () => {
    render(() => <AdminScreen api={mockAdminApi()} {...offlineShell()} />);
    await findRegion('Dashboard');
    const users = within(nav()).getByRole('link', { name: 'Users' });
    users.focus();
    fireEvent.keyDown(nav(), { key: 'ArrowDown' });
    expect(document.activeElement).toBe(within(nav()).getByRole('link', { name: 'Domains' }));
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
    render(() => <AdminScreen api={api} {...offlineShell()} />);
    const form = await screen.findByRole('form', { name: 'Admin sign in' });
    fireEvent.submit(form);
    await waitFor(() => expect(api.login).toHaveBeenCalled());
    expect(await screen.findByRole('navigation', { name: 'Admin sections' })).toBeInTheDocument();
  });

  it('sign out calls the server and returns to the gate', async () => {
    const api = mockAdminApi();
    render(() => <AdminScreen api={api} {...offlineShell()} />);
    await findRegion('Dashboard');
    fireEvent.click(screen.getAllByRole('button', { name: 'Sign out' })[0]!);
    await waitFor(() => expect(api.logout).toHaveBeenCalled());
    expect(await screen.findByRole('form', { name: 'Admin sign in' })).toBeInTheDocument();
  });
});

describe('capability-aware navigation (GET /admin/me)', () => {
  it('a 404 from /admin/me (no RBAC yet) shows every section', async () => {
    render(() => <AdminScreen api={mockAdminApi()} {...offlineShell()} />);
    await findRegion('Dashboard');
    expect(within(nav()).getAllByRole('link')).toHaveLength(15);
  });

  it('hides the sections the admin lacks and refuses their URL', async () => {
    history.replaceState(null, '', '/admin#/plugins');
    render(() => (
      <AdminScreen api={mockAdminApi()} {...offlineShell()} loadMe={holding(['dashboard.read', 'users.read'])} />
    ));
    expect(await screen.findByTestId('admin-unknown-section')).toBeInTheDocument();
    const labels = within(nav()).getAllByRole('link').map((a) => a.textContent?.trim());
    expect(labels).toEqual(['Dashboard', 'Users']);
    // Empty groups are not drawn.
    expect(within(nav()).queryByRole('group', { name: 'Configuration' })).toBeNull();
  });

  it('does not draw the navigation before the grants have loaded', async () => {
    let release!: () => void;
    const gate = new Promise<void>((r) => (release = r));
    render(() => (
      <AdminScreen
        api={mockAdminApi()}
        {...offlineShell()}
        loadMe={async () => {
          await gate;
          return { kind: 'listed', me: { username: 'x', displayName: null, roles: [], capabilities: ['users.read'], domainScope: null, totpEnrolled: false } };
        }}
      />
    ));
    await waitFor(() => expect(screen.getByText('Loading…')).toBeInTheDocument());
    expect(screen.queryByRole('link', { name: 'Plugins' })).toBeNull();
    release();
    expect(await screen.findByRole('link', { name: 'Users' })).toBeInTheDocument();
    expect(screen.queryByRole('link', { name: 'Plugins' })).toBeNull();
  });
});

describe('phone drawer and command palette', () => {
  it('the menu button opens the navigation as a modal drawer; Escape closes it and focus returns', async () => {
    render(() => <AdminScreen api={mockAdminApi()} {...offlineShell()} />);
    await findRegion('Dashboard');
    const menu = screen.getByRole('button', { name: 'Menu' });
    expect(menu).toHaveAttribute('aria-expanded', 'false');
    menu.focus();
    fireEvent.click(menu);
    const drawer = await screen.findByRole('dialog', { name: 'Menu' });
    expect(drawer).toHaveAttribute('aria-modal', 'true');
    expect(menu).toHaveAttribute('aria-expanded', 'true');
    expect(within(drawer).getByRole('link', { name: 'Users' })).toBeInTheDocument();
    await waitFor(() => expect(drawer.contains(document.activeElement)).toBe(true));
    fireEvent.keyDown(drawer, { key: 'Escape' });
    await waitFor(() => expect(screen.queryByRole('dialog', { name: 'Menu' })).toBeNull());
    expect(document.activeElement).toBe(menu);
  });

  it('choosing a section in the drawer closes it and opens the section', async () => {
    render(() => <AdminScreen api={mockAdminApi()} {...offlineShell()} />);
    await findRegion('Dashboard');
    fireEvent.click(screen.getByRole('button', { name: 'Menu' }));
    const drawer = await screen.findByRole('dialog', { name: 'Menu' });
    fireEvent.click(within(drawer).getByRole('link', { name: 'Observability' }));
    expect(await findRegion('Observability')).toBeInTheDocument();
    expect(screen.queryByRole('dialog', { name: 'Menu' })).toBeNull();
  });

  it('Ctrl+K opens the palette; typing filters sections; Enter opens the active one', async () => {
    render(() => <AdminScreen api={mockAdminApi()} {...offlineShell()} />);
    await findRegion('Dashboard');
    fireEvent.keyDown(document, { key: 'k', ctrlKey: true });
    const dialog = await screen.findByRole('dialog', { name: 'Search the admin console' });
    const input = within(dialog).getByRole('combobox');
    fireEvent.input(input, { target: { value: 'obs' } });
    const options = within(dialog).getAllByRole('option');
    expect(options.map((o) => o.textContent)).toEqual([expect.stringContaining('Observability')]);
    fireEvent.keyDown(input, { key: 'Enter' });
    expect(await findRegion('Observability')).toBeInTheDocument();
    expect(screen.queryByRole('dialog', { name: 'Search the admin console' })).toBeNull();
  });
});
