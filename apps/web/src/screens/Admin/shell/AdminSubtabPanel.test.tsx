import { beforeEach, expect, it, vi } from 'vitest';
import { fireEvent, render, screen, waitFor, within } from '@solidjs/testing-library';
import { createResource, lazy, onCleanup, Show } from 'solid-js';
import { AdminScreen } from '../index.tsx';
import { mockAdminApi } from '../testkit.tsx';
import { offlineShell } from './testkit.tsx';
import { useShell } from './context.ts';
import { sectionPath } from './router.ts';
import { SectionTabs } from './SectionTabs.tsx';
import { AdminSubtabPanel } from './AdminSubtabPanel.tsx';
import type { SectionDef } from './sections.ts';

beforeEach(() => history.replaceState(null, '', '/admin#/domains'));

it.each(['resource', 'lazy chunk'] as const)('keeps admin navigation and the section mounted while a subtab waits for its %s', async kind => {
  let release!: () => void;
  const pending = new Promise<void>(resolve => { release = resolve; });
  const mounted = vi.fn();
  const disposed = vi.fn();
  const api = mockAdminApi();
  function ResourcePanel() {
    const [value] = createResource(async () => { await pending; return 'Details ready'; });
    return <p>{value()}</p>;
  }
  const Detail = kind === 'resource' ? ResourcePanel : lazy(async () => {
    await pending;
    return { default: () => <p>Details ready</p> };
  });
  function Section() {
    mounted();
    onCleanup(disposed);
    const shell = useShell();
    const page = () => shell.router.route().rest[0] ?? 'overview';
    return <>
      <input aria-label="Retained section draft" />
      <SectionTabs label="Test subtabs" selected={page()} navigate={path => void shell.router.navigate(path)} items={[
        { id: 'overview', label: 'Overview', path: sectionPath('domains') },
        { id: 'detail', label: 'Detail', path: sectionPath('domains', ['detail']) },
      ]} />
      <AdminSubtabPanel><Show when={page() === 'detail'} fallback={<p>Overview ready</p>}><Detail /></Show></AdminSubtabPanel>
    </>;
  }
  const sections: SectionDef[] = [{ id: 'domains', group: 'configuration', icon: 'domains', label: () => 'Domains', component: Section }];
  render(() => <AdminScreen api={api} {...offlineShell()} sections={sections} />);
  const tabs = await screen.findByRole('navigation', { name: 'Test subtabs' });
  const draft = screen.getByRole('textbox', { name: 'Retained section draft' });
  fireEvent.input(draft, { target: { value: 'Keep this draft' } });
  expect(fireEvent.click(within(tabs).getByRole('link', { name: 'Detail' }))).toBe(false);
  await waitFor(() => expect(location.hash).toBe('#/domains/detail'));
  expect(screen.getByRole('navigation', { name: 'Test subtabs' })).toBe(tabs);
  expect(within(tabs).getByRole('link', { name: 'Detail' })).toHaveAttribute('aria-current', 'page');
  expect(draft).toBeInTheDocument();
  expect(draft).toHaveValue('Keep this draft');
  expect(screen.queryByTestId('admin-page-skeleton')).toBeNull();
  expect(mounted).toHaveBeenCalledTimes(1);
  expect(disposed).not.toHaveBeenCalled();
  release();
  await screen.findByText('Details ready');
  history.replaceState(null, '', '/admin#/domains');
  window.dispatchEvent(new PopStateEvent('popstate'));
  await screen.findByText('Overview ready');
  expect(screen.getByRole('navigation', { name: 'Test subtabs' })).toBe(tabs);
  expect(draft).toHaveValue('Keep this draft');
  expect(mounted).toHaveBeenCalledTimes(1);
  expect(api.session).toHaveBeenCalledTimes(1);
});
