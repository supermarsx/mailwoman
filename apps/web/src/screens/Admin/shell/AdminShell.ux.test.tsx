// The shell parts of the UX seam: appearance control in the header, field hits in
// the palette (capability-filtered), and `?field=` focusing the field (UX10, DC5).
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, fireEvent, waitFor, within } from '@solidjs/testing-library';
import { AdminScreen } from '../index.tsx';
import { mockAdminApi } from '../testkit.tsx';
import { holding, offlineShell } from './testkit.tsx';
import type { SectionDef } from './sections.ts';
import { fieldAnchorAttrs, registerFieldAnchors } from './fieldAnchors.ts';
import { ADMIN_APPEARANCE_KEY } from './appearanceStore.ts';

function Demo() {
  return (
    <section aria-labelledby="demo-title">
      <h2 id="demo-title">Demo</h2>
      <div {...fieldAnchorAttrs('demo.port')}>
        <label for="demo-port">Relay port</label>
        <input id="demo-port" />
      </div>
    </section>
  );
}

const SECTIONS: SectionDef[] = [
  { id: 'dashboard', group: 'overview', icon: 'dashboard', label: () => 'Dashboard', capability: 'dashboard.read', component: () => <p>Home</p> },
  { id: 'demo', group: 'configuration', icon: 'settings', label: () => 'Demo', capability: 'settings.read', component: Demo },
  { id: 'users', group: 'people', icon: 'users', label: () => 'Users', capability: 'users.read', component: () => <p>Users</p> },
];

let removers: (() => void)[] = [];
beforeEach(() => {
  history.replaceState(null, '', '/admin');
  localStorage.clear();
  removers = [
    registerFieldAnchors({ section: 'demo', anchors: [{ id: 'demo.port', labelId: 'admin-ux-stream-filter', keywords: ['MW_DEMO_PORT'] }] }),
    registerFieldAnchors({ section: 'users', anchors: [{ id: 'users.quota', labelId: 'admin-ux-stream-pause' }] }),
    registerFieldAnchors({ section: 'demo', anchors: [{ id: 'demo.secret', labelId: 'admin-ux-stream-clear', capability: 'settings.write' }] }),
  ];
});
afterEach(() => {
  for (const r of removers) r();
  vi.unstubAllGlobals();
});

const openPalette = async (): Promise<HTMLElement> => {
  fireEvent.keyDown(document, { key: 'k', ctrlKey: true });
  return screen.findByRole('combobox');
};

describe('admin shell: UX seam', () => {
  it('keeps appearance controls out of both headers', async () => {
    render(() => <AdminScreen api={mockAdminApi()} {...offlineShell()} sections={SECTIONS} />);
    await screen.findByText('Home');
    expect(screen.queryByRole('button', { name: 'Appearance' })).toBeNull();
    expect(document.querySelector('[data-testid="admin-appearance-button"]')).toBeNull();
  });

  it('the palette lists fields of sections the admin may open, matched by label or keyword; Enter opens ?field=', async () => {
    render(() => <AdminScreen api={mockAdminApi()} loadMe={holding(['dashboard.read', 'settings.read'])} loadDashboard={offlineShell().loadDashboard} search={offlineShell().search} sections={SECTIONS} />);
    await screen.findByText('Home');
    const input = await openPalette();
    fireEvent.input(input, { target: { value: 'MW_DEMO' } });
    const hit = await screen.findByRole('option', { name: /Filter/ });
    expect(within(hit).getByText('Field')).toBeInTheDocument();
    expect(hit).toHaveTextContent('Demo');
    // users.read is not held (section hidden) and settings.write is not held (field capability).
    fireEvent.input(input, { target: { value: 'users.quota' } });
    await waitFor(() => expect(screen.queryByRole('option', { name: /Pause/ })).toBeNull());
    fireEvent.input(input, { target: { value: 'demo.secret' } });
    await waitFor(() => expect(screen.queryByRole('option', { name: /Clear/ })).toBeNull());
    fireEvent.input(input, { target: { value: 'demo.port' } });
    await screen.findByRole('option', { name: /Filter/ });
    fireEvent.keyDown(input, { key: 'Enter' });
    await waitFor(() => expect(location.hash).toBe('#/demo?field=demo.port'));
    await waitFor(() => expect(document.activeElement).toBe(screen.getByLabelText('Relay port')));
    expect(screen.getByLabelText('Relay port').parentElement).toHaveAttribute('data-field-highlight');
  });

  it('a link with ?field= opens the section and focuses the field once it renders', async () => {
    history.replaceState(null, '', '/admin#/demo?field=demo.port');
    render(() => <AdminScreen api={mockAdminApi()} {...offlineShell()} sections={SECTIONS} />);
    await waitFor(() => expect(document.activeElement).toBe(screen.getByLabelText('Relay port')));
  });

  it('the dashboard and every existing section keep working without inheriting another browser user webmail appearance', async () => {
    localStorage.setItem('mw.theme.prefs', JSON.stringify({ mode: 'fixed', theme: 'paper', chosen: true }));
    render(() => <AdminScreen api={mockAdminApi()} {...offlineShell()} sections={SECTIONS} />);
    await screen.findByText('Home');
    expect(document.documentElement).toHaveAttribute('data-theme', 'dark');
    expect(localStorage.getItem(ADMIN_APPEARANCE_KEY)).toBeNull();
  });
});
